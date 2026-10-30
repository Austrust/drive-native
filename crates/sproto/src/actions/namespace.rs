//! Native namespace contracts from the official handlers and lab wire samples.
//! Replies and observed desired states are separate from proof of request identity.
//! Never delete a rename destination or replay an uncertain mutation here.
#[cfg(any(feature = "protocol-research", test))]
use super::list::parse_node_info;
use super::{
    list::{FileType, NodeInfo},
    native::{EventBatch, EventKind, NativeEvent, NativeVersion, Watch, validate_path},
};
#[cfg(feature = "protocol-research")]
use crate::frame;
use crate::{Client, Error, Result, pstream::PObject};

#[derive(Clone, Debug)]
pub struct NamespaceTarget {
    pub view_id: u64,
    pub path: String,
    pub file_id: String,
    pub base: NativeVersion,
    pub file_type: FileType,
}
impl NamespaceTarget {
    pub fn from_node(view_id: u64, path: String, node: &NodeInfo) -> Result<Self> {
        let value = Self {
            view_id,
            path,
            file_id: node.file_id.clone(),
            base: NativeVersion {
                sync_id: node.sync_id,
                max_id: node.max_id,
            },
            file_type: node.file_type,
        };
        value.validate()?;
        if node.is_removed || value.path.rsplit('/').next() != Some(node.name.as_str()) {
            return Err(Error::InvalidConfig(
                "namespace target does not match live metadata".into(),
            ));
        }
        Ok(value)
    }
    fn validate(&self) -> Result<()> {
        leaf_path(&self.path)?;
        if self.view_id == 0
            || self.file_id.is_empty()
            || !self.file_id.bytes().all(|b| b.is_ascii_digit())
            || self.base.sync_id == 0
            || self.base.max_id == 0
            || self.file_type == FileType::Symlink
        {
            return Err(Error::InvalidConfig(
                "invalid namespace identity or base".into(),
            ));
        }
        Ok(())
    }
}
fn leaf_path(path: &str) -> Result<()> {
    validate_path(path)?;
    if path == "/" {
        return Err(Error::InvalidConfig("cannot mutate Drive root".into()));
    }
    Ok(())
}
fn rename_fields(target: &NamespaceTarget, destination: &str) -> Result<PObject> {
    target.validate()?;
    leaf_path(destination)?;
    if destination == target.path || destination.starts_with(&format!("{}/", target.path)) {
        return Err(Error::InvalidConfig("invalid rename destination".into()));
    }
    Ok(
        pmap! {"view_id"=>target.view_id,"src_path"=>target.path.as_str(),
        "dst_path"=>destination,"sync_id"=>target.base.sync_id,"max_id"=>target.base.max_id},
    )
}
#[cfg(any(feature = "protocol-research", test))]
fn remove_fields(target: &NamespaceTarget) -> Result<PObject> {
    target.validate()?;
    Ok(
        pmap! {"view_id"=>target.view_id,"path"=>target.path.as_str(),
        "sync_id"=>target.base.sync_id,"max_id"=>target.base.max_id},
    )
}

#[cfg(any(feature = "protocol-research", test))]
fn remove_identity_fields(target: &NamespaceTarget) -> Result<PObject> {
    target.validate()?;
    let path = super::transfer::identity_upload_path(&target.file_id, None)?;
    Ok(pmap! {"path"=>path, "sync_id"=>target.base.sync_id, "max_id"=>target.base.max_id})
}

#[derive(Debug)]
pub struct DirectoryReceipt {
    pub file_id: String,
    pub node_id: u64,
    pub version: NativeVersion,
}
/// The remove reply contains no tombstone or revision. The event channel must
/// confirm the original identity disappeared before the journal is settled.
#[derive(Debug)]
pub struct RemoveAccepted {
    _private: (),
}

