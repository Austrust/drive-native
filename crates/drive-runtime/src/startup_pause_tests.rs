//! Issue #61: startup handling of the persisted `sync_paused_v1` flag.
//!
//! A pause is a session control, not a durable user preference: a stop, crash
//! or `kill -9` while paused must not make the next process start silently
//! paused. The one-shot `startup-pause-v1.json` handover intent stays one-shot
//! and is the only way a new process starts paused. Every case reports through
//! the additive `GetOverview.paused` / `paused_by` / `startup_pause` fields.
use super::*;
use drive_core::{Node, NodeMetadata};
use std::os::unix::fs::PermissionsExt;

const CA: &str = include_str!("../../sproto/tests/fixtures/trust/ca.pem");
const STARTUP_PAUSE_INTENT: &str = "startup-pause-v1.json";

fn profile(task_id: &str) -> Profile {
    Profile {
        schema: 1,
        task_id: task_id.into(),
        host: "unused.invalid".into(),
        tls_name: "unused.invalid".into(),
        trusted_ca_pem: vec![CA.into()],
        username: "synthetic".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/test".into(),
        },
        mount: PathBuf::from("/tmp/startup-pause-test-unused"),
        writable: false,
        local_policy: None,
        writable_source: None,
        exclude_patterns: Vec::new(),
    }
}

fn prepared_store(dir: &std::path::Path) -> Store {
    let mut store = Store::open(dir).unwrap();
    store
        .install_remote_snapshot(
            &[Node {
                id: "root".into(),
                parent: None,
                name: "root".into(),
                version: "v1".into(),
                directory: true,
                metadata: Some(NodeMetadata {
                    size: 0,
                    mtime: 1,
                    mode: 0o755,
                }),
            }],
            "1",
            "server",
        )
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
}

/// Build the service the way the FSD does on a fresh process start. `prepare`
/// runs against the reopened store before `Service::new`, so a test can persist
/// the state a killed/stopped previous process would have left behind.
fn start_service(
    dir: &std::path::Path,
    task_id: &str,
    prepare: impl FnOnce(&mut Store),
) -> Arc<Service> {
    let mut store = prepared_store(dir);
    prepare(&mut store);
    Service::new(profile(task_id), store, PathBuf::from("/unused-worker")).unwrap()
}

fn overview_json(service: &Service) -> serde_json::Value {
    serde_json::from_str(&crate::desktop::overview(service).unwrap()).unwrap()
}

fn write_startup_intent(dir: &std::path::Path, task_id: &str) {
    let path = dir.join(STARTUP_PAUSE_INTENT);
    std::fs::write(
        &path,
        serde_json::json!({"schema":1,"task_id":task_id}).to_string(),
    )
    .unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
}

fn backdate(path: &std::path::Path, seconds: u64) {
    std::fs::OpenOptions::new()
        .write(true)
        .open(path)
        .unwrap()
        .set_modified(std::time::SystemTime::now() - std::time::Duration::from_secs(seconds))
        .unwrap();
}

