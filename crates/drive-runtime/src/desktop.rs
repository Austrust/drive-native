//! Versioned local desktop API. Paths resolve against persistent metadata;
//! querying an icon never opens a FUSE file or performs a NAS request.
use crate::service::Service;
use drive_core::{Node, Status, Store};
#[path = "desktop_tree.rs"]
mod status_tree;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Component, Path},
    sync::{atomic::Ordering, Arc},
};
pub const BUS_NAME: &str = "org.drivenative.Client1";
pub const OBJECT_PATH: &str = "/org/drivenative/Client1";

#[derive(Clone, Serialize)]
pub struct EntryState {
    pub id: String,
    pub directory: bool,
    pub status: &'static str,
    pub pinned: bool,
    pub offline_available: bool,
    pub sync_error: Option<drive_core::download_jobs::DownloadFailure>,
}
#[derive(Serialize)]
struct PathState {
    path: String,
    #[serde(flatten)]
    entry: Option<EntryState>,
    error: Option<&'static str>,
}
fn other() -> io::Error {
    io::Error::other("local state unavailable")
}
pub fn resolve(store: &Store, root: &str, mount: &Path, input: &str) -> io::Result<Node> {
    if input.len() > 8192 || input.contains('\0') {
        return Err(io::Error::other("invalid path"));
    }
    let path = Path::new(input);
    let relative = if path.is_absolute() {
        path.strip_prefix(mount)
            .map_err(|_| io::Error::other("path outside mount"))?
    } else {
        path
    };
    let mut node = store.node(root).map_err(|_| other())?;
    for (depth, part) in relative.components().enumerate() {
        if depth >= 128 {
            return Err(io::Error::other("path depth limit"));
        }
        match part {
            Component::CurDir => {}
            Component::Normal(name) => {
                let name = name.to_str().ok_or_else(other)?;
                if !node.directory {
                    return Err(other());
                }
                node = store.lookup_node(&node.id, name).map_err(|_| other())?;
            }
            _ => return Err(io::Error::other("path traversal is not allowed")),
        }
    }
    Ok(node)
}
fn status_name(status: &Status) -> &'static str {
    match status {
        Status::OnlineOnly => "online_only",
        Status::LocalComplete => "local_complete",
        Status::PinnedComplete => "pinned_complete",
        Status::Updating => "updating",
        Status::Pinning => "pinning",
        Status::PendingUpload => "pending_upload",
        Status::Conflict => "conflict",
        Status::Deleted => "deleted",
    }
}
fn entry(
    store: Option<&Store>,
    id: &str,
    directory: bool,
    active: &HashSet<String>,
    memo: &mut HashMap<String, EntryState>,
    visiting: &mut HashSet<String>,
    tree: &mut Option<status_tree::StatusTree>,
    depth: usize,
) -> io::Result<EntryState> {
    if let Some(state) = memo.get(id) {
        return Ok(state.clone());
    }
    if depth > 128 || !visiting.insert(id.to_owned()) {
        return Err(other());
    }
    let (pinned, detailed, overlay) = match tree {
        Some(t) => t.flags(id)?,
        None => (
            store.unwrap().is_pinned(id).map_err(|_| other())?,
            true,
            false,
        ),
    };
    let captured = tree
        .as_ref()
        .and_then(|t| t.detail(id))
        .transpose()
        .map_err(|_| other())?;
    let own = if let Some((own, _, _)) = &captured {
        own.clone()
    } else if detailed {
        store.unwrap().status(id).map_err(|_| other())?
    } else if overlay {
        Status::PendingUpload
    } else if pinned {
        Status::Pinning
    } else {
        Status::OnlineOnly
    };
    let mut status = status_name(&own);
    let offline;
    let mut sync_error = None;
    if directory {
        let children = tree.as_ref().unwrap().children(id);
        let states: Vec<_> = children
            .iter()
            .map(|(id, directory)| {
                entry(
                    store,
                    id,
                    *directory,
                    active,
                    memo,
                    visiting,
                    tree,
                    depth + 1,
                )
            })
            .collect::<io::Result<_>>()?;
        offline = states.iter().all(|s| s.offline_available);
        sync_error = states
            .iter()
            .filter_map(|s| s.sync_error.clone())
            .max_by_key(|f| f.attempts);
        let has = |label: &str| states.iter().any(|s| s.status == label);
        status = if own == Status::Conflict || has("conflict") {
            "conflict"
        } else if own == Status::PendingUpload || has("pending_upload") {
            "pending_upload"
        } else if has("download_failed") || has("update_failed") {
            if offline {
                "update_failed"
            } else {
                "download_failed"
            }
        } else if has("download_retry") || has("update_retry") {
            if offline {
                "update_retry"
            } else {
                "download_retry"
            }
        } else if has("updating") {
            "updating"
        } else if has("downloading") {
            "downloading"
        } else if has("pinning") || pinned && !offline {
            "pinning"
        } else if offline {
            if pinned {
                "pinned_complete"
            } else {
                "local_complete"
            }
        } else if states
            .iter()
            .any(|s| s.offline_available || s.status == "mixed")
        {
            "mixed"
        } else {
            "online_only"
        };
    } else {
        offline = if let Some((_, offline, _)) = &captured {
            *offline
        } else {
            detailed
                && (store
                    .unwrap()
                    .open_dirty_draft(id)
                    .map_err(|_| other())?
                    .is_some()
                    || store.unwrap().open_cached(id).is_ok())
        };
        if detailed && !matches!(own, Status::Conflict | Status::PendingUpload) {
            sync_error = if let Some((_, _, failure)) = &captured {
                failure.clone()
            } else {
                store.unwrap().download_failure(id).map_err(|_| other())?
            };
            if let Some(failure) = &sync_error {
                status = match (offline, failure.retry_at.is_some()) {
                    (true, true) => "update_retry",
                    (true, false) => "update_failed",
                    (false, true) => "download_retry",
                    (false, false) => "download_failed",
                };
            }
        }
        if active.contains(id) && !matches!(own, Status::Conflict | Status::PendingUpload) {
            status = if offline { "updating" } else { "downloading" };
        }
    }
    visiting.remove(id);
    let state = EntryState {
        id: id.to_owned(),
        directory: directory,
        status,
        pinned,
        offline_available: offline,
        sync_error,
    };
    memo.insert(id.to_owned(), state.clone());
    Ok(state)
}
pub fn states(service: &Service, paths: Vec<String>) -> io::Result<String> {
    if paths.len() > 256 {
        return Err(io::Error::other("at most 256 paths per batch"));
    }
    let active = service.bridge.active_downloads().into_iter().collect();
    let store = service.bridge.store.lock().unwrap();
    store
        .ensure_fs_root(&service.profile.scope.file_id)
        .map_err(|_| other())?;
    let resolved: Vec<_> = paths
        .into_iter()
        .map(|path| {
            let node = resolve(
                &store,
                &service.profile.scope.file_id,
                &service.profile.mount,
                &path,
            );
            (path, node)
        })
        .collect();
    let mut tree = if resolved
        .iter()
        .any(|(_, n)| n.as_ref().is_ok_and(|n| n.directory))
    {
        Some(status_tree::StatusTree::load(
            &store,
            &resolved
                .iter()
                .filter_map(|(_, n)| n.as_ref().ok().map(|n| n.id.clone()))
                .collect::<Vec<_>>(),
        )?)
    } else {
        None
    };
    // Folder aggregation is CPU-only after capture. Release the writer's mutex
    // before walking the tree, so FUSE and overview requests can make progress.
    let guard = if tree.is_some() {
        drop(store);
        None
    } else {
        Some(store)
    };
    let mut memo = HashMap::new();
    let mut out = Vec::new();
    for (path, node) in resolved {
        let result = node.and_then(|node| {
            entry(
                guard.as_deref(),
                &node.id,
                node.directory,
                &active,
                &mut memo,
                &mut HashSet::new(),
                &mut tree,
                0,
            )
        });
        out.push(match result {
            Ok(entry) => PathState {
                path,
                entry: Some(entry),
                error: None,
            },
            Err(_) => PathState {
                path,
                entry: None,
                error: Some("path_or_local_state_unavailable"),
            },
        });
    }
    Ok(serde_json::json!({"schema":1,"entries":out}).to_string())
}
pub fn transfers(service: &Service) -> io::Result<String> {
    let entries = service.bridge.transfer_entries();
    let store = service.bridge.store.lock().unwrap();
    let entries = entries
        .into_iter()
        .map(|entry| {
            let path = store.transfer_path(&entry.node).ok();
            let mut value = serde_json::to_value(entry).unwrap();
            value["path"] = path.into();
            value
        })
        .collect::<Vec<_>>();
    Ok(
        serde_json::json!({"schema":1,"task_id":service.profile.task_id,"entries":entries})
            .to_string(),
    )
}
pub fn transfer_settings(service: &Service) -> io::Result<String> {
    let limits = service
        .bridge
        .store
        .lock()
        .unwrap()
        .transfer_limits()
        .map_err(io::Error::other)?;
    Ok(serde_json::json!({"schema":1,"task_id":service.profile.task_id,"limits":limits,"version":limits.fingerprint()}).to_string())
}
pub fn set_transfer_settings(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Request {
        schema: u8,
        task_id: String,
        expected_version: String,
        limits: drive_core::transfer_policy::TransferLimits,
    }
    if body.len() > 4096 {
        return Err(io::Error::other("transfer settings too large"));
    }
    let request: Request = serde_json::from_str(body)?;
    if request.schema != 1 || request.task_id != service.profile.task_id {
        return Err(io::Error::other("wrong task or settings version"));
    }
    service.set_transfer_limits(&request.expected_version, request.limits)?;
    transfer_settings(service)
}
pub fn overview(service: &Service) -> io::Result<String> {
    let (state, error, revision) = {
        let c = service.control.lock().unwrap();
        (c.state, c.error, c.revision)
    };
    let store = service.bridge.store.lock().unwrap();
    let pending = store.pending_operations().map_err(|_| other())?
        + store.pending_namespace_operations().map_err(|_| other())?;
    Ok(serde_json::json!({"schema":1,"task_id":service.profile.task_id,"mount":service.profile.mount,
        "connection":state,"error":error,"background_error":*service.bridge.last_error.lock().unwrap(),
        "mounted":service.mounted.load(Ordering::Acquire),"writable":service.profile.writable,
        "skipped_symlinks":store.skipped_symlink_count().map_err(|_|other())?,
        "skipped_permission_directories":store.inaccessible_count().map_err(|_|other())?,
        "pending_operations":pending,"revision":revision,"local_revision":store.local_revision(),
        "downloads_started":service.bridge.downloads.load(Ordering::Relaxed),
        "download_errors":store.download_failure_count().map_err(|_|other())?,
        "download_diagnostics":{"failures_seen":service.bridge.download_failures_seen.load(Ordering::Relaxed),
            "last_failure":service.bridge.last_download_failure.lock().unwrap().as_ref().map(|(kind,code)|serde_json::json!({"kind":kind,"server_code":code}))},
        "host":service.profile.host,"username":service.profile.username,"scope_path":service.profile.scope.path}).to_string())
}
fn pin(service: &Service, path: &str, value: bool) -> io::Result<()> {
    let mut store = service.bridge.store.lock().unwrap();
    let node = resolve(
        &store,
        &service.profile.scope.file_id,
        &service.profile.mount,
        path,
    )?;
    store.pin(&node.id, value).map_err(|_| other())?;
    drop(store);
    service.bridge.signal_upload();
    Ok(())
}
fn release(service: &Service, path: &str) -> io::Result<String> {
    let ids = {
        let store = service.bridge.store.lock().unwrap();
        let node = resolve(
            &store,
            &service.profile.scope.file_id,
            &service.profile.mount,
            path,
        )?;
        let mut pending = vec![node];
        let mut ids = Vec::new();
        let mut seen = HashSet::new();
        while let Some(node) = pending.pop() {
            if !seen.insert(node.id.clone()) {
                return Err(other());
            }
            if node.directory {
                pending.extend(
                    store
                        .children(&node.id)
                        .map_err(|_| other())?
                        .into_iter()
                        .map(|r| r.node),
                );
            } else {
                ids.push(node.id);
            }
        }
        ids
    };
    let mut released = Vec::new();
    let mut protected = Vec::new();
    for id in ids {
        if service.bridge.release_content(&id).is_ok() {
            released.push(id);
        } else {
            protected.push(id);
        }
    }
    let bytes = service
        .bridge
        .store
        .lock()
        .unwrap()
        .collect_garbage()
        .map_err(|_| other())?;
    Ok(serde_json::json!({"schema":1,"released":released,"protected_or_failed":protected,"reclaimed_bytes":bytes}).to_string())
}

