use drive_core::{Store, Node, NodeMetadata, DownloadTicket, sync_history::{HistoryQuery, SyncHistoryRecord}, write_permissions as permissions};

fn node(id: &str, name: &str, version: &str) -> Node {
    Node { id:id.into(), parent:Some("root".into()), name:name.into(), version:version.into(), directory:false,
        metadata:Some(NodeMetadata{size:3,mtime:1,mode:0o644}) }
}
fn fixture() -> (tempfile::TempDir, Store) {
    let dir=tempfile::tempdir().unwrap(); let mut store=Store::open(dir.path()).unwrap();
    let mut root=node("root","root","r1");root.parent=None;root.directory=true;root.metadata.as_mut().unwrap().mode=0o755;
    store.install_remote_snapshot(&[root,node("10","tmp","s1"),node("20","A.doc","t1")],"1","synthetic").unwrap();
    store.ensure_fs_root("root").unwrap();
    for (id,version,bytes) in [("10","s1",b"new"),("20","t1",b"old")] {
        store.publish_download(&DownloadTicket{id:id.into(),version:version.into()},bytes,&drive_core::sha256(bytes)).unwrap();
    }
    (dir,store)
}
fn history(store:&Store, node_id:&str, action:&str) -> SyncHistoryRecord {
    store.sync_history(&HistoryQuery::default()).unwrap().records.into_iter().find(|r|r.node.as_deref()==Some(node_id) && r.action==action).expect("expected history")
}
fn detail(row:&SyncHistoryRecord,action:&str,code:u32) {
    assert_eq!(row.detail.as_ref().unwrap()["permission_response"],serde_json::json!({"action":action,"code":code}));
}
#[test]
fn first_invalidation_creates_and_advances_authoritative_revision() {
    let (_dir,s)=fixture();
    let before=permissions::policy(&s,"root").unwrap().write_policy_revision;
    permissions::invalidate_remote_permissions(&s,"root").unwrap();
    assert_eq!(permissions::policy(&s,"root").unwrap().write_policy_revision,before+1);
    permissions::invalidate_remote_permissions(&s,"root").unwrap();
    assert_eq!(permissions::policy(&s,"root").unwrap().write_policy_revision,before+2);
}
#[test]
fn upload_confirmed_history_clears_hint_but_retains_original_response_detail() {
    let (_dir,mut s)=fixture();let op=s.save_local("10",b"saved new body").unwrap();s.mark_upload_started(&op).unwrap();
    permissions::mutation_permission_uncertain(&s,&op,false,0x9004).unwrap();
    let prior=history(&s,"10","upload");assert_eq!(prior.error_kind.as_deref(),Some("permission_commit_uncertain"));detail(&prior,"upload",0x9004);
    permissions::reset_session(&s).unwrap();permissions::invalidate_remote_permissions(&s,"root").unwrap();
    assert_eq!(s.upload_job(&op).unwrap().state,"uncertain");assert!(s.mark_upload_started(&op).is_err());
    assert_eq!(permissions::decide(&s,"root","10",permissions::Action::WriteContent).unwrap(),permissions::Decision::Unknown);
    s.confirm_upload(&op,"s2").unwrap();let after=history(&s,"10","upload");
    assert_eq!(after.result,"ok");assert!(after.error_kind.is_none());assert!(after.error_summary.is_none());assert_eq!(after.detail,prior.detail);
    let mut bytes=Vec::new();std::io::Read::read_to_end(&mut s.open_cached("10").unwrap(),&mut bytes).unwrap();assert_eq!(bytes,b"saved new body");
}
#[test]
fn mkdir_conflict_history_replaces_hint_and_retains_original_response_detail() {
    let (_dir,mut s)=fixture();let directory=s.create_local_directory("root","new-dir",0o755).unwrap();
    let op=s.scheduled_directory().unwrap().unwrap().job.operation;s.mark_directory_started(&op).unwrap();
    permissions::mutation_permission_uncertain(&s,&op,true,0x9003).unwrap();let prior=history(&s,&directory.id,"mkdir");detail(&prior,"upload",0x9003);
    permissions::reset_session(&s).unwrap();assert_eq!(s.scheduled_directory().unwrap().unwrap().state,"uncertain");assert!(s.mark_directory_started(&op).is_err());
    s.conflict_directory(&op).unwrap();let after=history(&s,&directory.id,"mkdir");assert_eq!(after.result,"failed");assert_eq!(after.error_kind.as_deref(),Some("conflict"));assert_eq!(after.detail,prior.detail);
}
#[test]
fn mkdir_confirmed_history_clears_hint_but_retains_original_response_detail() {
    let (_dir,mut s)=fixture();let mut directory=s.create_local_directory("root","confirmed-dir",0o755).unwrap();
    let op=s.scheduled_directory().unwrap().unwrap().job.operation;s.mark_directory_started(&op).unwrap();
    permissions::mutation_permission_uncertain(&s,&op,true,0x9004).unwrap();let prior=history(&s,&directory.id,"mkdir");detail(&prior,"upload",0x9004);
    permissions::invalidate_remote_permissions(&s,"root").unwrap();assert_eq!(s.scheduled_directory().unwrap().unwrap().state,"uncertain");
    directory.version="d2".into();s.confirm_directory(&op,&directory,"300").unwrap();
    let after=history(&s,&directory.id,"mkdir");assert_eq!(after.result,"ok");assert!(after.error_kind.is_none());assert!(after.error_summary.is_none());assert_eq!(after.detail,prior.detail);
}
#[test]
fn replace_cleanup_uncertainty_never_replays_and_final_history_keeps_reply_detail() {
    let (_dir,mut s)=fixture();s.replace_local("root","tmp","root","A.doc","10","20").unwrap();
    let q=s.next_replace().unwrap().unwrap();let op=q.job.operation.clone();s.mark_replace_started(&op).unwrap();
    permissions::mutation_permission_uncertain(&s,&op,true,0x9004).unwrap();permissions::reset_session(&s).unwrap();
    assert_eq!(s.next_replace().unwrap().unwrap().state,"uncertain");assert!(s.mark_replace_started(&op).is_err());
    let mut confirmed=node("20","A.doc","t2");confirmed.metadata.as_mut().unwrap().mtime=u64::from(q.job.mtime.unwrap());
    s.confirm_replace_upload(&op,&confirmed,"20").unwrap();s.mark_replace_cleanup_started(&op).unwrap();
    permissions::mutation_permission_uncertain(&s,&op,true,0xa004).unwrap();permissions::reset_session(&s).unwrap();
    let q=s.next_replace().unwrap().unwrap();assert_eq!(q.state,"cleanup_uncertain");assert!(s.mark_replace_cleanup_started(&op).is_err());
    let prior=history(&s,&q.job.source.id,"replace");detail(&prior,"remove",0xa004);assert_eq!(prior.error_kind.as_deref(),Some("permission_commit_uncertain"));
    s.confirm_replace_cleanup(&op,q.job.cleanup_node.as_deref().unwrap()).unwrap();
    let after=history(&s,&q.job.source.id,"replace");assert_eq!(after.result,"ok");assert!(after.error_kind.is_none());assert!(after.error_summary.is_none());assert_eq!(after.detail,prior.detail);
    assert!(s.next_replace().unwrap().is_none());let mut bytes=Vec::new();std::io::Read::read_to_end(&mut s.open_cached("10").unwrap(),&mut bytes).unwrap();assert_eq!(bytes,b"new");
}
