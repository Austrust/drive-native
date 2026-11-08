use std::sync::{Arc, Mutex};
use std::time::Duration;

use tokio::sync::{OwnedSemaphorePermit, Semaphore};

use crate::{
    channel::{Channel, TlsMode},
    error::{Error, Result},
    pstream::PObject,
};

#[allow(
    clippy::struct_field_names,
    reason = "tls_config is different from tls"
)]
pub struct Config {
    pub port: u16,
    pub host: String,
    pub tls_server_name: Option<String>,
    pub tls: TlsMode,
    pub session: String,
    pub restore_id: String,
    pub device_uuid: String,
    pub max_channels: usize,
    /// Upper bound on callers queued for a shared metadata channel.
    ///
    /// Dispatch concurrency can exceed the channel budget (initially 32 vs 6),
    /// so without a bound the wait queue would grow without limit. Rejecting
    /// with [`Error::Timeout`] keeps metadata latency observable and bounded.
    pub max_waiters: usize,
    /// Maximum time a caller may wait for a shared channel permit.
    pub queue_wait_timeout: Duration,
    pub tls_config: Option<Arc<rustls::ClientConfig>>,
}

pub struct Pool {
    config: Arc<Config>,
    semaphore: Arc<Semaphore>,
    channels: Arc<Mutex<Vec<Channel>>>,
    waiters: Arc<Semaphore>,
    long_poll: Arc<Semaphore>,
    long_poll_channel: Arc<Mutex<Option<Channel>>>,
    transfers: Arc<Semaphore>,
}

