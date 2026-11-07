//! Versioned local desktop API. Paths resolve against persistent metadata;
//! querying an icon never opens a FUSE file or performs a NAS request.
use crate::service::Service;
use drive_core::{Node, Status, Store};
#[path = "desktop_preview.rs"]
mod preview;
#[path = "desktop_tree.rs"]
mod status_tree;
use serde::Serialize;
use std::{
    collections::{HashMap, HashSet},
    io,
    path::{Component, Path},
    sync::{Arc, atomic::Ordering},
};
pub const BUS_NAME: &str = "org.drivenative.Client1";
pub const OBJECT_PATH: &str = "/org/drivenative/Client1";
/// Available while shutdown checkpoints hold Store/control for local durability.
pub fn shutdown_status(service: &Service) -> String {
    serde_json::json!({"task_id":service.profile.task_id,
        "shutdown":service.shutdown_diagnostics.snapshot(),
        "mounted":service.mounted.load(Ordering::Acquire)})
    .to_string()
}

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
    if preview::active(&store, &service.profile.scope.file_id)? {
        return preview::states(
            &store,
            &service.profile.scope.file_id,
            &service.profile.mount,
            paths,
        );
    }
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
pub fn initialization(service: &Service) -> String {
    serde_json::json!({"schema":1,"task_id":service.profile.task_id,
        "initialization":service.bridge.initialization.snapshot()})
    .to_string()
}
pub fn overview(service: &Service) -> io::Result<String> {
    let (state, error, revision) = {
        let c = service.control.lock().unwrap();
        (c.state, c.error, c.revision)
    };
    let active_transfers = service.bridge.transfer_entries().len();
    let initialization = service.bridge.initialization.snapshot();
    let store = service.bridge.store.lock().unwrap();
    let pending = store.pending_operations().map_err(|_| other())?
        + store.pending_namespace_operations().map_err(|_| other())?;
    let preview_only = preview::active(&store, &service.profile.scope.file_id)?;
    let permission = drive_core::write_permissions::policy(&store, &service.profile.scope.file_id).map_err(|_|other())?;
    let writable = drive_core::write_permissions::mount_writable(&store, &service.profile.scope.file_id).map_err(|_|other())?;
    let queue_health = store.queue_health().map_err(|_| other())?;
    let issue_count = store.sync_issue_count(&service.profile.scope.file_id).ok();
    let cache = store.cache_availability().map_err(|_| other())?;
    let uncertain = queue_health.namespace_uncertain + queue_health.upload_uncertain;
    let sync_state = if error.is_some()
        || state == crate::service::ConnectionState::NeedsAttention
    {
        "needs_attention"
    } else if state == crate::service::ConnectionState::Paused {
        "paused"
    } else if active_transfers > 0 {
        "transferring"
    } else if initialization.active
        && initialization.phase != crate::initialization::Phase::Waiting
    {
        "checking"
    } else if pending > 0 || uncertain > 0 {
        "pending"
    } else {
        "idle"
    };
    let background = service.bridge.background_failure();
    let background_error = background.as_ref().map(|failure| failure.message);
    let background_error_kind = background.as_ref().map(|failure| failure.source.as_str());
    let background_error_since_ms = background.as_ref().map(|failure| failure.since_ms);
    Ok(serde_json::json!({"schema":1,"task_id":service.profile.task_id,"mount":service.profile.mount,
        "connection":state,"error":error,"background_error":background_error,
        "background_error_kind":background_error_kind,
        "background_error_since_ms":background_error_since_ms,
        "upload_block":*service.bridge.upload_block.lock().unwrap(),
        "recovery_actions":{"recheck_issues":true},
        "mounted":service.mounted.load(Ordering::Acquire),"writable":writable && !preview_only,
        "writable_source":permission.writable_source,"local_policy":permission.local_policy,
        "server_capability":permission.server_capability,"write_policy_revision":permission.write_policy_revision,
        "configured_writable":service.profile.writable,"preview_only":preview_only,
        "skipped_symlinks":store.skipped_symlink_count().map_err(|_|other())?,
        "skipped_permission_directories":store.inaccessible_count().map_err(|_|other())?,
        "pending_operations":pending,"revision":revision,"local_revision":store.local_revision(),
        "queue_health":queue_health,
        "downloads_started":service.bridge.downloads.load(Ordering::Relaxed),
        "download_errors":store.download_failure_count().map_err(|_|other())?,
        "download_diagnostics":{"failures_seen":service.bridge.download_failures_seen.load(Ordering::Relaxed),
            "last_failure":service.bridge.last_download_failure.lock().unwrap().as_ref().map(|(kind,code)|serde_json::json!({"kind":kind,"server_code":code}))},
        "connection_diagnostics":service.diagnostics.snapshot(),
        "sync_state":sync_state,"active_transfers":active_transfers,"issue_count":issue_count,
        "cache_availability":{"state":cache.state,"total":cache.total,"offline":cache.offline,"pinned":cache.pinned},
        "version":{"build":env!("CARGO_PKG_VERSION"),"state_schema":drive_core::STATE_SCHEMA_VERSION,"ipc":crate::wire::VERSION},
        "shutdown":service.shutdown_diagnostics.snapshot(),
        "initialization":initialization,
        "host":service.profile.host,"username":service.profile.username,"scope_path":service.profile.scope.path}).to_string())
}

