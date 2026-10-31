use super::*;
use crate::download_jobs::DownloadFailureKind;

fn failure(attempts: u32, retry: bool) -> DownloadFailure {
    DownloadFailure {
        schema: 1,
        kind: DownloadFailureKind::Transport,
        server_code: None,
        attempts,
        retry_at: retry.then_some(50),
    }
}
fn facts(own: Status, availability: Availability) -> LeafFacts {
    LeafFacts {
        own,
        inherited_pin: false,
        availability,
        active_downloads: 0,
        failure: None,
    }
}
fn summary(label: Label, offline: bool) -> Summary {
    Summary {
        status: label,
        pinned: false,
        offline_available: offline,
        sync_error: None,
    }
}
fn directory() -> DirectoryAggregate {
    DirectoryAggregate::new(AggregateBudget {
        children: 100,
        bytes: 100 * (CHILD_BYTES + ERROR_BYTES),
    })
}
fn current(d: &DirectoryAggregate) -> Summary {
    d.summary(&Status::OnlineOnly, false).unwrap()
}

#[test]
fn leaf_preserves_version_availability_pin_and_every_transfer_failure_state() {
    for (available, retry, label) in [
        (false, false, Label::DownloadFailed),
        (false, true, Label::DownloadRetry),
        (true, false, Label::UpdateFailed),
        (true, true, Label::UpdateRetry),
    ] {
        let mut input = facts(
            if available {
                Status::Updating
            } else {
                Status::OnlineOnly
            },
            if available {
                Availability::Complete
            } else {
                Availability::Absent
            },
        );
        input.failure = Some(failure(2, retry));
        input.inherited_pin = true;
        let s = leaf(&input).unwrap();
        assert_eq!(s.status, label);
        assert_eq!(s.offline_available, available);
        assert!(s.pinned);
        input.active_downloads = 2;
        let active = leaf(&input).unwrap();
        assert_eq!(
            active.status,
            if available {
                Label::Updating
            } else {
                Label::Downloading
            }
        );
        assert_eq!(active.sync_error, input.failure);
        input.active_downloads = 1;
        assert_eq!(leaf(&input).unwrap(), active); // finishing one version is not all readers finishing
        input.active_downloads = 0;
        assert_eq!(leaf(&input).unwrap(), s);
        input.failure = None;
        assert!(leaf(&input).unwrap().sync_error.is_none());
    }
}

#[test]
fn local_edits_and_conflicts_are_not_overwritten_by_active_downloads_or_failures() {
    for (own, label) in [
        (Status::PendingUpload, Label::PendingUpload),
        (Status::Conflict, Label::Conflict),
    ] {
        let mut input = facts(own, Availability::Complete);
        input.active_downloads = 5;
        input.failure = Some(failure(3, false));
        let s = leaf(&input).unwrap();
        assert_eq!(s.status, label);
        assert!(s.offline_available);
        assert!(s.sync_error.is_none());
    }
}

#[test]
fn unverified_or_contradictory_content_never_reports_complete() {
    assert_eq!(
        leaf(&facts(Status::LocalComplete, Availability::Unverified)),
        Err(AggregateError::UnverifiedContent)
    );
    assert_eq!(
        leaf(&facts(Status::LocalComplete, Availability::Absent)),
        Err(AggregateError::InvalidFacts)
    );
    assert_eq!(
        leaf(&facts(Status::Deleted, Availability::Complete)),
        Err(AggregateError::NotVisible)
    );
    for (own, label) in [
        (Status::OnlineOnly, Label::OnlineOnly),
        (Status::Pinning, Label::Pinning),
        (Status::LocalComplete, Label::LocalComplete),
        (Status::PinnedComplete, Label::PinnedComplete),
    ] {
        let available = matches!(own, Status::LocalComplete | Status::PinnedComplete);
        assert_eq!(
            leaf(&facts(
                own,
                if available {
                    Availability::Complete
                } else {
                    Availability::Absent
                }
            ))
            .unwrap()
            .status,
            label
        );
    }
}

#[test]
fn directory_priority_counts_remove_cleanly_without_stale_high_priority_state() {
    // Distinct children coexist. Removing the highest priority reveals the next
    // rather than retaining a single cached maximum forever.
    let priority = [
        Label::Conflict,
        Label::PendingUpload,
        Label::DownloadFailed,
        Label::DownloadRetry,
        Label::Updating,
        Label::Downloading,
        Label::Pinning,
        Label::Mixed,
        Label::OnlineOnly,
    ];
    let mut d = directory();
    for (id, label) in priority.iter().enumerate() {
        d.set_child(id as u64, Some(ChildState::Known(summary(*label, false))))
            .unwrap();
    }
    for (id, label) in priority.iter().enumerate() {
        assert_eq!(current(&d).status, *label);
        d.set_child(id as u64, None).unwrap();
    }
    assert_eq!(current(&d).status, Label::LocalComplete);
    assert_eq!(d.accounted_bytes(), 0);
    assert_eq!(d.retained_allowance(), 0);
    assert!(d.is_empty());
}

