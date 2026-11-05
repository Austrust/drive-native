use super::super::tests::{batch, event, scope};
use super::*;

fn empty(next: u64) -> EventBatch {
    EventBatch {
        events: vec![],
        next: EventCursor(next),
        recovery_required: false,
    }
}
fn root(next: u64, version: u64) -> EventBatch {
    let mut event = event();
    event.file_id = scope().file_id;
    event.path = scope().path;
    event.parent_id = "outside".into();
    event.file_type = FileType::Dir;
    event.size = 0;
    event.hash.clear();
    event.version.sync_id = version;
    event.version.max_id = version;
    batch(event, next)
}
async fn seed() -> SnapshotSeed {
    let mut pages = VecDeque::from([root(10, 5), empty(15), empty(15)]);
    let mut requested = Vec::new();
    let result = collect_snapshot_seed(&scope(), "server", None, 1, |cursor| {
        requested.push(cursor.0);
        std::future::ready(Ok(pages.pop_front().unwrap()))
    })
    .await
    .unwrap();
    assert_eq!(requested, [0, 10, 15]);
    assert_eq!(result.cursor, EventCursor(15));
    result
}

#[tokio::test]
async fn retry_starts_at_completed_seed_and_reduces_root_updates_before_new_boundary() {
    let old = seed().await;
    let mut pages = VecDeque::from([root(20, 9), empty(22), empty(22)]);
    let mut requested = Vec::new();
    let next = collect_snapshot_seed(&scope(), "server", Some(&old), 2, |cursor| {
        requested.push(cursor.0);
        std::future::ready(Ok(pages.pop_front().unwrap()))
    })
    .await
    .unwrap();
    assert_eq!(requested, [15, 20, 22]);
    assert_eq!(next.cursor, EventCursor(22));
    assert_ne!(old.root.version, next.root.version);
    assert_eq!(next.root.id, old.root.id);
    assert_eq!(old.cursor, EventCursor(15));
}

#[tokio::test]
async fn unchanged_cached_seed_and_normalized_return_keep_the_selected_identity() {
    let old = seed().await;
    let next = collect_snapshot_seed(&scope(), "server", Some(&old), 2, |cursor| {
        assert_eq!(cursor, old.cursor);
        std::future::ready(Ok(empty(cursor.0)))
    })
    .await
    .unwrap();
    assert_eq!(next.cursor, old.cursor);
    assert_eq!(next.root, old.root);
    // scoped_events has already removed a verified historical move-away. Its
    // remaining return event still carries the original path outside scope.
    let mut returned = root(25, 10);
    returned.events[0].path = "/lab/temporary".into();
    returned.events[0].kind = EventKind::Move {
        destination: scope().path,
        parent_id: "outside".into(),
    };
    let mut pages = VecDeque::from([returned, empty(25)]);
    let restored = collect_snapshot_seed(&scope(), "server", Some(&old), 3, |_| {
        std::future::ready(Ok(pages.pop_front().unwrap()))
    })
    .await
    .unwrap();
    assert_eq!(restored.root.id, old.root.id);
    assert_eq!(restored.cursor, EventCursor(25));
}

#[tokio::test]
async fn root_deletion_replacement_and_type_change_abort_cached_seed() {
    let old = seed().await;
    for kind in 0..3 {
        let mut page = root(20, 9);
        let event = &mut page.events[0];
        match kind {
            0 => {
                event.removed = true;
                event.kind = EventKind::Remove;
            }
            1 => event.file_id = "replacement".into(),
            _ => event.file_type = FileType::File,
        }
        let mut page = Some(page);
        let result = collect_snapshot_seed(&scope(), "server", Some(&old), 2, |_| {
            std::future::ready(Ok(page.take().unwrap()))
        })
        .await;
        assert!(matches!(result, Err(Error::RootUnavailable)));
        assert_eq!(old.cursor, EventCursor(15));
    }
}

