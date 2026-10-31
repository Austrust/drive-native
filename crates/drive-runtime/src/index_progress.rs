//! Coalesce actual completed indexing work; never block the network task on IPC.
use crate::wire::{Envelope, Response, VERSION};
use drive_core::snapshot_progress::{Observer, Progress};
use std::{
    os::fd::OwnedFd,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::{Duration, Instant},
};

#[derive(Default)]
struct State {
    latest: Option<Progress>,
    sent: Option<(Progress, Instant)>,
}
pub(crate) struct Reporter {
    id: u64,
    output: SyncSender<(Envelope<Response>, Option<OwnedFd>)>,
    state: Mutex<State>,
}
impl Reporter {
    pub(crate) fn new(
        id: u64,
        output: SyncSender<(Envelope<Response>, Option<OwnedFd>)>,
    ) -> Arc<Self> {
        Arc::new(Self {
            id,
            output,
            state: Mutex::new(State::default()),
        })
    }
    pub(crate) fn observer(self: &Arc<Self>) -> Observer {
        let reporter = self.clone();
        Arc::new(move |progress| reporter.observe(progress))
    }
    fn send_latest(&self, state: &mut State) {
        let Some(progress) = state.latest else { return };
        if state.sent.is_some_and(|(sent, _)| sent == progress) {
            return;
        }
        if self
            .output
            .try_send((
                Envelope {
                    version: VERSION,
                    id: self.id,
                    body: Response::IndexProgress(progress),
                },
                None,
            ))
            .is_ok()
        {
            state.sent = Some((progress, Instant::now()));
        }
    }
    fn observe(&self, progress: Progress) {
        let mut state = self.state.lock().unwrap();
        let boundary = state
            .latest
            .is_none_or(|old| old.attempt != progress.attempt || old.phase != progress.phase);
        if boundary {
            self.send_latest(&mut state);
        }
        state.latest = Some(progress);
        if boundary
            || state
                .sent
                .is_none_or(|(_, time)| time.elapsed() >= Duration::from_millis(250))
        {
            self.send_latest(&mut state);
        }
    }
    pub(crate) fn flush(&self) {
        self.send_latest(&mut self.state.lock().unwrap());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drive_core::snapshot_progress::Phase;
    #[test]
    fn boundary_and_completion_flush_actual_final_counts() {
        let (tx, rx) = std::sync::mpsc::sync_channel(8);
        let reporter = Reporter::new(7, tx);
        let first = Progress {
            attempt: 1,
            phase: Phase::Seed,
            units: 0,
            items: 0,
        };
        reporter.observe(first);
        reporter.observe(Progress {
            units: 3,
            items: 9,
            ..first
        });
        let next = Progress {
            phase: Phase::Enumerate,
            ..first
        };
        reporter.observe(next);
        reporter.observe(Progress {
            units: 2,
            items: 12,
            ..next
        });
        reporter.flush();
        reporter.flush();
        let values: Vec<_> = rx
            .try_iter()
            .map(|(env, fd)| {
                assert!(fd.is_none());
                match env.body {
                    Response::IndexProgress(p) => p,
                    _ => panic!(),
                }
            })
            .collect();
        assert_eq!(values.len(), 4);
        assert_eq!(values[1].units, 3);
        assert_eq!(values[3].items, 12);
    }
    #[test]
    fn saturated_ipc_keeps_only_latest_and_does_not_block() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        let reporter = Reporter::new(1, tx);
        for units in 0..500 {
            reporter.observe(Progress {
                attempt: 1,
                phase: Phase::Seed,
                units,
                items: units,
            });
        }
        rx.try_recv().unwrap();
        reporter.flush();
        assert!(matches!(
            rx.try_recv().unwrap().0.body,
            Response::IndexProgress(Progress { units: 499, .. })
        ));
    }
}
