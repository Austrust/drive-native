use super::*;
use crate::initialization_store::scheduler::{FIRST_PAGE, NEXT_PAGE};
use rusqlite::StatementStatus;

fn dir(id: &str, parent: &str) -> Node {
    node(id, Some(parent), id, true, 7, "server")
}

#[test]
fn claim_fills_after_promoted_descendants_and_issues_atomic_unique_tickets() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &[dir("900", "100"), dir("950", "100")],
    );
    list(
        &mut store,
        &run,
        "900",
        &(0..8)
            .map(|i| dir(&format!("2{i}"), "900"))
            .collect::<Vec<_>>(),
    );
    invalidate(&store.db, &run.run_id, "900", true).unwrap();
    // The old LIMIT-before-dedup selector can only return the same ancestor.
    assert_eq!(
        store
            .initialization_pending_directories(&run, 2)
            .unwrap()
            .len(),
        1
    );
    let before = store.cursor().unwrap();
    let page = store
        .claim_initialization_directories(&run, 2, None)
        .unwrap();
    assert_eq!(
        page.jobs
            .iter()
            .map(|j| j.node.id.as_str())
            .collect::<Vec<_>>(),
        ["900", "950"]
    );
    assert_eq!(page.jobs[0].path, "/Lab/900");
    assert_eq!(page.stats.ancestor_promotions, 1);
    assert!(page.stats.coalesced >= 7);
    assert_eq!(page.stats.previously_claimed, 1);
    assert_ne!(
        page.jobs[0].ticket.generation,
        page.jobs[1].ticket.generation
    );
    for job in &page.jobs {
        assert!(
            store
                .initialization_directory_ticket_current(&job.ticket)
                .unwrap()
        );
    }
    assert_eq!(store.cursor().unwrap(), before);
    assert!(store.initialization_writeback_blocked().unwrap());
}

#[test]
fn loading_prefix_is_bounded_and_continuation_reaches_independent_work() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &[dir("800", "100"), dir("999", "100")],
    );
    list(
        &mut store,
        &run,
        "800",
        &(0..80)
            .map(|i| dir(&format!("3{i:03}"), "800"))
            .collect::<Vec<_>>(),
    );
    let loading = store.begin_initialization_directory(&run, "800").unwrap();
    let first = store
        .claim_initialization_directories(&run, 4, None)
        .unwrap();
    assert!(first.jobs.is_empty());
    assert_eq!(first.stats.examined, 64);
    assert_eq!(first.stats.blocked_loading, 64);
    assert!(first.next.is_some());
    let second = store
        .claim_initialization_directories(&run, 4, first.next.as_ref())
        .unwrap();
    assert_eq!(second.jobs.len(), 1);
    assert_eq!(second.jobs[0].node.id, "999");
    assert!(second.next.is_none());
    assert!(
        store
            .initialization_directory_ticket_current(&loading)
            .unwrap()
    );
    // An indexed scan end has no authority to complete outstanding coverage.
    let fence = store.begin_initialization_fence(&run).unwrap();
    assert!(
        store
            .confirm_initialization_fence(&fence, fence.cursor())
            .is_err()
    );
}

