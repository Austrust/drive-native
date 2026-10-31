use super::*;
use crate::{DownloadTicket, Node, sha256, status_changes::JournalBudget};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: None,
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    s.install_remote_snapshot(
        &[
            node("root", None, true),
            node("p", Some("root"), true),
            node("q", Some("root"), true),
            node("a", Some("p"), false),
            node("b", Some("q"), false),
        ],
        "1",
        "s",
    )
    .unwrap();
    let c = s.status_change_watermark().unwrap().cursor;
    s.prune_status_changes(&c).unwrap();
    (dir, s)
}
fn ready<T: std::fmt::Debug>(value: Capture<T>) -> T {
    match value {
        Capture::Ready(x) => x,
        other => panic!("unexpected capture {other:?}"),
    }
}
fn delta(s: &Store, c: &StatusCursor) -> ProjectionBatch {
    ready(
        s.capture_status_delta(c, ProjectionBudget::default())
            .unwrap(),
    )
}
fn cursor(s: &Store) -> StatusCursor {
    s.status_change_watermark().unwrap().cursor
}
fn ids(b: &ProjectionBatch) -> Vec<&str> {
    b.nodes.iter().map(|n| n.id.as_str()).collect()
}

#[test]
fn snapshot_is_visible_metadata_only_and_content_references_are_unverified() {
    let (dir, mut s) = fixture();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "v1".into(),
        },
        b"abc",
        &sha256(b"abc"),
    )
    .unwrap();
    let stamp = s.local_revision();
    let before = cursor(&s);
    let snap = ready(s.snapshot_status(ProjectionBudget::default()).unwrap());
    assert_eq!(snap.through, before);
    assert_eq!(snap.atoms.len(), 5);
    for atom in &snap.atoms {
        assert_eq!(atom.own, s.status(&atom.id).unwrap())
    }
    let a = snap.atoms.iter().find(|a| a.id == "a").unwrap();
    assert_eq!(a.availability, Availability::Unverified);
    assert_eq!(a.own, Status::LocalComplete);
    std::fs::remove_file(
        dir.path()
            .join("blobs")
            .join(a.content.blob.as_ref().unwrap()),
    )
    .unwrap();
    assert!(s.status("a").is_err());
    assert_eq!(
        ready(s.snapshot_status(ProjectionBudget::default()).unwrap()),
        snap
    );
    assert_eq!(s.local_revision(), stamp);
    assert_eq!(cursor(&s), before);
}

#[test]
fn full_commit_is_returned_or_budget_rebuild_never_a_transaction_prefix() {
    let (_d, s) = fixture();
    let from = cursor(&s);
    s.db.execute_batch("BEGIN; UPDATE nodes SET conflict=1 WHERE id='a'; UPDATE nodes SET conflict=1 WHERE id='b';").unwrap();
    assert!(s.snapshot_status(ProjectionBudget::default()).is_err());
    assert!(
        s.capture_status_delta(&from, ProjectionBudget::default())
            .is_err()
    );
    s.db.execute_batch("COMMIT").unwrap();
    let mut small = ProjectionBudget::default();
    small.max_changes = 1;
    assert!(matches!(
        s.capture_status_delta(&from, small).unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Budget,
            ..
        }
    ));
    let b = delta(&s, &from);
    assert_eq!(ids(&b), vec!["a", "b"]);
    assert_eq!(b.through, cursor(&s));
    assert!(
        b.nodes
            .iter()
            .all(|n| n.atom.as_ref().unwrap().own == Status::Conflict)
    );
    assert_eq!(delta(&s, &b.through).nodes.len(), 0);
    assert_eq!(delta(&s, &from), b); // projection does not consume/acknowledge
}

#[test]
fn rollback_is_empty_and_commit_before_outer_failure_remains_projectable() {
    let (_d, s) = fixture();
    let from = cursor(&s);
    s.db.execute_batch("BEGIN; UPDATE nodes SET pinned=1 WHERE id='p'; ROLLBACK;")
        .unwrap();
    assert!(delta(&s, &from).nodes.is_empty());
    fn outer(s: &Store) -> Result<()> {
        s.db.execute_batch("BEGIN; UPDATE nodes SET conflict=1 WHERE id='a'; COMMIT;")?;
        Err(Error::State("failure after committed side effect".into()))
    }
    assert!(outer(&s).is_err());
    assert_eq!(ids(&delta(&s, &from)), vec!["a"]);
}

