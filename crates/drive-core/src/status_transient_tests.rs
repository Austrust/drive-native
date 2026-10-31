use super::*;
use crate::{
    Status,
    status_aggregate::{Availability, Label},
    status_changes::StatusCursor,
    status_index::{IndexBudget, IndexError, IndexOwner},
    status_projection::{
        ContentIdentity, DraftIdentity, ProjectedNode, ProjectionBatch, StatusAtom, StatusSnapshot,
    },
};
// Every observation is paired with a fresh source-issued probe ticket.
fn record(
    s: &mut TransientSource,
    id: &str,
    key: &ContentKey,
    result: ContentResult,
) -> Result<bool, TransientError> {
    match s.begin_verification(id, key)? {
        Some(ticket) => s.record_availability(&ticket, result),
        None => Ok(false),
    }
}
fn database(seq: u64) -> StatusCursor {
    StatusCursor {
        epoch: "db".into(),
        sequence: seq,
    }
}
fn atom(cached: bool) -> StatusAtom {
    StatusAtom {
        id: "f".into(),
        parent: Some("root".into()),
        name: "file".into(),
        directory: false,
        own: if cached {
            Status::LocalComplete
        } else {
            Status::OnlineOnly
        },
        inherited_pin: false,
        availability: if cached {
            Availability::Unverified
        } else {
            Availability::Absent
        },
        failure: None,
        content: ContentIdentity {
            remote_version: "remote-v1".into(),
            local_version: cached.then(|| "local-v1".into()),
            blob: cached.then(|| "blob-one".into()),
            dirty_blob: None,
            draft: None,
        },
    }
}
fn index_with(a: StatusAtom, budget: IndexBudget) -> IndexOwner {
    let mut root = atom(false);
    root.id = "root".into();
    root.parent = None;
    root.name = String::new();
    root.directory = true;
    IndexOwner::build(
        "root".into(),
        StatusSnapshot {
            through: database(10),
            atoms: vec![root, a],
        },
        budget,
    )
    .unwrap()
}
fn source() -> TransientSource {
    TransientSource::new("fsd-one".into(), TransientBudget::default()).unwrap()
}
fn setup(index: &mut IndexOwner, source: &TransientSource) {
    let db = index.stamp().database.clone();
    index
        .install_transient_snapshot(&db, source.snapshot())
        .unwrap();
}
fn batch(source: &TransientSource, after: &TransientCursor) -> TransientBatch {
    match source.read(after, 8192).unwrap() {
        TransientRead::Batch(b) => b,
        _ => panic!("unexpected gap"),
    }
}
fn pump(index: &mut IndexOwner, source: &TransientSource) {
    let b = batch(source, index.stamp().transient.as_ref().unwrap());
    index.apply_transient(b).unwrap();
}
fn db_change(index: &mut IndexOwner, next: Option<StatusAtom>) {
    let from = index.stamp().database.clone();
    let mut through = from.clone();
    through.sequence += 1;
    index
        .apply(ProjectionBatch {
            from,
            through,
            nodes: vec![ProjectedNode {
                id: "f".into(),
                atom: next,
            }],
            affected_buckets: vec![],
        })
        .unwrap();
}

