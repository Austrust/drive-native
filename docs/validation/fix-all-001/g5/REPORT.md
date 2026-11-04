# G5 固定样本配对基准

固定候选 `bae1ef1ddc4061df586221557011f4e30f04d134` 与基线 `d555944bfe3058cb223bf7d8ebc71244f1e4d7a3`。三树各3轮first/repeat完成，另3树各1轮诊断，合计72次保存索引扫描。missing/extra/duplicate/invalid/属性不符均为0，hydration均为0，全部自建fixture正常停止、无残留挂载。性能是否达到规格必须分别看树级秒数与≤0.15ms/目录列；不能把局部改善称为近本地达标。

原始数字见 [primary summary](primary-001/summary.json)、[diagnostic summary](diagnostic-001/summary.json)、[机器结论](result.json)。窗口primary为2026-11-04T15:51:30.883436+00:00至2026-11-04T16:51:16.440498+00:00，diagnostic为2026-11-04T16:51:16.751368+00:00至2026-11-04T17:15:35.657136+00:00；START授权和完整启动命令记录于window/及harness/。环境为WSL2 Ubuntu24.04 ext4普通用户，非Mint实机。全程未连接NAS/凭据/生产文件、未push或发布。

## 主结果

单位为**monotonic实测elapsed秒**；每格为3个固定样本中位数，完整逐轮原值保留。每目录成本为FSearch总monotonic elapsed毫秒/含root目录数，包含client/helper启动、遍历、排序及保存，不是FUSE净成本，也不冒充严格宿主墙钟。树规模100k=102,130项/2,066目录；dense=319,679项/56,073目录；既有真实manifest=327,750项/56,554目录。后者只读取事先已保存清单，未扫描用户源目录。

| 树 | 状态 | .9 | 候选 | 本地 | 候选/本地 | 树级目标 | 候选ms/目录：≤0.15 |
|---|---|---:|---:|---:|---:|---|---|
| 100k | first | 9.637 | 6.482 | 0.521 | 12.43× | 2s：未达 | 3.1375：未达 |
| 100k | repeat | 9.286 | 6.432 | 0.521 | 12.34× | 1.6s：未达 | 3.1131：未达 |
| dense-319k | first | 56.018 | 63.028 | 1.725 | 36.54× | 13s：未达 | 1.1240：未达 |
| dense-319k | repeat | 53.512 | 61.326 | 1.574 | 38.95× | 12s：未达 | 1.0937：未达 |
| manifest-327750 | first | 96.183 | 112.513 | 1.774 | 63.43× | 15s：未达 | 1.9895：未达 |
| manifest-327750 | repeat | 97.939 | 115.875 | 1.625 | 71.30× | — | 2.0489：未达 |

同口径本地first每目录成本：100k=0.2524ms、dense=0.0308ms、manifest=0.0314ms。所有角色逐轮成本在原JSON中；不因本地自身的启动成本而改写规格门槛。

首扫为新fixture与新FSearch数据库，不清宿主页缓存。单调钟计时在fixture ready/索引构建完成后、打开scan log及Popen之前开始，覆盖dbus-run-session、xvfb-run与原版FSearch，至process.wait完成；随后audit不在此elapsed内。本地参考由同清单真实size/mtime的稀疏文件构造，非空文件镜像；FUSE两列始终扫描真实挂载。正entry TTL始终0，attr TTL固定1000ms，无放宽TTL、过滤挂载或调大预算。

## UTC跨度与单调钟边界

[逐扫描clock-audit](clock-audit.json)保存全部72组原值和分组中位数。实际顺序为UTC start→load_before→argv/getrusage→monotonic start→log.open/Popen/wait/log.close→monotonic stop→getrusage→UTC end→load_after/SHA→audit。UTC end**不包含load_after、SHA与audit**；UTC跨度与monotonic elapsed**不是完全同一采样区间**。最大绝对差为主样本11.789638s、诊断11.732234s。边界开销与时间源差异没有独立测量，原因未定位，不能归因于宿主暂停或校时。阈值与上述主表均保留原monotonic口径，未用UTC差替换或修正数据。fixture CPU另由外层采样包围scan/audit，不能误作仅FSearch计时区间内的CPU。

