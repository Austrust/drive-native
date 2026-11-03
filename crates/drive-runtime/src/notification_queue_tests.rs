use super::*;

#[test]
fn upstream_queue_overflow_disables_mount_leases_without_blocking() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    let hub = drive_fs::NotifyHub::with_ttl_ms(1000);
    hub.activate_ttl_for_tests(1000);
    let target = Mutex::new(Some(hub.clone()));
    enqueue_invalidation(&tx, &target, Invalidation::Inode(1));
    assert!(!hub.ttl().is_zero());
    enqueue_invalidation(&tx, &target, Invalidation::Inode(2));
    assert!(hub.ttl().is_zero());
    assert!(hub.stats().failed);
    assert_eq!(hub.stats().failures, 1);
    assert!(matches!(rx.try_recv(), Ok(Invalidation::Inode(1))));
    assert!(rx.try_recv().is_err());
}

#[test]
fn disconnected_feeder_disables_leases_and_keeps_failure_observable() {
    let (tx, rx) = std::sync::mpsc::sync_channel(1);
    drop(rx);
    let hub = drive_fs::NotifyHub::with_ttl_ms(1000);
    hub.activate_ttl_for_tests(1000);
    let target = Mutex::new(Some(hub.clone()));
    enqueue_invalidation(&tx, &target, Invalidation::Inode(1));
    enqueue_invalidation(&tx, &target, Invalidation::Inode(2));
    assert!(hub.ttl().is_zero());
    assert_eq!(hub.stats().failures, 1);
}