#[test]
fn ordinary_leaf_and_directory_status_do_not_expand_unchanged_descendants() {
    let (_d, s) = fixture();
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET conflict=1 WHERE id='a'", [])
        .unwrap();
    assert_eq!(ids(&delta(&s, &c)), vec!["a"]);
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET conflict=1 WHERE id='p'", [])
        .unwrap();
    assert_eq!(ids(&delta(&s, &c)), vec!["p"]);
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET pinned=1 WHERE id='p'", [])
        .unwrap();
    let d = delta(&s, &c);
    assert_eq!(ids(&d), vec!["a", "p"]);
    assert!(
        d.nodes
            .iter()
            .all(|n| n.atom.as_ref().unwrap().inherited_pin)
    );
}

#[test]
fn acl_and_remote_overlay_edges_hide_and_restore_descendants() {
    let (_d, s) = fixture();
    let remote = serde_json::to_string(&node("a", Some("p"), false)).unwrap();
    s.db.execute("UPDATE nodes SET parent='q' WHERE id='a'", [])
        .unwrap();
    s.db.execute(
        "INSERT INTO namespace_overlay(node,remote) VALUES('a',?1)",
        [remote],
    )
    .unwrap();
    let c = cursor(&s);
    s.db.execute("INSERT INTO access_denied(node) VALUES('p')", [])
        .unwrap();
    let blocked = delta(&s, &c);
    assert_eq!(ids(&blocked), vec!["a", "p"]);
    assert!(blocked.nodes.iter().all(|n| n.atom.is_none()));
    let snapshot = ready(s.snapshot_status(ProjectionBudget::default()).unwrap());
    assert_eq!(
        snapshot
            .atoms
            .iter()
            .map(|n| n.id.as_str())
            .collect::<Vec<_>>(),
        vec!["b", "q", "root"]
    );
    let c = cursor(&s);
    s.db.execute("DELETE FROM access_denied WHERE node='p'", [])
        .unwrap();
    let restored = delta(&s, &c);
    assert_eq!(ids(&restored), vec!["a", "p"]);
    let a = restored
        .nodes
        .iter()
        .find(|n| n.id == "a")
        .unwrap()
        .atom
        .as_ref()
        .unwrap();
    assert_eq!(a.parent.as_deref(), Some("q"));
    assert_eq!(a.own, Status::PendingUpload);
    let c = cursor(&s);
    s.db.execute("DELETE FROM namespace_overlay WHERE node='a'", [])
        .unwrap();
    assert_eq!(
        delta(&s, &c).nodes[0].atom.as_ref().unwrap().own,
        Status::OnlineOnly
    );
}

#[test]
fn ancestor_overlay_and_pin_are_derived_without_body_reads() {
    let (_d, s) = fixture();
    let c = cursor(&s);
    let remote = serde_json::to_string(&node("p", Some("root"), true)).unwrap();
    s.db.execute(
        "INSERT INTO namespace_overlay(node,remote) VALUES('p',?1)",
        [remote],
    )
    .unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["a", "p"]);
    assert!(
        b.nodes
            .iter()
            .all(|n| n.atom.as_ref().unwrap().own == Status::PendingUpload)
    );
}

#[test]
fn local_creation_shadow_and_unshadow_project_both_identities_and_old_new_buckets() {
    let (_d, s) = fixture();
    let c = cursor(&s);
    s.db.execute_batch("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('local','p','a','local:1',0); INSERT INTO local_creations(node,ready) VALUES('local',1);").unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["a", "local"]);
    assert!(b.nodes[0].atom.is_none());
    assert!(b.nodes[1].atom.is_some());
    let c = cursor(&s);
    s.db.execute(
        "UPDATE nodes SET parent='q',name='moved' WHERE id='local'",
        [],
    )
    .unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["a", "local"]);
    assert!(b.nodes.iter().all(|n| n.atom.is_some()));
    assert!(b.affected_buckets.contains(&NameBucket {
        parent: Some("p".into()),
        name: "a".into()
    }));
    assert!(b.affected_buckets.contains(&NameBucket {
        parent: Some("q".into()),
        name: "moved".into()
    }));
    let c = cursor(&s);
    s.db.execute("DELETE FROM nodes WHERE id='local'", [])
        .unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["local"]);
    assert!(b.nodes[0].atom.is_none());
}