| 候选树 | 状态 | monotonic中位数秒 | UTC跨度中位数秒 |
|---|---|---:|---:|
| 100k | first | 6.482005 | 9.415629 |
| 100k | repeat | 6.431585 | 6.431700 |
| dense-319k | first | 63.027660 | 68.906129 |
| dense-319k | repeat | 61.325762 | 67.163795 |
| manifest-327750 | first | 112.513445 | 124.182656 |
| manifest-327750 | repeat | 115.875268 | 124.718273 |

## 索引准备成本和实际命中

预装成本未计入扫描elapsed，以下单独保留候选每轮ready原值；bytes为0且degraded为true的样本仍按实际SQL回退测量，未重跑挑样。普通ELF未启用诊断，ready不能单独证明每次请求实际命中；命中计数来自下方另跑的诊断ELF。

| 树 | 轮 | 构建秒 | bytes | partial | degraded |
|---|---:|---:|---:|---|---|
| 100k | 1 | 0.695802 | 43527984 | False | False |
| 100k | 2 | 0.846218 | 43527984 | False | False |
| 100k | 3 | 0.746130 | 43527984 | False | False |
| dense-319k | 1 | 1.059868 | 0 | True | True |
| dense-319k | 2 | 1.061280 | 0 | True | True |
| dense-319k | 3 | 1.059916 | 0 | True | True |
| manifest-327750 | 1 | 1.134941 | 0 | True | True |
| manifest-327750 | 2 | 1.139726 | 0 | True | True |
| manifest-327750 | 3 | 1.140869 | 0 | True | True |

## 可测时间分解

以下主样本的fixture CPU与client进程树CPU是实测CPU秒，client含启动辅助进程；不能从elapsed相减当作内核耗时。fixture采样围住scan/audit调用，audit只访问已保存本地索引，未读取挂载正文。表内中位数分别计算，不强行凑和。

| 树 | 角色 | 状态 | 主monotonic秒 | fixture CPU秒 | client/helpers CPU秒 |
|---|---|---|---:|---:|---:|
| 100k | baseline | first | 9.637 | 6.320 | 3.539 |
| 100k | baseline | repeat | 9.286 | 6.150 | 3.280 |
| 100k | candidate | first | 6.482 | 2.930 | 3.453 |
| 100k | candidate | repeat | 6.432 | 2.980 | 3.347 |
| 100k | local | first | 0.521 | 0.000 | 0.752 |
| 100k | local | repeat | 0.521 | 0.000 | 0.729 |
| dense-319k | baseline | first | 56.018 | 41.460 | 17.355 |
| dense-319k | baseline | repeat | 53.512 | 40.210 | 15.854 |
| dense-319k | candidate | first | 63.028 | 49.450 | 16.987 |
| dense-319k | candidate | repeat | 61.326 | 48.620 | 16.032 |
| dense-319k | local | first | 1.725 | 0.000 | 3.372 |
| dense-319k | local | repeat | 1.574 | 0.000 | 3.287 |
| manifest-327750 | baseline | first | 96.183 | 70.240 | 29.113 |
| manifest-327750 | baseline | repeat | 97.939 | 72.050 | 28.970 |
| manifest-327750 | candidate | first | 112.513 | 87.290 | 29.780 |
| manifest-327750 | candidate | repeat | 115.875 | 90.520 | 29.818 |
| manifest-327750 | local | first | 1.774 | 0.000 | 3.655 |
| manifest-327750 | local | repeat | 1.625 | 0.000 | 3.578 |

另1轮诊断实际请求计数如下，**不与主3轮混算**：

