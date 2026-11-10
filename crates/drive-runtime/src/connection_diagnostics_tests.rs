use super::*;
use crate::bridge::WorkerProcess;

fn failure() -> Record {
    Record::new(
        Stage::WaitNotification,
        &Err(io::Error::new(
            io::ErrorKind::TimedOut,
            Cause::NativeReceiveIdle,
        )),
        false,
        Duration::from_secs(601),
        Some(123),
        ProcessState::Running,
    )
}

#[test]
fn restart_retains_failure_and_requested_stop_does_not_replace_it() {
    let root = tempfile::tempdir().unwrap();
    let recorder = Recorder::open(root.path());
    recorder.record(failure());
    let interrupted = Record::new(
        Stage::WaitNotification,
        &Err(io::Error::new(
            io::ErrorKind::NotConnected,
            "synthetic-private-input",
        )),
        true,
        Duration::from_secs(2),
        Some(456),
        ProcessState::Running,
    );
    recorder.record(interrupted);
    let file = root.path().join("connection-diagnostics.json");
    assert_eq!(file.metadata().unwrap().mode() & 0o777, 0o600);
    let raw = fs::read_to_string(&file).unwrap();
    assert!(!raw.contains("synthetic-private-input"));
    let saved = Recorder::open(root.path()).snapshot();
    assert!(!saved.load_failed);
    assert!(!saved.persistence_failed);
    assert_eq!(
        saved.last_failure.unwrap().cause,
        Some(Cause::NativeReceiveIdle)
    );
    assert_eq!(
        saved.last_transition.unwrap().recovery,
        Recovery::RequestedInterruption
    );
}

#[test]
fn persistence_failure_keeps_old_complete_record_and_reports_memory_failure() {
    let root = tempfile::tempdir().unwrap();
    let recorder = Recorder::open(root.path());
    recorder.record(failure());
    let mut next = failure();
    next.cause = Some(Cause::WorkerDisconnected);
    recorder.record_with(next, |_| Err(io::Error::from_raw_os_error(libc::ENOSPC)));
    let live = recorder.snapshot();
    assert!(live.persistence_failed);
    assert_eq!(
        live.last_failure.unwrap().cause,
        Some(Cause::WorkerDisconnected)
    );
    assert_eq!(
        Recorder::open(root.path())
            .snapshot()
            .last_failure
            .unwrap()
            .cause,
        Some(Cause::NativeReceiveIdle)
    );
    recorder.record(failure());
    assert!(!recorder.snapshot().persistence_failed);
}

#[test]
fn invalid_or_symlinked_diagnostics_are_not_trusted_or_followed() {
    let root = tempfile::tempdir().unwrap();
    let file = root.path().join("connection-diagnostics.json");
    for bytes in [b"incomplete".to_vec(), vec![b'x'; 16_385]] {
        fs::write(&file, bytes).unwrap();
        let snapshot = Recorder::open(root.path()).snapshot();
        assert!(snapshot.load_failed);
        assert!(snapshot.last_failure.is_none());
    }
    fs::remove_file(&file).unwrap();
    let other = root.path().join("other");
    fs::write(&other, b"synthetic-private-input").unwrap();
    std::os::unix::fs::symlink(&other, &file).unwrap();
    let recorder = Recorder::open(root.path());
    assert!(recorder.snapshot().load_failed);
    recorder.record(failure());
    assert_eq!(fs::read(&other).unwrap(), b"synthetic-private-input");
    assert!(!file.symlink_metadata().unwrap().file_type().is_symlink());
}

