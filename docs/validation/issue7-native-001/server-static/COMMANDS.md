# 复现命令与静态范围

以下命令从仓库根目录运行，所有产物只写 `work/issue7-native-evidence/public-server`。Python 解析文件字节；没有调用任何 Synology 可执行文件。运行环境 Python 3.12，Capstone 为本机已有解析库；PyNaCl/msgpack 使用任务目录的预编译 wheel。

```powershell
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/fetch-public-package.py
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/fetch-reference-sources.py
C:/Users/Public/Python3123/python.exe -m pip install --disable-pip-version-check --only-binary=:all: --target work/issue7-native-evidence/public-server/python-libs pynacl msgpack
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/extract-public-spk.py
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/extract-selected-members.py
```

公开下载首次在默认网络沙箱中 DNS 失败，经网络升级审批后成功。仅访问公开 Synology、GitHub raw/API、Python wheel 源，无认证；GitHub commits API 返回 403，参考源码改用公开 main URL，精确字节由 [reference-sources.json](reference-sources.json) 的 SHA 冻结，未声称锁定 commit。

静态反汇编复用仓库 `scripts/analyze_server_elf.py` 的[冻结副本](analyze-server-elf.py)，[来源 SHA](source-tools.json)，读取 PT_DYNAMIC / GNU hash、动态函数区间、RIP 引用及 PLT。命令模板：

```powershell
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/analyze-server-elf.py work/issue7-native-evidence/public-server/syncd.elf --out work/issue7-native-evidence/public-server/symbols --pattern '^$' --python-libs C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/pylibs
C:/Users/Public/Python3123/python.exe work/issue7-native-evidence/public-server/inspect-selected-structure.py --python-libs C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/pylibs
```

各目录 `analysis.json` 保留实际 pattern、输入 SHA、函数地址、完整选定函数大小及引用。可用相同模板替换 binary、out 和 pattern 复现：

| 输出 | 输入 | pattern 位置 |
|---|---|---|
| acl-boundary | syncd.elf | [analysis.json](acl-boundary/analysis.json) |
| dispatch-boundary | syncd.elf | [analysis.json](dispatch-boundary/analysis.json) |
| control-details | syncd.elf | [analysis.json](control-details/analysis.json) |
| flow | syncd.elf | [analysis.json](flow/analysis.json) |
| filter-boundary | syncd.elf | [analysis.json](filter-boundary/analysis.json) |
| core-capability | libsynosyncservercore.so.elf | [analysis.json](core-capability/analysis.json) |
| acl-path | libsynosyncservercore.so.elf | [analysis.json](acl-path/analysis.json) |
| filter-source | libsynosyncservercore.so.elf | [analysis.json](filter-source/analysis.json) |

并非所有输出函数都已作完整语义审计，结论只覆盖 [REPORT.md](REPORT.md) 明确引用的路径。立即数扫描只查四个数字码，不是全项目/全协议审计。工具的 RIP 注释可能包含紧邻的多个常量块；本报告按指令写入长度重建必要字符串。

大型 SPK、提取 ELF、package.tgz 和 `python-libs` 不进入证据提交；[下载来源](download-manifest.json)与[成员摘要](selected-members-manifest.json)用于重新获取和校验。离线选择性成员脚本重跑时只接受与记录相同的字节，不覆盖不同内容。
