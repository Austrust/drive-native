# 离线复现

在项目根目录运行下列 PowerShell 命令。它们只解析已存在的软件字节，不运行 ELF、WSL、产品构建或测试。新输出应写入复现目录，不能覆盖冻结证据包。

```powershell
$l2Python = 'C:/Users/Public/Python3123/python.exe'
$l2Libraries = 'C:/Users/a_tas/Documents/deepseek-harness/default-workspace/synodrive/pylibs'
$l2Source = 'work/issue7-native-evidence/public-server'
$l2Evidence = 'work/issue7-native-evidence/l2-supplement-001'
$l2Reproduction = 'work/issue7-native-evidence/l2-reproduction'

Get-FileHash -Algorithm SHA256 -LiteralPath "$l2Source/syncd.elf","$l2Source/libsynosyncserver.so.elf","$l2Source/libsynosyncservercore.so.elf"

& $l2Python "$l2Evidence/analyze-server-elf.py" "$l2Source/syncd.elf" --out "$l2Reproduction/selected" --pattern 'QueryUserInfoHandler(13AppenUserView|25AppendOtherViewAttributes|11GetHomeView)|ListTeamFolderHandler6Handle' --python-libs $l2Libraries

& $l2Python "$l2Evidence/analyze-server-elf.py" "$l2Source/libsynosyncserver.so.elf" --out "$l2Reproduction/sdk" --pattern 'GetAdvSharePrivilege' --python-libs $l2Libraries

& $l2Python "$l2Evidence/extract-helper.py" "$l2Source/libsynosyncserver.so.elf" --reader "$l2Evidence/analyze-server-elf.py" --out "$l2Reproduction/sdk" --python-libs $l2Libraries

& $l2Python "$l2Evidence/analyze-server-elf.py" "$l2Source/libsynosyncservercore.so.elf" --out "$l2Reproduction/fields" --pattern 'DriveAcl10Capability9ToPObject' --python-libs $l2Libraries

& $l2Python "$l2Evidence/analyze-server-elf.py" "$l2Source/libsynosyncservercore.so.elf" --out "$l2Reproduction/core" --pattern '^(_ZN9synodrive4core14NodeCapability3GetERKNS1_5ParamE|_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl3GetERKNS1_5ParamE|_ZN9synodrive4core14NodeCapability18NodeCapabilityImpl9GetByUserERKNS1_5ParamE|_ZN8DriveAcl13GetCapabilityERKNS_8UserInfoEjPNSt7__cxx1112basic_stringIcSt11char_traitsIcESaIcEEES9_)$' --python-libs $l2Libraries
```

本次实际新增提取只执行上述 syncd、SDK 与 helper 三组；fields 与 core 的选定原件复用此前同 SHA 软件生成的冻结反汇编，逐字节复制而未改写。core 的 [analysis.json](core/analysis.json) 只保留所选函数元数据并注明来源；重新提取的分析 JSON 包含通用解析器元数据，比较时以函数地址、区间和 `.asm` 字节为准。

- [通用解析器](analyze-server-elf.py)：PT_DYNAMIC 动态符号、GNU/SysV hash、Capstone 反汇编及直接/PLT 调用解析。
- [名单 helper 提取器](extract-helper.py)：输入 SHA 固定，保存 `[0x10ae80,0x10b120)`；窗口尾界在下一个可见独立序言前，不冒称拥有动态符号长度。
- [syncd 提取元数据](selected/analysis.json)、[SDK 提取元数据](sdk/analysis.json)、[helper 窗口摘要](sdk/helper-window.json)、[字段元数据](fields/analysis.json)。

地址注释是工具辅助；字符串键经编译器立即数拼接时，应结合原始指令和本文标出的写入点核对，不能把相邻常量注释当作整个字段内容。大型 SPK、ELF 和 Python 依赖保留原位置，不复制进本包。