#[test]
fn distinct_versions_and_tokens_survive_old_completion_and_partial_cancel() {
    let mut s = source();
    let mut i = index_with(atom(false), IndexBudget::default());
    setup(&mut i, &s);
    s.start("f", "v1", "old").unwrap();
    s.start("f", "v2", "new").unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file").unwrap().status, Label::Downloading);
    let db = i.stamp().database.clone();
    s.finish("f", "v1", "old").unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file").unwrap().status, Label::Downloading);
    let stamp = s.cursor().clone();
    assert!(!s.finish("f", "v1", "old").unwrap());
    assert_eq!(s.cursor(), &stamp);
    s.start("f", "v2", "reader-independent").unwrap();
    s.finish("f", "v2", "new").unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file").unwrap().status, Label::Downloading);
    s.finish("f", "v2", "reader-independent").unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file").unwrap().status, Label::OnlineOnly);
    assert_eq!(i.stamp().database, db);
}
#[test]
fn actual_blob_key_survives_remote_version_but_new_body_needs_new_evidence() {
    let mut a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a.clone(), IndexBudget::default());
    setup(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
    a.content.remote_version = "remote-v2".into();
    a.own = Status::Updating;
    db_change(&mut i, Some(a.clone()));
    assert_eq!(i.query("file").unwrap().status, Label::Updating);
    assert!(i.query("").unwrap().offline_available);
    a.content.blob = Some("blob-two".into());
    a.content.local_version = Some("local-v2".into());
    a.own = Status::LocalComplete;
    db_change(&mut i, Some(a.clone()));
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    let new_key = ContentKey::from_content(&a.content).unwrap();
    s.bind_content("f", Some(new_key.clone())).unwrap();
    let before = s.cursor().clone();
    assert!(!record(&mut s, "f", &key, ContentResult::Complete).unwrap());
    assert_eq!(s.cursor(), &before);
    record(&mut s, "f", &new_key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
}
#[test]
fn missing_body_and_epoch_reset_never_publish_complete() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a, IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    record(&mut s, "f", &key, ContentResult::Missing).unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    assert_eq!(i.query(""), Err(IndexError::UnknownContent));
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    let replacement = TransientSource::new("fsd-two".into(), TransientBudget::default()).unwrap();
    setup(&mut i, &replacement);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    assert_eq!(i.stamp().database, database(10));
}
#[test]
fn draft_generation_is_actual_identity_but_checkpoint_and_remote_expectation_are_not() {
    let mut a = atom(true);
    a.content.draft = Some(DraftIdentity {
        token: "draft".into(),
        base: "base".into(),
        generation: 1,
        checkpoint: 0,
        dirty: true,
    });
    a.own = Status::PendingUpload;
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a.clone(), IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    a.content.draft.as_mut().unwrap().checkpoint = 1;
    a.content.remote_version = "remote-v2".into();
    assert_eq!(ContentKey::from_content(&a.content), Some(key.clone()));
    db_change(&mut i, Some(a.clone()));
    assert!(i.query("file").unwrap().offline_available);
    a.content.draft.as_mut().unwrap().generation = 2;
    db_change(&mut i, Some(a));
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
}
#[test]
fn exact_replay_is_noop_but_gap_overlap_mutated_replay_and_wrong_epoch_are_rejected() {
    let mut s = source();
    let mut i = index_with(atom(false), IndexBudget::default());
    setup(&mut i, &s);
    s.start("f", "v1", "old").unwrap();
    let first = batch(&s, i.stamp().transient.as_ref().unwrap());
    i.apply_transient(first.clone()).unwrap();
    s.start("f", "v2", "new").unwrap();
    pump(&mut i, &s);
    let before = i.stamp().clone();
    assert!(!i.apply_transient(first.clone()).unwrap().changed);
    assert_eq!(i.stamp(), &before);
    let mut altered = first;
    altered.changes[0].state.flights[0].token = "different".into();
    assert_eq!(
        i.apply_transient(altered),
        Err(IndexError::Transient(TransientError::Discontinuous))
    );
    s.finish("f", "v1", "old").unwrap();
    s.finish("f", "v2", "new").unwrap();
    let mut gap = batch(&s, i.stamp().transient.as_ref().unwrap());
    gap.changes.remove(0);
    assert_eq!(
        i.apply_transient(gap),
        Err(IndexError::Transient(TransientError::Discontinuous))
    );
    assert_eq!(i.stamp(), &before);
    let mut wrong = batch(&s, i.stamp().transient.as_ref().unwrap());
    wrong.through.epoch = "other".into();
    assert_eq!(
        i.apply_transient(wrong),
        Err(IndexError::Transient(TransientError::Epoch))
    );
}
#[test]
fn overflow_requires_snapshot_associated_with_the_current_database() {
    let mut budget = TransientBudget::default();
    budget.journal_entries = 1;
    let mut s = TransientSource::new("small".into(), budget).unwrap();
    let mut i = index_with(atom(false), IndexBudget::default());
    setup(&mut i, &s);
    s.start("f", "v1", "a").unwrap();
    s.start("f", "v2", "b").unwrap();
    assert!(matches!(
        s.read(i.stamp().transient.as_ref().unwrap(), 1).unwrap(),
        TransientRead::SnapshotRequired { .. }
    ));
    let before = i.stamp().clone();
    assert_eq!(
        i.install_transient_snapshot(&database(9), s.snapshot()),
        Err(IndexError::Discontinuous)
    );
    assert_eq!(i.stamp(), &before);
    setup(&mut i, &s);
    assert_eq!(i.query("file").unwrap().status, Label::Downloading);
}
#[test]
fn deleting_metadata_does_not_cancel_other_flights_or_require_rebuilding_the_tree() {
    let mut s = source();
    let a = atom(false);
    let mut i = index_with(a.clone(), IndexBudget::default());
    setup(&mut i, &s);
    s.start("f", "v1", "a").unwrap();
    pump(&mut i, &s);
    db_change(&mut i, None);
    assert!(i.query("file").is_err());
    s.start("f", "v2", "b").unwrap();
    pump(&mut i, &s);
    s.finish("f", "v1", "a").unwrap();
    pump(&mut i, &s);
    db_change(&mut i, Some(a));
    assert_eq!(i.query("file").unwrap().status, Label::Downloading);
}
#[test]
fn source_and_owner_budget_failures_leave_cursors_and_summaries_unchanged() {
    let mut budget = TransientBudget::default();
    budget.flights_per_node = 1;
    let mut s = TransientSource::new("limited".into(), budget).unwrap();
    s.start("f", "v1", "a").unwrap();
    let before = s.snapshot();
    assert!(s.start("f", "v2", "b").is_err());
    assert_eq!(s.snapshot(), before);
    let mut index_budget = IndexBudget::default();
    index_budget.transient.bytes = 2048;
    let mut i = index_with(atom(false), index_budget);
    let mut source = source();
    setup(&mut i, &source);
    let old = i.stamp().clone();
    source.start("f", &"x".repeat(1000), "big").unwrap();
    assert_eq!(
        i.apply_transient(batch(&source, i.stamp().transient.as_ref().unwrap())),
        Err(IndexError::Transient(TransientError::Budget))
    );
    assert_eq!(i.stamp(), &old);
    assert_eq!(i.query("file").unwrap().status, Label::OnlineOnly);
}
#[test]
fn source_sequence_exhaustion_and_invalid_payloads_fail_before_mutation() {
    let mut s = source();
    s.cursor.sequence = u64::MAX;
    assert_eq!(
        s.start("f", "v", "x"),
        Err(TransientError::SequenceExhausted)
    );
    assert!(s.nodes.is_empty());
    assert!(s.bind_content("", None).is_err());
    assert!(s.start("f", "", "x").is_err());
}