impl Pool {
    /// Create a new connection pool from the given config.
    pub fn new(config: Config) -> Result<Self> {
        if config.max_channels == 0 {
            return Err(Error::InvalidConfig(
                "max_channels must be at least 1".into(),
            ));
        }

        let max = config.max_channels;
        let max_waiters = config.max_waiters;
        // Keep metadata capacity for every transfer: with `max` shared
        // channels, at most `max - 2` transfers may hold one at a time. A
        // smaller pool still allows one transfer.
        let transfer_lanes = max.saturating_sub(2).max(1);
        Ok(Self {
            config: Arc::new(config),
            channels: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max)),
            waiters: Arc::new(Semaphore::new(max_waiters)),
            long_poll: Arc::new(Semaphore::new(1)),
            long_poll_channel: Arc::new(Mutex::new(None)),
            transfers: Arc::new(Semaphore::new(transfer_lanes)),
        })
    }

    /// Pre-create channels in parallel to avoid cold-start latency.
    ///
    /// Creates up to `count` channels (capped at the pool's max).
    #[allow(
        clippy::missing_panics_doc,
        reason = "mutex is never held across a panic"
    )]
    pub async fn warm(&self, count: usize) -> Result<()> {
        // Acquire permits to reserve capacity and prevent concurrent
        // acquire() calls from creating channels for the same slots.
        let mut permits = Vec::new();
        for _ in 0..count.min(self.config.max_channels) {
            match self.semaphore.clone().try_acquire_owned() {
                Ok(p) => permits.push(p),
                Err(_) => break,
            }
        }

        // Some reserved slots may already have usable idle channels.
        // Only count non-expired ones — expired channels will be discarded
        // by acquire(), so they don't save us a reconnect.
        let existing = {
            let mut channels = self.channels.lock().unwrap();
            channels.retain(|ch| !ch.is_expired());
            channels.len()
        };
        let to_create = permits.len().saturating_sub(existing);
        // Release permits we won't use so acquire() isn't blocked needlessly.
        drop(permits.drain(to_create..));

        let mut handles = Vec::new();
        tracing::debug!(to_create, existing, "warming pool");

        for _ in 0..to_create {
            let config = Arc::clone(&self.config);
            handles.push(tokio::spawn(async move { connect_channel(&config).await }));
        }

        let mut new_channels = Vec::with_capacity(handles.len());
        for handle in handles {
            new_channels.push(handle.await.map_err(|e| {
                crate::error::Error::Decode(format!("pool warm task failed: {e}"))
            })??);
        }

        {
            let mut channels = self.channels.lock().unwrap();
            channels.append(&mut new_channels);
            tracing::debug!(total = channels.len(), "pool warm complete");
        }
        drop(permits);

        Ok(())
    }

    /// Donate an existing channel to the pool (e.g. the auth channel).
    /// The channel must already be TLS-connected; this performs the connect handshake.
    #[allow(
        clippy::missing_panics_doc,
        reason = "mutex is never held across a panic"
    )]
    pub async fn donate(&self, mut ch: Channel) -> Result<()> {
        handshake(&mut ch, &self.config).await?;
        self.channels.lock().unwrap().push(ch);
        Ok(())
    }

    /// Acquire a channel from the pool, creating a new one if none are idle.
    ///
    /// The wait is bounded twice: at most [`Config::max_waiters`] callers may
    /// queue, and a queued caller gives up after [`Config::queue_wait_timeout`].
    /// Both limits fail with [`Error::Timeout`] so callers can surface a
    /// bounded-latency failure instead of queueing forever.
    #[allow(
        clippy::missing_panics_doc,
        reason = "mutex is never held across a panic"
    )]
    pub async fn acquire(&self) -> Result<Guard> {
        let waiter = self
            .waiters
            .clone()
            .try_acquire_owned()
            .map_err(|_| Error::Timeout("native pool queue full"))?;

        let mut waiting = crate::diagnostics::pool_wait();
        let permit = match tokio::time::timeout(
            self.config.queue_wait_timeout,
            self.semaphore.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(Error::ConnectionClosed),
            Err(_) => return Err(Error::Timeout("native pool queue wait")),
        };
        waiting.acquired = true;
        drop(waiting);
        drop(waiter);

        self.take_channel(permit).await
    }

    /// Acquire a channel for a file transfer.
    ///
    /// Transfers hold a channel for the whole download/upload, so they get
    /// their own lane: at most `max_channels - 2` transfers run concurrently,
    /// leaving metadata channels available even while transfers are saturated.
    pub async fn acquire_transfer(&self) -> Result<Guard> {
        let transfer = match tokio::time::timeout(
            self.config.queue_wait_timeout,
            self.transfers.clone().acquire_owned(),
        )
        .await
        {
            Ok(Ok(permit)) => permit,
            Ok(Err(_)) => return Err(Error::ConnectionClosed),
            Err(_) => return Err(Error::Timeout("native transfer lane wait")),
        };
        let mut guard = self.acquire().await?;
        guard._transfer = Some(transfer);
        Ok(guard)
    }

    /// Acquire the dedicated long-poll lane.
    ///
    /// A long poll blocks server-side for several minutes (`long_poll` timeout
    /// 400s plus 600s read idle). Sharing a metadata channel for that duration
    /// would silently remove one of `max_channels` from the metadata budget, so
    /// the lane owns one separate connection that never counts against the
    /// shared pool permits.
    pub async fn acquire_long_poll(&self) -> Result<LongPollGuard> {
        let permit = self
            .long_poll
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::ConnectionClosed)?;
        let existing = self.long_poll_channel.lock().unwrap().take();
        let channel = match existing {
            Some(ch) if !ch.is_expired() => {
                tracing::debug!("pool: reusing dedicated long-poll channel");
                ch
            }
            _ => {
                tracing::debug!("pool: creating dedicated long-poll channel");
                connect_channel(&self.config).await?
            }
        };
        Ok(LongPollGuard {
            reusable: false,
            channel: Some(channel),
            _permit: permit,
            slot: Arc::clone(&self.long_poll_channel),
        })
    }

    async fn take_channel(&self, permit: OwnedSemaphorePermit) -> Result<Guard> {
        // Pop channels until we find one that hasn't expired, discarding stale ones.
        let existing = {
            let mut channels = self.channels.lock().unwrap();
            loop {
                match channels.pop() {
                    Some(ch) if ch.is_expired() => {
                        tracing::debug!("pool: discarding expired channel");
                        drop(ch);
                    }
                    other => break other,
                }
            }
        };

        crate::diagnostics::pool_acquired(
            existing.is_none(),
            self.config
                .max_channels
                .saturating_sub(self.semaphore.available_permits()),
        );
        let channel = if let Some(ch) = existing {
            tracing::debug!("pool: reusing idle channel");
            ch
        } else {
            tracing::debug!("pool: creating new channel");
            connect_channel(&self.config).await?
        };

        Ok(Guard {
            reusable: false,
            _permit: permit,
            _transfer: None,
            channel: Some(channel),
            channels: Arc::clone(&self.channels),
        })
    }
}

