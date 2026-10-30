//! Explicit task-local barrier before a native upload goes onto the wire.
//! Used only by protocol experiments; product requests have no observer.
use crate::{Error, Result};
use tokio::sync::{mpsc, oneshot};
#[derive(Clone, Debug)]
pub struct UploadBoundary {
    pub view: Option<u64>,
    pub path: String,
}
pub struct PausedUpload {
    pub boundary: UploadBoundary,
    pub resume: oneshot::Sender<Result<()>>,
}
tokio::task_local! {static OBSERVER:mpsc::Sender<PausedUpload>;}
pub async fn with_upload_observer<F: std::future::Future>(
    observer: mpsc::Sender<PausedUpload>,
    operation: F,
) -> F::Output {
    OBSERVER.scope(observer, operation).await
}
pub(crate) async fn before_send(boundary: UploadBoundary) -> Result<()> {
    let Ok(observer) = OBSERVER.try_with(Clone::clone) else {
        return Ok(());
    };
    let (tx, rx) = oneshot::channel();
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        observer
            .send(PausedUpload {
                boundary,
                resume: tx,
            })
            .await
            .map_err(|_| Error::ConnectionClosed)?;
        rx.await.map_err(|_| Error::ConnectionClosed)?
    })
    .await
    .map_err(|_| Error::Timeout("research upload observer timed out"))?
}
#[cfg(test)]
mod tests {
    use super::*;
    fn point() -> UploadBoundary {
        UploadBoundary {
            view: None,
            path: "id:123/file".into(),
        }
    }
    #[tokio::test]
    async fn observer_does_not_apply_to_independent_requests_or_silence_failure() {
        let (tx, mut rx) = mpsc::channel(1);
        let job = tokio::spawn(with_upload_observer(tx, async {
            tokio::time::timeout(
                std::time::Duration::from_millis(500),
                tokio::spawn(before_send(point())),
            )
            .await
            .unwrap()
            .unwrap()
            .unwrap();
            before_send(point()).await
        }));
        let pause = rx.recv().await.unwrap();
        assert!(!job.is_finished());
        pause
            .resume
            .send(Err(Error::Decode("synthetic mutation failed".into())))
            .unwrap();
        assert!(matches!(job.await.unwrap(), Err(Error::Decode(_))));
        before_send(point()).await.unwrap();
    }
}
