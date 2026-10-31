use super::*;
use crate::native::tests::{batch, event, scope};

fn entry(id: &str, parent: &str, path: &str, directory: bool, version: u64) -> Node {
    let mut e = event();
    e.file_id = id.into();
    e.parent_id = parent.into();
    e.path = path.into();
    e.file_type = if directory {
        FileType::Dir
    } else {
        FileType::File
    };
    e.version.sync_id = version;
    e.version.max_id = version;
    match translate(&scope(), "server", "", batch(e, 10))
        .unwrap()
        .unwrap()
        .events
        .remove(0)
        .change
    {
        Change::Upsert(node) => node,
        _ => panic!("fixture must upsert"),
    }
}
fn root() -> Node {
    entry("root", "outside", "/lab/run", true, 1)
}
fn directory(id: &str) -> Node {
    entry(id, "root", &format!("/lab/run/{id}"), true, 1)
}
fn file(id: &str, parent: &str, name: &str, version: u64) -> Node {
    entry(
        id,
        parent,
        &format!("/lab/run/{parent}/{name}"),
        false,
        version,
    )
}
fn initial() -> RemoteSnapshot {
    RemoteSnapshot {
        scope: scope(),
        restore: "server".into(),
        cursor: "native-v1:10".into(),
        nodes: vec![
            root(),
            directory("left"),
            directory("right"),
            file("a", "left", "A.doc", 5),
            file("z", "right", "Z.doc", 1),
        ],
        inaccessible: vec![],
        skipped_symlinks: 0,
    }
}
fn changed() -> NativeEvent {
    let mut e = event();
    e.path = "/lab/run/left/A.doc".into();
    e.parent_id = "left".into();
    e
}
fn empty(next: u64) -> EventBatch {
    EventBatch {
        events: vec![],
        next: EventCursor(next),
        recovery_required: false,
    }
}

