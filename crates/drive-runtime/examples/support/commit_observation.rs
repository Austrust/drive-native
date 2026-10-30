//! A separate authenticated session observes metadata without reading the
//! committing connection's response. The transfer helper supplies a 30s bound.
pub async fn file_after(
    client: &sproto::Client,
    view: u64,
    parent: &str,
    name: &str,
    size: u64,
    after_sync: u64,
) -> sproto::Result<()> {
    loop {
        let nodes = client.list_dir(view, parent).await?;
        if nodes.iter().any(|n| {
            n.name == name
                && !n.is_removed
                && n.file_type == sproto::actions::list::FileType::File
                && n.file_size == size
                && n.sync_id > after_sync
        }) {
            return Ok(());
        }
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
    }
}
pub fn assert_unread_fault(result: sproto::Result<sproto::actions::transfer::ContentVersion>) {
    assert!(
        matches!(result, Err(sproto::Error::Timeout(reason))
        if reason == "injected unread upload reply after independent commit observation"),
        "independent commit observation must complete before injecting receipt loss"
    );
}
