//! Read-only desktop projection. Never used by pin/release or write commands.
use super::*;
use drive_core::preview::PreviewRecord;

/// UI routing only, not permission to disclose candidate metadata. A denied or
/// not-yet-confirmed root must keep diagnostics available and actions disabled.
pub(super) fn active(store: &Store, root: &str) -> io::Result<bool> {
    match store.node(root) {
        Ok(_) => Ok(false),
        Err(error) if error.is_not_found() => Ok(store
            .initialization_status()
            .map_err(preview_error)?
            .is_some_and(|status| status.state != "ready")),
        Err(error) => Err(preview_error(error)),
    }
}

fn resolve_preview(
    store: &Store,
    root: &str,
    mount: &Path,
    input: &str,
) -> io::Result<PreviewRecord> {
    if input.len() > 8192 || input.contains('\0') {
        return Err(io::Error::from_raw_os_error(libc::EINVAL));
    }
    let path = Path::new(input);
    let relative = if path.is_absolute() {
        path.strip_prefix(mount)
            .map_err(|_| io::Error::from_raw_os_error(libc::EINVAL))?
    } else {
        path
    };
    let mut record = store
        .preview_mount_root(root)
        .map_err(preview_error)?
        .ok_or_else(|| io::Error::from_raw_os_error(libc::ESTALE))?;
    for (depth, part) in relative.components().enumerate() {
        if depth >= 128 {
            return Err(io::Error::from_raw_os_error(libc::ELOOP));
        }
        match part {
            Component::CurDir => {}
            Component::Normal(name) => {
                let name = name
                    .to_str()
                    .ok_or_else(|| io::Error::from_raw_os_error(libc::EINVAL))?;
                if !record.directory {
                    return Err(io::Error::from_raw_os_error(libc::ENOTDIR));
                }
                record = store
                    .preview_lookup(record.ino, name)
                    .map_err(preview_error)?;
            }
            _ => return Err(io::Error::from_raw_os_error(libc::EINVAL)),
        }
    }
    Ok(record)
}
fn preview_error(error: drive_core::Error) -> io::Error {
    if error.is_not_found() {
        return io::Error::from_raw_os_error(libc::ENOENT);
    }
    match error {
        drive_core::Error::Io(e) => e,
        _ => other(),
    }
}
fn row(path: String, record: PreviewRecord) -> PathState {
    PathState {
        path,
        entry: Some(EntryState {
            id: record.id,
            directory: record.directory,
            status: "initializing",
            pinned: false,
            offline_available: false,
            sync_error: None,
        }),
        error: None,
    }
}
pub(super) fn states(
    store: &Store,
    root: &str,
    mount: &Path,
    paths: Vec<String>,
    rules: Option<&drive_core::exclude::ExcludeRules>,
) -> io::Result<String> {
    let entries: Vec<_> = paths
        .into_iter()
        .map(|path| match resolve_preview(store, root, mount, &path) {
            // Issue #50: excluded candidate nodes stay invisible even while the
            // namespace is still initializing.
            Ok(record)
                if rules.is_some_and(|rules| {
                    path_excluded(rules, mount, &path, record.directory)
                }) =>
            {
                PathState {
                    path,
                    entry: None,
                    error: Some("preview_not_available"),
                }
            }
            Ok(record) => row(path, record),
            Err(_) => PathState {
                path,
                entry: None,
                error: Some("preview_not_available"),
            },
        })
        .collect();
    Ok(serde_json::json!({"schema":1,"preview_only":true,"entries":entries}).to_string())
}
pub(super) fn directory(
    store: &Store,
    root: &str,
    mount: &Path,
    path: &str,
    offset: u32,
    rules: Option<&drive_core::exclude::ExcludeRules>,
) -> io::Result<String> {
    let page = resolve_preview(store, root, mount, path).and_then(|node| {
        if rules.is_some_and(|rules| path_excluded(rules, mount, path, node.directory)) {
            return Err(excluded_error());
        }
        store
            .preview_children_page(node.ino, offset as usize, 256)
            .map_err(preview_error)
    });
    match page {
        Ok(page) => {
            let entries: Vec<_> = page
                .entries
                .into_iter()
                .filter(|record| {
                    !rules.is_some_and(|rules| {
                        path_excluded(
                            rules,
                            mount,
                            &child_path(path, &record.name),
                            record.directory,
                        )
                    })
                })
                .map(|record| {
                    let child_path = child_path(path, &record.name);
                    row(child_path, record)
                })
                .collect();
            Ok(
                serde_json::json!({"schema":1,"preview_only":true,"directory_state":"received",
                "preview_generation":page.stamp.fingerprint(),"entries":entries,
                "total":page.total,"offset":offset,"path":path})
                .to_string(),
            )
        }
        Err(error) => {
            let state = match error.raw_os_error() {
                Some(libc::EAGAIN) | Some(libc::ESTALE) => "pending",
                Some(libc::EACCES) => "denied",
                _ => return Err(error),
            };
            // null means unknown, never a successfully enumerated empty list.
            Ok(
                serde_json::json!({"schema":1,"preview_only":true,"directory_state":state,
                "entries":[],"total":null,"offset":offset,"path":path})
                .to_string(),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use drive_core::{
        NodeMetadata,
        initialization_events::InitializationDirectoryEntry,
        initialization_store::{InitializationBinding, InitializationBudget, InitializationRun},
    };
    fn node(id: &str, parent: Option<&str>, name: &str, directory: bool) -> Node {
        Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: name.into(),
            directory,
            version:
                serde_json::json!({"schema":1,"restore":"server","view":1,"root":"100","file":id,
                "sync":100,"max":100,"size":3,"hash":"synthetic","directory":directory})
                .to_string(),
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 100,
                mode: if directory { 0o755 } else { 0o644 },
            }),
        }
    }
    fn setup() -> (tempfile::TempDir, Store, InitializationRun) {
        let temp = tempfile::tempdir().unwrap();
        let mut store = Store::open(temp.path()).unwrap();
        let scope = serde_json::json!({"view_id":1,"node_id":2,"file_id":"100","path":"/Lab"});
        store
            .bind_native_scope("server", &scope.to_string())
            .unwrap();
        let binding = InitializationBinding {
            task_id: "preview-desktop".into(),
            scope,
            restore: "server".into(),
            start_cursor: "native-v1:100".into(),
            expected_cursor: "".into(),
rules_digest: String::new(),
        };
        let run = store
            .begin_initialization(
                &binding,
                &node("100", None, "Lab", true),
                InitializationBudget::default(),
            )
            .unwrap();
        (temp, store, run)
    }
    fn list(store: &mut Store, run: &InitializationRun, children: Vec<Node>) {
        let ticket = store.begin_initialization_directory(run, "100").unwrap();
        let entries = children
            .into_iter()
            .enumerate()
            .map(|(i, node)| InitializationDirectoryEntry {
                node,
                native_node_id: 1000 + i as u64,
            })
            .collect::<Vec<_>>();
        store
            .store_initialization_native_directory(&ticket, 1, &entries)
            .unwrap();
    }
    fn data(value: io::Result<String>) -> serde_json::Value {
        serde_json::from_str(&value.unwrap()).unwrap()
    }

    #[test]
    fn unready_permission_and_preview_status_remain_distinct_from_authoritative_empty_state() {
        let (_temp, mut store, run) = setup();
        let mount = Path::new("/tmp/preview-desktop");
        let pending = data(directory(&store, "100", mount, "", 0, None));
        assert_eq!(pending["directory_state"], "pending");
        assert!(pending["total"].is_null());
        list(
            &mut store,
            &run,
            vec![
                node("200", Some("100"), "Dir", true),
                node("101", Some("100"), "A.doc", false),
            ],
        );
        let received = data(directory(&store, "100", mount, "", 0, None));
        assert_eq!(received["total"], 2);
        assert_eq!(received["directory_state"], "received");
        for entry in received["entries"].as_array().unwrap() {
            assert_eq!(entry["status"], "initializing");
            assert_eq!(entry["offline_available"], false);
        }
        assert!(store.node("100").is_err());
        assert_eq!(store.pending_operations().unwrap(), 0);
        let child = data(directory(&store, "100", mount, "Dir", 0, None));
        assert!(child["total"].is_null());
        let ticket = store.begin_initialization_directory(&run, "200").unwrap();
        store.store_initialization_denied(&ticket, 1).unwrap();
        let denied = data(directory(&store, "100", mount, "Dir", 0, None));
        assert_eq!(denied["directory_state"], "denied");
        assert!(denied["total"].is_null());
        let paths = data(states(
            &store,
            "100",
            mount,
            vec![
                "A.doc".into(),
                "../outside".into(),
                "/tmp/preview-desktop-other/A.doc".into(),
            ],
            None,
        ));
        assert_eq!(paths["entries"][0]["status"], "initializing");
        assert!(paths["entries"][1]["error"].is_string());
        assert!(paths["entries"][2]["error"].is_string());
        store.revoke_initialization_preview_root(&run).unwrap();
        assert!(active(&store, "100").unwrap());
        let denied_root = data(directory(&store, "100", mount, "", 0, None));
        assert_eq!(denied_root["directory_state"], "denied");
        assert!(denied_root["total"].is_null());
        assert!(denied_root["entries"].as_array().unwrap().is_empty());
    }

    #[test]
    fn desktop_pages_share_a_generation_until_the_direct_listing_is_replaced() {
        let (_temp, mut store, run) = setup();
        let mount = Path::new("/tmp/preview-desktop");
        let nodes = (0..300)
            .map(|i| {
                node(
                    &(1000 + i).to_string(),
                    Some("100"),
                    &format!("file-{i:03}"),
                    false,
                )
            })
            .collect::<Vec<_>>();
        list(&mut store, &run, nodes.clone());
        let first = data(directory(&store, "100", mount, "", 0, None));
        let second = data(directory(&store, "100", mount, "", 256, None));
        assert_eq!(first["entries"].as_array().unwrap().len(), 256);
        assert_eq!(second["entries"].as_array().unwrap().len(), 44);
        assert_eq!(first["preview_generation"], second["preview_generation"]);
        list(&mut store, &run, nodes);
        let newer = data(directory(&store, "100", mount, "", 256, None));
        assert_ne!(first["preview_generation"], newer["preview_generation"]);
        assert!(store.node("100").is_err());
        assert_eq!(store.cursor().unwrap(), "");
    }
}
