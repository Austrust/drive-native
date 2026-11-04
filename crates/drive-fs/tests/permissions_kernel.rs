//! Synthetic ACL truth only. No remote protocol, credentials, or user files.
#![cfg(feature = "test-seams")]
use drive_core::{
    write_permissions::{self as p, Action, Decision},
    *,
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    sync::Arc,
};
struct Fixture {
    session: Option<fuser::BackgroundSession>,
    store: Arc<SharedStore>,
    mount: std::path::PathBuf,
    _temp: tempfile::TempDir,
}
impl Drop for Fixture {
    fn drop(&mut self) {
        if let Some(s) = self.session.take() {
            s.join();
        }
    }
}
impl Fixture {
    fn new(root: &str) -> Self {
        assert_ne!(unsafe { libc::getuid() }, 0);
        let temp = tempfile::tempdir().unwrap();
        let mount = temp.path().join("mount");
        fs::create_dir(&mount).unwrap();
        let mut s = Store::open(temp.path().join("state")).unwrap();
        let nodes = [
            ("root", None, true),
            ("A", Some("root"), true),
            ("B", Some("root"), true),
            ("file", Some("A"), false),
            ("target", Some("B"), false),
        ]
        .map(|(id, parent, directory)| Node {
            id: id.into(),
            name: id.into(),
            parent: parent.map(str::to_owned),
            directory,
            version: "v1".into(),
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: if directory { 0o755 } else { 0o644 },
            }),
        });
        s.install_remote_snapshot(&nodes, "1", "server").unwrap();
        for id in ["file", "target"] {
            s.publish_download(
                &DownloadTicket {
                    id: id.into(),
                    version: "v1".into(),
                },
                b"old",
                &sha256(b"old"),
            )
            .unwrap();
        }
        let store = Arc::new(SharedStore::new(s));
        let mut fs = drive_fs::CacheFilesystem::new(
            store.clone(),
            root,
            unsafe { libc::getuid() },
            unsafe { libc::getgid() },
        )
        .unwrap()
        .with_writes_enabled();
        if drive_fs::namespace_index_enabled() {
            let index = namespace_index::build(&store.lock().unwrap()).unwrap();
            fs = fs.with_namespace_index(Arc::new(index));
        }
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
        Self {
            session: Some(session),
            store,
            mount,
            _temp: temp,
        }
    }
    fn decision(&self, root: &str, node: &str, action: Action, decision: Decision) {
        p::inject(&self.store.lock().unwrap(), root, node, action, decision).unwrap();
    }
}
fn denied<T: std::fmt::Debug>(r: std::io::Result<T>) {
    assert_eq!(r.unwrap_err().raw_os_error(), Some(libc::EACCES));
}

#[test]
#[ignore = "requires ordinary Linux user, /dev/fuse and test-seams; isolated ACL truth"]
fn root_a_b_and_action_classes_are_enforced_in_one_mount() {
    let f = Fixture::new("root");
    for node in ["root", "B"] {
        f.decision("root", node, Action::CreateDir, Decision::Deny);
    }
    f.decision("root", "A", Action::CreateDir, Decision::Allow);
    f.decision("root", "A", Action::CreateFile, Decision::Deny);
    denied(fs::create_dir(f.mount.join("bad")));
    denied(fs::create_dir(f.mount.join("B/bad")));
    fs::create_dir(f.mount.join("A/good")).unwrap();
    denied(fs::write(f.mount.join("A/new"), b"bytes"));
    f.decision("root", "file", Action::WriteContent, Decision::Allow);
    f.decision("root", "file", Action::Setattr, Decision::Deny);
    f.decision("root", "file", Action::Delete, Decision::Deny);
    f.decision("root", "file", Action::RenameSource, Decision::Deny);
    let mut file = OpenOptions::new()
        .write(true)
        .open(f.mount.join("A/file"))
        .unwrap();
    file.write_all(b"edit").unwrap();
    file.set_len(2).unwrap();
    denied(fs::remove_file(f.mount.join("A/file")));
    denied(fs::rename(f.mount.join("A/file"), f.mount.join("B/moved")));
    assert_eq!(fs::read(f.mount.join("A/file")).unwrap(), b"ed");
}

#[test]
#[ignore = "requires ordinary Linux user, /dev/fuse and test-seams; isolated ACL truth"]
fn live_revocation_restoration_and_list_vs_traverse() {
    let f = Fixture::new("root");
    let mut file = std::fs::File::open(f.mount.join("A/file")).unwrap();
    f.decision("root", "A", Action::List, Decision::Deny);
    denied(fs::read_dir(f.mount.join("A")));
    assert_eq!(fs::read(f.mount.join("A/file")).unwrap(), b"old");
    f.decision("root", "file", Action::Read, Decision::Deny);
    let mut bytes = [0; 3];
    denied(file.read(&mut bytes));
    f.decision("root", "file", Action::Read, Decision::Unknown);
    file.seek(SeekFrom::Start(0)).unwrap();
    file.read_exact(&mut bytes).unwrap();
    assert_eq!(&bytes, b"old");
    f.decision("root", "A", Action::Traverse, Decision::Deny);
    denied(std::fs::File::open(f.mount.join("A/file")));
    assert_eq!(fs::read(f.mount.join("B/target")).unwrap(), b"old");
}

#[test]
#[ignore = "requires ordinary Linux user, /dev/fuse and test-seams; isolated ACL truth"]
fn target_parent_cover_target_and_independent_submount() {
    {
        let f = Fixture::new("root");
        f.decision("root", "B", Action::RenameTarget, Decision::Deny);
        denied(fs::rename(f.mount.join("A/file"), f.mount.join("B/new")));
        f.decision("root", "B", Action::RenameTarget, Decision::Allow);
        f.decision("root", "target", Action::Delete, Decision::Deny);
        denied(fs::rename(f.mount.join("A/file"), f.mount.join("B/target")));
        f.decision("root", "target", Action::Delete, Decision::Allow);
        fs::rename(f.mount.join("A/file"), f.mount.join("B/target")).unwrap();
        assert_eq!(fs::read(f.mount.join("B/target")).unwrap(), b"old");
    }
    let f = Fixture::new("A");
    f.decision("A", "root", Action::CreateDir, Decision::Deny);
    f.decision("A", "A", Action::CreateDir, Decision::Allow);
    fs::create_dir(f.mount.join("independent")).unwrap();
    p::set_local_policy(
        &f.store.lock().unwrap(),
        "A",
        p::LocalPolicy::ExplicitReadOnly,
    )
    .unwrap();
    assert_eq!(
        fs::create_dir(f.mount.join("ro"))
            .unwrap_err()
            .raw_os_error(),
        Some(libc::EROFS)
    );
}
