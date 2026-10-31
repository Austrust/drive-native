//! Optional, task-local observation of completed metadata work. No paths,
//! identities or cursors are exposed and observers cannot affect the protocol.
use serde::{Deserialize, Serialize};
use std::sync::Arc;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Phase {
    Seed,
    Enumerate,
    Fence,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Progress {
    pub attempt: u8,
    pub phase: Phase,
    /// Completed event pages (seed/fence) or directories (enumeration).
    pub units: u64,
    /// Events inspected (seed/fence) or nodes collected (enumeration).
    pub items: u64,
}
pub type Observer = Arc<dyn Fn(Progress) + Send + Sync>;
tokio::task_local! { static OBSERVER: Observer; }
pub async fn scope<T>(observer: Observer, future: impl std::future::Future<Output = T>) -> T {
    OBSERVER.scope(observer, future).await
}
pub(crate) fn emit(progress: Progress) {
    let _ = OBSERVER.try_with(|observer| observer(progress));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[tokio::test]
    async fn observations_are_task_scoped_and_not_implicitly_inherited() {
        let values = Arc::new(std::sync::Mutex::new(Vec::new()));
        let result = values.clone();
        let progress = Progress {
            attempt: 1,
            phase: Phase::Enumerate,
            units: 2,
            items: 7,
        };
        emit(progress);
        scope(Arc::new(move |p| result.lock().unwrap().push(p)), async {
            emit(progress);
            tokio::spawn(async move { emit(progress) }).await.unwrap();
        })
        .await;
        emit(progress);
        assert_eq!(values.lock().unwrap().len(), 1);
    }
}
