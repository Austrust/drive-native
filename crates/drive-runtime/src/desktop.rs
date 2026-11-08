//! Versioned local desktop API. Paths resolve against persistent metadata;
//! querying an icon never opens a FUSE file or performs a NAS request.
use crate::service::Service;
use crate::status_cache::StatusProjection;
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
    time::Instant,
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
    tree: &mut Option<status_tree::StatusTree<'_>>,
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
/// Private read-only handle for desktop projections. Opening it neither takes
/// the global `SharedStore` mutex nor `writer.lock`, so a long read (or the
/// writer's own transaction) cannot stall FUSE and D-Bus replies. It shares the
/// writer's handle map and scheduler observations, so derived fields keep the
/// writer's meaning. `None` keeps the historical serialized path available as a
/// fail-safe.
fn read_only(service: &Service) -> Option<Store> {
    service.bridge.store.read_only().ok()
}
fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}
pub fn states(service: &Service, paths: Vec<String>) -> io::Result<String> {
    if paths.len() > 256 {
        return Err(io::Error::other("at most 256 paths per batch"));
    }
    let active = service.bridge.active_downloads().into_iter().collect();
    let ro = read_only(service);
    states_with(service, paths, &active, ro.as_ref())
}
fn states_with(
    service: &Service,
    paths: Vec<String>,
    active: &HashSet<String>,
    read_only: Option<&Store>,
) -> io::Result<String> {
    // The whole projection is a pure read. Try it on a private read-only
    // snapshot first; any unexpected failure falls back to the serialized
    // writer path so error and initialization semantics stay unchanged.
    if let Some(store) = read_only {
        if let Ok(response) = states_off_lock(store, service, paths.clone(), active) {
            return Ok(response);
        }
    }
    states_locked(service, paths, active)
}
fn states_off_lock(
    store: &Store,
    service: &Service,
    paths: Vec<String>,
    active: &HashSet<String>,
) -> io::Result<String> {
    let root = &service.profile.scope.file_id;
    if preview::active(store, root)? {
        return preview::states(store, root, &service.profile.mount, paths);
    }
    store.ensure_fs_root(root).map_err(|_| other())?;
    let resolved = resolve_all(store, service, paths);
    // A ready projection answers folder aggregation without another scoped SQL
    // read; unready (None) keeps the authoritative scoped read.
    let projection = service.status_projection();
    let mut tree = match projection.as_deref() {
        Some(projection) => load_tree_with_projection(store, projection, &resolved)?,
        None => load_tree(store, &resolved)?,
    };
    Ok(render_states(resolved, active, Some(store), &mut tree))
}
fn states_locked(
    service: &Service,
    paths: Vec<String>,
    active: &HashSet<String>,
) -> io::Result<String> {
    let root = &service.profile.scope.file_id;
    let store = service.bridge.store.lock().unwrap();
    if preview::active(&store, root)? {
        return preview::states(&store, root, &service.profile.mount, paths);
    }
    store.ensure_fs_root(root).map_err(|_| other())?;
    let resolved = resolve_all(&store, service, paths);
    let mut tree = load_tree(&store, &resolved)?;
    // Folder aggregation is CPU-only after capture. Release the writer's mutex
    // before walking the tree, so FUSE and overview requests can make progress.
    let guard = if tree.is_some() {
        drop(store);
        None
    } else {
        Some(store)
    };
    Ok(render_states(resolved, active, guard.as_deref(), &mut tree))
}
fn resolve_all(
    store: &Store,
    service: &Service,
    paths: Vec<String>,
) -> Vec<(String, io::Result<Node>)> {
    paths
        .into_iter()
        .map(|path| {
            let node = resolve(
                store,
                &service.profile.scope.file_id,
                &service.profile.mount,
                &path,
            );
            (path, node)
        })
        .collect()
}
fn load_tree(
    store: &Store,
    resolved: &[(String, io::Result<Node>)],
) -> io::Result<Option<status_tree::StatusTree<'static>>> {
    let Some(roots) = tree_roots(resolved) else {
        return Ok(None);
    };
    Ok(Some(status_tree::StatusTree::load(store, &roots)?))
}
/// Root ids of a batch whose directories need tree aggregation; `None` keeps the
/// historical shortcut for batches that only resolve files.
fn tree_roots(resolved: &[(String, io::Result<Node>)]) -> Option<Vec<String>> {
    if !resolved
        .iter()
        .any(|(_, n)| n.as_ref().is_ok_and(|n| n.directory))
    {
        return None;
    }
    Some(
        resolved
            .iter()
            .filter_map(|(_, n)| n.as_ref().ok().map(|n| n.id.clone()))
            .collect(),
    )
}
/// Counts batches served by the committed projection. Test-only evidence that
/// the wiring actually prefers it; never compiled into the production library.
#[cfg(test)]
static PROJECTION_TREE_LOADS: std::sync::atomic::AtomicUsize =
    std::sync::atomic::AtomicUsize::new(0);