#[test]
fn pin_and_offline_are_independent_and_unknown_children_remain_visible_to_aggregation() {
    let mut d = directory();
    assert_eq!(
        d.summary(&Status::OnlineOnly, true).unwrap().status,
        Label::PinnedComplete
    );
    d.set_child(
        1,
        Some(ChildState::Known(summary(Label::LocalComplete, true))),
    )
    .unwrap();
    d.set_child(
        2,
        Some(ChildState::Known(summary(Label::OnlineOnly, false))),
    )
    .unwrap();
    assert_eq!(current(&d).status, Label::Mixed);
    assert_eq!(
        d.summary(&Status::OnlineOnly, true).unwrap().status,
        Label::Pinning
    );
    d.set_child(2, Some(ChildState::Unverified)).unwrap();
    assert_eq!(
        d.summary(&Status::Conflict, true),
        Err(AggregateError::UnverifiedChildren(1))
    );
    assert_eq!(d.len(), 2);
    d.set_child(
        2,
        Some(ChildState::Known(summary(Label::PinnedComplete, true))),
    )
    .unwrap();
    assert!(current(&d).offline_available);
    assert_eq!(current(&d).status, Label::LocalComplete);
    assert!(!current(&d).pinned); // a pinned child does not pin its parent
}

#[test]
fn errors_have_deterministic_successors_and_retry_clear_removes_details() {
    let mut d = directory();
    for (id, attempts) in [(1, 2), (2, 5), (3, 5)] {
        let mut s = summary(Label::UpdateFailed, true);
        let mut e = failure(attempts, false);
        e.server_code = Some(id as u32);
        s.sync_error = Some(e);
        d.set_child(id, Some(ChildState::Known(s))).unwrap();
    }
    assert_eq!(current(&d).status, Label::UpdateFailed);
    assert_eq!(current(&d).sync_error.unwrap().server_code, Some(3));
    d.set_child(3, None).unwrap();
    assert_eq!(current(&d).sync_error.unwrap().server_code, Some(2));
    d.set_child(
        2,
        Some(ChildState::Known(summary(Label::LocalComplete, true))),
    )
    .unwrap();
    assert_eq!(current(&d).sync_error.unwrap().attempts, 2);
    d.set_child(1, None).unwrap();
    assert_eq!(current(&d).status, Label::LocalComplete);
    assert!(current(&d).sync_error.is_none());
    let mut retry = summary(Label::UpdateRetry, true);
    retry.sync_error = Some(failure(1, true));
    d.set_child(2, Some(ChildState::Known(retry))).unwrap();
    assert_eq!(current(&d).status, Label::UpdateRetry);
    d.set_child(
        4,
        Some(ChildState::Known(summary(Label::OnlineOnly, false))),
    )
    .unwrap();
    assert_eq!(current(&d).status, Label::DownloadRetry);
}

#[test]
fn identical_updates_and_failed_budget_changes_do_not_damage_previous_summary() {
    let mut d = DirectoryAggregate::new(AggregateBudget {
        children: 1,
        bytes: CHILD_BYTES,
    });
    let known = ChildState::Known(summary(Label::LocalComplete, true));
    assert!(d.set_child(1, Some(known.clone())).unwrap());
    let before = current(&d);
    let bytes = d.accounted_bytes();
    assert!(!d.set_child(1, Some(known.clone())).unwrap());
    assert!(!d.set_child(999, None).unwrap());
    assert_eq!(d.set_child(2, Some(known)), Err(AggregateError::Capacity));
    let mut failed = summary(Label::UpdateFailed, true);
    failed.sync_error = Some(failure(1, false));
    assert_eq!(
        d.set_child(1, Some(ChildState::Known(failed))),
        Err(AggregateError::Capacity)
    );
    assert_eq!(current(&d), before);
    assert_eq!(d.accounted_bytes(), bytes);
}

#[test]
fn move_contributions_update_old_and_new_parents_and_nested_mixed_status() {
    let mut old = directory();
    let mut new = directory();
    let mut root = directory();
    let complete = ChildState::Known(summary(Label::LocalComplete, true));
    let missing = ChildState::Known(summary(Label::OnlineOnly, false));
    old.set_child(10, Some(complete.clone())).unwrap();
    old.set_child(11, Some(missing)).unwrap();
    root.set_child(1, Some(ChildState::Known(current(&old))))
        .unwrap();
    root.set_child(2, Some(ChildState::Known(current(&new))))
        .unwrap();
    assert_eq!(current(&root).status, Label::Mixed);
    // The future tree owner must publish both parent changes atomically.
    old.set_child(10, None).unwrap();
    new.set_child(10, Some(complete)).unwrap();
    root.set_child(1, Some(ChildState::Known(current(&old))))
        .unwrap();
    root.set_child(2, Some(ChildState::Known(current(&new))))
        .unwrap();
    assert_eq!(current(&old).status, Label::OnlineOnly);
    assert_eq!(current(&new).status, Label::LocalComplete);
    assert_eq!(current(&root).status, Label::Mixed);
    assert_eq!(
        root.summary(&Status::PendingUpload, false).unwrap().status,
        Label::PendingUpload
    );
    assert_eq!(
        root.summary(&Status::Conflict, false).unwrap().status,
        Label::Conflict
    );
}