pub struct Guard {
    // Fail closed: dropping a cancelled request must close its partially consumed stream.
    reusable: bool,
    channel: Option<Channel>,
    _permit: OwnedSemaphorePermit,
    _transfer: Option<OwnedSemaphorePermit>,
    channels: Arc<Mutex<Vec<Channel>>>,
}

impl Guard {
    /// Get a mutable reference to the underlying channel.
    ///
    /// # Panics
    /// Panics if the channel has been poisoned.
    pub const fn channel(&mut self) -> &mut Channel {
        self.reusable = false;
        self.channel.as_mut().unwrap()
    }

    /// Poison the channel if `result` is an error, then return the result.
    pub fn poison_on_err<T>(&mut self, result: Result<T>) -> Result<T> {
        self.reusable = result.is_ok();
        if result.is_err() {
            self.channel = None;
        }

        result
    }
}

impl Drop for Guard {
    fn drop(&mut self) {
        if self.reusable
            && let Some(ch) = self.channel.take()
        {
            self.channels.lock().unwrap().push(ch);
        }
    }
}

/// Guard for the dedicated long-poll channel.
///
/// Mirrors [`Guard`]'s fail-closed reuse policy but hands the channel back to
/// the long-poll slot instead of the shared channel list.
pub struct LongPollGuard {
    reusable: bool,
    channel: Option<Channel>,
    _permit: OwnedSemaphorePermit,
    slot: Arc<Mutex<Option<Channel>>>,
}

impl LongPollGuard {
    /// Get a mutable reference to the underlying channel.
    ///
    /// # Panics
    /// Panics if the channel has been poisoned.
    pub const fn channel(&mut self) -> &mut Channel {
        self.reusable = false;
        self.channel.as_mut().unwrap()
    }

    /// Poison the channel if `result` is an error, then return the result.
    pub fn poison_on_err<T>(&mut self, result: Result<T>) -> Result<T> {
        self.reusable = result.is_ok();
        if result.is_err() {
            self.channel = None;
        }

        result
    }
}

impl Drop for LongPollGuard {
    fn drop(&mut self) {
        if self.reusable
            && let Some(ch) = self.channel.take()
        {
            *self.slot.lock().unwrap() = Some(ch);
        }
    }
}

/// Build the `_agent` `PObject` block used in handshake and requests.
#[must_use]
pub fn agent_map(device_uuid: &str, restore_id: &str) -> PObject {
    pmap! {
        "type" => "drive",
        "platform" => "linux",
        "restore_id" => restore_id,
        "device_uuid" => device_uuid,
        "version" => pmap! {
            "major" => 4u64,
            "minor" => 0u64,
            "mini" => 0u64,
            "build" => crate::frame::CLIENT_BUILD,
        },
    }
}

/// Open a new channel and perform the connect handshake.
async fn connect_channel(config: &Config) -> Result<Channel> {
    let mut span = crate::diagnostics::setup();
    let result = async {
        let mut ch = Channel::connect(
            &config.host,
            config.port,
            config.tls_server_name.as_deref(),
            config.tls,
            config.tls_config.as_ref(),
        )
        .await?;
        handshake(&mut ch, config).await?;
        Ok(ch)
    }
    .await;
    span.outcome = Some(result.is_ok());
    result
}

