//! #31 backpressure: a command burst must queue, not end the worker session.
//!
//! The worker used to `break` as soon as 32 jobs were in flight, aborting every
//! running job and turning a local overload into a whole-mount failure. These
//! tests pin the replacement queue semantics: at most `MAX_IN_FLIGHT` job bodies
//! run concurrently, up to `MAX_ACCEPTED` commands may wait, cancellations abort
//! queued or running jobs, and shutdown aborts everything promptly.
use super::{JobQueue, MAX_ACCEPTED, MAX_IN_FLIGHT};
use std::sync::{
    atomic::{AtomicUsize, Ordering},
    Arc,
};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Let queued commands start, then drain exactly `count` completions.
async fn drain(queue: &mut JobQueue, count: usize) {
    for _ in 0..count {
        queue
            .next_completed()
            .await
            .expect("every accepted job must complete");
    }
}

#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
async fn commands_beyond_in_flight_queue_without_ending_the_session() {
    // Eight accepted commands with only two execution permits: all eight must
    // stay accepted (queued), never dropped and never running more than two.
    let gate = Arc::new(Semaphore::new(0));
    let running = Arc::new(AtomicUsize::new(0));
    let peak = Arc::new(AtomicUsize::new(0));
    let done = Arc::new(AtomicUsize::new(0));
    let mut queue = JobQueue::new(2, 16);
    for id in 1..=8u64 {
        let gate = gate.clone();
        let running = running.clone();
        let peak = peak.clone();
        let done = done.clone();
        queue.spawn(id, async move {
            let now = running.fetch_add(1, Ordering::SeqCst) + 1;
            peak.fetch_max(now, Ordering::SeqCst);
            let _permit = gate.acquire().await.expect("gate open");
            running.fetch_sub(1, Ordering::SeqCst);
            done.fetch_add(1, Ordering::SeqCst);
        });
    }
    assert_eq!(queue.accepted(), 8, "burst commands must be accepted");
    assert!(queue.can_accept(), "accepted cap is 16");
    gate.add_permits(8);
    while done.load(Ordering::SeqCst) < 8 {
        queue
            .next_completed()
            .await
            .expect("queued jobs must complete");
    }
    while queue.accepted() > 0 {
        queue
            .next_completed()
            .await
            .expect("queued jobs must complete");
    }
    assert_eq!(queue.accepted(), 0, "finished jobs leave the accepted set");
    assert!(
        peak.load(Ordering::SeqCst) <= 2,
        "in-flight cap exceeded: {} bodies ran concurrently",
        peak.load(Ordering::SeqCst)
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn accepted_limit_is_the_only_reason_to_refuse_a_command() {
    let gate = Arc::new(Semaphore::new(0));
    let mut queue = JobQueue::new(1, 3);
    for id in 1..=3u64 {
        let gate = gate.clone();
        queue.spawn(id, async move {
            let _permit = gate.acquire().await.expect("gate open");
        });
    }
    assert_eq!(queue.accepted(), 3);
    assert!(!queue.can_accept(), "the accepted cap must be observable");
    assert!(queue.cancel(2), "cancelling an accepted command succeeds");
    assert_eq!(queue.accepted(), 2);
    assert!(queue.can_accept(), "a freed slot admits the next command");
    gate.add_permits(10);
    drain(&mut queue, 2).await;
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn cancelling_a_queued_command_never_runs_its_body() {
    let gate = Arc::new(Semaphore::new(0));
    let started = Arc::new(std::sync::Mutex::new(Vec::<u64>::new()));
    let mut queue = JobQueue::new(1, 4);
    for id in 1..=2u64 {
        let gate = gate.clone();
        let started = started.clone();
        queue.spawn(id, async move {
            started.lock().unwrap().push(id);
            let _permit = gate.acquire().await.expect("gate open");
        });
    }
    assert!(queue.cancel(2));
    gate.add_permits(4);
    drain(&mut queue, 1).await;
    assert_eq!(
        started.lock().unwrap().as_slice(),
        &[1],
        "a cancelled queued command must not execute"
    );
}

#[tokio::test(flavor = "multi_thread", worker_threads = 2)]
async fn shutdown_aborts_queued_and_running_jobs_promptly() {
    let gate = Arc::new(Semaphore::new(0));
    let mut queue = JobQueue::new(1, 4);
    for id in 1..=2u64 {
        let gate = gate.clone();
        queue.spawn(id, async move {
            let _permit = gate.acquire().await.expect("gate open");
        });
    }
    let started = Instant::now();
    tokio::time::timeout(Duration::from_secs(2), queue.shutdown())
        .await
        .expect("shutdown must abort queued and running jobs");
    assert!(started.elapsed() < Duration::from_secs(2));
}

#[test]
fn constants_keep_backpressure_bounded() {
    const _: () = assert!(MAX_IN_FLIGHT > 0);
    const _: () = assert!(MAX_ACCEPTED > MAX_IN_FLIGHT);
    assert_eq!(MAX_IN_FLIGHT, 32, "existing in-flight concurrency target");
    assert!(
        MAX_ACCEPTED <= 256,
        "queued commands need a hard memory bound"
    );
}
