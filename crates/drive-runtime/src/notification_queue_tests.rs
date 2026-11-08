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
