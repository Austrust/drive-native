use super::super::tests::{batch, event, scope};
use super::*;

fn collision() -> EnumerationConflict {
    let mut identities = HashSet::new();
    let mut first_directory = HashMap::new();
    assert!(remember_child(&mut identities, &mut first_directory, "a", "A.doc").is_none());
    let mut second_directory = HashMap::new();
    remember_child(&mut identities, &mut second_directory, "a", "A.doc").unwrap()
}
fn error() -> Error {
    Error::State("snapshot identity or name changed during enumeration".into())
}
fn changed(conflict: &EnumerationConflict, observed: &EventBatch) -> bool {
    matches!(
        classify_enumeration_error(error(), conflict, &scope(), EventCursor(10), observed),
        Error::SnapshotChanged
    )
}

#[test]
fn empty_global_advance_and_unrelated_events_cannot_explain_collision() {
    let conflict = collision();
    assert!(!changed(
        &conflict,
        &EventBatch {
            events: vec![],
            next: EventCursor(11),
            recovery_required: false
        }
    ));
    let mut unrelated = event();
    unrelated.file_id = "other".into();
    assert!(!changed(&conflict, &batch(unrelated, 11)));
    let mut outside = event();
    outside.path = "/lab/run-other/A.doc".into();
    assert!(!changed(&conflict, &batch(outside, 11)));
}

#[test]
fn matching_identity_movement_retains_bounded_namespace_race_recovery() {
    let conflict = collision();
    let mut moved = event();
    moved.kind = EventKind::Move {
        destination: "/lab/run/second/A.doc".into(),
        parent_id: "second".into(),
    };
    assert!(changed(&conflict, &batch(moved.clone(), 11)));
    moved.path = "/outside/A.doc".into(); // move into the selected scope
    assert!(changed(&conflict, &batch(moved.clone(), 11)));
    moved.path = "/lab/run/A.doc".into();
    moved.kind = EventKind::Move {
        destination: "/outside/A.doc".into(),
        parent_id: "outside".into(),
    };
    assert!(changed(&conflict, &batch(moved, 11)));
}

#[test]
fn same_name_collision_tracks_both_identities_without_global_tree_copy() {
    let mut identities = HashSet::new();
    let mut names = HashMap::new();
    assert!(remember_child(&mut identities, &mut names, "a", "A.doc").is_none());
    let conflict = remember_child(&mut identities, &mut names, "b", "A.doc").unwrap();
    for id in ["a", "b"] {
        let mut removed = event();
        removed.file_id = id.into();
        removed.removed = true;
        removed.kind = EventKind::Remove;
        assert!(changed(&conflict, &batch(removed, 11)));
    }
    let mut other = event();
    other.file_id = "c".into();
    assert!(!changed(&conflict, &batch(other, 11)));
}

#[test]
fn matching_event_still_requires_forward_cursor_and_no_recovery_signal() {
    let conflict = collision();
    for (next, recovery, expected) in [
        (11, false, true),
        (10, false, false),
        (9, false, false),
        (11, true, false),
    ] {
        let mut observed = batch(event(), next);
        observed.recovery_required = recovery;
        assert_eq!(changed(&conflict, &observed), expected);
    }
}

#[tokio::test]
async fn permission_transport_protocol_capacity_and_root_errors_do_not_probe_or_become_races() {
    let errors = vec![
        Error::Native(sproto::Error::Server {
            code: 12290,
            reason: "synthetic denial".into(),
        }),
        Error::Native(sproto::Error::Timeout("native response")),
        Error::Native(sproto::Error::ConnectionClosed),
        Error::Native(sproto::Error::Decode("synthetic malformed reply".into())),
        Error::State("snapshot node budget exceeded".into()),
        Error::Unsupported("snapshot identity/version/type not supported"),
        Error::RootUnavailable,
        Error::Io(std::io::ErrorKind::Interrupted.into()),
        // Merely matching the former error text is not structural evidence.
        error(),
    ];
    for original in errors {
        let description = original.to_string();
        let discriminant = std::mem::discriminant(&original);
        let observed =
            resolve_enumeration_failure(original, None, &scope(), EventCursor(10), |_| {
                panic!("non-collision failure must not issue an extra Pull");
                #[allow(unreachable_code)]
                std::future::ready(Ok(batch(event(), 11)))
            })
            .await;
        assert_eq!(std::mem::discriminant(&observed), discriminant);
        assert_eq!(observed.to_string(), description);
    }
}

#[tokio::test]
async fn failed_collision_verification_preserves_transport_error() {
    let result = resolve_enumeration_failure(
        error(),
        Some(collision()),
        &scope(),
        EventCursor(10),
        |_| {
            std::future::ready(Err(Error::Native(sproto::Error::Timeout(
                "native response",
            ))))
        },
    )
    .await;
    assert!(matches!(
        result,
        Error::Native(sproto::Error::Timeout("native response"))
    ));
}

#[tokio::test]
async fn verified_collision_is_the_only_path_that_requests_race_retry() {
    let called = std::cell::Cell::new(0);
    let result = resolve_enumeration_failure(
        error(),
        Some(collision()),
        &scope(),
        EventCursor(10),
        |_| {
            called.set(called.get() + 1);
            std::future::ready(Ok(batch(event(), 11)))
        },
    )
    .await;
    assert_eq!(called.get(), 1);
    assert!(matches!(result, Error::SnapshotChanged));
}

#[tokio::test]
async fn related_event_on_later_page_is_not_lost_after_empty_global_advancement() {
    let mut cursors = Vec::new();
    let result = resolve_enumeration_failure(
        error(),
        Some(collision()),
        &scope(),
        EventCursor(10),
        |cursor| {
            cursors.push(cursor.0);
            let observed = if cursor.0 == 10 {
                EventBatch {
                    events: vec![],
                    next: EventCursor(11),
                    recovery_required: false,
                }
            } else {
                batch(event(), 12)
            };
            std::future::ready(Ok(observed))
        },
    )
    .await;
    assert_eq!(cursors, [10, 11]);
    assert!(matches!(result, Error::SnapshotChanged));
}

#[tokio::test]
async fn unrelated_pages_stop_at_boundary_and_never_authorize_retry() {
    let mut calls = 0;
    let original = error().to_string();
    let result = resolve_enumeration_failure(
        error(),
        Some(collision()),
        &scope(),
        EventCursor(10),
        |cursor| {
            calls += 1;
            let observed = if calls == 1 {
                let mut unrelated = event();
                unrelated.file_id = "other".into();
                batch(unrelated, 11)
            } else {
                EventBatch {
                    events: vec![],
                    next: cursor,
                    recovery_required: false,
                }
            };
            std::future::ready(Ok(observed))
        },
    )
    .await;
    assert_eq!(calls, 2);
    assert_eq!(result.to_string(), original);
}

#[tokio::test]
async fn incomplete_collision_probe_is_bounded_and_keeps_original_failure() {
    let mut calls = 0;
    let original = error().to_string();
    let result = resolve_enumeration_failure(
        error(),
        Some(collision()),
        &scope(),
        EventCursor(10),
        |cursor| {
            calls += 1;
            std::future::ready(Ok(EventBatch {
                events: vec![],
                next: EventCursor(cursor.0 + 1),
                recovery_required: false,
            }))
        },
    )
    .await;
    assert_eq!(calls, 4096);
    assert_eq!(result.to_string(), original);
}
