#![cfg(feature="native")]
use drive_core::{*,native::{RemoteSnapshot,Scope}};
use std::io::Read;
fn snapshot(cursor:&str,denied:bool)->RemoteSnapshot {
 let scope=Scope{view_id:12,node_id:1,file_id:"100".into(),path:"/".into()};
 let entries=vec![("100",None,"root",true),("200",Some("100"),"restricted",true),("300",Some("100"),"visible",true),("201",Some("200"),"A.txt",false),("301",Some("300"),"B.txt",false)];
 let nodes=entries.into_iter().filter(|(id,_,_,_)| !denied || *id!="201").map(|(id,parent,name,directory)|Node{id:id.into(),parent:parent.map(str::to_owned),name:name.into(),directory,metadata:Some(NodeMetadata{size:3,mtime:1,mode:if directory{0o755}else{0o644}}),version:serde_json::json!({"schema":1,"restore":"s","view":12,"root":"100","file":id,"sync":1,"max":1,"size":3,"hash":"hash","directory":directory}).to_string()}).collect();
 RemoteSnapshot{scope,restore:"s".into(),cursor:cursor.into(),nodes,skipped_symlinks:0,inaccessible:if denied{vec!["200".into()]}else{vec![]}}
}
fn install(s:&mut Store,snap:&RemoteSnapshot){let c=s.cursor().unwrap();s.bind_native_scope("s",&serde_json::to_string(&snap.scope).unwrap()).unwrap();s.install_native_snapshot(snap,&c).unwrap();s.ensure_fs_root("100").unwrap();}
#[test]fn initial_partial_tree_hides_denied_entry_but_preserves_siblings(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",true));
 assert_eq!(s.children("100").unwrap().iter().map(|r|r.node.id.as_str()).collect::<Vec<_>>(),vec!["300"]);
 assert!(s.lookup("100","restricted").is_err());assert_eq!(s.child_nodes("300").unwrap().len(),1);assert_eq!(s.inaccessible_count().unwrap(),1);
}
#[test]fn revocation_preserves_cache_draft_and_pin_across_restart_and_restoration(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",false));
 let version=s.node("201").unwrap().version;s.publish_download(&DownloadTicket{id:"201".into(),version},b"old",&sha256(b"old")).unwrap();s.save_local("201",b"private edit").unwrap();s.pin("200",true).unwrap();
 let before=s.pending_operations().unwrap();install(&mut s,&snapshot("native-v1:2",true));
 assert!(s.open_cached("201").is_err());assert!(s.node("201").is_err());assert!(s.upload_operations("pending").unwrap().is_empty());assert!(s.downloads_needed().unwrap().is_empty());assert_eq!(s.pending_operations().unwrap(),before);s.collect_garbage().unwrap();
 drop(s);let mut s=Store::open(d.path()).unwrap();assert!(s.node("200").is_err());
 // A later filtered listing can omit the denied identity entirely: still not deletion.
 let mut absent=snapshot("native-v1:3",true);absent.nodes.retain(|n|n.id!="200");absent.inaccessible.clear();install(&mut s,&absent);assert_eq!(s.inaccessible_count().unwrap(),1);
 install(&mut s,&snapshot("native-v1:4",false));assert_eq!(s.inaccessible_count().unwrap(),0);let mut bytes=Vec::new();s.open_cached("201").unwrap().read_to_end(&mut bytes).unwrap();assert_eq!(bytes,b"private edit");assert!(s.is_pinned("201").unwrap());assert_eq!(s.upload_operations("pending").unwrap().len(),1);
}
#[test]fn blocked_namespace_job_does_not_stop_accessible_sibling(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",false));
 s.rename_local("200","A.txt","300","renamed.txt",false).unwrap();s.create_local_directory("300","new",0o755).unwrap();
 install(&mut s,&snapshot("native-v1:2",true));assert_eq!(s.next_namespace_kind().unwrap().as_deref(),Some("mkdir"));assert!(s.next_rename().unwrap().is_none());assert!(s.next_directory().unwrap().is_some());assert!(s.node("201").is_err());
 install(&mut s,&snapshot("native-v1:3",false));assert_eq!(s.next_namespace_kind().unwrap().as_deref(),Some("rename"));
}
#[test]fn malformed_denial_cannot_quarantine_root_or_nonexistent_identity(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",false));
 for id in ["100","999","301"] {let mut bad=snapshot("native-v1:2",false);bad.inaccessible=vec![id.into()];assert!(s.install_native_snapshot(&bad,"native-v1:1").is_err());assert_eq!(s.inaccessible_count().unwrap(),0);assert_eq!(s.cursor().unwrap(),"native-v1:1");}
}

#[test]fn schema17_upgrade_backs_up_and_preserves_pending_content(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",false));s.save_local("201",b"pending").unwrap();drop(s);
 let db=rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();db.execute_batch("DROP VIEW access_blocked_operations;DROP VIEW access_blocked;DROP TABLE access_denied;PRAGMA user_version=17;").unwrap();drop(db);
 let mut s=Store::open(d.path()).unwrap();assert_eq!(s.inaccessible_count().unwrap(),0);install(&mut s,&snapshot("native-v1:2",true));drop(s);
 let backup=std::fs::read_dir(d.path()).unwrap().map(|e|e.unwrap().path()).find(|p|p.file_name().unwrap().to_string_lossy().starts_with("state-before-v18-")).unwrap();let db=rusqlite::Connection::open(backup).unwrap();assert_eq!(db.query_row("PRAGMA user_version",[],|r|r.get::<_,i64>(0)).unwrap(),17);assert_eq!(db.query_row("PRAGMA integrity_check",[],|r|r.get::<_,String>(0)).unwrap(),"ok");
 let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:3",false));let mut bytes=Vec::new();s.open_cached("201").unwrap().read_to_end(&mut bytes).unwrap();assert_eq!(bytes,b"pending");
}

#[test]fn filtered_link_replacement_retains_unsynced_content_and_count_after_restart(){
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();install(&mut s,&snapshot("native-v1:1",false));
 s.save_local("301",b"unsynced draft").unwrap();
 let mut next=snapshot("native-v1:2",false);next.nodes.retain(|n|n.id!="301");next.skipped_symlinks=1;install(&mut s,&next);
 assert_eq!(s.skipped_symlink_count().unwrap(),1);assert!(s.upload_operations("pending").unwrap().is_empty());
 s.collect_garbage().unwrap();drop(s);
 let s=Store::open(d.path()).unwrap();assert_eq!(s.skipped_symlink_count().unwrap(),1);
 let db=rusqlite::Connection::open(d.path().join("state.sqlite")).unwrap();
 let (dirty,conflict):(Option<String>,bool)=db.query_row("SELECT dirty_blob,conflict FROM nodes WHERE id='301'",[],|r|Ok((r.get(0)?,r.get(1)?))).unwrap();
 assert!(dirty.is_some());assert!(conflict);
}
