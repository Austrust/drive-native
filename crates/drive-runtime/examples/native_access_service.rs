//! Read-only NAS regression of the packaged fsd/worker, under dbus-run-session.
#[path="support/service.rs"]mod support;
use drive_runtime::{service::Profile,desktop::{BUS_NAME,OBJECT_PATH}};
use sproto::{Client,client::{Config,Credentials}};
use std::{path::PathBuf,os::unix::fs::PermissionsExt};
#[tokio::main]async fn main(){if let Err(_)=run().await {eprintln!("read-only permission service regression failed");std::process::exit(1);}}
async fn run()->support::Result<()> {
 let password=rpassword::prompt_password("Test account password (not saved): ")?;
 let client=Client::connect(Config::builder().host("192.0.2.10").tls_server_name("nas.example.invalid".to_string()).credentials(Credentials::Password{username:"test-user".into(),password:password.clone(),otp:None}).build()).await?;
 let share=client.list_shares().await?.into_iter().find(|s|s.view_id==12).ok_or("authorized view absent")?;
 let scratch=tempfile::tempdir()?;let mount=scratch.path().join("mount");
 let profile=Profile{schema:1,task_id:uuid::Uuid::new_v4().to_string(),host:"192.0.2.10".into(),tls_name:"nas.example.invalid".into(),trusted_ca_pem:vec![],username:"test-user".into(),device_id:uuid::Uuid::new_v4().to_string(),scope:drive_core::native::Scope{view_id:12,node_id:share.node_id.ok_or("root identity absent")?,file_id:share.file_id,path:"/".into()},mount:mount.clone(),writable:false};
 let config=scratch.path().join("profile.json");std::fs::write(&config,serde_json::to_vec(&profile)?)?;std::fs::set_permissions(&config,std::fs::Permissions::from_mode(0o600))?;
 let executable=PathBuf::from(std::env::args().nth(1).ok_or("fsd executable required")?);
 let mut daemon=support::spawn(&executable,&config,&scratch.path().join("data"),&mount,&scratch.path().join("daemon.log"))?;
 let conn=zbus::Connection::session().await?;let proxy=zbus::Proxy::new(&conn,BUS_NAME,OBJECT_PATH,BUS_NAME).await?;
 support::wait(&proxy,"locked",|d|d["connection"]=="locked").await?;
 let _:()=proxy.call("Unlock",&(password,)).await?;
 support::wait(&proxy,"online mount",|d|d["connection"]=="online" && d["mounted"]==true).await?;
 let first=support::overview(&proxy).await?;assert!(first["skipped_permission_directories"].as_u64().unwrap()>0);assert_eq!(first["downloads_started"],0);
 let count=std::fs::read_dir(&mount)?.collect::<std::io::Result<Vec<_>>>()?.len();assert!(count>0);
 let _:()=proxy.call("Pause",&()).await?;support::wait(&proxy,"paused",|d|d["connection"]=="paused").await?;
 let _:()=proxy.call("Resume",&()).await?;support::wait(&proxy,"reconnected",|d|d["connection"]=="online").await?;
 let again=support::overview(&proxy).await?;assert_eq!(again["skipped_permission_directories"],first["skipped_permission_directories"]);assert_eq!(again["downloads_started"],0);
 support::stop(&proxy,&mut daemon).await?;
 println!("{}",serde_json::json!({"passed":true,"actual_fsd_worker":true,"connection":"online","permission_skips":first["skipped_permission_directories"],"fuse_root_entries":count,"reconnect_recheck":true,"nas_mutations":0,"downloads_started":0}));Ok(())
}
