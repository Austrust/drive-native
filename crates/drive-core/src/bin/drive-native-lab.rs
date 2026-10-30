//! An offline acceptance-scenario runner, not a mountable client.
use drive_core::*;
use std::io::Read;

fn main() -> Result<()> {
    if std::env::args().nth(1).as_deref() != Some("--simulate") {
        eprintln!(
            "drive-native-lab --simulate\nOffline state-engine demonstration; no NAS connection or mount. Native event/write release gates remain closed."
        );
        std::process::exit(2);
    }
    let root = tempfile::tempdir()?;
    let mut store = Store::open(root.path())?;
    let node = |v: &str| Node {
        id: "doc-identity".into(),
        parent: None,
        name: "A.doc".into(),
        version: v.into(),
        directory: false,
        metadata: None,
    };
    let first = Batch {
        from: "".into(),
        to: "cursor-1".into(),
        events: vec![Event {
            id: "event-1".into(),
            change: Change::Upsert(node("v1")),
        }],
    };
    store.receive(&first)?;
    store.drain()?;
    store.publish_download(
        &DownloadTicket {
            id: "doc-identity".into(),
            version: "v1".into(),
        },
        b"original document",
        &sha256(b"original document"),
    )?;
    // Simulated server event; no file-open occurs between the event and background publication.
    let second = Batch {
        from: "cursor-1".into(),
        to: "cursor-2".into(),
        events: vec![Event {
            id: "event-2".into(),
            change: Change::Upsert(node("v2")),
        }],
    };
    store.receive(&second)?;
    store.drain()?;
    let jobs = store.downloads_needed()?;
    if jobs.len() != 1 {
        return Err(Error::State("expected one background update".into()));
    }
    store.publish_download(
        &jobs[0],
        b"updated on another device",
        &sha256(b"updated on another device"),
    )?;
    drop(store);
    // Restart with only local SQLite and blobs; no server object is available here.
    let store = Store::open(root.path())?;
    let mut bytes = Vec::new();
    store.open_cached("doc-identity")?.read_to_end(&mut bytes)?;
    if bytes != b"updated on another device" {
        return Err(Error::State("offline content is stale".into()));
    }
    println!(
        "{}",
        serde_json::json!({
            "kind":"offline simulation", "scenario":"unpinned cached A.doc refreshed without reopening",
            "passed":true,"pinned":store.is_pinned("doc-identity")?,"cursor":store.cursor()?,
            "status":format!("{:?}",store.status("doc-identity")?),"offline_sha256":sha256(&bytes),
            "nas_contacted":false,"fuse_mounted":false,"writable_release_ready":false
        })
    );
    Ok(())
}
