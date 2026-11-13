# 按任务正则排除（issue #50 阶段 1–4；issue #56B 双模式）

任务 profile 里有可选字段 `exclude_patterns`（字符串数组），用于在挂载范围内排除一组路径及其子树；`exclude_mode` 决定命中路径在**本机视图**中的呈现方式：

- `local_only`（**默认**，issue #56B）：命中路径在本机挂载中**可见、可读写**（readdir/lookup/getattr/open/opendir 正常，`GetDirectory`/`GetStates` 正常列出），但**不参与同步**：不上传、不下载、不产生远端删除，远端已存在的同名内容保留。适合 `.git`、构建缓存、本地工具目录。
- `hidden`（issue #50 原行为，显式选择）：命中路径在本机视图**完全隐藏**（readdir 不显示，lookup/getattr/open 返回 `ENOENT`），已导入节点被本地标记隐藏并回收缓存正文。

两种模式都接入：初始化导入剪枝（匹配节点不进入候选树、不查询目录、不下载）、远端事件与恢复快照过滤、上传/下载/缓存调度防御。**默认行为变化**：旧 profile 没有 `exclude_mode` 字段时按 `local_only` 处理；如果依赖 #50 的隐藏语义，需要显式写 `"exclude_mode": "hidden"`。

## 配置写法

`profile.json`（通常位于 `$XDG_CONFIG_HOME/drive-native/profile.json`，多挂载在 `profiles/<instance>.json`）示例：

```json
{
  "schema": 1,
  "task_id": "…",
  "exclude_mode": "local_only",
  "exclude_patterns": ["^\\.", "(^|/)\\.tmp-", "(^|/)node_modules/", "\\.iso$"]
}
```

- `exclude_mode` 可省略，缺省 `local_only`；只接受 `"local_only"` / `"hidden"`，其它值在加载 profile 时报错并带原始值（不静默回退）。
- `exclude_patterns` 可省略；省略或空数组表示不排除任何路径，与旧版行为一致，读路径保持零成本快路径。
- 每个元素是一个 Rust `regex` crate 正则；**大小写敏感**，需要忽略大小写时在规则内写 `(?i)`。
- 校验在 worker 启动与 GUI 保存时进行：非法正则会报错并带上原始 pattern 与编译器给出的原因，profile 不会被接受。

## 匹配语义

- 匹配对象是相对任务 scope 根（profile 的 `scope.path`）的 POSIX 路径，例如 `Pinned`、`Sub/C.txt`；没有前导 `/`，也不含 `.`/`..` 组件。
- 文件按其完整相对路径匹配；目录按「目录路径」和「目录路径 + `/`」两种形式匹配，因此 `(^|/)node_modules/` 这类写法可命中目录本身。
- 规则命中某个目录或文件后，其整棵子树都被排除；祖先目录命中也同样排除后代（规则无需写成 `^Pinned/.*`）。
- scope 根本身永远不被排除。
- 空规则列表走快速路径，不产生正则匹配开销；正则只在加载 profile 时编译一次并复用。

示例：

| 规则 | 命中的相对路径 |
| --- | --- |
| `^\.` | `.snapshot`、`.cache/a` |
| `(^|/)node_modules/` | `node_modules`、`app/node_modules/x.js` |
| `\.iso$` | `images/install.iso` |
| `^Pinned$` | `Pinned` 及其全部后代 |

## 索引构建时机

- 挂载前在调用方 Store 锁内一次性读取 `nodes ⋈ fs_inodes` 投影；正则匹配在该投影读出后、本模块内执行，不额外获取锁（调用方传锁边界见 `ExclusionState::refresh_shared`）。FUSE 读路径只按 inode/目录项查预计算索引，不取 store 锁。
- 索引在以下时机重建：挂载启动时；初始化 owner 完成命名空间安装后；断连恢复快照（`recovery snapshot`）安装后；`Pull` 事件应用后仅在 `events_may_affect_index` 判定本批事件可能改变排除集合时重建。正常批次中被排除的事件已在应用前丢弃，因此通常不会触发重建，避免 210k 命名空间每次 Pull 都做约 550 ms 的全量投影（基准见 `crates/drive-core/tests/exclude_index_build_bench.rs`，以 `--ignored` 显式运行）。
- 初始化 preview 阶段（committed root 尚未出现、只能挂载 preview 根）例外：此时排除索引为空，不读取 committed 命名空间；初始化完成或恢复快照安装后由上述 bridge 刷新路径建立索引。
- 因此「挂载时 store 为空、初始化随后才导入」的场景仍然正确：被排除路径在索引刷新后保持不可见；相应的 mounted 回归测试会显式验证这一点。
- 事件过滤在同一批事件内解析父子链：父目录事件被丢弃时，其子节点的 Upsert/Delete 也一并丢弃；批内顺序不影响结果。

