# 按任务正则排除（issue #50，阶段 1/2）

任务 profile 里新增可选字段 `exclude_patterns`（字符串数组），用于在挂载范围内排除一组路径及其子树。本阶段只接入初始化导入路径：匹配的节点不会进入初始化候选树，也不会发起目录查询或下载；FUSE 视图过滤、上传/下载防御过滤与规则变更后的存量清理属于后续阶段。

## 配置写法

`profile.json`（通常位于 `$XDG_CONFIG_HOME/drive-native/profile.json`，多挂载在 `profiles/<instance>.json`）示例：

```json
{
  "schema": 1,
  "task_id": "…",
  "exclude_patterns": ["^\\.", "(^|/)\\.tmp-", "(^|/)node_modules/", "\\.iso$"]
}
```

- 字段可省略；省略或空数组表示不排除任何路径，与旧版行为一致。
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

## 生效方式与 GUI

- 规则变更需要停止并重新启动该挂载才能完全生效（初始化候选会按新规则重新校验；已有视图的存量清理属于后续阶段）。
- GUI 目前没有单独的排除规则编辑页（后续阶段补充）；保存 profile 时若含 `exclude_patterns`，会先用 Python `re` 做与 worker 等价的合法性校验，非法规则拒绝保存并在窗口内提示 pattern 与错误原因。
- 旧 profile 不含该字段时仍可正常读取、保存和挂载。