#[test]
fn worker_observation_distinguishes_live_cleanup_from_an_existing_exit() {
    let root = tempfile::tempdir().unwrap();
    let script = root.path().join("worker");
    fs::write(&script, b"#!/bin/sh\nexec sleep 30\n").unwrap();
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).unwrap();
    let recorder = Recorder::open(root.path());
    let mut child = WorkerProcess::start(&script).unwrap();
    let before = child.before_cleanup();
    assert_eq!(before, ProcessState::Running);
    let mut record = failure();
    record.worker_pid = Some(child.pid());
    record.before_cleanup = before;
    recorder.record(record);
    child.stop();
    assert_eq!(
        Recorder::open(root.path())
            .snapshot()
            .last_failure
            .unwrap()
            .before_cleanup,
        ProcessState::Running
    );
    drop(child);

    fs::write(&script, b"#!/bin/sh\nexit 23\n").unwrap();
    let mut child = WorkerProcess::start(&script).unwrap();
    let deadline = std::time::Instant::now() + Duration::from_secs(3);
    let status = loop {
        let status = child.before_cleanup();
        if status != ProcessState::Running {
            break status;
        }
        assert!(std::time::Instant::now() < deadline);
        std::thread::sleep(Duration::from_millis(5));
    };
    assert_eq!(
        status,
        ProcessState::Exited {
            code: Some(23),
            signal: None
        }
    );
    child.stop();
    assert_eq!(child.before_cleanup(), status);
}

// #10: a RecoveryRequired marker is a retry, never human attention; the
// exhausted terminal error is a plain attention-eligible InvalidData.
#[test]
fn recovery_required_marker_is_retried_instead_of_attention() {
    let error = crate::bridge::recovery_test_error("rescan_later");
    assert_eq!(
        crate::connection_diagnostics::recovery_cause(&error),
        Some("rescan_later")
    );
    assert!(!crate::connection_diagnostics::retryable(&error));
    assert!(
        crate::connection_diagnostics::retryable(&error)
            || crate::connection_diagnostics::recovery_cause(&error).is_some()
    );
    let exhausted = io::Error::new(
        io::ErrorKind::InvalidData,
        "native_state_recovery_required:rescan_later",
    );
    assert!(crate::connection_diagnostics::recovery_cause(&exhausted).is_none());
    assert!(!crate::connection_diagnostics::retryable(&exhausted));
}
#[test]
fn attempt_lifetime_alias_and_stall_heartbeat_are_recorded() {
    let mut record = failure()
        .with_business_progress(Some(1_700_000_000_000))
        .with_stall(
            crate::initialization::StallReason::NoBusinessProgress,
            Some(1_700_000_000_000),
            false,
        );
    assert_eq!(record.attempt_lifetime_ms, record.attempt_elapsed_ms);
    let json = serde_json::to_value(&record).unwrap();
    assert_eq!(json["attempt_elapsed_ms"], json["attempt_lifetime_ms"]);
    assert_eq!(json["stalled_reason"], "no_business_progress");
    assert_eq!(json["last_business_progress_at"], 1_700_000_000_000u64);
    // The watchdog closes the connection; the diagnostic must still name the
    // real cause instead of the resulting transport error.
    assert_eq!(json["cause"]["kind"], "attempt_stalled");
    assert_eq!(json["recovery"], "reconnect");
    record = failure().with_stall(
        crate::initialization::StallReason::NoBusinessProgress,
        None,
        true,
    );
    assert_eq!(record.recovery, Recovery::NeedsAttention);
    assert_eq!(
        serde_json::to_value(&record).unwrap()["recovery"],
        "needs_attention"
    );
}

#[test]
fn diagnostic_file_without_issue58_fields_still_loads() {
    let root = tempfile::tempdir().unwrap();
    let old = serde_json::json!({
        "schema": 1u8,
        "last_transition": {
            "unix_ms": 1u64,
            "attempt_elapsed_ms": 2000u64,
            "worker_pid": 7u32,
            "stage": "wait_notification",
            "cause": {"kind": "native_receive_idle"},
            "recovery": "reconnect",
            "before_cleanup": {"state": "running"}
        },
        "last_failure": null,
        "load_failed": false,
        "persistence_failed": false,
    });
    fs::write(
        root.path().join("connection-diagnostics.json"),
        old.to_string(),
    )
    .unwrap();
    let saved = Recorder::open(root.path()).snapshot();
    assert!(!saved.load_failed);
    let transition = saved.last_transition.expect("older record loads");
    assert_eq!(transition.attempt_elapsed_ms, 2000);
    assert_eq!(
        transition.attempt_lifetime_ms, 0,
        "older files predate the alias"
    );
    assert!(transition.last_business_progress_at.is_none());
    assert!(transition.stalled_reason.is_none());
}