/// Read-only browser pages from local metadata, including before network unlock.
pub fn directory(service: &Service, path: &str, offset: u32) -> io::Result<String> {
    let (paths, total) = {
        let store = service.bridge.store.lock().unwrap();
        store
            .ensure_fs_root(&service.profile.scope.file_id)
            .map_err(|_| other())?;
        let node = resolve(
            &store,
            &service.profile.scope.file_id,
            &service.profile.mount,
            path,
        )?;
        let entries = store.children(&node.id).map_err(|_| other())?;
        let total = entries.len();
        let paths = entries
            .into_iter()
            .skip(offset as usize)
            .take(256)
            .map(|r| {
                if path.is_empty() {
                    r.node.name
                } else {
                    format!("{}/{}", path.trim_end_matches('/'), r.node.name)
                }
            })
            .collect();
        (paths, total)
    };
    let mut result: serde_json::Value = serde_json::from_str(&states(service, paths)?)?;
    result["total"] = total.into();
    result["offset"] = offset.into();
    result["path"] = path.into();
    Ok(result.to_string())
}
pub fn retry_downloads(service: &Service, path: &str) -> io::Result<String> {
    let mut store = service.bridge.store.lock().unwrap();
    let node = resolve(
        &store,
        &service.profile.scope.file_id,
        &service.profile.mount,
        path,
    )?;
    let mut pending = vec![node];
    let mut seen = HashSet::new();
    let mut count = 0;
    while let Some(node) = pending.pop() {
        if !seen.insert(node.id.clone()) {
            return Err(other());
        }
        if node.directory {
            pending.extend(
                store
                    .children(&node.id)
                    .map_err(|_| other())?
                    .into_iter()
                    .map(|r| r.node),
            );
        } else if store.retry_download(&node.id).map_err(|_| other())? {
            count += 1;
        }
    }
    drop(store);
    service.bridge.signal_upload();
    Ok(serde_json::json!({"schema":1,"reset_downloads":count}).to_string())
}
pub fn issues(service: &Service, after: &str) -> io::Result<String> {
    let store = service.bridge.store.lock().unwrap();
    let page = store
        .sync_issues(&service.profile.scope.file_id, after)
        .map_err(io::Error::other)?;
    Ok(serde_json::json!({"schema":1,"entries":page.entries,"next":page.next}).to_string())
}
pub fn keep_both(service: &Service, id: &str, token: &str, name: &str) -> io::Result<String> {
    let result =
        service
            .bridge
            .keep_conflict_both(&service.profile.scope.file_id, id, token, name)?;
    Ok(serde_json::json!({"schema":1,"original":result.original,"copy_id":result.copy.id,"path":result.path}).to_string())
}
pub fn keep_directory_both(
    service: &Service,
    id: &str,
    token: &str,
    name: &str,
) -> io::Result<String> {
    let result = service.bridge.keep_directory_conflict_both(
        &service.profile.scope.file_id,
        id,
        token,
        name,
    )?;
    Ok(
        serde_json::json!({"schema":1,"directory_id":result.node.id,"path":result.path})
            .to_string(),
    )
}
pub fn trust_settings(service: &Service) -> io::Result<String> {
    let settings = service.trust_settings();
    let version = settings.fingerprint();
    let der =
        sproto::trust::parse_trusted_pem(&settings.trusted_ca_pem).map_err(io::Error::other)?;
    let fingerprints: Vec<_> = der.iter().map(|b| drive_core::sha256(b)).collect();
    Ok(serde_json::json!({"schema":1,"settings":settings,"version":version,"host":service.profile.host,"tls_name":service.profile.tls_name,"fingerprints":fingerprints}).to_string())
}
pub fn set_trust_settings(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Change {
        expected_version: String,
        settings: crate::trust::TrustSettings,
    }
    if body.len() > 65536 {
        return Err(io::Error::other("trust request too large"));
    }
    let change: Change =
        serde_json::from_str(body).map_err(|_| io::Error::other("invalid trust request"))?;
    service.set_trust_settings(&change.expected_version, change.settings)?;
    trust_settings(service)
}
pub fn cache_settings(service: &Service) -> io::Result<String> {
    let usage = service.bridge.cache_usage()?;
    let state = service.control.lock().unwrap().state;
    Ok(serde_json::json!({"schema":1,"task_id":service.profile.task_id,"usage":usage,"connection":state,"error":*service.bridge.cache_error.lock().unwrap(),"recent_download_seconds":drive_core::cache_policy::RECENT_DOWNLOAD_SECONDS}).to_string())
}
pub fn set_cache_settings(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Settings {
        schema: u8,
        task_id: String,
        limit_bytes: u64,
    }
    if body.len() > 1024 {
        return Err(io::Error::other("cache settings too large"));
    }
    let settings: Settings = serde_json::from_str(body).map_err(io::Error::other)?;
    if settings.schema != 1
        || settings.task_id != service.profile.task_id
        || settings.limit_bytes % 1048576 != 0
    {
        return Err(io::Error::other("unsupported cache settings version"));
    }
    service
        .bridge
        .store
        .lock()
        .unwrap()
        .set_cache_limit(settings.limit_bytes)
        .map_err(io::Error::other)?;
    service.cache_policy_changed();
    cache_settings(service)
}
pub struct Desktop(pub Arc<Service>);
async fn local<T: Send + 'static>(
    f: impl FnOnce() -> io::Result<T> + Send + 'static,
) -> zbus::fdo::Result<T> {
    tokio::task::spawn_blocking(f)
        .await
        .map_err(|_| zbus::fdo::Error::Failed("local task failed".into()))?
        .map_err(|_| zbus::fdo::Error::Failed("operation unavailable; inspect local state".into()))
}
#[zbus::interface(name = "org.drivenative.Client1")]
impl Desktop {
    async fn get_trust_settings(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || trust_settings(&s)).await
    }
    async fn set_trust_settings(&self, body: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || set_trust_settings(&s, &body)).await
    }
    async fn get_cache_settings(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || cache_settings(&s)).await
    }
    async fn set_cache_settings(&self, body: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || set_cache_settings(&s, &body)).await
    }
    async fn get_issues(&self, after: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || issues(&s, &after)).await
    }
    async fn keep_both(
        &self,
        id: String,
        token: String,
        name: String,
    ) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || keep_both(&s, &id, &token, &name)).await
    }

    async fn keep_directory_both(
        &self,
        id: String,
        token: String,
        name: String,
    ) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || keep_directory_both(&s, &id, &token, &name)).await
    }
    async fn retry_downloads(&self, path: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || retry_downloads(&s, &path)).await
    }
    async fn get_directory(&self, path: String, offset: i32) -> zbus::fdo::Result<String> {
        if offset < 0 {
            return Err(zbus::fdo::Error::InvalidArgs("negative offset".into()));
        }
        let s = self.0.clone();
        local(move || directory(&s, &path, offset as u32)).await
    }
    async fn get_transfers(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || transfers(&s)).await
    }
    async fn get_transfer_settings(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || transfer_settings(&s)).await
    }
    async fn set_transfer_settings(&self, body: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || set_transfer_settings(&s, &body)).await
    }
    async fn get_overview(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || overview(&s)).await
    }
    async fn get_states(&self, paths: Vec<String>) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || states(&s, paths)).await
    }
    async fn set_pinned(&self, path: String, value: bool) -> zbus::fdo::Result<()> {
        let s = self.0.clone();
        local(move || pin(&s, &path, value)).await
    }
    async fn release_space(&self, path: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || release(&s, &path)).await
    }
    fn pause(&self) {
        self.0.pause();
    }
    fn resume(&self) {
        self.0.resume();
    }
    fn unlock(&self, password: String) -> zbus::fdo::Result<()> {
        self.0.unlock(password).map_err(|_| {
            zbus::fdo::Error::InvalidArgs("invalid credential input or service stopping".into())
        })
    }
    fn stop_mount(&self) {
        self.0.shutdown();
    }
    #[zbus(signal)]
    pub async fn status_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        revision: u64,
    ) -> zbus::Result<()>;
}