#[test]
fn stale_inbound_body_observation_cannot_overwrite_current_verified_content() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a.clone(), IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    let from = i.stamp().transient.clone().unwrap();
    let mut through = from.clone();
    through.sequence += 1;
    let obsolete = ContentKey::Blob {
        blob_id: "obsolete-body".into(),
        version: Some("obsolete-version".into()),
    };
    i.apply_transient(TransientBatch {
        from,
        through: through.clone(),
        changes: vec![TransientChange {
            sequence: through.sequence,
            state: NodeTransient {
                id: "f".into(),
                flights: vec![],
                content: Some(obsolete),
                availability: Some(ContentResult::Missing),
            },
        }],
    })
    .unwrap();
    assert!(i.query("file").unwrap().offline_available);
    let mut update = a;
    update.content.remote_version = "remote-v2".into();
    update.own = Status::Updating;
    db_change(&mut i, Some(update));
    assert!(i.query("file").unwrap().offline_available);
}

#[test]
fn malformed_second_change_cannot_publish_the_first_change() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a, IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    let before = i.stamp().clone();
    record(&mut s, "f", &key, ContentResult::Missing).unwrap();
    s.start("unseen", "v", "token").unwrap();
    let mut malformed = batch(&s, i.stamp().transient.as_ref().unwrap());
    let duplicate = malformed.changes[1].state.flights[0].clone();
    malformed.changes[1].state.flights.push(duplicate);
    assert_eq!(
        i.apply_transient(malformed),
        Err(IndexError::Transient(TransientError::Invalid))
    );
    assert_eq!(i.stamp(), &before);
    assert!(i.query("file").unwrap().offline_available);
    // The runtime must mask this old generation while the rejected source
    // update is pending; retaining it atomically is not a freshness claim.
}

