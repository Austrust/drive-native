//! Synthetic kernel timestamp operations. No server or personal mount involved.
use drive_core::*;
use std::{
    os::unix::fs::MetadataExt,
    sync::Arc,
    time::{Duration, UNIX_EPOCH},
};
fn setup() -> (
    tempfile::TempDir,
    Arc<SharedStore>,
    fuser::BackgroundSession,
    std::path::PathBuf,
) {
    let uid = unsafe { libc::getuid() };
    assert_ne!(uid, 0);
    let d = tempfile::tempdir().unwrap();
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount).unwrap();
    let mut s = Store::open(d.path().join("state")).unwrap();
    let nodes =
        [("root", true, 0), ("file", false, 3), ("missing", false, 3)].map(|(id, dir, size)| {
            Node {
                id: id.into(),
                parent: (!dir).then(|| "root".into()),
                name: id.into(),
                version: "1".into(),
                directory: dir,
                metadata: Some(NodeMetadata {
                    size,
                    mtime: 1700000000,
                    mode: if dir { 0o755 } else { 0o644 },
                }),
            }
        });
    s.install_remote_snapshot(&nodes, "1", "fixture").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "file".into(),
            version: "1".into(),
        },
        b"abc",
        &sha256(b"abc"),
    )
    .unwrap();
    let s = Arc::new(SharedStore::new(s));
    let fs = drive_fs::CacheFilesystem::new(s.clone(), "root", uid, unsafe { libc::getgid() })
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
    (d, s, session, mount)
}
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn touch_and_cp_preserve_file_mtime_with_durable_upload_intent() {
    let (d, s, session, mount) = setup();
    assert!(
        std::process::Command::new("touch")
            .arg(mount.join("created"))
            .status()
            .unwrap()
            .success()
    );
    let held = std::fs::File::open(mount.join("file")).unwrap();
    held.set_modified(UNIX_EPOCH + Duration::new(1234567890, 987654321))
        .unwrap();
    assert_eq!(held.metadata().unwrap().mtime(), 1234567890);
    assert_eq!(held.metadata().unwrap().mtime_nsec(), 0);
    assert_eq!(std::fs::read(mount.join("file")).unwrap(), b"abc");
    let source = d.path().join("source");
    std::fs::write(&source, b"copy bytes").unwrap();
    std::fs::File::open(&source)
        .unwrap()
        .set_modified(UNIX_EPOCH + Duration::from_secs(1234567000))
        .unwrap();
    assert!(
        std::process::Command::new("cp")
            .arg("-p")
            .arg(&source)
            .arg(mount.join("copied"))
            .status()
            .unwrap()
            .success()
    );
    assert_eq!(
        std::fs::metadata(mount.join("copied")).unwrap().mtime(),
        1234567000
    );
    let store = s.lock().unwrap();
    let ops = store.upload_operations("pending").unwrap();
    let jobs = ops
        .iter()
        .map(|op| store.upload_job(op).unwrap())
        .collect::<Vec<_>>();
    assert!(
        jobs.iter()
            .any(|j| j.node.id == "file" && j.mtime == Some(1234567890))
    );
    assert!(
        jobs.iter()
            .any(|j| j.node.name == "copied" && j.mtime == Some(1234567000))
    );
    drop(store);
    drop(held);
    drop(session);
}
#[test]
#[ignore = "requires ordinary Linux user and /dev/fuse; run explicitly"]
fn atime_omit_does_not_download_and_directory_mtime_is_honestly_unsupported() {
    let (_d, s, session, mount) = setup();
    // Opening a missing body can be lazy; utimens with only atime must not read it.
    let missing = std::fs::File::open(mount.join("missing")).unwrap();
    missing
        .set_times(std::fs::FileTimes::new().set_accessed(UNIX_EPOCH + Duration::from_secs(111)))
        .unwrap();
    assert_eq!(missing.metadata().unwrap().atime(), 111);
    assert_eq!(missing.metadata().unwrap().mtime(), 1700000000);
    assert!(
        s.lock()
            .unwrap()
            .upload_operations("pending")
            .unwrap()
            .is_empty()
    );
    assert!(s.lock().unwrap().downloads_needed().unwrap().is_empty());
    let dir = std::fs::File::open(&mount).unwrap();
    let error = dir
        .set_times(
            std::fs::FileTimes::new()
                .set_accessed(UNIX_EPOCH + Duration::from_secs(222))
                .set_modified(UNIX_EPOCH + Duration::from_secs(333)),
        )
        .unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::EOPNOTSUPP));
    assert_eq!(
        s.lock().unwrap().local_atime("root", 1700000000).unwrap(),
        (1700000000, 0)
    );
    let file = std::fs::File::open(mount.join("file")).unwrap();
    let error = file
        .set_modified(UNIX_EPOCH - Duration::from_secs(1))
        .unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::EINVAL));
    drop(file);
    drop(dir);
    drop(missing);
    drop(session);
}