fn fixture() -> (tempfile::TempDir, String) {
    let dir = tempfile::tempdir().unwrap();
    std::fs::set_permissions(dir.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let task_id = uuid::Uuid::new_v4().to_string();
    (dir, task_id)
}

/// AC 1/3: a pause persisted by a previous process lifetime (graceful stop or
/// `kill -9` after the Pause commit) is auto-resumed with a structured,
/// explicit report instead of silently pausing the new process.
#[test]
fn persisted_pause_from_a_previous_lifetime_is_auto_resumed_and_reported() {
    let (dir, task_id) = fixture();
    let service = start_service(dir.path(), &task_id, |store| {
        store.set_sync_paused(true).unwrap();
    });
    assert!(
        !service.control.lock().unwrap().paused,
        "a persisted pause from a previous lifetime must not silently pause the new process"
    );
    assert!(
        !service.bridge.store.lock().unwrap().sync_paused().unwrap(),
        "auto-resume must clear the persisted flag"
    );
    let overview = overview_json(&service);
    assert_eq!(overview["paused"], false);
    assert_eq!(overview["paused_by"], "persisted_flag_auto_resumed");
    assert_eq!(overview["startup_pause"]["auto_resumed"], true);
    assert_eq!(overview["startup_pause"]["persisted_flag_found"], true);
    assert_eq!(overview["startup_pause"]["maintenance_intent"], false);
    assert!(overview["startup_pause"]["at"].as_u64().unwrap() > 0);
    assert_ne!(overview["sync_state"], "paused");
}

/// AC 3: `kill -9` while paused leaves the durable flag set without a graceful
/// shutdown. The next start must follow the documented auto-resume policy.
#[test]
fn kill_nine_while_paused_does_not_silently_start_paused() {
    let (dir, task_id) = fixture();
    // Simulate the crash: the Pause commit is durable, nothing cleared it.
    let service = start_service(dir.path(), &task_id, |store| {
        store.set_sync_paused(true).unwrap();
    });
    let overview = overview_json(&service);
    assert_ne!(overview["sync_state"], "paused");
    assert_eq!(overview["paused_by"], "persisted_flag_auto_resumed");
    assert!(service.control.lock().unwrap().password.is_none());
}

/// AC 2: Pause -> Resume -> stop -> start has no residual paused state.
#[test]
fn resume_before_restart_leaves_no_paused_state() {
    let (dir, task_id) = fixture();
    let service = start_service(dir.path(), &task_id, |store| {
        store.set_sync_paused(true).unwrap();
        store.set_sync_paused(false).unwrap();
    });
    assert!(!service.control.lock().unwrap().paused);
    let overview = overview_json(&service);
    assert!(overview["paused_by"].is_null());
    assert_eq!(overview["startup_pause"]["auto_resumed"], false);
    assert_eq!(overview["startup_pause"]["persisted_flag_found"], false);
}

/// AC 4: a fresh startup intent still pauses the new owner exactly once, and
/// the overview explains that this is a maintenance handover (`startup_intent`),
/// not a user pause.
#[test]
fn fresh_startup_intent_pauses_with_startup_intent_origin() {
    let (dir, task_id) = fixture();
    write_startup_intent(dir.path(), &task_id);
    let service = start_service(dir.path(), &task_id, |_| {});
    assert!(
        service.control.lock().unwrap().paused,
        "a fresh one-shot intent must still pause the new owner"
    );
    assert!(!dir.path().join(STARTUP_PAUSE_INTENT).exists());
    let overview = overview_json(&service);
    assert_eq!(overview["paused"], true);
    assert_eq!(overview["paused_by"], "startup_intent");
    assert_eq!(overview["sync_state"], "paused");
    assert_eq!(overview["startup_pause"]["maintenance_intent"], true);
    assert_eq!(overview["startup_pause"]["auto_resumed"], false);
}

/// AC 4/5: a stale intent (a handover that never happened) is consumed but must
/// not re-pause a profile whose last durable state was an explicit user Resume.
#[test]
fn stale_intent_does_not_repause_an_explicitly_resumed_profile() {
    let (dir, task_id) = fixture();
    write_startup_intent(dir.path(), &task_id);
    backdate(&dir.path().join(STARTUP_PAUSE_INTENT), 2 * 60 * 60);
    let service = start_service(dir.path(), &task_id, |store| {
        store.set_sync_paused(false).unwrap();
    });
    assert!(
        !service.control.lock().unwrap().paused,
        "a stale intent must not override an explicit user Resume"
    );
    assert!(
        !dir.path().join(STARTUP_PAUSE_INTENT).exists(),
        "the stale intent must still be consumed exactly once"
    );
    let overview = overview_json(&service);
    assert!(overview["paused_by"].is_null());
    assert_eq!(overview["startup_pause"]["stale_intent_discarded"], true);
}

/// AC 5: an old intent must not mask the persisted-flag policy either: the
/// previous-lifetime pause is still auto-resumed and the stale artifact is
/// reported separately.
#[test]
fn stale_intent_does_not_mask_persisted_pause_auto_resume() {
    let (dir, task_id) = fixture();
    write_startup_intent(dir.path(), &task_id);
    backdate(&dir.path().join(STARTUP_PAUSE_INTENT), 2 * 60 * 60);
    let service = start_service(dir.path(), &task_id, |store| {
        store.set_sync_paused(true).unwrap();
    });
    assert!(!service.control.lock().unwrap().paused);
    let overview = overview_json(&service);
    assert_eq!(overview["paused_by"], "persisted_flag_auto_resumed");
    assert_eq!(overview["startup_pause"]["stale_intent_discarded"], true);
    assert_eq!(overview["startup_pause"]["auto_resumed"], true);
}

/// User pauses in the running session: the overview reports the explicit
/// `user` origin, distinct from a startup artifact.
#[test]
fn user_pause_in_this_session_is_reported_as_user() {
    let (dir, task_id) = fixture();
    let service = start_service(dir.path(), &task_id, |_| {});
    service.pause().unwrap();
    let overview = overview_json(&service);
    assert_eq!(overview["paused"], true);
    assert_eq!(overview["paused_by"], "user");
    service.resume().unwrap();
    let overview = overview_json(&service);
    assert_eq!(overview["paused"], false);
    assert!(overview["paused_by"].is_null());
}
