use drive_core::*;
use std::io::{Cursor, Read};

fn setup(s: &mut Store, from: &str, to: &str) {
    s.receive(&Batch {
        from: from.into(),
        to: to.into(),
        events: vec![Event {
            id: to.into(),
            change: Change::Upsert(Node {
                id: "a".into(),
                parent: None,
                name: "A.doc".into(),
                version: to.into(),
                directory: false,
                metadata: None,
            }),
        }],
    })
    .unwrap();
    s.drain().unwrap();
}
fn ticket(version: &str) -> DownloadTicket {
    DownloadTicket {
        id: "a".into(),
        version: version.into(),
    }
}
#[test]
fn native_scope_binding_is_atomic_and_rejects_different_task() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.bind_native_scope("server-a", "root-a").unwrap();
    assert!(s.bind_native_scope("server-a", "root-b").is_err());
    assert!(s.bind_native_scope("server-b", "root-a").is_err());
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    s.bind_native_scope("server-a", "root-a").unwrap();
}
fn read(s: &Store) -> Vec<u8> {
    let mut bytes = Vec::new();
    s.open_cached("a").unwrap().read_to_end(&mut bytes).unwrap();
    bytes
}

#[test]
fn streamed_import_keeps_complete_old_content_on_hash_size_and_read_failure() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    setup(&mut s, "", "v1");
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    setup(&mut s, "v1", "v2");
    assert!(
        s.publish_download_reader(
            &ticket("v2"),
            &mut Cursor::new(b"partial"),
            100,
            &sha256(b"complete")
        )
        .is_err()
    );
    assert!(
        s.publish_download_reader(&ticket("v2"), &mut Cursor::new(b"bad"), 3, &sha256(b"new"))
            .is_err()
    );
    struct Broken;
    impl Read for Broken {
        fn read(&mut self, _: &mut [u8]) -> std::io::Result<usize> {
            Err(std::io::Error::other("injected read fault"))
        }
    }
    assert!(
        s.publish_download_reader(&ticket("v2"), &mut Broken, 3, &sha256(b"new"))
            .is_err()
    );
    assert_eq!(read(&s), b"old");
    s.publish_download_reader(&ticket("v2"), &mut Cursor::new(b"new"), 3, &sha256(b"new"))
        .unwrap();
    assert_eq!(read(&s), b"new");
}

#[test]
fn gc_preserves_open_old_version_and_dirty_journal_then_reclaims_released_data() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    setup(&mut s, "", "v1");
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    let mut opened = s.open_cached("a").unwrap();
    setup(&mut s, "v1", "v2");
    s.publish_download(&ticket("v2"), b"new", &sha256(b"new"))
        .unwrap();
    assert_eq!(s.collect_garbage().unwrap(), 0);
    let mut old = Vec::new();
    opened.read_to_end(&mut old).unwrap();
    assert_eq!(old, b"old");
    drop(opened);
    assert_eq!(s.collect_garbage().unwrap(), 3);
    s.save_local("a", b"offline edit").unwrap();
    assert!(s.release("a").is_err());
    assert_eq!(s.collect_garbage().unwrap(), 0);
    assert_eq!(read(&s), b"offline edit");
}

#[test]
fn releasing_a_clean_cache_reclaims_physical_bytes_without_remote_delete() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    setup(&mut s, "", "v1");
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    s.release("a").unwrap();
    assert_eq!(s.collect_garbage().unwrap(), 3);
    assert_eq!(s.pending_operations().unwrap(), 0);
    assert_eq!(s.status("a").unwrap(), Status::OnlineOnly);
}

#[test]
fn migration_backs_up_v1_and_rejects_future_schema() {
    let d = tempfile::tempdir().unwrap();
    drop(Store::open(d.path()).unwrap());
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.execute("DROP TABLE transfers", []).unwrap();
    db.pragma_update(None, "user_version", 1).unwrap();
    drop(db);
    drop(Store::open(d.path()).unwrap());
    assert!(std::fs::read_dir(d.path()).unwrap().any(|e| {
        e.unwrap()
            .file_name()
            .to_string_lossy()
            .starts_with(&format!(
                "state-before-v{}-",
                drive_core::STATE_SCHEMA_VERSION
            ))
    }));
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    db.pragma_update(None, "user_version", 99).unwrap();
    drop(db);
    assert!(Store::open(d.path()).is_err());
}

#[test]
fn restore_identity_and_duplicate_transfer_intent_survive_restart() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    s.bind_native_server("server-restore-1").unwrap();
    setup(&mut s, "", "v1");
    let a = s.queue_download(&ticket("v1")).unwrap();
    assert_eq!(a, s.queue_download(&ticket("v1")).unwrap());
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert_eq!(a, s.queue_download(&ticket("v1")).unwrap());
    s.bind_native_server("server-restore-1").unwrap();
    assert!(s.bind_native_server("server-restore-2").is_err());
    assert_eq!(s.cursor().unwrap(), "v1");
}

#[test]
fn only_one_state_writer_can_open_a_task() {
    let d = tempfile::tempdir().unwrap();
    let first = Store::open(d.path()).unwrap();
    assert!(Store::open(d.path()).is_err());
    drop(first);
    Store::open(d.path()).unwrap();
}

#[test]
fn restart_preserves_an_inflight_upload_as_uncertain() {
    let d = tempfile::tempdir().unwrap();
    let mut s = Store::open(d.path()).unwrap();
    setup(&mut s, "", "v1");
    s.publish_download(&ticket("v1"), b"old", &sha256(b"old"))
        .unwrap();
    let op = s.save_local("a", b"edit").unwrap();
    s.mark_upload_started(&op).unwrap();
    drop(s);
    let mut s = Store::open(d.path()).unwrap();
    assert!(s.mark_upload_started(&op).is_err());
    assert_eq!(read(&s), b"edit");
    let db = rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
    let state: String = db
        .query_row("SELECT state FROM operations WHERE id=?1", [op], |r| {
            r.get(0)
        })
        .unwrap();
    assert_eq!(state, "uncertain");
}

#[test]
#[cfg(unix)]
fn closed_writer_unlocks_even_while_a_forked_helper_inherits_the_descriptor() {
    let d = tempfile::tempdir().unwrap();
    let s = Store::open(d.path()).unwrap();
    let mut pipe = [0; 2];
    assert_eq!(unsafe { libc::pipe(pipe.as_mut_ptr()) }, 0);
    // Only async-signal-safe libc calls occur in the child; it never touches the
    // copied SQLite connection or Rust allocator after fork in the test runner.
    let pid = unsafe { libc::fork() };
    assert!(pid >= 0);
    if pid == 0 {
        unsafe {
            libc::close(pipe[1]);
            let mut b = 0u8;
            libc::read(pipe[0], (&mut b as *mut u8).cast(), 1);
            libc::_exit(0);
        }
    }
    unsafe {
        libc::close(pipe[0]);
    }
    drop(s);
    let reopened = Store::open(d.path());
    unsafe {
        let b = 1u8;
        libc::write(pipe[1], (&b as *const u8).cast(), 1);
        libc::close(pipe[1]);
        libc::waitpid(pid, std::ptr::null_mut(), 0);
    }
    assert!(
        reopened.is_ok(),
        "former writer retained a lock through the helper: {:?}",
        reopened.err()
    );
}
