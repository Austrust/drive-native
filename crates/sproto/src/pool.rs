use std::sync::{Arc, Mutex};

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
    pub tls_config: Option<Arc<rustls::ClientConfig>>,
}

pub struct Pool {
    config: Arc<Config>,
    semaphore: Arc<Semaphore>,
    channels: Arc<Mutex<Vec<Channel>>>,
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
        Ok(Self {
            config: Arc::new(config),
            channels: Arc::new(Mutex::new(Vec::new())),
            semaphore: Arc::new(Semaphore::new(max)),
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
    #[allow(
        clippy::missing_panics_doc,
        reason = "mutex is never held across a panic"
    )]
    pub async fn acquire(&self) -> Result<Guard> {
        let permit = self
            .semaphore
            .clone()
            .acquire_owned()
            .await
            .map_err(|_| Error::ConnectionClosed)?;

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

    fn test_config(max_channels: usize) -> Config {
        Config {
            port: 0,
            max_channels,
            tls_config: None,
            tls: TlsMode::None,
            host: String::new(),
            tls_server_name: None,
            session: String::new(),
            restore_id: String::new(),
            device_uuid: String::new(),
        }
    }

    fn dummy_channel() -> Channel {
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
mod cancellation_tests {
    use super::*;
    async fn guard() -> (Guard, Arc<Mutex<Vec<Channel>>>) {
        let channels = Arc::new(Mutex::new(Vec::new()));
        let (stream, _peer) = tokio::io::duplex(16);
        let guard = Guard {
            reusable: false,
            channel: Some(Channel::from_stream(stream)),
            _permit: Arc::new(Semaphore::new(1)).acquire_owned().await.unwrap(),
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
