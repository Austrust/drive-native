//! Synthetic ACL truth only. No remote protocol, credentials, or user files.
#![cfg(feature = "test-seams")]
use drive_core::{
    write_permissions::{self as p, Action, Decision},
    *,
};
use std::{
    fs::{self, OpenOptions},
    io::{Read, Seek, SeekFrom, Write},
    os::unix::fs::PermissionsExt,
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
        let mut nodes = [
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
        })
        .to_vec();
        if root == "A" {
            // An independent task indexes the selected subtree, just as
            // native::project does. Do not change an existing mount's inode 1.
            nodes.retain(|node| node.id == "A" || node.parent.as_deref() == Some("A"));
            nodes.iter_mut().find(|node| node.id == "A").unwrap().parent = None;
        }
        s.install_remote_snapshot(&nodes, "1", "server").unwrap();
        for id in ["file", "target"] {
            if !nodes.iter().any(|node| node.id == id) {
                continue;
            }
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
fn parent_self_permissions_do_not_deny_independent_child_removal_or_rename() {
    {
        let f = Fixture::new("root");
        for action in [Action::CreateFile, Action::CreateDir, Action::Delete,
            Action::RenameSource, Action::RenameTarget, Action::Setattr] {
            f.decision("root", "A", action, Decision::Deny);
        }
        f.decision("root", "file", Action::Delete, Decision::Allow);
        // default_permissions must not turn the parent's self-action denials
        // into a mode-based child deletion refusal before FUSE sees the target.
        fs::remove_file(f.mount.join("A/file")).unwrap();
        denied(fs::remove_dir(f.mount.join("A")));
        denied(fs::create_dir(f.mount.join("A/forbidden")));
        denied(fs::write(f.mount.join("A/forbidden-file"), b"no"));
    }
    {
        let f = Fixture::new("root");
        f.decision("root", "root", Action::Delete, Decision::Deny);
        f.decision("root", "root", Action::CreateDir, Decision::Deny);
        f.decision("root", "A", Action::RenameSource, Decision::Deny);
        f.decision("root", "B", Action::Delete, Decision::Deny);
        f.decision("root", "file", Action::RenameSource, Decision::Allow);
        f.decision("root", "target", Action::Delete, Decision::Allow);
        fs::rename(f.mount.join("A/file"), f.mount.join("B/target")).unwrap();
        assert_eq!(fs::read(f.mount.join("B/target")).unwrap(), b"old");
        // Removing now-empty A does not require the root's create-directory
        // permission, nor permission to remove root itself.
        fs::remove_dir(f.mount.join("A")).unwrap();
    }
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
    denied(fs::set_permissions(f.mount.join("A/file"), fs::Permissions::from_mode(0o600)));
    assert_eq!(fs::read(f.mount.join("A/file")).unwrap(), b"ed");
}

#[test]
#[ignore = "requires ordinary Linux user, /dev/fuse and test-seams; isolated ACL truth"]
fn setattr_only_persists_mtime_without_granting_content_writes_or_uploads() {
    let mut f = Fixture::new("root");
    f.decision("root", "file", Action::WriteContent, Decision::Deny);
    f.decision("root", "file", Action::Setattr, Decision::Allow);
    let path = f.mount.join("A/file");
    let modified = std::time::UNIX_EPOCH + std::time::Duration::from_secs(1234);
    let file = fs::File::open(&path).unwrap();
    file.set_modified(modified).unwrap();
    assert_eq!(fs::metadata(&path).unwrap().modified().unwrap(), modified);
    assert_eq!(fs::read(&path).unwrap(), b"old");
    denied(OpenOptions::new().write(true).open(&path));
    denied(OpenOptions::new().write(true).truncate(true).open(&path));
    {
        let s = f.store.lock().unwrap();
        let pending = s.upload_operations("pending").unwrap();
        assert_eq!(pending.len(), 1);
        let job = s.upload_job(&pending[0]).unwrap();
        assert_eq!(job.mtime, Some(1234));
        // Native mtime still uses a full-body upload. A local Setattr grant
        // must never override that distinct remote operation's content gate.
        assert!(p::targets_denied(&s, &p::upload_targets(&job.node)).unwrap());
        assert_eq!(s.pending_operations().unwrap(), 1);
    }
    drop(file);
    // Keep only this synthetic state while stopping the mount and closing Store.
    let saved = std::mem::replace(&mut f._temp, tempfile::tempdir().unwrap());
    drop(f);
    let s = Store::open(saved.path().join("state")).unwrap();
    let draft = s.open_dirty_draft("file").unwrap().unwrap();
    assert_eq!(draft.mtime().unwrap(), 1234);
    let mut bytes = [0; 3];
    assert_eq!(draft.read_at(&mut bytes, 0).unwrap(), 3);
    assert_eq!(&bytes, b"old");
    assert_eq!(s.upload_operations("pending").unwrap().len(), 1);
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
    denied(fs::metadata(f.mount.join("A/missing")));
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
