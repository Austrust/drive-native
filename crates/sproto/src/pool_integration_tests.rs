//! Pool lane behaviour against the in-process protocol fixture.
//!
//! These tests cover issue #33's pool starvation requirements: a pending server
//! long poll must not consume a shared metadata channel, and shared-channel
//! waits must be bounded rather than unbounded queueing.

use std::time::{Duration, Instant};

use crate::actions::native::{EventCursor, Watch};
use crate::test_support;

async fn wait_until(mut condition: impl FnMut() -> bool) {
    let deadline = Instant::now() + Duration::from_secs(5);
    while !condition() {
        assert!(Instant::now() < deadline, "condition was not reached");
        tokio::time::sleep(Duration::from_millis(2)).await;
    }
}

/// A pending long poll occupies its dedicated connection only: metadata
/// listings keep using the shared pool and complete quickly.
#[tokio::test]
async fn pending_long_poll_keeps_metadata_channels_available() {
    let fixture = test_support::start().await;
    fixture.set_files("/big", 10);
    let client = fixture.connect(2).await;
    fixture.reset_stats();

    let polling_client = client.clone();
    let polling = tokio::spawn(async move {
        let watch = Watch {
            view_id: 54,
            node_id: 3,
            path: "/big".into(),
            cursor: EventCursor(0),
        };
        polling_client
            .wait_changes(&[watch], "fixture-profile")
            .await
    });

    // The fixture holds the long poll open without answering.
    wait_until(|| fixture.stats().other_requests >= 1).await;

    let started = Instant::now();
    let listing = client.list_dir(54, "/big").await.unwrap();
    assert_eq!(listing.len(), 10);
    assert!(
        started.elapsed() < Duration::from_secs(2),
        "metadata request was blocked by the pending long poll"
    );
    assert!(
        fixture.stats().connections <= 2,
        "long poll must use one dedicated connection, not a shared channel"
    );

    polling.abort();
}
