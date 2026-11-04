//! Independent validation harness: frozen production Desktop, never start a worker.
use drive_core::{Node, NodeMetadata, Store};
use drive_runtime::{desktop, service::{Profile, Service}};
use std::{path::PathBuf, time::Duration};

struct DelayedHistory;
#[zbus::interface(name = "org.drivenative.Client1")]
impl DelayedHistory {
    async fn get_sync_history(&self, _body: String) -> String {
        tokio::time::sleep(Duration::from_secs(60)).await;
        "{\"records\":[],\"next_cursor\":null}".into()
    }
}

#[tokio::main(flavor="multi_thread", worker_threads=2)]
async fn main() {
    let args: Vec<String> = std::env::args().collect();
    let state = PathBuf::from(&args[2]);
    if args[1] == "init" {
        let mut store = Store::open(&state).unwrap();
        store.install_remote_snapshot(&[Node { id:"root".into(), parent:None,
            name:"root".into(), version:"1".into(), directory:true,
            metadata:Some(NodeMetadata {size:0,mtime:1,mode:0o755}) }], "1", "synthetic").unwrap();
        store.ensure_fs_root("root").unwrap();
        return;
    }
    assert_eq!(args[1], "serve");
    let profile = Profile { schema:1, task_id:"00000000-0000-0000-0000-000000000002".into(),
        host:"example.invalid".into(), tls_name:"example.invalid".into(), trusted_ca_pem:vec![],
        username:"synthetic".into(), device_id:"00000000-0000-0000-0000-000000000003".into(),
        scope:drive_core::native::Scope {view_id:1,node_id:1,file_id:"root".into(),path:"/synthetic".into()},
        mount:state.join("never-mounted"), writable:false, local_policy:None,writable_source:None };
    let service = Service::new(profile, Store::open(&state).unwrap(), state.join("nonexistent-worker")).unwrap();
    let _actual = zbus::connection::Builder::session().unwrap()
        .name("org.drivenative.Client1.i00000000000000000000000000000002").unwrap()
        .serve_at(desktop::OBJECT_PATH, desktop::Desktop(service)).unwrap().build().await.unwrap();
    let _delayed = zbus::connection::Builder::session().unwrap()
        .name("org.drivenative.Client1.i00000000000000000000000000000001").unwrap()
        .serve_at(desktop::OBJECT_PATH, DelayedHistory).unwrap().build().await.unwrap();
    println!("READY actual Desktop + synthetic delayed peer; no worker, no mount");
    tokio::signal::ctrl_c().await.unwrap();
}
