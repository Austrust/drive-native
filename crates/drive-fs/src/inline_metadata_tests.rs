//! The dispatcher-thread fast path for cheap authority metadata.
use super::*;
use drive_core::{Node, NodeMetadata, SharedStore};

fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
    Node {
        id: id.into(),
        parent: parent.map(str::to_owned),
        name: id.into(),
        version: "v1".into(),
        directory,
        metadata: Some(NodeMetadata {
            size: if directory { 0 } else { 3 },
            mtime: 123,
            mode: if directory { 0o755 } else { 0o644 },
        }),
    }
}
fn shared() -> (tempfile::TempDir, Arc<SharedStore>) {
    let temp = tempfile::tempdir().unwrap();
    let mut store = Store::open(temp.path()).unwrap();
    store
        .install_remote_snapshot(
            &[
                node("root", None, true),
                node("a", Some("root"), false),
                node("d", Some("root"), true),
            ],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    (temp, Arc::new(SharedStore::new(store)))
}
fn empty_handles() -> Mutex<Handles> {
    Mutex::new(Handles {
        next: 1,
        entries: HashMap::new(),
    })
}

#[test]
fn inline_authority_lookup_and_getattr_answer_without_writes() {
    let (_temp, store) = shared();
    let handles = empty_handles();
    let revision = store.lock().unwrap().local_revision();
    let lookup = preview::try_inline_lookup(&store, 1000, 1000, 1, "a")
        .expect("authority lookup should be served inline")
        .expect("lookup");
    assert_eq!(lookup.size, 3);
    assert!(lookup.ino > 1);
    assert_eq!(lookup.perm, 0o644);
    let getattr = preview::try_inline_getattr(&store, &handles, 1000, 1000, lookup.ino, None)
        .expect("authority getattr should be served inline")
        .expect("getattr");
    assert_eq!(getattr.ino, lookup.ino);
    assert_eq!(getattr.size, 3);
    // The directory parent also projects inline.
    let dir = preview::try_inline_lookup(&store, 1000, 1000, 1, "d")
        .expect("directory lookup inline")
        .expect("directory");
    assert_eq!(dir.kind, FileType::Directory);
    assert_eq!(dir.perm, 0o755);
    // Both projections are read-only.
    assert_eq!(store.lock().unwrap().local_revision(), revision);
}

#[test]
fn inline_path_defers_when_the_store_is_busy() {
    let (_temp, store) = shared();
    let handles = empty_handles();
    let guard = store.lock().unwrap();
    assert!(preview::try_inline_lookup(&store, 1000, 1000, 1, "a").is_none());
    assert!(preview::try_inline_getattr(&store, &handles, 1000, 1000, 1, None).is_none());
    drop(guard);
    // Once free, the same request is served inline.
    assert!(preview::try_inline_getattr(&store, &handles, 1000, 1000, 1, None).is_some());
}

#[test]
fn inline_path_defers_missing_names_and_inodes_to_the_offloaded_routing() {
    let (_temp, store) = shared();
    let handles = empty_handles();
    // A missing authority child may be a preview entry, and a missing inode may
    // be a preview inode: both keep the original offloaded fallback.
    assert!(preview::try_inline_lookup(&store, 1000, 1000, 1, "missing").is_none());
    assert!(preview::try_inline_getattr(&store, &handles, 1000, 1000, 999, None).is_none());
    // A non-directory parent keeps its authority error inline, exactly as the
    // offloaded path would report it.
    let file = preview::try_inline_lookup(&store, 1000, 1000, 1, "a")
        .unwrap()
        .unwrap();
    let result = preview::try_inline_lookup(&store, 1000, 1000, file.ino, "child")
        .expect("authority ENOTDIR is answered inline");
    assert_eq!(result.unwrap_err(), libc::ENOTDIR);
}

#[test]
fn inline_getattr_reports_open_handle_state_like_the_offloaded_path() {
    let (_temp, store) = shared();
    let file = preview::try_inline_lookup(&store, 1000, 1000, 1, "a")
        .unwrap()
        .unwrap();
    let handles = empty_handles();
    {
        let mut h = handles.lock().unwrap();
        h.next = 7;
        h.entries.insert(
            7,
            Handle {
                ino: file.ino,
                node: "a".into(),
                flags: libc::O_RDONLY,
                content: OpenContent::Missing(node("a", Some("root"), false)),
            },
        );
    }
    // An unknown/cross-inode handle reports EBADF inline.
    assert_eq!(
        preview::try_inline_getattr(&store, &handles, 1000, 1000, file.ino, Some(9))
            .unwrap()
            .unwrap_err(),
        libc::EBADF
    );
    // A matching missing-content handle replies the projected attributes.
    let attr = preview::try_inline_getattr(&store, &handles, 1000, 1000, file.ino, Some(7))
        .unwrap()
        .unwrap();
    assert_eq!(attr.size, 3);
}