#[test]
fn permission_removal_of_a_shadowed_directory_does_not_reveal_remote_children() {
    let (_d, s) = fixture();
    s.db.execute_batch("INSERT INTO nodes(id,parent,name,remote_version,directory) VALUES('local','root','p','local:1',1); INSERT INTO local_creations(node,ready) VALUES('local',1);").unwrap();
    let snap = ready(s.snapshot_status(ProjectionBudget::default()).unwrap());
    assert!(!snap.atoms.iter().any(|n| n.id == "a" || n.id == "p"));
    let c = cursor(&s);
    s.db.execute("INSERT INTO access_denied(node) VALUES('local')", [])
        .unwrap();
    let b = delta(&s, &c);
    assert!(b.nodes.iter().all(|n| n.atom.is_none()));
}

#[test]
fn explicit_gap_epoch_and_budget_fail_closed_without_mutating_journal() {
    let (dir, mut s) = fixture();
    let c = cursor(&s);
    s.pin("p", true).unwrap();
    let stamp = s.local_revision();
    let before = s.status_change_watermark().unwrap();
    for budget in [
        ProjectionBudget {
            max_nodes: 1,
            ..ProjectionBudget::default()
        },
        ProjectionBudget {
            max_bytes: 16,
            ..ProjectionBudget::default()
        },
        ProjectionBudget {
            max_work: 1,
            ..ProjectionBudget::default()
        },
    ] {
        assert!(matches!(
            s.capture_status_delta(&c, budget).unwrap(),
            Capture::Rebuild {
                reason: RebuildReason::Budget,
                ..
            }
        ));
        assert!(matches!(
            s.snapshot_status(budget).unwrap(),
            Capture::Rebuild {
                reason: RebuildReason::Budget,
                ..
            }
        ));
    }
    assert_eq!(s.local_revision(), stamp);
    assert_eq!(s.status_change_watermark().unwrap(), before);
    s.prune_status_changes(&before.cursor).unwrap();
    assert!(matches!(
        s.capture_status_delta(&c, ProjectionBudget::default())
            .unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Journal(GapReason::HistoryDiscarded),
            ..
        }
    ));
    drop(s);
    let mut s = Store::open(dir.path()).unwrap();
    assert!(matches!(
        s.capture_status_delta(&c, ProjectionBudget::default())
            .unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Journal(GapReason::EpochChanged),
            ..
        }
    ));
    s.set_status_journal_budget(JournalBudget {
        entries: 1,
        bytes: 1024,
    })
    .unwrap();
    let c = cursor(&s);
    s.db.execute_batch(
        "UPDATE nodes SET conflict=1 WHERE id='a'; UPDATE nodes SET conflict=1 WHERE id='b'",
    )
    .unwrap();
    assert!(matches!(
        s.capture_status_delta(&c, ProjectionBudget::default())
            .unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::Journal(GapReason::HistoryDiscarded),
            ..
        }
    ));
}

#[test]
fn invalid_parent_graph_and_remote_cycles_require_rebuild() {
    let (_d, s) = fixture();
    s.db.execute("UPDATE nodes SET parent='absent' WHERE id='a'", [])
        .unwrap();
    assert!(matches!(
        s.snapshot_status(ProjectionBudget::default()).unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::InvalidGraph,
            ..
        }
    ));
    s.db.execute("UPDATE nodes SET parent='p' WHERE id='a'", [])
        .unwrap();
    let remote = serde_json::to_string(&node("p", Some("a"), true)).unwrap();
    s.db.execute(
        "INSERT INTO namespace_overlay(node,remote) VALUES('p',?1)",
        [remote],
    )
    .unwrap();
    assert!(matches!(
        s.snapshot_status(ProjectionBudget::default()).unwrap(),
        Capture::Rebuild {
            reason: RebuildReason::InvalidGraph,
            ..
        }
    ));
}

