use super::*;
use std::time::{Duration, Instant};

fn wait_until(deadline: Duration, mut condition: impl FnMut() -> bool) -> bool {
    let end = Instant::now() + deadline;
    while Instant::now() < end {
        if condition() {
            return true;
        }
        std::thread::sleep(Duration::from_millis(5));
    }
    condition()
}

#[test]
fn upstream_queue_overflow_degrades_then_recovers_without_restart() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let hub = drive_fs::NotifyHub::with_ttl_ms(20);
    hub.activate_ttl_for_tests(20);
    let target = Mutex::new(Some(hub.clone()));
    assert!(enqueue_invalidation(&tx, &target, Invalidation::Inode(1)));
    assert!(!hub.ttl().is_zero());
    assert!(!enqueue_invalidation(&tx, &target, Invalidation::Inode(2)));

    // A lost committed invalidation suspends new leases immediately, but the
    // mount is not permanently closed: the hub recovers after one old-lease
    // quiet window, without a restart or remount.
    assert!(hub.ttl().is_zero());
    let stats = hub.stats();
    assert!(stats.degraded);
    assert_eq!(
        stats.reason,
        Some(drive_fs::publication::DegradeReason::QueueOverflow)
    );
    assert_eq!(stats.overflows, 1);
    assert_eq!(stats.failures, 1);
    assert!(matches!(rx.try_recv(), Ok(Invalidation::Inode(1))));
    assert!(rx.try_recv().is_err());

    assert!(
        wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()),
        "overflowed mount never resumed cacheable publication"
    );
    assert!(hub.reply_guard().is_some());
    let stats = hub.stats();
    assert!(!stats.degraded);
    assert_eq!(stats.reason, None);
    assert_eq!(stats.recoveries, 1);
}

#[test]
fn disconnected_feeder_degrades_and_recovers_observably() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    drop(rx);
    let hub = drive_fs::NotifyHub::with_ttl_ms(20);
    hub.activate_ttl_for_tests(20);
    let target = Mutex::new(Some(hub.clone()));
    assert!(!enqueue_invalidation(&tx, &target, Invalidation::Inode(1)));
    assert!(hub.ttl().is_zero());
    assert_eq!(hub.stats().failures, 1);
    assert!(wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()));
    assert!(!hub.stats().degraded);
    assert_eq!(hub.stats().recoveries, 1);
}

#[test]
fn queue_overflow_exposes_health_and_degradation_window() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let hub = drive_fs::NotifyHub::with_ttl_ms(20);
    hub.activate_ttl_for_tests(20);
    let target = Mutex::new(Some(hub.clone()));
    assert!(enqueue_invalidation(&tx, &target, Invalidation::Inode(1)));
    assert!(!enqueue_invalidation(&tx, &target, Invalidation::Inode(2)));
    let health = hub.health();
    assert!(health.stats.degraded);
    assert_eq!(
        health.stats.reason,
        Some(drive_fs::publication::DegradeReason::QueueOverflow)
    );
    assert_eq!(health.stats.overflows, 1);
    assert!(health.degraded_for_ms.is_some());
    assert_eq!(health.last_degraded_ms, 0);
    assert!(wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()));
    let health = hub.health();
    assert!(!health.stats.degraded);
    assert_eq!(health.stats.recoveries, 1);
    assert_eq!(health.stats.overflows, 1);
    assert!(health.last_degraded_ms >= 100);
}

#[test]
fn bridge_notification_health_counts_collapsed_snapshot_generations() {
    let dir = tempfile::tempdir().unwrap();
    let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    )))
    .unwrap();
    let hub = drive_fs::NotifyHub::with_ttl_ms(20);
    hub.activate_ttl_for_tests(20);
    bridge.set_notify_hub(hub.clone());
    let invalidations = (0..210_000usize)
        .map(|index| (Some(index as u64), Some(index as u64), "entry".to_string()))
        .collect::<Vec<_>>();
    bridge.enqueue_snapshot_invalidations(invalidations);
    let health = bridge.notification_health();
    assert_eq!(health.generation_invalidations, 1);
    assert_eq!(health.overflows, 0);
    assert_eq!(health.overflows, bridge.notification_overflows());
    assert!(wait_until(Duration::from_secs(3), || {
        hub.stats().invalidations == 1
    }));
}

#[test]
fn notification_overflow_emits_structured_stderr_event() {
    let _ = super::drain_captured_events();
    let dir = tempfile::tempdir().unwrap();
    let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    )))
    .unwrap();
    let hub = drive_fs::NotifyHub::with_ttl_ms(20);
    hub.activate_ttl_for_tests(20);
    bridge.set_notify_hub(hub.clone());
    // Stall the feeder on the hub slot so the bounded channel fills and at
    // least one committed invalidation is rejected.
    let target = bridge.notify_hub.clone();
    let (acquired, ready) = std::sync::mpsc::channel();
    let holder = std::thread::spawn(move || {
        let guard = target.lock().unwrap();
        acquired.send(()).unwrap();
        std::thread::sleep(Duration::from_millis(300));
        drop(guard);
    });
    ready.recv_timeout(Duration::from_secs(2)).unwrap();
    for ino in 0..1100u64 {
        bridge.enqueue_invalidation(Invalidation::Inode(ino));
    }
    holder.join().unwrap();
    let health = bridge.notification_health();
    assert!(
        health.overflows >= 1,
        "expected a rejected queue send: {health:?}"
    );
    assert_eq!(health.overflows, bridge.notification_overflows());
    let events = super::drain_captured_events();
    let joined = events.join("\n");
    assert!(joined.contains("event=notification_overflow"), "{joined}");
    assert!(joined.contains("dropped="), "{joined}");
    // The mount suspended cacheable publication but recovers without restart.
    assert!(wait_until(Duration::from_secs(2), || !hub.ttl().is_zero()));
    assert!(!hub.health().stats.degraded);
}

#[test]
fn generation_invalidation_emits_structured_stderr_event() {
    let _ = super::drain_captured_events();
    let dir = tempfile::tempdir().unwrap();
    let bridge = Bridge::offline(Arc::new(drive_core::SharedStore::new(
        Store::open(dir.path()).unwrap(),
    )))
    .unwrap();
    let invalidations = (0..210_000usize)
        .map(|index| (Some(index as u64), Some(index as u64), "entry".to_string()))
        .collect::<Vec<_>>();
    bridge.enqueue_snapshot_invalidations(invalidations);
    let events = super::drain_captured_events();
    let joined = events.join("\n");
    assert!(joined.contains("event=generation_invalidation"), "{joined}");
    assert!(joined.contains("generation_invalidations=1"), "{joined}");
    assert!(joined.contains("per_node_messages="), "{joined}");
}
