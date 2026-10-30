use drive_core::*;
use std::io::Read;

#[test]
fn schema14_upgrade_keeps_pending_bytes_pin_and_coherent_backup() {
    let directory=tempfile::tempdir().unwrap();
    let mut store=Store::open(directory.path()).unwrap();
    let root=Node{id:"root".into(),parent:None,name:"root".into(),version:"1".into(),directory:true,metadata:None};
    let file=Node{id:"file".into(),parent:Some("root".into()),name:"A.doc".into(),version:"v1".into(),directory:false,
        metadata:Some(NodeMetadata{size:3,mtime:1,mode:0o600})};
    store.install_remote_snapshot(&[root,file],"cursor1","server1").unwrap();
    store.publish_download(&DownloadTicket{id:"file".into(),version:"v1".into()},b"old",&sha256(b"old")).unwrap();
    store.pin("file",true).unwrap();
    let operation=store.save_local("file",b"offline changes").unwrap();
    drop(store);
    let db=rusqlite::Connection::open(directory.path().join("state.sqlite")).unwrap();
    db.execute_batch("DROP TABLE engine_upload_stages;DROP TABLE engine_bridge_tombstones;DROP TABLE engine_bridge_intents;DROP TABLE cache_releases;PRAGMA user_version=14;").unwrap();
    drop(db);
    let mut store=Store::open(directory.path()).unwrap();
    assert!(store.is_pinned("file").unwrap());
    assert_eq!(store.upload_job(&operation).unwrap().state,"pending");
    let mut bytes=Vec::new();store.open_cached("file").unwrap().read_to_end(&mut bytes).unwrap();
    assert_eq!(bytes,b"offline changes");
    let backup=std::fs::read_dir(directory.path()).unwrap().map(|v|v.unwrap().path())
        .find(|p|p.file_name().unwrap().to_string_lossy().starts_with(&format!("state-before-v{}-",STATE_SCHEMA_VERSION))).unwrap();
    let backup=rusqlite::Connection::open(backup).unwrap();
    assert_eq!(backup.query_row("PRAGMA user_version",[],|r|r.get::<_,i64>(0)).unwrap(),14);
    assert_eq!(backup.query_row("PRAGMA integrity_check",[],|r|r.get::<_,String>(0)).unwrap(),"ok");
    assert_eq!(store.engine_intent_counts().unwrap(),(0,0));
    assert!(store.engine_upload_receipts().unwrap().is_empty());
    // A newer database is rejected rather than silently downgraded.
    drop(store);
    let db=rusqlite::Connection::open(directory.path().join("state.sqlite")).unwrap();
    db.pragma_update(None,"user_version",STATE_SCHEMA_VERSION+1).unwrap();drop(db);
    assert!(Store::open(directory.path()).is_err());
}
