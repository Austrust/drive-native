//! Request-scoped, content-free observations of actual receive progress.
//!
//! The reader belongs below BufReader and above the decrypted transport. Reads
//! satisfied from an existing buffer, empty reads, EOF and writes do not renew
//! its clock. The protocol caller still owns cancellation and pool poisoning.
use std::{
    future::Future,
    io,
    pin::Pin,
    sync::{Arc, Mutex},
    task::{Context, Poll},
    time::Duration,
};
use tokio::{
    io::{AsyncRead, ReadBuf},
    time::Instant,
};

pub type Observer = Arc<dyn Fn(usize) + Send + Sync>;
tokio::task_local! { static OBSERVER: Observer; }

/// Observe only actual bytes received while polling this request. The callback
/// must not block; callers should coalesce notifications before sending IPC.
pub async fn scope<F: Future>(observer: Observer, future: F) -> F::Output {
    OBSERVER.scope(observer, future).await
}

#[derive(Clone)]
pub(crate) struct Clock(Arc<Mutex<Instant>>);
impl Clock {
    pub(crate) fn new() -> Self {
        Self(Arc::new(Mutex::new(Instant::now())))
    }

    fn received(&self, bytes: usize) {
        if bytes == 0 {
            return;
        }
        *self.0.lock().unwrap() = Instant::now();
        // No payload or paths cross this observer boundary.
        let _ = OBSERVER.try_with(|observer| observer(bytes));
    }

    /// Start a fresh receive-idle budget for one exclusively owned channel.
    /// No watchdog task survives cancellation of the enclosing future.
    pub(crate) async fn timeout<F: Future>(
        &self,
        budget: Duration,
        future: F,
    ) -> Result<F::Output, ()> {
        *self.0.lock().unwrap() = Instant::now();
        tokio::pin!(future);
        loop {
            let deadline = *self.0.lock().unwrap() + budget;
            tokio::select! {
                output = &mut future => return Ok(output),
                _ = tokio::time::sleep_until(deadline) => {
                    if Instant::now().duration_since(*self.0.lock().unwrap()) >= budget {
                        return Err(());
                    }
                }
            }
        }
    }
}

