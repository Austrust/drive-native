use super::*;
use tokio::io::AsyncWriteExt;

fn encoded(obj: &PObject) -> Vec<u8> {
    let mut bytes = Vec::new();
    pstream::encode(obj, &mut bytes).unwrap();
    bytes
}

#[tokio::test(start_paused = true)]
async fn split_frames_and_keepalives_extend_receive_idle_not_total_time() {
    let (client, mut server) = tokio::io::duplex(4096);
    let writer = tokio::spawn(async move {
        let keepalive = encoded(&pmap! { "type" => "keep_alive" });
        for _ in 0..3 {
            tokio::time::sleep(Duration::from_secs(300)).await;
            server.write_all(&keepalive).await.unwrap();
        }
        let response = encoded(&pmap! { "type" => "response", "sync_id" => 123u64 });
        let split = response.len() / 2;
        for part in [&response[..split], &response[split..]] {
            tokio::time::sleep(Duration::from_secs(300)).await;
            server.write_all(part).await.unwrap();
        }
    });
    let start = tokio::time::Instant::now();
    let result = Channel::from_stream(client)
        .request_with_read_idle(
            frame::SCMD_REQUEST,
            &pmap! { "_action" => "long_poll" },
            Duration::from_secs(600),
        )
        .await
        .unwrap();
    assert_eq!(result.get("sync_id").and_then(PObject::as_int), Some(123));
    assert_eq!(start.elapsed(), Duration::from_secs(1500));
    writer.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn legal_quiet_response_after_old_450_second_deadline_succeeds() {
    let (client, mut server) = tokio::io::duplex(4096);
    let writer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(489)).await;
        server
            .write_all(&encoded(&pmap! { "type" => "response" }))
            .await
            .unwrap();
    });
    Channel::from_stream(client)
        .request_with_read_idle(frame::SCMD_REQUEST, &pmap! {}, Duration::from_secs(600))
        .await
        .unwrap();
    writer.await.unwrap();
}

#[tokio::test(start_paused = true)]
async fn partial_response_then_silence_expires_from_last_actual_read() {
    let (client, mut server) = tokio::io::duplex(4096);
    let writer = tokio::spawn(async move {
        tokio::time::sleep(Duration::from_secs(500)).await;
        server.write_all(&[0x42]).await.unwrap(); // unfinished map
        std::future::pending::<()>().await;
    });
    let start = tokio::time::Instant::now();
    let error = Channel::from_stream(client)
        .request_with_read_idle(frame::SCMD_REQUEST, &pmap! {}, Duration::from_secs(600))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Timeout("native receive idle")));
    assert_eq!(start.elapsed(), Duration::from_secs(1100));
    writer.abort();
}

#[tokio::test(start_paused = true)]
async fn eof_remains_transport_error_and_send_is_bounded_separately() {
    let (client, server) = tokio::io::duplex(4096);
    drop(server);
    let error = Channel::from_stream(client)
        .request_with_read_idle(frame::SCMD_REQUEST, &pmap! {}, Duration::from_secs(600))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Io(_)));
    let (client, _server) = tokio::io::duplex(1); // request cannot fit
    let start = tokio::time::Instant::now();
    let error = Channel::from_stream(client)
        .request_with_read_idle(frame::SCMD_REQUEST, &pmap! {}, Duration::from_secs(600))
        .await
        .unwrap_err();
    assert!(matches!(error, Error::Timeout("native request send")));
    assert_eq!(start.elapsed(), Duration::from_secs(30));
}
