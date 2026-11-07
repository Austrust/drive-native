//! A read-only handle is a second connection for callers that must not queue
//! behind the serialized writer: it takes no `writer.lock`, creates no files
//! and never migrates schema. These tests pin the public contract and the
//! failure modes that make callers fall back to the `SharedStore` boundary.
use drive_core::*;
use std::io::Read;

fn node(id: &str, parent: Option<&str>, version: &str, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: format!("{id}.doc"),
        version: version.into(),
        directory,
        metadata: None,
    }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.install_remote_snapshot(
        &[
            node("root", None, "r", true),
            node("a", Some("root"), "v1", false),
        ],
        "1",
        "server",
    )
    .unwrap();
    s.ensure_fs_root("root").unwrap();
    s.publish_download(
        &DownloadTicket {
            id: "a".into(),
            version: "v1".into(),
        },
        b"old",
        &sha256(b"old"),
    )
    .unwrap();
    (d, s)
}
fn receive(s: &mut Store, from: &str, to: &str, n: Node) {
    s.receive(&Batch {
        from: from.into(),
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change: Change::Upsert(n),
        }],
    })
    .unwrap();
}
fn open_err(root: &std::path::Path) -> Error {
    match Store::open_read_only(root) {
        Ok(_) => panic!("read-only open unexpectedly succeeded"),
        Err(err) => err,
    }
}

#[test]
fn read_only_handle_works_while_the_writer_lock_is_held() {
    let (d, mut s) = fixture();
    let ro = Store::open_read_only(d.path()).unwrap();
    assert_eq!(ro.root(), d.path());
    assert_eq!(ro.node("a").unwrap().version, "v1");
    assert_eq!(ro.cursor().unwrap(), "1");
    // The handle took no ownership: the live writer still excludes a second
    // writer through writer.lock.
    assert!(matches!(Store::open(d.path()), Err(Error::State(_))));
    // Committed writer transactions become visible without reopening the handle.
    receive(&mut s, "1", "2", node("a", Some("root"), "v2", false));
    s.drain().unwrap();
    assert_eq!(ro.node("a").unwrap().version, "v2");
    // Cached content reads are read-only as well: no Store lock is needed.
    let mut bytes = Vec::new();
    ro.open_cached("a")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"old");
}

#[test]
fn read_only_handle_does_not_observe_uncommitted_inbox_events() {
    let (d, mut s) = fixture();
    let ro = Store::open_read_only(d.path()).unwrap();
    // receive() only stages the batch; drain() commits it in one transaction.
    receive(&mut s, "1", "2", node("a", Some("root"), "v2", false));
    assert_eq!(ro.node("a").unwrap().version, "v1");
    assert_eq!(ro.cursor().unwrap(), "1");
    s.drain().unwrap();
    assert_eq!(ro.node("a").unwrap().version, "v2");
    assert_eq!(ro.cursor().unwrap(), "2");
}

#[test]
fn read_only_handle_rejects_missing_database_and_creates_nothing() {
    let d = tempfile::tempdir().unwrap();
    let err = open_err(d.path());
    assert!(
        matches!(&err, Error::State(m) if m.contains("does not exist")),
        "{err:?}"
    );
    assert!(!d.path().join("state.sqlite").exists());
    assert!(!d.path().join("writer.lock").exists());
}

#[test]
fn read_only_handle_rejects_schema_mismatch_without_backup_or_migration() {
    let (d, s) = fixture();
    drop(s);
    // Holding a connection keeps the WAL side files alive while the read-only
    // open below is expected to fail on the version check.
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION + 1)
        .unwrap();
    let err = open_err(d.path());
    assert!(
        matches!(&err, Error::State(m) if m.contains("newer")),
        "{err:?}"
    );
    db.pragma_update(None, "user_version", STATE_SCHEMA_VERSION - 1)
        .unwrap();
    let err = open_err(d.path());
    assert!(
        matches!(&err, Error::State(m) if m.contains("current schema")),
        "{err:?}"
    );
    // A refused read-only open must not migrate, back up or otherwise write.
    assert!(!std::fs::read_dir(d.path()).unwrap().any(|entry| {
        entry
            .unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with("state-before-")
    }));
    assert_eq!(
        db.pragma_query_value(None, "user_version", |r| r.get::<_, i64>(0))
            .unwrap(),
        STATE_SCHEMA_VERSION - 1
    );
}

#[test]
fn read_only_handle_rejects_non_wal_database() {
    let (d, s) = fixture();
    drop(s);
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.pragma_update(None, "journal_mode", "DELETE").unwrap();
    let err = open_err(d.path());
    assert!(
        matches!(&err, Error::State(m) if m.contains("WAL")),
        "{err:?}"
    );
}

#[test]
fn read_only_handle_takes_no_writer_lock_and_rejects_writes() {
    let (d, s) = fixture();
    drop(s);
    std::fs::remove_file(d.path().join("writer.lock")).unwrap();
    let mut ro = Store::open_read_only(d.path()).unwrap();
    assert!(!d.path().join("writer.lock").exists());
    // A writer can start beside the read-only handle because it holds nothing.
    let writer = Store::open(d.path()).unwrap();
    assert_eq!(ro.node("a").unwrap().version, "v1");
    // query_only rejects accidental writes through the mutable Store API too.
    assert!(ro.pin("a", true).is_err());
    assert!(!writer.is_pinned("a").unwrap());
}

#[test]
fn shared_store_answers_root_without_waiting_for_the_store_lock() {
    let (d, s) = fixture();
    assert_eq!(s.root(), d.path());
    let shared = std::sync::Arc::new(SharedStore::new(s));
    let guard = shared.lock().unwrap();
    let root = shared.root();
    // Reading the captured root from another thread must not wait for the
    // Store mutex; a timeout here means SharedStore::root started locking.
    let (tx, rx) = std::sync::mpsc::channel();
    let other = shared.clone();
    let thread = std::thread::spawn(move || {
        let _ = tx.send(other.root());
    });
    let from_thread = rx
        .recv_timeout(std::time::Duration::from_secs(10))
        .expect("SharedStore::root must not wait for the Store lock");
    assert_eq!(from_thread.as_path(), d.path());
    assert_eq!(root.as_path(), d.path());
    drop(guard);
    thread.join().unwrap();
}
