# Filesystem and issue recovery: functional and performance validation

Status as of 2026-11-04: functional and package-build gates passed for source
`dfb5aca4bf571f35ade337395ba15c761cc55075`, targeting development version
`0.1.0~dev20261103.9` (schema 23, IPC 20). Same-window performance measurements
have also completed; near-local performance acceptance remains open. The isolated
installation and upgrade gates passed. Release files and visibility are listed on
the [development release page](https://github.com/Austrust/drive-native/releases/tag/dev20261103.9).
This report contains public conclusions;
full logs and experimental artifacts remain in the local evidence archive.

## Scope and behavior

The integration scopes upload/remove dependencies so an unrelated pending
removal cannot stall every upload. Related identities, paths and ancestors
remain guarded; an uncertain commit is reconciled rather than blindly resent.
The issue window offers an asynchronous recheck with explicit accepted/busy
feedback while preserving pause, certificate and authentication gates.

Directory construction and inode allocation reduce repeated metadata work.
Name entries always use TTL zero; clean authoritative attributes have their
own TTL. Preview publication retains its permission/stamp guard through the
reply, and invalidation queue loss disables new attribute leases. Details:
[recovery changes](../../FS_RECOVERY_20261103.md),
[metadata cache policy](../../METADATA_CACHE_POLICY.md), and
[directory construction](../../DIRECTORY_CONSTRUCTION_SCALE.md).

## Completed checks

| Evidence batch | Actual result | Scope |
| --- | --- | --- |
| Corrected functional snapshot 002 | 55 vendored fuser unit tests passed | Includes unequal entry/attribute TTL wire serialization |
| Snapshot 002 explicit release kernel runs | 31 passed with default environment; 31 passed with attribute-TTL environment set to zero | Real synthetic FUSE mounts; the two path-revalidation cases explicitly set attribute TTL to 30 seconds in both runs |
| Snapshot 002 runtime kernel case | 1 passed | Explicit filesystem shutdown regression |
| Supplemental snapshot 003 | 888 Rust passed, 0 failed, 35 ignored | Workspace regression; product input matches 002, test Builder API corrected |
| Snapshot 003 separate D-Bus session | 1 passed | Real local session bus recovery request |
| Standard package014 build | 888 Rust and 117 GUI passed | Frozen source identity checked after build |
| Package014 additional debug and release kernel checks | 31 FUSE plus 1 runtime case passed in each mode | Explicit ignored tests; zero failures; no NAS |

These are separate executions, not additive counts of distinct features.
Ignored workspace tests are not silently treated as passed; the stated explicit
kernel cases were run separately.

## Original FSearch behavior verification

An independent diagnostic fixture was built from the immutable snapshot 002.
Only the example received `cfg(drive_fs_diagnostics)`; the original source,
ordinary fixture ELF and all 387 copied dependency rlibs retained their hashes.
The target copy used no hardlinks. This establishes a diagnostic build's
provenance, not byte identity with the ordinary fixture executable.

Using the original upstream FSearch 0.3.1 and its saved-index auditor:

- The main scan had issued 10 OPENDIR requests before cancellation. The other
  synthetic mount completed 17 stat/listdir samples during that scan.
- Cancellation used SIGTERM on the verifier-owned scan process group, not the
  graphical application's cancel button. The mount remained usable afterwards.
- A complete rescan after cancellation and another after local edits each
  matched all 11,630 truth entries: zero missing, extra, duplicate, invalid or
  attribute-mismatched entries, with a fresh saved index each time.
- Rename, unlink, a 49-byte local draft, immediate read-back and timestamp
  changes were visible. Scanning preserved the unsent draft.
- 200 wide-directory open/read/release cycles returned 1,000 entries each.
- Both mounts recorded zero body hydration attempts and stopped normally with
  exit code zero; neither mount remained.

These runs shared resources with package014 and supplemental validation builds.
Their elapsed times are not performance comparisons. Diagnostic-fixture
behavior coverage is not presented as the same test on the ordinary ELF.

## Ordinary-ELF performance measurements

The completed performance window used the ordinary frozen fixture, original
FSearch 0.3.1 and saved-index truth checks. Positive entry TTL was **0 ms**;
clean authoritative attribute TTL was **1000 ms**. Host page caches were not
dropped. These are fresh-mount first scans and subsequent scans, not cold-disk
measurements. The `.8` comparison alternated baseline and fixed inputs in the
same window, with three rounds per condition.

| Synthetic tree / scan | Published .8 median | This integration median | Local ext4 median |
| --- | ---: | ---: | ---: |
| 102,130 entries, first scan | 74.9664 s | 10.3359 s | 0.5151 s |
| 102,130 entries, repeat scan | 67.3995 s | 11.1387 s | 0.5150 s |
| 319,679 entries / 56,073 directories, first scan | Not measured in this comparison | 59.4316 s | 1.7678 s |
| 319,679 entries / 56,073 directories, repeat scan | Not measured in this comparison | 56.5244 s | 1.7678 s |

The paired 102,130-entry medians reduced elapsed time by 86.2% for first scans
and 83.5% for repeats. All **30 complete scans** across the two tree sizes passed
saved-index truth checks, with zero missing, extra, duplicate or attribute-error
entries and zero hydration attempts.

The dense fixture matches aggregate entry/directory counts only; it does not
reproduce a real account's contents or directory shape. Its remaining roughly
minute-long scans are far slower than local ext4. **Near-local experience is not
achieved; issue #5 remains open.** This window does not test NAS initialization.
The old approximately four-second result from positive-name caching is not used.

[Machine-readable benchmark summary](BENCHMARK_SUMMARY.json) preserves the
sanitized authoritative measurements and source/binary identity. The dense
harness's obsolete descriptive label is not used as provenance; frozen source
manifests and the exact ordinary ELF identify these runs.

## Preserved failures and corrections

Functional snapshot 001 failed before workspace and D-Bus test execution due
to a test's zbus Builder API mismatch. Its explicit FUSE matrices each stopped
after 10 passes and two old-path revalidation failures; later preview tests had
not run. The failure exposed a real wrapper error: `ReplyEntry` reversed the
internal `(attr_ttl, entry_ttl)` arguments. Intended entry zero / attribute
30 seconds became a 30-second name cache. The wrapper order was fixed, with
unequal seconds/nanoseconds ABI tests for both ordinary entry and PLUS replies.
Kernel regressions additionally verify that Store no longer resolves the old
name before attempting old-path `O_TRUNC`, and that moved content remains intact.

Snapshot 002 then passed its vendor/kernel gates but still failed workspace and
D-Bus compilation on two remaining unnecessary Builder `unwrap` calls. Its
nonzero overall result is retained. Snapshot 003 corrected those test-only API
calls and passed the workspace and real D-Bus checks above.

FSearch behavior attempt 001 stopped at its diagnostic-capability prerequisite:
the ordinary fixture's sample omitted diagnostic counters, so no active-scan,
cancel, rescan or editing check ran. The verifier exited before its fixture
handle reached normal cleanup; the fixture logged Broken pipe on closed stdout.
Subsequent checks found no remaining mount or fixture process. Attempt 002 used
the separately built diagnostic example and completed the stated behavior checks.

## Installed package and remaining gates

The completed measurements above establish this batch's improvement, while
near-local performance acceptance remains open.
The actual `.8` to `.9` paused handover passed in an existing Ubuntu 24.04
chroot. Schema 23 and IPC 20 are unchanged. Both dirty drafts, both pending
operations, explicit modification-time intents and paired worker images were
preserved. The installed backend advertised the new recovery capability and
returned the expected paused response without resuming the task. Same-version
reinstall, purge retaining user data, reinstall and GUI single-instance checks
passed. The old paired `.8` ELF successfully read back the same-schema state;
an installed package downgrade was not attempted. Cleanup was normal and host
packages were unchanged. Details: [package014](../native-package-014/REPORT.md).

No new NAS round trip, original user's queue recovery, Mint desktop/theme or
suspend/resume acceptance is claimed. Issue #7's per-directory native write
permission foundation is excluded and remains unresolved. Persistent history
(#2), near-local complete filesystem experience (#5), ordinary full-directory
synchronization and remaining initialization/data-safety gates remain open.
The previously paused long-running goal is not declared resumed or complete by
this functional report.
