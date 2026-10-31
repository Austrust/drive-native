//! Calls the actual legacy desktop batch implementation. Local synthetic Store
//! only: never starts Service tasks, mounts FUSE, unlocks, or contacts a server.
use drive_core::{
    Batch, Change, DownloadTicket, Event, Node, Store,
    download_jobs::DownloadFailureKind,
    status_aggregate::Availability,
    status_index::{IndexBudget, IndexOwner},
    status_projection::{Capture, ProjectionBudget, StatusAtom},
};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use std::{path::PathBuf, sync::Arc};
fn node(id: &str, parent: Option<&str>, directory: bool, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        directory,
        version: version.into(),
        metadata: None,
    }
}
fn ready<T: std::fmt::Debug>(value: Capture<T>) -> T {
    match value {
        Capture::Ready(x) => x,
        other => panic!("capture rejected {other:?}"),
    }
}
// Test-only availability oracle. This is deliberately not production index
// wiring: the future runtime source must be identity-bound and asynchronous.
fn verify(store: &Store, atom: &mut StatusAtom) {
    if !atom.directory {
        atom.availability = if store.open_dirty_draft(&atom.id).unwrap().is_some()
            || store.open_cached(&atom.id).is_ok()
        {
            Availability::Complete
        } else {
            Availability::Absent
        };
    }
}
fn assert_parity(service: &Service, index: &IndexOwner) {
    let paths: Vec<String> = [
        "",
        "folder",
        "other",
        "folder/one",
        "folder/two",
        "other/one",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    let legacy: serde_json::Value =
        serde_json::from_str(&desktop::states(service, paths.clone()).unwrap()).unwrap();
    for (path, row) in paths.iter().zip(legacy["entries"].as_array().unwrap()) {
        if !row["error"].is_null() {
            assert!(index.query(path).is_err(), "{path}");
            continue;
        }
        let actual = serde_json::to_value(index.query(path).unwrap()).unwrap();
        for key in ["status", "pinned", "offline_available", "sync_error"] {
            assert_eq!(actual[key], row[key], "{path} {key}");
        }
    }
}
fn advance(service: &Service, index: &mut IndexOwner) {
    let store = service.bridge.store.lock().unwrap();
    let mut delta = ready(
        store
            .capture_status_delta(&index.stamp().database, ProjectionBudget::default())
            .unwrap(),
    );
    for n in &mut delta.nodes {
        if let Some(atom) = &mut n.atom {
            verify(&store, atom)
        }
    }
    index.apply(delta).unwrap();
    drop(store);
    assert_parity(service, index);
}
fn remote(service: &Service, from: &str, to: &str, change: Change) {
    let mut store = service.bridge.store.lock().unwrap();
    store
        .receive(&Batch {
            from: from.into(),
            to: to.into(),
            events: vec![Event {
                id: format!("event-{to}"),
                change,
            }],
        })
        .unwrap();
    store.drain().unwrap();
}
#[test]
fn committed_index_matches_real_desktop_for_cache_pin_update_failure_edit_conflict_and_move() {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true, "v1"),
                node("folder", Some("root"), true, "v1"),
                node("other", Some("root"), true, "v1"),
                node("one", Some("folder"), false, "v1"),
                node("two", Some("folder"), false, "v1"),
            ],
            "1",
            "synthetic",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let mut snapshot = ready(store.snapshot_status(ProjectionBudget::default()).unwrap());
    for atom in &mut snapshot.atoms {
        verify(&store, atom)
    }
    let mut index = IndexOwner::build("root".into(), snapshot, IndexBudget::default()).unwrap();
    let profile = Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "example.invalid".into(),
        tls_name: "example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "synthetic".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/synthetic".into(),
        },
        mount: temp.path().join("mount"),
        writable: false,
    };
    let service: Arc<Service> = Service::new(
        profile,
        store,
        PathBuf::from("/nonexistent-synthetic-worker"),
    )
    .unwrap();
    assert_parity(&service, &index);
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .pin("folder", true)
        .unwrap();
    advance(&service, &mut index);
    for id in ["one", "two"] {
        service
            .bridge
            .store
            .lock()
            .unwrap()
            .publish_download(
                &DownloadTicket {
                    id: id.into(),
                    version: "v1".into(),
                },
                b"abc",
                &drive_core::sha256(b"abc"),
            )
            .unwrap();
        advance(&service, &mut index);
    }
    remote(
        &service,
        "1",
        "2",
        Change::Upsert(node("one", Some("folder"), false, "v2")),
    );
    advance(&service, &mut index);
    {
        let mut s = service.bridge.store.lock().unwrap();
        for _ in 0..5 {
            s.fail_download(
                &DownloadTicket {
                    id: "one".into(),
                    version: "v2".into(),
                },
                DownloadFailureKind::ServerRejected,
                Some(32772),
                100,
            )
            .unwrap();
        }
    }
    advance(&service, &mut index);
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .retry_download("one")
        .unwrap();
    advance(&service, &mut index);
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .save_local("two", b"local edit")
        .unwrap();
    advance(&service, &mut index);
    remote(
        &service,
        "2",
        "3",
        Change::Upsert(node("two", Some("folder"), false, "v2")),
    );
    advance(&service, &mut index);
    remote(
        &service,
        "3",
        "4",
        Change::Upsert(node("one", Some("other"), false, "v2")),
    );
    advance(&service, &mut index);
    remote(&service, "4", "5", Change::Delete { id: "one".into() });
    advance(&service, &mut index);
    assert_eq!(
        service
            .bridge
            .downloads
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}
