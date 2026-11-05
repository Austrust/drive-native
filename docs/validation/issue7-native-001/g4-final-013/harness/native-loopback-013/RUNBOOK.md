# 合成原生协议回环验证

本目录只提供独立验证 harness，不修改产品源码。冻结候选由协调者通过 `--candidate` 指定，并与 `--source` 的不可变归档对应；脚本不硬锁某个历史候选。编写者仅做静态审阅，**未构建、未实测**；执行结果由 G4 协调者保存到私有 run。

`main.rs` 以公开 `sproto::Client` 完成登录、建 channel 和动作请求。服务端只绑定 `127.0.0.1:0`，自行实现受限 pstream codec；所有用户名、密码、会话和文件内容都是写死的合成值。没有 NAS、真实凭据或外部 DNS 请求。该验证不覆盖真实 worker ELF、TLS、服务端实际 ACL 或真实 NAS 权限配置。

设计矩阵为 66 项：下载响应 7 项、登录/连接阶段误分类防护 2 项、upload/rename/remove/mkdir 直接动作响应 44 项、long-poll profile 通知 13 项。错误码为十六进制。下载仅 `3002/3008` 预期为 `DownloadPermissionDenied`；`3004/8004/9005/a005` 保持普通 Server，宽整数 `0x1_0000_3002` 必须 Decode。变更操作分类使用独立硬编码预期表，匹配时必须为 `MutationPermissionUncertain`，不表示无部分提交或可以重试。

long-poll 包括空/其他 view 的 profile-only=true、false、旧 digest 保留、缺失标记兼容，以及非法标记、缺失/错误 digest、缺失/错误 changes。服务端检查实际 action、身份路径、请求 cursor/digest；上传必须确实接收合成正文。下载拒绝后原目标内容必须保留，上传后源快照不变。

每项独立连接；客户端 connect/action 各 6 秒超时、peer 读写各 4 秒、整项 18 秒。产品 panic 由独立 Tokio task 捕获为该项失败，继续后续项目。stdout 为逐项 JSON，最终文件包含 pass/fail/ignored/total。退出码 0=全部通过，1=矩阵失败，2=准备或输出错误。临时文件仅在 `--work-dir` 内，输出文件必须不存在。

由协调者在主矩阵使用 target 结束后按以下顺序执行。变量均为绝对路径，`probe` 约定为 run 下的 `native-loopback-crate`；不要把 crate 放进冻结 source。`harness` 指本目录，`source_dir` 指最终不可变归档，`target_dir` 指已授权且当前无竞争的独立验证 target。

```bash
python3 "$harness/prepare.py" \
  --source "$source_dir" --output "$probe" \
  --candidate "$candidate_commit"

# 从复制的候选 Cargo.lock 起步，offline metadata 仅更新外部 crate 的锁。
# 不先 generate-lockfile 或重新解析整套未锁定依赖。
CARGO_TARGET_DIR="$target_dir" cargo metadata --offline \
  --manifest-path "$probe/Cargo.toml" --format-version 1 \
  > "$probe/cargo-metadata.json"
python3 "$harness/verify-lock.py" --crate "$probe" \
  --source "$source_dir" --metadata "$probe/cargo-metadata.json"

CARGO_TARGET_DIR="$target_dir" cargo build --offline --locked \
  --manifest-path "$probe/Cargo.toml" -j2
"$target_dir/debug/native-loopback-probe" \
  --work-dir "$run/native-loopback-scratch" \
  --output "$run/native-loopback-results.json"
```

依赖核对要求所有 registry `(name, version, source, checksum)` 均存在于候选锁，并检查 metadata 中的 registry 包都在生成锁内。允许的 path 包只有此 crate 与冻结 sproto。若离线解析引入新依赖或版本，脚本保留 `lock-verification.json` 并非零退出，协调者应停止该构建步骤，不静默接受漂移。准备和核对产生 `provenance.json`、`cargo-metadata.json`、`lock-verification.json`；执行者应另外保存完整命令、日志及 ELF SHA-256。
