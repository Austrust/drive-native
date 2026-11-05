# Drive Server 公开包权限静态证据

- [结论与实现边界](REPORT.md)：提交前 ACL 拒绝、同码的 profile 路径过滤、逐节点能力查询与父目录语义。
- [复现命令](COMMANDS.md)：公开下载、离线提取与静态反汇编；不启动软件。
- [公开下载来源与摘要](download-manifest.json)、[SPK 校验与提取结果](spk-extraction.json)、[所选软件成员摘要](selected-members-manifest.json)。
- [虚表解析](vtables.txt)、[四个错误码立即数扫描](permission-code-immediates.json)。
- [文件校验清单](SHA256SUMS.txt)：冻结证据文件；大型软件包、二进制、Python 依赖保留本地，不属于提交材料。

本文包是此前有限本地搜索之后新增的公开来源证据；没有改写此前未找到 `syncd` 的历史记录。`syncd` 是 Drive Server 的服务程序，本次只读取其文件字节，未安装或运行。
