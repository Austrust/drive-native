# Package014: build and isolated installation validation

Development version `0.1.0~dev20261103.9` completed the build, explicit kernel,
paused-upgrade and installation-lifecycle checks below. The package uses SQLite
schema **23** and internal IPC **20**.

- Frozen build source: `dfb5aca4bf571f35ade337395ba15c761cc55075`.
- Package: `drive-native_0.1.0~dev20261103.9_amd64.deb`.
- Size: **54,027,296 bytes**.
- SHA256: `07701d94ee015703d71750bfd7873cdeaebe7420fe7d7ae6f73c21bf501c6e81`.

The complete Git archive and extracted build inputs were checked against the
frozen commit before and after the build, including the vendored fuser provenance
files. This run used a fresh, independent Cargo target. Detailed logs, synthetic
state and machine-specific evidence remain in the local archive.

## Build and kernel checks

| Check | Actual result |
| --- | --- |
| Standard debhelper build and workspace tests | 888 Rust passed, 0 failed, 35 ignored |
| GUI tests during the package build | 117 passed across 16 test modules |
| Explicit debug kernel tests | 31 FUSE and 1 runtime passed |
| Explicit release kernel tests | 31 FUSE and 1 runtime passed |
| Source identity after build | All frozen tracked inputs matched |

The kernel commands enabled all features, including test seams, and filesystem
diagnostics. The preview READDIRPLUS regression therefore checked actual request
counters. Debug and release results are separate executions, not additional
distinct features. Ignored workspace tests are not counted as passed.

## Paused upgrade and recovery interface

The existing isolated Ubuntu 24.04 environment initially contained
`0.1.0~dev20261102.8`, also schema23/IPC20. An independent synthetic managed task
created **two dirty drafts and two pending operations**, then paused. The old FSD
remained alive while the new package replaced its installed files.

The actual Qt upgrade controller requested normal shutdown and confirmed a new
D-Bus owner, matching task and installed executable identity, a mounted directory,
and preserved pause. Both draft bodies, cursor, pending-operation semantics and
explicit upload mtime intentions remained unchanged. The running old FSD retained
its original worker executable image; the new FSD retained the new paired image.

The installed new FSD advertised
`GetOverview.recovery_actions.recheck_issues=true`. Its real local D-Bus
`RecheckIssues` call returned `accepted=false`, `paused_preserved=true` and
`message_code=paused`; a subsequent overview remained paused. No busy retry was
needed. This verifies a usable installed recovery interface without claiming that
the request resolved an issue or resumed synchronization.

## Installation lifecycle and data retention

A separate synthetic fixture verified offline edits and rename, installed GUI
startup and single-instance behavior, successful readback using the original
paired `.8` FSD/worker, reopening with `.9`, same-version reinstall, package purge
with user data retained, and reinstall followed by another reopen.

Across these stages the fixture retained **two dirty drafts, two pending
operations and zero uncertain operations**. Protected configuration and content
files remained byte-identical; logical queue, namespace, cursor and timestamp
intent checks also passed. Raw SQLite/journal differences were retained privately
and were not confused with content loss.

This was a **same-schema23** upgrade. No migration backup was required, no old
package was installed over the new one, and the old ELF was expected to read the
database successfully. The result does not establish arbitrary-version downgrade
compatibility. All synthetic services and mounts stopped normally; final isolation
checks found no remaining test process or mount. No force cleanup, NAS access,
host-package change or original user-task replacement was performed.

## Remaining acceptance boundaries

These results cover an existing Ubuntu 24.04 isolated installation environment,
not a fresh operating-system installation or Mint desktop acceptance. The locked
synthetic tasks verify paired worker images, not authenticated network IPC.
Unknown-commit operations and active initialization-candidate upgrades were not
present in these upgrade fixtures. Production keyring startup, upgrade fault
recovery and broader long-running acceptance remain separate gates.

The package validation does not prove complete Personal_Sync initialization,
native ACL support, or near-local filesystem performance. Functional and measured
filesystem findings are recorded in the
[filesystem and recovery report](../fs-recovery-001/REPORT.md). User-facing upgrade
behavior and historical migration boundaries are in
[upgrade and recovery](../../UPGRADE_RECOVERY.md).
