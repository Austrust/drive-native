//! Synthetic directory-only construction cost. No mount, TTL, content or NAS.
//! Compare the SAME example source grafted onto baseline and candidate builds.
use drive_core::{Node, NodeMetadata, Store};
use std::time::Instant;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let a: Vec<_> = std::env::args().collect();
    let directories: usize = a.get(1).map_or(Ok(2000), |s| s.parse())?;
    let files: usize = a.get(2).map_or(Ok(4), |s| s.parse())?;
    let rounds: usize = a.get(3).map_or(Ok(3), |s| s.parse())?;
    if directories == 0
        || directories > 60000
        || files > 64
        || rounds == 0
        || rounds > 10
        || directories
            .checked_mul(files + 1)
            .is_none_or(|n| n > 500000)
    {
        return Err("synthetic fixture budget".into());
    }
    let temp = tempfile::tempdir()?;
    let mut store = Store::open(temp.path())?;
    let make = |id: String, parent: Option<String>, directory: bool| Node {
        id: id.clone(),
        parent,
        name: id,
        version: "1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 4 },
            mtime: 17,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    };
    let mut nodes = vec![make("root".into(), None, true)];
    for d in 0..directories {
        let id = format!("d{d:06}");
        nodes.push(make(id.clone(), Some("root".into()), true));
        for f in 0..files {
            nodes.push(make(format!("f{d:06}_{f:02}"), Some(id.clone()), false));
        }
    }
    store.install_remote_snapshot(&nodes, "1", "synthetic")?;
    store.ensure_fs_root("root")?;
    let root = store.directory_snapshot(1, 64 * 1024 * 1024)?;
    let inodes: Vec<_> = root.entries.into_iter().skip(2).map(|e| e.ino).collect();
    assert_eq!(inodes.len(), directories);
    for round in 0..rounds {
        let revision = store.local_revision();
        let started = Instant::now();
        let mut stage = [0u64; 5];
        let mut writes = 0;
        let mut entries = 0;
        for &ino in &inodes {
            let (view, t) = store.directory_snapshot_measured(ino, 1024 * 1024)?;
            assert_eq!(view.entries.len(), files + 2);
            assert_eq!(view.entries[0].ino, ino);
            assert_eq!(view.entries[1].ino, 1);
            entries += view.entries.len();
            writes += t.inode_writes;
            for (sum, n) in stage.iter_mut().zip([
                t.identity_us,
                t.count_us,
                t.inode_allocation_us,
                t.collect_us,
                t.commit_us,
            ]) {
                *sum += n;
            }
        }
        let elapsed = started.elapsed();
        assert_eq!(writes, 0);
        assert_eq!(store.local_revision(), revision);
        println!(
            "{}",
            serde_json::json!({"round":round,"directories":directories,"files_per_directory":files,"visited_entries_with_dots":entries,"seconds":elapsed.as_secs_f64(),"per_directory_us":elapsed.as_micros() as f64/directories as f64,"stage_us":{"identity":stage[0],"count":stage[1],"inodes":stage[2],"collect":stage[3],"commit":stage[4]},"inode_writes":writes,"authoritative_revision_unchanged":true})
        );
    }
    Ok(())
}
