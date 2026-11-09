//! No kernel/NAS: exercise the exact candidate publication boundary used by PLUS.
use super::*;
use drive_core::initialization_store::{InitializationBinding, InitializationBudget, InitializationRun};
use drive_core::initialization_events::InitializationDirectoryEntry;
use std::sync::mpsc;

fn fixture() -> (tempfile::TempDir, Arc<drive_core::SharedStore>, InitializationRun, drive_core::preview::PreviewStamp) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    let root = Node { id: "100".into(), parent: None, name: "Lab".into(), directory: true,
        version: serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":"100","sync":100,"max":100,"size":3,"hash":"synthetic","directory":true}).to_string(), metadata: Some(drive_core::NodeMetadata {size:3,mtime:1,mode:0o755}) };
    let scope = serde_json::json!({"view_id":1,"node_id":2,"file_id":"100","path":"/Lab"});
    store.bind_native_scope("server", &scope.to_string()).unwrap();
    let binding = InitializationBinding { task_id:"publication-fixture".into(), scope,
        restore:"server".into(), start_cursor:"native-v1:100".into(), expected_cursor:"".into(), rules_digest: String::new() };
    let run = store.begin_initialization(&binding, &root, InitializationBudget::default()).unwrap();
    let t = store.begin_initialization_directory(&run,"100").unwrap();
    let child = Node { id:"200".into(), parent:Some("100".into()),name:"Child".into(), version:serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":"200","sync":100,"max":100,"size":3,"hash":"synthetic","directory":true}).to_string(), ..root.clone() };
    store.store_initialization_native_directory(&t,1,&[InitializationDirectoryEntry{node:child,native_node_id:50}]).unwrap();
    store.preview_mount_root("100").unwrap().unwrap();
    let stamp = store.preview_directory_snapshot(1,1024*1024).unwrap().stamp;
    (temp,Arc::new(drive_core::SharedStore::new(store)),run,stamp)
}

#[test]
fn candidate_reply_holds_validation_guard_through_send() {
    let (_t,store,_run,stamp)=fixture();
    let mut sent=false;
    publish_view(&store,&stamp,|valid| {
        valid.unwrap();
        assert!(matches!(store.try_lock(),Err(std::sync::TryLockError::WouldBlock)));
        sent=true;
    });
    assert!(sent);
    assert!(store.try_lock().is_ok());
}

#[test]
fn busy_store_never_bypasses_preview_revoke() {
    let (_t,store,run,stamp)=fixture();
    let mut guard=store.lock().unwrap();
    let t=guard.begin_initialization_directory(&run,"200").unwrap();
    guard.store_initialization_denied(&t,1).unwrap();
    let (started_tx,started_rx)=mpsc::channel();
    let (reply_tx,reply_rx)=mpsc::channel();
    let worker_store=store.clone();
    let worker=std::thread::spawn(move||{
        started_tx.send(()).unwrap();
        publish_view(&worker_store,&stamp,|valid|{reply_tx.send(valid.map_err(|e|errno(&e))).unwrap();});
    });
    started_rx.recv().unwrap();
    let early=reply_rx.recv_timeout(Duration::from_millis(100));
    drop(guard);
    worker.join().unwrap();
    assert!(matches!(early,Err(mpsc::RecvTimeoutError::Timeout)),"busy preview must not send old names");
    let code=reply_rx.recv().unwrap().unwrap_err();
    assert!(matches!(code,libc::EACCES|libc::ESTALE));
}

#[test]
fn a_promoted_candidate_cannot_publish_its_old_directory_view() {
    let (_t,store,run,stamp)=fixture();
    {
        let mut s=store.lock().unwrap();
        let ticket=s.begin_initialization_directory(&run,"200").unwrap();
        s.store_initialization_native_directory(&ticket,1,&[]).unwrap();
        let fence=s.begin_initialization_fence(&run).unwrap();
        s.confirm_initialization_fence(&fence,"native-v1:100").unwrap();
        s.complete_initialization(&run).unwrap();
    }
    publish_view(&store,&stamp,|valid|assert_eq!(errno(&valid.unwrap_err()),libc::ESTALE));
}
