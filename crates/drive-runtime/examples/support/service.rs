//! Shared real-service integration helpers; tests run in an isolated session bus.
use std::{
    path::{Path, PathBuf},
    process::{Child, Command, Stdio},
    time::{Duration, Instant},
};
pub type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
pub struct Daemon {
    pub child: Child,
    mount: PathBuf,
}
impl Drop for Daemon {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = Command::new("fusermount3")
            .args(["-u", "--"])
            .arg(&self.mount)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status();
    }
}
pub fn spawn(
    executable: &Path,
    config: &Path,
    data: &Path,
    mount: &Path,
    log: &Path,
) -> Result<Daemon> {
    let stderr = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log)?;
    Ok(Daemon {
        child: Command::new(executable)
            .arg("--config")
            .arg(config)
            .env("XDG_DATA_HOME", data)
            .stdin(Stdio::null())
            .stdout(Stdio::null())
            .stderr(stderr)
            .spawn()?,
        mount: mount.into(),
    })
}
pub async fn overview(p: &zbus::Proxy<'_>) -> Result<serde_json::Value> {
    let s: String = p.call("GetOverview", &()).await?;
    Ok(serde_json::from_str(&s)?)
}
pub async fn states(p: &zbus::Proxy<'_>, path: &str) -> Result<serde_json::Value> {
    let s: String = p.call("GetStates", &(vec![path.to_owned()],)).await?;
    Ok(serde_json::from_str::<serde_json::Value>(&s)?["entries"][0].clone())
}
pub async fn wait(
    p: &zbus::Proxy<'_>,
    what: &str,
    predicate: impl Fn(&serde_json::Value) -> bool,
) -> Result<()> {
    let start = Instant::now();
    loop {
        if let Ok(value) = overview(p).await {
            if predicate(&value) {
                return Ok(());
            }
            if value["connection"] == "needs_attention" {
                return Err(format!("service needs attention: {}", value["error"]).into());
            }
        }
        if start.elapsed() > Duration::from_secs(75) {
            return Err(format!("service deadline: {what}").into());
        }
        tokio::time::sleep(Duration::from_millis(75)).await;
    }
}
pub async fn wait_state(p: &zbus::Proxy<'_>, path: &str, status: &str) -> Result<()> {
    let start = Instant::now();
    loop {
        if let Ok(value) = states(p, path).await {
            if value["status"] == status {
                return Ok(());
            }
        }
        if start.elapsed() > Duration::from_secs(75) {
            return Err(format!("state deadline: {path} -> {status}").into());
        }
        tokio::time::sleep(Duration::from_millis(75)).await;
    }
}
pub async fn stop(p: &zbus::Proxy<'_>, daemon: &mut Daemon) -> Result<()> {
    let _: () = p.call("StopMount", &()).await?;
    let start = Instant::now();
    loop {
        if let Some(status) = daemon.child.try_wait()? {
            if status.success() {
                return Ok(());
            }
            return Err("daemon shutdown failed".into());
        }
        if start.elapsed() > Duration::from_secs(10) {
            return Err("daemon did not stop".into());
        }
        tokio::time::sleep(Duration::from_millis(50)).await;
    }
}
