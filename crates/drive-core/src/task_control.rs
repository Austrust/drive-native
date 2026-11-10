//! Task-local durable controls, independent of credentials and remote state.
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;
use serde::Serialize;

/// A `startup-pause-v1.json` handover intent is meaningful only while the
/// backend handover it was written for is in progress. Anything older than this
/// window is a stale artifact from a handover that never happened and must not
/// pause a later start (issue #61).
pub const STARTUP_PAUSE_INTENT_MAX_AGE_SECONDS: u64 = 60 * 60;

/// Structured decision of the startup pause policy. It is exposed additively in
/// `GetOverview.startup_pause` and explains why the new process is (or is not)
/// paused; a normal start reports all-false booleans.
#[derive(Clone, Debug, Default, Serialize)]
pub struct StartupPauseRecovery {
    pub schema: u32,
    /// A fresh one-shot intent was present and applied: the new owner keeps the
    /// maintenance pause requested by the handover.
    pub maintenance_intent: bool,
    /// A stale intent was consumed (deleted) but deliberately not applied.
    pub stale_intent_discarded: bool,
    /// `sync_paused_v1=1` was found in the store at startup.
    pub persisted_flag_found: bool,
    /// The persisted-only pause belonged to a previous process lifetime and was
    /// auto-resumed (no maintenance intent was pending).
    pub auto_resumed: bool,
    /// Effective pause state for the new process.
    pub paused: bool,
    pub at: u64,
}

fn unix_seconds() -> u64 {
    u64::try_from(crate::commit_diagnostics::unix_now()).unwrap_or(0)
}

impl Store {
    /// Read and validate the one-shot handover intent, if present. The file is
    /// not consumed here: the caller decides whether it is fresh or stale.
    fn startup_pause_intent(
        &self,
        task_id: &str,
    ) -> Result<Option<(std::path::PathBuf, u64)>> {
        use std::io::Read;
        use std::os::unix::fs::{MetadataExt, OpenOptionsExt};
        let path = self.storage_directory().join("startup-pause-v1.json");
        let file = match std::fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(&path)
        {
            Ok(file) => file,
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(None),
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
        let modified = meta
            .modified()
            .ok()
            .and_then(|time| time.duration_since(std::time::UNIX_EPOCH).ok())
            .map(|elapsed| elapsed.as_secs())
            .unwrap_or(0);
        Ok(Some((path, modified)))
    }

    fn remove_startup_pause_intent(&self, path: &std::path::Path) -> Result<()> {
        std::fs::remove_file(path)?;
        std::fs::File::open(self.storage_directory())?.sync_all()?;
        Ok(())
    }

    fn startup_pause_intent_is_stale(modified: u64, now: u64) -> bool {
        now.saturating_sub(modified) > STARTUP_PAUSE_INTENT_MAX_AGE_SECONDS
    }

    /// A GUI may publish this intent while an older FSD still owns the database.
    /// Only the new exclusive state owner commits and consumes it, before unlock.
    /// Returns whether the intent was applied. A stale intent is still consumed
    /// (one-shot semantics) but does not pause the profile.
    pub fn consume_startup_pause(&mut self, task_id: &str) -> Result<bool> {
        let now = unix_seconds();
        let Some((path, modified)) = self.startup_pause_intent(task_id)? else {
            return Ok(false);
        };
        let stale = Self::startup_pause_intent_is_stale(modified, now);
        if !stale {
            self.set_sync_paused(true)?;
        }
        self.remove_startup_pause_intent(&path)?;
        Ok(!stale)
    }

    /// Issue #61 startup policy, run once by the new exclusive state owner
    /// before unlock:
    ///
    /// 1. A fresh one-shot handover intent wins: consume it, pause this
    ///    lifetime, report `maintenance_intent`.
    /// 2. Otherwise a persisted `sync_paused_v1=1` belongs to a previous process
    ///    lifetime (stop, crash or `kill -9`); a stop is not a user pause, so it
    ///    is auto-resumed and reported (`persisted_flag_auto_resumed`) instead of
    ///    silently starting paused. An explicit pause in the *current* session is
    ///    never touched: it happens after startup.
    /// 3. A stale intent is consumed but never applied, so it cannot mask the
    ///    persisted flag or an explicit user Resume.
    pub fn reconcile_startup_pause(&mut self, task_id: &str) -> Result<StartupPauseRecovery> {
        let now = unix_seconds();
        let persisted_flag_found = self.sync_paused()?;
        let mut outcome = StartupPauseRecovery {
            schema: 1,
            persisted_flag_found,
            at: now,
            ..StartupPauseRecovery::default()
        };
        if let Some((path, modified)) = self.startup_pause_intent(task_id)? {
            if !Self::startup_pause_intent_is_stale(modified, now) {
                self.set_sync_paused(true)?;
                self.remove_startup_pause_intent(&path)?;
                outcome.maintenance_intent = true;
                outcome.paused = true;
                return Ok(outcome);
            }
            self.remove_startup_pause_intent(&path)?;
            outcome.stale_intent_discarded = true;
        }
        if persisted_flag_found {
            self.set_sync_paused(false)?;
            outcome.auto_resumed = true;
        }
        Ok(outcome)
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
        // The child must start a test binary and open SQLite before it can
        // reject the FIFO intent; on a loaded shared build host that can take
        // longer than 5s. A real regression blocks forever (or far past 30s),
        // so a wider budget keeps the assertion meaningful.
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(30);
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
    /// Issue #61: a one-shot intent that was never picked up by a handover (the
    /// old backend kept running) is an artifact. It must be consumed exactly
    /// once, but it must not pause a profile whose last durable state was
    /// "running", otherwise a stale file silently masks an explicit user Resume.
    #[test]
    fn stale_startup_intent_is_consumed_without_pausing() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let path = intent(d.path(), "test-task");
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(
                std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 60 * 60),
            )
            .unwrap();
        store.consume_startup_pause("test-task").unwrap();
        assert!(
            !store.sync_paused().unwrap(),
            "a stale one-shot intent must not silently pause the profile"
        );
        assert!(!path.exists(), "the stale intent must still be consumed");
    }

