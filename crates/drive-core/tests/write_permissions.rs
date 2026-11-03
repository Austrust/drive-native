//! Per-directory write policy: explicit source, ancestor inheritance, fail
//! closed on unknown, durable revision and schema migration.
use drive_core::write_permissions::{EffectiveWritePermission, WritePermission};
use drive_core::*;

fn n(id: &str, parent: Option<&str>, name: &str, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: version.into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[
            n("1", None, "root", "r-1", true),
            n("2", Some("1"), "dirA", "v-2", true),
            n("3", Some("2"), "a.doc", "v-3", false),
            n("4", Some("1"), "root.doc", "v-4", false),
        ],
        "1",
        "server",
    )
    .unwrap();
    (d, s)
}
fn versioned(s: &Store, id: &str) -> EffectiveWritePermission {
    s.effective_write_permission(id).unwrap()
}

#[test]
fn missing_policy_is_unknown_and_never_inherits_share_capability() {
    let (_d, s) = fixture();
    for id in ["1", "2", "3", "4"] {
        assert_eq!(
            versioned(&s, id),
            EffectiveWritePermission::unknown(),
            "node {id}"
        );
    }
}

#[test]
fn nearest_directory_decision_wins_and_unknown_stays_conservative() {
    let (_d, mut s) = fixture();
    s.set_directory_permission("1", WritePermission::Denied, "setup")
        .unwrap();
    let denied = versioned(&s, "3");
    assert_eq!(denied.permission, WritePermission::Denied);
    assert_eq!(denied.source.as_deref(), Some("1"));
    assert_eq!(denied.origin.as_deref(), Some("setup"));

    s.set_directory_permission("2", WritePermission::Allowed, "file_station")
        .unwrap();
    let allowed = versioned(&s, "3");
    assert_eq!(allowed.permission, WritePermission::Allowed);
    assert_eq!(allowed.source.as_deref(), Some("2"));
    // Sibling files still inherit the root rejection.
    assert_eq!(versioned(&s, "4").permission, WritePermission::Denied);

    // An explicit unknown is not silently promoted.
    s.set_directory_permission("2", WritePermission::Unknown, "probe")
        .unwrap();
    assert_eq!(versioned(&s, "3").permission, WritePermission::Unknown);

    assert!(s.clear_directory_permission("2").unwrap());
    assert!(!s.clear_directory_permission("2").unwrap());
    assert_eq!(versioned(&s, "3").permission, WritePermission::Denied);
    s.clear_directory_permission("1").unwrap();
    assert_eq!(versioned(&s, "3"), EffectiveWritePermission::unknown());
}

#[test]
fn policy_revision_advances_only_with_changes_and_survives_restart() {
    let (d, mut s) = fixture();
    assert_eq!(s.write_policy_revision().unwrap(), 0);
    s.set_directory_permission("1", WritePermission::Denied, "setup")
        .unwrap();
    let first = s.write_policy_revision().unwrap();
    assert_eq!(first, 1);
    // Idempotent value still records a new durable decision generation.
    s.set_directory_permission("1", WritePermission::Denied, "setup")
        .unwrap();
    assert_eq!(s.write_policy_revision().unwrap(), 2);
    assert!(!s.clear_directory_permission("missing").unwrap());
    assert_eq!(s.write_policy_revision().unwrap(), 2);
    drop(s);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(s.write_policy_revision().unwrap(), 2);
    assert_eq!(versioned(&s, "4").permission, WritePermission::Denied);
}

#[test]
fn allowed_directory_discovery_is_scoped_and_effective() {
    let (_d, mut s) = fixture();
    s.set_directory_permission("1", WritePermission::Denied, "setup")
        .unwrap();
    assert!(!s.has_allowed_directory_permission("1").unwrap());
    s.set_directory_permission("2", WritePermission::Allowed, "file_station")
        .unwrap();
    assert!(s.has_allowed_directory_permission("1").unwrap());
    assert_eq!(
        s.effective_write_permission("3").unwrap().permission,
        WritePermission::Allowed
    );
}

#[test]
fn invalid_inputs_are_rejected_without_changing_policy() {
    let (_d, mut s) = fixture();
    assert!(
        s.set_directory_permission("missing", WritePermission::Allowed, "setup")
            .is_err()
    );
    assert!(
        s.set_directory_permission("1", WritePermission::Allowed, "")
            .is_err()
    );
    assert!(
        s.set_directory_permission("1", WritePermission::Allowed, "bad\norigin")
            .is_err()
    );
    assert!(WritePermission::parse("maybe").is_err());
    assert_eq!(s.write_policy_revision().unwrap(), 0);
}

#[test]
fn schema_23_upgrade_adds_policy_table_without_inventing_decisions() {
    let (d, s) = fixture();
    s.node("1").unwrap();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE directory_permissions; PRAGMA user_version=23;")
        .unwrap();
    drop(db);
    let s = Store::open(d.path()).unwrap();
    assert_eq!(versioned(&s, "3"), EffectiveWritePermission::unknown());
    assert_eq!(s.write_policy_revision().unwrap(), 0);
    let count: i64 = rusqlite::Connection::open(d.path().join("state.sqlite"))
        .unwrap()
        .query_row("SELECT count(*) FROM directory_permissions", [], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(count, 0);
}