/// Prefer the committed projection (issue #8): it answers the batch without a
/// scoped SQLite read. A projection that cannot describe every resolved root
/// (stale snapshot, missing ancestor, disabled cache) falls back to the
/// authoritative scoped read instead of returning a truncated tree.
fn load_tree_with_projection<'p>(
    store: &Store,
    projection: &'p StatusProjection,
    resolved: &[(String, io::Result<Node>)],
) -> io::Result<Option<status_tree::StatusTree<'p>>> {
    let Some(roots) = tree_roots(resolved) else {
        return Ok(None);
    };
    match status_tree::StatusTree::load_with_projection(store, projection, &roots) {
        Ok(tree) => {
            #[cfg(test)]
            PROJECTION_TREE_LOADS.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
            Ok(Some(tree))
        }
        Err(_) => load_tree(store, resolved),
    }
}
fn render_states(
    resolved: Vec<(String, io::Result<Node>)>,
    active: &HashSet<String>,
    store: Option<&Store>,
    tree: &mut Option<status_tree::StatusTree<'_>>,
) -> String {
    let mut memo = HashMap::new();
    let mut out = Vec::new();
    for (path, node) in resolved {
        let result = node.and_then(|node| {
            entry(
                store,
                &node.id,
                node.directory,
                active,
                &mut memo,
                &mut HashSet::new(),
                tree,
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
    serde_json::json!({"schema":1,"entries":out}).to_string()
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
/// Cost breakdown for [`overview_measured`], covering the issue #21 acceptance
/// segments. `query_us` is the Store read; `serialize_us` is JSON assembly off
/// every lock. `lock_wait_us`/`lock_hold_us` are only non-zero on the fail-safe
/// path that a writer-owned transaction forces.
#[derive(Clone, Copy, Debug, Default, Serialize)]
pub struct OverviewTimings {
    pub total_us: u64,
    pub query_us: u64,
    pub serialize_us: u64,
    pub lock_wait_us: u64,
    pub lock_hold_us: u64,
    pub read_only: bool,
    pub derived_cached: bool,
}
/// Expensive counters reused while the committed Store generation is unchanged.
/// `sync_issue_count` walks the issue candidate set and `cache_availability`
/// scans every live node; both only depend on committed SQLite rows, which
/// `key` identifies without taking the Store lock.
#[derive(Clone, Copy)]
pub(crate) struct OverviewDerived {
    pub(crate) key: drive_core::shared_store::CommitState,
    pub(crate) issue_count: Option<u64>,
    pub(crate) cache: drive_core::CacheAvailability,
}
#[derive(Clone, Debug)]
struct StoreOverview {
    pending: i64,
    preview_only: bool,
    writable: bool,
    permission: drive_core::write_permissions::Policy,
    queue_health: drive_core::namespace_schedule::QueueHealth,
    issue_count: Option<u64>,
    cache: drive_core::CacheAvailability,
    skipped_symlinks: u64,
    inaccessible: i64,
    download_errors: u64,
}
fn elapsed_us(start: Instant) -> u64 {
    u64::try_from(start.elapsed().as_micros()).unwrap_or(u64::MAX)
}
/// Small metadata snapshot. Every read is a plain SELECT; on the private
/// read-only connection it never touches the serialized writer.
fn overview_store(
    service: &Service,
    store: &Store,
    derived: Option<OverviewDerived>,
) -> io::Result<StoreOverview> {
    let root = &service.profile.scope.file_id;
    let pending = store.pending_operations().map_err(|_| other())?
        + store.pending_namespace_operations().map_err(|_| other())?;
    let preview_only = preview::active(store, root)?;
    let permission = drive_core::write_permissions::policy(store, root).map_err(|_| other())?;
    let writable =
        drive_core::write_permissions::mount_writable(store, root).map_err(|_| other())?;
    let queue_health = store.queue_health().map_err(|_| other())?;
    let (issue_count, cache) = match derived {
        Some(derived) => (derived.issue_count, derived.cache),
        None => (
            store.sync_issue_count(root).ok(),
            store.cache_availability().map_err(|_| other())?,
        ),
    };
    Ok(StoreOverview {
        pending,
        preview_only,
        writable,
        permission,
        queue_health,
        issue_count,
        cache,
        skipped_symlinks: store.skipped_symlink_count().map_err(|_| other())?,
        inaccessible: store.inaccessible_count().map_err(|_| other())?,
        download_errors: store.download_failure_count().map_err(|_| other())?,
    })
}
pub fn overview(service: &Service) -> io::Result<String> {
    Ok(overview_measured(service)?.0)
}
/// Steady-state overview: a private read-only connection plus a generation-keyed
/// derived cache, so neither FUSE nor other desktop reads queue behind
/// GetOverview. The serialized writer remains a fail-safe; when used it is
/// released before JSON assembly and its cost is reported in the timings.
pub fn overview_measured(service: &Service) -> io::Result<(String, OverviewTimings)> {
    let started = Instant::now();
    let mut timings = OverviewTimings::default();
    let (state, error, revision) = {
        let c = service.control.lock().unwrap();
        (c.state, c.error, c.revision)
    };
    let active_transfers = service.bridge.transfer_entries().len();
    let initialization = service.bridge.initialization.snapshot();
    let key = service.bridge.store.change_state();
    let cached = service.overview_derived(key);
    let ro = read_only(service);
    let mut query_us = 0;
    let mut snapshot = None;
    if let Some(store) = ro.as_ref() {
        let query = Instant::now();
        if let Ok(found) = overview_store(service, store, cached) {
            query_us = elapsed_us(query);
            timings.read_only = true;
            snapshot = Some(found);
        }
    }
    timings.query_us = query_us;
    let (store_overview, local_revision) = match snapshot {
        Some(found) => (found, service.bridge.store.local_revision()),
        None => {
            let wait = Instant::now();
            let guard = service.bridge.store.lock().unwrap();
            timings.lock_wait_us = elapsed_us(wait);
            let hold = Instant::now();
            let found = overview_store(service, &guard, None)?;
            let local_revision = guard.local_revision();
            timings.lock_hold_us = elapsed_us(hold);
            drop(guard);
            (found, local_revision)
        }
    };
    timings.derived_cached = timings.read_only && cached.is_some();
    if timings.read_only && cached.is_none() {
        // Only publish when no writer committed between the key read and the
        // queries: a stale pair must never survive into the next projection.
        let after = service.bridge.store.change_state();
        if after == key {
            service.cache_overview_derived(OverviewDerived {
                key: after,
                issue_count: store_overview.issue_count,
                cache: store_overview.cache,
            });
        }
    }
    let StoreOverview {
        pending,
        preview_only,
        writable,
        permission,
        queue_health,
        issue_count,
        cache,
        skipped_symlinks,
        inaccessible,
        download_errors,
    } = store_overview;
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
    let serialize = Instant::now();
    let body = serde_json::json!({"schema":1,"task_id":service.profile.task_id,"mount":service.profile.mount,
        "connection":state,"error":error,"background_error":background_error,
        "background_error_kind":background_error_kind,
        "background_error_since_ms":background_error_since_ms,
        "upload_block":*service.bridge.upload_block.lock().unwrap(),
        "recovery_actions":{"recheck_issues":true},
        "mounted":service.mounted.load(Ordering::Acquire),"writable":writable && !preview_only,
        "writable_source":permission.writable_source,"local_policy":permission.local_policy,
        "server_capability":permission.server_capability,"write_policy_revision":permission.write_policy_revision,
        "configured_writable":service.profile.writable,"preview_only":preview_only,
        "skipped_symlinks":skipped_symlinks,
        "skipped_permission_directories":inaccessible,
        "pending_operations":pending,"revision":revision,"local_revision":local_revision,
        "queue_health":queue_health,
        "downloads_started":service.bridge.downloads.load(Ordering::Relaxed),
        "download_errors":download_errors,
        "download_diagnostics":{"failures_seen":service.bridge.download_failures_seen.load(Ordering::Relaxed),
            "last_failure":service.bridge.last_download_failure.lock().unwrap().as_ref().map(|(kind,code)|serde_json::json!({"kind":kind,"server_code":code}))},
        "connection_diagnostics":service.diagnostics.snapshot(),
        "sync_state":sync_state,"active_transfers":active_transfers,"issue_count":issue_count,
        "cache_availability":{"state":cache.state,"total":cache.total,"offline":cache.offline,"pinned":cache.pinned},
        "version":{"build":env!("CARGO_PKG_VERSION"),"state_schema":drive_core::STATE_SCHEMA_VERSION,"ipc":crate::wire::VERSION},
        "shutdown":service.shutdown_diagnostics.snapshot(),
        "initialization":initialization,
        "host":service.profile.host,"username":service.profile.username,"scope_path":service.profile.scope.path}).to_string();
    timings.serialize_us = elapsed_us(serialize);
    timings.total_us = elapsed_us(started);
    Ok((body, timings))
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
    let ro = read_only(service);
    issues_with(service, after, ro.as_ref())
}
fn issues_with(service: &Service, after: &str, read_only: Option<&Store>) -> io::Result<String> {
    // `sync_issues` is the slowest desktop query on large stores. Run the page
    // on a private read-only snapshot so paging the issue center never holds
    // the global writer mutex that FUSE/GetStates need; a failed open or query
    // falls back to the historical locked path.
    if let Some(store) = read_only {
        if let Ok(response) = issues_on(store, service, after) {
            return Ok(response);
        }
    }
    issues_locked(service, after)
}
fn issues_on(store: &Store, service: &Service, after: &str) -> io::Result<String> {
    let page = store
        .sync_issues(&service.profile.scope.file_id, after)
        .map_err(io::Error::other)?;
    Ok(issues_json(page))
}
fn issues_locked(service: &Service, after: &str) -> io::Result<String> {
    let page = service
        .bridge
        .store
        .lock()
        .unwrap()
        .sync_issues(&service.profile.scope.file_id, after)
        .map_err(io::Error::other)?;
    Ok(issues_json(page))
}
fn issues_json(page: drive_core::conflicts::IssuePage) -> String {
    serde_json::json!({"schema":1,"entries":page.entries,"next":page.next}).to_string()
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
    let ro = read_only(service);
    cache_settings_with(service, ro.as_ref())
}
fn cache_settings_with(service: &Service, read_only: Option<&Store>) -> io::Result<String> {
    // Usage accounting may scan every cached candidate; keep that read off the
    // global writer mutex too. A read-only handle sees the committed database
    // but not this process's open handles, so fall back when it cannot read.
    // It also cannot see files or drafts the writer has open, and those must not
    // be reported as reclaimable while the trimmer protects them. Only project
    // off-lock when nothing is open so protected/reclaimable stays identical to
    // the serialized projection.
    let usage = read_only
        .filter(|_| service.bridge.store.open_handle_count() == 0)
        .and_then(|store| {
            let active = service.bridge.active_downloads().into_iter().collect();
            store.cache_usage(unix_now(), &active).ok()
        });
    let usage = match usage {
        Some(usage) => usage,
        None => service.bridge.cache_usage()?,
    };
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

#[cfg(test)]
mod readpath_tests {
    //! The private read-only handle must be interchangeable with the serialized
    //! writer path: identical JSON, and `None` exercises the exact branch taken
    //! when `Store::open_read_only` fails.
    use super::*;
    use drive_core::{Node, NodeMetadata, native::Scope};
    use std::path::PathBuf;

    fn node(id: &str, parent: Option<&str>, directory: bool) -> Node {
        Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: id.into(),
            version: "v1".into(),
            directory,
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: if directory { 0o755 } else { 0o644 },
            }),
        }
    }
    fn profile() -> crate::service::Profile {
        crate::service::Profile {
            schema: 1,
            task_id: "9f6b8a52-5f4c-4c5e-9d0a-2c1b3d4e5f60".into(),
            host: "example.invalid".into(),
            tls_name: "example.invalid".into(),
            trusted_ca_pem: Vec::new(),
            username: "test".into(),
            device_id: "1f6b8a52-5f4c-4c5e-9d0a-2c1b3d4e5f61".into(),
            scope: Scope {
                view_id: 1,
                node_id: 1,
                file_id: "root".into(),
                path: "/test".into(),
            },
            mount: PathBuf::from("/tmp/drive-native-readpath-mount"),
            writable: false,
            local_policy: None,
            writable_source: None,
        }
    }
    fn fixture() -> (tempfile::TempDir, Arc<Service>) {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        store
            .install_remote_snapshot(
                &[
                    node("root", None, true),
                    node("folder", Some("root"), true),
                    node("one", Some("folder"), false),
                ],
                "1",
                "server",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        (
            dir,
            Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap(),
        )
    }
    fn active(service: &Service) -> HashSet<String> {
        service.bridge.active_downloads().into_iter().collect()
    }

    #[test]
    fn read_only_handle_opens_next_to_the_live_writer_and_rejects_writes() {
        let (_dir, service) = fixture();
        let mut ro = read_only(&service).expect("read-only handle must open next to the writer");
        assert_eq!(ro.root(), service.bridge.store.root().as_path());
        assert!(ro.cache_limit().is_ok());
        // Read-only means writes are rejected instead of racing the writer.
        assert!(ro.set_cache_limit(1024).is_err());
    }

    #[test]
    fn fallback_states_path_matches_the_locked_projection() {
        let (_dir, service) = fixture();
        let paths = vec![
            "folder".to_string(),
            "folder/one".to_string(),
            "missing".to_string(),
        ];
        let active = active(&service);
        let locked = states_locked(&service, paths.clone(), &active).unwrap();
        // `None` is the exact branch taken when the read-only handle cannot open.
        let fallback = states_with(&service, paths.clone(), &active, None).unwrap();
        assert_eq!(locked, fallback);
        let ro = read_only(&service).expect("read-only handle");
        let off_lock = states_with(&service, paths.clone(), &active, Some(&ro)).unwrap();
        assert_eq!(locked, off_lock);
        let value: serde_json::Value = serde_json::from_str(&off_lock).unwrap();
        assert_eq!(value["schema"], 1);
        assert_eq!(value["entries"].as_array().unwrap().len(), 3);
        assert_eq!(value["entries"][2]["error"], "path_or_local_state_unavailable");
        // File-only batches have no StatusTree; `entry` then reads status/pins
        // straight from the handle argument, so compare that branch as well.
        let files = vec!["folder/one".to_string(), "missing".to_string()];
        let locked_files = states_locked(&service, files.clone(), &active).unwrap();
        let off_lock_files = states_with(&service, files.clone(), &active, Some(&ro)).unwrap();
        assert_eq!(locked_files, off_lock_files);
        let file_value: serde_json::Value = serde_json::from_str(&off_lock_files).unwrap();
        assert_eq!(file_value["entries"][0]["id"], "one");
        assert_eq!(file_value["entries"][0]["directory"], false);
    }

    #[test]
    fn fallback_issues_path_matches_the_locked_page() {
        let (_dir, service) = fixture();
        assert_eq!(
            issues_locked(&service, "").unwrap(),
            issues_with(&service, "", None).unwrap()
        );
        let ro = read_only(&service).expect("read-only handle");
        assert_eq!(
            issues_locked(&service, "").unwrap(),
            issues_with(&service, "", Some(&ro)).unwrap()
        );
        let value: serde_json::Value =
            serde_json::from_str(&issues_with(&service, "", Some(&ro)).unwrap()).unwrap();
        assert_eq!(value["schema"], 1);
        assert!(value["entries"].is_array());
        assert!(value.get("next").is_some());
        // A rejected cursor keeps the same error on both paths.
        let cursor = "x".repeat(8193);
        assert!(issues_with(&service, &cursor, None).is_err());
        assert!(issues_with(&service, &cursor, Some(&ro)).is_err());
    }

    #[test]
    fn read_only_cache_usage_matches_the_locked_projection() {
        let (_dir, service) = fixture();
        let ro = read_only(&service).expect("read-only handle");
        let off_lock: serde_json::Value =
            serde_json::from_str(&cache_settings_with(&service, Some(&ro)).unwrap()).unwrap();
        let locked: serde_json::Value =
            serde_json::from_str(&cache_settings_with(&service, None).unwrap()).unwrap();
        assert_eq!(off_lock["schema"], 1);
        assert_eq!(off_lock["task_id"], service.profile.task_id);
        // No open handles exist in this fixture, so usage must agree exactly.
        assert_eq!(off_lock["usage"], locked["usage"]);
        assert_eq!(
            off_lock["recent_download_seconds"],
            drive_core::cache_policy::RECENT_DOWNLOAD_SECONDS
        );
    }

    // ---- issue #8: status projection batches ---------------------------------
    fn store_fixture() -> (tempfile::TempDir, Store) {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        store
            .install_remote_snapshot(
                &[
                    node("root", None, true),
                    node("folderA", Some("root"), true),
                    node("subA", Some("folderA"), true),
                    node("oneA", Some("folderA"), false),
                    node("deep", Some("subA"), false),
                    node("folderB", Some("root"), true),
                    node("twoB", Some("folderB"), false),
                ],
                "1",
                "server",
            )
            .unwrap();
        store.ensure_fs_root("root").unwrap();
        store.pin("oneA", true).unwrap();
        (dir, store)
    }
    fn visited(tree: &super::status_tree::StatusTree<'_>, roots: &[String]) -> Vec<String> {
        let mut seen: Vec<String> = Vec::new();
        let mut pending = roots.to_vec();
        while let Some(id) = pending.pop() {
            if seen.contains(&id) {
                continue;
            }
            seen.push(id.clone());
            pending.extend(tree.children(&id).into_iter().map(|(child, _)| child));
        }
        // Row order depends on which SQL read produced the seeds; the batch only
        // promises the same membership.
        seen.sort();
        seen
    }
    fn children_sorted(
        tree: &super::status_tree::StatusTree<'_>,
        id: &str,
    ) -> Vec<(String, bool)> {
        let mut children = tree.children(id);
        children.sort();
        children
    }
    /// Comparable shape of a detail row without requiring `DownloadFailure: PartialEq`.
    fn detail_shape(
        tree: &super::status_tree::StatusTree<'_>,
        id: &str,
    ) -> Option<Result<(String, bool, bool), ()>> {
        tree.detail(id).map(|detail| {
            detail.map(|(status, offline, failure)| {
                (format!("{status:?}"), offline, failure.is_some())
            })
        })
    }

    #[test]
    fn projected_tree_matches_the_scoped_tree() {
        let (_dir, store) = store_fixture();
        let projection = StatusProjection::from_rows(store.desktop_status_seeds().unwrap(), 3);
        for roots in [
            vec!["root".to_string()],
            vec!["folderA".to_string()],
            vec!["subA".to_string()],
            vec!["folderA".to_string(), "folderB".to_string()],
            vec!["oneA".to_string()],
        ] {
            let mut scoped = super::status_tree::StatusTree::load(&store, &roots).unwrap();
            let mut projected =
                super::status_tree::StatusTree::load_with_projection(&store, &projection, &roots)
                    .unwrap();
            let ids = visited(&scoped, &roots);
            assert_eq!(ids, visited(&projected, &roots));
            for id in ids {
                assert_eq!(
                    children_sorted(&scoped, &id),
                    children_sorted(&projected, &id),
                    "children of {id}"
                );
                assert_eq!(
                    scoped.flags(&id).ok(),
                    projected.flags(&id).ok(),
                    "flags of {id}"
                );
                assert_eq!(
                    detail_shape(&scoped, &id),
                    detail_shape(&projected, &id),
                    "detail of {id}"
                );
            }
        }
        // A projection that cannot describe a root (or an ancestor of it) must be
        // rejected so callers fall back instead of reporting a shrunken tree.
        let without_folder_a: Vec<_> = store
            .desktop_status_seeds()
            .unwrap()
            .into_iter()
            .filter(|row| row.id != "folderA")
            .collect();
        let partial = StatusProjection::from_rows(without_folder_a, 1);
        assert!(
            super::status_tree::StatusTree::load_with_projection(
                &store,
                &partial,
                &["subA".to_string()]
            )
            .is_err()
        );
        assert!(
            super::status_tree::StatusTree::load_with_projection(
                &store,
                &projection,
                &["missing".to_string()]
            )
            .is_err()
        );
    }

    #[test]
    fn projected_states_json_matches_the_scoped_render() {
        let (_dir, service) = fixture();
        let store = service.bridge.store.lock().unwrap();
        let projection = StatusProjection::from_rows(store.desktop_status_seeds().unwrap(), 2);
        let active = active(&service);
        let paths = vec![
            ".".to_string(),
            "folder".to_string(),
            "folder/one".to_string(),
            "missing".to_string(),
        ];
        let resolved = resolve_all(&store, &service, paths.clone());
        let mut scoped = load_tree(&store, &resolved).unwrap();
        let scoped_json = render_states(resolved, &active, Some(&store), &mut scoped);
        let resolved = resolve_all(&store, &service, paths);
        let mut projected = load_tree_with_projection(&store, &projection, &resolved).unwrap();
        let projected_json = render_states(resolved, &active, Some(&store), &mut projected);
        assert_eq!(scoped_json, projected_json);
        // File-only batches build no tree on either path.
        let files = vec!["folder/one".to_string(), "missing".to_string()];
        let resolved_files = resolve_all(&store, &service, files.clone());
        let mut scoped_files = load_tree(&store, &resolved_files).unwrap();
        let scoped_files_json =
            render_states(resolved_files, &active, Some(&store), &mut scoped_files);
        let resolved_files = resolve_all(&store, &service, files);
        let mut projected_files =
            load_tree_with_projection(&store, &projection, &resolved_files).unwrap();
        let projected_files_json =
            render_states(resolved_files, &active, Some(&store), &mut projected_files);
        assert_eq!(scoped_files_json, projected_files_json);
        assert!(scoped_files.is_none() && projected_files.is_none());
    }

    #[test]
    fn stale_projection_falls_back_to_the_scoped_read() {
        let (_dir, service) = fixture();
        let stale = {
            let store = service.bridge.store.lock().unwrap();
            StatusProjection::from_rows(store.desktop_status_seeds().unwrap(), 1)
        };
        {
            let mut store = service.bridge.store.lock().unwrap();
            store
                .install_remote_snapshot(
                    &[
                        node("root", None, true),
                        node("folder", Some("root"), true),
                        node("lateDir", Some("folder"), true),
                    ],
                    "2",
                    "server",
                )
                .unwrap();
        }
        let ro = read_only(&service).unwrap();
        let resolved = resolve_all(&ro, &service, vec!["folder/lateDir".to_string()]);
        assert!(
            super::status_tree::StatusTree::load_with_projection(
                &ro,
                &stale,
                &["lateDir".to_string()]
            )
            .is_err(),
            "a projection that predates the root must be rejected"
        );
        let mut fallback = load_tree_with_projection(&ro, &stale, &resolved)
            .unwrap()
            .expect("fallback tree");
        let mut scoped = load_tree(&ro, &resolved).unwrap().expect("scoped tree");
        for id in ["lateDir"] {
            assert_eq!(fallback.flags(id).ok(), scoped.flags(id).ok());
            assert_eq!(fallback.children(id), scoped.children(id));
            assert_eq!(detail_shape(&fallback, id), detail_shape(&scoped, id));
        }
    }

    #[test]
    fn ready_projection_serves_states_while_the_store_lock_is_held() {
        let (_dir, service) = fixture();
        let paths = vec![
            ".".to_string(),
            "folder".to_string(),
            "folder/one".to_string(),
        ];
        let legacy = states(&service, paths.clone()).unwrap();
        let tasks = service.clone().start();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while service.status_projection().is_none() {
            assert!(
                std::time::Instant::now() < deadline,
                "status projection did not become ready within 10s"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let with_projection = states(&service, paths.clone()).unwrap();
        assert_eq!(legacy, with_projection);
        // A ready projection must answer a directory batch without the writer mutex.
        let held = service.bridge.store.lock().unwrap();
        let probe = service.clone();
        let probe_paths = paths.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let _ = tx.send(states(&probe, probe_paths));
        });
        let response = rx
            .recv_timeout(std::time::Duration::from_secs(5))
            .expect("GetStates waited on the global Store lock");
        drop(held);
        thread.join().unwrap();
        assert_eq!(with_projection, response.unwrap());
        assert!(
            PROJECTION_TREE_LOADS.load(std::sync::atomic::Ordering::Relaxed) > 0,
            "states kept falling back to the scoped read while a projection was ready"
        );
        let started = std::time::Instant::now();
        service.shutdown();
        tasks.join();
        assert!(
            started.elapsed() < std::time::Duration::from_secs(2),
            "service stop+join exceeded 2s: {:?}",
            started.elapsed()
        );
    }

    /// Timing contrast for the issue #8 report. A GetStates batch on a large tree
    /// pays the scoped seed read once on the legacy path; the published projection
    /// answers from memory. Run with `--nocapture` to see the `PROJECTION_TIMING`
    /// line (debug build; use it as relative evidence, not a release benchmark).
    #[test]
    fn projection_avoids_the_scoped_seed_read_on_a_large_tree() {
        let dir = tempfile::tempdir().unwrap();
        let mut store = Store::open(dir.path()).unwrap();
        let mut nodes = vec![node("root", None, true)];
        for folder in 0..100 {
            let folder_id = format!("dir{folder:03}");
            nodes.push(node(&folder_id, Some("root"), true));
            for file in 0..200 {
                let file_id = format!("file{folder:03}-{file:03}");
                nodes.push(node(&file_id, Some(&folder_id), false));
            }
        }
        let node_count = nodes.len();
        store.install_remote_snapshot(&nodes, "1", "server").unwrap();
        store.ensure_fs_root("root").unwrap();
        let service =
            Service::new(profile(), store, PathBuf::from("/nonexistent-worker")).unwrap();
        let ro = read_only(&service).expect("read-only handle");
        let rows = {
            let store = service.bridge.store.lock().unwrap();
            store.desktop_status_seeds().unwrap()
        };
        let seed_rows = rows.len();
        let projection = StatusProjection::from_rows(rows, 1);
        let roots = vec!["root".to_string()];
        let started = std::time::Instant::now();
        let scoped = super::status_tree::StatusTree::load(&ro, &roots).unwrap();
        let scoped_load_ms = started.elapsed().as_millis();
        let started = std::time::Instant::now();
        let projected =
            super::status_tree::StatusTree::load_with_projection(&ro, &projection, &roots).unwrap();
        let projected_load_ms = started.elapsed().as_millis();
        let ids = visited(&scoped, &roots);
        assert_eq!(ids, visited(&projected, &roots));
        for id in &ids {
            assert_eq!(
                children_sorted(&scoped, id),
                children_sorted(&projected, id),
                "children of {id}"
            );
        }
        let paths = vec![".".to_string()];
        let started = std::time::Instant::now();
        let legacy = states(&service, paths.clone()).unwrap();
        let legacy_ms = started.elapsed().as_millis();
        let tasks = service.clone().start();
        let deadline = std::time::Instant::now() + std::time::Duration::from_secs(10);
        while service.status_projection().is_none() {
            assert!(
                std::time::Instant::now() < deadline,
                "status projection did not become ready within 10s"
            );
            std::thread::sleep(std::time::Duration::from_millis(20));
        }
        let started = std::time::Instant::now();
        let cached = states(&service, paths.clone()).unwrap();
        let cached_ms = started.elapsed().as_millis();
        assert_eq!(legacy, cached);
        // The writer mutex is held for the whole request here; a projection-served
        // batch must stay bounded instead of queueing behind the writer.
        let held = service.bridge.store.lock().unwrap();
        let started = std::time::Instant::now();
        let held_json = states(&service, paths.clone()).unwrap();
        let held_lock_ms = started.elapsed().as_millis();
        drop(held);
        assert_eq!(legacy, held_json);
        assert!(held_lock_ms < 5_000, "held-lock batch took {held_lock_ms}ms");
        eprintln!(
            "PROJECTION_TIMING nodes={node_count} seed_rows={seed_rows} \
             scoped_load_ms={scoped_load_ms} projected_load_ms={projected_load_ms} \
             legacy_states_ms={legacy_ms} cached_states_ms={cached_ms} \
             cached_states_with_writer_lock_held_ms={held_lock_ms}"
        );
        service.shutdown();
        tasks.join();
    }

    fn overview_fields(body: &str) -> serde_json::Value {
        serde_json::from_str(body).unwrap()
    }

    #[test]
    fn overview_answers_while_the_writer_lock_is_held() {
        let (_dir, service) = fixture();
        let held = service.bridge.store.lock().unwrap();
        let probe = service.clone();
        let (tx, rx) = std::sync::mpsc::channel();
        let thread = std::thread::spawn(move || {
            let _ = tx.send(overview_measured(&probe));
        });
        // Issue #21: GetOverview used to take the global Store mutex, so a held
        // writer (or a long FUSE operation) stalled every desktop poll.
        let response = rx
            .recv_timeout(std::time::Duration::from_secs(2))
            .expect("GetOverview waited on the global Store lock");
        drop(held);
        thread.join().unwrap();
        let (body, timings) = response.unwrap();
        assert!(timings.read_only, "overview fell back to the writer lock");
        assert_eq!(timings.lock_wait_us, 0);
        assert_eq!(timings.lock_hold_us, 0);
        let value = overview_fields(&body);
        assert_eq!(value["schema"], 1);
        assert_eq!(value["issue_count"], 0);
        assert_eq!(value["cache_availability"]["total"], 1);
    }

    #[test]
    fn overview_off_lock_snapshot_matches_the_locked_snapshot() {
        let (_dir, service) = fixture();
        let locked = {
            let guard = service.bridge.store.lock().unwrap();
            overview_store(&service, &guard, None).unwrap()
        };
        let (body, timings) = overview_measured(&service).unwrap();
        assert!(timings.read_only, "expected the read-only path");
        let value = overview_fields(&body);
        assert_eq!(value["pending_operations"], locked.pending);
        assert_eq!(value["preview_only"], locked.preview_only);
        assert_eq!(value["writable"], locked.writable && !locked.preview_only);
        assert_eq!(value["issue_count"], serde_json::json!(locked.issue_count));
        assert_eq!(value["cache_availability"]["state"], locked.cache.state);
        assert_eq!(value["cache_availability"]["total"], locked.cache.total);
        assert_eq!(value["cache_availability"]["offline"], locked.cache.offline);
        assert_eq!(value["cache_availability"]["pinned"], locked.cache.pinned);
        assert_eq!(
            value["queue_health"]["namespace_uncertain"],
            serde_json::json!(locked.queue_health.namespace_uncertain)
        );
        assert_eq!(
            value["skipped_symlinks"],
            serde_json::json!(locked.skipped_symlinks)
        );
        assert_eq!(
            value["skipped_permission_directories"],
            serde_json::json!(locked.inaccessible)
        );
        assert_eq!(value["download_errors"], serde_json::json!(locked.download_errors));
        assert_eq!(
            value["local_policy"],
            serde_json::json!(locked.permission.local_policy)
        );
    }

    #[test]
    fn overview_reuses_and_drops_counters_with_the_committed_generation() {
        let (_dir, service) = fixture();
        let (first, first_timings) = overview_measured(&service).unwrap();
        assert!(first_timings.read_only);
        assert!(
            !first_timings.derived_cached,
            "first overview has no committed generation to reuse"
        );
        let (second, second_timings) = overview_measured(&service).unwrap();
        assert!(
            second_timings.derived_cached,
            "an unchanged generation must reuse the expensive counters"
        );
        assert_eq!(overview_fields(&first)["issue_count"], 0);
        assert_eq!(
            overview_fields(&first)["cache_availability"],
            overview_fields(&second)["cache_availability"]
        );
        // Any committed write invalidates the cached pair; the new generation
        // must report its own counters (pin changes the availability summary).
        let before_revision = service.bridge.store.local_revision();
        service
            .bridge
            .store
            .lock()
            .unwrap()
            .pin("one", true)
            .unwrap();
        let (third, third_timings) = overview_measured(&service).unwrap();
        assert!(
            !third_timings.derived_cached,
            "a committed write must drop the generation-keyed counters"
        );
        let value = overview_fields(&third);
        assert_eq!(value["cache_availability"]["pinned"], 1);
        assert_eq!(
            value["local_revision"].as_u64().unwrap(),
            service.bridge.store.local_revision()
        );
        assert!(service.bridge.store.local_revision() > before_revision);
    }
}
