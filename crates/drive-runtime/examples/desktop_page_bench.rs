//! Compare desktop paging with the pre-pagination algorithm on identical local data.
//! No NAS or FUSE. This does not measure whole-directory enumeration or P1 status cost.
use drive_core::{Node, NodeMetadata, Store};
use drive_runtime::{desktop, service::{Profile, Service}};
use serde_json::{Value, json};
use std::{io, path::PathBuf, time::Instant};

fn legacy_root_page(service: &Service, offset: u32) -> io::Result<String> {
    let (paths, total) = {
        let store = service.bridge.store.lock().unwrap();
        store.ensure_fs_root("root").map_err(io::Error::other)?;
        let entries = store.children("root").map_err(io::Error::other)?;
        let total = entries.len();
        let paths = entries.into_iter().skip(offset as usize).take(256)
            .map(|r| r.node.name).collect();
        (paths, total)
    };
    let mut result: Value = serde_json::from_str(&desktop::states(service, paths)?)?;
    result["total"] = total.into();
    result["offset"] = offset.into();
    result["path"] = "".into();
    Ok(result.to_string())
}

fn summary(samples: &[f64]) -> Value {
    let mut sorted = samples.to_vec();
    sorted.sort_by(f64::total_cmp);
    let percentile = |p: f64| sorted[((sorted.len() as f64 * p).ceil() as usize).saturating_sub(1)];
    json!({"samples_ms":samples,"p50_ms":percentile(0.5),"p95_ms":percentile(0.95),
           "p99_ms":percentile(0.99),"max_ms":sorted.last(),"errors":0})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = tempfile::tempdir()?;
    let mut store = Store::open(directory.path())?;
    let mut nodes = vec![Node { id:"root".into(), parent:None, name:"root".into(),
        version:"1".into(), directory:true, metadata:Some(NodeMetadata {size:0,mtime:1,mode:0o755}) }];
    for i in 0..2000 {
        nodes.push(Node {id:format!("f{i:04}"),parent:Some("root".into()),name:format!("f{i:04}"),
            version:"1".into(),directory:false,metadata:Some(NodeMetadata {size:8192,mtime:1,mode:0o644})});
    }
    store.install_remote_snapshot(&nodes,"1","server")?;
    store.ensure_fs_root("root")?;
    let service = Service::new(Profile {schema:1,task_id:uuid::Uuid::new_v4().to_string(),
        host:"example.invalid".into(),tls_name:"example.invalid".into(),trusted_ca_pem:vec![],
        username:"synthetic".into(),device_id:uuid::Uuid::new_v4().to_string(),
        scope:drive_core::native::Scope {view_id:1,node_id:1,file_id:"root".into(),path:"/".into()},
        mount:PathBuf::from("/tmp/desktop-page-bench"),writable:false},store,PathBuf::from("/nonexistent-worker"))?;
    let mut legacy = vec![];
    let mut paged = vec![];
    for round in 0..35 {
        // Three warmup pairs, then 32 alternating pairs; include first and last pages.
        let offset = if round % 2 == 0 {0} else {1792};
        let mut results = vec![];
        for old in if round % 2 == 0 {[true,false]} else {[false,true]} {
            let start = Instant::now();
            let result = if old {legacy_root_page(&service,offset)} else {desktop::directory(&service,"",offset)};
            let elapsed = start.elapsed().as_secs_f64()*1000.;
            // Failures terminate the benchmark; do not silently drop failed samples.
            let value: Value = serde_json::from_str(&result?)?;
            results.push(value);
            if round >= 3 { if old {legacy.push(elapsed)} else {paged.push(elapsed)} }
        }
        assert_eq!(results[0],results[1],"page semantics changed at offset {offset}");
    }
    println!("{}",json!({"scope":"synthetic desktop page, warm metadata, no NAS/FUSE",
        "children":2000,"page_size":256,"warmup_pairs":3,"pairs":32,
        "order":"alternating legacy/paged; first/last pages","equivalent_responses":true,
        "legacy":summary(&legacy),"paged":summary(&paged)}));
    Ok(())
}