| 树 | 状态 | lookup | getattr | opendir | readdir | readdirplus | releasedir | IndexHits | IndexFallback | IndexDegraded |
|---|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|
| 100k | first | 106210 | 7 | 2066 | 2086 | 2088 | 2066 | 112457 | 0 | 0 |
| 100k | repeat | 106210 | 6 | 2066 | 2093 | 2066 | 2066 | 112441 | 0 | 0 |
| dense-319k | first | 377766 | 62 | 56073 | 56156 | 56158 | 56073 | 0 | 433901 | 433901 |
| dense-319k | repeat | 377766 | 60 | 56073 | 56177 | 56073 | 56073 | 0 | 433899 | 433899 |
| manifest-327750 | first | 928781 | 112 | 56554 | 56596 | 56844 | 56554 | 0 | 985447 | 985447 |
| manifest-327750 | repeat | 928781 | 110 | 56554 | 56864 | 56554 | 56554 | 0 | 985445 | 985445 |

同一候选诊断扫描的CPU范围：client user/system均覆盖FSearch及dbus/Xvfb辅助进程，system CPU不是独立的纯FUSE内核往返计时。

| 树 | 状态 | 诊断monotonic秒 | fixture CPU秒 | client user秒 | client system秒 |
|---|---|---:|---:|---:|---:|
| 100k | first | 6.532 | 2.970 | 0.628 | 2.827 |
| 100k | repeat | 6.281 | 2.950 | 0.634 | 2.572 |
| dense-319k | first | 62.829 | 49.400 | 2.931 | 13.962 |
| dense-319k | repeat | 61.375 | 48.970 | 2.896 | 12.930 |
| manifest-327750 | first | 113.373 | 87.860 | 4.204 | 25.658 |
| manifest-327750 | repeat | 111.670 | 87.140 | 3.923 | 24.654 |

诊断elapsed阶段允许重叠、嵌套。IndexReadUs的`_index_time`作用域在**命中时**覆盖索引路径直到reply调用返回，含reply/lease范围；**回退时**仅覆盖lease/索引尝试，退出该块后才走SQL与reply，不含后续SQL路径。它**不是纯HashMap读取，也不是CPU归因**；以下列项不能相加或互减成互斥分解。100k的106,210次LOOKUP及SQL projection计数0是实测请求路径事实，不能据此宣称未经独立跟踪的硬性能下限。DirectoryInodeWrites与IndexBytes的total_us字段分别装行数/bytes，不当作微秒；完整计数与直方图保留在原JSON。所有诊断EntryTtlNonZero=0。

