use drive_core::{Node,NodeMetadata,Store};
fn node(id:&str,parent:Option<&str>,directory:bool)->Node {
 Node{id:id.into(),parent:parent.map(str::to_owned),name:id.into(),version:"v1".into(),directory,
 metadata:Some(NodeMetadata{size:0,mtime:1,mode:0o644})}
}
#[test]
fn repeated_metadata_queries_do_not_write_or_advance_inode_sequence() {
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();
 s.install_remote_snapshot(&[node("root",None,true),node("a",Some("root"),false)],"1","s").unwrap();
 s.ensure_fs_root("root").unwrap();let ino=s.lookup("root","a").unwrap().ino;
 let db=rusqlite::Connection::open_with_flags(d.path().join("state.sqlite"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
 let seq=||db.query_row("SELECT seq FROM sqlite_sequence WHERE name='fs_inodes'",[],|r|r.get::<_,i64>(0)).unwrap();
 let before=seq();let revision=s.local_revision();
 for _ in 0..100 {
  s.ensure_fs_root("root").unwrap();s.record_by_inode(ino).unwrap();s.lookup("root","a").unwrap();s.children("root").unwrap();
 }
 assert_eq!(before,seq());assert_eq!(revision,s.local_revision());
}
#[test]
fn desktop_namespace_navigation_never_allocates_filesystem_inodes() {
 let d=tempfile::tempdir().unwrap();let mut s=Store::open(d.path()).unwrap();
 s.install_remote_snapshot(&[node("root",None,true),node("folder",Some("root"),true),node("a",Some("folder"),false)],"1","s").unwrap();
 s.ensure_fs_root("root").unwrap();let rev=s.local_revision();
 assert_eq!(s.lookup_node("root","folder").unwrap().id,"folder");
 assert_eq!(s.child_nodes("folder").unwrap()[0].id,"a");
 assert_eq!(s.local_revision(),rev);
 let db=rusqlite::Connection::open_with_flags(d.path().join("state.sqlite"),rusqlite::OpenFlags::SQLITE_OPEN_READ_ONLY).unwrap();
 let count:i64=db.query_row("SELECT COUNT(*) FROM fs_inodes",[],|r|r.get(0)).unwrap();assert_eq!(count,1);
 assert!(s.downloads_needed().unwrap().is_empty());
}
