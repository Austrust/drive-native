//! Kernel regression for the queue dependency, not a throughput benchmark.
use super::*;
use drive_core::{DownloadTicket, NodeMetadata, sha256};
use std::{io::Write, os::unix::fs::FileExt, sync::mpsc, time::Instant};

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn held_content_reads_survive_all_metadata_workers_waiting_for_store() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let temp = tempfile::tempdir().unwrap();
    let mount = temp.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut store = Store::open(temp.path().join("state")).unwrap();
    let nodes: Vec<_> = ["root", "cached", "draft"]
        .into_iter()
        .map(|id| Node {
            id: id.into(),
            parent: (id != "root").then(|| "root".into()),
            name: id.into(),
            directory: id == "root",
            version: "1".into(),
            metadata: Some(NodeMetadata {
                size: if id == "root" { 0 } else { 1024 * 1024 },
                mtime: 1,
                mode: 0o755,
            }),
        })
        .collect();
    store
        .install_remote_snapshot(&nodes, "1", "synthetic")
        .unwrap();
    let data = vec![17; 1024 * 1024];
    for id in ["cached", "draft"] {
        store
            .publish_download(
                &DownloadTicket {
                    id: id.into(),
                    version: "1".into(),
                },
                &data,
                &sha256(&data),
            )
            .unwrap();
    }
    let store = Arc::new(drive_core::SharedStore::new(store));
    let fs = CacheFilesystem::new(store.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let metadata_jobs = fs.queue.sender.as_ref().unwrap().clone();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::FSName("drive-native-isolation-test".into()),
        ],
    )
    .unwrap();
    let cached = std::fs::File::open(mount.join("cached")).unwrap();
    let mut draft = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(mount.join("draft"))
        .unwrap();
    draft.write_all(b"new").unwrap();
    draft.sync_all().unwrap();
    // Simulate promotion completing before its first write while a new kernel
    // handle opens. That handle must join the working copy, not the old blob.
    let clean = {
        let mut s = store.lock().unwrap();
        let source = s.open_cached("cached").unwrap();
        s.begin_draft("cached", "1", Some(&source)).unwrap()
    };
    let joined = std::fs::File::open(mount.join("cached")).unwrap();
    store
        .lock()
        .unwrap()
        .draft_write(&clean, 0, b"join", false)
        .unwrap();
    let mut joined_bytes = [0; 4];
    assert_eq!(joined.read_at(&mut joined_bytes, 0).unwrap(), 4);
    assert_eq!(&joined_bytes, b"join");
    let guard = store.lock().unwrap();
    let (started, ready) = mpsc::channel();
    for _ in 0..4 {
        let store = store.clone();
        let started = started.clone();
        metadata_jobs
            .send(Box::new(move || {
                started.send(()).unwrap();
                drop(store.lock().unwrap());
            }))
            .unwrap();
    }
    let all_started = (0..4).all(|_| ready.recv_timeout(Duration::from_secs(3)).is_ok());
    if !all_started {
        drop(guard);
        drop((cached, draft, joined, clean, metadata_jobs));
        session.join();
        panic!("metadata workers did not reach the Store barrier");
    }
    let (result, completed) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let begin = Instant::now();
        let mut bytes = vec![0; 1024 * 1024];
        let cached_ok = cached
            .read_at(&mut bytes, 0)
            .is_ok_and(|n| n == bytes.len())
            && bytes == data;
        let draft_ok = draft.read_at(&mut bytes, 0).is_ok_and(|n| n == bytes.len())
            && &bytes[..3] == b"new"
            && bytes[3..].iter().all(|b| *b == 17);
        result.send((cached_ok, draft_ok, begin.elapsed())).unwrap();
        // FUSE release/flush may wait for the metadata workers; the test unlocks
        // Store before joining this thread, including on regression failure.
    });
    let before_unlock = completed.recv_timeout(Duration::from_secs(2));
    drop(guard);
    reader.join().unwrap();
    drop(joined);
    drop(clean);
    drop(metadata_jobs);
    session.join();
    let (cached, draft, elapsed) =
        before_unlock.expect("held content was blocked by metadata/Store");
    assert!(cached && draft, "cached or mutable content changed");
    eprintln!("held cached + draft reads completed under blocked metadata: {elapsed:?}");
}