#[test]
fn dirty_draft_conflict_unlink_and_current_version_failure_follow_core_rules() {
    let (_d, s) = fixture();
    let c = cursor(&s);
    s.db.execute_batch("INSERT INTO drafts(node,token,base,generation,checkpoint,dirty) VALUES('a','draft-token','v1',3,2,1);").unwrap();
    let a = delta(&s, &c).nodes[0].atom.clone().unwrap();
    assert_eq!(a.own, Status::PendingUpload);
    assert_eq!(a.availability, Availability::Unverified);
    assert_eq!(a.content.draft.unwrap().generation, 3);
    let c = cursor(&s);
    s.db.execute("UPDATE drafts SET base='v0' WHERE node='a'", [])
        .unwrap();
    assert_eq!(
        delta(&s, &c).nodes[0].atom.as_ref().unwrap().own,
        Status::Conflict
    );
    let c = cursor(&s);
    s.db.execute(
        "INSERT INTO local_unlinks(node,confirmed) VALUES('a',0)",
        [],
    )
    .unwrap();
    assert_eq!(
        delta(&s, &c).nodes[0].atom.as_ref().unwrap().own,
        Status::PendingUpload
    );
    s.db.execute("DELETE FROM drafts WHERE node='a'", [])
        .unwrap();
    let failure = DownloadFailure {
        schema: 1,
        kind: crate::download_jobs::DownloadFailureKind::Transport,
        server_code: None,
        attempts: 2,
        retry_at: Some(123),
    };
    let c = cursor(&s);
    s.db.execute("INSERT INTO transfers(id,node,version,direction,state,error) VALUES('t','a','v1','download','failed',?1)",[serde_json::to_string(&failure).unwrap()]).unwrap();
    assert_eq!(
        delta(&s, &c).nodes[0].atom.as_ref().unwrap().failure,
        Some(failure)
    );
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET remote_version='v2' WHERE id='a'", [])
        .unwrap();
    assert!(
        delta(&s, &c).nodes[0]
            .atom
            .as_ref()
            .unwrap()
            .failure
            .is_none()
    );
}

#[test]
fn irrelevant_failure_payload_does_not_override_directory_or_pending_state() {
    let (_d, s) = fixture();
    let c = cursor(&s);
    s.db.execute_batch("UPDATE nodes SET conflict=1 WHERE id='a'; INSERT INTO transfers(id,node,version,direction,state,error) VALUES('ta','a','v1','download','failed','bad json'); INSERT INTO transfers(id,node,version,direction,state,error) VALUES('tp','p','v1','download','failed','bad json');").unwrap();
    let b = delta(&s, &c);
    assert!(
        b.nodes
            .iter()
            .all(|n| n.atom.as_ref().unwrap().failure.is_none())
    );
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET conflict=0 WHERE id='a'", [])
        .unwrap();
    assert!(
        s.capture_status_delta(&c, ProjectionBudget::default())
            .is_err()
    );
}

#[test]
fn moving_directory_reprojects_descendant_inheritance_and_both_parent_buckets() {
    let (_d, s) = fixture();
    s.db.execute("UPDATE nodes SET pinned=1 WHERE id='q'", [])
        .unwrap();
    let c = cursor(&s);
    s.db.execute("UPDATE nodes SET parent='q' WHERE id='p'", [])
        .unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["a", "p"]);
    assert!(
        b.nodes
            .iter()
            .all(|n| n.atom.as_ref().unwrap().inherited_pin)
    );
    assert!(b.affected_buckets.contains(&NameBucket {
        parent: Some("root".into()),
        name: "p".into()
    }));
    assert!(b.affected_buckets.contains(&NameBucket {
        parent: Some("q".into()),
        name: "p".into()
    }));
}

#[test]
fn atomic_replace_keeps_source_identity_and_removes_target_in_one_batch() {
    let (_d, mut s) = fixture();
    s.ensure_fs_root("root").unwrap();
    for id in ["a", "b"] {
        s.publish_download(
            &DownloadTicket {
                id: id.into(),
                version: "v1".into(),
            },
            b"abc",
            &sha256(b"abc"),
        )
        .unwrap();
    }
    let c = cursor(&s);
    s.replace_local("p", "a", "q", "b", "a", "b").unwrap();
    let b = delta(&s, &c);
    assert_eq!(ids(&b), vec!["a", "b"]);
    let a = b.nodes[0].atom.as_ref().unwrap();
    assert_eq!((a.parent.as_deref(), a.name.as_str()), (Some("q"), "b"));
    assert_eq!(a.own, Status::PendingUpload);
    assert_eq!(a.availability, Availability::Unverified);
    assert!(b.nodes[1].atom.is_none());
    assert_eq!(b.through, cursor(&s));
    let snap = ready(s.snapshot_status(ProjectionBudget::default()).unwrap());
    assert_eq!(snap.atoms.iter().find(|n| n.id == "a"), Some(a));
    assert!(!snap.atoms.iter().any(|n| n.id == "b"));
}