| 树 | 状态 | 阶段 | 次数 | 聚合elapsed秒 |
|---|---|---|---:|---:|
| 100k | first | IndexReadUs | 112457 | 1.856673 |
| 100k | first | LookupQueue | 0 | 0.000000 |
| 100k | first | LookupStoreWait | 0 | 0.000000 |
| 100k | first | LookupProjection | 0 | 0.000000 |
| 100k | first | DirectoryStoreWait | 0 | 0.000000 |
| 100k | first | DirectoryProjection | 0 | 0.000000 |
| 100k | first | ReaddirReply | 2086 | 0.032939 |
| 100k | first | ReaddirPlusProjection | 0 | 0.000000 |
| 100k | repeat | IndexReadUs | 112441 | 1.850264 |
| 100k | repeat | LookupQueue | 0 | 0.000000 |
| 100k | repeat | LookupStoreWait | 0 | 0.000000 |
| 100k | repeat | LookupProjection | 0 | 0.000000 |
| 100k | repeat | DirectoryStoreWait | 0 | 0.000000 |
| 100k | repeat | DirectoryProjection | 0 | 0.000000 |
| 100k | repeat | ReaddirReply | 2093 | 0.032887 |
| 100k | repeat | ReaddirPlusProjection | 0 | 0.000000 |
| dense-319k | first | IndexReadUs | 433901 | 0.001299 |
| dense-319k | first | LookupQueue | 0 | 0.000000 |
| dense-319k | first | LookupStoreWait | 0 | 0.000000 |
| dense-319k | first | LookupProjection | 377766 | 6.543371 |
| dense-319k | first | DirectoryStoreWait | 56073 | 0.000307 |
| dense-319k | first | DirectoryProjection | 56073 | 3.109252 |
| dense-319k | first | ReaddirReply | 56156 | 0.957323 |
| dense-319k | first | ReaddirPlusProjection | 56241 | 3.335611 |
| dense-319k | repeat | IndexReadUs | 433899 | 0.000995 |
| dense-319k | repeat | LookupQueue | 0 | 0.000000 |
| dense-319k | repeat | LookupStoreWait | 0 | 0.000000 |
| dense-319k | repeat | LookupProjection | 377766 | 6.455307 |
| dense-319k | repeat | DirectoryStoreWait | 56073 | 0.000317 |
| dense-319k | repeat | DirectoryProjection | 56073 | 3.094055 |
| dense-319k | repeat | ReaddirReply | 56177 | 0.944915 |
| dense-319k | repeat | ReaddirPlusProjection | 56075 | 3.252857 |
| manifest-327750 | first | IndexReadUs | 985447 | 0.002596 |
| manifest-327750 | first | LookupQueue | 0 | 0.000000 |
| manifest-327750 | first | LookupStoreWait | 0 | 0.000000 |
| manifest-327750 | first | LookupProjection | 928781 | 18.192911 |
| manifest-327750 | first | DirectoryStoreWait | 56554 | 0.000325 |
| manifest-327750 | first | DirectoryProjection | 56554 | 3.539226 |
| manifest-327750 | first | ReaddirReply | 56596 | 0.988220 |
| manifest-327750 | first | ReaddirPlusProjection | 56937 | 4.733079 |
| manifest-327750 | repeat | IndexReadUs | 985445 | 0.002579 |
| manifest-327750 | repeat | LookupQueue | 0 | 0.000000 |
| manifest-327750 | repeat | LookupStoreWait | 0 | 0.000000 |
| manifest-327750 | repeat | LookupProjection | 928781 | 18.024113 |
| manifest-327750 | repeat | DirectoryStoreWait | 56554 | 0.000361 |
| manifest-327750 | repeat | DirectoryProjection | 56554 | 3.519520 |
| manifest-327750 | repeat | ReaddirReply | 56864 | 0.990095 |
| manifest-327750 | repeat | ReaddirPlusProjection | 56862 | 4.664975 |

诊断另测串行目录scandir：每目录在途是monotonic计时的scandir调用总时间/实际调用数，包含Python、FUSE处理、内核与调度；与上文总FSearch elapsed/目录分列，不混称。

| 树 | 目录数 | probe monotonic秒 | scandir在途秒 | 在途ms/目录 | fixture CPU秒 |
|---|---:|---:|---:|---:|---:|
| 100k | 2066 | 0.666 | 0.659 | 0.3189 | 0.360 |
| dense-319k | 56073 | 41.069 | 40.980 | 0.7308 | 34.710 |
| manifest-327750 | 56554 | 95.782 | 95.658 | 1.6914 | 75.920 |

纯内核往返/传输成本 **未单独实测**。这些计数、CPU及elapsed保留了差距证据，不把剩余elapsed武断归给某一未测阶段。

## 负载与证据边界

Windows2s CIM采样初次隐藏启动未产出样本；受控session尝试得到“无法从客户端中访问CIM资源”，保存错误后停止。Windows宿主CPU/内存 **未实测**，未安装或修改系统设置。错误原件见load/，不能把WSL guest数据冒充宿主安静证明。WSL逐秒/proc/stat与loadavg、开始/结束进程名快照均在两个summary中；主窗口guest平均busy=9.30%，诊断=9.29%，这是包含树准备/audit在内的整体窗口观察值，不是每个扫描独占CPU使用率。

[build.json](build/build.json)保存四ELF、同源fixture、归档与toolchain哈希；879个基线归档文件（同源fixture除外）逐一验证仍为d555944，基线ELF复用证明与构建原log保留。普通与诊断分开，无产品RUSTFLAGS，基线example仅用drive_fs_baseline去除新NSI API。FSearch与audit为预存的原版工具，hash已逐次核对。

本包保留numeric summary、逐scan结果、负载、完整命令/harness/hash；真实清单成员名、audit TSV、索引数据库、truth、稀疏镜像、大型ELF不入包。完整私有原件保留于[位置与hash清单](private-originals.json)，其路径仅使用harness固定artifact名。失败或未达结果均未删除或覆盖。