/// Perform the `connect` handshake on a newly created channel.
async fn handshake(ch: &mut Channel, config: &Config) -> Result<()> {
    let request = pmap! {
        "_action" => "connect",
        "client_type" => "drive",
        "session" => config.session.as_str(),
        "restore_id" => config.restore_id.as_str(),
        "client_version" => crate::frame::CLIENT_VERSION,
        "_agent" => agent_map(&config.device_uuid, &config.restore_id),
    };

    tracing::debug!("sending connect handshake");
    let response = ch.request(crate::frame::SCMD_CONNECT, &request).await?;
    tracing::debug!("connect handshake complete");

    let alive = response
        .get("alive")
        .and_then(PObject::as_int)
        .unwrap_or(300);
    if alive == 0 {
        return Err(Error::Server {
            code: 0,
            reason: "server refused connection (alive=0)".into(),
        });
    }

    ch.alive_interval = Some(alive);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    pub(super) fn test_config(max_channels: usize) -> Config {
        Config {
            port: 0,
            max_channels,
            max_waiters: max_channels.saturating_mul(8).max(1),
            queue_wait_timeout: Duration::from_millis(200),
            tls_config: None,
            tls: TlsMode::None,
            host: String::new(),
            tls_server_name: None,
            session: String::new(),
            restore_id: String::new(),
            device_uuid: String::new(),
        }
    }

    pub(super) fn dummy_channel() -> Channel {
        let (a, _b) = tokio::io::duplex(1);
        Channel::from_stream(a)
    }

    #[test]
    fn zero_max_channels_returns_error() {
        let result = Pool::new(test_config(0));
        assert!(
            matches!(&result, Err(Error::InvalidConfig(_))),
            "expected InvalidConfig"
        );
    }

    #[tokio::test]
    async fn warm_holds_permits_while_creating() {
        let pool = Pool::new(test_config(3)).unwrap();
        assert_eq!(pool.semaphore.available_permits(), 3);

        // Manually acquire all permits to simulate warm holding them.
        // Then verify acquire() cannot proceed.
        let p1 = pool.semaphore.clone().try_acquire_owned().unwrap();
        let p2 = pool.semaphore.clone().try_acquire_owned().unwrap();
        let p3 = pool.semaphore.clone().try_acquire_owned().unwrap();
        assert_eq!(pool.semaphore.available_permits(), 0);

        // acquire() should not be able to get a permit.
        assert!(pool.semaphore.clone().try_acquire_owned().is_err());

        drop((p1, p2, p3));
        assert_eq!(pool.semaphore.available_permits(), 3);
    }

    #[tokio::test]
    async fn warm_skips_existing_idle_channels() {
        let pool = Pool::new(test_config(5)).unwrap();

        // Pre-fill the pool with 3 dummy channels.
        {
            let mut channels = pool.channels.lock().unwrap();
            for _ in 0..3 {
                channels.push(dummy_channel());
            }
        }

        // warm(5) should acquire at most 5 permits, subtract the 3 existing,
        // and only try to create 2 channels. Since there's no server, those
        // spawned tasks will fail — but we can verify the permit math.
        let _ = pool.warm(5).await;

        // After warm (even if it fails), all permits should be released.
        assert_eq!(pool.semaphore.available_permits(), 5);
        // The 3 pre-filled channels should still be in the pool.
        assert!(pool.channels.lock().unwrap().len() >= 3);
    }

    #[tokio::test]
    async fn warm_releases_permits_on_connection_failure() {
        let pool = Pool::new(test_config(3)).unwrap();

        // warm will fail to connect (no server), but must release all permits.
        let _ = pool.warm(3).await;

        assert_eq!(
            pool.semaphore.available_permits(),
            3,
            "permits must be fully restored after failed warm"
        );
    }
}

#[cfg(test)]
mod lane_tests {
    use super::tests::{dummy_channel, test_config};
    use super::*;

    async fn wait_until(mut condition: impl FnMut() -> bool) {
        for _ in 0..1_000 {
            if condition() {
                return;
            }
            tokio::task::yield_now().await;
        }
        panic!("condition was not reached");
    }

