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
