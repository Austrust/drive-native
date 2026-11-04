# FSearch performance and actionable issue recovery

This development integration starts from the published `.8` source and retains
its initialization, timestamp, local draft and upgrade safeguards. It combines
the directory work from `codex/fsearch-perf-1`, the dependency-scoped upload
barrier from `fd7bddd`, and the directory construction changes from `d78a122`.
The experimental per-directory write-permission foundation is excluded: there
is no verified production native ACL source for that policy. Schema remains 23
and the FSD/worker wire version remains 20.

## Directory scanning

The inode completeness hint avoids a global missing-inode check per directory.
Prepared statements and a lightweight directory/parent identity projection
reduce repeated SQL preparation and full file-record construction. Existing
bounded directory views, stable continuation cookies, local creation precedence,
permission filtering and inode repair remain in effect. See
[directory construction](DIRECTORY_CONSTRUCTION_SCALE.md).

Positive names always have entry TTL zero. Clean authoritative attributes may
use a separate bounded TTL. The integration corrects reversed entry/attribute
arguments in the vendored FUSE reply wrapper; equal timeouts had hidden this
error. Raw reply tests use unequal values and kernel tests attempt to truncate
an old path after moving its inode. See [cache policy](METADATA_CACHE_POLICY.md).

Candidate preview replies keep the current permission/stamp check protected
through publication. A busy Store cannot authorize reuse of an old preview.
Loss at either invalidation queue disables new attribute leases.

The new ordinary-ELF window used entry TTL 0 and attribute TTL 1000 ms. Three
paired rounds on 102,130 entries reduced first-scan median from `.8`'s 74.9664 s
to 10.3359 s and repeat median from 67.3995 s to 11.1387 s. A denser synthetic
tree of 319,679 entries and 56,073 directories still took 59.4316 s first and
56.5244 s on repeat, compared with local ext4's 1.7678 s. All 30 complete scans
passed saved-index truth checks with zero hydration. These results improve
scanning but do not meet near-local experience; #5 remains open. See the
[performance results](validation/fs-recovery-001/REPORT.md#ordinary-elf-performance-measurements).

Earlier approximately four-second scans used positive name caching and are
**not** evidence for this entry-zero integration. The synthetic dense tree is
not a real account's contents or directory shape; NAS initialization was not
validated by this window.

## Recovering from a visible issue

The issue window presents the reason, the next usable action, and a safe way to
request reconciliation. It also offers pause while preserving local data,
asynchronous navigation to an available parent, and copying allowlisted
diagnostic fields. Unknown commits are not reset to pending or blindly resent.
The request acknowledgement does not mean an issue has been resolved.

`RecheckIssues` is an additive D-Bus method, advertised by
`GetOverview.recovery_actions.recheck_issues`. It preserves user pause and
authentication/certificate gates. When Control is busy it immediately answers
`accepted=false, message_code=busy`; the GUI explains that nothing was queued
and permits a later retry. Normal independently queued writes may proceed when
the existing scheduler wakes. Older backends without this capability get a
visible explanation rather than an unsupported call.

The upload barrier now checks overlapping identities, ancestry and paths,
rather than allowing an unrelated pending removal to block every upload.
The overview reports this blockage separately from connection state. See
[issue recovery interaction](../desktop/ISSUE_RECOVERY.md).

## Validation and remaining scope

Functional validation and the standard package014 build have completed for
frozen source `dfb5aca4bf571f35ade337395ba15c761cc55075`, targeting development
version `0.1.0~dev20261103.9`. Snapshot 003 passed 888 Rust tests and a separate
real local D-Bus case. Snapshot 002 passed 55 vendor tests, two 31-case explicit
release FUSE matrices and one runtime kernel case. The package build passed
888 Rust and 117 GUI tests; additional debug/release checks each passed 31 FUSE
and one runtime case. These are separate runs, not distinct-feature totals.

Original FSearch cancellation, rescan and local-edit behavior passed on an
independent diagnostic fixture, including two complete 11,630-entry truth
comparisons and zero hydration. That run shared build resources and provides
no timing comparison. Initial API/TTL failures and the first behavior harness's
diagnostic prerequisite failure remain documented in the
[functional validation report](validation/fs-recovery-001/REPORT.md).

[Isolated installation and paused upgrade](validation/native-package-014/REPORT.md)
passed, including preserved drafts, pending operations, same-schema old-reader
compatibility and reinstall/purge retention. Performance measurements have
completed, while near-local acceptance stays open. Release scope and files:
[development .9](RELEASE_20261103_9.md). No new NAS round trip or original-user
queue recovery is claimed.

Issue #2 still includes persistent history and broader desktop work. Issue #5
still includes near-local application experience. Issue #7 needs a verified
per-directory native permission contract. This batch does not close those
larger requirements or restart the paused long-running project goal.
