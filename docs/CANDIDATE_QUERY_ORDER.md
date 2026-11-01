# Candidate recursive traversal

2026-11-01. Development fix for recursive invalidation and removal; schema 20
and IPC 17 remain unchanged. Package `.4` is planned, not yet released.

## Observed failure

An isolated read-only Personal_Sync trial used the actual `.3` package with a
fresh database and private session bus. A controlled FSD restart retained the
candidate, original event boundary and 45 completed directory receipts. Later,
with about 28,000 indexed nodes, a status query raised a GLib error. The observer
did not retain its numeric error code; it must not be retrospectively classified
as a specific D-Bus timeout. Normal shutdown took 76.327 seconds and exited zero.

After stopping, one received Upsert batch had been applied and all 5,131
directories were pending again. The event described a regular file whose
identity and parent were absent from the partial candidate. Existing conservative
reconciliation therefore required whole-tree rechecking. No NAS write operation
was issued. The trial did not reach a mounted, complete initial index.

## Root cause and change

The recursive CTE allowed SQLite to put the candidate entries outside the
recursive frontier. Its recursive plan used only `run=?` on the parent index,
then scanned the frontier. This repeatedly scanned a whole candidate rather
than looking up each parent's direct children.

The three production statements now put `tree` first with `CROSS JOIN`, and
constrain the existing `(run,parent)` index using both keys. They retain `UNION`
deduplication, outer run constraints, receipt invalidation, counters and
transactions. No unknown-parent or permission checks were removed.

## Verified scope

- Product dependency SQLite 3.51.3 reproduced the old plan on a backup of the
  stopped database: 27,886 nodes, 5,131 affected directories, 72.119 seconds.
- The revised statement affected the same 5,131 rows in 0.093 seconds. Its plan
  used `run=? AND parent=?`. Both mutations were rolled back.
- This is one old-then-new local SQL replay, with potential page-cache order
  effects. It is not a paired whole-client or NAS throughput claim. The old
  32-bit VM-step counter may wrap and is not used to calculate a speedup.
- All 44 initialization tests passed, including five new tests exercising the
  actual three production SQL constants: branching and depth, colliding IDs in
  separate runs, deletion counters and FK cascades, rollback and cycle
  termination, and bounded VM-work growth from 256 to 1,024 nodes.
- Workspace all-features/all-targets check passed. The first replay-tool build
  failed because its counter used `u64` instead of SQLite's `i64`; no replay or
  tests ran in that attempt. It is preserved separately.

Reproduction tool: `cargo run -p drive-core --release --example
initialization_query_replay -- /absolute/isolated/replay-copy.sqlite`. Use only
an explicit database backup; raw database contents and server evidence are not
distributed. The tool never connects to a NAS and rolls back both updates.

Full frozen-package regressions, installation, and a fixed FSD real-NAS trial
remain pending. Whole-tree invalidation on ambiguous events, progressive
directory visibility, normal-folder full sync, and final safety/desktop/long-run
gates remain open. See [DELIVERY_GATES.md](../DELIVERY_GATES.md).
