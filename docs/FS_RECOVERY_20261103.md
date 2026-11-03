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

Earlier approximately four-second scans used positive name caching and are
**not** performance evidence for this integration. Complete original FSearch
scans must be timed again, with saved-index truth checks and zero hydration.

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

Integration validation is in progress. No new package, real-user queue recovery,
NAS round trip or near-local performance acceptance is claimed by this document.
The initial functional snapshot exposed a D-Bus test API mismatch and the
reversed FUSE timeout arguments; its failed records are retained. A corrected
immutable snapshot must pass the workspace, explicit kernel, raw reply and
real local D-Bus gates before performance measurement and packaging.

Issue #2 still includes persistent history and broader desktop work. Issue #5
still includes near-local application experience. Issue #7 needs a verified
per-directory native permission contract. This batch does not close those
larger requirements or restart the paused long-running project goal.
