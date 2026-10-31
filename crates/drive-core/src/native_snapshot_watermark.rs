//! A current event boundary is only a scan starting point, never a checkpoint.
use super::*;

impl NativeReadSession {
    pub(super) async fn current_snapshot_seed(&self, attempt: u8) -> Result<SnapshotSeed> {
        let watch = self.watch("")?;
        capture_current_seed(
            &self.scope,
            self.client.restore_identity(),
            attempt,
            || async { Ok(self.client.current_sync_watermark(&watch).await?) },
            || async {
                let info = self
                    .client
                    .current_selected_node(
                        self.scope.view_id,
                        &self.scope.path,
                        &self.scope.file_id,
                        self.scope.node_id,
                    )
                    .await?
                    .ok_or(Error::RootUnavailable)?;
                self.snapshot_node(&self.scope.file_id, &info)
            },
        )
        .await
    }
}

/// Order matters: capture S0 before reading current metadata. Enumeration and
/// the later event fence cover changes after S0. Never obtain a later S0 and
/// attach it to previously read metadata, or substitute it for committed state.
async fn capture_current_seed<W, WF, R, RF>(
    scope: &Scope,
    restore: &str,
    attempt: u8,
    watermark: W,
    root: R,
) -> Result<SnapshotSeed>
where
    W: FnOnce() -> WF,
    WF: std::future::Future<Output = Result<EventCursor>>,
    R: FnOnce() -> RF,
    RF: std::future::Future<Output = Result<Node>>,
{
    capture_current_seed_until(
        scope,
        restore,
        attempt,
        watermark,
        root,
        tokio::time::Instant::now() + std::time::Duration::from_secs(900),
    )
    .await
}

