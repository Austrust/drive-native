//! Synthetic queue replay. No network, external state or private names.
//! Each mode gets a new Store with the same completed context and 64 pending
//! directories. Only claim calls are timed; setup and receipt commits are not.
use drive_core::initialization_store::{InitializationBinding, InitializationBudget};
use drive_core::{Node, NodeMetadata, Store};
use serde_json::json;
use std::time::{Duration, Instant};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        directory,
        version: "synthetic-v1".into(),
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}

fn run(mode: &str, context: usize) -> Result<serde_json::Value, Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let mut store = Store::open(temp.path())?;
    let scope = json!({"view_id":1,"node_id":2,"file_id":"root","path":"/root"});
    store.bind_native_scope("synthetic-server", &scope.to_string())?;
    let binding = InitializationBinding {
        task_id: "queue-replay".into(),
        scope,
        restore: "synthetic-server".into(),
        start_cursor: "native-v1:100".into(),
        expected_cursor: String::new(),
    };
    let run = store.begin_initialization(
        &binding,
        &node("root", None, true),
        InitializationBudget::default(),
    )?;
    let mut children = vec![node("context", Some("root"), true)];
    children.extend((0..64).map(|i| node(&format!("queue-{i:04}"), Some("root"), true)));
    let ticket = store.begin_initialization_directory(&run, "root")?;
    store.store_initialization_directory(&ticket, 1, &children)?;
    let ticket = store.begin_initialization_directory(&run, "context")?;
    let contents = (0..context)
        .map(|i| node(&format!("context-file-{i:06}"), Some("context"), false))
        .collect::<Vec<_>>();
    store.store_initialization_directory(&ticket, 1, &contents)?;
    let mut claim_time = Duration::ZERO;
    let mut calls = 0;
    let mut claimed = 0;
    let mut examined = 0;
    loop {
        let began = Instant::now();
        let jobs = if mode == "legacy" {
            let mut jobs = Vec::new();
            for node in store.initialization_pending_directories(&run, 4)? {
                let path = store.initialization_directory_path(&run, &node.id)?;
                let ticket = store.begin_initialization_directory(&run, &node.id)?;
                jobs.push((node, path, ticket));
            }
            jobs
        } else {
            let page = store.claim_initialization_directories(&run, 4, None)?;
            examined += page.stats.examined;
            page.jobs
                .into_iter()
                .map(|j| (j.node, j.path, j.ticket))
                .collect()
        };
        claim_time += began.elapsed();
        calls += 1;
        if jobs.is_empty() {
            break;
        }
        for (node, path, ticket) in jobs {
            assert_eq!(path, format!("/root/{}", node.id));
            assert!(store.initialization_directory_ticket_current(&ticket)?);
            store.store_initialization_directory(&ticket, 1, &[])?;
            claimed += 1;
        }
    }
    assert_eq!(claimed, 64);
    assert_eq!(
        store.initialization_status()?.unwrap().node_count,
        context as u64 + 66
    );
    assert!(store.cursor()?.is_empty());
    assert!(store.initialization_writeback_blocked()?);
    Ok(
        json!({"mode":mode,"context_nodes":context,"claimed":claimed,"claim_calls":calls,
        "examined":if mode=="legacy" {None} else {Some(examined)},
        "claim_seconds":claim_time.as_secs_f64(),"authoritative_cursor_unchanged":true}),
    )
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mode = std::env::args().nth(1).unwrap_or_else(|| "batched".into());
    if !matches!(mode.as_str(), "legacy" | "batched") {
        return Err("expected legacy or batched".into());
    }
    let context = std::env::args()
        .nth(2)
        .map(|n| n.parse())
        .transpose()?
        .unwrap_or(8192);
    if context > 50_000 {
        return Err("context budget exceeded".into());
    }
    println!("{}", run(&mode, context)?);
    Ok(())
}
