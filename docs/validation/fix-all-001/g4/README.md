# G4 独立验证证据

- [验收结果、计数与原件入口](REPORT.md)
- [规格逐条映射与未实测边界](COVERAGE.md)
- [机器结论](final-result.json)
- [FUSE唯一测试及INDEX=0不适用原因](kernel-unique-tests.json)
- [全部原始证据](evidence/evidence-manifest.json)

harness/ 保存独立验证脚本，provenance/ 保存每轮冻结提交和归档哈希。源码与target不复制；通过指定仓库提交重建。脚本中Linux绝对路径记录本次隔离环境，复跑需新run目录并调整环境路径。原WSL二进制和状态保留，报告内哈希绑定它们。此包不含真实清单文件名。
