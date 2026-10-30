//! Tiny, offline-only fixture for measuring persistence boundaries through FUSE.
use drive_core::{DownloadTicket, Node, NodeMetadata, Store, sha256};
use std::path::PathBuf;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: durability_fixture <new-directory>".into());
    }
    let root = PathBuf::from(&args[1]);
    std::fs::create_dir(&root)?;
    let body: Vec<u8> = (0..4096).map(|i| (i % 251) as u8).collect();
    let make = |id: &str, parent: Option<&str>, name: &str, directory: bool| Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: name.into(),
        version: "fixture-v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { body.len() as u64 },
            mtime: 1_700_000_000,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    };
    let nodes = [
        make("root", None, "fixture", true),
        make("lab", Some("root"), "lab", true),
        make("baseline-file", Some("lab"), "A.bin", false),
    ];
    let mut store = Store::open(root.join("state"))?;
    store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
    store.ensure_fs_root("root")?;
    store.publish_download(
        &DownloadTicket {
            id: "baseline-file".into(),
            version: "fixture-v1".into(),
        },
        &body,
        &sha256(&body),
    )?;
    drop(store);
    std::fs::write(
        root.join("fixture.json"),
        serde_json::to_vec_pretty(&serde_json::json!({
            "schema": 1, "nodes": 3, "body_size": body.len(),
            "body_sha256": sha256(&body), "file": "lab/A.bin",
            "network": "none; synthetic identities cannot authenticate"
        }))?,
    )?;
    Ok(())
}
