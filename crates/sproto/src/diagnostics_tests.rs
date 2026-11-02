use super::*;
use crate::{channel::Channel, frame, pstream};
use std::time::Duration;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn pull_counts_distinguish_raw_empty_no_advance_and_normalized_empty() {
    let capture = Capture::new();
    scope(capture.clone(), async {
        record_pull_response(2, false, false);
        record_pull_projection(None);
        record_pull_response(0, true, false);
        record_pull_projection(Some(0));
        record_pull_response(2, true, false);
        record_root_roundtrip_filtered(1);
        record_pull_projection(Some(1));
    })
    .await;
    let snapshot = capture.snapshot();
    assert!(snapshot.valid());
    assert_eq!(snapshot.pull.responses, 3);
    assert_eq!(snapshot.pull.raw_event_count, 4);
    assert_eq!(snapshot.pull.raw_empty_responses, 1);
    assert_eq!(snapshot.pull.no_advance_results, 1);
    assert_eq!(snapshot.pull.cursor_advanced_responses, 2);
    assert_eq!(snapshot.pull.root_roundtrip_filtered_count, 1);
    assert_eq!(snapshot.pull.projected_event_count, 1);
    assert_eq!(snapshot.pull.projected_empty_pages, 1);
    let mut sum = Snapshot::default();
    assert!(sum.merge(&snapshot));
    assert!(sum.merge(&snapshot));
    assert_eq!(sum.pull.responses, 6);
    assert!(sum.valid());
    let before = sum.clone();
    let mut invalid = snapshot.clone();
    invalid.pull.cursor_unchanged_responses = 99;
    assert!(!invalid.valid());
    assert!(!sum.merge(&invalid));
    assert_eq!(sum, before);
    let mut overflow = Snapshot::default();
    overflow.pull.responses = u64::MAX;
    overflow.pull.cursor_unchanged_responses = u64::MAX;
    assert!(overflow.valid());
    assert!(!sum.merge(&overflow));
    assert_eq!(sum, before);
}

fn encode(object: &PObject) -> Vec<u8> {
    let mut bytes = Vec::new();
    pstream::encode(object, &mut bytes).unwrap();
    bytes
}

#[tokio::test(start_paused = true)]
async fn partial_progress_does_not_reset_total_response_deadline() {
    let (stream, mut peer) = tokio::io::duplex(4096);
    let writer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(10)).await;
        peer.write_all(&encode(&pmap! {"type"=>"keep_alive"}))
            .await
            .unwrap();
        tokio::time::sleep(Duration::from_secs(10)).await;
        peer.write_all(&[0x42]).await.unwrap();
        std::future::pending::<()>().await;
    });
    let capture = Capture::new();
    let began = tokio::time::Instant::now();
    let result = scope(
        capture.clone(),
        Channel::from_stream(stream).request(
            frame::SCMD_LIST,
            &pmap! {"_action"=>"list","path"=>"private-path"},
        ),
    )
    .await;
    assert!(matches!(result, Err(Error::Timeout("native response"))));
    assert_eq!(began.elapsed(), Duration::from_secs(30));
    let sample = capture.snapshot().last_failure.unwrap();
    assert_eq!(sample.kind, RequestKind::List);
    assert_eq!(sample.phase, RequestPhase::Receive);
    assert_eq!(sample.outcome, RequestOutcome::ResponseTimeout);
    assert_eq!(sample.keepalive_frames, 1);
    assert!(sample.received_bytes > 1);
    writer.abort();
    let _ = writer.await;
}

#[tokio::test]
async fn continuation_counts_and_safe_serialization_keep_payload_private() {
    let (stream, mut peer) = tokio::io::duplex(8192);
    let a = pmap! {"@proto"=>pmap!{"body-continue"=>true},"node_list"=>PObject::Array(vec![PObject::from("private-name")])};
    let b = pmap! {"@proto"=>pmap!{"body-continue"=>false},"node_list"=>PObject::Array(vec![PObject::from("private-id")])};
    let writer = tokio::spawn(async move {
        peer.write_all(&encode(&a)).await.unwrap();
        peer.write_all(&encode(&b)).await.unwrap();
    });
    let capture = Capture::new();
    let result = scope(
        capture.clone(),
        Channel::from_stream(stream).request(
            frame::SCMD_LIST,
            &pmap! {"_action"=>"list","session"=>"secret-session"},
        ),
    )
    .await
    .unwrap();
    assert_eq!(
        result.get("node_list").unwrap().as_array().unwrap().len(),
        2
    );
    writer.await.unwrap();
    let snapshot = capture.snapshot();
    let sample = snapshot.last_request.as_ref().unwrap();
    assert_eq!(sample.frames, 2);
    assert_eq!(sample.continuation_frames, 1);
    assert_eq!(sample.outcome, RequestOutcome::Success);
    let json = serde_json::to_string(&snapshot).unwrap();
    for secret in ["private-name", "private-id", "secret-session"] {
        assert!(!json.contains(secret));
    }
    assert_eq!(serde_json::from_str::<Snapshot>(&json).unwrap(), snapshot);
}