    /// Issue #61 policy: a pause persisted by a previous process lifetime is
    /// auto-resumed at startup (a stop is not a user pause) and reported.
    #[test]
    fn persisted_pause_from_a_previous_lifetime_is_auto_resumed() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        store.set_sync_paused(true).unwrap();
        let outcome = store.reconcile_startup_pause("test-task").unwrap();
        assert!(outcome.persisted_flag_found);
        assert!(outcome.auto_resumed);
        assert!(!outcome.paused);
        assert!(!outcome.maintenance_intent);
        assert!(!store.sync_paused().unwrap());
    }

    /// A fresh handover intent still pauses the new owner exactly once; a later
    /// startup (no intent) auto-resumes it because it belongs to the previous
    /// process lifetime.
    #[test]
    fn fresh_handover_intent_pauses_once_then_next_start_resumes() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        store.set_sync_paused(true).unwrap();
        let path = intent(d.path(), "test-task");
        let outcome = store.reconcile_startup_pause("test-task").unwrap();
        assert!(outcome.maintenance_intent);
        assert!(outcome.paused);
        assert!(!outcome.stale_intent_discarded);
        assert!(store.sync_paused().unwrap());
        assert!(!path.exists());
        let next = store.reconcile_startup_pause("test-task").unwrap();
        assert!(!next.paused);
        assert!(next.auto_resumed);
    }

    /// A stale intent is consumed once but never applied, so it cannot mask the
    /// persisted-flag policy.
    #[test]
    fn stale_intent_is_discarded_and_does_not_mask_persisted_pause() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        store.set_sync_paused(true).unwrap();
        let path = intent(d.path(), "test-task");
        std::fs::OpenOptions::new()
            .write(true)
            .open(&path)
            .unwrap()
            .set_modified(
                std::time::SystemTime::now() - std::time::Duration::from_secs(2 * 60 * 60),
            )
            .unwrap();
        let outcome = store.reconcile_startup_pause("test-task").unwrap();
        assert!(outcome.stale_intent_discarded);
        assert!(outcome.auto_resumed);
        assert!(!outcome.paused);
        assert!(!path.exists());
    }

    /// Nothing persisted: startup keeps the profile running and reports no event.
    #[test]
    fn clean_startup_reports_no_pause_event() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let outcome = store.reconcile_startup_pause("test-task").unwrap();
        assert!(!outcome.paused);
        assert!(!outcome.persisted_flag_found);
        assert!(!outcome.auto_resumed);
        assert!(!outcome.maintenance_intent);
        assert!(!outcome.stale_intent_discarded);
        assert!(!store.sync_paused().unwrap());
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
