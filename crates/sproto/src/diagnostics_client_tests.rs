use super::*;
use crate::pstream;
use tokio::io::AsyncWriteExt;

#[tokio::test]
async fn two_directory_pages_share_one_channel_and_count_actual_validated_nodes() {
    let client = Client {
        pool: Arc::new(
            Pool::new(pool::Config {
                port: 0,
                host: String::new(),
                tls_server_name: None,
                tls: TlsMode::None,
                session: "private-session".into(),
                restore_id: "private-restore".into(),
                device_uuid: "private-device".into(),
                max_channels: 1,
                tls_config: None,
            })
            .unwrap(),
        ),
        agent: pmap! {},
        session: "private-session".into(),
        server_build: 27892,
        restore_identity: "private-restore".into(),
    };
    let node = |n: u64| pmap! {"name"=>format!("name-{n}"),"file_hash"=>"private-hash","mtime"=>1u64,"node_id"=>n+1,"sync_id"=>1u64,"max_id"=>1u64,"file_id"=>format!("id-{n}"),"file_size"=>0u64,"file_type"=>"file"};
    let mut responses = Vec::new();
    pstream::encode(
        &pmap! {"node_list"=>PObject::Array((0..1000).map(node).collect()),"total_count"=>1001u64},
        &mut responses,
    )
    .unwrap();
    pstream::encode(
        &pmap! {"node_list"=>PObject::Array(vec![node(1000)]),"total_count"=>1001u64},
        &mut responses,
    )
    .unwrap();
    let (stream, mut peer) = tokio::io::duplex(512 * 1024);
    let writer = tokio::spawn(async move {
        peer.write_all(&responses).await.unwrap();
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    });
    let capture = crate::diagnostics::Capture::new();
    let nodes = crate::diagnostics::scope(
        capture.clone(),
        actions::list::list(
            &mut Channel::from_stream(stream),
            &client,
            1,
            "/private-dir",
        ),
    )
    .await
    .unwrap();
    let snapshot = capture.snapshot();
    assert_eq!(nodes.len(), 1001);
    assert_eq!(snapshot.list_pages, 2);
    assert_eq!(snapshot.list_items, 1001);
    assert_eq!(snapshot.requests.len(), 1);
    assert_eq!(
        snapshot.requests[0].kind,
        crate::diagnostics::RequestKind::List
    );
    assert_eq!(snapshot.requests[0].count, 2);
    assert_eq!(snapshot.channels_new, 0);
    assert!(
        !serde_json::to_string(&snapshot)
            .unwrap()
            .contains("private")
    );
    writer.abort();
    let _ = writer.await;
}
