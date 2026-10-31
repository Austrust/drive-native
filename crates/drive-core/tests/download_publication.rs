use drive_core::*;
use std::{
    io::{Cursor, Read},
    sync::{Arc, mpsc},
    time::Duration,
};

fn node(id: &str, version: &str) -> Node {
    Node {
        id: id.into(),
        parent: Some("root".into()),
        name: "file".into(),
        version: version.into(),
        directory: false,
        metadata: None,
    }
}
fn root() -> Node {
    Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "r1".into(),
        directory: true,
        metadata: None,
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let dir = tempfile::tempdir().unwrap();
    let mut store = Store::open(dir.path()).unwrap();
    store
        .install_remote_snapshot(&[root(), node("a", "v1")], "1", "server")
        .unwrap();
    store
        .publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    (dir, store)
}
fn ticket(version: &str) -> DownloadTicket {
    DownloadTicket {
        id: "a".into(),
        version: version.into(),
    }
}
fn read(store: &Store) -> Vec<u8> {
    let mut out = Vec::new();
    store
        .open_cached("a")
        .unwrap()
        .read_to_end(&mut out)
        .unwrap();
    out
}
fn prepared(store: &Store, bytes: &[u8]) -> download_publication::PreparedDownload {
    store
        .prepare_download(&ticket("v1"))
        .unwrap()
        .prepare(
            &mut Cursor::new(bytes),
            bytes.len() as u64,
            &sha256(bytes),
            || false,
        )
        .unwrap()
}
struct GatedReader {
    step: usize,
    reached: mpsc::Sender<usize>,
    resume: mpsc::Receiver<()>,
}
impl Read for GatedReader {
    fn read(&mut self, out: &mut [u8]) -> std::io::Result<usize> {
        if self.step > 0 {
            self.reached.send(self.step).unwrap();
            self.resume.recv_timeout(Duration::from_secs(10)).unwrap();
        }
        self.step += 1;
        if self.step <= 2 {
            out[..4].copy_from_slice(b"data");
            Ok(4)
        } else {
            Ok(0)
        }
    }
}
#[test]
fn slow_copy_and_pre_verification_do_not_hold_store_and_gc_protects_both_phases() {
    let (_dir, store) = fixture();
    let store = Arc::new(SharedStore::new(store));
    let reservation = store
        .lock()
        .unwrap()
        .prepare_download(&ticket("v1"))
        .unwrap();
    let (reached, wait) = mpsc::channel();
    let (resume, gate) = mpsc::channel();
    let worker = std::thread::spawn(move || {
        reservation
            .prepare(
                &mut GatedReader {
                    step: 0,
                    reached,
                    resume: gate,
                },
                8,
                &sha256(b"datadata"),
                || false,
            )
            .unwrap()
    });
    for expected in [1, 2] {
        assert_eq!(
            wait.recv_timeout(Duration::from_secs(10)).unwrap(),
            expected
        );
        // A try_lock is a deterministic concurrency assertion, not a latency
        // benchmark. Check once mid-copy and once before EOF/hash finalization.
        let mut guard = store.try_lock().expect("preparation retained Store");
        assert_eq!(guard.node("a").unwrap().version, "v1");
        assert_eq!(read(&guard), b"old");
        assert_eq!(guard.collect_garbage().unwrap(), 0);
        drop(guard);
        resume.send(()).unwrap();
    }
    let body = worker.join().unwrap();
    let mut guard = store.lock().unwrap();
    // Durable UUID exists now, but there is still no database reference.
    assert_eq!(guard.collect_garbage().unwrap(), 0);
    let mut opened_old = guard.open_cached("a").unwrap();
    guard.publish_prepared_download(body).unwrap();
    assert_eq!(read(&guard), b"datadata");
    assert_eq!(guard.collect_garbage().unwrap(), 0);
    let mut old = Vec::new();
    opened_old.read_to_end(&mut old).unwrap();
    assert_eq!(old, b"old");
    drop(opened_old);
    assert_eq!(guard.collect_garbage().unwrap(), 3);
}
#[test]
fn competing_publication_remote_version_and_different_identity_reject_late_body() {
    let (_dir, mut store) = fixture();
    let late = prepared(&store, b"late");
    store
        .publish_download(&ticket("v1"), b"winner", &sha256(b"winner"))
        .unwrap();
    assert!(store.publish_prepared_download(late).is_err());
    assert_eq!(read(&store), b"winner");
    let late = prepared(&store, b"late");
    store
        .install_remote_snapshot(&[root(), node("a", "v2")], "2", "server")
        .unwrap();
    assert!(store.publish_prepared_download(late).is_err());
    assert_eq!(read(&store), b"winner");
    let late = store
        .prepare_download(&ticket("v2"))
        .unwrap()
        .prepare(&mut Cursor::new(b"late"), 4, &sha256(b"late"), || false)
        .unwrap();
    store
        .install_remote_snapshot(&[root(), node("replacement", "v2")], "3", "server")
        .unwrap();
    assert!(store.publish_prepared_download(late).is_err());
    assert!(store.open_cached("replacement").is_err());
}
#[test]
fn dirty_blob_and_live_draft_prevent_prepared_download_from_overwriting_edits() {
    let (_dir, mut store) = fixture();
    let late = prepared(&store, b"late");
    store.save_local("a", b"edit").unwrap();
    assert!(store.publish_prepared_download(late).is_err());
    assert_eq!(read(&store), b"edit");
    let (_dir2, mut store) = fixture();
    let late = prepared(&store, b"late");
    let old = store.open_cached("a").unwrap();
    let draft = store.begin_draft("a", "v1", Some(&old)).unwrap();
    store.draft_write(&draft, 0, b"edit", false).unwrap();
    assert!(store.publish_prepared_download(late).is_err());
    let mut bytes = [0; 4];
    assert_eq!(draft.read_at(&mut bytes, 0).unwrap(), 4);
    assert_eq!(&bytes, b"edit");
}
#[test]
fn integrity_read_fault_and_cancellation_keep_old_body_and_release_reservations() {
    let (dir, mut store) = fixture();
    for (bytes, size, digest) in [
        (b"bad".as_slice(), 3, sha256(b"new")),
        (b"short".as_slice(), 20, sha256(b"short")),
    ] {
        assert!(
            store
                .prepare_download(&ticket("v1"))
                .unwrap()
                .prepare(&mut Cursor::new(bytes), size, &digest, || false)
                .is_err()
        );
        assert_eq!(read(&store), b"old");
    }
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("synthetic read fault"))
        }
    }
    assert!(
        store
            .prepare_download(&ticket("v1"))
            .unwrap()
            .prepare(&mut Broken, 3, &sha256(b"new"), || false)
            .is_err()
    );
    let before: Vec<_> = std::fs::read_dir(dir.path().join("blobs"))
        .unwrap()
        .map(|e| e.unwrap().file_name())
        .collect();
    let result = store.prepare_download(&ticket("v1")).unwrap().prepare(
        &mut Cursor::new(b"new"),
        3,
        &sha256(b"new"),
        || {
            // Cancel after the durable final name appears, before publication.
            std::fs::read_dir(dir.path().join("blobs"))
                .unwrap()
                .any(|e| {
                    let e = e.unwrap();
                    e.file_type().unwrap().is_file()
                        && e.file_name()
                            .to_str()
                            .is_some_and(|name| uuid::Uuid::parse_str(name).is_ok())
                        && !before.contains(&e.file_name())
                })
        },
    );
    assert!(matches!(result,Err(Error::Io(e)) if e.kind()==std::io::ErrorKind::Interrupted));
    assert_eq!(read(&store), b"old");
    assert_eq!(store.collect_garbage().unwrap(), 3);
}
#[test]
fn prepared_body_is_bound_to_store_lifetime_and_not_reusable_after_reopen() {
    let (dir, store) = fixture();
    let body = prepared(&store, b"new");
    drop(store);
    let mut reopened = Store::open(dir.path()).unwrap();
    assert!(reopened.publish_prepared_download(body).is_err());
    assert_eq!(read(&reopened), b"old");
    assert_eq!(reopened.collect_garbage().unwrap(), 3);
}

