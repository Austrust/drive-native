//! Local IPC accounting for native notification reads. No timer heartbeats,
//! server data, paths or authentication material cross this boundary.
use crate::wire::{Envelope, Response, VERSION};
use std::{
    io,
    sync::{Arc, Mutex, mpsc::SyncSender},
    time::Duration,
};

pub(crate) fn monotonic_ns() -> io::Result<u64> {
    let mut time = libc::timespec {
        tv_sec: 0,
        tv_nsec: 0,
    };
    // Both IPC peers are local processes in the same time namespace.
    if unsafe { libc::clock_gettime(libc::CLOCK_MONOTONIC, &mut time) } != 0 {
        return Err(io::Error::last_os_error());
    }
    u64::try_from(time.tv_sec)
        .ok()
        .and_then(|secs| secs.checked_mul(1_000_000_000))
        .and_then(|ns| ns.checked_add(time.tv_nsec as u64))
        .ok_or_else(|| io::Error::other("invalid monotonic clock"))
}

pub(crate) struct WaitClock {
    started: u64,
    last: u64,
    bytes: u64,
}
impl WaitClock {
    pub(crate) fn new() -> io::Result<Self> {
        let now = monotonic_ns()?;
        Ok(Self {
            started: now,
            last: now,
            bytes: 0,
        })
    }
    pub(crate) fn update(&mut self, bytes: u64, received: u64) -> io::Result<()> {
        self.update_at(bytes, received, monotonic_ns()?)
    }
    fn update_at(&mut self, bytes: u64, received: u64, now: u64) -> io::Result<()> {
        if bytes == 0 || bytes < self.bytes || received > now || received < self.started {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid wait progress",
            ));
        }
        if bytes == self.bytes {
            // Duplicate delivery cannot extend the idle window.
            return Ok(());
        }
        if received < self.last {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "reversed wait progress",
            ));
        }
        self.bytes = bytes;
        self.last = received;
        Ok(())
    }
    pub(crate) fn remaining(&self, budget: Duration) -> Duration {
        // Clock failure fails closed instead of leaving a request immortal.
        monotonic_ns()
            .map(|now| self.remaining_at(budget, now))
            .unwrap_or_default()
    }
    fn remaining_at(&self, budget: Duration, now: u64) -> Duration {
        budget.saturating_sub(Duration::from_nanos(now.saturating_sub(self.last)))
    }
}

pub(crate) fn observer(
    id: u64,
    output: SyncSender<(Envelope<Response>, Option<std::os::fd::OwnedFd>)>,
) -> sproto::receive_activity::Observer {
    // Coalesce only on actual reads. Never spawn a ticker to report activity.
    let state = Mutex::new((0u64, 0u64));
    Arc::new(move |count| {
        if count == 0 {
            return;
        }
        let Ok(now) = monotonic_ns() else { return };
        let mut state = state.lock().unwrap();
        let Some(bytes) = state.0.checked_add(count as u64) else {
            return;
        };
        state.0 = bytes;
        if state.1 != 0 && now.saturating_sub(state.1) < 250_000_000 {
            return;
        }
        if output
            .try_send((
                Envelope {
                    version: VERSION,
                    id,
                    body: Response::WaitProgress {
                        bytes,
                        received_at_ns: now,
                    },
                },
                None,
            ))
            .is_ok()
        {
            state.1 = now;
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn delayed_and_duplicate_messages_do_not_freshen_the_clock() {
        let mut c = WaitClock {
            started: 100,
            last: 100,
            bytes: 0,
        };
        c.update_at(5, 150, 190).unwrap();
        assert_eq!(
            c.remaining_at(Duration::from_nanos(100), 190),
            Duration::from_nanos(60)
        );
        c.update_at(5, 195, 200).unwrap();
        assert_eq!(
            c.remaining_at(Duration::from_nanos(100), 250),
            Duration::ZERO
        );
        for (bytes, time) in [(0, 150), (4, 150), (6, 149), (6, 99), (6, 201)] {
            assert!(c.update_at(bytes, time, 200).is_err());
        }
    }
    #[test]
    fn saturated_output_is_nonblocking_and_progress_is_cumulative() {
        let (tx, rx) = std::sync::mpsc::sync_channel(1);
        tx.send((
            Envelope {
                version: VERSION,
                id: 99,
                body: Response::Conflict,
            },
            None,
        ))
        .unwrap();
        let observe = observer(7, tx);
        observe(3); // full: no blocking, retain total for the next real read
        rx.recv().unwrap();
        observe(2);
        let (env, fd) = rx.recv().unwrap();
        assert!(fd.is_none());
        assert_eq!(env.id, 7);
        assert!(matches!(env.body, Response::WaitProgress { bytes: 5, .. }));
        observe(0);
        assert!(rx.try_recv().is_err());
    }
}
