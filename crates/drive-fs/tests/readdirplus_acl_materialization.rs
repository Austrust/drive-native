//! Issue #26 residual kernel test: an ACL-mount READDIRPLUS traversal materializes
//! only bounded page slots, not the whole directory tuple list on every
//! continuation.
//!
//! Same test name and fixture run against the pre-fix tree (evidence branch) and
//! the fixed tree, so the counters give a real red -> green pair:
//!
//! * pre-fix: every reply `iter().map(...).collect()`s all N entries, so
//!   `materialized_max_per_reply == N` and the traversal total is O(N * replies);
//! * fixed: each reply builds at most `PLUS_PAGE == 128` slots per batch, so the
//!   max batch is <= 128 and the traversal total is O(N).
//!
//! Requires an ordinary Linux user and `/dev/fuse`; run explicitly, serially:
//! `cargo test -p drive-fs --all-features --locked --test readdirplus_acl_materialization -- --ignored --test-threads=1`
use drive_core::{Node, NodeMetadata, Store};
use drive_fs::publication::NotifyHub;
use std::sync::Arc;

const ENTRIES: usize = 8_192;

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool, size: u64) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size,
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn acl_readdirplus_materializes_only_bounded_pages() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0, "run as the unprivileged test user");
    let dir = tempfile::tempdir().unwrap();
    let mount = dir.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut store = Store::open(dir.path().join("state")).unwrap();
    let mut nodes = vec![
        node("root", None, "root", true, 0),
        node("big", Some("root"), "big", true, 0),
        node("hidden", Some("root"), "hidden", true, 0),
        node("hidden/secret", Some("hidden"), "secret", false, 3),
    ];
    for index in 0..ENTRIES {
        nodes.push(node(
            &format!("big/f{index:05}"),
            Some("big"),
            &format!("f{index:05}"),
            false,
            3,
        ));
    }
    store.install_remote_snapshot(&nodes, "1", "synthetic").unwrap();
    store.ensure_fs_root("root").unwrap();
    // One permission denial in an unrelated subtree keeps `big` readable but
    // makes the mount-level `has_permission_denials()` route READDIRPLUS
    // through `send_acl_plus` (it also disables the index fast path).
    drive_core::write_permissions::inject(
        &store,
        "root",
        "hidden",
        drive_core::write_permissions::Action::List,
        drive_core::write_permissions::Decision::Deny,
    )
    .unwrap();
    let store = Arc::new(drive_core::SharedStore::new(store));
    assert!(
        store.has_permission_denials(),
        "the fixture denial is not visible to the mount, so READDIRPLUS would not \
         take the ACL path under test"
    );
    // READDIRPLUS is enabled at FUSE_INIT only for a non-zero TTL (see
    // CacheFilesystem::init); keep the notifier profile the readdirplus tests use.
    let hub = NotifyHub::with_ttl_ms(1_000);
    let fs = drive_fs::CacheFilesystem::new(store, "root", uid, gid)
        .unwrap()
        .with_notify_hub(hub.clone());
    let session = fuser::Session::new(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-acl-materialization".into()),
        ],
    )
    .unwrap();
    hub.install(session.notifier());
    let session = fuser::BackgroundSession::new(session).unwrap();

    let target = mount.join("big");
    // Warm the inode before snapshotting the counters.
    let _ = std::fs::metadata(&target).unwrap();
    let before = drive_fs::diagnostics::readdir_plus_materialization();
    let count = std::fs::read_dir(&target)
        .unwrap()
        .collect::<std::io::Result<Vec<_>>>()
        .unwrap()
        .len();
    let after = drive_fs::diagnostics::readdir_plus_materialization();
    let _ = session.join_result();

    assert_eq!(count, ENTRIES, "the ACL traversal skipped or duplicated entries");
    let replies = after.replies.saturating_sub(before.replies);
    assert!(
        replies >= 1,
        "the traversal did not take the ACL READDIRPLUS path (no materialization recorded)"
    );
    let materialized = after.materialized_total.saturating_sub(before.materialized_total);
    let max_per_reply = after.materialized_max_per_reply;
    let batch_slots_max = after.batch_slots_max;
    println!(
        "issue #26 kernel: entries={ENTRIES} replies={replies} materialized_total={materialized} \
         materialized_max_per_reply={max_per_reply} batches={} batch_slots_max={batch_slots_max}",
        after.batches.saturating_sub(before.batches)
    );
    assert!(
        max_per_reply <= 128,
        "one ACL READDIRPLUS reply materialized {max_per_reply} entries; the whole-directory \
         collect is back (pre-fix: {ENTRIES} per reply)"
    );
    assert!(
        batch_slots_max <= 128,
        "one ACL page built {batch_slots_max} slots (page batch must stay <= 128)"
    );
    assert!(
        materialized <= ENTRIES as u64 * 2 + 256,
        "the ACL traversal materialized {materialized} entries for {ENTRIES} entries; expected \
         O(N), not O(N * replies)"
    );
}
