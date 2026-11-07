use drive_core::{Batch, Change, Event, Node, NodeMetadata, Store};
use drive_runtime::{
    desktop,
    service::{ConnectionState, Profile, Service},
};
use std::{os::unix::fs::PermissionsExt, path::PathBuf};
fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
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
        mount: PathBuf::from("/tmp/drive-native-test-mount"),
        writable: false, local_policy: None, writable_source: None,
    }
}
fn fixture() -> (tempfile::TempDir, std::sync::Arc<Service>) {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
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
    s.ensure_fs_root("root").unwrap();
    (
        d,
        Service::new(profile(), s, PathBuf::from("/nonexistent-worker")).unwrap(),
    )
}
#[test]
fn legacy_profiles_keep_their_original_read_only_choice() {
    let original=profile();let mut value=serde_json::to_value(&original).unwrap();
    value.as_object_mut().unwrap().remove("local_policy");value.as_object_mut().unwrap().remove("writable_source");
    for (writable,expected) in [(false,drive_core::write_permissions::LocalPolicy::ExplicitReadOnly),(true,drive_core::write_permissions::LocalPolicy::FollowRemote)] {
        value["writable"]=writable.into();let decoded:Profile=serde_json::from_value(value.clone()).unwrap();assert_eq!(decoded.effective_local_policy(),expected);
    }
}
#[test]
fn permission_overview_query_and_user_policy_revision_are_consistent() {
    let(_d,s)=fixture();
    let overview:serde_json::Value=serde_json::from_str(&desktop::overview(&s).unwrap()).unwrap();
    assert_eq!(overview["local_policy"],"explicit_read_only");assert_eq!(overview["writable_source"],"legacy");assert_eq!(overview["server_capability"],"unknown");
    let query:serde_json::Value=serde_json::from_str(&desktop::directory_permissions(&s,r#"{"path":"folder"}"#).unwrap()).unwrap();
    assert_eq!(query["actions"]["create_dir"]["state"],"denied");assert_eq!(query["actions"]["list"]["state"],"unknown");
    let request=serde_json::json!({"task_id":s.profile.task_id,"expected_revision":overview["write_policy_revision"],"local_policy":"explicit_read_only"}).to_string();
    desktop::set_write_policy(&s,&request).unwrap();assert!(desktop::set_write_policy(&s,&request).is_err());
}
fn get(s: &Service, path: &str) -> serde_json::Value {
    serde_json::from_str::<serde_json::Value>(&desktop::states(s, vec![path.into()]).unwrap())
        .unwrap()["entries"][0]
        .clone()
}

fn remote_history(service: &Service) -> i64 {
    let mut store=service.bridge.store.lock().unwrap();
    let mut changed=store.node("one").unwrap();
    changed.version="v2".into();
    store.receive(&Batch { from:"1".into(),to:"2".into(),events:vec![Event {
        id:"history-event".into(),change:Change::Upsert(changed),
    }]}).unwrap();
    store.drain().unwrap();
    store.sync_history(&Default::default()).unwrap().records[0].id
}

#[test]
fn history_target_handles_online_offline_cached_deleted_and_unmounted() {
    let (_dir,service)=fixture();
    let id=remote_history(&service);
    let query=serde_json::json!({"id":id}).to_string();
    service.mounted.store(true,std::sync::atomic::Ordering::Release);
    service.control.lock().unwrap().state=ConnectionState::Online;
    let target=||serde_json::from_str::<serde_json::Value>(&desktop::resolve_history_target(&service,&query).unwrap()).unwrap();
    let online=target();
    assert_eq!(online["reason"],"online_only");
    assert_eq!(online["exists"],true);
    assert_eq!(online["offline_available"],false);
    assert_eq!(online["node"],"one");
    assert_eq!(online["path"],"/tmp/drive-native-test-mount/folder/one");
    service.control.lock().unwrap().state=ConnectionState::Offline;
    let offline=target();
    assert_eq!(offline["reason"],"offline");
    assert!(offline["path"].is_null());
    {
        let mut store=service.bridge.store.lock().unwrap();
        store.publish_download(&drive_core::DownloadTicket{id:"one".into(),version:"v2".into()},b"abc",&drive_core::sha256(b"abc")).unwrap();
    }
    let cached=target();
    assert_eq!(cached["offline_available"],true);
    assert!(!cached["path"].is_null());
    assert!(cached["reason"].is_null());
    {
        let mut store=service.bridge.store.lock().unwrap();
        store.receive(&Batch{from:"2".into(),to:"3".into(),events:vec![Event{id:"history-deletion".into(),change:Change::Delete{id:"one".into()}}]}).unwrap();
        store.drain().unwrap();
    }
    let deleted=target();
    assert_eq!(deleted["reason"],"deleted");
    assert_eq!(deleted["exists"],false);
    assert_eq!(deleted["parent_path"],"/tmp/drive-native-test-mount/folder");
    service.mounted.store(false,std::sync::atomic::Ordering::Release);
    let unmounted=target();
    assert!(unmounted["path"].is_null());
    assert!(unmounted["parent_path"].is_null());
}

#[test]
fn history_target_follows_renamed_identity_instead_of_event_path() {
    let (_dir,service)=fixture();
    let id=remote_history(&service);
    service.mounted.store(true,std::sync::atomic::Ordering::Release);
    service.control.lock().unwrap().state=ConnectionState::Online;
    service.bridge.store.lock().unwrap().rename_local("folder","one","root","renamed.txt",true).unwrap();
    let result:serde_json::Value=serde_json::from_str(&desktop::resolve_history_target(&service,&serde_json::json!({"id":id}).to_string()).unwrap()).unwrap();
    assert_eq!(result["path"],"/tmp/drive-native-test-mount/renamed.txt");
    assert_eq!(result["parent_path"],"/tmp/drive-native-test-mount/");
}

#[test]
fn history_queries_clear_only_history_and_reject_unsafe_clear_requests() {
    let (_dir,service)=fixture();
    remote_history(&service);
    service.bridge.store.lock().unwrap().save_local("two",b"keep dirty body").unwrap();
    let page:serde_json::Value=serde_json::from_str(&desktop::sync_history(&service,r#"{"mount":"root","result":"ok","name":"ONE","limit":1}"#).unwrap()).unwrap();
    assert_eq!(page["records"].as_array().unwrap().len(),1);
    assert_eq!(page["records"][0]["action"],"remote_change");
    assert!(desktop::clear_sync_history(&service,r#"{"mode":"older_than_days","days":-2}"#).is_err());
    assert!(desktop::clear_sync_history(&service,r#"{"mode":"all","unexpected":true}"#).is_err());
    let cleared:serde_json::Value=serde_json::from_str(&desktop::clear_sync_history(&service,r#"{"mode":"all"}"#).unwrap()).unwrap();
    assert_eq!(cleared["removed"],1);
    let store=service.bridge.store.lock().unwrap();
    assert_eq!(store.open_cached("two").unwrap().size().unwrap(),15);
    assert_eq!(store.pending_operations().unwrap(),1);
}

#[test]
fn history_ipc_busy_returns_without_waiting_for_store_or_control() {
    let (_dir,service)=fixture();
    let _store=service.bridge.store.lock().unwrap();
    for response in [desktop::sync_history(&service,"{}"),desktop::recent_activity(&service,"{}"),
        desktop::clear_sync_history(&service,r#"{"mode":"all"}"#),desktop::resolve_history_target(&service,r#"{"id":1}"#)] {
        assert_eq!(serde_json::from_str::<serde_json::Value>(&response.unwrap()).unwrap()["error"],"busy");
    }
    drop(_store);
    let _control=service.control.lock().unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&desktop::resolve_history_target(&service,r#"{"id":1}"#).unwrap()).unwrap()["error"],"busy");
}

#[test]
fn history_target_can_open_an_offline_dirty_draft_without_any_cached_blob() {
    let (_dir,service)=fixture();
    let id=remote_history(&service);
    service.mounted.store(true,std::sync::atomic::Ordering::Release);
    service.control.lock().unwrap().state=ConnectionState::Offline;
    {
        let mut store=service.bridge.store.lock().unwrap();
        assert!(store.open_cached("one").is_err());
        let draft=store.begin_draft("one","v2",None).unwrap();
        store.draft_truncate(&draft,0).unwrap();
        store.draft_write(&draft,0,b"uncheckpointed draft",false).unwrap();
        assert!(store.open_cached("one").is_err());
        assert!(store.open_dirty_draft("one").unwrap().is_some());
    }
    let target:serde_json::Value=serde_json::from_str(&desktop::resolve_history_target(&service,
        &serde_json::json!({"id":id}).to_string()).unwrap()).unwrap();
    assert_eq!(target["offline_available"],true);
    assert_eq!(target["mount_online"],false);
    assert_eq!(target["path"],"/tmp/drive-native-test-mount/folder/one");
    assert!(target["reason"].is_null());
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
#[ignore = "run under a dedicated dbus-run-session; no network worker or NAS"]
async fn recovery_action_real_dbus_preserves_unknown_queue_and_user_pause() {
    assert!(std::env::var_os("DBUS_SESSION_BUS_ADDRESS").is_some());
    let (_dir, service) = fixture();
    {
        let mut store = service.bridge.store.lock().unwrap();
        let op = store.save_local("one", b"must survive recovery request").unwrap();
        store.mark_upload_started(&op).unwrap();
        store.mark_upload_uncertain(&op).unwrap();
    }
    let name = format!("org.drivenative.RecoveryTest.t{}", uuid::Uuid::new_v4().simple());
    let server = zbus::connection::Builder::session()
        .name(name.as_str())
        .serve_at(desktop::OBJECT_PATH, desktop::Desktop(service.clone()))
        .build().await.unwrap();
    let client = zbus::Connection::session().await.unwrap();
    let proxy = zbus::Proxy::new(&client, name.as_str(), desktop::OBJECT_PATH, desktop::BUS_NAME).await.unwrap();
    let overview: String = proxy.call("GetOverview", &()).await.unwrap();
    let overview: serde_json::Value = serde_json::from_str(&overview).unwrap();
    assert_eq!(overview["recovery_actions"]["recheck_issues"], true);
    let initial: String = proxy.call("RecheckIssues", &()).await.unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&initial).unwrap()["message_code"], "login_required");
    let _: () = proxy.call("Unlock", &("synthetic-not-sent",)).await.unwrap();
    service.control.lock().unwrap().state = ConnectionState::Online;
    let queued: String = proxy.call("RecheckIssues", &()).await.unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&queued).unwrap()["message_code"], "queued");
    let _: () = proxy.call("Pause", &()).await.unwrap();
    let paused: String = proxy.call("RecheckIssues", &()).await.unwrap();
    let paused: serde_json::Value = serde_json::from_str(&paused).unwrap();
    assert_eq!(paused["message_code"], "paused");
    assert_eq!(paused["accepted"], false);
    let store = service.bridge.store.lock().unwrap();
    assert!(store.sync_paused().unwrap());
    assert_eq!(store.queue_health().unwrap().upload_uncertain, 1);
    assert_eq!(store.pending_operations().unwrap(), 1);
    drop(store);
    let history:String=proxy.call("GetSyncHistory", &(r#"{"limit":1}"#,)).await.unwrap();
    let history:serde_json::Value=serde_json::from_str(&history).unwrap();
    assert_eq!(history["records"][0]["result"],"pending");
    assert_eq!(history["records"][0]["error_kind"],"uncertain_commit");
    let recent:String=proxy.call("GetRecentActivity", &(r#"{"limit":12}"#,)).await.unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&recent).unwrap()["records"].as_array().unwrap().len(),1);
    let target_request=serde_json::json!({"id":history["records"][0]["id"]}).to_string();
    let target:String=proxy.call("ResolveHistoryTarget", &(target_request,)).await.unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&target).unwrap()["exists"],false);
    let cleared:String=proxy.call("ClearSyncHistory", &(r#"{"mode":"all"}"#,)).await.unwrap();
    assert_eq!(serde_json::from_str::<serde_json::Value>(&cleared).unwrap()["removed"],1);
    assert_eq!(service.bridge.store.lock().unwrap().pending_operations().unwrap(),1);
    drop(proxy);drop(client);drop(server);
}

#[test]
fn overview_reports_exact_unknown_commits_without_running_dependency_scheduler() {
    let (_d, service) = fixture();
    {
        let mut store = service.bridge.store.lock().unwrap();
        // The shared service fixture is legacy read-only. Authorize these
        // synthetic jobs before exercising the scheduler and unknown commits.
        drive_core::write_permissions::set_local_policy(&store, "root",
            drive_core::write_permissions::LocalPolicy::FollowRemote).unwrap();
        let upload = store.save_local("one", b"unconfirmed local edit").unwrap();
        store.mark_upload_started(&upload).unwrap();
        store.mark_upload_uncertain(&upload).unwrap();
        store
            .create_local_directory("root", "new-directory", 0o700)
            .unwrap();
        let operation = store.next_directory().unwrap().unwrap().job.operation;
        store.mark_directory_started(&operation).unwrap();
        store.mark_directory_uncertain(&operation).unwrap();
    }
    let before = service.bridge.store.lock().unwrap().queue_health().unwrap();
    let overview: serde_json::Value =
        serde_json::from_str(&desktop::overview(&service).unwrap()).unwrap();
    let health = &overview["queue_health"];
    assert_eq!(health["namespace_uncertain"], 1);
    assert_eq!(health["upload_uncertain"], 1);
    assert_eq!(health["namespace_unfinished"], 1);
    assert_eq!(health["upload_unfinished"], 1);
    assert_eq!(
        health["schedule"],
        serde_json::to_value(before.schedule).unwrap()
    );
    assert_eq!(health["recovery_attempts_observed"], 0);
    assert_eq!(health["next_recovery_ms"], serde_json::Value::Null);
    // Overview neither issues observations nor refreshes the cached schedule.
    let after = service.bridge.store.lock().unwrap().queue_health().unwrap();
    assert!(after.schedule_age_ms >= before.schedule_age_ms);
    assert_eq!(after.recovery_attempts_observed, 0);
}

#[test]
fn initialization_remains_available_while_store_and_control_are_held() {
    let (_root, service) = fixture();
    service.bridge.initialization.begin();
    service
        .bridge
        .initialization
        .phase(drive_runtime::initialization::Phase::Importing);
    let store = service.bridge.store.lock().unwrap();
    let control = service.control.lock().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let reader = service.clone();
    let thread = std::thread::spawn(move || send.send(desktop::initialization(&reader)).unwrap());
    let result = receive.recv_timeout(std::time::Duration::from_secs(1));
    drop(control);
    drop(store);
    thread.join().unwrap();
    let data: serde_json::Value =
        serde_json::from_str(&result.expect("progress must not wait for Store/control")).unwrap();
    assert_eq!(data["schema"], 1);
    assert_eq!(data["task_id"], service.profile.task_id);
    assert_eq!(data["initialization"]["phase"], "importing");
}

#[test]
fn directory_pages_are_complete_without_inode_allocation_or_downloads() {
    let d = tempfile::tempdir().unwrap();
    let mut store = Store::open(d.path()).unwrap();
    let mut nodes = vec![node("root", None, true)];
    nodes.extend(
        (0..513)
            .rev()
            .map(|i| node(&format!("item-{i:04}"), Some("root"), false)),
    );
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    let service = Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
    // Service startup persists the task policy once. Measure only the queries,
    // which must still allocate no inodes, hydrate nothing, and mutate no state.
    let before = service.bridge.store.lock().unwrap().local_revision();
    let mut paths = Vec::new();
    for (offset, expected_count) in [(0, 256), (256, 256), (512, 1), (513, 0), (u32::MAX, 0)] {
        let page: serde_json::Value =
            serde_json::from_str(&desktop::directory(&service, "", offset).unwrap()).unwrap();
        assert_eq!(page["total"], 513);
        assert_eq!(page["offset"], offset);
        let entries = page["entries"].as_array().unwrap();
        assert_eq!(entries.len(), expected_count);
        for entry in entries {
            assert_eq!(entry["status"], "online_only");
            assert!(entry["error"].is_null());
            paths.push(entry["path"].as_str().unwrap().to_owned());
        }
    }
    assert_eq!(
        paths,
        (0..513).map(|i| format!("item-{i:04}")).collect::<Vec<_>>()
    );
    assert_eq!(
        service.bridge.store.lock().unwrap().local_revision(),
        before
    );
    assert!(
        service
            .bridge
            .store
            .lock()
            .unwrap()
            .downloads_needed()
            .unwrap()
            .is_empty()
    );
    assert_eq!(
        std::fs::read_dir(d.path().join("blobs")).unwrap().count(),
        0
    );
}

#[test]
fn large_online_only_tree_has_bounded_local_status_cost() {
    let d = tempfile::tempdir().unwrap();
    let mut store = Store::open(d.path()).unwrap();
    let mut nodes = vec![node("root", None, true)];
    for folder in 0..100 {
        let id = format!("folder-{folder}");
        nodes.push(node(&id, Some("root"), true));
        for file in 0..100 {
            nodes.push(node(&format!("file-{folder}-{file}"), Some(&id), false));
        }
    }
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    assert!(store.downloads_needed().unwrap().is_empty());
    let service = Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
    let start = std::time::Instant::now();
    let state = get(&service, ".");
    assert_eq!(state["status"], "online_only");
    assert_eq!(state["offline_available"], false);
    // Deliberately generous: catches recursive per-node SQL regressions without
    // requiring benchmark-grade timing on ordinary CI machines.
    assert!(start.elapsed() < std::time::Duration::from_secs(15));
    assert_eq!(
        std::fs::read_dir(d.path().join("blobs")).unwrap().count(),
        0
    );
}
#[test]
fn folder_completion_follows_every_child_and_stale_cache_remains_visible() {
    let (_d, s) = fixture();
    assert_eq!(get(&s, "folder")["status"], "online_only");
    {
        let mut store = s.bridge.store.lock().unwrap();
        store.pin("folder", true).unwrap();
        store
            .publish_download(
                &drive_core::DownloadTicket {
                    id: "one".into(),
                    version: "v1".into(),
                },
                b"one",
                &drive_core::sha256(b"one"),
            )
            .unwrap();
    }
    assert_eq!(get(&s, "folder")["status"], "pinning");
    assert_eq!(get(&s, "folder")["offline_available"], false);
    {
        let mut store = s.bridge.store.lock().unwrap();
        store
            .publish_download(
                &drive_core::DownloadTicket {
                    id: "two".into(),
                    version: "v1".into(),
                },
                b"two",
                &drive_core::sha256(b"two"),
            )
            .unwrap();
    }
    assert_eq!(get(&s, "folder")["status"], "pinned_complete");
    {
        let mut store = s.bridge.store.lock().unwrap();
        let mut updated = node("one", Some("folder"), false);
        updated.version = "v2".into();
        store
            .receive(&Batch {
                from: "1".into(),
                to: "2".into(),
                events: vec![Event {
                    id: "e".into(),
                    change: Change::Upsert(updated),
                }],
            })
            .unwrap();
        store.drain().unwrap();
    }
    assert_eq!(get(&s, "folder")["status"], "updating");
    assert_eq!(get(&s, "folder/one")["offline_available"], true);
    assert_eq!(
        s.bridge
            .downloads
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}
#[test]
fn metadata_queries_reject_traversal_and_never_hydrate() {
    let (_d, s) = fixture();
    for path in [
        "../secret",
        "/etc/passwd",
        "folder/../../secret",
        "folder/one/child",
        "folder\0",
    ] {
        assert!(!get(&s, path)["error"].is_null(), "{path}");
    }
    assert_eq!(
        get(&s, "/tmp/drive-native-test-mount/folder/one")["status"],
        "online_only"
    );
    assert!(desktop::states(&s, vec!["folder".into(); 257]).is_err());
    assert_eq!(
        s.bridge
            .downloads
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}
#[test]
fn locked_service_has_no_worker_and_pause_resume_stop_need_no_credentials() {
    let (_d, s) = fixture();
    let tasks = s.start();
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Locked);
    s.pause().unwrap();
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Paused);
    s.resume().unwrap();
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Locked);
    assert!(s.unlock(String::new()).is_err());
    s.shutdown();
    tasks.join();
    assert!(s.unlock("must not restart".into()).is_err());
}
#[test]
fn private_profile_refuses_extra_secret_fields_symlink_and_shared_permissions() {
    let d = tempfile::tempdir().unwrap();
    let p = d.path().join("profile.json");
    let mut value = serde_json::to_value(profile()).unwrap();
    value["password"] = "not-a-real-secret".into();
    std::fs::write(&p, serde_json::to_vec(&value).unwrap()).unwrap();
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o600)).unwrap();
    assert!(Profile::read(&p).is_err());
    value.as_object_mut().unwrap().remove("password");
    std::fs::write(&p, serde_json::to_vec(&value).unwrap()).unwrap();
    assert!(Profile::read(&p).is_ok());
    let link = d.path().join("link.json");
    std::os::unix::fs::symlink(&p, &link).unwrap();
    assert!(Profile::read(&link).is_err());
    std::fs::set_permissions(&p, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(Profile::read(&p).is_err());
}
#[test]
fn missing_content_does_not_claim_offline_ready_or_folder_synced() {
    let (d, s) = fixture();
    {
        let mut store = s.bridge.store.lock().unwrap();
        store
            .publish_download(
                &drive_core::DownloadTicket {
                    id: "one".into(),
                    version: "v1".into(),
                },
                b"one",
                &drive_core::sha256(b"one"),
            )
            .unwrap();
    }
    for blob in std::fs::read_dir(d.path().join("blobs")).unwrap() {
        std::fs::remove_file(blob.unwrap().path()).unwrap();
    }
    assert!(!get(&s, "folder/one")["error"].is_null());
    assert!(!get(&s, "folder")["error"].is_null());
}

#[test]
fn permanent_startup_failure_requires_explicit_action_and_diagnostics_hide_credentials() {
    let (_d, s) = fixture();
    let tasks = s.start();
    let secret = "synthetic-private-input";
    s.unlock(secret.into()).unwrap();
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
    while s.control.lock().unwrap().state != ConnectionState::NeedsAttention {
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    let overview = desktop::overview(&s).unwrap();
    assert!(!overview.contains(secret));
    s.pause().unwrap();
    s.resume().unwrap();
    assert_eq!(
        s.control.lock().unwrap().state,
        ConnectionState::NeedsAttention
    );
    s.shutdown();
    tasks.join();
}
#[test]
fn handshake_failures_distinguish_transient_disconnect_from_rejected_login() {
    use drive_runtime::{bridge::Bridge, rpc::Rpc, transport, wire::*};
    use std::os::unix::net::UnixStream;
    for (category, kind) in [
        ("transport", std::io::ErrorKind::NotConnected),
        ("tls_or_configuration", std::io::ErrorKind::InvalidInput),
        (
            "authentication_or_protocol",
            std::io::ErrorKind::PermissionDenied,
        ),
    ] {
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Rpc::new(a).unwrap();
        let peer = std::thread::spawn(move || {
            let (request, fd) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(fd.is_none());
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body: Response::Failed {
                        category: category.into(),
                    },
                },
                None,
            )
            .unwrap();
        });
        let p = profile();
        let error = Bridge::authenticate(
            &rpc,
            Login {
                host: p.host,
                tls_name: p.tls_name,
                trusted_ca_pem: Vec::new(),
                username: p.username,
                password: "synthetic-private-input".into(),
                device_id: p.device_id,
                scope: p.scope,
            },
        )
        .unwrap_err();
        assert_eq!(error.kind(), kind);
        assert!(!error.to_string().contains("synthetic-private-input"));
        peer.join().unwrap();
    }
}

#[test]
fn directory_pages_are_bounded_stable_and_available_before_unlock() {
    let d = tempfile::tempdir().unwrap();
    let mut store = Store::open(d.path()).unwrap();
    let mut nodes = vec![node("root", None, true)];
    for i in 0..300 {
        nodes.push(node(&format!("file-{i:03}"), Some("root"), false));
    }
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    let s = Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
    let page = |path: &str, offset: u32| -> serde_json::Value {
        serde_json::from_str(&desktop::directory(&s, path, offset).unwrap()).unwrap()
    };
    let first = page("", 0);
    let last = page("", 256);
    assert_eq!(first["total"], 300);
    assert_eq!(first["entries"].as_array().unwrap().len(), 256);
    assert_eq!(first["entries"][0]["path"], "file-000");
    assert_eq!(last["entries"].as_array().unwrap().len(), 44);
    assert_eq!(last["entries"][0]["path"], "file-256");
    assert_eq!(page("", 300)["entries"].as_array().unwrap().len(), 0);
    assert_eq!(
        page("/tmp/drive-native-test-mount", 0)["entries"][0]["status"],
        "online_only"
    );
    for path in ["file-000", "..", "/etc", "missing"] {
        assert!(desktop::directory(&s, path, 0).is_err());
    }
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Locked);
    assert_eq!(
        s.bridge
            .downloads
            .load(std::sync::atomic::Ordering::Relaxed),
        0
    );
}

#[test]
fn persistent_download_failure_propagates_to_folder_and_retry_is_read_only() {
    use drive_core::download_jobs::DownloadFailureKind;
    let (_d, s) = fixture();
    {
        let mut store = s.bridge.store.lock().unwrap();
        let ticket = drive_core::DownloadTicket {
            id: "one".into(),
            version: "v1".into(),
        };
        store.pin("folder", true).unwrap();
        for _ in 0..5 {
            store
                .fail_download(
                    &ticket,
                    DownloadFailureKind::ServerRejected,
                    Some(32772),
                    100,
                )
                .unwrap();
        }
    }
    assert_eq!(get(&s, "folder/one")["status"], "download_failed");
    assert_eq!(get(&s, "folder")["status"], "download_failed");
    assert_eq!(get(&s, "folder/one")["sync_error"]["server_code"], 32772);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&desktop::overview(&s).unwrap()).unwrap()["download_errors"],
        1
    );
    let result: serde_json::Value =
        serde_json::from_str(&desktop::retry_downloads(&s, "folder").unwrap()).unwrap();
    assert_eq!(result["reset_downloads"], 1);
    assert_eq!(get(&s, "folder")["status"], "pinning");
    let store = s.bridge.store.lock().unwrap();
    assert_eq!(store.pending_operations().unwrap(), 0);
    assert_eq!(store.pending_namespace_operations().unwrap(), 0);
}

#[test]
fn cache_settings_are_task_bound_and_never_erase_content_while_paused() {
    let (_dir, s) = fixture();
    let before = desktop::cache_settings(&s).unwrap();
    let before: serde_json::Value = serde_json::from_str(&before).unwrap();
    assert_eq!(before["usage"]["limit_bytes"], 0);
    for invalid in [
        serde_json::json!({"schema":1,"task_id":"other","limit_bytes":1048576}),
        serde_json::json!({"schema":1,"task_id":s.profile.task_id,"limit_bytes":1}),
        serde_json::json!({"schema":2,"task_id":s.profile.task_id,"limit_bytes":1048576}),
    ] {
        assert!(desktop::set_cache_settings(&s, &invalid.to_string()).is_err());
    }
    s.pause().unwrap();
    let body = serde_json::json!({"schema":1,"task_id":s.profile.task_id,"limit_bytes":1048576})
        .to_string();
    let result: serde_json::Value =
        serde_json::from_str(&desktop::set_cache_settings(&s, &body).unwrap()).unwrap();
    assert_eq!(result["usage"]["limit_bytes"], 1048576);
    assert_eq!(result["connection"], "paused");
    assert_eq!(
        s.bridge.store.lock().unwrap().pending_operations().unwrap(),
        0
    );
}

#[test]
fn trust_settings_bind_task_and_version_and_preserve_pause_and_cached_content() {
    let (_d, s) = fixture();
    let initial: serde_json::Value =
        serde_json::from_str(&desktop::trust_settings(&s).unwrap()).unwrap();
    let original_version = initial["version"].as_str().unwrap();
    let ca = include_str!("../../sproto/tests/fixtures/trust/ca.pem");
    let make = |task: &str, version: &str| {
        serde_json::json!({"expected_version":version,"settings":{"schema":1,"task_id":task,"trusted_ca_pem":[ca]}}).to_string()
    };
    assert!(desktop::set_trust_settings(&s, &make("wrong-task", original_version)).is_err());
    assert!(desktop::set_trust_settings(&s, &make(&s.profile.task_id, "stale")).is_err());
    s.pause().unwrap();
    let result: serde_json::Value = serde_json::from_str(
        &desktop::set_trust_settings(&s, &make(&s.profile.task_id, original_version)).unwrap(),
    )
    .unwrap();
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Paused);
    assert_eq!(result["fingerprints"].as_array().unwrap().len(), 1);
    assert_ne!(result["version"], initial["version"]);
    assert!(desktop::set_trust_settings(&s, &make(&s.profile.task_id, original_version)).is_err());
    assert_eq!(get(&s, "folder/one")["status"], "online_only");
    let store = s.bridge.store.lock().unwrap();
    assert_eq!(store.children("folder").unwrap().len(), 2);
    assert!(store.upload_operations("pending").unwrap().is_empty());
}
#[test]
fn failed_trust_publication_does_not_change_runtime_settings() {
    let (d, s) = fixture();
    std::fs::create_dir(d.path().join("connection-trust.json")).unwrap();
    let before: serde_json::Value =
        serde_json::from_str(&desktop::trust_settings(&s).unwrap()).unwrap();
    let body=serde_json::json!({"expected_version":before["version"],"settings":{"schema":1,"task_id":s.profile.task_id,"trusted_ca_pem":[include_str!("../../sproto/tests/fixtures/trust/ca.pem")]}}).to_string();
    assert!(desktop::set_trust_settings(&s, &body).is_err());
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&desktop::trust_settings(&s).unwrap()).unwrap(),
        before
    );
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Locked);
}

#[test]
fn shutdown_diagnostics_remain_available_while_store_and_control_are_held() {
    let (_root, service) = fixture();
    service
        .shutdown_diagnostics
        .phase(drive_runtime::shutdown::Phase::CheckpointingLocal);
    let store = service.bridge.store.lock().unwrap();
    let control = service.control.lock().unwrap();
    let (send, receive) = std::sync::mpsc::channel();
    let reader = service.clone();
    let thread = std::thread::spawn(move || {
        let _ = send.send(desktop::shutdown_status(&reader));
    });
    let result = receive.recv_timeout(std::time::Duration::from_secs(1));
    drop(control);
    drop(store);
    // Release locks before asserting, so a future lock regression exits naturally.
    if result.is_ok() {
        thread.join().unwrap();
    }
    let data: serde_json::Value =
        serde_json::from_str(&result.expect("shutdown status waited on Store/control")).unwrap();
    assert_eq!(data["task_id"], service.profile.task_id);
    assert_eq!(data["shutdown"]["phase"], "checkpointing_local");
    assert_eq!(data["mounted"], false);
}

#[test]
fn overview_background_failure_fields_are_additive_and_null_when_healthy() {
    let (_dir, service) = fixture();
    let overview: serde_json::Value =
        serde_json::from_str(&desktop::overview(&service).unwrap()).unwrap();
    assert_eq!(overview["schema"].as_u64().unwrap(), 1);
    assert!(overview["background_error"].is_null());
    assert!(overview["background_error_kind"].is_null());
    assert!(overview["background_error_since_ms"].is_null());
    assert!(overview["upload_block"].is_null());
    assert!(overview["pending_operations"].is_number());
}
