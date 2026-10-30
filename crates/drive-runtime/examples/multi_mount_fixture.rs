//! Isolated offline fixture for actual multi-process FUSE/D-Bus regression.
use drive_core::{Node,NodeMetadata,Store,DownloadTicket,sha256};
use drive_runtime::service::{Profile,prepare_paths};
use std::{path::PathBuf,os::unix::fs::PermissionsExt};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let base=PathBuf::from(std::env::args().nth(1).ok_or("fixture directory required")?);
 let mut records=Vec::new();
 for (slot,name,content) in [("","Legacy","legacy"),("aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa","科研 文档","alpha"),("bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb","团队资料","beta")] {
 let p=Profile{schema:1,task_id:uuid::Uuid::new_v4().to_string(),host:"example.invalid".into(),tls_name:"example.invalid".into(),trusted_ca_pem:vec![],username:"fixture".into(),device_id:uuid::Uuid::new_v4().to_string(),scope:drive_core::native::Scope{view_id:1,node_id:1,file_id:"root".into(),path:"/".into()},mount:base.join("mounts").join(name),writable:false};
 let state=prepare_paths(&p,&base.join("data"))?;let mut store=Store::open(&state)?;
 store.install_remote_snapshot(&[Node{id:"root".into(),parent:None,name:"root".into(),version:"1".into(),directory:true,metadata:Some(NodeMetadata{size:0,mtime:1,mode:0o755})},Node{id:"file".into(),parent:Some("root".into()),name:"A.txt".into(),version:"1".into(),directory:false,metadata:Some(NodeMetadata{size:content.len() as u64,mtime:1,mode:0o644})}],"1","fixture")?;
 store.ensure_fs_root("root")?;store.publish_download(&DownloadTicket{id:"file".into(),version:"1".into()},content.as_bytes(),&sha256(content.as_bytes()))?;
 let config=base.join("config/drive-native");std::fs::create_dir_all(config.join("profiles"))?;
 std::fs::set_permissions(&config,std::fs::Permissions::from_mode(0o700))?;std::fs::set_permissions(config.join("profiles"),std::fs::Permissions::from_mode(0o700))?;
 let path=if slot.is_empty(){config.join("profile.json")}else{config.join("profiles").join(format!("{slot}.json"))};
 std::fs::write(&path,serde_json::to_vec(&p)?)?;std::fs::set_permissions(&path,std::fs::Permissions::from_mode(0o600))?;
 records.push(serde_json::json!({"slot":slot,"profile":path,"mount":p.mount,"content":content,"task_id":p.task_id}));
 }
 println!("{}",serde_json::to_string(&records)?);Ok(())
}