#[derive(Debug)]
pub enum RenameReconciliation {
    /// The same identity is now at the destination and absent at the old path.
    /// This is a read observation, not attribution to a particular request.
    DesiredStateObserved(NodeInfo),
    Diverged {
        source: Option<NodeInfo>,
        destination: Option<NodeInfo>,
    },
    ChangedDuringVerification,
}
#[derive(Debug)]
pub enum RemoveReconciliation {
    /// A post-base tombstone for this identity was observed. A replacement at
    /// the old path is independent and MUST NOT be removed by a replay.
    TombstoneObserved {
        event: NativeEvent,
        replacement: Option<NodeInfo>,
    },
    StillPresent(NodeInfo),
    /// Path absence or replacement alone is not enough: it may have been moved.
    AwaitingEvidence,
    HistoryGap,
    ChangedDuringVerification,
}

fn same_node(a: &NodeInfo, b: &NodeInfo) -> bool {
    a.file_id == b.file_id
        && a.node_id == b.node_id
        && a.sync_id == b.sync_id
        && a.max_id == b.max_id
        && a.name == b.name
        && a.file_type == b.file_type
        && a.file_size == b.file_size
        && a.hash == b.hash
        && a.mtime == b.mtime
        && a.is_removed == b.is_removed
}
fn same_optional(a: &Option<NodeInfo>, b: &Option<NodeInfo>) -> bool {
    match (a, b) {
        (None, None) => true,
        (Some(a), Some(b)) => same_node(a, b),
        _ => false,
    }
}
fn classify_rename(
    target: &NamespaceTarget,
    source: Option<NodeInfo>,
    destination: Option<NodeInfo>,
) -> RenameReconciliation {
    if source.is_none()
        && destination.as_ref().is_some_and(|n| {
            n.file_id == target.file_id
                && n.file_type == target.file_type
                && !n.is_removed
                && n.sync_id > target.base.sync_id
                && n.max_id >= target.base.max_id
        })
    {
        return RenameReconciliation::DesiredStateObserved(destination.unwrap());
    }
    RenameReconciliation::Diverged {
        source,
        destination,
    }
}
fn classify_remove(
    target: &NamespaceTarget,
    watch: &Watch,
    batch: EventBatch,
    current: Option<NodeInfo>,
) -> RemoveReconciliation {
    if batch.recovery_required {
        return RemoveReconciliation::HistoryGap;
    }
    if let Some(n) = current.as_ref().filter(|n| n.file_id == target.file_id) {
        return RemoveReconciliation::StillPresent(n.clone());
    }
    // Historical modify/move entries may carry current tombstone metadata;
    // only an actual remove event is sufficient evidence for this operation.
    let event = batch
        .events
        .into_iter()
        .filter(|e| {
            e.file_id == target.file_id
                && e.kind == EventKind::Remove
                && e.removed
                && e.file_type == target.file_type
                // Identity removal follows the original object when its parent
                // moves. Require a valid post-base tombstone inside this watch,
                // rather than the obsolete display path. Restored identities
                // carry removed=false and cannot be settled by historical data.
                && validate_path(&e.path).is_ok()
                && (watch.path == "/" || e.path.starts_with(&format!("{}/", watch.path)))
                && e.version.sync_id > target.base.sync_id
                && e.version.sync_id > watch.cursor.0
                && e.version.max_id >= target.base.max_id
        })
        .max_by_key(|e| e.version.sync_id);
    match event {
        Some(event) => RemoveReconciliation::TombstoneObserved {
            event,
            replacement: current,
        },
        None => RemoveReconciliation::AwaitingEvidence,
    }
}
fn validate_watch(target: &NamespaceTarget, watch: &Watch) -> Result<()> {
    target.validate()?;
    validate_path(&watch.path)?;
    if target.view_id != watch.view_id
        || watch.node_id == 0
        || !(watch.path == "/" || target.path.starts_with(&format!("{}/", watch.path)))
    {
        return Err(Error::InvalidConfig(
            "remove reconciliation watch is outside target scope".into(),
        ));
    }
    Ok(())
}
#[cfg(any(feature = "protocol-research", test))]
fn text(r: &PObject, key: &str) -> Result<String> {
    r.get(key)
        .and_then(PObject::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Decode(format!("missing namespace string: {key}")))
}
#[cfg(any(feature = "protocol-research", test))]
fn number(r: &PObject, key: &str) -> Result<u64> {
    r.get(key)
        .and_then(PObject::as_int)
        .ok_or_else(|| Error::Decode(format!("missing namespace integer: {key}")))
}
#[cfg(any(feature = "protocol-research", test))]
fn directory_receipt(r: &PObject, path: &str) -> Result<DirectoryReceipt> {
    if text(r, "file_type")? != "dir" || number(r, "is_removed")? != 0 || text(r, "path")? != path {
        return Err(Error::Decode(
            "mkdir acknowledgement mismatch; reconcile before retry".into(),
        ));
    }
    let receipt = DirectoryReceipt {
        file_id: text(r, "file_id")?,
        node_id: number(r, "node_id")?,
        version: NativeVersion {
            sync_id: number(r, "sync_id")?,
            max_id: number(r, "max_id")?,
        },
    };
    if receipt.file_id.is_empty()
        || !receipt.file_id.bytes().all(|b| b.is_ascii_digit())
        || receipt.node_id == 0
        || receipt.version.sync_id == 0
        || receipt.version.max_id == 0
    {
        return Err(Error::Decode(
            "invalid mkdir identity/version; reconcile before retry".into(),
        ));
    }
    Ok(receipt)
}
#[cfg(any(feature = "protocol-research", test))]
fn rename_receipt(r: &PObject, target: &NamespaceTarget, destination: &str) -> Result<NodeInfo> {
    let node = parse_node_info(r)?;
    if text(r, "path")? != destination
        || number(r, "is_removed")? != 0
        || node.name != destination.rsplit('/').next().unwrap_or_default()
        || node.file_id != target.file_id
        || node.file_type != target.file_type
        || node.sync_id <= target.base.sync_id
        || node.max_id < target.base.max_id
    {
        return Err(Error::Decode(
            "rename acknowledgement mismatch; reconcile before retry".into(),
        ));
    }
    Ok(node)
}

