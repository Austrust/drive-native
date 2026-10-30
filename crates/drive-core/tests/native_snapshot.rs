#![cfg(feature = "native")]
use drive_core::{
    native::{RemoteSnapshot, Scope},
    *,
};
fn snapshot(cursor: &str) -> RemoteSnapshot {
    let scope = Scope {
        view_id: 10,
        node_id: 20,
        file_id: "100".into(),
        path: "/Lab".into(),
    };
    let nodes=[("100",None,"Lab",true),("101",Some("100"),"A.doc",false)].into_iter().map(|(id,parent,name,directory)|Node{
        id:id.into(),parent:parent.map(str::to_owned),name:name.into(),directory,metadata:Some(NodeMetadata{size:3,mtime:5,mode:if directory{0o755}else{0o644}}),
        version:serde_json::json!({"schema":1,"restore":"server","view":10,"root":"100","file":id,"sync":7,"max":7,"size":3,"hash":"hash","directory":directory}).to_string()
    }).collect();
    RemoteSnapshot {
        inaccessible: vec![], skipped_symlinks: 0,
        scope,
        restore: "server".into(),
        cursor: cursor.into(),
        nodes,
    }
}
#[test]
fn checked_snapshot_rejects_stale_scope_restore_and_corrupted_version_without_changes() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let initial = snapshot("native-v1:7");
    s.bind_native_scope("server", &serde_json::to_string(&initial.scope).unwrap())
        .unwrap();
    s.install_native_snapshot(&initial, "").unwrap();
    assert!(
        s.install_native_snapshot(&snapshot("native-v1:9"), "")
            .is_err()
    );
    for case in 0..6 {
        let mut bad = snapshot("native-v1:9");
        match case {
            0 => bad.scope.path = "/Elsewhere".into(),
            1 => bad.restore = "other".into(),
            2 => bad.cursor = "invalid".into(),
            3 => bad.nodes[1].version = bad.nodes[0].version.clone(),
            4 => bad.nodes[1].parent = Some("missing".into()),
            _ => bad.nodes.clear(),
        }
        assert!(s.install_native_snapshot(&bad, "native-v1:7").is_err());
        assert_eq!(s.cursor().unwrap(), "native-v1:7");
        assert_eq!(s.node("101").unwrap().name, "A.doc");
    }
}
#[test]
fn local_changes_after_enumeration_survive_install_and_restart() {
    let dir = tempfile::tempdir().unwrap();
    let mut s = Store::open(dir.path()).unwrap();
    let candidate = snapshot("native-v1:7");
    s.bind_native_scope("server", &serde_json::to_string(&candidate.scope).unwrap())
        .unwrap();
    s.install_native_snapshot(&candidate, "").unwrap();
    s.ensure_fs_root("100").unwrap();
    let copy = s.create_local_file("100", "Local.doc", 0o644).unwrap();
    s.save_local(&copy.id, b"new").unwrap();
    s.creation_saved(&copy.id).unwrap();
    s.save_local("101", b"edited after enumeration").unwrap();
    s.pin("100", true).unwrap();
    s.install_native_snapshot(&snapshot("native-v1:8"), "native-v1:7")
        .unwrap();
    assert!(
        s.snapshot_invalidations()
            .unwrap()
            .iter()
            .any(|(ino, _, _)| *ino == Some(1))
    );
    drop(s);
    let s = Store::open(dir.path()).unwrap();
    use std::io::Read;
    let mut bytes = Vec::new();
    s.open_cached("101")
        .unwrap()
        .read_to_end(&mut bytes)
        .unwrap();
    assert_eq!(bytes, b"edited after enumeration");
    assert_eq!(s.node(&copy.id).unwrap().name, "Local.doc");
    assert!(s.is_pinned(&copy.id).unwrap());
    assert_eq!(s.cursor().unwrap(), "native-v1:8");
}
