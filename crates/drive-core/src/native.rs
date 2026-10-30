//! Bridge from verified native contracts to the durable local state engine.
//! Research-gated writes cover file creation and content updates. Namespace
//! operations are journaled; complete bootstrap recovery remains a gate.
use crate::{Batch, Change, Error, Event, Node, Result, Store, backend::DownloadedFile};
use serde::{Deserialize, Serialize};
use sproto::{
    Client,
    actions::{
        list::FileType,
        native::{EventBatch, EventCursor, EventKind, NativeVersion, Watch},
        transfer::ContentVersion,
    },
};
use std::{fs::File, path::Path};

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub struct Scope {
    pub view_id: u64,
    pub node_id: u64,
    pub file_id: String,
    pub path: String,
}
impl Scope {
    fn validate(&self) -> Result<()> {
        if self.file_id.is_empty()
            || !self.path.starts_with('/')
            || self.path.contains('\0')
            || (self.path != "/"
                && self.path[1..]
                    .split('/')
                    .any(|p| matches!(p, "" | "." | "..")))
        {
            return Err(Error::State("invalid native sync scope".into()));
        }
        Ok(())
    }
    fn contains(&self, path: &str) -> bool {
        self.path == "/"
            || path == self.path
            || path
                .strip_prefix(&self.path)
                .is_some_and(|s| s.starts_with('/'))
    }
}

/// Serialized into Node.version so hashes/versions survive worker restart.
/// Its contents are private adapter metadata, not invented wire fields.
#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct VersionToken {
    schema: u8,
    restore: String,
    view: u64,
    root: String,
    file: String,
    sync: u64,
    max: u64,
    size: u64,
    hash: String,
    directory: bool,
}

pub struct NativeReadSession {
    client: Client,
    scope: Scope,
    root_hint: std::sync::atomic::AtomicU64,
    root_lost: std::sync::atomic::AtomicBool,
}
impl NativeReadSession {
    pub fn new(client: Client, scope: Scope) -> Result<Self> {
        scope.validate()?;
        Ok(Self {
            client,
            scope,
            root_hint: 0.into(),
            root_lost: false.into(),
        })
    }
    pub fn bind(&self, store: &mut Store) -> Result<()> {
        store.bind_native_scope(
            self.client.restore_identity(),
            &serde_json::to_string(&self.scope)?,
        )
    }
    pub fn watch(&self, cursor: &str) -> Result<Watch> {
        let cursor = if cursor.is_empty() {
            0
        } else {
            cursor
                .strip_prefix("native-v1:")
                .and_then(|s| s.parse().ok())
                .ok_or_else(|| Error::State("invalid native cursor; recovery required".into()))?
        };
        Ok(Watch {
            view_id: self.scope.view_id,
            node_id: self.scope.node_id,
            path: self.scope.path.clone(),
            cursor: EventCursor(cursor),
        })
    }
    /// None means no advancement, not an empty directory snapshot.
    pub async fn pull(&self, cursor: &str) -> Result<Option<Batch>> {
        let watch = self.watch(cursor)?;
        let batch = self.scoped_events(&watch).await?;
        translate(&self.scope, self.client.restore_identity(), cursor, batch)
    }
    pub async fn wait(
        &self,
        cursor: &str,
        digest: &str,
    ) -> Result<sproto::actions::native::Notification> {
        if self.root_lost.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::RootUnavailable);
        }
        let result = self
            .client
            .wait_changes(&[self.watch(cursor)?], digest)
            .await
            .map_err(|error| match error {
                sproto::Error::Server { code: 12291, .. } => Error::RootUnavailable,
                error => Error::Native(error),
            });
        self.root_guard_result(result)
    }
    /// Caller imports the descriptor on the fsd state owner, off the FUSE loop.
    pub async fn download(&self, node: &Node, staging: &Path) -> Result<DownloadedFile> {
        let token: VersionToken = serde_json::from_str(&node.version)?;
        if token.schema != 1
            || token.restore != self.client.restore_identity()
            || token.view != self.scope.view_id
            || token.root != self.scope.file_id
            || (!native_file_id(&token.file))
            || token.directory
            || node.directory
        {
            return Err(Error::State(
                "download token does not belong to this file/scope/server".into(),
            ));
        }
        let temp = tempfile::tempdir_in(staging)?;
        let path = temp.path().join("complete");
        let expected = ContentVersion {
            file_id: token.file,
            version: NativeVersion {
                sync_id: token.sync,
                max_id: token.max,
            },
            size: token.size,
            hash: token.hash,
        };
        let verified = self.client.download_checked(&expected, &path).await?;
        // On Linux unlinking the temporary pathname retains this open descriptor.
        let file = File::open(&path)?;
        Ok(DownloadedFile {
            file,
            identity: node.id.clone(),
            version: node.version.clone(),
            size: verified.remote.size,
            sha256: verified.sha256,
        })
    }
}