#[tokio::test]
async fn changing_file_rechecks_only_its_parent_and_fences_after_every_recheck() {
    let mut pages = VecDeque::from([
        batch(changed(), 11),
        batch(changed(), 12),
        empty(13),
        empty(13),
    ]);
    let mut cursors = vec![];
    let mut paths = vec![];
    let mut replies = VecDeque::from([
        vec![file("a", "left", "A.doc", 6)],
        vec![file("a", "left", "A.doc", 7)],
    ]);
    let result = converge(
        initial(),
        1,
        |cursor| {
            cursors.push(cursor.0);
            std::future::ready(Ok(pages.pop_front().unwrap()))
        },
        |id, path| {
            paths.push((id, path));
            std::future::ready(Ok(replies.pop_front().unwrap()))
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(cursors, [10, 11, 12, 13]);
    assert_eq!(paths, vec![("left".into(), "/lab/run/left".into()); 2]);
    assert_eq!(result.cursor, "native-v1:13");
    assert_eq!(
        result.nodes.iter().find(|n| n.id == "a"),
        Some(&file("a", "left", "A.doc", 7))
    );
    assert_eq!(
        result.nodes.iter().find(|n| n.id == "z"),
        Some(&file("z", "right", "Z.doc", 1))
    );
}

#[tokio::test]
async fn cross_parent_rename_replaces_both_direct_lists_as_one_private_update() {
    let mut moved = changed();
    moved.kind = EventKind::Move {
        destination: "/lab/run/right/B.doc".into(),
        parent_id: "right".into(),
    };
    let mut pages = VecDeque::from([batch(moved, 11), empty(11)]);
    let mut paths = vec![];
    let snapshot = converge(
        initial(),
        1,
        |_| std::future::ready(Ok(pages.pop_front().unwrap())),
        |id, path| {
            paths.push(path);
            std::future::ready(Ok(if id == "left" {
                vec![]
            } else {
                vec![
                    file("a", "right", "B.doc", 6),
                    file("z", "right", "Z.doc", 1),
                ]
            }))
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(paths, ["/lab/run/left", "/lab/run/right"]);
    assert_eq!(snapshot.nodes.iter().filter(|n| n.id == "a").count(), 1);
    assert_eq!(
        snapshot
            .nodes
            .iter()
            .find(|n| n.id == "a")
            .unwrap()
            .parent
            .as_deref(),
        Some("right")
    );
}

#[tokio::test]
async fn deletion_same_name_replacement_uses_current_listing_not_historical_event_metadata() {
    let mut deleted = changed();
    deleted.kind = EventKind::Remove;
    deleted.removed = true;
    let mut replacement = changed();
    replacement.file_id = "b".into();
    let mut pages = VecDeque::from([
        EventBatch {
            events: vec![deleted, replacement],
            next: EventCursor(12),
            recovery_required: false,
        },
        empty(12),
    ]);
    let mut calls = 0;
    let snapshot = converge(
        initial(),
        1,
        |_| std::future::ready(Ok(pages.pop_front().unwrap())),
        |_, _| {
            calls += 1;
            std::future::ready(Ok(vec![file("b", "left", "A.doc", 8)]))
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(calls, 1);
    assert!(!snapshot.nodes.iter().any(|n| n.id == "a"));
    assert_eq!(
        snapshot.nodes.iter().find(|n| n.id == "b"),
        Some(&file("b", "left", "A.doc", 8))
    );
}

#[tokio::test]
async fn leaf_delete_and_moves_across_scope_recheck_only_known_in_scope_parents() {
    for mode in 0..3 {
        let mut e = changed();
        let expected = match mode {
            0 => {
                e.removed = true;
                e.kind = EventKind::Remove;
                vec![]
            }
            1 => {
                e.kind = EventKind::Move {
                    destination: "/elsewhere/A.doc".into(),
                    parent_id: "outside".into(),
                };
                vec![]
            }
            _ => {
                e.path = "/elsewhere/new.doc".into();
                e.parent_id = "outside".into();
                e.file_id = "new".into();
                e.kind = EventKind::Move {
                    destination: "/lab/run/left/new.doc".into(),
                    parent_id: "left".into(),
                };
                vec![
                    file("a", "left", "A.doc", 5),
                    file("new", "left", "new.doc", 8),
                ]
            }
        };
        let mut pages = VecDeque::from([batch(e, 11), empty(11)]);
        let snapshot = converge(
            initial(),
            1,
            |_| std::future::ready(Ok(pages.pop_front().unwrap())),
            |id, _| {
                assert_eq!(id, "left");
                std::future::ready(Ok(expected.clone()))
            },
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(
            snapshot
                .nodes
                .iter()
                .filter(|n| n.parent.as_deref() == Some("left"))
                .count(),
            expected.len()
        );
    }
}

#[tokio::test]
async fn directory_changes_moves_acl_and_missing_parents_retain_full_retry() {
    for mode in 0..6 {
        let mut e = changed();
        match mode {
            0 => e.file_type = FileType::Dir,
            1 => {
                e.file_id = "left".into();
                e.file_type = FileType::Dir;
                e.kind = EventKind::Remove;
            }
            2 => {
                e.file_id = "left".into();
                e.file_type = FileType::Dir;
                e.kind = EventKind::Move {
                    destination: "/lab/run/renamed".into(),
                    parent_id: "root".into(),
                };
            }
            3 => e.parent_id = "unknown".into(),
            4 => e.path = "/lab/run/wrong/A.doc".into(),
            _ => e.file_id = "left".into(), // Contradictory file metadata for a known directory.
        }
        let mut page = Some(batch(e, 11));
        assert!(
            converge(
                initial(),
                1,
                |_| std::future::ready(Ok(page.take().unwrap())),
                |_, _| {
                    panic!("unsafe event must not start local queries");
                    #[allow(unreachable_code)]
                    std::future::ready(Ok(vec![]))
                }
            )
            .await
            .unwrap()
            .is_none()
        );
    }
    let mut snapshot = initial();
    snapshot.inaccessible.push("left".into());
    assert!(
        converge(
            snapshot,
            1,
            |_| std::future::ready(Ok(batch(changed(), 11))),
            |_, _| {
                panic!("inaccessible parent must not use leaf repair");
                #[allow(unreachable_code)]
                std::future::ready(Ok(vec![]))
            }
        )
        .await
        .unwrap()
        .is_none()
    );
}

#[tokio::test]
async fn listing_new_removed_or_changed_directory_cannot_silently_drop_descendants() {
    for mode in 0..3 {
        let snapshot = initial();
        let mut e = event();
        e.file_id = "new".into();
        let mut dirs = vec![directory("left"), directory("right")];
        match mode {
            0 => dirs.push(directory("newdir")),
            1 => {
                dirs.remove(0);
            }
            _ => dirs[0].metadata.as_mut().unwrap().mtime += 1,
        }
        assert!(
            converge(
                snapshot,
                1,
                |_| std::future::ready(Ok(batch(e.clone(), 11))),
                |_, _| std::future::ready(Ok(dirs.clone()))
            )
            .await
            .unwrap()
            .is_none()
        );
    }
}

#[tokio::test]
async fn duplicate_names_identities_and_cross_unqueried_parent_replacements_fall_back() {
    for listing in [
        vec![file("a", "left", "A.doc", 6), file("b", "left", "A.doc", 6)],
        vec![file("a", "left", "A.doc", 6), file("a", "left", "B.doc", 6)],
        vec![file("z", "left", "Z.doc", 6)],
        vec![file("a", "right", "A.doc", 6)],
    ] {
        assert!(
            converge(
                initial(),
                1,
                |_| std::future::ready(Ok(batch(changed(), 11))),
                |_, _| std::future::ready(Ok(listing.clone()))
            )
            .await
            .unwrap()
            .is_none()
        );
    }
}

#[tokio::test]
async fn an_intermediate_repair_is_not_published_when_next_fence_is_unsafe_or_fails() {
    for mode in 0..3 {
        let mut e = changed();
        e.file_type = FileType::Dir;
        let mut pages = VecDeque::from([
            Ok(batch(changed(), 11)),
            match mode {
                0 => Ok(batch(e, 12)),
                1 => Err(Error::State("synthetic transport".into())),
                _ => Ok(EventBatch {
                    recovery_required: true,
                    ..empty(12)
                }),
            },
        ]);
        let result = converge(
            initial(),
            1,
            |_| std::future::ready(pages.pop_front().unwrap()),
            |_, _| std::future::ready(Ok(vec![])),
        )
        .await;
        if mode == 0 {
            assert!(result.unwrap().is_none());
        } else {
            assert!(result.is_err());
        }
    }
}

#[tokio::test]
async fn permission_denial_retries_but_transport_errors_keep_their_category() {
    let result = converge(
        initial(),
        1,
        |_| std::future::ready(Ok(batch(changed(), 11))),
        |_, _| {
            std::future::ready(Err(Error::Native(sproto::Error::Server {
                code: 12290,
                reason: "synthetic".into(),
            })))
        },
    )
    .await;
    assert!(result.unwrap().is_none());
    let result = converge(
        initial(),
        1,
        |_| std::future::ready(Ok(batch(changed(), 11))),
        |_, _| std::future::ready(Err(Error::State("synthetic transport".into()))),
    )
    .await;
    assert!(matches!(result,Err(Error::State(message)) if message=="synthetic transport"));
}

#[tokio::test]
async fn stationary_nonempty_backwards_root_loss_and_global_recovery_are_rejected() {
    for page in [
        batch(changed(), 10),
        empty(9),
        EventBatch {
            recovery_required: true,
            ..empty(11)
        },
    ] {
        let mut page = Some(page);
        assert!(
            converge(
                initial(),
                1,
                |_| std::future::ready(Ok(page.take().unwrap())),
                |_, _| std::future::ready(Ok(vec![]))
            )
            .await
            .is_err()
        );
    }
    let mut e = changed();
    e.file_id = "root".into();
    e.kind = EventKind::Remove;
    assert!(matches!(
        converge(
            initial(),
            1,
            |_| std::future::ready(Ok(batch(e.clone(), 11))),
            |_, _| std::future::ready(Ok(vec![]))
        )
        .await,
        Err(Error::RootUnavailable)
    ));
}

#[tokio::test]
async fn advancing_empty_pages_and_continuous_local_work_share_the_original_bound() {
    for active in [false, true] {
        let mut calls = 0;
        let result = converge(
            initial(),
            1,
            |cursor| {
                calls += 1;
                std::future::ready(Ok(if active {
                    batch(changed(), cursor.0 + 1)
                } else {
                    empty(cursor.0 + 1)
                }))
            },
            |_, _| std::future::ready(Ok(vec![file("a", "left", "A.doc", 6)])),
        )
        .await;
        assert!(
            matches!(result,Err(Error::State(message)) if message=="snapshot convergence did not reach boundary")
        );
        assert_eq!(calls, PAGE_LIMIT);
    }
}

#[tokio::test]
async fn stationary_success_preserves_exclusions_and_never_requeries_the_tree() {
    let mut snapshot = initial();
    snapshot.inaccessible.push("right".into());
    snapshot.skipped_symlinks = 3;
    let output = converge(
        snapshot,
        1,
        |_| std::future::ready(Ok(empty(10))),
        |_, _| {
            panic!("stationary fence needs no listing");
            #[allow(unreachable_code)]
            std::future::ready(Ok(vec![]))
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(output.cursor, "native-v1:10");
    assert_eq!(output.inaccessible, ["right"]);
    assert_eq!(output.skipped_symlinks, 3);
}

#[tokio::test]
async fn cancellation_while_rechecking_does_not_return_a_candidate_or_advance_any_cursor() {
    use std::future::Future;
    use std::task::Poll;
    let mut cursors = vec![];
    {
        let future = converge(
            initial(),
            1,
            |cursor| {
                cursors.push(cursor.0);
                std::future::ready(Ok(batch(changed(), 11)))
            },
            |_, _| std::future::pending::<Result<Vec<Node>>>(),
        );
        let mut future = std::pin::pin!(future);
        std::future::poll_fn(|cx| {
            assert!(future.as_mut().poll(cx).is_pending());
            Poll::Ready(())
        })
        .await;
        // Dropping the pending operation owns and discards the only candidate.
    }
    assert_eq!(cursors, [10]);
}

#[test]
fn deepest_enumerable_file_is_valid_but_cycles_and_wrong_scope_tokens_are_not() {
    let mut snapshot = initial();
    snapshot.nodes = vec![root()];
    let mut parent = "root".to_owned();
    for depth in 1..=128 {
        let id = format!("d{depth}");
        snapshot
            .nodes
            .push(entry(&id, &parent, &format!("/lab/run/{id}"), true, 1));
        parent = id;
    }
    snapshot.nodes.push(file("leaf", &parent, "leaf", 1));
    assert!(Candidate::new(snapshot).is_ok());
    let mut snapshot = initial();
    snapshot.nodes[1].parent = Some("right".into());
    snapshot.nodes[2].parent = Some("left".into());
    assert!(Candidate::new(snapshot).is_err());
    let mut snapshot = initial();
    snapshot.restore = "replacement-server".into();
    assert!(Candidate::new(snapshot).is_err());
}

#[tokio::test]
async fn historical_move_rechecks_third_parent_where_enumeration_already_saw_file() {
    let mut snapshot = initial();
    snapshot.nodes.push(directory("third"));
    *snapshot.nodes.iter_mut().find(|n| n.id == "a").unwrap() = file("a", "third", "C.doc", 9);
    let mut moved = changed();
    moved.kind = EventKind::Move {
        destination: "/lab/run/right/B.doc".into(),
        parent_id: "right".into(),
    };
    let mut pages = VecDeque::from([batch(moved, 11), empty(11)]);
    let mut queried = vec![];
    let result = converge(
        snapshot,
        1,
        |_| std::future::ready(Ok(pages.pop_front().unwrap())),
        |id, _| {
            queried.push(id.clone());
            std::future::ready(Ok(match id.as_str() {
                "third" => vec![file("a", "third", "C.doc", 9)],
                "right" => vec![file("z", "right", "Z.doc", 1)],
                _ => vec![],
            }))
        },
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(queried, ["left", "right", "third"]);
    assert_eq!(
        result.nodes.iter().find(|n| n.id == "a"),
        Some(&file("a", "third", "C.doc", 9))
    );
}

#[tokio::test]
async fn future_current_listing_is_not_downgraded_by_later_old_event_metadata() {
    let mut old = changed();
    old.version.sync_id = 6;
    old.version.max_id = 6;
    let mut pages = VecDeque::from([batch(changed(), 11), batch(old, 12), empty(12)]);
    let result = converge(
        initial(),
        1,
        |_| std::future::ready(Ok(pages.pop_front().unwrap())),
        |_, _| std::future::ready(Ok(vec![file("a", "left", "A.doc", 20)])),
    )
    .await
    .unwrap()
    .unwrap();
    assert_eq!(
        result.nodes.iter().find(|n| n.id == "a"),
        Some(&file("a", "left", "A.doc", 20))
    );
    assert_eq!(result.cursor, "native-v1:12");
}

#[tokio::test]
async fn absolute_deadline_cancels_pending_listing_and_fetch() {
    use std::sync::{
        Arc,
        atomic::{AtomicBool, Ordering},
    };
    struct Pending(Arc<AtomicBool>);
    impl std::future::Future for Pending {
        type Output = Result<Vec<Node>>;
        fn poll(
            self: std::pin::Pin<&mut Self>,
            _: &mut std::task::Context<'_>,
        ) -> std::task::Poll<Self::Output> {
            std::task::Poll::Pending
        }
    }
    impl Drop for Pending {
        fn drop(&mut self) {
            self.0.store(true, Ordering::Release);
        }
    }
    let dropped = Arc::new(AtomicBool::new(false));
    let result = converge_until(
        initial(),
        1,
        |_| std::future::ready(Ok(batch(changed(), 11))),
        |_, _| Pending(dropped.clone()),
        Instant::now() + Duration::from_millis(50),
    )
    .await;
    assert!(
        matches!(result,Err(Error::State(message)) if message=="snapshot convergence time budget exceeded")
    );
    assert!(dropped.load(Ordering::Acquire));
    let result = converge_until(
        initial(),
        1,
        |_| std::future::pending::<Result<EventBatch>>(),
        |_, _| std::future::ready(Ok(vec![])),
        Instant::now() + Duration::from_millis(1),
    )
    .await;
    assert!(
        matches!(result,Err(Error::State(message)) if message=="snapshot convergence time budget exceeded")
    );
}

#[tokio::test]
async fn reconciled_snapshot_still_uses_transactional_install_and_protects_dirty_draft() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let snapshot = initial();
    store
        .bind_native_scope("server", &serde_json::to_string(&scope()).unwrap())
        .unwrap();
    store.install_native_snapshot(&snapshot, "").unwrap();
    let version = store.node("a").unwrap().version;
    let draft = store.begin_draft("a", &version, None).unwrap();
    store
        .draft_write(&draft, 0, b"offline draft", false)
        .unwrap();
    store.checkpoint_draft(&draft).unwrap();
    let pending = store.pending_operations().unwrap();
    assert!(pending > 0);
    let mut failed_pages = VecDeque::from([
        Ok(batch(changed(), 11)),
        Err(Error::State("fence interrupted".into())),
    ]);
    assert!(
        converge(
            snapshot.clone(),
            1,
            |_| std::future::ready(failed_pages.pop_front().unwrap()),
            |_, _| std::future::ready(Ok(vec![]))
        )
        .await
        .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:10");
    assert_eq!(store.pending_operations().unwrap(), pending);
    let mut pages = VecDeque::from([batch(changed(), 11), empty(11)]);
    let ready = converge(
        snapshot,
        1,
        |_| std::future::ready(Ok(pages.pop_front().unwrap())),
        |_, _| std::future::ready(Ok(vec![file("a", "left", "A.doc", 6)])),
    )
    .await
    .unwrap()
    .unwrap();
    assert!(
        store
            .install_native_snapshot(&ready, "native-v1:9")
            .is_err()
    );
    assert_eq!(store.cursor().unwrap(), "native-v1:10");
    assert_eq!(store.pending_operations().unwrap(), pending);
    store
        .install_native_snapshot(&ready, "native-v1:10")
        .unwrap();
    assert_eq!(store.cursor().unwrap(), "native-v1:11");
    assert!(store.draft_conflicts("a").unwrap());
    drop(draft);
    drop(store);
    let store = Store::open(temp.path()).unwrap();
    let draft = store.open_dirty_draft("a").unwrap().unwrap();
    let mut body = [0u8; 32];
    let count = draft.read_at(&mut body, 0).unwrap();
    assert_eq!(&body[..count], b"offline draft");
    assert!(store.draft_conflicts("a").unwrap());
    assert_eq!(store.cursor().unwrap(), "native-v1:11");
}
