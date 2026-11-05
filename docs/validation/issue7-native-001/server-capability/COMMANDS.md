# 静态补充证据复现

输入是既有官方公开包中提取的 `syncd.elf` 和 `libsynosyncservercore.so.elf`，摘要见 [报告](REPORT.md)。先前下载及提取过程见原 evidence-bundle-002；本补充不执行软件，不产生真实请求。

以下 PowerShell 在仓库根目录运行；`raw` 初次不存在时执行。重现时把 `--out` 换为新的目录，脚本拒绝覆盖已有结果。

```powershell
$evidence = 'work/issue7-native-evidence/public-server'
& 'C:/Users/Public/Python3123/python.exe' "$evidence/capability-supplement-001/reproduce-static.py" --syncd "$evidence/syncd.elf" --core "$evidence/libsynosyncservercore.so.elf" --python-libs 'C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/pylibs' --out "$evidence/capability-supplement-001/raw"
```

`reproduce-static.py` 只启动同一个 Python 解释器和随包保留的 `analyze-server-elf.py`，解析 ELF 动态符号、程序段及所选函数指令。输入软件文件只按字节读取；不加载其共享库，不执行其中代码。Capstone 仅解码 x86-64 指令。虚表读取采用相同 ELF reader，记录 address point 相对槽位及已定义符号目标。

原件保留函数全部指令和符号名称。自动 RIP 字符串注释有时包含相邻常量，不应把这些注释直接当完整协议文本；报告里的能力字段可由 ToPObject 固定字节写入重建。

文件清单对所有普通文件（除 SHA256SUMS 自身）计算 SHA-256。冻结后不改内容；新结论另建编号补充包。只读完整性检查可逐行读取 SHA256SUMS，并对对应相对路径重新计算摘要。该检查不是产品测试。
