use super::*;
use crate::{
    status_aggregate::{Availability, Label},
    status_projection::{ContentIdentity, ProjectedNode},
};
fn cursor(n: u64) -> StatusCursor {
    StatusCursor {
        epoch: "one".into(),
        sequence: n,
    }
}
fn atom(id: &str, parent: Option<&str>, name: &str, directory: bool, complete: bool) -> StatusAtom {
    StatusAtom {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        directory,
        own: if complete {
            Status::LocalComplete
        } else {
            Status::OnlineOnly
        },
        inherited_pin: false,
        availability: if complete {
            Availability::Complete
        } else {
            Availability::Absent
        },
        failure: None,
        content: ContentIdentity {
            remote_version: "v1".into(),
            local_version: complete.then(|| "v1".into()),
            blob: complete.then(|| "blob".into()),
            dirty_blob: None,
            draft: None,
        },
    }
}
fn seed() -> Vec<StatusAtom> {
    vec![
        atom("r", None, "", true, false),
        atom("a", Some("r"), "a", true, false),
        atom("b", Some("r"), "b", true, false),
        atom("f", Some("a"), "file", false, true),
    ]
}
fn owner() -> IndexOwner {
    IndexOwner::build(
        "r".into(),
        StatusSnapshot {
            through: cursor(10),
            atoms: seed(),
        },
        IndexBudget::default(),
    )
    .unwrap()
}
fn batch(from: u64, to: u64, nodes: Vec<(&str, Option<StatusAtom>)>) -> ProjectionBatch {
    ProjectionBatch {
        from: cursor(from),
        through: cursor(to),
        nodes: nodes
            .into_iter()
            .map(|(id, atom)| ProjectedNode {
                id: id.into(),
                atom,
            })
            .collect(),
        affected_buckets: vec![],
    }
}

#[test]
fn single_leaf_updates_only_its_ancestors_and_unknown_never_completes() {
    let mut atoms = seed();
    for i in 0..4000 {
        atoms.push(atom(
            &format!("wide{i}"),
            Some("b"),
            &format!("file{i}"),
            false,
            false,
        ));
    }
    let mut s = IndexOwner::build(
        "r".into(),
        StatusSnapshot {
            through: cursor(10),
            atoms,
        },
        IndexBudget::default(),
    )
    .unwrap();
    let mut f = s.nodes["f"].atom.clone();
    f.availability = Availability::Unverified;
    let result = s.apply(batch(10, 11, vec![("f", Some(f))])).unwrap();
    assert_eq!(result.touched_nodes, 3);
    assert_eq!(s.query("a/file"), Err(IndexError::UnknownContent));
    assert_eq!(s.query("a"), Err(IndexError::UnknownContent));
    assert_eq!(s.query(""), Err(IndexError::UnknownContent));
    assert_eq!(s.query("b/file3999").unwrap().status, Label::OnlineOnly);
}
#[test]
fn move_and_rename_publish_paths_and_both_parents_together() {
    let mut s = owner();
    let mut f = s.nodes["f"].atom.clone();
    f.parent = Some("b".into());
    f.name = "renamed".into();
    let changed = s.apply(batch(10, 11, vec![("f", Some(f))])).unwrap();
    assert_eq!(changed.touched_nodes, 4);
    assert!(s.query("a/file").is_err());
    assert_eq!(s.query("b/renamed").unwrap().status, Label::LocalComplete);
    assert!(s.query("a").unwrap().offline_available);
}
#[test]
fn duplicate_receipts_are_noops_but_modified_old_batches_and_gaps_are_rejected() {
    let mut s = owner();
    let deletion = batch(10, 11, vec![("f", None)]);
    s.apply(deletion.clone()).unwrap();
    let replacement = atom("f", Some("a"), "file", false, false);
    s.apply(batch(11, 12, vec![("f", Some(replacement.clone()))]))
        .unwrap();
    let before = s.stamp().clone();
    assert!(!s.apply(deletion).unwrap().changed);
    assert_eq!(s.stamp(), &before);
    assert_eq!(s.query("a/file").unwrap().status, Label::OnlineOnly);
    assert_eq!(
        s.apply(batch(10, 11, vec![("a", None)])),
        Err(IndexError::Discontinuous)
    );
    assert_eq!(
        s.apply(batch(11, 13, vec![("f", None)])),
        Err(IndexError::Discontinuous)
    );
    assert_eq!(
        s.apply(batch(13, 14, vec![])),
        Err(IndexError::Discontinuous)
    );
    let mut epoch = batch(12, 13, vec![]);
    epoch.through.epoch = "two".into();
    assert_eq!(s.apply(epoch), Err(IndexError::Epoch));
    assert_eq!(s.stamp(), &before);
}
#[test]
fn deletion_acl_hiding_and_invalid_parent_never_publish_partial_state() {
    let mut s = owner();
    let before = s.stamp().clone();
    let memory = s.accounted_bytes();
    assert_eq!(
        s.apply(batch(10, 11, vec![("a", None)])),
        Err(IndexError::InvalidGraph)
    );
    assert_eq!(s.stamp(), &before);
    assert_eq!(s.accounted_bytes(), memory);
    assert!(s.query("a/file").is_ok());
    s.apply(batch(10, 11, vec![("a", None), ("f", None)]))
        .unwrap();
    assert!(s.query("a").is_err());
    assert!(s.query("").unwrap().offline_available);
}
#[test]
fn sibling_collision_cycle_and_budget_rejection_leave_previous_generation() {
    let mut s = owner();
    let before = s.stamp().clone();
    let mut b = s.nodes["b"].atom.clone();
    b.name = "a".into();
    assert_eq!(
        s.apply(batch(10, 11, vec![("b", Some(b))])),
        Err(IndexError::AmbiguousName)
    );
    let mut a = s.nodes["a"].atom.clone();
    a.parent = Some("a".into());
    assert_eq!(
        s.apply(batch(10, 11, vec![("a", Some(a))])),
        Err(IndexError::InvalidGraph)
    );
    s.budget.bytes = s.bytes;
    let mut f = s.nodes["f"].atom.clone();
    f.name = "new".into();
    assert_eq!(
        s.apply(batch(10, 11, vec![("f", Some(f))])),
        Err(IndexError::Budget)
    );
    assert_eq!(s.stamp(), &before);
    assert!(s.query("a/file").is_ok());
    assert!(s.query("b").is_ok());
}
#[test]
fn atomic_sibling_swap_and_directory_type_change_require_complete_children() {
    let mut s = owner();
    let mut a = s.nodes["a"].atom.clone();
    let mut b = s.nodes["b"].atom.clone();
    a.name = "b".into();
    b.name = "a".into();
    s.apply(batch(10, 11, vec![("a", Some(a)), ("b", Some(b))]))
        .unwrap();
    assert!(s.query("b/file").is_ok());
    assert!(s.query("a/file").is_err());
    let a = atom("a", Some("r"), "b", false, false);
    assert_eq!(
        s.apply(batch(11, 12, vec![("a", Some(a.clone()))])),
        Err(IndexError::InvalidGraph)
    );
    s.apply(batch(11, 12, vec![("a", Some(a)), ("f", None)]))
        .unwrap();
    assert_eq!(s.query("b").unwrap().status, Label::OnlineOnly);
}
#[test]
fn no_change_batch_advances_cursor_without_icon_generation() {
    let mut s = owner();
    let generation = s.stamp.generation;
    let same = s.nodes["f"].atom.clone();
    let stats = s.apply(batch(10, 11, vec![("f", Some(same))])).unwrap();
    assert!(!stats.changed);
    assert_eq!(stats.generation, generation);
    assert_eq!(s.stamp.database, cursor(11));
}

