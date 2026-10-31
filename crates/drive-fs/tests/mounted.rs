//! Explicit kernel test: synthetic local contents, no NAS or user directories.
use drive_core::*;
use std::{
    io::{Read, Seek, SeekFrom},
    sync::Arc,
};
fn node(id: &str, size: u64, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: if id == "root" {
            None
        } else {
            Some("root".into())
        },
        name: if id == "a" { "A.doc".into() } else { id.into() },
        version: version.into(),
        directory: id == "root",
        metadata: Some(NodeMetadata {
            size,
            mtime: 1700000000,
            mode: if id == "root" { 0o755 } else { 0o644 },
        }),
    }
}
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_cache_mount_metadata_reads_version_switch_and_offline_error() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0, "run as the unprivileged test user");
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    s.install_remote_snapshot(
        &[
            node("root", 0, "1"),
            node("a", 3, "1"),
            node("missing", 10, "1"),
        ],
        "1",
        "test",
    )
    .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid).unwrap();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoSuid,
            fuser::MountOption::NoDev,
            fuser::MountOption::FSName("drive-native-kernel-test".into()),
        ],
    )
    .unwrap();
    assert_eq!(std::fs::read_dir(&mount).unwrap().count(), 2);
    assert_eq!(std::fs::metadata(mount.join("missing")).unwrap().len(), 10);
    assert!(s.lock().unwrap().downloads_needed().unwrap().is_empty());
    let mut old = std::fs::File::open(mount.join("A.doc")).unwrap();
    let mut text = String::new();
    old.read_to_string(&mut text).unwrap();
    assert_eq!(text, "old");
    {
        let mut s = s.lock().unwrap();
        s.install_remote_snapshot(
            &[
                node("root", 0, "1"),
                node("a", 7, "2"),
                node("missing", 10, "1"),
            ],
            "2",
            "test",
        )
        .unwrap();
        assert_eq!(s.record("a").unwrap().size, 3);
        s.publish_download(
            &DownloadTicket {
                id: "a".into(),
                version: "2".into(),
            },
            b"new doc",
            &sha256(b"new doc"),
        )
        .unwrap();
    }
    session
        .notifier()
        .inval_inode(s.lock().unwrap().record("a").unwrap().ino, 0, 0)
        .unwrap();
    assert_eq!(std::fs::read(mount.join("A.doc")).unwrap(), b"new doc");
    old.seek(SeekFrom::Start(0)).unwrap();
    text.clear();
    old.read_to_string(&mut text).unwrap();
    assert_eq!(text, "old");
    let error = std::fs::read(mount.join("missing")).unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::ENETDOWN));
    assert!(std::fs::write(mount.join("A.doc"), b"must reject").is_err());
    let concurrent = (0..12)
        .map(|_| {
            let path = mount.join("A.doc");
            std::thread::spawn(move || {
                for _ in 0..20 {
                    assert_eq!(std::fs::read(&path).unwrap(), b"new doc")
                }
            })
        })
        .collect::<Vec<_>>();
    for thread in concurrent {
        thread.join().unwrap();
    }
    assert_eq!(
        std::fs::metadata(mount.join("absent"))
            .unwrap_err()
            .raw_os_error(),
        Some(libc::ENOENT)
    );
    drop(old);
    session.join();
    assert_eq!(s.lock().unwrap().pending_operations().unwrap(), 0);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_random_write_shared_handles_truncate_fsync_and_recovery() {
    use std::io::Write;
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let state = d.path().join("state");
    let mut s = Store::open(&state).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1"), node("a", 6, "1")], "1", "server")
        .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"abcdef",
        &sha256(b"abcdef"),
    )
    .unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    let path = mount.join("A.doc");
    let mut read = std::fs::File::open(&path).unwrap();
    let mut write = std::fs::OpenOptions::new()
        .write(true)
        .read(true)
        .open(&path)
        .unwrap();
    write.seek(SeekFrom::Start(2)).unwrap();
    write.write_all(b"XY").unwrap();
    let mut body = Vec::new();
    read.read_to_end(&mut body).unwrap();
    assert_eq!(body, b"abXYef");
    write.set_len(5).unwrap();
    write.sync_all().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"abXYe");
    assert_eq!(
        s.lock().unwrap().status("a").unwrap(),
        Status::PendingUpload
    );
    drop(read);
    drop(write);
    let jobs = (0..8)
        .map(|_| {
            let path = path.clone();
            std::thread::spawn(move || {
                let mut f = std::fs::OpenOptions::new().append(true).open(path).unwrap();
                f.write_all(b"!").unwrap();
                f.sync_all().unwrap();
            })
        })
        .collect::<Vec<_>>();
    for j in jobs {
        j.join().unwrap()
    }
    assert_eq!(std::fs::read(&path).unwrap(), b"abXYe!!!!!!!!");
    std::fs::OpenOptions::new()
        .write(true)
        .truncate(true)
        .open(&path)
        .unwrap()
        .write_all(b"replacement")
        .unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement");
    session.join();
    drop(s);
    let s = Store::open(&state).unwrap();
    let mut bytes = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"replacement");
    assert!(s.pending_operations().unwrap() > 0);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_unlink_open_handles_same_name_recreate_and_offline_missing_guard() {
    use std::{io::Write, os::unix::fs::MetadataExt};
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let state = d.path().join("state");
    let mut s = Store::open(&state).unwrap();
    let mut dir = node("dir", 0, "1");
    dir.directory = true;
    s.install_remote_snapshot(
        &[
            node("root", 0, "1"),
            node("a", 3, "1"),
            node("missing", 9, "1"),
            dir,
        ],
        "1",
        "server",
    )
    .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    let path = mount.join("A.doc");
    let mut read = std::fs::File::open(&path).unwrap();
    let mut write = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let ino = write.metadata().unwrap().ino();
    std::fs::remove_file(&path).unwrap();
    assert!(!path.exists());
    assert_eq!(write.metadata().unwrap().nlink(), 0);
    write.set_len(0).unwrap();
    write.write_all(b"orphan writes").unwrap();
    write.sync_all().unwrap();
    let mut bytes = Vec::new();
    read.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"orphan writes");
    std::fs::write(&path, b"new identity").unwrap();
    assert_ne!(std::fs::metadata(&path).unwrap().ino(), ino);
    assert_eq!(std::fs::read(&path).unwrap(), b"new identity");
    let pending = s.lock().unwrap().upload_operations("pending").unwrap();
    assert_eq!(pending.len(), 1);
    assert_ne!(
        s.lock().unwrap().upload_job(&pending[0]).unwrap().node.id,
        "a"
    );
    let missing = std::fs::File::open(mount.join("missing")).unwrap();
    assert_eq!(
        std::fs::remove_file(mount.join("missing"))
            .unwrap_err()
            .raw_os_error(),
        Some(libc::ENETDOWN)
    );
    assert!(mount.join("missing").exists());
    drop(missing);
    std::fs::remove_file(mount.join("missing")).unwrap(); // unopened missing file needs no content
    std::fs::remove_dir(mount.join("dir")).unwrap();
    assert!(!mount.join("dir").exists());
    drop(read);
    drop(write);
    session.join();
    drop(s);
    let s = Store::open(state).unwrap();
    assert_eq!(s.status("a").unwrap(), Status::Deleted);
    let orphan = s.open_dirty_draft("a").unwrap().unwrap();
    let mut bytes = [0; 13];
    orphan.read_at(&mut bytes, 0).unwrap();
    assert_eq!(&bytes, b"orphan writes");
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_atomic_replace_keeps_source_inode_and_old_target_handle() {
    use std::{
        io::Write,
        os::unix::fs::{MetadataExt, PermissionsExt},
    };
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1"), node("a", 3, "1")], "1", "server")
        .unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    let path = mount.join("A.doc");
    let temp = mount.join(".save.tmp");
    let mut old = std::fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(&path)
        .unwrap();
    let mut src = std::fs::File::create(&temp).unwrap();
    src.write_all(b"replacement bytes").unwrap();
    src.sync_all().unwrap();
    let src_ino = src.metadata().unwrap().ino();
    std::fs::rename(&temp, &path).unwrap();
    assert!(!temp.exists());
    assert_eq!(std::fs::metadata(&path).unwrap().ino(), src_ino);
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement bytes");
    assert_eq!(old.metadata().unwrap().nlink(), 0);
    let mut bytes = Vec::new();
    old.read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes, b"old");
    old.set_len(0).unwrap();
    old.write_all(b"orphan old target").unwrap();
    old.sync_all().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement bytes");
    src.write_all(b"!").unwrap();
    src.sync_all().unwrap();
    assert_eq!(std::fs::read(&path).unwrap(), b"replacement bytes!");
    assert_eq!(
        std::fs::metadata(&path).unwrap().permissions().mode() & 0o777,
        0o644
    );
    let q = s.lock().unwrap().next_replace().unwrap().unwrap();
    assert_eq!(q.state, "pending");
    assert!(s.lock().unwrap().next_remove().unwrap().is_none());
    drop(src);
    drop(old);
    session.join();
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_permissions_owner_restore_and_local_capacity_are_real() {
    use std::{
        ffi::CString,
        os::unix::fs::{MetadataExt, PermissionsExt},
    };
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1"), node("a", 3, "1")], "1", "test")
        .unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    let file = mount.join("A.doc");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o640)).unwrap();
    assert_eq!(std::fs::metadata(&file).unwrap().mode() & 0o777, 0o640);
    let path = CString::new(file.to_str().unwrap()).unwrap();
    assert_eq!(unsafe { libc::chown(path.as_ptr(), uid, gid) }, 0);
    assert_eq!(unsafe { libc::chown(path.as_ptr(), uid + 1, gid) }, -1);
    assert_eq!(
        std::io::Error::last_os_error().raw_os_error(),
        Some(libc::EPERM)
    );
    assert_eq!(std::fs::metadata(&file).unwrap().uid(), uid);
    let mut stat = std::mem::MaybeUninit::<libc::statvfs>::uninit();
    assert_eq!(
        unsafe { libc::statvfs(path.as_ptr(), stat.as_mut_ptr()) },
        0
    );
    let stat = unsafe { stat.assume_init() };
    assert!(stat.f_blocks > 0 && stat.f_bavail > 0);
    assert!(stat.f_bavail <= stat.f_blocks);
    assert!(s.lock().unwrap().downloads_needed().unwrap().is_empty());
    assert_eq!(s.lock().unwrap().pending_operations().unwrap(), 0);
    session.join();
    drop(s);
    let s = Store::open(d.path().join("state")).unwrap();
    assert_eq!(s.record("a").unwrap().mode, 0o640);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_offline_nested_mkdir_moves_and_restart_preserve_tree_and_child_bytes() {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1")], "1", "test")
        .unwrap();
    s.pin("root", true).unwrap();
    let s = Arc::new(drive_core::SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, gid)
        .unwrap()
        .with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )
    .unwrap();
    std::fs::create_dir_all(mount.join("A/B")).unwrap();
    std::fs::write(mount.join("A/B/note"), b"offline nested bytes").unwrap();
    assert_eq!(
        std::fs::remove_dir(mount.join("A/B"))
            .unwrap_err()
            .raw_os_error(),
        Some(libc::ENOTEMPTY)
    );
    std::fs::rename(mount.join("A"), mount.join("Renamed")).unwrap();
    assert_eq!(
        std::fs::read(mount.join("Renamed/B/note")).unwrap(),
        b"offline nested bytes"
    );
    let a = s.lock().unwrap().lookup("root", "Renamed").unwrap().node.id;
    let b = s.lock().unwrap().lookup(&a, "B").unwrap().node.id;
    let file = s.lock().unwrap().lookup(&b, "note").unwrap().node.id;
    assert!(s.lock().unwrap().is_pinned(&file).unwrap());
    session.join();
    drop(s);
    let s = Store::open(d.path().join("state")).unwrap();
    assert_eq!(s.lookup("root", "Renamed").unwrap().node.id, a);
    let mut body = Vec::new();
    s.open_cached(&file)
        .unwrap()
        .read_to_end(&mut body)
        .unwrap();
    assert_eq!(body, b"offline nested bytes");
}
