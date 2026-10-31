//! Explicit current-boundary query reconstructed from the official clients.
//! A received boundary is not a committed cursor or an atomic directory snapshot.
//! Production bootstrap must still validate its root, enumeration and event fence.
use super::{
    list::{FileType, NodeInfo},
    native::{EventCursor, Watch, validate_path},
};
use crate::{Client, Error, Result, frame, pstream::PObject};

fn watermark_fields(watch: &Watch, restore: &str) -> Result<PObject> {
    validate_path(&watch.path)?;
    if watch.view_id == 0 || watch.node_id == 0 || restore.is_empty() {
        return Err(Error::InvalidConfig(
            "watermark requires a bound native root".into(),
        ));
    }
    // ProtocolFactory supplies root_node_id/root_node_path, not the pull-event
    // node_id field. GetMaxSyncId separately adds the same entry path as `path`.
    Ok(pmap! {
        "view_id" => watch.view_id,
        "root_node_id" => watch.node_id,
        "root_node_path" => watch.path.as_str(),
        "path" => watch.path.as_str(),
        "server_id" => pmap! {"restore_id" => restore},
    })
}

fn parse_watermark(response: &PObject) -> Result<EventCursor> {
    response
        .get("max_sync_id")
        .and_then(PObject::as_int)
        .map(EventCursor)
        .ok_or_else(|| Error::Decode("missing or invalid max_sync_id".into()))
}

fn validate_selected_node(node: &NodeInfo, path: &str, file_id: &str, node_id: u64) -> Result<()> {
    if node.file_id != file_id
        || node.node_id != node_id
        || node.file_type != FileType::Dir
        || node.is_removed
        || path.rsplit('/').next() != Some(node.name.as_str())
    {
        return Err(Error::Decode("selected root identity changed".into()));
    }
    Ok(())
}

impl Client {
    /// Capture the server's current numeric event boundary for a bound root.
    /// This operation does not install metadata or advance any durable cursor.
    /// Official-client static evidence establishes its request shape; callers
    /// must validate server/scope compatibility before changing bootstrap policy.
    pub async fn current_sync_watermark(&self, watch: &Watch) -> Result<EventCursor> {
        let req = self.build_request(
            "get_max_sync_id",
            watermark_fields(watch, self.restore_identity())?,
        );
        let mut guard = self.pool.acquire().await?;
        let result = async {
            let response = guard.channel().request(frame::SCMD_REQUEST, &req).await?;
            parse_watermark(&response)
        }
        .await;
        guard.poison_on_err(result)
    }

    /// Read a non-root path through the already supported parent directory list.
    /// The exact file identity is mandatory: a same-name replacement is not the
    /// selected root. Missing/inaccessible parents remain distinct from identity
    /// mismatch. `/` has no parent listing and is deliberately unsupported here.
    pub async fn current_selected_node(
        &self,
        view: u64,
        path: &str,
        expected_file_id: &str,
        expected_node_id: u64,
    ) -> Result<Option<NodeInfo>> {
        if expected_file_id.is_empty() || expected_node_id == 0 {
            return Err(Error::InvalidConfig(
                "selected root identity missing".into(),
            ));
        }
        let node = self.current_path(view, path).await?;
        if let Some(node) = &node {
            validate_selected_node(node, path, expected_file_id, expected_node_id)?;
        }
        Ok(node)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn watch() -> Watch {
        Watch {
            view_id: 7,
            node_id: 19,
            path: "/selected".into(),
            cursor: EventCursor(999),
        }
    }
    #[test]
    fn reconstructed_context_does_not_confuse_root_or_event_fields() {
        let fields = watermark_fields(&watch(), "synthetic-restore").unwrap();
        assert_eq!(fields.get("view_id").and_then(PObject::as_int), Some(7));
        assert_eq!(
            fields.get("root_node_id").and_then(PObject::as_int),
            Some(19)
        );
        assert_eq!(
            fields.get("root_node_path").and_then(PObject::as_str),
            Some("/selected")
        );
        assert_eq!(
            fields.get("path").and_then(PObject::as_str),
            Some("/selected")
        );
        assert_eq!(
            fields
                .get("server_id")
                .unwrap()
                .get("restore_id")
                .and_then(PObject::as_str),
            Some("synthetic-restore")
        );
        for key in ["node_id", "sync_id", "recursive", "cursor"] {
            assert!(fields.get(key).is_none());
        }
    }
    #[test]
    fn boundary_is_strict_unsigned_and_not_a_file_version() {
        for n in [0, 1, u64::MAX] {
            assert_eq!(
                parse_watermark(&pmap! {"max_sync_id"=>n}).unwrap(),
                EventCursor(n)
            );
        }
        for value in [
            pmap! {},
            pmap! {"sync_id"=>3u64},
            pmap! {"max_id"=>3u64},
            pmap! {"max_sync_id"=>"3"},
            pmap! {"max_sync_id"=>PObject::Null},
            pmap! {"max_sync_id"=>PObject::Array(vec![])},
        ] {
            assert!(parse_watermark(&value).is_err());
        }
    }
    #[test]
    fn invalid_root_is_rejected_before_a_request() {
        for path in ["", "relative", "/a/../b", "/a//b", "/a\0b"] {
            let mut w = watch();
            w.path = path.into();
            assert!(watermark_fields(&w, "r").is_err());
        }
        let mut w = watch();
        w.node_id = 0;
        assert!(watermark_fields(&w, "r").is_err());
        assert!(watermark_fields(&watch(), "").is_err());
    }
    #[test]
    fn selected_root_requires_all_identity_fields() {
        let good = NodeInfo {
            node_id: 19,
            sync_id: 5,
            max_id: 5,
            name: "selected".into(),
            file_size: 0,
            mtime: 1,
            hash: String::new(),
            file_id: "synthetic-id".into(),
            file_type: FileType::Dir,
            is_removed: false,
        };
        assert!(validate_selected_node(&good, "/parent/selected", "synthetic-id", 19).is_ok());
        for changed in 0..5 {
            let mut node = good.clone();
            match changed {
                0 => node.file_id = "replacement".into(),
                1 => node.node_id = 20,
                2 => node.name = "other".into(),
                3 => node.file_type = FileType::File,
                _ => node.is_removed = true,
            };
            assert!(validate_selected_node(&node, "/parent/selected", "synthetic-id", 19).is_err());
        }
    }
}