#[test]
fn acknowledged_consumer_never_overflows_merely_because_of_total_history_length() {
    let mut limits = TransientBudget::default();
    limits.journal_entries = 2;
    let mut s = TransientSource::new("ack".into(), limits).unwrap();
    let mut i = index_with(atom(false), IndexBudget::default());
    setup(&mut i, &s);
    let oldest = s.cursor().clone();
    for step in 0..12 {
        if step % 2 == 0 {
            s.start("f", "v1", "token").unwrap();
        } else {
            s.finish("f", "v1", "token").unwrap();
        }
        pump(&mut i, &s);
        let before = s.cursor().clone();
        s.acknowledge(i.stamp().transient.as_ref().unwrap())
            .unwrap();
        assert_eq!(s.cursor(), &before);
        assert!(s.journal.is_empty());
        assert_eq!(s.journal.capacity(), 0);
    }
    assert!(matches!(
        s.read(&oldest, 1).unwrap(),
        TransientRead::SnapshotRequired { .. }
    ));
    assert_eq!(i.query("file").unwrap().status, Label::OnlineOnly);
}

#[test]
fn real_zero_length_creation_and_dirty_draft_produce_valid_content_keys() {
    use crate::{
        Node, Store,
        status_projection::{Capture, ProjectionBudget},
    };
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[Node {
                id: "root".into(),
                parent: None,
                name: "root".into(),
                directory: true,
                version: "v1".into(),
                metadata: None,
            }],
            "1",
            "local-test",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let created = store.create_local_file("root", "empty", 0o600).unwrap();
    assert_eq!(created.version, crate::creations::CREATE_VERSION);
    assert!(!created.version.is_empty());
    let Capture::Ready(snapshot) = store.snapshot_status(ProjectionBudget::default()).unwrap()
    else {
        panic!()
    };
    let node = snapshot.atoms.iter().find(|a| a.id == created.id).unwrap();
    let key = ContentKey::from_content(&node.content).unwrap();
    assert!(matches!(key, ContentKey::DirtyBlob { .. }));
    let mut s = source();
    s.bind_content(&created.id, Some(key.clone())).unwrap();
    record(&mut s, &created.id, &key, ContentResult::Complete).unwrap();
    let mut byte = [0u8; 1];
    assert_eq!(
        store
            .open_cached(&created.id)
            .unwrap()
            .read_at(&mut byte, 0)
            .unwrap(),
        0
    );
    let draft = store
        .begin_draft(&created.id, &created.version, None)
        .unwrap();
    store.draft_truncate(&draft, 0).unwrap();
    let Capture::Ready(snapshot) = store.snapshot_status(ProjectionBudget::default()).unwrap()
    else {
        panic!()
    };
    let node = snapshot.atoms.iter().find(|a| a.id == created.id).unwrap();
    let key = ContentKey::from_content(&node.content).unwrap();
    let ContentKey::Draft {
        base, generation, ..
    } = &key
    else {
        panic!("dirty draft must take precedence")
    };
    assert_eq!(base, crate::creations::CREATE_VERSION);
    assert!(*generation > 0);
    s.bind_content(&created.id, Some(key.clone())).unwrap();
    record(&mut s, &created.id, &key, ContentResult::Complete).unwrap();
}

#[test]
fn same_key_probe_aba_and_out_of_order_completion_cannot_restore_missing_content() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a, IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
    let before_unbind = s.begin_verification("f", &key).unwrap().unwrap();
    s.bind_content("f", None).unwrap();
    pump(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    s.bind_content("f", Some(key.clone())).unwrap();
    let current = s.begin_verification("f", &key).unwrap().unwrap();
    assert!(
        s.record_availability(&current, ContentResult::Missing)
            .unwrap()
    );
    let stamp = s.cursor().clone();
    assert!(
        !s.record_availability(&before_unbind, ContentResult::Complete)
            .unwrap()
    );
    assert!(
        !s.record_availability(&current, ContentResult::Complete)
            .unwrap()
    );
    assert_eq!(s.cursor(), &stamp);
    pump(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));

    let slow = s.begin_verification("f", &key).unwrap().unwrap();
    let latest = s.begin_verification("f", &key).unwrap().unwrap();
    // Equal Missing still consumes the newer ticket. The older probe must not
    // overwrite it simply because neither binding nor body key changed.
    assert!(
        !s.record_availability(&latest, ContentResult::Missing)
            .unwrap()
    );
    assert!(
        !s.record_availability(&slow, ContentResult::Complete)
            .unwrap()
    );
    assert!(
        !s.record_availability(&latest, ContentResult::Complete)
            .unwrap()
    );
    assert_eq!(s.cursor(), &stamp);
    let final_probe = s.begin_verification("f", &key).unwrap().unwrap();
    assert!(
        s.record_availability(&final_probe, ContentResult::Complete)
            .unwrap()
    );
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
}

