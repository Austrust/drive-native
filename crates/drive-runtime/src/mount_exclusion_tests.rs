//! Issue #50 (B2): a mount that starts in the initialization preview phase must
//! not fail when exclusion rules are configured.
//!
//! `mount_if_ready` supports mounting a preview-only root before the committed
//! namespace exists (`preview_mount_root`). The exclusion index is built from
//! the committed `nodes` table, which is empty during that phase; building it
//! eagerly would return `mount root is missing from the namespace` and the
//! error would escape `mount_if_ready`, taking down the FSD loop. The fix keeps
//! the preview mount alive with an empty index and lets the bridge rebuild the
//! index after initialization commits (owner / recovery snapshot refresh).
use super::*;
use drive_core::exclude::{ExcludeRules, ExclusionState};
use drive_core::initialization_events::InitializationDirectoryEntry;
use drive_core::initialization_store::{
    InitializationBinding, InitializationBudget, InitializationRun,
};
use drive_core::native::Scope;
use std::path::PathBuf;

fn scope() -> Scope {
    Scope {
        view_id: 1,
        node_id: 2,
        file_id: "100".into(),
        path: "/Lab".into(),
    }
}

fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> drive_core::Node {
    drive_core::Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        directory,
        version: serde_json::json!({
            "schema": 1,
            "restore": "server",
            "view": 1,
            "root": "100",
            "file": id,
            "sync": 7,
            "max": 7,
            "size": if directory { 0 } else { 4 },
            "hash": "synthetic",
            "directory": directory,
        })
        .to_string(),
        metadata: Some(drive_core::NodeMetadata {
            size: if directory { 0 } else { 4 },
            mtime: 7,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn root_node() -> drive_core::Node {
    node("100", None, "Lab", true)
}

fn entry(node: drive_core::Node, native_node_id: u64) -> InitializationDirectoryEntry {
    InitializationDirectoryEntry {
        node,
        native_node_id,
    }
}

fn active_state(patterns: &[&str]) -> ExclusionState {
    ExclusionState::new(ExcludeRules::compile(patterns).expect("valid test rules"))
}

fn preview_store(
    dir: &tempfile::TempDir,
) -> (Store, PathBuf, InitializationRun) {
    let mut store = Store::open(dir.path().join("state")).unwrap();
    store
        .bind_native_scope("server", &serde_json::to_string(&scope()).unwrap())
        .unwrap();
    let binding = InitializationBinding {
        task_id: "preview-task".into(),
        scope: serde_json::to_value(scope()).unwrap(),
        restore: "server".into(),
        start_cursor: "native-v1:100".into(),
        expected_cursor: String::new(),
        rules_digest: String::new(),
    };
    let run = store
        .begin_initialization(&binding, &root_node(), InitializationBudget::default())
        .unwrap();
    assert!(
        store.preview_mount_root("100").unwrap().is_some(),
        "fixture must be in the initialization preview phase"
    );
    assert!(
        store.node("100").is_err(),
        "fixture must not have a committed root yet"
    );
    let mount = dir.path().join("mnt");
    std::fs::create_dir(&mount).unwrap();
    (store, mount, run)
}

fn finish_initialization(store: &mut Store, run: &InitializationRun) {
    let ticket = store.begin_initialization_directory(run, "100").unwrap();
    store
        .store_initialization_native_directory(
            &ticket,
            1,
            &[
                entry(node("101", Some("100"), "A.txt", false), 51),
                entry(node("200", Some("100"), "node_modules", true), 71),
            ],
        )
        .unwrap();
    let sub = store.begin_initialization_directory(run, "200").unwrap();
    store
        .store_initialization_native_directory(&sub, 1, &[])
        .unwrap();
    let fence = store.begin_initialization_fence(run).unwrap();
    store
        .confirm_initialization_fence(&fence, "native-v1:100")
        .unwrap();
    store.complete_initialization(run).unwrap();
}

fn preview_profile(mount: PathBuf) -> Profile {
    Profile {
        schema: 1,
        task_id: "d4b8bd5c-8ce8-4c2e-9a39-5f86f4f9b2d1".into(),
        host: "preview.invalid".into(),
        tls_name: "preview.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "tester".into(),
        device_id: "0f4b4d3e-0c9b-4a0b-9f7a-5d2f8c1e6a77".into(),
        scope: scope(),
        mount,
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: vec![r"(^|/)node_modules/".into()],
        exclude_mode: drive_core::exclude::ExcludeMode::default(),
    }
}

/// The FSD calls `mount_if_ready` while the initialization preview namespace is
/// the only mountable root. With active exclusion rules this used to answer
/// `Err(mount root is missing from the namespace)`, which ends the FSD loop.
#[test]
#[ignore = "requires an ordinary Linux user and /dev/fuse; run explicitly"]
fn preview_phase_mount_with_active_exclusion_rules_succeeds() {
    let dir = tempfile::tempdir().unwrap();
    let (store, mount, _run) = preview_store(&dir);
    let service = Service::new(
        preview_profile(mount),
        store,
        PathBuf::from("/nonexistent-worker"),
    )
    .unwrap();
    let session = service
        .mount_if_ready()
        .expect("preview-phase mount must not fail when exclusion rules are active");
    assert!(session.is_some(), "preview root is mountable");
    drop(session);
}

/// The preview phase must keep an empty index instead of failing: the committed
/// namespace does not exist yet.
#[test]
fn preview_phase_skips_the_index_build_and_keeps_it_empty() {
    let dir = tempfile::tempdir().unwrap();
    let (store, _mount, _run) = preview_store(&dir);
    let state = active_state(&[r"(^|/)node_modules/"]);
    refresh_exclusion_before_mount(&state, &store, "100", false)
        .expect("preview-phase refresh must be a no-op");
    assert!(state.is_active());
    assert_eq!(state.current().entries_len(), 0);
    assert_eq!(state.current().inodes_len(), 0);
}

/// Outside the preview phase a broken committed namespace stays a hard error;
/// swallowing it would leak excluded paths into the mount view.
#[test]
fn committed_phase_refresh_errors_are_not_swallowed() {
    let dir = tempfile::tempdir().unwrap();
    let (store, _mount, _run) = preview_store(&dir);
    let state = active_state(&[r"(^|/)node_modules/"]);
    let error = refresh_exclusion_before_mount(&state, &store, "100", true)
        .expect_err("a missing committed root must fail the committed-phase build");
    assert!(
        error.to_string().contains("mount root is missing"),
        "unexpected error: {error}"
    );
}

/// The index is rebuilt from the committed namespace once initialization
/// completes (the bridge calls the same refresh after the owner installs the
/// snapshot); the excluded subtree is then hidden.
#[test]
fn preview_then_committed_refresh_builds_the_index_and_hides_excluded_paths() {
    let dir = tempfile::tempdir().unwrap();
    let (mut store, _mount, run) = preview_store(&dir);
    let state = active_state(&[r"(^|/)node_modules/"]);
    refresh_exclusion_before_mount(&state, &store, "100", false).unwrap();
    assert_eq!(state.current().entries_len(), 0);

    finish_initialization(&mut store, &run);
    refresh_exclusion_before_mount(&state, &store, "100", true)
        .expect("committed refresh after initialization must succeed");

    let index = state.current();
    assert_eq!(index.entries_len(), 1);
    let excluded_ino = store.lookup("100", "node_modules").unwrap().ino;
    let sibling_ino = store.lookup("100", "A.txt").unwrap().ino;
    assert!(
        index.inode_excluded(excluded_ino),
        "excluded directory is indexed"
    );
    assert!(
        !index.inode_excluded(sibling_ino),
        "sibling file stays outside the excluded subtree"
    );
}

/// No rules means no index and no failure in either phase.
#[test]
fn empty_rules_keep_the_mount_refresh_a_no_op() {
    let dir = tempfile::tempdir().unwrap();
    let (store, _mount, _run) = preview_store(&dir);
    let state = ExclusionState::new(ExcludeRules::default());
    refresh_exclusion_before_mount(&state, &store, "100", false).unwrap();
    refresh_exclusion_before_mount(&state, &store, "100", true).unwrap();
    assert!(!state.is_active());
    assert_eq!(state.current().entries_len(), 0);
}