#[tokio::test]
async fn interrupted_or_invalid_reseed_cannot_publish_partial_root_and_cursor() {
    let old = seed().await;
    let initial_root = old.root.clone();
    let mut pages = VecDeque::from([
        Ok(root(20, 9)),
        Err(Error::State("synthetic transport failure".into())),
    ]);
    assert!(
        collect_snapshot_seed(&scope(), "server", Some(&old), 2, |_| {
            std::future::ready(pages.pop_front().unwrap())
        })
        .await
        .is_err()
    );
    assert_eq!(old.root, initial_root);
    assert_eq!(old.cursor, EventCursor(15));
    for page in [
        empty(14),
        root(15, 9),
        EventBatch {
            recovery_required: true,
            ..empty(20)
        },
    ] {
        let mut page = Some(page);
        assert!(
            collect_snapshot_seed(&scope(), "server", Some(&old), 2, |_| {
                std::future::ready(Ok(page.take().unwrap()))
            })
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn seed_binding_cannot_cross_scope_or_server_identity() {
    let old = seed().await;
    let mut different_scope = scope();
    different_scope.view_id += 1;
    for (scope, restore) in [(different_scope, "server"), (scope(), "another-server")] {
        assert!(
            collect_snapshot_seed(&scope, restore, Some(&old), 2, |_| {
                panic!("mismatched seed must be rejected before fetching");
                #[allow(unreachable_code)]
                std::future::ready(Ok(empty(15)))
            })
            .await
            .is_err()
        );
    }
}

#[tokio::test]
async fn fresh_seed_still_requires_selected_root_and_stationary_empty_boundary() {
    assert!(
        collect_snapshot_seed(&scope(), "server", None, 1, |_| {
            std::future::ready(Ok(empty(0)))
        })
        .await
        .is_err()
    );
    let old = seed().await;
    let mut count = 0;
    assert!(
        collect_snapshot_seed(&scope(), "server", Some(&old), 2, |cursor| {
            count += 1;
            std::future::ready(Ok(empty(cursor.0 + 1)))
        })
        .await
        .is_err()
    );
    assert_eq!(count, 4096);
}

#[tokio::test]
async fn cancelled_reseed_keeps_only_the_previous_completed_boundary() {
    let old = seed().await;
    let original_root = old.root.clone();
    let result = tokio::time::timeout(
        std::time::Duration::from_millis(10),
        collect_snapshot_seed(&scope(), "server", Some(&old), 2, |_| {
            std::future::pending::<Result<EventBatch>>()
        }),
    )
    .await;
    assert!(result.is_err());
    assert_eq!(old.cursor, EventCursor(15));
    assert_eq!(old.root, original_root);
}


#[tokio::test]
async fn seed_retries_transport_failures_from_the_same_cursor_then_completes() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    let page = Rc::new(Cell::new(0u32));
    let attempts = Rc::new(Cell::new(0u32));
    let attempted_cursors = Rc::new(RefCell::new(Vec::<u64>::new()));
    let page_outer = page.clone();
    let attempts_inner = attempts.clone();
    let cursors_inner = attempted_cursors.clone();
    let result = collect_snapshot_seed(&scope(), "server", None, 1, |cursor| {
        page_outer.set(page_outer.get() + 1);
        let this_page = page_outer.get();
        let attempts = attempts_inner.clone();
        let cursors = cursors_inner.clone();
        async move {
            super::super::retry::read(move || {
                attempts.set(attempts.get() + 1);
                cursors.borrow_mut().push(cursor.0);
                std::future::ready(if this_page == 2 && attempts.get() <= 4 {
                    Err(sproto::Error::Io(std::io::Error::from(
                        std::io::ErrorKind::UnexpectedEof,
                    )))
                } else if this_page == 1 {
                    Ok(root(2000, 5))
                } else {
                    Ok(empty(4000))
                })
            })
            .await
            .map_err(Error::Native)
        }
    })
    .await
    .unwrap();
    assert_eq!(result.cursor, EventCursor(4000));
    assert_eq!(
        attempted_cursors.borrow().as_slice(),
        &[0, 2000, 2000, 2000, 2000, 4000]
    );
    assert_eq!(attempts.get(), 6);
}

#[tokio::test]
async fn seed_stops_after_bounded_retries_without_skipping_a_page() {
    use std::cell::{Cell, RefCell};
    use std::rc::Rc;
    let page = Rc::new(Cell::new(0u32));
    let attempts = Rc::new(Cell::new(0u32));
    let attempted_cursors = Rc::new(RefCell::new(Vec::<u64>::new()));
    let page_outer = page.clone();
    let attempts_inner = attempts.clone();
    let cursors_inner = attempted_cursors.clone();
    let result = collect_snapshot_seed(&scope(), "server", None, 1, |cursor| {
        page_outer.set(page_outer.get() + 1);
        let this_page = page_outer.get();
        let attempts = attempts_inner.clone();
        let cursors = cursors_inner.clone();
        async move {
            super::super::retry::read(move || {
                attempts.set(attempts.get() + 1);
                cursors.borrow_mut().push(cursor.0);
                std::future::ready(if this_page == 1 {
                    Ok(root(2000, 5))
                } else {
                    Err(sproto::Error::Io(std::io::Error::from(
                        std::io::ErrorKind::UnexpectedEof,
                    )))
                })
            })
            .await
            .map_err(Error::Native)
        }
    })
    .await;
    assert!(result.is_err());
    assert_eq!(attempts.get(), 1 + super::super::retry::MAX_ATTEMPTS);
    assert_eq!(
        attempted_cursors.borrow().as_slice(),
        &[0, 2000, 2000, 2000, 2000, 2000]
    );
}

#[tokio::test]
async fn seed_does_not_retry_a_server_refusal() {
    use std::cell::Cell;
    use std::rc::Rc;
    let page = Rc::new(Cell::new(0u32));
    let attempts = Rc::new(Cell::new(0u32));
    let page_outer = page.clone();
    let attempts_inner = attempts.clone();
    let result = collect_snapshot_seed(&scope(), "server", None, 1, |_cursor| {
        page_outer.set(page_outer.get() + 1);
        let this_page = page_outer.get();
        let attempts = attempts_inner.clone();
        async move {
            super::super::retry::read(move || {
                attempts.set(attempts.get() + 1);
                std::future::ready(if this_page == 1 {
                    Ok(root(2000, 5))
                } else {
                    Err(sproto::Error::Server {
                        code: 12290,
                        reason: "denied".into(),
                    })
                })
            })
            .await
            .map_err(Error::Native)
        }
    })
    .await;
    assert!(result.is_err());
    assert_eq!(attempts.get(), 2);
    assert_eq!(page.get(), 2);
}
