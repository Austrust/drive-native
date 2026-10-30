use drive_core::{Node,NodeMetadata,Store,DownloadTicket,sha256};
use drive_runtime::{desktop,service::{Profile,Service}};
use std::{path::PathBuf,time::Instant,io::Read};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let dir=tempfile::tempdir()?;let mut store=Store::open(dir.path())?;
 let count=std::env::args().nth(1).unwrap_or("2000".into()).parse::<usize>()?;
 let mut nodes=vec![Node{id:"root".into(),parent:None,name:"root".into(),version:"1".into(),directory:true,
 metadata:Some(NodeMetadata{size:0,mtime:1,mode:0o755})}];
 for i in 0..count {nodes.push(Node{id:format!("f{i}"),parent:Some("root".into()),name:format!("f{i}"),
 version:"1".into(),directory:false,metadata:Some(NodeMetadata{size:8192,mtime:1,mode:0o644})});}
 store.install_remote_snapshot(&nodes,"1","server")?;store.ensure_fs_root("root")?;
 let start=Instant::now();store.children("root")?;let first=start.elapsed();
 let start=Instant::now();for i in 0..count {store.lookup("root",&format!("f{i}"))?;}let lookup=start.elapsed();
 let bytes=vec![7;2*1024*1024];let digest=sha256(&bytes);let start=Instant::now();
 store.publish_download(&DownloadTicket{id:"f0".into(),version:"1".into()},&bytes,&digest)?;
 let verify=start.elapsed();
 let start=Instant::now();for _ in 0..100 {let mut b=Vec::new();store.open_cached("f0")?.read_to_end(&mut b)?;assert_eq!(b,bytes);}let cached=start.elapsed();
 let p=Profile{schema:1,task_id:uuid::Uuid::new_v4().to_string(),host:"example.invalid".into(),tls_name:"example.invalid".into(),
 trusted_ca_pem:vec![],username:"benchmark".into(),device_id:uuid::Uuid::new_v4().to_string(),
 scope:drive_core::native::Scope{view_id:1,node_id:1,file_id:"root".into(),path:"/".into()},mount:PathBuf::from("/tmp/metadata-bench"),writable:false};
 let svc=Service::new(p,store,PathBuf::from("/nonexistent-worker"))?;
 let start=Instant::now();desktop::states(&svc,vec![".".into()])?;let status=start.elapsed();
 let mount=tempfile::tempdir()?;
 let fs=drive_fs::CacheFilesystem::new(svc.bridge.store.clone(),"root",unsafe{libc::geteuid()},unsafe{libc::getegid()})?;
 let session=fuser::spawn_mount2(fs,mount.path(),&[fuser::MountOption::FSName("drive-native-bench".into())])?;
 let start=Instant::now();let entries=std::fs::read_dir(mount.path())?.collect::<std::io::Result<Vec<_>>>()?;assert_eq!(entries.len(),count);
 let fuse_list=start.elapsed();
 let start=Instant::now();assert_eq!(std::fs::read(mount.path().join("f0"))?,bytes);let fuse_read=start.elapsed();
 let start=Instant::now();assert_eq!(std::fs::read(mount.path().join("f0"))?,bytes);let fuse_again=start.elapsed();
 drop(session);
 println!("{}",serde_json::json!({"files":count,"directory_ms":first.as_secs_f64()*1000.,"lookups_ms":lookup.as_secs_f64()*1000.,"cached_100_reads_ms":cached.as_secs_f64()*1000.,"folder_status_ms":status.as_secs_f64()*1000.,
 "verify_publish_2m_ms":verify.as_secs_f64()*1000.,"fuse_list_ms":fuse_list.as_secs_f64()*1000.,"fuse_cached_read_2m_ms":fuse_read.as_secs_f64()*1000.,"fuse_cached_reopen_2m_ms":fuse_again.as_secs_f64()*1000.}));
 Ok(())
}
