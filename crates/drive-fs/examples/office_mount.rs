//! Explicit real Writer application test on an unprivileged local FUSE mount.
use drive_core::*;
use std::{
    path::Path,
    process::Command,
    sync::Arc,
};
fn office(
    helper: &Path,
    action: &str,
    document: &Path,
    expected: &str,
    text: Option<&str>,
    trace: Option<&Path>,
) -> Result<()> {
    let mut cmd = Command::new("/usr/bin/timeout");
    cmd.args(["--signal=TERM", "--kill-after=5", "120", "/usr/bin/python3"])
        .arg(helper)
        .arg(action)
        .arg(document)
        .arg(expected);
    if let Some(text) = text {
        cmd.arg("--text").arg(text);
    }
    if let Some(trace) = trace {
        cmd.arg("--trace").arg(trace);
    }
    let result = cmd.output()?;
    print!("{}", String::from_utf8_lossy(&result.stdout));
    if !result.status.success() {
        eprintln!("{}", String::from_utf8_lossy(&result.stderr));
        return Err(Error::State("LibreOffice command failed".into()));
    }
    Ok(())
}
fn main() -> Result<()> {
    let uid = unsafe { libc::getuid() };
    let gid = unsafe { libc::getgid() };
    assert_ne!(uid, 0);
    let args: Vec<_> = std::env::args().collect();
    let helper = Path::new(args.get(1).expect("UNO helper"));
    let evidence = Path::new(args.get(2).expect("trace directory"));
    std::fs::create_dir_all(evidence)?;
    let d = tempfile::tempdir()?;
    let seed = d.path().join("seed.docx");
    office(helper, "create", &seed, "Drive Native seed", None, None)?;
    let bytes = std::fs::read(&seed)?;
    let mut s = Store::open(d.path().join("state"))?;
    let root = Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "root1".into(),
        directory: true,
        metadata: Some(NodeMetadata {
            size: 0,
            mtime: 1700000000,
            mode: 0o755,
        }),
    };
    let file = Node {
        id: "20".into(),
        parent: Some("root".into()),
        name: "A.docx".into(),
        version: "v1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: bytes.len() as u64,
            mtime: 1700000000,
            mode: 0o644,
        }),
    };
    s.install_remote_snapshot(&[root, file], "1", "server")?;
    s.publish_download(
        &DownloadTicket {
            id: "20".into(),
            version: "v1".into(),
        },
        &bytes,
        &sha256(&bytes),
    )?;
    let store = Arc::new(drive_core::SharedStore::new(s));
    let mount = d.path().join("mount");
    std::fs::create_dir(&mount)?;
    let fs = drive_fs::CacheFilesystem::new(store.clone(), "root", uid, gid)?.with_writes_enabled();
    let session = fuser::spawn_mount2(
        fs,
        &mount,
        &[
            fuser::MountOption::RW,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
        ],
    )?;
    let path = mount.join("A.docx");
    let result = office(
        helper,
        "edit",
        &path,
        "Drive Native seed",
        Some("Saved by real LibreOffice Writer"),
        Some(&evidence.join("writer-trace")),
    );
    let read = if result.is_ok() {
        office(
            helper,
            "read",
            &path,
            "Saved by real LibreOffice Writer",
            None,
            None,
        )
    } else {
        Ok(())
    };
    let names = std::fs::read_dir(&mount)?
        .map(|e| e.unwrap().file_name().to_string_lossy().to_string())
        .collect::<Vec<_>>();
    println!("remaining_names={names:?}");
    println!(
        "queue_kind={:?}",
        store.lock().unwrap().next_namespace_kind()?
    );
    session.join();
    result?;
    read?;
    assert_eq!(names, vec!["A.docx"]);
    println!("real_writer_local_fuse_save_reopen_passed");
    Ok(())
}