fn history_request<T: serde::de::DeserializeOwned>(body: &str) -> io::Result<T> {
    if body.len() > 8192 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "history query too large"));
    }
    serde_json::from_str(body).map_err(|_| io::Error::new(io::ErrorKind::InvalidInput, "invalid history query"))
}

/// History never waits on Store/control while transfers or shutdown own them.
pub fn sync_history(service: &Service, body: &str) -> io::Result<String> {
    let query: drive_core::sync_history::HistoryQuery = history_request(body)?;
    let Ok(store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let page = store.sync_history(&query).map_err(|_| other())?;
    serde_json::to_string(&page).map_err(io::Error::other)
}

pub fn recent_activity(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Query { limit: Option<u32> }
    let request: Query = history_request(body)?;
    let limit = request.limit.unwrap_or(12);
    if limit == 0 || limit > 100 {
        return Err(io::Error::new(io::ErrorKind::InvalidInput, "invalid recent activity limit"));
    }
    let Ok(store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let page = store.sync_history(&drive_core::sync_history::HistoryQuery {
        limit: Some(limit), ..Default::default()
    }).map_err(|_| other())?;
    Ok(serde_json::json!({"records":page.records}).to_string())
}

pub fn clear_sync_history(service: &Service, body: &str) -> io::Result<String> {
    let request: drive_core::sync_history::ClearHistory = history_request(body)?;
    let Ok(mut store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let removed = store.clear_sync_history(&request).map_err(|_| other())?;
    Ok(serde_json::json!({"removed":removed}).to_string())
}

/// Retention IPC is metadata-only: it never deletes files, drafts, cached
/// bodies, or queued operations. A busy Store still answers immediately.
pub fn get_sync_history_retention(service: &Service) -> io::Result<String> {
    let Ok(store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let (days, records) = store.sync_history_retention().map_err(|_| other())?;
    Ok(serde_json::json!({"schema":1,"task_id":service.profile.task_id,
        "days":days,"records":records}).to_string())
}

pub fn set_sync_history_retention(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Request {
        schema: u8,
        task_id: String,
        days: u32,
        records: u32,
    }
    let request: Request = history_request(body)?;
    if request.schema != 1 || request.task_id != service.profile.task_id {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "wrong task or retention settings version",
        ));
    }
    let Ok(mut store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    store
        .set_sync_history_retention(request.days, request.records)
        .map_err(|_| other())?;
    Ok(serde_json::json!({"saved":true,"days":request.days,
        "records":request.records}).to_string())
}

/// Slot identifier of the D-Bus instance serving this mount. Legacy
/// single-mount installations have none; the GUI falls back to its own source
/// slot in both cases.
fn mount_slot() -> Option<String> {
    std::env::var("DRIVE_NATIVE_INSTANCE")
        .ok()
        .filter(|slot| !slot.is_empty())
}

/// Resolve identity now, not the event's obsolete pathname. This function only
/// reads metadata and cached bodies; it never touches FUSE or the network.
pub fn resolve_history_target(service: &Service, body: &str) -> io::Result<String> {
    #[derive(serde::Deserialize)]
    #[serde(deny_unknown_fields)]
    struct Query { id: i64 }
    let request: Query = history_request(body)?;
    if request.id<=0 { return Err(io::Error::new(io::ErrorKind::InvalidInput,"invalid history id")); }
    let Ok(control) = service.control.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let mounted = service.mounted.load(Ordering::Acquire);
    let online = mounted && control.state == crate::service::ConnectionState::Online;
    drop(control);
    let Ok(store) = service.bridge.store.try_lock() else {
        return Ok(serde_json::json!({"error":"busy"}).to_string());
    };
    let record = store.sync_history_record(request.id).map_err(|_| other())?;
    let mut response = serde_json::json!({"path":null,"exists":false,"mount_online":online,
        "reason":"not_found","parent_path":null,"offline_available":false,"cached":false,
        "mount_slot":mount_slot(),"suggested_action":null,"node":null});
    let Some(record) = record else { return Ok(response.to_string()); };
    if record.mount != service.profile.scope.file_id { return Ok(response.to_string()); }
    let Some(id) = record.node else { return Ok(response.to_string()); };
    let id = store.local_identity(&id).map_err(|_| other())?;
    response["node"] = id.clone().into();
    // Parent fallback comes from the current node identity, never by joining an
    // unchecked historical path. Deleted nodes can still retain a parent identity.
    if mounted {
        if let Some(parent) = store.sync_history_parent(&id).map_err(|_| other())? {
            if let Some(path) = history_current_path(&store, service, &parent)? {
                if drive_core::write_permissions::decide(&store,&service.profile.scope.file_id,&parent,drive_core::write_permissions::Action::List).map_err(|_|other())?!=drive_core::write_permissions::Decision::Deny {response["parent_path"] = path.into();}
            }
        }
    }
    if store.is_inaccessible(&id).map_err(|_| other())? || drive_core::write_permissions::decide(&store,&service.profile.scope.file_id,&id,drive_core::write_permissions::Action::Read).map_err(|_|other())?==drive_core::write_permissions::Decision::Deny {
        response["reason"] = "permission_denied".into();
        return Ok(response.to_string());
    }
    let node = match store.node(&id) {
        Ok(node) => node,
        Err(error) if error.is_not_found() => {
            response["reason"] = "deleted".into();
            return Ok(response.to_string());
        }
        Err(_) => { response["reason"]="unavailable".into(); return Ok(response.to_string()); }
    };
    let cached = node.directory
        || store.open_dirty_draft(&id).map_err(|_| other())?.is_some()
        || store.open_cached(&id).is_ok();
    response["offline_available"] = cached.into();
    response["cached"] = cached.into();
    response["exists"] = true.into();
    // Unmounted history still reports Store-level existence and cache state so
    // the GUI can offer a mount action. Paths stay null: they live in the mount.
    if !mounted {
        response["reason"] = "unavailable".into();
        response["suggested_action"] = "start_mount".into();
        return Ok(response.to_string());
    }
    let Some(path) = history_current_path(&store, service, &id)? else {
        response["reason"] = "unavailable".into();
        return Ok(response.to_string());
    };
    if cached || online {
        response["path"] = path.into();
        response["reason"] = if cached { serde_json::Value::Null } else { "online_only".into() };
    } else {
        response["reason"] = "offline".into();
    }
    Ok(response.to_string())
}

fn history_current_path(store: &Store, service: &Service, id: &str) -> io::Result<Option<String>> {
    if store.is_inaccessible(id).map_err(|_| other())? { return Ok(None); }
    if drive_core::write_permissions::decide(store,&service.profile.scope.file_id,id,drive_core::write_permissions::Action::Traverse).map_err(|_|other())?==drive_core::write_permissions::Decision::Deny{return Ok(None);}
    let relative = match store.transfer_path(id) { Ok(path)=>path,Err(_)=>return Ok(None) };
    let resolved = match resolve(store,&service.profile.scope.file_id,&service.profile.mount,&relative) {
        Ok(node)=>node,Err(_)=>return Ok(None)
    };
    if resolved.id!=id { return Ok(None); }
    Ok(Some(service.profile.mount.join(relative).to_string_lossy().into_owned()))
}
pub fn directory_permissions(service:&Service,body:&str)->io::Result<String> {
    #[derive(serde::Deserialize)] #[serde(deny_unknown_fields)] struct Query{path:String}
    let q:Query=history_request(body)?;
    let store=service.bridge.store.lock().unwrap();
    let node=resolve(&store,&service.profile.scope.file_id,&service.profile.mount,&q.path)?;
    let mut value=drive_core::write_permissions::snapshot(&store,&service.profile.scope.file_id,&node.id).map_err(io::Error::other)?;
    value["task_id"]=service.profile.task_id.clone().into();value["path"]=q.path.into();
    Ok(value.to_string())
}
pub fn set_write_policy(service:&Service,body:&str)->io::Result<String> {
    #[derive(serde::Deserialize)] #[serde(deny_unknown_fields)]
    struct Query{task_id:String,expected_revision:u64,local_policy:drive_core::write_permissions::LocalPolicy}
    let q:Query=history_request(body)?;
    if q.task_id!=service.profile.task_id{return Err(io::Error::other("wrong task"));}
    if q.local_policy==drive_core::write_permissions::LocalPolicy::FollowRemote && !cfg!(feature="native-research"){return Err(io::Error::other("writable native protocol release gate is closed"));}
    let store=service.bridge.store.lock().unwrap();
    drive_core::write_permissions::set_local_policy_checked(&store,&service.profile.scope.file_id,q.local_policy,q.expected_revision).map_err(io::Error::other)?;
    Ok(serde_json::json!({"saved":true,"remount_required":service.mounted.load(Ordering::Acquire)}).to_string())
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
        if preview::active(&store, &service.profile.scope.file_id)? {
            return preview::directory(
                &store,
                &service.profile.scope.file_id,
                &service.profile.mount,
                path,
                offset,
            );
        }
        store
            .ensure_fs_root(&service.profile.scope.file_id)
            .map_err(|_| other())?;
        let node = resolve(
            &store,
            &service.profile.scope.file_id,
            &service.profile.mount,
            path,
        )?;
        let total = store.child_node_count(&node.id).map_err(|_| other())?;
        let entries = store
            .child_nodes_page(&node.id, offset as usize, 256)
            .map_err(|_| other())?;
        let paths = entries
            .into_iter()
            .map(|r| {
                if path.is_empty() {
                    r.name
                } else {
                    format!("{}/{}", path.trim_end_matches('/'), r.name)
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
    async fn get_sync_history(&self, body: String) -> zbus::fdo::Result<String> {
        let s=self.0.clone();
        local(move || sync_history(&s,&body)).await
    }
    async fn get_recent_activity(&self, body: String) -> zbus::fdo::Result<String> {
        let s=self.0.clone();
        local(move || recent_activity(&s,&body)).await
    }
    async fn resolve_history_target(&self, body: String) -> zbus::fdo::Result<String> {
        let s=self.0.clone();
        local(move || resolve_history_target(&s,&body)).await
    }
    async fn get_directory_permissions(&self,body:String)->zbus::fdo::Result<String>{
        let s=self.0.clone();local(move||directory_permissions(&s,&body)).await
    }
    async fn set_write_policy(&self,body:String)->zbus::fdo::Result<String>{
        let s=self.0.clone();local(move||set_write_policy(&s,&body)).await
    }
    async fn clear_sync_history(&self, body: String) -> zbus::fdo::Result<String> {
        let s=self.0.clone();
        local(move || clear_sync_history(&s,&body)).await
    }
    async fn get_sync_history_retention(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || get_sync_history_retention(&s)).await
    }
    async fn set_sync_history_retention(&self, body: String) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || set_sync_history_retention(&s, &body)).await
    }
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
    async fn recheck_issues(&self) -> zbus::fdo::Result<String> {
        let s = self.0.clone();
        local(move || Ok(serde_json::to_string(&s.recheck_issues()?)?)).await
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
    // Intentionally independent of Store/control: snapshot import can hold Store.
    fn get_initialization(&self) -> String {
        initialization(&self.0)
    }
    fn get_shutdown(&self) -> String {
        shutdown_status(&self.0)
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
    async fn pause(&self) -> zbus::fdo::Result<()> {
        let s = self.0.clone();
        local(move || s.pause()).await
    }
    async fn resume(&self) -> zbus::fdo::Result<()> {
        let s = self.0.clone();
        local(move || s.resume()).await
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
    #[zbus(signal)]
    pub async fn shutdown_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        revision: u64,
    ) -> zbus::Result<()>;
    #[zbus(signal)]
    pub async fn progress_changed(
        emitter: &zbus::object_server::SignalEmitter<'_>,
        revision: u64,
    ) -> zbus::Result<()>;
}
