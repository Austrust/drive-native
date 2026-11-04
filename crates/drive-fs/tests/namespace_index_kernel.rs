//! NSI-specific kernel regressions. These deliberately enable the index; the
//! original SQL-path directory tests remain separate and unchanged.
use drive_core::{Node, NodeMetadata, Store};
use std::{
    fs::{File, OpenOptions},
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    sync::{Arc, mpsc},
    time::Duration,
};

fn page(file: &File) -> Vec<(u64, i64, String)> {
    let mut buffer = [0u8; 512];
    let count = unsafe {
        libc::syscall(
            libc::SYS_getdents64,
            file.as_raw_fd(),
            buffer.as_mut_ptr(),
            buffer.len(),
        )
    };
    assert!(count >= 0, "getdents: {}", std::io::Error::last_os_error());
    let mut rows = Vec::new();
    let mut position = 0;
    while position < count as usize {
        let entry = &buffer[position..count as usize];
        let length = u16::from_ne_bytes(entry[16..18].try_into().unwrap()) as usize;
        assert!(length >= 20 && length <= entry.len());
        let end = entry[19..length].iter().position(|b| *b == 0).unwrap() + 19;
        rows.push((
            u64::from_ne_bytes(entry[..8].try_into().unwrap()),
            i64::from_ne_bytes(entry[8..16].try_into().unwrap()),
            String::from_utf8(entry[19..end].to_vec()).unwrap(),
        ));
        position += length;
    }
    rows
}
fn remaining(file: &File) -> Vec<(u64, i64, String)> {
    let mut entries = Vec::new();
    loop {
        let page = page(file);
        if page.is_empty() {
            break;
        }
        entries.extend(page);
    }
    entries
}
fn seek(file: &File, cookie: i64) {
    assert_eq!(
        unsafe { libc::lseek(file.as_raw_fd(), cookie, libc::SEEK_SET) },
        cookie
    );
}
fn open(path: &std::path::Path) -> File {
    OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_DIRECTORY)
        .open(path)
        .unwrap()
}

fn continuation(plus: bool) {
    assert!(drive_fs::namespace_index_enabled(), "this regression requires NSI enabled");
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let gid = unsafe { libc::getgid() };
    let temp = tempfile::tempdir().unwrap();
    let mount = temp.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(temp.path().join("state")).unwrap();
    let make = |id: &str| Node {
        id: id.into(),
        name: id.into(),
        parent: (id != "root").then(|| "root".into()),
        directory: id == "root",
        version: "1".into(),
        metadata: Some(NodeMetadata {
            size: 3,
            mtime: 1,
            mode: 0o755,
        }),
    };
    let mut nodes = vec![make("root")];
    nodes.extend((0..2000).map(|n| make(&format!("item-{n:04}"))));
    s.install_remote_snapshot(&nodes, "1", "s").unwrap();
    let store = Arc::new(drive_core::SharedStore::new(s));
    let mut fs = drive_fs::CacheFilesystem::new(store.clone(), "root", uid, gid).unwrap();
    let index = { let guard = store.lock().unwrap();
        drive_core::namespace_index::NamespaceIndex::build(&guard).unwrap() };
    assert!(!index.is_degraded());
    fs = fs.with_namespace_index(Arc::new(index));
    let hub = drive_fs::NotifyHub::with_ttl_ms(1000);
    if plus { fs = fs.with_notify_hub(hub.clone()); }
    let session = fuser::Session::new(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    if plus { hub.install(session.notifier()); }
    let session = fuser::BackgroundSession::new(session).unwrap();
    let original = open(&mount);
    let first = page(&original);
    assert!(!first.is_empty() && first.len() < 2000);
    // New opens should observe the new namespace; the old stream must retain
    // its original order/inodes/cookies through rename, removal and creation.
    nodes[1].name = "renamed".into();
    nodes.remove(2);
    nodes.push(make("added"));
    store
        .lock()
        .unwrap()
        .install_remote_snapshot(&nodes, "2", "s")
        .unwrap();
    let fresh = open(&mount);
    let guard = store.lock().unwrap();
    let (tx, rx) = mpsc::channel();
    let reader = std::thread::spawn(move || {
        let mut old = first;
        old.extend(remaining(&original));
        assert!(page(&original).is_empty());
        let cookie = old[7].1;
        seek(&original, cookie);
        let suffix = remaining(&original);
        assert_eq!(suffix, old[8..]);
        seek(&original, 0);
        assert_eq!(remaining(&original), old);
        let new = remaining(&fresh);
        tx.send((old, new)).unwrap();
    });
    let result = rx.recv_timeout(Duration::from_secs(3));
    drop(guard);
    reader.join().unwrap();
    session.join();
    let (old, new) = result.expect("directory continuation waited for Store");
    assert_eq!(old.len(), 2002);
    assert_eq!(new.len(), 2002);
    let names =
        |items: &Vec<(u64, i64, String)>| items.iter().map(|e| e.2.clone()).collect::<Vec<_>>();
    let old_names = names(&old);
    let new_names = names(&new);
    assert!(old_names.contains(&"item-0000".into()) && old_names.contains(&"item-0001".into()));
    assert!(!new_names.contains(&"item-0000".into()) && !new_names.contains(&"item-0001".into()));
    assert!(new_names.contains(&"renamed".into()) && new_names.contains(&"added".into()));
    assert_eq!(
        old.iter().find(|e| e.2 == "item-0000").unwrap().0,
        new.iter().find(|e| e.2 == "renamed").unwrap().0
    );
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; NSI enabled"]
fn indexed_plain_continuation_and_seek_do_not_wait_for_store() { continuation(false); }

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; NSI enabled"]
fn indexed_plus_continuation_and_seek_do_not_wait_for_store() { continuation(true); }
