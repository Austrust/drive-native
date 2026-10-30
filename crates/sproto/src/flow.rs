//! Per-task payload accounting and shared directional pacing. Metadata stays unthrottled.
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Duration,
};
use tokio::time::Instant;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Direction {
    Download,
    Upload,
}
impl Direction {
    fn index(self) -> usize {
        match self {
            Self::Download => 0,
            Self::Upload => 1,
        }
    }
    pub fn name(self) -> &'static str {
        match self {
            Self::Download => "download",
            Self::Upload => "upload",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Queued,
    Preparing,
    Transferring,
    Verifying,
    AwaitingConfirmation,
}
impl Phase {
    pub fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Preparing => "preparing",
            Self::Transferring => "transferring",
            Self::Verifying => "verifying",
            Self::AwaitingConfirmation => "awaiting_confirmation",
        }
    }
}
#[derive(Clone, Debug)]
pub struct Progress {
    pub epoch: u64,
    pub direction: Direction,
    pub phase: Phase,
    pub bytes: u64,
    pub total: u64,
}
struct Bucket {
    rate: u64,
    tokens: f64,
    last: Instant,
}
impl Bucket {
    fn capacity(&self) -> usize {
        if self.rate == 0 {
            65536
        } else {
            (self.rate / 10).clamp(1, 65536) as usize
        }
    }
    fn allowance(&mut self, wanted: usize, now: Instant) -> Option<usize> {
        if self.rate == 0 {
            return Some(wanted.min(65536));
        }
        let capacity = self.capacity();
        self.tokens = (self.tokens
            + now.duration_since(self.last).as_secs_f64() * self.rate as f64)
            .min(capacity as f64);
        self.last = now;
        let take = wanted.min(capacity);
        if self.tokens >= take as f64 {
            self.tokens -= take as f64;
            Some(take)
        } else {
            None
        }
    }
}
pub struct TransferControl {
    buckets: Mutex<[Bucket; 2]>,
}
impl TransferControl {
    pub fn new(download: u64, upload: u64) -> Arc<Self> {
        let now = Instant::now();
        Arc::new(Self {
            buckets: Mutex::new([
                Bucket {
                    rate: download,
                    tokens: 0.0,
                    last: now,
                },
                Bucket {
                    rate: upload,
                    tokens: 0.0,
                    last: now,
                },
            ]),
        })
    }
    pub fn set_limits(&self, download: u64, upload: u64) {
        let now = Instant::now();
        let mut buckets = self.buckets.lock().unwrap();
        for (bucket, rate) in buckets.iter_mut().zip([download, upload]) {
            if bucket.rate != rate {
                *bucket = Bucket {
                    rate,
                    tokens: 0.0,
                    last: now,
                };
            }
        }
    }
    pub async fn acquire(&self, direction: Direction, wanted: usize) -> usize {
        if wanted == 0 {
            return 0;
        }
        loop {
            let wait = {
                let mut buckets = self.buckets.lock().unwrap();
                let b = &mut buckets[direction.index()];
                if let Some(n) = b.allowance(wanted, Instant::now()) {
                    return n;
                }
                ((wanted.min(b.capacity()) as f64 - b.tokens) / b.rate as f64).clamp(0.001, 0.1)
            };
            // Small bounded sleeps allow a changed limit or cancellation to take effect.
            tokio::time::sleep(Duration::from_secs_f64(wait)).await;
        }
    }
}
struct State {
    progress: Option<Progress>,
    last_activity: Instant,
    last_emit: Instant,
}
#[derive(Clone)]
struct Context {
    control: Arc<TransferControl>,
    state: Arc<Mutex<State>>,
    observer: Arc<dyn Fn(Progress) + Send + Sync>,
}
tokio::task_local! {static CURRENT: Context;}
pub async fn scope<F: Future>(
    control: Arc<TransferControl>,
    observer: Arc<dyn Fn(Progress) + Send + Sync>,
    future: F,
) -> F::Output {
    let now = Instant::now();
    CURRENT
        .scope(
            Context {
                control,
                state: Arc::new(Mutex::new(State {
                    progress: None,
                    last_activity: now,
                    last_emit: now,
                })),
                observer,
            },
            future,
        )
        .await
}
pub fn begin(direction: Direction, total: u64, phase: Phase) {
    let _ = CURRENT.try_with(|c| {
        let event = {
            let mut state = c.state.lock().unwrap();
            let epoch = state.progress.as_ref().map_or(1, |p| p.epoch + 1);
            let p = Progress {
                epoch,
                direction,
                phase,
                bytes: 0,
                total,
            };
            state.progress = Some(p.clone());
            state.last_activity = Instant::now();
            state.last_emit = state.last_activity;
            p
        };
        (c.observer)(event);
    });
}
pub fn phase(phase: Phase) {
    let _ = CURRENT.try_with(|c| {
        let event = {
            let mut state = c.state.lock().unwrap();
            let now = Instant::now();
            state.last_activity = now;
            state.last_emit = now;
            state.progress.as_mut().map(|p| {
                p.phase = phase;
                p.clone()
            })
        };
        if let Some(p) = event {
            (c.observer)(p);
        }
    });
}
pub async fn allowance(direction: Direction, wanted: usize) -> usize {
    match CURRENT.try_with(|c| c.control.clone()) {
        Ok(control) => control.acquire(direction, wanted).await,
        Err(_) => wanted,
    }
}
pub fn advance(direction: Direction, n: usize) {
    let _ = CURRENT.try_with(|c| {
        let event = {
            let mut state = c.state.lock().unwrap();
            let now = Instant::now();
            let emit = now.duration_since(state.last_emit) >= Duration::from_millis(100);
            let Some(p) = state.progress.as_mut() else {
                return;
            };
            if p.direction != direction {
                return;
            }
            p.bytes = p.bytes.saturating_add(n as u64);
            p.phase = if p.bytes == p.total {
                match direction {
                    Direction::Download => Phase::Verifying,
                    Direction::Upload => Phase::AwaitingConfirmation,
                }
            } else {
                Phase::Transferring
            };
            let event = (emit || p.bytes >= p.total).then(|| p.clone());
            state.last_activity = now;
            if event.is_some() {
                state.last_emit = now;
            }
            event
        };
        if let Some(p) = event {
            (c.observer)(p);
        }
    });
}
/// A transfer may outlive the old wall-clock timeout while making payload progress.
/// Awaiting a reply or stalled data still times out; pauses cancel the enclosing task.
pub async fn idle_timeout<F: Future>(
    duration: Duration,
    future: F,
) -> std::result::Result<F::Output, ()> {
    let Ok(state) = CURRENT.try_with(|c| c.state.clone()) else {
        return tokio::time::timeout(duration, future).await.map_err(|_| ());
    };
    state.lock().unwrap().last_activity = Instant::now();
    tokio::pin!(future);
    loop {
        let last = state.lock().unwrap().last_activity;
        let deadline = last + duration;
        tokio::select! {output=&mut future=>return Ok(output),_=tokio::time::sleep_until(deadline)=>{
            if Instant::now().duration_since(state.lock().unwrap().last_activity)>=duration{return Err(());}
        }}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn directional_budget_is_shared_and_no_initial_burst() {
        let control = TransferControl::new(1000, 2000);
        let mut b = control.buckets.lock().unwrap();
        let now = b[0].last;
        assert_eq!(b[0].allowance(100, now), None);
        assert_eq!(
            b[0].allowance(100, now + Duration::from_millis(100)),
            Some(100)
        );
        assert_eq!(b[0].allowance(100, now + Duration::from_millis(100)), None);
        assert_eq!(
            b[1].allowance(200, now + Duration::from_millis(100)),
            Some(200)
        );
    }
    #[tokio::test]
    async fn progress_comes_from_actual_bytes_and_has_no_synced_phase() {
        let log = Arc::new(Mutex::new(Vec::new()));
        let sink = log.clone();
        scope(
            TransferControl::new(0, 0),
            Arc::new(move |p| sink.lock().unwrap().push(p)),
            async {
                begin(Direction::Upload, 3, Phase::Preparing);
                assert_eq!(allowance(Direction::Upload, 3).await, 3);
                advance(Direction::Upload, 3);
                phase(Phase::AwaitingConfirmation);
                begin(Direction::Download, 0, Phase::Queued);
                phase(Phase::Verifying);
            },
        )
        .await;
        let events = log.lock().unwrap();
        assert_eq!(events[0].bytes, 0);
        assert!(events.iter().any(|p| p.direction == Direction::Upload
            && p.bytes == 3
            && p.phase == Phase::AwaitingConfirmation));
        assert_eq!(events.last().unwrap().epoch, 2);
        assert_eq!(events.last().unwrap().total, 0);
    }
    #[tokio::test]
    async fn changed_limit_wakes_pacing_and_cancel_releases_no_shared_lock() {
        let control = TransferControl::new(1, 0);
        let c = control.clone();
        let job = tokio::spawn(async move { c.acquire(Direction::Download, 100).await });
        tokio::time::sleep(Duration::from_millis(20)).await;
        control.set_limits(0, 0);
        assert_eq!(
            tokio::time::timeout(Duration::from_millis(300), job)
                .await
                .unwrap()
                .unwrap(),
            100
        );
        control.set_limits(1, 0);
        let c = control.clone();
        let job = tokio::spawn(async move { c.acquire(Direction::Download, 100).await });
        job.abort();
        let _ = job.await;
        control.set_limits(0, 0);
        assert_eq!(control.acquire(Direction::Download, 100).await, 100);
    }
    #[tokio::test]
    async fn progressing_transfer_can_outlive_deadline_but_stalled_transfer_cannot() {
        scope(TransferControl::new(0, 0), Arc::new(|_| {}), async {
            begin(Direction::Download, 4, Phase::Transferring);
            assert!(
                idle_timeout(Duration::from_millis(150), async {
                    for _ in 0..4 {
                        tokio::time::sleep(Duration::from_millis(60)).await;
                        advance(Direction::Download, 1);
                    }
                })
                .await
                .is_ok()
            );
            assert!(
                idle_timeout(Duration::from_millis(30), std::future::pending::<()>())
                    .await
                    .is_err()
            );
        })
        .await;
    }
}
