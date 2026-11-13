//! Issue #56: a partial rule change must not leave previously purged paths
//! hidden. The old restore ran only when every rule had been removed, so
//! deleting one of two rules kept its paths invisible until the user cleared
//! the whole list and restarted twice.
//!
//! The reconciler restores the ids recorded in `excluded_purged` that the
//! *current* rules no longer match, while the paths the remaining rules still
//! match stay hidden. These tests drive the mount reconciliation without FUSE.
use super::*;
use drive_core::{Node, NodeMetadata};
use std::os::unix::fs::PermissionsExt;

const GIT_RULE: &str = r"(^|/)\.git(/|$)";
const CACHE_RULE: &str = r"(^|/)cache(/|$)";
const ROOT: &str = "100";
/// One mount instance keeps its task id across restarts (trust settings are
/// keyed by it).
const TASK_ID: &str = "d4b8bd5c-8ce8-4c2e-9a39-5f86f4f9b2d1";

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn scope() -> Scope {
    Scope {
        view_id: 1,
        node_id: 2,
        file_id: ROOT.into(),
        path: "/Lab".into(),
    }
}

fn profile(mount: PathBuf, patterns: &[&str]) -> Profile {
    Profile {
        schema: 1,
        task_id: TASK_ID.into(),
        host: "unused.invalid".into(),
        tls_name: "unused.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "tester".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: scope(),
        mount,
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: patterns.iter().map(|pattern| (*pattern).to_owned()).collect(), exclude_mode: Default::default(),
    }
}

fn mount_dir(dir: &std::path::Path) -> PathBuf {
    let mount = dir.join("mnt");
    std::fs::create_dir_all(&mount).unwrap();
    mount
}