#[test]
fn inherited_pin_requires_descendant_projection_and_empty_folders_are_complete() {
    let mut s = owner();
    let mut a = s.nodes["a"].atom.clone();
    a.inherited_pin = true;
    assert_eq!(
        s.apply(batch(10, 11, vec![("a", Some(a.clone()))])),
        Err(IndexError::InvalidGraph)
    );
    let mut f = s.nodes["f"].atom.clone();
    f.inherited_pin = true;
    f.own = Status::PinnedComplete;
    s.apply(batch(10, 11, vec![("a", Some(a)), ("f", Some(f))]))
        .unwrap();
    assert_eq!(s.query("a/file").unwrap().status, Label::PinnedComplete);
    assert_eq!(s.query("a").unwrap().status, Label::PinnedComplete);
    assert!(!s.query("").unwrap().pinned);
    assert!(s.query("b").unwrap().offline_available);
}

#[test]
fn randomized_leaf_changes_and_moves_match_fresh_aggregation() {
    let mut atoms = seed();
    for n in 0..30 {
        atoms.push(atom(
            &format!("f{n}"),
            Some(if n % 2 == 0 { "a" } else { "b" }),
            &format!("file{n}"),
            false,
            false,
        ));
    }
    let mut s = IndexOwner::build(
        "r".into(),
        StatusSnapshot {
            through: cursor(10),
            atoms,
        },
        IndexBudget::default(),
    )
    .unwrap();
    let mut random = 52341u64;
    for seq in 10..180 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let id = format!("f{}", random % 30);
        let mut a = s.nodes[&id].atom.clone();
        a.parent = Some(if random & 32 == 0 { "a" } else { "b" }.into());
        match random % 5 {
            0 => {
                a.own = Status::OnlineOnly;
                a.availability = Availability::Absent;
            }
            1 => {
                a.own = Status::LocalComplete;
                a.availability = Availability::Complete;
            }
            2 => {
                a.own = Status::PendingUpload;
                a.availability = Availability::Complete;
            }
            3 => {
                a.own = Status::Conflict;
                a.availability = Availability::Complete;
            }
            _ => {
                a.own = Status::LocalComplete;
                a.availability = Availability::Unverified;
            }
        }
        s.apply(batch(seq, seq + 1, vec![(&id, Some(a))])).unwrap();
        let fresh = IndexOwner::build(
            "r".into(),
            StatusSnapshot {
                through: cursor(seq + 1),
                atoms: s.nodes.values().map(|r| r.atom.clone()).collect(),
            },
            IndexBudget::default(),
        )
        .unwrap();
        for path in ["", "a", "b", "a/file", "b/file29"] {
            assert_eq!(s.query(path), fresh.query(path));
        }
    }
}