pub(crate) struct Reader<R> {
    inner: R,
    clock: Clock,
}
impl<R> Reader<R> {
    pub(crate) fn new(inner: R) -> Self {
        Self {
            inner,
            clock: Clock::new(),
        }
    }
    pub(crate) fn clock(&self) -> Clock {
        self.clock.clone()
    }
}
impl<R: AsyncRead + Unpin> AsyncRead for Reader<R> {
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        let this = self.get_mut();
        let before = buf.filled().len();
        let result = Pin::new(&mut this.inner).poll_read(cx, buf);
        if let Poll::Ready(Ok(())) = &result {
            this.clock.received(buf.filled().len() - before);
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::atomic::{AtomicUsize, Ordering};
    use tokio::io::{AsyncReadExt, AsyncWriteExt, BufReader};

    #[tokio::test(start_paused = true)]
    async fn actual_progress_can_outlive_the_initial_deadline() {
        let (input, mut output) = tokio::io::duplex(1);
        let mut reader = Reader::new(input);
        let clock = reader.clock();
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        let writer = tokio::spawn(async move {
            for byte in b"ABC" {
                tokio::time::sleep(Duration::from_millis(75)).await;
                output.write_all(&[*byte]).await.unwrap();
            }
        });
        let start = Instant::now();
        let mut bytes = [0; 3];
        scope(
            Arc::new(move |n| {
                observed.fetch_add(n, Ordering::Relaxed);
            }),
            clock.timeout(Duration::from_millis(100), reader.read_exact(&mut bytes)),
        )
        .await
        .unwrap()
        .unwrap();
        writer.await.unwrap();
        assert_eq!(&bytes, b"ABC");
        assert_eq!(count.load(Ordering::Relaxed), 3);
        assert_eq!(start.elapsed(), Duration::from_millis(225));
    }

    #[tokio::test(start_paused = true)]
    async fn partial_frame_then_silence_expires_from_the_last_real_byte() {
        let (input, mut output) = tokio::io::duplex(1);
        let mut reader = Reader::new(input);
        let clock = reader.clock();
        let writer = tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(75)).await;
            output.write_all(b"A").await.unwrap();
            std::future::pending::<()>().await;
        });
        let start = Instant::now();
        let mut bytes = [0; 2];
        assert!(
            clock
                .timeout(Duration::from_millis(100), reader.read_exact(&mut bytes))
                .await
                .is_err()
        );
        assert_eq!(start.elapsed(), Duration::from_millis(175));
        writer.abort();
        assert!(writer.await.unwrap_err().is_cancelled());
    }

    #[tokio::test(start_paused = true)]
    async fn buffer_hits_and_empty_reads_do_not_emit_or_renew() {
        let mut reader = BufReader::with_capacity(8, Reader::new(&b"ABCD"[..]));
        let clock = reader.get_ref().clock();
        let count = Arc::new(AtomicUsize::new(0));
        let observed = count.clone();
        scope(
            Arc::new(move |n| {
                observed.fetch_add(n, Ordering::Relaxed);
            }),
            async {
                assert_eq!(reader.read_u8().await.unwrap(), b'A');
                let last = *clock.0.lock().unwrap();
                tokio::time::sleep(Duration::from_millis(50)).await;
                assert_eq!(reader.read_u8().await.unwrap(), b'B');
                assert_eq!(reader.read(&mut []).await.unwrap(), 0);
                assert_eq!(*clock.0.lock().unwrap(), last);
            },
        )
        .await;
        assert_eq!(count.load(Ordering::Relaxed), 4);
    }

    #[tokio::test(start_paused = true)]
    async fn eof_is_immediate_and_never_reported_as_progress() {
        let mut reader = Reader::new(&b""[..]);
        let clock = reader.clock();
        let start = Instant::now();
        scope(Arc::new(|_| panic!("EOF is not progress")), async {
            let mut bytes = [0];
            let error = clock
                .timeout(Duration::from_secs(600), reader.read_exact(&mut bytes))
                .await
                .unwrap()
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::UnexpectedEof);
        })
        .await;
        assert_eq!(start.elapsed(), Duration::ZERO);
    }

    struct Failed;
    impl AsyncRead for Failed {
        fn poll_read(
            self: Pin<&mut Self>,
            _cx: &mut Context<'_>,
            _buf: &mut ReadBuf<'_>,
        ) -> Poll<io::Result<()>> {
            Poll::Ready(Err(io::ErrorKind::ConnectionReset.into()))
        }
    }
    #[tokio::test(start_paused = true)]
    async fn transport_errors_remain_distinct_from_idle_timeouts() {
        let mut reader = Reader::new(Failed);
        let clock = reader.clock();
        scope(Arc::new(|_| panic!("error is not progress")), async {
            let mut bytes = [0];
            let error = clock
                .timeout(Duration::from_secs(600), reader.read_exact(&mut bytes))
                .await
                .unwrap()
                .unwrap_err();
            assert_eq!(error.kind(), io::ErrorKind::ConnectionReset);
        })
        .await;
    }

    #[tokio::test(start_paused = true)]
    async fn observer_is_request_scoped_and_not_inherited_by_other_tasks() {
        scope(Arc::new(|_| panic!("unrelated task progress")), async {
            tokio::spawn(async {
                let mut reader = Reader::new(&b"A"[..]);
                assert_eq!(reader.read_u8().await.unwrap(), b'A');
            })
            .await
            .unwrap();
        })
        .await;
        let mut reader = Reader::new(&b"B"[..]);
        assert_eq!(reader.read_u8().await.unwrap(), b'B');
    }
}
