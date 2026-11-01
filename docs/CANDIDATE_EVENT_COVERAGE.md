# Initialization event dependencies

The `.5` development change keeps a received file event pending while its parent
has not yet been discovered. It does not immediately invalidate every completed
directory. This is a client scheduling policy, not a reverse-engineered claim
about the official Windows client's unknown-parent algorithm.

## State and recovery

`InitializationApplyResult` distinguishes `Applied`, `AlreadyApplied`, and
`AwaitingCoverage`. Waiting leaves the receipt durable, the applied cursor
unchanged, and completed directory coverage intact. The owner advances up to
sixteen directories before reconsidering the dependency. It pauses additional
Pull requests while waiting instead of accumulating an unbounded event inbox.
Reopening preserves the receipt and still blocks writeback and final publication.

The complete batch is inspected before waiting. Root changes, directory changes,
unknown deletions and invalid parent types retain conservative handling. If all
directory coverage is terminal and the parent is still absent, the existing
whole-tree fallback remains; absence is not treated as proof of an external or
irrelevant event. The native request and path validation already constrain the
selected scope.

An independently reproduced counterexample required a second recovery path:
the first page refers to a missing parent, while a later page reports a rename
of a directory still queued under its old path. Pausing Pull could repeatedly
query that old path and fail before reading the rename. Now, only a typed
`native_snapshot_changed` directory response during dependency waiting permits
a conservative escape. All concurrent reads have joined before the durable head
receipt is applied with a complete recheck. Pull can then resume. Other errors
are not reclassified as directory races. At most sixteen escapes are allowed
per owner invocation, including its internal retries.

Receipt application and invalidation remain in one transaction. Neither the
escape nor ordinary waiting commits the authoritative checkpoint. A sequence-only
lookup avoids decoding an entire event body merely to find its sequence; actual
application still validates run, epoch, continuity and the source cursor.

## Actual validation

- Initial isolated run: 59 core initialization tests, 12 runtime owner tests and
  workspace all-feature/all-target checks passed. This was insufficient coverage.
- A new local RPC counterexample, with identical production code, actually failed
  (0 passed, 1 failed). It preserved existing cache, dirty content, queued work,
  the authoritative cursor and writeback barriers.
- After correction: 62 core initialization tests, 15 owner tests and the same
  full-target check passed. Cases include the previously failing cross-page
  rename, unknown-parent restart, stale tickets, same-name replacement, dangerous
  mixed batches, non-race errors and exhausting the sixteen-escape budget.
- Only the six changed Rust files were formatted. Formatting both the tested
  source and final worktree produced identical bytes; tested source copies and
  failed evidence were not overwritten.

These are synthetic local tests, not a new NAS or throughput result. Frozen
package regressions, installation and actual initialization remain to be run.
Detailed logs and source manifests remain local.

## Remaining work

Later-page root and permission changes can be learned later while Pull is
paused. Existing barriers prevent remote writes and candidate publication; they
do not prove immediate revocation of existing local cached access. Candidate
permission visibility needs its own implementation.

Unknown parents after complete coverage and directory events can still cause
whole-tree rechecking. More precise reconciliation needs retained event paths
and independently checked parent identities; current directory reply identity
echoes the requested identity. Large pending pages can also incur repeated
application work while waiting. No whole-client performance claim is made.

Progressive FUSE visibility, ordinary-folder full sync, incremental desktop state,
Mint, fault recovery and final long-run gates remain open. See
[DELIVERY_GATES.md](../DELIVERY_GATES.md), [GOAL.md](../GOAL.md) and the previous
[query-order repair](CANDIDATE_QUERY_ORDER.md).