    /// Excess callers are rejected immediately instead of extending the queue.
    #[tokio::test]
    async fn queue_bound_rejects_excess_waiters() {
        let mut config = test_config(1);
        config.max_waiters = 1;
        let pool = Arc::new(Pool::new(config).unwrap());
        let held = pool.semaphore.clone().try_acquire_owned().unwrap();

        let task_pool = Arc::clone(&pool);
        let queued = tokio::spawn(async move { task_pool.acquire().await.map(|_| ()) });
        wait_until(|| pool.waiters.available_permits() == 0).await;

        let Err(error) = pool.acquire().await else {
            panic!("expected acquire rejection");
        };
        assert!(
            matches!(error, Error::Timeout("native pool queue full")),
            "unexpected error: {error}"
        );

        queued.abort();
        drop(held);
    }

    /// A queued caller never blocks forever: the wait itself times out.
    #[tokio::test(start_paused = true)]
    async fn queue_wait_times_out_instead_of_blocking_forever() {
        let mut config = test_config(1);
        config.queue_wait_timeout = Duration::from_millis(50);
        let pool = Pool::new(config).unwrap();
        let _held = pool.semaphore.clone().try_acquire_owned().unwrap();

        let Err(error) = pool.acquire().await else {
            panic!("expected acquire rejection");
        };
        assert!(
            matches!(error, Error::Timeout("native pool queue wait")),
            "unexpected error: {error}"
        );
    }

    /// The long-poll lane uses its own connection and leaves every shared
    /// metadata permit available.
    #[tokio::test]
    async fn long_poll_lane_does_not_consume_metadata_channels() {
        let pool = Pool::new(test_config(2)).unwrap();
        {
            let mut channels = pool.channels.lock().unwrap();
            channels.push(dummy_channel());
            channels.push(dummy_channel());
        }
        *pool.long_poll_channel.lock().unwrap() = Some(dummy_channel());

        let mut long_poll = pool.acquire_long_poll().await.unwrap();
        let _ = long_poll.channel();
        assert_eq!(
            pool.semaphore.available_permits(),
            2,
            "long poll must not consume shared permits"
        );

        let metadata = pool.acquire().await.unwrap();
        assert_eq!(pool.semaphore.available_permits(), 1);
        drop(metadata);

        long_poll.poison_on_err(Ok(())).unwrap();
        drop(long_poll);
        assert!(
            pool.long_poll_channel.lock().unwrap().is_some(),
            "completed long-poll channel is reused"
        );
        assert_eq!(pool.long_poll.available_permits(), 1);
    }

    /// Transfers reserve metadata capacity: a full transfer lane still leaves
    /// at least two shared channels for metadata calls.
    #[tokio::test]
    async fn transfer_lane_reserves_metadata_capacity() {
        let pool = Pool::new(test_config(3)).unwrap();
        {
            let mut channels = pool.channels.lock().unwrap();
            for _ in 0..3 {
                channels.push(dummy_channel());
            }
        }

        let transfer = pool.acquire_transfer().await.unwrap();
        assert_eq!(
            pool.semaphore.available_permits(),
            2,
            "one transfer may hold one of three channels"
        );
        let first = pool.acquire().await.unwrap();
        let second = pool.acquire().await.unwrap();
        assert_eq!(pool.semaphore.available_permits(), 0);

        // The transfer lane caps transfers below max_channels.
        let Err(error) = pool.acquire_transfer().await else {
            panic!("expected transfer-lane timeout");
        };
        assert!(
            matches!(error, Error::Timeout("native transfer lane wait")),
            "unexpected error: {error}"
        );

        drop((first, second, transfer));
    }
}

