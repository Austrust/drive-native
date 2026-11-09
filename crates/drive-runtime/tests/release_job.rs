//! Release-space integration coverage: background jobs report progress and a
//! consistent released/skipped/protected result while the synchronous
//! `ReleaseSpace` compatibility path keeps its established reply shape.
use drive_core::{DownloadTicket, Node, NodeMetadata, Status, Store, sha256};
use drive_runtime::{
    desktop,
    service::{Profile, Service},
};
use serde_json::Value;
use std::{
    os::unix::fs::PermissionsExt,
    path::PathBuf,
    sync::Arc,
    time::{Duration, Instant},
};

const MOUNT: &str = "/tmp/drive-native-release-mount";

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn profile() -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "example.invalid".into(),
        tls_name: "example.invalid".into(),
        trusted_ca_pem: Vec::new(),
        username: "test".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/test".into(),
        },
        mount: PathBuf::from(MOUNT),
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: Vec::new(),
    }
}

fn fixture(cached: &[&str]) -> (tempfile::TempDir, Arc<Service>) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("folder", Some("root"), true),
                node("one", Some("folder"), false),
                node("two", Some("folder"), false),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    for id in cached {
        store
            .publish_download(
                &DownloadTicket {
                    id: (*id).into(),
                    version: "1".into(),
                },
                b"abc",
                &sha256(b"abc"),
            )
            .unwrap();
    }
    (
        dir,
        Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap(),
    )
}

fn mount(relative: &str) -> String {
    format!("{MOUNT}/{relative}")
}

fn value(text: &str) -> Value {
    serde_json::from_str(text).unwrap()
}

fn wait_for(service: &Service, job: &str, what: &str, ready: impl Fn(&Value) -> bool) -> Value {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        let snapshot = value(&desktop::release_job(service, job).unwrap());
        if ready(&snapshot) {
            return snapshot;
        }
        assert!(
            Instant::now() < deadline,
            "release job never reported {what}: {snapshot}"
        );
        std::thread::sleep(Duration::from_millis(5));
    }
}

fn finished(service: &Service, job: &str) -> Value {
    wait_for(service, job, "finished", |snapshot| snapshot["finished"] == true)
}

fn begin(service: &Arc<Service>, path: &str) -> (String, Value) {
    let started = value(&desktop::begin_release(service, &[path.into()]).unwrap());
    assert_eq!(started["schema"], 1);
    let job = started["job_id"].as_str().unwrap().to_string();
    (job, started)
}

fn strings(value: &Value, key: &str) -> Vec<String> {
    value[key]
        .as_array()
        .unwrap()
        .iter()
        .map(|entry| entry.as_str().unwrap().to_string())
        .collect()
}

fn sorted_strings(value: &Value, key: &str) -> Vec<String> {
    let mut entries = strings(value, key);
    entries.sort();
    entries
}

#[test]
fn release_job_reports_progress_and_reclaims_cached_bodies() {
    let (_dir, service) = fixture(&["one", "two"]);
    let (job, started) = begin(&service, &mount("folder"));
    assert_eq!(started["total"], 2);

    let mut last_done = 0;
    let final_state = loop {
        let snapshot = value(&desktop::release_job(&service, &job).unwrap());
        assert_eq!(snapshot["schema"], 1);
        assert_eq!(snapshot["job_id"], job.as_str());
        assert_eq!(snapshot["total"], 2);
        let done = snapshot["done"].as_u64().unwrap();
        assert!(done >= last_done, "progress must never move backwards");
        last_done = done;
        if snapshot["finished"] == true {
            break snapshot;
        }
        std::thread::sleep(Duration::from_millis(2));
    };

    assert_eq!(final_state["available"], true);
    assert_eq!(final_state["finished"], true);
    assert_eq!(final_state["cancelled"], false);
    assert_eq!(final_state["done"], 2);
    assert_eq!(final_state["released_count"], 2);
    assert_eq!(final_state["skipped_count"], 0);
    assert_eq!(final_state["protected_count"], 0);
    assert_eq!(final_state["reclaimed_bytes"], 6);
    assert_eq!(sorted_strings(&final_state, "released"), vec!["one", "two"]);
    assert!(strings(&final_state, "skipped").is_empty());
    assert!(strings(&final_state, "protected_or_failed").is_empty());

    let store = service.bridge.store.lock().unwrap();
    assert_eq!(store.status("one").unwrap(), Status::OnlineOnly);
    assert_eq!(store.status("two").unwrap(), Status::OnlineOnly);
    drop(store);
    assert_eq!(
        service.bridge.store.lock().unwrap().collect_garbage().unwrap(),
        0
    );
}