fn translate(scope: &Scope, restore: &str, from: &str, batch: EventBatch) -> Result<Option<Batch>> {
    if batch.recovery_required {
        return Err(Error::Unsupported("native event recovery/rescan signal"));
    }
    let to = format!("native-v1:{}", batch.next.0);
    if to == from {
        return Ok(None);
    }
    let mut events = Vec::new();
    for e in batch.events {
        root_guard::root_event(scope, &e)?;
        let (path, parent) = match &e.kind {
            EventKind::Move {
                destination,
                parent_id,
            } => (destination, parent_id),
            _ => (&e.path, &e.parent_id),
        };
        if !scope.contains(&e.path) && !scope.contains(path) {
            return Err(Error::State("event outside selected native subtree".into()));
        }
        let change = if e.removed || matches!(e.kind, EventKind::Remove) || !scope.contains(path) {
            Change::Delete {
                id: e.file_id.clone(),
            }
        } else {
            if e.file_type == FileType::Symlink {
                return Err(Error::Unsupported("native symlink target contract"));
            }
            let root = e.file_id == scope.file_id;
            if root != (path == &scope.path) {
                return Err(Error::State("native sync root identity changed".into()));
            }
            let token = VersionToken {
                schema: 1,
                restore: restore.into(),
                view: scope.view_id,
                root: scope.file_id.clone(),
                file: e.file_id.clone(),
                sync: e.version.sync_id,
                max: e.version.max_id,
                size: e.size,
                hash: e.hash,
                directory: e.file_type == FileType::Dir,
            };
            Change::Upsert(Node {
                id: e.file_id.clone(),
                parent: if root { None } else { Some(parent.clone()) },
                name: if path == "/" {
                    "My Drive".into()
                } else {
                    path.rsplit('/').next().unwrap().into()
                },
                version: serde_json::to_string(&token)?,
                directory: token.directory,
                metadata: Some(crate::NodeMetadata {
                    size: e.size,
                    mtime: e.mtime,
                    mode: if token.directory { 0o755 } else { 0o644 },
                }),
            })
        };
        // Real historical events can carry current tombstone metadata. Hash the
        // normalized change rather than treating sync_id as an immutable event ID.
        let digest = crate::sha256(&serde_json::to_vec(&change)?);
        events.push(Event {
            id: format!("native:{}:{}:{digest}", e.version.sync_id, e.file_id),
            change,
        });
    }
    Ok(Some(Batch {
        from: from.into(),
        to,
        events,
    }))
}

#[cfg(test)]
mod tests {
    use super::*;
    use sproto::actions::native::NativeEvent;
    pub(super) fn scope() -> Scope {
        Scope {
            view_id: 54,
            node_id: 3,
            file_id: "root".into(),
            path: "/lab/run".into(),
        }
    }
    pub(super) fn event() -> NativeEvent {
        NativeEvent {
            committer: None,
            kind: EventKind::Modify,
            path: "/lab/run/A.doc".into(),
            file_id: "a".into(),
            parent_id: "root".into(),
            version: NativeVersion {
                sync_id: 5,
                max_id: 5,
            },
            file_type: FileType::File,
            size: 3,
            hash: "0123456789abcdef0123456789abcdef".into(),
            mtime: 0,
            removed: false,
        }
    }
    pub(super) fn batch(event: NativeEvent, next: u64) -> EventBatch {
        EventBatch {
            events: vec![event],
            next: EventCursor(next),
            recovery_required: false,
        }
    }
    #[test]
    fn native_events_are_durable_and_replayed_tombstones_do_not_resurrect() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        let b = translate(&scope(), "server", "", batch(event(), 5))
            .unwrap()
            .unwrap();
        store.receive(&b).unwrap();
        drop(store);
        let mut store = Store::open(d.path()).unwrap();
        store.drain().unwrap();
        assert_eq!(store.node("a").unwrap().name, "A.doc");
        let mut later = event();
        later.removed = true;
        let b = translate(&scope(), "server", "native-v1:5", batch(later, 6))
            .unwrap()
            .unwrap();
        store.receive(&b).unwrap();
        store.drain().unwrap();
        assert_eq!(store.status("a").unwrap(), crate::Status::Deleted);
    }
    #[test]
    fn move_uses_destination_parent_and_leaving_scope_deletes_only_local_index() {
        let mut e = event();
        e.kind = EventKind::Move {
            destination: "/lab/run/sub/B.doc".into(),
            parent_id: "sub".into(),
        };
        let b = translate(&scope(), "s", "", batch(e.clone(), 5))
            .unwrap()
            .unwrap();
        match &b.events[0].change {
            Change::Upsert(n) => {
                assert_eq!(n.parent.as_deref(), Some("sub"));
                assert_eq!(n.name, "B.doc");
            }
            _ => panic!(),
        }
        e.kind = EventKind::Move {
            destination: "/lab/run-other/B.doc".into(),
            parent_id: "other".into(),
        };
        let b = translate(&scope(), "s", "", batch(e, 5)).unwrap().unwrap();
        assert!(matches!(b.events[0].change, Change::Delete { .. }));
    }
    #[test]
    fn unknown_recovery_scope_and_symlinks_fail_closed() {
        let mut b = batch(event(), 5);
        b.recovery_required = true;
        assert!(translate(&scope(), "s", "", b).is_err());
        let mut e = event();
        e.path = "/lab/run-other/a".into();
        assert!(translate(&scope(), "s", "", batch(e, 5)).is_err());
        let mut e = event();
        e.file_type = FileType::Symlink;
        assert!(translate(&scope(), "s", "", batch(e, 5)).is_err());
    }
}

