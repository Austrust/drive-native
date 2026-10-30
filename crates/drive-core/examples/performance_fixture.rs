//! Deterministic local-only P0 fixture. No credentials, network, or existing state.
use drive_core::{DownloadTicket, Node, NodeMetadata, Store, sha256};
use std::{path::PathBuf, time::Instant};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 2 {
        return Err("usage: performance_fixture <new-directory>".into());
    }
    let root = PathBuf::from(&args[1]);
    std::fs::create_dir(&root)?; // Refuse overwriting any prior experiment.
    let start = Instant::now();
    let make = |id: String, parent: Option<String>, name: String, directory, size| Node {
        id, parent, name, version: "fixture-v1".into(), directory,
        metadata: Some(NodeMetadata {size, mtime: 1_700_000_000, mode: if directory {0o755} else {0o644}}),
    };
    let mut nodes = vec![make("root".into(), None, "fixture".into(), true, 0)];
    let mut file_index = 0;
    // Exactly 210,000 total nodes: root + 105 directories + 209,894 files.
    for dir in 0..105 {
        let parent = format!("d{dir:03}");
        nodes.push(make(parent.clone(), Some("root".into()), parent.clone(), true, 0));
        for child in 0..if dir == 104 {1894} else {2000} {
            nodes.push(make(format!("file-{file_index:06}"), Some(parent.clone()),
                format!("f{child:05}"), false, if file_index == 0 {1_048_576} else {0}));
            file_index += 1;
        }
    }
    assert_eq!(nodes.len(), 210_000);
    let dataset_hash = sha256(&serde_json::to_vec(&nodes)?);
    let mut store = Store::open(root.join("state"))?;
    store.install_remote_snapshot(&nodes, "fixture-cursor-1", "fixture-server-1")?;
    store.ensure_fs_root("root")?;
    let body: Vec<u8> = (0..1_048_576).map(|i| (i % 251) as u8).collect();
    let content_hash = sha256(&body);
    store.publish_download(&DownloadTicket {id: "file-000000".into(), version: "fixture-v1".into()}, &body, &content_hash)?;
    drop(store);
    let manifest = serde_json::json!({"schema":1,"nodes":210000,"directories_including_root":106,
        "files":209894,"directory":"d000","directory_children":2000,"cached_file":"d000/f00000",
        "cached_node":"file-000000","cached_bytes":1048576,"content_sha256":content_hash,
        "dataset_sha256":dataset_hash,"purpose":"local fixed workload; no native protocol or remote identity validation",
        "generation_seconds":start.elapsed().as_secs_f64()});
    std::fs::write(root.join("fixture.json"), serde_json::to_vec_pretty(&manifest)?)?;
    println!("{}", manifest);
    Ok(())
}
