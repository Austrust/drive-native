use super::*;
fn fixture() -> (tempfile::TempDir, Store) {
    let t = tempfile::tempdir().unwrap();
    let mut s = Store::open(t.path()).unwrap();
    let node = |id: &str, parent: Option<&str>, directory: bool| Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        directory,
        version: "v1".into(),
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 17,
            mode: 0o755,
        }),
    };
    s.install_remote_snapshot(
        &[
            node("root", None, true),
            node("left", Some("root"), true),
            node("right", Some("root"), true),
            node("sub", Some("left"), true),
            node("leaf", Some("sub"), false),
        ],
        "1",
        "synthetic",
    )
    .unwrap();
    (t, s)
}
fn ino(s: &Store, id: &str) -> u64 {
    s.db.query_row("SELECT ino FROM fs_inodes WHERE node=?1", [id], |r| {
        read_unsigned(r, 0)
    })
    .unwrap()
}
#[test]
fn cached_directory_query_rechecks_acl_and_overlay_ancestors_on_every_execution() {
    let (_t, mut s) = fixture();
    let sub = ino(&s, "sub");
    assert_eq!(s.directory_snapshot(sub, 1 << 20).unwrap().entries.len(), 3);
    s.rename_local("left", "sub", "right", "moved", false)
        .unwrap();
    assert_eq!(
        s.directory_snapshot(sub, 1 << 20).unwrap().entries[1].ino,
        ino(&s, "right")
    );
    s.db.execute("INSERT INTO access_denied VALUES('left')", [])
        .unwrap();
    assert!(
        s.directory_snapshot(sub, 1 << 20).is_err(),
        "old remote parent must still revoke locally moved child"
    );
    assert_eq!(
        s.directory_snapshot(ino(&s, "right"), 1 << 20)
            .unwrap()
            .entries
            .len(),
        2
    );
    s.db.execute("DELETE FROM access_denied", []).unwrap();
    assert_eq!(s.directory_snapshot(sub, 1 << 20).unwrap().entries.len(), 3);
    s.db.execute("INSERT INTO access_denied VALUES('sub')", [])
        .unwrap();
    assert!(s.directory_snapshot(sub, 1 << 20).is_err());
}
#[test]
fn identity_query_rejects_file_deleted_node_and_missing_or_deleted_parent() {
    let (_t, s) = fixture();
    let sub = ino(&s, "sub");
    assert!(s.directory_snapshot(ino(&s, "leaf"), 1 << 20).is_err());
    s.db.execute("UPDATE nodes SET parent='missing' WHERE id='sub'", [])
        .unwrap();
    assert!(s.directory_snapshot(sub, 1 << 20).is_err());
    s.db.execute("UPDATE nodes SET parent='left' WHERE id='sub'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='left'", [])
        .unwrap();
    assert!(s.directory_snapshot(sub, 1 << 20).is_err());
    s.db.execute("UPDATE nodes SET deleted=0 WHERE id='left'", [])
        .unwrap();
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='sub'", [])
        .unwrap();
    assert!(s.directory_snapshot(sub, 1 << 20).is_err());
}
#[test]
fn lazy_inode_repair_preserves_budget_and_rolls_back_on_child_failure() {
    let (_t, s) = fixture();
    let sub = ino(&s, "sub");
    // Completeness hints are intentionally not trusted by the projection.
    s.db.execute("DELETE FROM fs_inodes WHERE node IN ('left','leaf')", [])
        .unwrap();
    let before: i64 =
        s.db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get(0))
            .unwrap();
    assert!(s.directory_snapshot(sub, 1).is_err());
    assert_eq!(
        s.db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        before
    );
    s.db.execute_batch("CREATE TEMP TRIGGER fail_child BEFORE INSERT ON fs_inodes WHEN NEW.node='leaf' BEGIN SELECT RAISE(ABORT,'synthetic failure'); END;").unwrap();
    assert!(s.directory_snapshot(sub, 1 << 20).is_err());
    assert_eq!(
        s.db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get::<_, i64>(0))
            .unwrap(),
        before,
        "parent allocation must roll back too"
    );
    s.db.execute_batch("DROP TRIGGER fail_child").unwrap();
    let (view, t) = s.directory_snapshot_measured(sub, 1 << 20).unwrap();
    assert_eq!(t.inode_writes, 2);
    assert_eq!(view.entries[0].ino, sub);
    assert_eq!(view.entries[1].ino, ino(&s, "left"));
    assert_eq!(view.entries[2].ino, ino(&s, "leaf"));
    let revision = s.local_revision();
    let (repeat, t) = s.directory_snapshot_measured(sub, view.bytes).unwrap();
    assert_eq!(repeat.entries, view.entries);
    assert_eq!(t.inode_writes, 0);
    assert_eq!(s.local_revision(), revision);
}
#[test]
fn unicode_budget_shadow_and_dirty_body_remain_metadata_only() {
    let (_t, mut s) = fixture();
    let sub = ino(&s, "sub");
    let local = s.create_local_file("sub", "文档", 0o600).unwrap();
    let mut remote = s.node("leaf").unwrap();
    remote.name = "文档".into();
    remote.version = "v2".into();
    s.receive(&crate::Batch {
        from: "1".into(),
        to: "2".into(),
        events: vec![crate::Event {
            id: "remote-same-name".into(),
            change: crate::Change::Upsert(remote),
        }],
    })
    .unwrap();
    s.drain().unwrap();
    let view = s.directory_snapshot(sub, 1 << 20).unwrap();
    assert_eq!(view.entries.len(), 3);
    assert_eq!(view.entries[2].name, "文档");
    assert_eq!(view.entries[2].ino, ino(&s, &local.id));
    assert!(s.directory_snapshot(sub, view.bytes - 1).is_err());
    let draft = s.begin_draft(&local.id, &local.version, None).unwrap();
    s.draft_write(&draft, 0, b"unsaved body", false).unwrap();
    // A directory query must not touch the body, even when an application has
    // a dirty handle whose backing file is temporarily unavailable.
    let draft_file = s.storage_directory().join("drafts").join(&draft.token);
    std::fs::remove_file(draft_file).unwrap();
    assert_eq!(
        s.directory_snapshot(sub, view.bytes).unwrap().entries,
        view.entries
    );
}

