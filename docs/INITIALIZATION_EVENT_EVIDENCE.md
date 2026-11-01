# Persistent evidence for local initialization reconciliation

Initialization now retains paths and move destinations from the same native
event response as each normalized change. Directory entries retain the numeric
node identity returned in the same server row. These are inputs for a subsequent
local reconciliation scheduler. A subsequent narrow implementation now handles
known directory events as described below; unknown locations remain conservative.

Schema 21 stores event hints beside the unchanged Batch receipt, binding them to
the run, receipt digest, sequence, scope and server restore identity. Validation
checks correspondence before use. Exact replays of old receipts never manufacture
missing hints. Receipt and evidence bytes share the durable budget. Native node
identity changes invalidate the private candidate subtree and stale tickets.

IPC 18 carries the new initialization types. Both processes must be upgraded
together. Ordinary event pulls keep their existing Batch interface. The original
content version token is unchanged. A returned directory identity still echoes
the request; it does not prove independent server identity verification.

An event's sync_id is bounded by its returned event boundary, matching the native
parser. Historical sync IDs, including zero, remain accepted. Current metadata's
max_id is different and may exceed that boundary. Current metadata parents must
not be interpreted as historical move-source parents.

Isolated local validation on 2026-11-01 passed 257 core and 112 runtime library
tests and the workspace all-feature/all-target check. One test in each library
was ignored by its default harness. Cases include replay, corruption, binding,
budget rollback, persistence after reopen, numeric identity replacement and
event-boundary validation. No new NAS test, packaged schema upgrade, old ELF
refusal, or throughput result is claimed for this experiment.

## Known-directory local rechecks

A directory already observed under a matching source path can schedule its own
subtree and necessary direct parents for re-listing, leaving unrelated completed
directories intact. Every ancestry member requires a retained native identity;
denied ancestry, mismatched paths, missing identities, unknown locations and root
events retain conservative handling. Move destinations must have a known matching
parent path and cannot point into the moved subtree. Current metadata parents
are never used to infer a historical source parent.

The plan, candidate applied cursor, stale-ticket rejection and fence invalidation
share one transaction. Event metadata does not replace listed objects. Listing
denial or identity replacement removes stale candidate descendants and rejects
late replies. Existing final coverage, independent fence and writeback barriers
remain required. No extra parent-list request is added to ordinary enumeration.

The subsequent isolated run passed 267 core tests (including ten new local
recheck cases), 112 runtime tests and all-feature/all-target checks. One test in
each library remained ignored. Cases cover both parent-commit orders, late
responses, permission denial, same-name and numeric identity replacement,
restart, mixed conservative events, draft preservation and fence rejection.
This is synthetic Store validation, not a new NAS convergence or timing result.

Unknown deletion and unverified locations still retain the existing conservative
fallback. Progressive browsing, general local reconciliation and all final
acceptance gates remain open. This experiment is not in frozen `.5` (schema 20 / IPC 17).
See [event dependencies](CANDIDATE_EVENT_COVERAGE.md) and
[delivery gates](../DELIVERY_GATES.md).
