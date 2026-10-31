//! Task-local durable controls, independent of credentials and remote state.
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;

impl Store {
    /// A GUI may publish this intent while an older FSD still owns the database.
    /// Only the new exclusive state owner commits and consumes it, before unlock.
    pub fn consume_startup_pause(&mut self, task_id: &str) -> Result<()> {
        use std::io::Read;
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let path = self.storage_directory().join("startup-pause-v1.json");
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            Err(e) => return Err(e.into()),
        };
        let meta = file.metadata()?;
        if !meta.is_file()
            || meta.uid() != unsafe { libc::geteuid() }
            || meta.mode() & 0o077 != 0
            || meta.len() > 4096
        {
            return Err(Error::State("invalid startup pause permissions".into()));
        }
        #[derive(serde::Deserialize)]
        #[serde(deny_unknown_fields)]
        struct Intent {
            schema: u32,
            task_id: String,
        }
        let mut bytes = Vec::new();
        file.take(4097).read_to_end(&mut bytes)?;
        if bytes.len() > 4096 {
            return Err(Error::State("startup pause too large".into()));
        }
        let intent: Intent = serde_json::from_slice(&bytes)?;
        if intent.schema != 1 || intent.task_id != task_id {
            return Err(Error::State("startup pause task mismatch".into()));
        }
        self.set_sync_paused(true)?;
        std::fs::remove_file(path)?;
        std::fs::File::open(self.storage_directory())?.sync_all()?;
        Ok(())
    }

    pub fn sync_paused(&self) -> Result<bool> {
        let value: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='sync_paused_v1'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        match value.as_deref() {
            None | Some("0") => Ok(false),
            Some("1") => Ok(true),
            _ => Err(Error::State("invalid persisted pause state".into())),
        }
    }

    pub fn set_sync_paused(&mut self, paused: bool) -> Result<()> {
        // SQLite's FULL synchronous commit must succeed before acknowledging Pause.
        self.db.execute("INSERT INTO metadata(key,value) VALUES('sync_paused_v1',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value", [if paused { "1" } else { "0" }])?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::PermissionsExt;
    #[test]
    fn fifo_startup_intent_is_rejected_without_waiting_for_writer() {
        const CHILD_ROOT: &str = "DRIVE_NATIVE_PAUSE_FIFO_TEST_ROOT";
        if let Some(root) = std::env::var_os(CHILD_ROOT) {
            let mut store = Store::open(&root).unwrap();
            assert!(store.consume_startup_pause("test-task").is_err());
            assert!(!store.sync_paused().unwrap());
            assert!(
                std::path::Path::new(&root)
                    .join("startup-pause-v1.json")
                    .exists()
            );
            return;
        }
        use std::os::unix::ffi::OsStrExt;
        let root = tempfile::tempdir().unwrap();
        let path = root.path().join("startup-pause-v1.json");
        let path = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(path.as_ptr(), 0o600) }, 0);
        // A regression must fail within a bounded time, not hang the test suite.
        // Only this synthetic child is terminated if FIFO open waits for a writer.
        let mut child = std::process::Command::new(std::env::current_exe().unwrap())
            .args([
                "--exact",
                "task_control::tests::fifo_startup_intent_is_rejected_without_waiting_for_writer",
            ])
            .env(CHILD_ROOT, root.path())
            .stdout(std::process::Stdio::null())
            .stderr(std::process::Stdio::null())
            .spawn()
            .unwrap();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(5);
        loop {
            if let Some(status) = child.try_wait().unwrap() {
                assert!(status.success(), "FIFO rejection child failed");
                break;
            }
            if std::time::Instant::now() >= deadline {
                child.kill().unwrap();
                child.wait().unwrap();
                panic!("startup intent FIFO blocked without a writer");
            }
            std::thread::sleep(std::time::Duration::from_millis(10));
        }
    }
    fn intent(d: &std::path::Path, task: &str) -> std::path::PathBuf {
        let path = d.join("startup-pause-v1.json");
        std::fs::write(
            &path,
            serde_json::json!({"schema":1,"task_id":task}).to_string(),
        )
        .unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        path
    }
    #[test]
    fn old_backend_handover_intent_is_committed_before_consumption() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let path = intent(d.path(), "test-task");
        store.consume_startup_pause("test-task").unwrap();
        assert!(store.sync_paused().unwrap());
        assert!(!path.exists());
        drop(store);
        assert!(Store::open(d.path()).unwrap().sync_paused().unwrap());
    }
    #[test]
    fn unsafe_or_mismatched_intents_are_never_consumed() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let path = intent(d.path(), "different-task");
        assert!(store.consume_startup_pause("test-task").is_err());
        assert!(path.exists());
        assert!(!store.sync_paused().unwrap());
        let path = intent(d.path(), "test-task");
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
        assert!(store.consume_startup_pause("test-task").is_err());
        std::fs::remove_file(&path).unwrap();
        std::os::unix::fs::symlink(d.path().join("absent"), &path).unwrap();
        assert!(store.consume_startup_pause("test-task").is_err());
    }
    #[test]
    fn pause_survives_reopen_and_is_task_local() {
        let a = tempfile::tempdir().unwrap();
        let b = tempfile::tempdir().unwrap();
        let mut store = Store::open(a.path()).unwrap();
        assert!(!store.sync_paused().unwrap());
        store.set_sync_paused(true).unwrap();
        drop(store);
        let mut store = Store::open(a.path()).unwrap();
        assert!(store.sync_paused().unwrap());
        assert!(!Store::open(b.path()).unwrap().sync_paused().unwrap());
        store.set_sync_paused(false).unwrap();
        drop(store);
        assert!(!Store::open(a.path()).unwrap().sync_paused().unwrap());
    }
    #[test]
    fn rejected_commit_preserves_pause_and_corruption_is_not_resumed() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        store.set_sync_paused(true).unwrap();
        store.db.execute_batch("CREATE TRIGGER reject_pause BEFORE UPDATE ON metadata WHEN OLD.key='sync_paused_v1' BEGIN SELECT RAISE(FAIL,'synthetic write failure'); END;").unwrap();
        assert!(store.set_sync_paused(false).is_err());
        assert!(store.sync_paused().unwrap());
        store.db.execute_batch("DROP TRIGGER reject_pause; UPDATE metadata SET value='bad' WHERE key='sync_paused_v1';").unwrap();
        assert!(store.sync_paused().is_err());
    }
}