#[test]
fn release_job_skips_already_online_only_files_instead_of_faking_release() {
    let (_dir, service) = fixture(&["two"]);
    let (job, started) = begin(&service, &mount("folder"));
    assert_eq!(started["total"], 2);
    let final_state = finished(&service, &job);
    assert_eq!(sorted_strings(&final_state, "released"), vec!["two"]);
    assert_eq!(sorted_strings(&final_state, "skipped"), vec!["one"]);
    assert!(strings(&final_state, "protected_or_failed").is_empty());
    assert_eq!(final_state["released_count"], 1);
    assert_eq!(final_state["skipped_count"], 1);
    assert_eq!(final_state["protected_count"], 0);
    assert_eq!(final_state["reclaimed_bytes"], 3);
    assert_eq!(final_state["done"], 2);
}

#[test]
fn unknown_job_id_is_reported_unavailable_instead_of_failing_the_client() {
    let (_dir, service) = fixture(&[]);
    let snapshot = value(&desktop::release_job(&service, "424242").unwrap());
    assert_eq!(snapshot["schema"], 1);
    assert_eq!(snapshot["available"], false);
    assert_eq!(snapshot["job_id"], "424242");
}

#[test]
fn release_jobs_are_memory_only_and_do_not_survive_a_service_restart() {
    let (dir, service) = fixture(&["one"]);
    let (job, _started) = begin(&service, &mount("folder"));
    let final_state = finished(&service, &job);
    assert_eq!(final_state["released_count"], 1);

    drop(service);
    let store = Store::open(dir.path()).unwrap();
    let restarted =
        Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
    let snapshot = value(&desktop::release_job(&restarted, &job).unwrap());
    assert_eq!(snapshot["available"], false);
    assert_eq!(snapshot["job_id"], job.as_str());
}

#[test]
fn overview_and_cache_settings_expose_release_job_additively() {
    let (_dir, service) = fixture(&["one", "two"]);
    let overview = value(&desktop::overview(&service).unwrap());
    assert_eq!(overview["schema"], 1);
    assert_eq!(overview["release_job"], Value::Null);
    // Unrelated protocol fields stay untouched by the additive key.
    assert_eq!(overview["version"]["ipc"], 21);
    assert_eq!(
        overview["version"]["state_schema"],
        drive_core::STATE_SCHEMA_VERSION
    );
    let settings = value(&desktop::cache_settings(&service).unwrap());
    assert_eq!(settings["schema"], 1);
    assert_eq!(settings["release_job"], Value::Null);
    assert!(settings["usage"]["limit_bytes"].is_u64());

    let (job, _started) = begin(&service, &mount("folder"));
    finished(&service, &job);

    let overview = value(&desktop::overview(&service).unwrap());
    let summary = &overview["release_job"];
    assert_eq!(summary["job_id"], job.as_str());
    assert_eq!(summary["total"], 2);
    assert_eq!(summary["done"], 2);
    assert_eq!(summary["reclaimed_bytes"], 6);
    assert_eq!(summary["finished"], true);
    assert_eq!(summary["cancelled"], false);
    let settings = value(&desktop::cache_settings(&service).unwrap());
    assert_eq!(settings["schema"], 1);
    assert_eq!(settings["release_job"]["job_id"], job.as_str());
    assert!(settings["usage"]["limit_bytes"].is_u64());
    assert_eq!(settings["connection"], overview["connection"]);
}

#[test]
fn synchronous_release_keeps_its_result_shape_and_distinguishes_skipped() {
    let (_dir, service) = fixture(&["one", "two"]);
    let reply = value(&desktop::release(&service, &mount("folder")).unwrap());
    assert_eq!(reply["schema"], 1);
    assert_eq!(reply["reclaimed_bytes"], 6);
    assert_eq!(sorted_strings(&reply, "released"), vec!["one", "two"]);
    assert!(strings(&reply, "protected_or_failed").is_empty());
    assert!(strings(&reply, "skipped").is_empty());

    // The second pass has nothing left to release; it reports skipped instead
    // of pretending the files were released again.
    let reply = value(&desktop::release(&service, &mount("folder")).unwrap());
    assert_eq!(reply["reclaimed_bytes"], 0);
    assert!(strings(&reply, "released").is_empty());
    assert!(strings(&reply, "protected_or_failed").is_empty());
    assert_eq!(sorted_strings(&reply, "skipped"), vec!["one", "two"]);
}