/// Committed namespace with a `.git` directory and a cache directory; the
/// fixture has inode identities because a real mount would have backfilled
/// them before the rules were applied.
fn committed_store(dir: &std::path::Path) -> Store {
    let mut store = Store::open(dir).unwrap();
    store
        .install_remote_snapshot(
            &[
                node(ROOT, None, "Lab", true),
                node("101", Some(ROOT), "A.txt", false),
                node("200", Some(ROOT), ".git", true),
                node("201", Some("200"), "config", false),
                node("300", Some(ROOT), "cache", true),
                node("301", Some("300"), "objects", false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root(ROOT).unwrap();
    store.ensure_filesystem_inodes(ROOT).unwrap();
    store
}

fn service_from_store(
    dir: &std::path::Path,
    patterns: &[&str],
    store: Store,
) -> Arc<Service> {
    // Trust settings and the state root must be private to the owner.
    std::fs::set_permissions(dir, std::fs::Permissions::from_mode(0o700)).unwrap();
    let service = Service::new(
        profile(mount_dir(dir), patterns),
        store,
        PathBuf::from("/unused-worker"),
    )
    .expect("service");
    service.trust_settings().save(dir).unwrap();
    service
}

/// Open the service the way `Service::new` does for a real mount on a freshly
/// seeded state root.
fn open_service(dir: &std::path::Path, patterns: &[&str]) -> Arc<Service> {
    service_from_store(dir, patterns, committed_store(dir))
}

/// Reopen the same state root after a restart; the committed namespace,
/// including the previous purge marker, is what a real restart sees.
fn reopen_service(dir: &std::path::Path, patterns: &[&str]) -> Arc<Service> {
    service_from_store(dir, patterns, Store::open(dir).unwrap())
}

fn refresh_index(service: &Service) {
    let exclusion = service.bridge.exclusion_state();
    let store = service.bridge.store.lock().unwrap();
    refresh_exclusion_before_mount(&exclusion, &store, ROOT, true).unwrap();
}

/// Issue #56 red scenario: rules A+B hide `.git` and cache; the user keeps B
/// and removes A, then restarts. `.git` must come back, cache must stay hidden,
/// and the purge marker must keep only the still-matched ids.
#[test]
fn partial_rule_removal_restores_only_the_unmatched_paths() {
    let dir = tempfile::tempdir().unwrap();
    let first = open_service(dir.path(), &[GIT_RULE, CACHE_RULE]);
    refresh_index(&first);
    first.bridge.purge_excluded_from_store().unwrap();
    {
        let store = first.bridge.store.lock().unwrap();
        assert!(store.node("200").is_err(), ".git hidden under A+B");
        assert!(store.node("300").is_err(), "cache hidden under A+B");
        assert!(
            store.next_remove().unwrap().is_none(),
            "purge must not queue a remote delete"
        );
        assert_eq!(store.pending_operations().unwrap(), 0);
    }
    drop(first);

    // Restart with only the cache rule active.
    let second = reopen_service(dir.path(), &[CACHE_RULE]);
    refresh_index(&second);
    second.bridge.restore_excluded_from_store().unwrap();
    {
        let store = second.bridge.store.lock().unwrap();
        assert!(
            store.node("200").is_ok(),
            ".git must be restored after its rule was removed"
        );
        assert!(
            store.node("300").is_err(),
            "cache stays hidden while its rule is active"
        );
        assert!(
            store.next_remove().unwrap().is_none(),
            "restore must not queue a remote delete"
        );
        assert_eq!(store.pending_operations().unwrap(), 0);
    }
    drop(second);

    // The marker keeps only the still-hidden subtree (cache dir + child), so a
    // later full restore brings back those two rows and nothing else.
    let mut reopened = Store::open(dir.path()).unwrap();
    assert_eq!(
        reopened.restore_excluded_nodes().unwrap(),
        2,
        "only the cache subtree remains in excluded_purged"
    );
    assert!(reopened.node("300").is_ok());
    assert!(reopened.node("301").is_ok());
}

/// The same partial-rule restart driven through the mount reconciler: the
/// service-level wiring must restore unmatched rows while a rule stays active.
#[test]
fn mount_reconciliation_restores_a_partially_removed_rule_set() {
    let dir = tempfile::tempdir().unwrap();
    let first = open_service(dir.path(), &[GIT_RULE, CACHE_RULE]);
    first.reconcile_exclusion_before_mount().unwrap();
    {
        let store = first.bridge.store.lock().unwrap();
        assert!(store.node("200").is_err(), ".git hidden under A+B");
        assert!(store.node("300").is_err(), "cache hidden under A+B");
    }
    drop(first);

    let second = reopen_service(dir.path(), &[CACHE_RULE]);
    second.reconcile_exclusion_before_mount().unwrap();
    {
        let store = second.bridge.store.lock().unwrap();
        assert!(
            store.node("200").is_ok(),
            ".git must be restored after its rule was removed"
        );
        assert!(
            store.node("300").is_err(),
            "cache stays hidden while its rule is active"
        );
        assert!(
            store.next_remove().unwrap().is_none(),
            "reconciliation must not queue a remote delete"
        );
        assert_eq!(store.pending_operations().unwrap(), 0);
    }
    drop(second);

    let mut reopened = Store::open(dir.path()).unwrap();
    assert_eq!(
        reopened.restore_excluded_nodes().unwrap(),
        2,
        "only the cache subtree remains in excluded_purged"
    );
}

/// Removing every rule keeps restoring the whole namespace (issue #50
/// behavior), with no remote delete queued.
#[test]
fn removing_every_rule_restores_the_whole_namespace() {
    let dir = tempfile::tempdir().unwrap();
    let first = open_service(dir.path(), &[GIT_RULE, CACHE_RULE]);
    first.reconcile_exclusion_before_mount().unwrap();
    drop(first);

    let restored = reopen_service(dir.path(), &[]);
    restored.reconcile_exclusion_before_mount().unwrap();
    {
        let store = restored.bridge.store.lock().unwrap();
        for id in ["200", "201", "300", "301"] {
            assert!(store.node(id).is_ok(), "{id} is visible again");
        }
        assert!(store.next_remove().unwrap().is_none());
        assert_eq!(store.pending_operations().unwrap(), 0);
    }
}