#[cfg(feature = "native-research")]
pub enum NativeWriteOutcome {
    Created { node: Node, remote: String },
    Confirmed(Node),
    Conflict,
    Uncertain,
}
#[cfg(feature = "native-research")]
impl NativeReadSession {
    async fn write_intent(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
    ) -> Result<sproto::actions::mutations::UploadIntent> {
        use sha2::Digest;
        let creating = node.version == crate::creations::CREATE_VERSION;
        if creating && !crate::creations::is_local_identity(&node.id) {
            return Err(Error::State("invalid local creation identity".into()));
        }
        let token: VersionToken = if creating {
            VersionToken {
                schema: 1,
                restore: self.client.restore_identity().into(),
                view: self.scope.view_id,
                root: self.scope.file_id.clone(),
                file: node.id.clone(),
                sync: 0,
                max: 0,
                size: 0,
                hash: String::new(),
                directory: false,
            }
        } else {
            serde_json::from_str(&node.version)?
        };
        if token.schema != 1
            || token.restore != self.client.restore_identity()
            || token.view != self.scope.view_id
            || token.root != self.scope.file_id
            || (!creating && !native_file_id(&token.file))
            || token.directory
            || !self.scope.contains(path)
            || path == self.scope.path
        {
            return Err(Error::State("write identity/scope mismatch".into()));
        }
        let source = source.to_owned();
        let expected = sha256.to_owned();
        let (size, md4, sha256) = tokio::task::spawn_blocking(move || -> Result<_> {
            use std::io::Read;
            let mut f = File::open(source)?;
            if !f.metadata()?.is_file() {
                return Err(Error::State(
                    "upload descriptor is not a regular file".into(),
                ));
            }
            let mut h = sha2::Sha256::new();
            let mut m = md4::Md4::new();
            let mut size = 0;
            let mut buf = [0; 65536];
            loop {
                let n = f.read(&mut buf)?;
                if n == 0 {
                    break;
                }
                h.update(&buf[..n]);
                m.update(&buf[..n]);
                size += n as u64;
            }
            let sha256 = format!("{:x}", h.finalize());
            if sha256 != expected {
                return Err(Error::Integrity);
            }
            Ok((size, format!("{:x}", m.finalize()), sha256))
        })
        .await
        .map_err(|_| Error::State("snapshot verifier stopped".into()))??;
        Ok(sproto::actions::mutations::UploadIntent {
            view_id: token.view,
            path: path.into(),
            original_file_id: if creating { None } else { Some(token.file) },
            base: NativeVersion {
                sync_id: token.sync,
                max_id: token.max,
            },
            size,
            md4,
            sha256,
        })
    }
    fn confirmed_node(&self, node: &Node, content: ContentVersion) -> Result<NativeWriteOutcome> {
        let creating = node.version == crate::creations::CREATE_VERSION;
        let mut token: VersionToken = if creating {
            VersionToken {
                schema: 1,
                restore: self.client.restore_identity().into(),
                view: self.scope.view_id,
                root: self.scope.file_id.clone(),
                file: content.file_id.clone(),
                sync: 0,
                max: 0,
                size: 0,
                hash: String::new(),
                directory: false,
            }
        } else {
            serde_json::from_str(&node.version)?
        };
        if content.file_id != token.file {
            return Err(Error::State("upload changed remote identity".into()));
        }
        token.sync = content.version.sync_id;
        token.max = content.version.max_id;
        token.size = content.size;
        token.hash = content.hash;
        let mut node = node.clone();
        node.version = serde_json::to_string(&token)?;
        if let Some(m) = &mut node.metadata {
            m.size = content.size;
        }
        if creating {
            Ok(NativeWriteOutcome::Created {
                node,
                remote: content.file_id,
            })
        } else {
            Ok(NativeWriteOutcome::Confirmed(node))
        }
    }
    async fn reconcile_intent(
        &self,
        node: &Node,
        intent: &sproto::actions::mutations::UploadIntent,
        staging: &Path,
    ) -> Result<NativeWriteOutcome> {
        use sproto::actions::mutations::UploadReconciliation as R;
        match self.client.reconcile_upload(intent, staging).await? {
            R::DesiredStatePresent(content) => self.confirmed_node(node, content),
            R::Diverged(current)
                if current.sync_id != intent.base.sync_id
                    || current.max_id != intent.base.max_id
                    || Some(&current.file_id) != intent.original_file_id.as_ref() =>
            {
                Ok(NativeWriteOutcome::Conflict)
            }
            _ => Ok(NativeWriteOutcome::Uncertain),
        }
    }
    pub async fn reconcile_existing(
        &self,
        node: &Node,
        path: &str,
        source: &Path,
        sha256: &str,
        staging: &Path,
    ) -> Result<NativeWriteOutcome> {
        if node.version == crate::creations::CREATE_VERSION {
            // Legacy callers lack durable author provenance. Matching bytes do
            // not authorize binding another client's file identity.
            return Ok(NativeWriteOutcome::Uncertain);
        }
        let intent = self.write_intent(node, path, source, sha256).await?;
        self.reconcile_intent(node, &intent, staging).await
    }
}