#[cfg(test)]
mod cancellation_tests {
    use super::*;
    #[tokio::test]
    async fn keepalive_then_truncated_metadata_eof_is_not_success_or_reusable() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let channels = Arc::new(Mutex::new(Vec::new()));
        let permits = Arc::new(Semaphore::new(1));
        let (stream, mut peer) = tokio::io::duplex(4096);
        let mut guard = Guard {
            reusable: false,
            channel: Some(Channel::from_stream(stream)),
            _permit: permits.clone().acquire_owned().await.unwrap(),
            _transfer: None,
            channels: channels.clone(),
        };
        let server = tokio::spawn(async move {
            let mut request = [0; 1024];
            assert!(peer.read(&mut request).await.unwrap() > 0);
            let mut keepalive = Vec::new();
            crate::pstream::encode(&pmap! {"type"=>"keep_alive"}, &mut keepalive).unwrap();
            let mut partial = Vec::new();
            crate::pstream::encode(&pmap! {"node_list"=>PObject::Array(vec![])}, &mut partial)
                .unwrap();
            assert_eq!(partial.pop(), Some(0x40));
            peer.write_all(&keepalive).await.unwrap();
            peer.write_all(&partial).await.unwrap();
            peer.shutdown().await.unwrap();
        });
        let capture = crate::diagnostics::Capture::new();
        let result = crate::diagnostics::scope(
            capture.clone(),
            guard
                .channel()
                .request(crate::frame::SCMD_LIST, &pmap! {"_action"=>"list"}),
        )
        .await;
        assert!(
            matches!(&result, Err(Error::Io(e)) if e.kind()==std::io::ErrorKind::UnexpectedEof)
        );
        assert!(guard.poison_on_err(result).is_err());
        drop(guard);
        server.await.unwrap();
        assert!(channels.lock().unwrap().is_empty());
        assert_eq!(permits.available_permits(), 1);
        let snapshot = capture.snapshot();
        assert!(snapshot.valid());
        let sample = snapshot.last_failure.unwrap();
        assert_eq!(
            sample.outcome,
            crate::diagnostics::RequestOutcome::Transport
        );
        assert_eq!(sample.frames, 1);
        assert_eq!(sample.keepalive_frames, 1);
        assert!(sample.received_bytes > 0 && sample.positive_read_count > 0);
    }

    #[tokio::test(start_paused = true)]
    async fn cancelled_partial_notification_closes_stream_and_releases_permit() {
        use tokio::io::{AsyncReadExt, AsyncWriteExt};
        let channels = Arc::new(Mutex::new(Vec::new()));
        let permits = Arc::new(Semaphore::new(1));
        let (stream, mut peer) = tokio::io::duplex(4096);
        let mut guard = Guard {
            reusable: false,
            channel: Some(Channel::from_stream(stream)),
            _permit: permits.clone().acquire_owned().await.unwrap(),
            _transfer: None,
            channels: channels.clone(),
        };
        let task = tokio::spawn(async move {
            let result = guard
                .channel()
                .request_with_read_idle(
                    crate::frame::SCMD_REQUEST,
                    &pmap! {},
                    std::time::Duration::from_secs(600),
                )
                .await;
            guard.poison_on_err(result)
        });
        let mut request = [0; 1024];
        assert!(peer.read(&mut request).await.unwrap() > 0);
        peer.write_all(&[0x42]).await.unwrap(); // incomplete response consumed before cancellation
        tokio::task::yield_now().await;
        task.abort();
        assert!(task.await.unwrap_err().is_cancelled());
        peer.read_to_end(&mut Vec::new()).await.unwrap(); // EOF, including any remaining request bytes
        assert_eq!(permits.available_permits(), 1);
        assert!(channels.lock().unwrap().is_empty());
    }

    async fn guard() -> (Guard, Arc<Mutex<Vec<Channel>>>) {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let (stream, _peer) = tokio::io::duplex(16);
        let guard = Guard {
            reusable: false,
            channel: Some(Channel::from_stream(stream)),
            _permit: Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap(),
            _transfer: None,
            channels: channels.clone(),
        };
        (guard, channels)
    }
    #[tokio::test]
    async fn cancelled_request_never_returns_connection_to_pool() {
        let (mut g, c) = guard().await;
        let _ = g.channel();
        drop(g);
        assert!(c.lock().unwrap().is_empty());
    }
    #[tokio::test]
    async fn only_completed_request_is_reusable() {
        let (mut g, c) = guard().await;
        g.poison_on_err(Ok(())).unwrap();
        drop(g);
        assert_eq!(c.lock().unwrap().len(), 1);
    }
    #[tokio::test]
    async fn new_borrow_invalidates_prior_success() {
        let (mut g, c) = guard().await;
        g.poison_on_err(Ok(())).unwrap();
        let _ = g.channel();
        drop(g);
        assert!(c.lock().unwrap().is_empty());
    }
}
