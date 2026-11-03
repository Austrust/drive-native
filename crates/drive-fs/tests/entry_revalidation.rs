//! New path-based opens must not use a moved inode through an old cached name.
//! No notification is sent: zero entry TTL must independently revalidate names.
use drive_core::{Node,NodeMetadata,Store,SharedStore,DownloadTicket};
use drive_fs::{CacheFilesystem,publication::NotifyHub};
use std::{fs, path::PathBuf, sync::Arc, time::{Duration,Instant}};

fn node(id:&str,parent:Option<&str>,directory:bool)->Node {
    Node{id:id.into(),parent:parent.map(str::to_owned),name:id.into(),directory,
        version:"v1".into(),metadata:Some(NodeMetadata{size:if directory{0}else{3},mtime:1,mode:if directory{0o755}else{0o644}})}
}
struct Fixture { _temp:tempfile::TempDir, mount:PathBuf, store:Arc<SharedStore>, session:Option<fuser::BackgroundSession> }
impl Fixture {
    fn new()->Self {
        assert_ne!(unsafe{libc::getuid()},0,"ordinary user required");
        let temp=tempfile::tempdir().unwrap();let mount=temp.path().join("mount");fs::create_dir(&mount).unwrap();
        let mut s=Store::open(temp.path().join("state")).unwrap();
        s.install_remote_snapshot(&[node("root",None,true),node("other",Some("root"),true),node("a",Some("root"),false)],"1","server").unwrap();
        s.publish_download(&DownloadTicket{id:"a".into(),version:"v1".into()},b"old",&drive_core::sha256(b"old")).unwrap();
        let store=Arc::new(SharedStore::new(s));let hub=NotifyHub::with_ttl_ms(30_000);
        let filesystem=CacheFilesystem::new(store.clone(),"root",unsafe{libc::getuid()},unsafe{libc::getgid()}).unwrap().with_writes_enabled().with_notify_hub(hub.clone());
        let session=fuser::Session::new(filesystem,&mount,&[fuser::MountOption::RW,fuser::MountOption::DefaultPermissions,fuser::MountOption::NoDev,fuser::MountOption::NoSuid]).unwrap();
        hub.install(session.notifier());
        Self{_temp:temp,mount,store,session:Some(fuser::BackgroundSession::new(session).unwrap())}
    }
}
impl Drop for Fixture { fn drop(&mut self) {if let Some(s)=self.session.take(){let _=s.join_result();}} }
fn old_name_cannot_truncate_moved_object(target:&str) {
    let f=Fixture::new();let old=f.mount.join("a");
    // Keep an existing handle: its identity survives rename, while a NEW open
    // of the old path must fail. Long configured attr TTL prevents expiry from
    // masquerading as proof that positive names were never cached.
    let held=fs::File::open(&old).unwrap();assert_eq!(fs::metadata(&old).unwrap().len(),3);
    let started=Instant::now();
    {
        let mut store = f.store.lock().unwrap();
        store.rename_local("root", "a", target, "moved", false).unwrap();
        assert!(store.lookup("root", "a").unwrap_err().is_not_found(), "old namespace must already be gone");
        assert_eq!(store.lookup(target, "moved").unwrap().node.id, "a");
        assert_eq!(store.node("a").unwrap().name, "moved");
    }
    let opened=fs::OpenOptions::new().write(true).truncate(true).open(&old);
    assert_eq!(opened.unwrap_err().raw_os_error(),Some(libc::ENOENT));
    assert!(started.elapsed()<Duration::from_secs(10),"must finish before configured 30s TTL");
    let destination=if target=="root"{f.mount.join("moved")}else{f.mount.join(target).join("moved")};
    assert_eq!(fs::read(destination).unwrap(),b"old");
    assert_eq!(held.metadata().unwrap().len(),3);
    assert!(f.store.lock().unwrap().open_dirty_draft("a").unwrap().is_none());
    drop(held);
}
#[test]
#[ignore="ordinary Linux user and /dev/fuse; explicit kernel validation"]
fn new_open_of_old_name_does_not_truncate_renamed_object(){old_name_cannot_truncate_moved_object("root");}
#[test]
#[ignore="ordinary Linux user and /dev/fuse; explicit kernel validation"]
fn new_open_of_old_path_does_not_truncate_object_moved_to_another_parent(){old_name_cannot_truncate_moved_object("other");}
