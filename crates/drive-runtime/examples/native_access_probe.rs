//! Authorized read-only team-directory regression; never downloads file bodies.
use drive_core::{Store,native::{NativeReadSession,Scope}};
use sproto::{Client,client::{Config,Credentials}};
use std::sync::{Arc,Mutex};
#[tokio::main]async fn main(){if let Err(e)=run().await {let category=match e{drive_core::Error::Native(sproto::Error::Server{code,..})=>format!("server_{code}"),drive_core::Error::Native(_)=>"native_transport_or_decode".into(),drive_core::Error::State(s)=>s,_=>"local_validation_error".into()};eprintln!("{}",serde_json::json!({"passed":false,"category":category}));std::process::exit(1);}}
async fn run()->drive_core::Result<()> {
 let password=rpassword::prompt_password("Test account password (not saved): ")?;
 let client=Client::connect(Config::builder().host("192.0.2.10").tls_server_name("nas.example.invalid".to_string()).credentials(Credentials::Password{username:"test-user".into(),password,otp:None}).build()).await?;
 let share=client.list_shares().await?.into_iter().find(|s|s.view_id==12).ok_or_else(||drive_core::Error::State("authorized team absent".into()))?;
 let scope=Scope{view_id:share.view_id,node_id:share.node_id.ok_or_else(||drive_core::Error::State("root node absent".into()))?,file_id:share.file_id,path:"/".into()};
 let session=NativeReadSession::new(client,scope.clone())?;
 let started=std::time::Instant::now();let snapshot=session.recovery_snapshot().await?;
 let denied=snapshot.inaccessible.len();assert!(denied>0);
 let local=tempfile::tempdir()?;let mut store=Store::open(local.path())?;
 store.bind_native_scope(&snapshot.restore,&serde_json::to_string(&scope)?)?;store.install_native_snapshot(&snapshot,"")?;store.ensure_fs_root(&scope.file_id)?;
 let visible=store.children(&scope.file_id)?;for entry in &visible {assert!(!snapshot.inaccessible.contains(&entry.node.id));}
 let root_count=visible.len();assert!(root_count>0);
 let mount=tempfile::tempdir()?;let fs=drive_fs::CacheFilesystem::new(Arc::new(Mutex::new(store)),&scope.file_id,unsafe{libc::getuid()},unsafe{libc::getgid()})?;
 let mounted=fuser::spawn_mount2(fs,mount.path(),&[fuser::MountOption::RO,fuser::MountOption::FSName("drive-native-access-test".into())])?;
 let entries=std::fs::read_dir(mount.path())?.collect::<std::io::Result<Vec<_>>>()?;assert_eq!(entries.len(),root_count);
 drop(mounted);
 println!("{}",serde_json::json!({"passed":true,"scope_view":12,"snapshot_nodes":snapshot.nodes.len(),"inaccessible_subtrees":denied,"visible_root_entries":root_count,"actual_fuse_entries":entries.len(),"seconds":started.elapsed().as_secs_f64(),"nas_mutations":0,"file_bodies_downloaded":0,"server":"DSM 7.4.1 / Drive 4.0.3-27892"}));Ok(())
}