#[test]
fn read_only_projection_requires_a_fully_mapped_namespace_and_never_writes() {
    let (_t, s) = fixture();
    let root = ino(&s, "root");
    let count = |s: &Store| -> i64 {
        s.db.query_row("SELECT count(*) FROM fs_inodes", [], |r| r.get(0))
            .unwrap()
    };
    // The writer has not repaired these child mappings yet: the read-only
    // projection must report "needs the writer" instead of allocating inodes.
    s.db.execute("DELETE FROM fs_inodes WHERE node IN ('left','right')", [])
        .unwrap();
    let before = count(&s);
    assert!(s.directory_snapshot_read(root, 1 << 20).unwrap().is_none());
    assert_eq!(count(&s), before, "read-only projection must not allocate");
    let full = s.directory_snapshot(root, 1 << 20).unwrap();
    assert_eq!(count(&s), before + 2);
    let read = s
        .directory_snapshot_read(root, full.bytes)
        .unwrap()
        .expect("fully mapped namespace");
    assert_eq!(read.entries, full.entries);
    assert!(
        s.directory_snapshot_read(root, full.bytes - 1).is_err(),
        "budget is enforced without allocating"
    );
    assert_eq!(count(&s), before + 2);
}

#[test]
fn directory_identity_rejects_unmapped_files_deleted_nodes_and_non_directories() {
    let (_t, s) = fixture();
    let root = ino(&s, "root");
    s.directory_identity(root).unwrap();
    let leaf = ino(&s, "leaf");
    assert!(s.directory_identity(leaf).is_err(), "file must not pass identity");
    assert!(s.directory_identity(u64::MAX).is_err(), "missing inode must fail");
    let left = ino(&s, "left");
    s.db.execute("UPDATE nodes SET deleted=1 WHERE id='left'", [])
        .unwrap();
    assert!(s.directory_identity(left).is_err(), "deleted directory must fail");
}