## 写路径、调度防御与规则变更清理（阶段 4）

- **hidden 模式**的写路径：`create`/`mkdir` 的目标名、`rename` 的源与目标、`unlink`/`rmdir`、`write`/`setattr`、`open` 命中排除路径或排除子树时返回 `ENOENT`，且不写入本地节点、不产生上传或命名空间 operation。判定使用索引里父目录的 scope 相对路径加目标名称，因此尚不存在的新名字也能被规则命中。
- **local_only 模式**的写路径：上述操作全部放行并写入本地视图；写入/创建完成后立即丢弃该节点的同步意图（upload operation、pending namespace operation、local creation/unlink intent、namespace overlay），节点本身、inode 映射、draft 与缓存正文保留，因此本地工具可继续使用。`local_only` 上重命名覆盖已有目标只影响本地副本，不产生远端删除。读取未缓存的远端正文不会触发下载（按 `ENOENT`/离线语义处理）。
- 规则变更清理在启动挂载时执行：`hidden` 模式把排除的已导入节点本地标记隐藏并回收正文；`local_only` 模式只清理同步意图（并先恢复被旧 hidden 规则隐藏的行），节点保持可见。**绝不**写入远端删除：测试断言 remove 队列为空、没有 `remove` namespace_ops，冲突节点也保留本地副本不清理。
- 调度入口防御：缓存下载候选（`refresh_cached`/hydrate）、上传 pending、命名空间 rename/remove/replace/mkdir 队列遇到被排除节点时跳过，不发送任何 RPC。
- 规则变更清理在启动挂载时执行：把规则排除的已导入节点在本地标记为隐藏（与远端删除的本地视图语义相同），取消其上传/下载/草稿/本地创建/命名空间 operation，清掉 dirty 与缓存正文引用并让 `collect_garbage` 回收 blob；节点行本身保留，被排除的 id 记录在 `excluded_purged` metadata 中。**绝不**写入远端删除：测试断言 remove 队列为空、没有 `remove` namespace_ops，冲突节点也保留本地副本不清理。
- 恢复快照安装前会先按规则清理被排除节点，避免「快照未包含该节点 → 被当成 recovery tombstone/conflict」；隐藏节点保持隐藏，兄弟节点正常更新。
当前规则重新判定 `excluded_purged` 中的节点行——不再命中任何剩余规则的恢复可见（`Store::restore_unexcluded_nodes`），仍命中的保持隐藏并留在 marker 中；全部规则移除时等价于历史的全量 `restore_excluded_nodes`。**部分撤回规则即可恢复对应路径**，无需先清空全部规则；已回收的正文需按需重新下载。
- 移除规则后重启：`restore_excluded_nodes` 恢复此前隐藏的节点行（已回收正文按需重新下载）。`local_only` 命中的本地内容在移除规则并重启后保留，但不会自动补传；之后的本地修改会重新进入正常同步（即“显式动作”）。
- 规则变更必须停止并重新启动该挂载才能完全生效；清理后 `GetDirectory` 的 `total` 与 `entries` 一致。GUI 目前没有单独的排除规则编辑页（由其他任务跟踪）；保存 profile 时若含 `exclude_patterns`，会先用 Python `re` 做与 worker 等价的合法性校验，非法规则拒绝保存并在窗口内提示 pattern 与错误原因。
- 旧 profile 不含 `exclude_patterns`/`exclude_mode` 时仍可正常读取、保存和挂载（mode 默认 `local_only`）。GUI 目前只编辑规则文本、不提供模式选择（待办）；保存规则只替换 `exclude_patterns` 键，`exclude_mode` 等其它 profile 字段原样保留。