async fn capture_current_seed_until<W, WF, R, RF>(
    scope: &Scope,
    restore: &str,
    attempt: u8,
    watermark: W,
    root: R,
    deadline: tokio::time::Instant,
) -> Result<SnapshotSeed>
where
    W: FnOnce() -> WF,
    WF: std::future::Future<Output = Result<EventCursor>>,
    R: FnOnce() -> RF,
    RF: std::future::Future<Output = Result<Node>>,
{
    scope.validate()?;
    if scope.path == "/" || restore.is_empty() {
        return Err(Error::Unsupported(
            "current root metadata contract unavailable",
        ));
    }
    emit(Progress {
        attempt,
        phase: Phase::Watermark,
        units: 0,
        items: 0,
    });
    let cursor = tokio::time::timeout_at(deadline, watermark())
        .await
        .map_err(|_| Error::Native(sproto::Error::Timeout("current snapshot seed deadline")))??;
    emit(Progress {
        attempt,
        phase: Phase::Watermark,
        units: 1,
        items: 0,
    });
    let node = tokio::time::timeout_at(deadline, root())
        .await
        .map_err(|_| Error::Native(sproto::Error::Timeout("current snapshot seed deadline")))??;
    let token: VersionToken = serde_json::from_str(&node.version)?;
    if node.id != scope.file_id
        || !native_file_id(&node.id)
        || node.parent.is_some()
        || !node.directory
        || scope.path.rsplit('/').next() != Some(node.name.as_str())
        || token.schema != 1
        || token.restore != restore
        || token.view != scope.view_id
        || token.root != scope.file_id
        || token.file != scope.file_id
        || !token.directory
        || token.sync == 0
        || token.max == 0
        || node.name.len() > 255
    {
        return Err(Error::RootUnavailable);
    }
    crate::state::valid_node(&node)?;
    emit(Progress {
        attempt,
        phase: Phase::Watermark,
        units: 2,
        items: 1,
    });
    Ok(SnapshotSeed {
        scope: scope.clone(),
        restore: restore.into(),
        cursor,
        root: node,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    fn scope() -> Scope {
        Scope {
            view_id: 5,
            node_id: 7,
            file_id: "123456789".into(),
            path: "/selected".into(),
        }
    }
    fn root(scope: &Scope) -> Node {
        Node {
            id: scope.file_id.clone(),
            parent: None,
            name: "selected".into(),
            directory: true,
            version: serde_json::to_string(&VersionToken {
                schema: 1,
                restore: "restore".into(),
                view: scope.view_id,
                root: scope.file_id.clone(),
                file: scope.file_id.clone(),
                sync: 8,
                max: 8,
                size: 0,
                hash: String::new(),
                directory: true,
            })
            .unwrap(),
            metadata: Some(crate::NodeMetadata {
                size: 0,
                mtime: 8,
                mode: 0o755,
            }),
        }
    }
    #[tokio::test]
    async fn history_length_does_not_add_requests_and_boundary_precedes_root_read() {
        for boundary in [8, 10_000, u64::MAX - 1] {
            let scope = scope();
            let calls = Arc::new(AtomicUsize::new(0));
            let a = calls.clone();
            let b = calls.clone();
            let node = root(&scope);
            let seed = capture_current_seed(
                &scope,
                "restore",
                1,
                || async move {
                    assert_eq!(a.fetch_add(1, Ordering::SeqCst), 0);
                    Ok(EventCursor(boundary))
                },
                || async move {
                    assert_eq!(b.fetch_add(1, Ordering::SeqCst), 1);
                    Ok(node)
                },
            )
            .await
            .unwrap();
            assert_eq!(seed.cursor, EventCursor(boundary));
            assert_eq!(calls.load(Ordering::SeqCst), 2);
            assert_eq!(seed.root.id, scope.file_id);
        }
    }
    #[tokio::test]
    async fn unavailable_watermark_never_reads_root_or_falls_back_to_skipping_history() {
        let result = capture_current_seed(
            &scope(),
            "restore",
            1,
            || async { Err(Error::Native(sproto::Error::Timeout("fixture"))) },
            || async { panic!("root read after failed watermark") },
        )
        .await;
        assert!(matches!(
            result,
            Err(Error::Native(sproto::Error::Timeout(_)))
        ));
    }
    #[tokio::test]
    async fn missing_root_never_returns_a_seed() {
        let result = capture_current_seed(
            &scope(),
            "restore",
            1,
            || async { Ok(EventCursor(10)) },
            || async { Err(Error::RootUnavailable) },
        )
        .await;
        assert!(matches!(result, Err(Error::RootUnavailable)));
    }
    #[tokio::test]
    async fn root_and_server_binding_are_required_even_after_successful_watermark() {
        for kind in 0..7 {
            let scope = scope();
            let mut node = root(&scope);
            match kind {
                0 => node.id = "replacement".into(),
                1 => node.parent = Some("outside".into()),
                2 => node.directory = false,
                3 => node.name = "renamed".into(),
                n => {
                    let mut v: VersionToken = serde_json::from_str(&node.version).unwrap();
                    match n {
                        4 => v.restore = "another".into(),
                        5 => v.root = "another".into(),
                        _ => v.view += 1,
                    }
                    node.version = serde_json::to_string(&v).unwrap();
                }
            }
            let result = capture_current_seed(
                &scope,
                "restore",
                1,
                || async { Ok(EventCursor(10)) },
                || async { Ok(node) },
            )
            .await;
            assert!(matches!(result, Err(Error::RootUnavailable)));
        }
    }
    #[tokio::test]
    async fn root_without_parent_query_contract_fails_before_any_request() {
        let mut scope = scope();
        scope.path = "/".into();
        let result = capture_current_seed(
            &scope,
            "restore",
            1,
            || async { panic!("watermark for unsupported root") },
            || async { panic!("metadata for unsupported root") },
        )
        .await;
        assert!(matches!(result, Err(Error::Unsupported(_))));
    }

    #[tokio::test]
    async fn one_deadline_bounds_a_pending_root_listing() {
        let result = capture_current_seed_until(
            &scope(),
            "restore",
            1,
            || async { Ok(EventCursor(10)) },
            std::future::pending,
            tokio::time::Instant::now() + std::time::Duration::from_millis(20),
        )
        .await;
        assert!(matches!(
            result,
            Err(Error::Native(sproto::Error::Timeout(_)))
        ));
    }

    #[tokio::test]
    async fn root_change_after_current_boundary_reseeds_before_final_fence() {
        let scope = scope();
        let seed = capture_current_seed(
            &scope,
            "restore",
            1,
            || async { Ok(EventCursor(10)) },
            || async { Ok(root(&scope)) },
        )
        .await
        .unwrap();
        let update = sproto::actions::native::NativeEvent {
            committer: None,
            kind: EventKind::Modify,
            path: scope.path.clone(),
            file_id: scope.file_id.clone(),
            parent_id: "1".into(),
            version: NativeVersion {
                sync_id: 11,
                max_id: 11,
            },
            file_type: FileType::Dir,
            size: 0,
            hash: String::new(),
            mtime: 11,
            removed: false,
        };
        let snapshot = |seed: &SnapshotSeed| RemoteSnapshot {
            scope: seed.scope.clone(),
            restore: seed.restore.clone(),
            cursor: format!("native-v1:{}", seed.cursor.0),
            nodes: vec![seed.root.clone()],
            inaccessible: vec![],
            skipped_symlinks: 0,
        };
        let declined = super::reconcile::converge(
            snapshot(&seed),
            1,
            |_| {
                std::future::ready(Ok(EventBatch {
                    events: vec![update.clone()],
                    next: EventCursor(11),
                    recovery_required: false,
                }))
            },
            |_, _| async { panic!("directory change must not use file-only local reconciliation") },
        )
        .await
        .unwrap();
        assert!(declined.is_none());
        assert_eq!(seed.cursor, EventCursor(10));
        let mut pages = VecDeque::from([
            EventBatch {
                events: vec![update],
                next: EventCursor(11),
                recovery_required: false,
            },
            EventBatch {
                events: vec![],
                next: EventCursor(11),
                recovery_required: false,
            },
        ]);
        let mut requests = Vec::new();
        let reseeded = collect_snapshot_seed(&scope, "restore", Some(&seed), 2, |cursor| {
            requests.push(cursor.0);
            std::future::ready(Ok(pages.pop_front().unwrap()))
        })
        .await
        .unwrap();
        assert_eq!(requests, [10, 11]);
        assert_ne!(seed.root.version, reseeded.root.version);
        let complete = super::reconcile::converge(
            snapshot(&reseeded),
            2,
            |cursor| {
                std::future::ready(Ok(EventBatch {
                    events: vec![],
                    next: cursor,
                    recovery_required: false,
                }))
            },
            |_, _| async { panic!("unchanged candidate requires no local lists") },
        )
        .await
        .unwrap()
        .unwrap();
        assert_eq!(complete.cursor, "native-v1:11");
        assert_eq!(complete.nodes[0].version, reseeded.root.version);
    }
}