#[cfg(feature = "native-research")]
#[path = "native_namespace.rs"]
mod namespace;
#[cfg(feature = "native-research")]
pub use namespace::{NativeRemoveOutcome, NativeRenameOutcome};

/// Only versions of the same native object/server/scope are comparable.
pub(crate) fn older_version(incoming: &str, current: &str) -> bool {
    let (Ok(i), Ok(c)) = (
        serde_json::from_str::<VersionToken>(incoming),
        serde_json::from_str::<VersionToken>(current),
    ) else {
        return false;
    };
    i.schema == c.schema
        && i.schema == 1
        && i.restore == c.restore
        && i.view == c.view
        && i.root == c.root
        && i.file == c.file
        && i.directory == c.directory
        && (i.sync < c.sync || (i.sync == c.sync && i.max < c.max))
}

fn native_file_id(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|b| b.is_ascii_digit())
}

// Local Node.id is the persistent inode identity and can be rebound by an
// atomic save. VersionToken.file is the remote identity; transfer receipts still
// check that identity, scope, restore ID, version, size and full content hash.

pub(crate) fn directory_subtree_advance(old: &str, new: &str) -> bool {
    let (Ok(a), Ok(b)) = (
        serde_json::from_str::<VersionToken>(old),
        serde_json::from_str::<VersionToken>(new),
    ) else {
        return false;
    };
    a.schema == 1
        && b.schema == 1
        && a.directory
        && b.directory
        && a.restore == b.restore
        && a.view == b.view
        && a.root == b.root
        && a.file == b.file
        && a.sync == b.sync
        && a.max <= b.max
        && a.size == b.size
        && a.hash == b.hash
}

#[cfg(feature = "native-research")]
pub enum NativeDirectoryOutcome {
    Prepared(crate::directories::DirectoryOrigin),
    Created { node: Node, remote: String },
    Conflict,
    Uncertain,
}
#[cfg(feature = "native-research")]
#[path = "native_directory.rs"]
mod directory;

#[path = "native_snapshot.rs"]
mod native_snapshot;
pub use native_snapshot::{RemoteSnapshot, SNAPSHOT_NODE_LIMIT};

#[cfg(feature = "native-research")]
#[path = "native_upload_origin.rs"]
mod upload_origin;

#[path = "native_root_guard.rs"]
mod root_guard;

#[cfg(feature = "native-research")]
#[path = "native_upload.rs"]
mod upload;
