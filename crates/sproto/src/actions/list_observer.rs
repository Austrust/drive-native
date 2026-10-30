//! Explicit, task-local pagination barriers for protocol experiments. No
//! environment flag or product IPC route can enable these pauses.
use crate::{Error, Result};
use tokio::sync::{mpsc, oneshot};

#[derive(Clone, Debug)]
pub struct PageBoundary {
    pub path: String,
    pub offset: u64,
    pub count: u64,
    pub total: u64,
    pub first_name: String,
    pub last_name: String,
}
pub struct PausedPage {
    pub boundary: PageBoundary,
    pub resume: oneshot::Sender<Result<()>>,
}
tokio::task_local! {
    static OBSERVER: mpsc::Sender<PausedPage>;
}
/// Only this future's list calls pause. Spawn an independent observer task;
/// task locals are not inherited by that task or by ordinary product requests.
pub async fn with_page_observer<F: std::future::Future>(
    observer: mpsc::Sender<PausedPage>,
    operation: F,
) -> F::Output {
    OBSERVER.scope(observer, operation).await
}
pub(crate) async fn at_page(boundary: PageBoundary) -> Result<()> {
    let Ok(observer) = OBSERVER.try_with(Clone::clone) else {
        return Ok(());
    };
    let (tx, rx) = oneshot::channel();
    tokio::time::timeout(std::time::Duration::from_secs(30), async {
        observer
            .send(PausedPage {
                boundary,
                resume: tx,
            })
            .await
            .map_err(|_| Error::ConnectionClosed)?;
        rx.await.map_err(|_| Error::ConnectionClosed)?
    })
    .await
    .map_err(|_| Error::Timeout("research page observer timed out"))?
}

#[cfg(test)]
mod tests {
    use super::*;
    fn boundary() -> PageBoundary {
        PageBoundary {
            path: "/lab".into(),
            offset: 0,
            count: 1000,
            total: 1002,
            first_name: "a".into(),
            last_name: "z".into(),
        }
    }
    #[tokio::test]
    async fn barrier_does_not_leak_into_observer_tasks_or_following_operations() {
        let (tx, mut rx) = mpsc::channel(1);
        let task = tokio::spawn(with_page_observer(tx, async {
            // An independent observer may itself list the same NAS directory.
            tokio::time::timeout(
                std::time::Duration::from_millis(500),
                tokio::spawn(at_page(boundary())),
            )
            .await
            .unwrap()
            .unwrap()
            .unwrap();
            at_page(boundary()).await
        }));
        let pause = rx.recv().await.unwrap();
        assert_eq!(pause.boundary.count, 1000);
        assert!(!task.is_finished());
        pause.resume.send(Ok(())).unwrap();
        task.await.unwrap().unwrap();
        assert!(rx.recv().await.is_none());
        at_page(boundary()).await.unwrap();
    }
    #[tokio::test]
    async fn observer_failure_is_not_silently_treated_as_a_complete_page() {
        let (tx, mut rx) = mpsc::channel(1);
        let task = tokio::spawn(with_page_observer(tx, at_page(boundary())));
        let pause = rx.recv().await.unwrap();
        pause
            .resume
            .send(Err(Error::Decode("mutation did not complete".into())))
            .unwrap();
        assert!(
            matches!(task.await.unwrap(), Err(Error::Decode(s)) if s=="mutation did not complete")
        );
        let (tx, rx) = mpsc::channel(1);
        drop(rx);
        assert!(matches!(
            with_page_observer(tx, at_page(boundary())).await,
            Err(Error::ConnectionClosed)
        ));
    }
}
