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
    // Atomic replacement must preserve the source mode, including the caller's
    // umask. The isolated validation runner deliberately uses a private umask.
    let src_mode = src.metadata().unwrap().permissions().mode() & 0o777;
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
        src_mode
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

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_sequential_100_mib_write_has_bounded_metadata_commits_and_fsyncs() {
    use std::io::Write;
    // Acceptance for #24: a 100 MiB sequential write must not commit or fsync
    // once per syscall. The only durable metadata commit while writing is the
    // first-edit epoch marker; the draft file is fsynced by the following
    // checkpoint, not by each write or by SETATTR(mtime).
    // SAFETY: this test binary runs a single filtered test, before any other
    // thread could read the environment.
    unsafe {
        std::env::set_var("DRIVE_NATIVE_FS_DIAGNOSTICS", "1");
    }
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let state = d.path().join("state");
    let mut s = Store::open(&state).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1"), node("a", 3, "1")], "1", "server")
        .unwrap();
    const CHUNK: usize = 128 * 1024;
    const CHUNKS: usize = 800;
    const TOTAL: usize = CHUNK * CHUNKS;
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
    let mut file = std::fs::OpenOptions::new().write(true).open(&path).unwrap();
    let before = s.lock().unwrap().draft_write_stats();
    let chunk = vec![0xa5u8; CHUNK];
    let started = std::time::Instant::now();
    // The first write lazily creates the draft: two one-time fsyncs (draft file
    // and drafts directory) plus the one durable dirty-epoch marker.
    file.write_all(&chunk).unwrap();
    let after_first = s.lock().unwrap().draft_write_stats();
    assert_eq!(
        after_first.draft_file_fsyncs - before.draft_file_fsyncs,
        2,
        "draft creation fsyncs once, before any bulk write"
    );
    assert_eq!(
        after_first.metadata_commits - before.metadata_commits,
        1,
        "the first edit durably marks the dirty epoch"
    );
    for _ in 0..CHUNKS - 1 {
        file.write_all(&chunk).unwrap();
    }
    let write_elapsed = started.elapsed();
    let after_writes = s.lock().unwrap().draft_write_stats();
    assert!(
        after_writes.writes - after_first.writes >= CHUNKS as u64 - 1,
        "kernel may split large writes; expected at least {} draft writes",
        CHUNKS - 1
    );
    assert_eq!(
        after_writes.metadata_commits - after_first.metadata_commits,
        0,
        "bulk writes must not commit metadata per syscall"
    );
    assert_eq!(
        after_writes.draft_file_fsyncs - after_first.draft_file_fsyncs,
        0,
        "bulk writes must not fsync the draft file"
    );
    assert_eq!(std::fs::metadata(&path).unwrap().len(), TOTAL as u64);

    let fsync_started = std::time::Instant::now();
    file.sync_all().unwrap();
    let fsync_elapsed = fsync_started.elapsed();
    let after_checkpoint = s.lock().unwrap().draft_write_stats();
    assert_eq!(
        after_checkpoint.metadata_commits - after_writes.metadata_commits,
        1,
        "checkpoint folds its row update into the snapshot transaction"
    );
    assert_eq!(
        after_checkpoint.checkpoints - after_writes.checkpoints,
        1,
        "explicit fsync checkpoints the dirty draft once"
    );
    assert_eq!(
        after_checkpoint.draft_file_fsyncs - after_writes.draft_file_fsyncs,
        1,
        "checkpoint fsyncs the draft file once"
    );
    assert_eq!(
        s.lock().unwrap().status("a").unwrap(),
        Status::PendingUpload
    );
    let samples = drive_fs::diagnostics::snapshot();
    let count_for = |stage: drive_fs::diagnostics::Stage| {
        samples
            .iter()
            .find(|sample| std::mem::discriminant(&sample.stage) == std::mem::discriminant(&stage))
            .map(|sample| sample.count)
            .unwrap_or(0)
    };
    assert!(count_for(drive_fs::diagnostics::Stage::WriteQueue) > 0);
    assert!(count_for(drive_fs::diagnostics::Stage::WriteStoreWait) > 0);
    assert!(count_for(drive_fs::diagnostics::Stage::WriteHold) > 0);
    assert!(
        count_for(drive_fs::diagnostics::Stage::CheckpointTotal) > 0,
        "explicit fsync checkpoint is instrumented"
    );
    println!(
        "#24 kernel sequential_write: {} chunks x {} KiB = {} MiB in {} draft writes, write {:?} ({} MiB/s), checkpoint {:?}, metadata_commits +{}, draft_file_fsyncs +{}",
        CHUNKS,
        CHUNK / 1024,
        TOTAL / (1024 * 1024),
        after_writes.writes - before.writes,
        write_elapsed,
        (TOTAL / (1024 * 1024)) as u128 * 1_000_000 / write_elapsed.as_micros().max(1),
        fsync_elapsed,
        after_checkpoint.metadata_commits - before.metadata_commits,
        after_checkpoint.draft_file_fsyncs - before.draft_file_fsyncs,
    );
    drop(file);
    session.join();
    drop(s);
    let s = Store::open(&state).unwrap();
    assert_eq!(s.status("a").unwrap(), Status::PendingUpload);
    let mut first = [0u8; 16];
    use std::io::Read as _;
    s.open_cached("a").unwrap().read_exact(&mut first).unwrap();
    assert_eq!(first, [0xa5u8; 16]);
    assert_eq!(s.record("a").unwrap().size, TOTAL as u64);
}

#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn kernel_hard_link_is_rejected_with_operation_not_supported() {
    // Acceptance for #18: tools that land files with "write temp + link into
    // place" must see EOPNOTSUPP instead of EPERM so they can fall back, and a
    // rejected link must leave the namespace untouched. Symlinks are a separate
    // limitation (fuser's default EPERM) and stay out of this issue's scope.
    use std::os::unix::fs::MetadataExt;
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    s.install_remote_snapshot(&[node("root", 0, "1")], "1", "test")
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
    let src = mount.join("src");
    std::fs::write(&src, b"hard link source").unwrap();
    let dst = mount.join("dst");
    let error = std::fs::hard_link(&src, &dst).unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::EOPNOTSUPP));
    // Linux defines ENOTSUP and EOPNOTSUPP as the same errno; a portable probe
    // must observe the support verdict too.
    assert_eq!(error.raw_os_error(), Some(libc::ENOTSUP));
    assert_eq!(error.kind(), std::io::ErrorKind::Unsupported);
    assert!(
        !dst.exists(),
        "a rejected hard link must not create the target name"
    );
    assert_eq!(std::fs::metadata(&src).unwrap().nlink(), 1);
    // Repeat the attempt: no cached state may turn the second failure into a
    // different errno.
    assert_eq!(
        std::fs::hard_link(&src, &dst).unwrap_err().raw_os_error(),
        Some(libc::EOPNOTSUPP)
    );
    // Out of scope for #18: symlink keeps fuser's default EPERM.
    assert_eq!(
        std::os::unix::fs::symlink("src", mount.join("sym"))
            .unwrap_err()
            .raw_os_error(),
        Some(libc::EPERM)
    );
    session.join();
    drop(s);
    let s = Store::open(d.path().join("state")).unwrap();
    assert_eq!(s.lookup("root", "src").unwrap().node.name, "src");
    assert!(s.lookup("root", "dst").is_err());
    assert!(s.lookup("root", "sym").is_err());
}