#[test]
fn cancelled_and_foreign_epoch_tickets_do_not_consume_current_probe() {
    let key = ContentKey::from_content(&atom(true).content).unwrap();
    let mut s = source();
    s.bind_content("f", Some(key.clone())).unwrap();
    let old = s.begin_verification("f", &key).unwrap().unwrap();
    let current = s.begin_verification("f", &key).unwrap().unwrap();
    assert!(!s.cancel_verification(&old));
    assert!(s.cancel_verification(&current));
    assert!(!s.cancel_verification(&current));
    assert!(
        !s.record_availability(&current, ContentResult::Complete)
            .unwrap()
    );
    let mut replacement =
        TransientSource::new("fsd-new".into(), TransientBudget::default()).unwrap();
    replacement.bind_content("f", Some(key.clone())).unwrap();
    let new_epoch = replacement.begin_verification("f", &key).unwrap().unwrap();
    assert!(
        !replacement
            .record_availability(&old, ContentResult::Complete)
            .unwrap()
    );
    assert!(!replacement.cancel_verification(&old));
    assert!(
        replacement
            .record_availability(&new_epoch, ContentResult::Complete)
            .unwrap()
    );
}

#[test]
fn folded_unbind_then_different_body_cannot_hide_explicit_proof_revocation() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut s = source();
    let mut i = index_with(a, IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
    s.bind_content("f", None).unwrap();
    let different = ContentKey::DirtyBlob {
        blob_id: "other".into(),
    };
    s.bind_content("f", Some(different.clone())).unwrap();
    record(&mut s, "f", &different, ContentResult::Complete).unwrap();
    // This single batch folds the empty row out. Neither the intermediate
    // revocation nor the mismatch with the still-current DB body may be lost.
    pump(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    assert_eq!(i.query(""), Err(IndexError::UnknownContent));
}

#[test]
fn gap_snapshot_revokes_unproven_old_body_when_unbind_history_is_gone() {
    let a = atom(true);
    let key = ContentKey::from_content(&a.content).unwrap();
    let mut budget = TransientBudget::default();
    budget.journal_entries = 2;
    let mut s = TransientSource::new("bounded".into(), budget).unwrap();
    let mut i = index_with(a, IndexBudget::default());
    setup(&mut i, &s);
    s.bind_content("f", Some(key.clone())).unwrap();
    record(&mut s, "f", &key, ContentResult::Complete).unwrap();
    pump(&mut i, &s);
    assert!(i.query("file").unwrap().offline_available);
    let after = i.stamp().transient.clone().unwrap();
    s.bind_content("f", None).unwrap();
    let different = ContentKey::DirtyBlob {
        blob_id: "different".into(),
    };
    s.bind_content("f", Some(different.clone())).unwrap();
    record(&mut s, "f", &different, ContentResult::Complete).unwrap();
    assert!(matches!(
        s.read(&after, 8192).unwrap(),
        TransientRead::SnapshotRequired { .. }
    ));
    // Snapshot contains only the latest binding, with no unbind row/receipt.
    setup(&mut i, &s);
    assert_eq!(i.query("file"), Err(IndexError::UnknownContent));
    assert_eq!(i.query(""), Err(IndexError::UnknownContent));
}

#[test]
fn probe_exhaustion_budget_and_failed_result_preserve_retryable_current_ticket() {
    let key = ContentKey::from_content(&atom(true).content).unwrap();
    let mut s = source();
    s.bind_content("f", Some(key.clone())).unwrap();
    let accounted = s.accounted_bytes();
    s.budget.bytes = accounted;
    assert_eq!(s.begin_verification("f", &key), Err(TransientError::Budget));
    assert!(s.probes.is_empty());
    assert_eq!(s.next_probe, 0);
    s.budget = TransientBudget::default();
    let ticket = s.begin_verification("f", &key).unwrap().unwrap();
    s.next_probe = u64::MAX;
    assert_eq!(
        s.begin_verification("f", &key),
        Err(TransientError::SequenceExhausted)
    );
    let stamp = s.cursor().clone();
    s.budget.bytes = 1;
    assert_eq!(
        s.record_availability(&ticket, ContentResult::Complete),
        Err(TransientError::Budget)
    );
    assert_eq!(s.cursor(), &stamp);
    s.budget = TransientBudget::default();
    assert!(
        s.record_availability(&ticket, ContentResult::Complete)
            .unwrap()
    );
    assert!(s.probes.is_empty());
    assert_eq!(s.probe_bytes, 0);
}