impl Client {
    /// Read only; callers retain their durable operation on divergence/uncertainty.
    /// Double metadata reads detect observed races; they are not a NAS transaction.
    pub async fn reconcile_rename(
        &self,
        target: &NamespaceTarget,
        destination: &str,
    ) -> Result<RenameReconciliation> {
        rename_fields(target, destination)?;
        let source = self.current_path(target.view_id, &target.path).await?;
        let dest = self.current_path(target.view_id, destination).await?;
        let source_after = self.current_path(target.view_id, &target.path).await?;
        let dest_after = self.current_path(target.view_id, destination).await?;
        if !same_optional(&source, &source_after) || !same_optional(&dest, &dest_after) {
            return Ok(RenameReconciliation::ChangedDuringVerification);
        }
        Ok(classify_rename(target, source, dest))
    }
    /// Use the persisted pre-operation watch cursor. Returned evidence does not
    /// advance the service cursor; the normal event inbox owns that transaction.
    pub async fn reconcile_remove(
        &self,
        target: &NamespaceTarget,
        watch: &Watch,
    ) -> Result<RemoveReconciliation> {
        validate_watch(target, watch)?;
        let before = self.current_path(target.view_id, &target.path).await?;
        let batch = self.pull_events(watch).await?;
        let after = self.current_path(target.view_id, &target.path).await?;
        if !same_optional(&before, &after) {
            return Ok(RemoveReconciliation::ChangedDuringVerification);
        }
        Ok(classify_remove(target, watch, batch, after))
    }
    #[cfg(feature = "protocol-research")]
    async fn namespace_request(
        &self,
        action: &str,
        fields: PObject,
        drop_reply: bool,
    ) -> Result<PObject> {
        super::namespace_observer::before_send(super::namespace_observer::NamespaceBoundary {
            action: action.into(),
            view: fields.get("view_id").and_then(PObject::as_int),
            path: fields
                .get("path")
                .and_then(PObject::as_str)
                .map(str::to_owned),
            source: fields
                .get("src_path")
                .and_then(PObject::as_str)
                .map(str::to_owned),
            destination: fields
                .get("dst_path")
                .and_then(PObject::as_str)
                .map(str::to_owned),
        })
        .await?;
        let request = self.build_request(action, fields);
        let mut guard = self.pool.acquire().await?;
        let result = async {
            if drop_reply {
                guard.channel().send(frame::SCMD_REQUEST, &request).await?;
                return Err(Error::Timeout(
                    "injected lost namespace reply; reconcile before retry",
                ));
            }
            guard.channel().request(frame::SCMD_REQUEST, &request).await
        }
        .await;
        guard.poison_on_err(result)
    }
    /// Explicit experiment: test existing native identity-path grammar for both
    /// rename operands. Not selected by the normal coordinator until verified.
    #[cfg(feature = "protocol-research")]
    pub async fn research_rename_identity(
        &self,
        target: &NamespaceTarget,
        parent: &str,
        name: &str,
        include_view: bool,
        source_identity: bool,
    ) -> Result<PObject> {
        target.validate()?;
        let src = if source_identity {
            super::transfer::identity_upload_path(&target.file_id, None)?
        } else {
            target.path.clone()
        };
        let dst = super::transfer::identity_upload_path(parent, Some(name))?;
        let mut fields = pmap! {"src_path"=>src,"dst_path"=>dst,"sync_id"=>target.base.sync_id,"max_id"=>target.base.max_id};
        if include_view {
            fields
                .as_map_mut()
                .unwrap()
                .insert("view_id".into(), target.view_id.into());
        }
        self.namespace_request("rename", fields, false).await
    }
    /// Native rename rejects an existing destination. No remove/create fallback.
    /// Persist intent before sending; any transport/decoding failure is uncertain.
    #[cfg(feature = "protocol-research")]
    pub async fn rename_native(
        &self,
        target: &NamespaceTarget,
        destination: &str,
    ) -> Result<NodeInfo> {
        let r = self
            .namespace_request("rename", rename_fields(target, destination)?, false)
            .await?;
        rename_receipt(&r, target, destination)
    }
    /// The destination parent is another persistent identity; a path match alone
    /// must not acknowledge a move into a concurrently replaced directory.
    #[cfg(feature = "protocol-research")]
    pub async fn rename_native_checked_parent(
        &self,
        target: &NamespaceTarget,
        destination: &str,
        parent_file_id: &str,
    ) -> Result<NodeInfo> {
        if parent_file_id.is_empty() || !parent_file_id.bytes().all(|c| c.is_ascii_digit()) {
            return Err(Error::InvalidConfig(
                "invalid destination parent identity".into(),
            ));
        }
        let response = self
            .namespace_request("rename", rename_fields(target, destination)?, false)
            .await?;
        let node = rename_receipt(&response, target, destination)?;
        if text(&response, "parent_id")? != parent_file_id {
            return Err(Error::Decode(
                "rename destination parent changed; reconcile before retry".into(),
            ));
        }
        Ok(node)
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_rename_drop_receipt(
        &self,
        target: &NamespaceTarget,
        destination: &str,
    ) -> Result<()> {
        self.namespace_request("rename", rename_fields(target, destination)?, true)
            .await?;
        unreachable!("fault injection never returns a receipt")
    }
    /// Official remove identity addressing. Keep source version conditions;
    /// acceptance alone never settles the persisted removal journal.
    #[cfg(feature = "protocol-research")]
    pub async fn remove_at_identity(&self, target: &NamespaceTarget) -> Result<RemoveAccepted> {
        let response = self
            .namespace_request("remove", remove_identity_fields(target)?, false)
            .await?;
        if text(&response, "type")? != "response" {
            return Err(Error::Decode(
                "invalid identity remove reply; reconcile before retry".into(),
            ));
        }
        Ok(RemoveAccepted { _private: () })
    }
    /// Explicit fault injection: wait for an independent native observer before
    /// dropping the unread response. Never return this connection to the pool.
    #[cfg(feature = "protocol-research")]
    pub async fn research_remove_identity_unread_after<F>(
        &self,
        target: &NamespaceTarget,
        observed: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = Result<()>>,
    {
        let request = self.build_request("remove", remove_identity_fields(target)?);
        let mut guard = self.pool.acquire().await?;
        guard.channel().send(frame::SCMD_REQUEST, &request).await?;
        tokio::time::timeout(std::time::Duration::from_secs(30), observed)
            .await
            .map_err(|_| Error::Timeout("identity remove observation timed out"))??;
        guard.poison_on_err(Err(Error::Timeout("injected unread identity remove reply")))
    }
    /// This primitive may remove a subtree for directories. The synchronization
    /// coordinator must validate and journal descendant operations first.
    #[cfg(feature = "protocol-research")]
    pub async fn remove_native(&self, target: &NamespaceTarget) -> Result<RemoveAccepted> {
        let response = self
            .namespace_request("remove", remove_fields(target)?, false)
            .await?;
        if text(&response, "type")? != "response" {
            return Err(Error::Decode(
                "invalid remove reply; reconcile before retry".into(),
            ));
        }
        Ok(RemoveAccepted { _private: () })
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_remove_drop_receipt(&self, target: &NamespaceTarget) -> Result<()> {
        self.namespace_request("remove", remove_fields(target)?, true)
            .await?;
        unreachable!("fault injection never returns a receipt")
    }
    /// Fault injection at the worker/coordinator boundary: the server reply is
    /// received, then deliberately hidden from the caller. This deterministically
    /// exercises committed-but-unconfirmed recovery. Immediate socket close in
    /// research_rename_drop_receipt may instead cancel before the server commits.
    #[cfg(feature = "protocol-research")]
    pub async fn research_rename_discard_ack(
        &self,
        target: &NamespaceTarget,
        destination: &str,
    ) -> Result<()> {
        let _receipt = self.rename_native(target, destination).await?;
        Err(Error::Timeout(
            "injected lost coordinator rename acknowledgement",
        ))
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_remove_discard_ack(&self, target: &NamespaceTarget) -> Result<()> {
        let _receipt = self.remove_native(target).await?;
        Err(Error::Timeout(
            "injected lost coordinator remove acknowledgement",
        ))
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_mkdir_discard_reply(&self, view_id: u64, path: &str) -> Result<()> {
        leaf_path(path)?;
        if view_id == 0 {
            return Err(Error::InvalidConfig("missing view identity".into()));
        }
        self.namespace_request(
            "upload",
            pmap! {"view_id"=>view_id,"path"=>path,
            "sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir",
            "conflict_policy"=>"overwrite_client"},
            true,
        )
        .await?;
        Ok(())
    }
    /// Research-only fault barrier: never read this request's response. A separate
    /// authenticated session must observe the commit before this socket is dropped.
    /// Timeout or cancellation also discards the unread channel, never reuses it.
    #[cfg(feature = "protocol-research")]
    pub async fn research_mkdir_discard_reply_after<F>(
        &self,
        view_id: u64,
        path: &str,
        independently_observed: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = Result<()>>,
    {
        leaf_path(path)?;
        if view_id == 0 {
            return Err(Error::InvalidConfig("missing view identity".into()));
        }
        let request = self.build_request(
            "upload",
            pmap! {"view_id"=>view_id,"path"=>path,
            "sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir",
            "conflict_policy"=>"overwrite_client"},
        );
        let mut guard = self.pool.acquire().await?;
        guard.channel().send(frame::SCMD_REQUEST, &request).await?;
        tokio::time::timeout(std::time::Duration::from_secs(30), independently_observed)
            .await
            .map_err(|_| Error::Timeout("independent mkdir observation timed out"))??;
        guard.poison_on_err(Err(Error::Timeout(
            "injected unread mkdir reply after independent commit observation",
        )))
    }
    /// Same official upload identity addressing used for files; caller still
    /// proves creation ownership through persisted native event provenance.
    #[cfg(feature = "protocol-research")]
    pub async fn mkdir_at_identity(
        &self,
        parent: &str,
        name: &str,
        expected_path: &str,
    ) -> Result<DirectoryReceipt> {
        leaf_path(expected_path)?;
        if expected_path.rsplit('/').next() != Some(name) {
            return Err(Error::InvalidConfig("directory leaf mismatch".into()));
        }
        let path = super::transfer::identity_upload_path(parent, Some(name))?;
        let r=self.namespace_request("upload",pmap! {"path"=>path,"sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir","conflict_policy"=>"overwrite_client"},false).await?;
        directory_receipt(&r, expected_path)
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_mkdir_identity_discard_reply(
        &self,
        parent: &str,
        name: &str,
    ) -> Result<()> {
        let path = super::transfer::identity_upload_path(parent, Some(name))?;
        self.namespace_request("upload",pmap! {"path"=>path,"sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir","conflict_policy"=>"overwrite_client"},true).await?;
        unreachable!("unread reply cannot be confirmed")
    }
    #[cfg(feature = "protocol-research")]
    pub async fn research_mkdir_identity_discard_reply_after<F>(
        &self,
        parent: &str,
        name: &str,
        observed: F,
    ) -> Result<()>
    where
        F: std::future::Future<Output = Result<()>>,
    {
        let path = super::transfer::identity_upload_path(parent, Some(name))?;
        let request=self.build_request("upload",pmap! {"path"=>path,"sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir","conflict_policy"=>"overwrite_client"});
        let mut guard = self.pool.acquire().await?;
        guard.channel().send(frame::SCMD_REQUEST, &request).await?;
        tokio::time::timeout(std::time::Duration::from_secs(30), observed)
            .await
            .map_err(|_| Error::Timeout("identity directory observation timed out"))??;
        guard.poison_on_err(Err(Error::Timeout(
            "injected unread identity directory reply",
        )))
    }
    /// Directory creation uses the official native upload action. On the tested
    /// server an existing directory returns its unchanged identity/version; this
    /// receipt MUST NOT be interpreted as proof that a new directory was created.
    /// Existing files must be independently rejected before any identity binding.
    #[cfg(feature = "protocol-research")]
    pub async fn mkdir_native(&self, view_id: u64, path: &str) -> Result<DirectoryReceipt> {
        leaf_path(path)?;
        if view_id == 0 {
            return Err(Error::InvalidConfig("missing view identity".into()));
        }
        let r = self
            .namespace_request(
                "upload",
                pmap! {"view_id"=>view_id,"path"=>path,
                "sync_id"=>0u64,"max_id"=>0u64,"is_dir"=>true,"file_type"=>"dir",
                "conflict_policy"=>"overwrite_client"},
                false,
            )
            .await?;
        directory_receipt(&r, path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn node() -> NodeInfo {
        NodeInfo {
            node_id: 4,
            sync_id: 10,
            max_id: 10,
            name: "a".into(),
            file_size: 3,
            mtime: 1,
            hash: "hash".into(),
            file_id: "99".into(),
            file_type: FileType::File,
            is_removed: false,
        }
    }
    fn target() -> NamespaceTarget {
        NamespaceTarget::from_node(54, "/lab/a".into(), &node()).unwrap()
    }
    fn watch() -> Watch {
        Watch {
            view_id: 54,
            node_id: 3,
            path: "/lab".into(),
            cursor: super::super::native::EventCursor(10),
        }
    }
    fn batch() -> EventBatch {
        EventBatch {
            events: vec![NativeEvent {
                committer: None,
                kind: EventKind::Remove,
                path: "/lab/a".into(),
                file_id: "99".into(),
                parent_id: "98".into(),
                version: NativeVersion {
                    sync_id: 11,
                    max_id: 10,
                },
                file_type: FileType::File,
                size: 3,
                hash: "hash".into(),
                mtime: 1,
                removed: true,
            }],
            next: super::super::native::EventCursor(11),
            recovery_required: false,
        }
    }
    fn object(v: serde_json::Value) -> PObject {
        match v {
            serde_json::Value::Null => PObject::Null,
            serde_json::Value::Bool(b) => b.into(),
            serde_json::Value::Number(n) => n.as_u64().unwrap().into(),
            serde_json::Value::String(s) => s.into(),
            serde_json::Value::Array(a) => PObject::Array(a.into_iter().map(object).collect()),
            serde_json::Value::Object(m) => {
                PObject::Map(m.into_iter().map(|(k, v)| (k, object(v))).collect())
            }
        }
    }
    #[test]
    fn replay_real_namespace_receipts_and_conditional_errors() {
        let mut observed = [0; 4];
        for line in include_str!("../../../../evidence/native-live.jsonl").lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            let r = object(v["response"].clone());
            match v["stage"].as_str().unwrap() {
                "create_lab" if v["response"]["error"].is_null() => {
                    directory_receipt(&r, v["request"]["path"].as_str().unwrap()).unwrap();
                    observed[0] += 1;
                }
                "rename" if v["response"]["error"].is_null() => {
                    let q = &v["request"];
                    let target = NamespaceTarget {
                        view_id: q["view_id"].as_u64().unwrap(),
                        path: q["src_path"].as_str().unwrap().into(),
                        file_id: v["response"]["file_id"].as_str().unwrap().into(),
                        base: NativeVersion {
                            sync_id: q["sync_id"].as_u64().unwrap(),
                            max_id: q["max_id"].as_u64().unwrap(),
                        },
                        file_type: FileType::File,
                    };
                    rename_receipt(&r, &target, q["dst_path"].as_str().unwrap()).unwrap();
                    observed[1] += 1;
                }
                "remove" if v["response"]["error"].is_null() => {
                    assert_eq!(text(&r, "type").unwrap(), "response");
                    assert!(r.get("file_id").is_none());
                    assert!(r.get("sync_id").is_none());
                    observed[2] += 1;
                }
                "remove" | "rename" => {
                    assert!(matches!(
                        crate::error::check_server_error(&r),
                        Err(Error::Server {
                            code: 40962 | 36872,
                            ..
                        })
                    ));
                    observed[3] += 1;
                }
                _ => {}
            }
        }
        assert_eq!(observed, [2, 1, 1, 2]);
    }
    #[test]
    fn invalid_namespace_never_reaches_wire() {
        for bad in ["/", "/lab/../a", "relative", "/lab//b", "/lab/\0"] {
            let mut t = target();
            t.path = bad.into();
            assert!(remove_fields(&t).is_err());
            assert!(rename_fields(&target(), bad).is_err());
        }
        for bad in ["/lab/a", "/lab/a/sub"] {
            assert!(rename_fields(&target(), bad).is_err());
        }
        let mut t = target();
        t.base.sync_id = 0;
        assert!(remove_fields(&t).is_err());
        t = target();
        t.file_id = "local:1".into();
        assert!(remove_fields(&t).is_err());
        assert!(NamespaceTarget::from_node(54, "/lab/b".into(), &node()).is_err());
    }
    #[test]
    fn path_absence_and_historical_payload_are_not_delete_confirmation() {
        let mut b = batch();
        b.events.clear();
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::AwaitingEvidence
        ));
        let mut b = batch();
        b.events[0].kind = EventKind::Modify;
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::AwaitingEvidence
        ));
        let mut b = batch();
        b.events[0].file_id = "100".into();
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::AwaitingEvidence
        ));
        let mut b = batch();
        b.events[0].version.sync_id = 10;
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::AwaitingEvidence
        ));
        let mut b = batch();
        b.recovery_required = true;
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::HistoryGap
        ));
    }
    #[test]
    fn deletion_can_preserve_replacement_and_cannot_hide_restored_identity() {
        let mut replacement = node();
        replacement.file_id = "100".into();
        assert!(
            matches!(classify_remove(&target(),&watch(),batch(),Some(replacement)),RemoveReconciliation::TombstoneObserved{replacement:Some(n),..} if n.file_id=="100")
        );
        assert!(matches!(
            classify_remove(&target(), &watch(), batch(), Some(node())),
            RemoveReconciliation::StillPresent(_)
        ));
    }
    #[test]
    fn rename_requires_original_identity_at_new_path_and_no_old_entry() {
        let mut moved = node();
        moved.name = "b".into();
        moved.sync_id = 11;
        moved.max_id = 11;
        assert!(matches!(
            classify_rename(&target(), None, Some(moved.clone())),
            RenameReconciliation::DesiredStateObserved(_)
        ));
        assert!(matches!(
            classify_rename(&target(), Some(node()), Some(moved.clone())),
            RenameReconciliation::Diverged { .. }
        ));
        moved.file_id = "100".into();
        assert!(matches!(
            classify_rename(&target(), None, Some(moved)),
            RenameReconciliation::Diverged { .. }
        ));
    }
    #[test]
    fn identity_delete_reconciliation_follows_parent_move_but_stays_in_watch() {
        let mut b = batch();
        b.events[0].path = "/lab/moved/a".into();
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::TombstoneObserved { .. }
        ));
        for path in ["/lab2/a", "/other/a", "/lab", "/lab/../a"] {
            let mut b = batch();
            b.events[0].path = path.into();
            assert!(matches!(
                classify_remove(&target(), &watch(), b, None),
                RemoveReconciliation::AwaitingEvidence
            ));
        }
        let mut b = batch();
        b.events[0].path = "/lab/moved/a".into();
        b.events[0].removed = false;
        assert!(matches!(
            classify_remove(&target(), &watch(), b, None),
            RemoveReconciliation::AwaitingEvidence
        ));
    }
    #[test]
    fn identity_remove_keeps_conditions_and_rejects_invalid_ids() {
        let f = remove_identity_fields(&target()).unwrap();
        assert_eq!(text(&f, "path").unwrap(), "id:99");
        assert_eq!(number(&f, "sync_id").unwrap(), 10);
        assert_eq!(number(&f, "max_id").unwrap(), 10);
        assert!(f.get("view_id").is_none());
        for id in ["0", "18446744073709551616", "99/a", "", "local:1"] {
            let mut t = target();
            t.file_id = id.into();
            assert!(remove_identity_fields(&t).is_err());
        }
    }
    #[test]
    fn reconciliation_watch_must_cover_target() {
        validate_watch(&target(), &watch()).unwrap();
        let mut w = watch();
        w.path = "/lab2".into();
        assert!(validate_watch(&target(), &w).is_err());
        w = watch();
        w.view_id += 1;
        assert!(validate_watch(&target(), &w).is_err());
        w = watch();
        w.path = "/".into();
        validate_watch(&target(), &w).unwrap();
    }
    #[test]
    fn incomplete_or_replaced_receipts_are_uncertain() {
        let mut r = pmap! {"path"=>"/lab/b","name"=>"b","file_hash"=>"hash","mtime"=>1u64,
        "node_id"=>4u64,"sync_id"=>11u64,"max_id"=>11u64,"file_id"=>"99",
        "file_size"=>3u64,"file_type"=>"file","is_removed"=>0u64};
        rename_receipt(&r, &target(), "/lab/b").unwrap();
        r.as_map_mut()
            .unwrap()
            .insert("file_id".into(), PObject::from("100"));
        assert!(rename_receipt(&r, &target(), "/lab/b").is_err());
        assert!(directory_receipt(&r, "/lab/b").is_err());
        r.as_map_mut()
            .unwrap()
            .insert("file_type".into(), PObject::from("dir"));
        directory_receipt(&r, "/lab/b").unwrap();
        r.as_map_mut().unwrap().remove("is_removed");
        assert!(directory_receipt(&r, "/lab/b").is_err());
    }
}