#[tokio::test]
async fn cancellation_is_recorded_and_observer_runs_outside_capture_lock() {
    let (stream, _peer) = tokio::io::duplex(4096);
    let mut capture = Capture::new();
    let state = capture.state.clone();
    capture.observer = Some(Arc::new(move |sample| {
        assert!(state.try_lock().is_ok());
        assert_eq!(sample.outcome, RequestOutcome::Cancelled);
    }));
    let observed = capture.clone();
    let task = tokio::spawn(scope(observed, async move {
        Channel::from_stream(stream)
            .request(frame::SCMD_LIST, &pmap! {"_action"=>"list"})
            .await
    }));
    tokio::task::yield_now().await;
    task.abort();
    assert!(task.await.unwrap_err().is_cancelled());
    assert_eq!(
        capture.snapshot().last_failure.unwrap().outcome,
        RequestOutcome::Cancelled
    );
    assert_eq!(capture.snapshot().requests[0].cancelled, 1);
}

#[tokio::test]
async fn bounded_totals_do_not_store_untrusted_action_or_server_reason() {
    let capture = Capture::new();
    scope(capture.clone(), async {
        for _ in 0..1000 {
            let kind = RequestKind::of(&pmap! {"_action"=>"secret-arbitrary-action"});
            let _: Result<()> = request(kind, async {
                Err(Error::Server {
                    code: 12290,
                    reason: "secret-server-path".into(),
                })
            })
            .await;
        }
    })
    .await;
    let snapshot = capture.snapshot();
    assert_eq!(snapshot.requests.len(), 1);
    assert_eq!(snapshot.requests[0].failed, 1000);
    let json = serde_json::to_string(&snapshot).unwrap();
    assert!(!json.contains("secret"));
    assert!(json.len() < 2500);
}

#[tokio::test(start_paused = true)]
async fn send_timeout_is_distinct_from_receive_timeout() {
    let (stream, _peer) = tokio::io::duplex(1);
    let capture = Capture::new();
    let result = scope(
        capture.clone(),
        Channel::from_stream(stream).request_with_read_idle(
            frame::SCMD_REQUEST,
            &pmap! {"_action"=>"long_poll"},
            Duration::from_secs(600),
        ),
    )
    .await;
    assert!(matches!(result, Err(Error::Timeout("native request send"))));
    let sample = capture.snapshot().last_failure.unwrap();
    assert_eq!(sample.phase, RequestPhase::Send);
    assert_eq!(sample.outcome, RequestOutcome::SendTimeout);
    assert_eq!(sample.received_bytes, 0);
}

#[tokio::test]
async fn cancelled_pool_wait_and_setup_are_visible_without_a_request() {
    let capture = Capture::new();
    scope(capture.clone(), async {
        let _waiting = pool_wait();
        let _setup = setup();
    })
    .await;
    let snapshot = capture.snapshot();
    assert_eq!(snapshot.pool_wait_cancelled, 1);
    assert_eq!(snapshot.channel_setup_cancelled, 1);
    assert!(snapshot.requests.is_empty());
}

#[tokio::test]
async fn merge_is_bounded_preserves_failures_and_does_not_sum_concurrency() {
    let capture = Capture::new();
    scope(capture.clone(), async {
        let _: Result<()> = request(RequestKind::List, async {
            Err(Error::Timeout("native response"))
        })
        .await;
        pool_acquired(false, 4);
    })
    .await;
    let first = capture.snapshot();
    assert!(first.valid());
    let mut aggregate = Snapshot::default();
    assert!(aggregate.merge(&first));
    assert!(aggregate.merge(&first));
    assert_eq!(aggregate.requests[0].count, 2);
    assert_eq!(aggregate.max_pool_inflight, 4);
    let good = capture.snapshot();
    let mut bad = good.clone();
    bad.requests.push(bad.requests[0].clone());
    assert!(!bad.valid());
    let before = aggregate.clone();
    assert!(!aggregate.merge(&bad));
    assert_eq!(aggregate, before);
    let success = Capture::new();
    scope(
        success.clone(),
        request(RequestKind::List, async { Ok(()) }),
    )
    .await
    .unwrap();
    assert!(aggregate.merge(&success.snapshot()));
    assert!(aggregate.last_failure.is_some());
    assert_eq!(
        aggregate.last_request.unwrap().outcome,
        RequestOutcome::Success
    );
}
