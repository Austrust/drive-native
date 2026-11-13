//! Issue #56B: `exclude_mode` selection and the `local_only` quiesce contract.
//!
//! The profile defaults to `local_only` (paths stay usable locally but never
//! sync); `hidden` keeps the issue #50 behaviour. These tests run without a
//! kernel mount so the mode and persistence contract is covered by the ordinary
//! unit suite.
use super::*;
use crate::{Node, NodeMetadata, Store};

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 4 },
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, "Lab", true),
                node("A", Some("root"), "A.txt", false),
                node("B", Some("root"), "B.txt", false),
                node("nm", Some("root"), "node_modules", true),
                node("nmc", Some("nm"), "pkg", true),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store.ensure_filesystem_inodes("root").unwrap();
    (dir, store)
}

#[test]
fn exclude_mode_defaults_to_local_only_and_rejects_unknown_values() {
    assert_eq!(ExcludeMode::default(), ExcludeMode::LocalOnly);
    assert_eq!(ExcludeMode::LocalOnly.as_str(), "local_only");
    assert_eq!(ExcludeMode::Hidden.as_str(), "hidden");
    let parsed: ExcludeMode = serde_json::from_str("\"hidden\"").unwrap();
    assert_eq!(parsed, ExcludeMode::Hidden);
    let error = serde_json::from_str::<ExcludeMode>("\"bogus\"")
        .unwrap_err()
        .to_string();
    assert!(
        error.contains("bogus"),
        "unknown mode error must name the raw value: {error}"
    );
}

#[test]
fn exclusion_state_keeps_the_mode_and_tracks_local_only_nodes() {
    let rules = ExcludeRules::compile(&["^node_modules$"]).unwrap();
    let local = ExclusionState::with_mode(rules.clone(), ExcludeMode::LocalOnly);
    assert_eq!(local.mode(), ExcludeMode::LocalOnly);
    assert!(!local.hides());
    assert!(!local.is_local_only_node("A"));
    local.mark_local_only_node("A");
    assert!(local.is_local_only_node("A"));
    // `new` stays the explicit legacy constructor so every #50 test keeps the
    // hidden behaviour; only the profile decides the default.
    let hidden = ExclusionState::new(rules);
    assert!(hidden.hides());
    assert_eq!(hidden.mode(), ExcludeMode::Hidden);
}

#[test]
fn quiescing_local_only_nodes_removes_sync_intents_but_keeps_local_content() {
    let (_dir, mut store) = fixture();
    let created = store.create_local_file("root", "local.bin", 0o644).unwrap();
    assert_eq!(store.upload_operations("pending").unwrap().len(), 1);
    let quiesced = store
        .quiesce_local_only_nodes(&[created.id.clone()])
        .unwrap();
    assert_eq!(quiesced, 1);
    assert!(store.upload_operations("pending").unwrap().is_empty());
    assert_eq!(store.pending_namespace_operations().unwrap(), 0);
    let kept = store.node(&created.id).expect("local-only node stays visible");
    assert!(!kept.directory);
    assert!(
        store.record(&created.id).is_ok(),
        "the inode mapping must survive"
    );
    assert!(
        store.open_cached(&created.id).is_ok(),
        "local content stays protected from the cleanup"
    );
}

/// Issue #56B F1 (RED first): a `local_only` delete of a remote-bound file leaves
/// a local tombstone plus a pending `remove` namespace operation. Quiescing the
/// explicit id must drop that intent even though the node is now `deleted=1`;
/// otherwise the namespace scheduler could delete the file on the server.
#[test]
fn quiescing_a_local_only_tombstone_drops_the_remove_intent() {
    let (_dir, mut store) = fixture();
    store.unlink_local("root", "A.txt", "A", false).unwrap();
    assert!(
        store.next_remove().unwrap().is_some(),
        "fixture must queue a remove intent before the fix is exercised"
    );
    assert_eq!(store.pending_namespace_operations().unwrap(), 1);
    let quiesced = store
        .quiesce_local_only_nodes(&["A".to_string()])
        .unwrap();
    assert_eq!(quiesced, 1);
    assert!(
        store.next_remove().unwrap().is_none(),
        "local_only must never leave a dispatchable remote remove"
    );
    assert_eq!(store.pending_namespace_operations().unwrap(), 0);
    let deleted: bool = store
        .db
        .query_row("SELECT deleted FROM nodes WHERE id='A'", [], |r| r.get(0))
        .unwrap();
    assert!(deleted, "the node stays a local tombstone");
}

/// Issue #56B kernel-rename follow-up: a `local_only` rename may replace an
/// existing local target. The target is retired locally (tombstone, no sync
/// intent) and the source takes its name; `Store::rename_local` alone refuses
/// the replace because remote replace owns that path outside local_only.
#[test]
fn local_only_rename_replacement_retires_the_target_without_sync() {
    let (_dir, mut store) = fixture();
    store
        .rename_local_replacing("root", "A.txt", "root", "B.txt", false)
        .unwrap();
    assert_eq!(store.lookup("root", "B.txt").unwrap().node.id, "A");
    let deleted: bool = store
        .db
        .query_row("SELECT deleted FROM nodes WHERE id='B'", [], |r| r.get(0))
        .unwrap();
    assert!(deleted, "the replaced target stays a local tombstone");
    assert!(
        store.next_rename().unwrap().is_none(),
        "no rename intent may reach the scheduler"
    );
    assert!(
        store.next_remove().unwrap().is_none(),
        "no remove intent may reach the scheduler"
    );
    assert_eq!(store.pending_namespace_operations().unwrap(), 0);
}