#[test]
fn claim_failure_rolls_back_all_issued_tickets_and_clock() {
    let (_temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &[dir("200", "100"), dir("300", "100")],
    );
    let before = store.initialization_coverage(&run).unwrap();
    let clock: i64 = store
        .db
        .query_row("SELECT directory_clock FROM initialization_run", [], |r| {
            r.get(0)
        })
        .unwrap();
    // Second candidate is disconnected, after the first would have been claimed.
    let broken = dir("300", "absent");
    store
        .db
        .execute(
            "UPDATE initialization_entries SET body=?1 WHERE id='300'",
            [serde_json::to_string(&broken).unwrap()],
        )
        .unwrap();
    assert!(
        store
            .claim_initialization_directories(&run, 4, None)
            .is_err()
    );
    assert_eq!(store.initialization_coverage(&run).unwrap(), before);
    let after: i64 = store
        .db
        .query_row("SELECT directory_clock FROM initialization_run", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(after, clock);
}

#[test]
fn generation_overflow_and_denied_ancestry_never_partially_claim_or_clear_fence() {
    for denied in [false, true] {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        list(
            &mut store,
            &run,
            "100",
            &[dir("200", "100"), dir("900", "100")],
        );
        list(&mut store, &run, "900", &[dir("300", "900")]);
        if denied {
            let ticket = store.begin_initialization_directory(&run, "900").unwrap();
            store.store_initialization_denied(&ticket, 1).unwrap();
            // This malformed queue cannot manufacture permission for its child.
            // Denied listing normally removes its children; reconstruct only a
            // private corrupt candidate to exercise the scheduler's guard.
            insert_node(&store.db, &run.run_id, &dir("300", "900")).unwrap();
        } else {
            store
                .db
                .execute(
                    "UPDATE initialization_run SET directory_clock=?1",
                    [i64::MAX - 1],
                )
                .unwrap();
        }
        let fence = store.begin_initialization_fence(&run).unwrap();
        let before = store.initialization_coverage(&run).unwrap();
        assert!(
            store
                .claim_initialization_directories(&run, 4, None)
                .is_err()
        );
        assert_eq!(store.initialization_coverage(&run).unwrap(), before);
        let retained: Option<u64> = store
            .db
            .query_row("SELECT fence_ticket FROM initialization_run", [], |r| {
                read_optional_counter(r, 0)
            })
            .unwrap();
        assert_eq!(retained, Some(fence.generation));
        assert!(store.initialization_writeback_blocked().unwrap());
    }
}

#[test]
fn claim_cursor_cannot_cross_reopen_epoch_and_no_claim_preserves_fence() {
    let (temp, mut store, binding) = setup();
    let run = start(&mut store, &binding);
    list(
        &mut store,
        &run,
        "100",
        &(0..3)
            .map(|i| dir(&format!("2{i}"), "100"))
            .collect::<Vec<_>>(),
    );
    let cursor = store
        .claim_initialization_directories(&run, 1, None)
        .unwrap()
        .next
        .unwrap();
    drop(store);
    let mut store = Store::open(temp.path()).unwrap();
    let recovered = store.initialization_status().unwrap().unwrap().run;
    assert!(
        store
            .claim_initialization_directories(&recovered, 1, Some(&cursor))
            .is_err()
    );
    assert!(
        store
            .claim_initialization_directories(&run, 1, None)
            .is_err()
    );
    let page = store
        .claim_initialization_directories(&recovered, 16, None)
        .unwrap();
    for job in page.jobs {
        store
            .store_initialization_directory(&job.ticket, 1, &[])
            .unwrap();
    }
    let fence = store.begin_initialization_fence(&recovered).unwrap();
    let empty = store
        .claim_initialization_directories(&recovered, 1, None)
        .unwrap();
    assert!(empty.jobs.is_empty());
    assert!(empty.next.is_none());
    // No mutation: the actual issued fence stays valid.
    store
        .confirm_initialization_fence(&fence, fence.cursor())
        .unwrap();
}

#[test]
fn pending_index_vm_work_does_not_scale_with_completed_candidate_rows() {
    fn work(completed: usize, next: bool) -> i32 {
        let (_temp, mut store, binding) = setup();
        let run = start(&mut store, &binding);
        let children = (0..completed + 100)
            .map(|i| dir(&format!("item-{i:06}"), "100"))
            .collect::<Vec<_>>();
        list(&mut store, &run, "100", &children);
        // Queue-shape cost fixture only: no fabricated completion is installed.
        store
            .db
            .execute(
                "UPDATE initialization_directories SET state='received' WHERE run=?1 AND id<?2",
                params![run.run_id, format!("item-{completed:06}")],
            )
            .unwrap();
        let mut statement = store
            .db
            .prepare(if next { NEXT_PAGE } else { FIRST_PAGE })
            .unwrap();
        {
            let mut rows = if next {
                statement
                    .query(params![
                        run.run_id,
                        format!("item-{:06}", completed + 20),
                        65
                    ])
                    .unwrap()
            } else {
                statement.query(params![run.run_id, 65]).unwrap()
            };
            let mut count = 0;
            while rows.next().unwrap().is_some() {
                count += 1;
            }
            assert_eq!(count, 65);
        }
        statement.get_status(StatementStatus::VmStep)
    }
    for next in [false, true] {
        let small = work(128, next);
        let large = work(4096, next);
        assert!(
            large <= small + 32,
            "fixed pending page scanned completed rows: {small} -> {large}"
        );
    }
}
