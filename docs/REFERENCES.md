# 参考线与证据级别

所有新增业务逻辑独立实现；本表不授权复制供应商代码。

| 参考线 | 已定位来源 | 可采用的约束与边界 |
|---|---|---|
| 群晖 Windows | 本项目静态记录 WINDOWS_PARITY_CONTRACT.md；WinCFWrapper.dll SHA-256 c25893b59b9c5c86ab7de856fe3078e7567a4ea4b24a2a6a93225566a134d22d；SetInSync RVA 0x23d10、PinToLocal 0x23dc0 | 固定、完整缓存、服务器同步分开；原生平台接口不可移植为Linux ABI。静态证据不证明完整运行时对等 |
| 群晖官方 Linux | 冻结的 official-drive-ondemand 实验与 ELF/DWARF 证据 | 已有元数据、按需读取、后台更新和离线实验保留。每次启用协议对照前必须重新核对具体构建/ELF哈希/调用链；当前不把内部接口视为稳定SDK |
| SeaDrive Linux | haiwen/seadrive-fuse，尚未开展本目标要求的版本化调用链与锁持有分析 | 待P1前绑定提交和文件/函数并记录独立设计差异。不得复制源码、头文件或链接libsearpc；当前未引入其代码或依赖 |

后续每项借鉴必须补同类问题、来源版本/函数、证据级别、独立实现差异及本项目压测收益，不能仅凭架构图断言性能。
