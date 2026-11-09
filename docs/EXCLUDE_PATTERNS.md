# 按任务正则排除（issue #50，阶段 1–3）

任务 profile 里新增可选字段 `exclude_patterns`（字符串数组），用于在挂载范围内排除一组路径及其子树。当前阶段已接入：初始化导入剪枝（匹配节点不进入候选树、不查询目录、不下载）、FUSE 读路径（readdir 不显示，lookup/getattr/open/opendir 返回 `ENOENT`，已打开句柄不受影响）、远端事件与恢复快照过滤、`GetDirectory`/`GetStates` 列表过滤。上传/下载/缓存调度防御与规则变更后的存量清理属于阶段 4。

## 配置写法

`profile.json`（通常位于 `$XDG_CONFIG_HOME/drive-native/profile.json`，多挂载在 `profiles/<instance>.json`）示例：

```json
{
  "schema": 1,
  "task_id": "…",
  "exclude_patterns": ["^\\.", "(^|/)\\.tmp-", "(^|/)node_modules/", "\\.iso$"]
}
```

- 字段可省略；省略或空数组表示不排除任何路径，与旧版行为一致，读路径保持零成本快路径。
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

- 挂载前在 store 锁内一次性读取 `nodes ⋈ fs_inodes` 投影，释放锁后再做正则匹配；FUSE 读路径只按 inode/目录项查预计算索引，不取 store 锁。
- 索引在以下时机重建：挂载启动时；初始化 owner 完成命名空间安装后；`Pull` 事件应用后；断连恢复快照（`recovery snapshot`）安装后。
- 初始化 preview 阶段（committed root 尚未出现、只能挂载 preview 根）例外：此时排除索引为空，不读取 committed 命名空间；初始化完成或恢复快照安装后由上述 bridge 刷新路径建立索引。
- 因此「挂载时 store 为空、初始化随后才导入」的场景仍然正确：被排除路径在索引刷新后保持不可见；相应的 mounted 回归测试会显式验证这一点。
- 事件过滤在同一批事件内解析父子链：父目录事件被丢弃时，其子节点的 Upsert/Delete 也一并丢弃；批内顺序不影响结果。

## 生效方式、GUI 与 total 语义

- 规则变更需要停止并重新启动该挂载才能完全生效（初始化候选与读路径索引都会按新规则重建；已有视图的存量清理属于阶段 4）。
- GUI 目前没有单独的排除规则编辑页（阶段 4 补充）；保存 profile 时若含 `exclude_patterns`，会先用 Python `re` 做与 worker 等价的合法性校验，非法规则拒绝保存并在窗口内提示 pattern 与错误原因。
- `GetDirectory` 返回的 `total` 仍是权威子节点计数：初始化导入与事件入口已剪枝后通常一致；规则变更后、阶段 4 清理前，历史遗留的被排除节点可能仍计入 `total`，但不会出现在 `entries` 中。
- 旧 profile 不含该字段时仍可正常读取、保存和挂载。