#[test]
fn startup_reclaims_only_generated_staging_files_after_writer_ownership() {
    let (dir, mut store) = fixture();
    store.save_local("a", b"pending upload").unwrap();
    let cached = store.open_cached("a").unwrap();
    let draft = store
        .begin_draft("a", cached.version(), Some(&cached))
        .unwrap();
    store.draft_write(&draft, 0, b"draft", false).unwrap();
    let token = draft.token.clone();
    drop(draft);
    drop(cached);
    let pending = store.pending_operations().unwrap();
    let staging = dir.path().join("blobs");
    std::fs::create_dir_all(&staging).unwrap();
    let generated = tempfile::Builder::new()
        .prefix(".drive-download-")
        .suffix(".part")
        .rand_bytes(12)
        .tempfile_in(&staging)
        .unwrap();
    let (_file, leftover) = generated.keep().unwrap();
    std::fs::write(&leftover, b"crash fragment").unwrap();
    std::fs::write(staging.join("unknown.txt"), b"retain").unwrap();
    let child = staging.join(".drive-download-123456789012.part");
    std::fs::create_dir(&child).unwrap();
    std::fs::write(child.join("nested"), b"retain").unwrap();
    let link = staging.join(".drive-download-abcdefghijkl.part");
    std::os::unix::fs::symlink(dir.path().join("drafts").join(&token), &link).unwrap();
    assert!(Store::open(dir.path()).is_err());
    assert!(
        leftover.is_file(),
        "a competing writer must not clean staging"
    );
    drop(store);
    let reopened = Store::open(dir.path()).unwrap();
    assert!(!leftover.exists());
    assert!(staging.join("unknown.txt").is_file());
    assert!(child.join("nested").is_file());
    assert!(
        std::fs::symlink_metadata(link)
            .unwrap()
            .file_type()
            .is_symlink()
    );
    assert_eq!(reopened.pending_operations().unwrap(), pending);
    assert_eq!(read(&reopened), b"pending upload");
    let draft = reopened.open_dirty_draft("a").unwrap().unwrap();
    let mut bytes = [0; 5];
    draft.read_at(&mut bytes, 0).unwrap();
    assert_eq!(&bytes, b"draft");
}
