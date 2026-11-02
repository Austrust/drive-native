//! Public synthetic replay; no network, external database, or private paths.
//! Copy this unchanged example to the baseline checkout for sequential comparison.
//! Timings are observations, never a pass threshold or a convergence claim.
use drive_core::initialization_events::InitializationDirectoryEntry;
use drive_core::initialization_store::{
    InitializationBinding, InitializationBudget, InitializationRun,
};
use drive_core::{Node, NodeMetadata, Store};
use serde_json::json;
use std::time::Instant;

fn node(id: u64, parent: Option<u64>, directory: bool, revision: u64) -> Node {
    Node {
        id: id.to_string(), parent: parent.map(|v| v.to_string()),
        name: if parent.is_none() { "Synthetic".into() } else { format!("item-{id}") },
        directory,
        version: json!({"schema":1,"restore":"synthetic-server","view":1,"root":"100","file":id.to_string(),"sync":revision,"max":revision,"size":0,"hash":"synthetic","directory":directory}).to_string(),
        metadata: Some(NodeMetadata { size:0, mtime:revision, mode:if directory {0o755} else {0o644} }),
    }
}
fn entries(id: &str, revision: u64) -> Vec<InitializationDirectoryEntry> {
    let children = match id {
        "100" => vec![node(200, Some(100), true, revision)],
        "200" => (1000..1100).map(|v| node(v, Some(200), true, 1)).collect(),
        other => {
            let directory: u64 = other.parse().unwrap();
            assert!((1000..1100).contains(&directory));
            (0..99)
                .map(|i| {
                    node(
                        10_000 + (directory - 1000) * 99 + i,
                        Some(directory),
                        false,
                        1,
                    )
                })
                .collect()
        }
    };
    children
        .into_iter()
        .map(|node| {
            // Explicit synthetic mapping, not a production file-ID conversion.
            let native_node_id = 1_000_000 + node.id.parse::<u64>().unwrap();
            InitializationDirectoryEntry {
                node,
                native_node_id,
            }
        })
        .collect()
}
fn list(
    store: &mut Store,
    run: &InitializationRun,
    id: &str,
    revision: u64,
) -> drive_core::Result<()> {
    let ticket = store.begin_initialization_directory(run, id)?;
    store.store_initialization_native_directory(&ticket, 1, &entries(id, revision))?;
    Ok(())
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let temp = tempfile::tempdir()?;
    let mut store = Store::open(temp.path())?;
    let scope = json!({"view_id":1,"node_id":2,"file_id":"100","path":"/Synthetic"});
    store.bind_native_scope("synthetic-server", &scope.to_string())?;
    let binding = InitializationBinding {
        task_id: "synthetic-refresh".into(),
        scope,
        restore: "synthetic-server".into(),
        start_cursor: "native-v1:100".into(),
        expected_cursor: String::new(),
    };
    let run = store.begin_initialization(
        &binding,
        &node(100, None, true, 1),
        InitializationBudget::default(),
    )?;
    list(&mut store, &run, "100", 1)?;
    list(&mut store, &run, "200", 1)?;
    for id in 1000..1100 {
        list(&mut store, &run, &id.to_string(), 1)?;
    }
    let before = store
        .initialization_status()?
        .ok_or("missing run")?
        .node_count;
    assert_eq!(before, 10_002);
    let ticket = store.begin_initialization_directory(&run, "100")?;
    let fresh = entries("100", 2);
    let began = Instant::now();
    store.store_initialization_native_directory(&ticket, 1, &fresh)?;
    let transaction_seconds = began.elapsed().as_secs_f64();
    let after = store
        .initialization_status()?
        .ok_or("missing run")?
        .node_count;
    let known_pending = store
        .initialization_coverage(&run)?
        .iter()
        .filter(|c| c.state == "pending")
        .count();
    // Both implementations must still perform all 101 directory reads. Keeping
    // metadata is not permission to keep stale coverage or skip the fence.
    let began = Instant::now();
    let mut reads = 0;
    loop {
        let pending = store.initialization_pending_directories(&run, 1)?;
        let Some(next) = pending.first() else { break };
        reads += 1;
        assert!(reads <= 101, "unexpected rescan loop");
        list(&mut store, &run, &next.id, 2)?;
    }
    assert_eq!(reads, 101);
    assert_eq!(
        store
            .initialization_status()?
            .ok_or("missing run")?
            .node_count,
        before
    );
    let fence = store.begin_initialization_fence(&run)?;
    store.confirm_initialization_fence(&fence, fence.cursor())?;
    println!(
        "{}",
        json!({"schema":1,"fixture":"same-directory-identity-10000-descendants","nodes_before":before,"nodes_after_parent_refresh":after,"descendants_retained":after-2,"known_pending_after_update":known_pending,"required_list_requests_until_fence":reads,"transaction_seconds":transaction_seconds,"recheck_seconds":began.elapsed().as_secs_f64(),"authoritative_cursor_unchanged":store.cursor()?.is_empty(),"nas_accessed":false})
    );
    Ok(())
}