#[test]
fn aggregate_patch_preflight_is_atomic_and_rejects_stale_commit() {
    let mut d = DirectoryAggregate::new(AggregateBudget {
        children: 2,
        bytes: 4096,
    });
    let local = ChildState::Known(Summary {
        status: Label::LocalComplete,
        pinned: false,
        offline_available: true,
        sync_error: None,
    });
    d.set_child(1, Some(local.clone())).unwrap();
    let patch = d
        .prepare(
            vec![(1, None), (2, Some(local.clone()))],
            &Status::OnlineOnly,
            false,
        )
        .unwrap();
    assert_eq!(d.len(), 1);
    assert_eq!(
        patch.summary().as_ref().unwrap().status,
        Label::LocalComplete
    );
    d.set_child(3, Some(local.clone())).unwrap();
    assert_eq!(d.commit(patch), Err(AggregateError::InvalidFacts));
    assert_eq!(d.len(), 2);
    let patch = d
        .prepare(
            vec![(1, None), (2, Some(local))],
            &Status::OnlineOnly,
            false,
        )
        .unwrap();
    d.commit(patch).unwrap();
    assert_eq!(d.len(), 2);
}

#[test]
fn moving_a_wide_directory_keeps_descendant_paths_without_rebuilding_children() {
    let mut atoms = seed();
    for n in 0..2000 {
        atoms.push(atom(
            &format!("child{n}"),
            Some("a"),
            &format!("child{n}"),
            false,
            false,
        ));
    }
    let mut s = IndexOwner::build(
        "r".into(),
        StatusSnapshot {
            through: cursor(10),
            atoms,
        },
        IndexBudget::default(),
    )
    .unwrap();
    let mut a = s.nodes["a"].atom.clone();
    a.parent = Some("b".into());
    let stats = s.apply(batch(10, 11, vec![("a", Some(a))])).unwrap();
    assert_eq!(stats.touched_nodes, 3);
    assert!(s.query("a/child1999").is_err());
    assert!(s.query("b/a/child1999").is_ok());
    assert!(s.query("b/a/file").unwrap().offline_available);
    let (stamp, rows) = s.query_many(&["b/a".into(), "b/a/file".into()]).unwrap();
    assert_eq!(&stamp, s.stamp());
    assert_eq!(rows.len(), 2);
    assert_eq!(
        s.query_many(&vec![String::new(); 257]),
        Err(IndexError::Budget)
    );
}

#[test]
fn aggregate_patch_cannot_be_committed_to_another_directory() {
    let mut a = DirectoryAggregate::new(AggregateBudget {
        children: 2,
        bytes: 4096,
    });
    let mut b = DirectoryAggregate::new(AggregateBudget {
        children: 2,
        bytes: 4096,
    });
    let patch = a
        .prepare(
            vec![(1, Some(ChildState::Unverified))],
            &Status::OnlineOnly,
            false,
        )
        .unwrap();
    assert_eq!(b.commit(patch), Err(AggregateError::InvalidFacts));
    assert_eq!(a.len(), 0);
    assert_eq!(b.len(), 0);
    let patch = a
        .prepare(
            vec![(1, Some(ChildState::Unverified))],
            &Status::OnlineOnly,
            false,
        )
        .unwrap();
    a.commit(patch).unwrap();
    assert!(a.summary(&Status::OnlineOnly, false).is_err());
}

#[test]
fn publication_generation_cannot_wrap_after_preflight() {
    let mut s = owner();
    s.stamp.generation = u64::MAX;
    let mut f = s.nodes["f"].atom.clone();
    f.name = "renamed".into();
    assert_eq!(
        s.apply(batch(10, 11, vec![("f", Some(f))])),
        Err(IndexError::Budget)
    );
    assert_eq!(s.stamp.database, cursor(10));
    assert_eq!(s.stamp.generation, u64::MAX);
    assert!(s.query("a/file").is_ok());
    assert!(s.query("a/renamed").is_err());
}
