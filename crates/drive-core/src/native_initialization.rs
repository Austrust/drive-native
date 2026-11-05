//! Replayable, session-bound native reads for the durable initialization owner.
//! Replies are candidates only: the caller commits receipts, replays events and
//! obtains an independent stationary fence before publishing authoritative data.
use super::*;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum InitializationRequest {
    Seed,
    Root,
    Directory { node: Node, path: String },
    Pull { cursor: String },
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationReply {
    pub scope: Scope,
    pub restore: String,
    pub body: InitializationBody,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub enum InitializationBody {
    Seed {
        cursor: String,
        root: Node,
    },
    Root {
        node: Node,
    },
    Directory {
        identity: String,
        children: Vec<InitializationDirectoryEntry>,
        denied: bool,
    },
    Events(Option<InitializationEventPage>),
}

impl NativeReadSession {
    pub async fn initialization(
        &self,
        request: InitializationRequest,
    ) -> Result<InitializationReply> {
        validate_session(&self.scope, self.client.restore_identity())?;
        if self.root_lost.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::RootUnavailable);
        }
        let result = self.initialization_body(request).await;
        let body = self.root_guard_result(result)?;
        // A concurrent pull can establish root loss while a list was in flight.
        if self.root_lost.load(std::sync::atomic::Ordering::Acquire) {
            return Err(Error::RootUnavailable);
        }
        Ok(InitializationReply {
            scope: self.scope.clone(),
            restore: self.client.restore_identity().into(),
            body,
        })
    }

    async fn initialization_body(
        &self,
        request: InitializationRequest,
    ) -> Result<InitializationBody> {
        match request {
            InitializationRequest::Seed => {
                let seed = self.initialization_seed().await?;
                Ok(InitializationBody::Seed {
                    cursor: format!("native-v1:{}", seed.cursor.0),
                    root: seed.root,
                })
            }
            InitializationRequest::Root => {
                // Root refresh is metadata only. Do not fetch a new watermark
                // or replace the persisted candidate's start cursor.
                let node = read_root(
                    &self.scope,
                    self.client.restore_identity(),
                    || {
                        super::super::retry::read(|| {
                            self.client.current_selected_node(
                                self.scope.view_id,
                                &self.scope.path,
                                &self.scope.file_id,
                                self.scope.node_id,
                            )
                        })
                    },
                    || async { Ok(self.legacy_initialization_seed().await?.root) },
                    |info| self.snapshot_node(&self.scope.file_id, info),
                )
                .await?;
                Ok(InitializationBody::Root { node })
            }
            InitializationRequest::Directory { node, path } => {
                validate_directory_request(
                    &self.scope,
                    self.client.restore_identity(),
                    &node,
                    &path,
                )?;
                let result = super::super::retry::list_dir(&self.client, self.scope.view_id, &path).await;
                directory_reply(&self.scope, &node, result, |info| {
                    self.snapshot_node(&node.id, info)
                })
            }
            InitializationRequest::Pull { cursor } => {
                validate_initialization_cursor(&cursor)?;
                // No private worker cursor is advanced; repeat the caller's
                // explicit cursor after lost replies or a worker restart.
                Ok(InitializationBody::Events(
                    self.pull_initialization_page(&cursor).await?,
                ))
            }
        }
    }

    async fn initialization_seed(&self) -> Result<SnapshotSeed> {
        select_seed(
            &self.scope,
            || self.current_snapshot_seed(1),
            || self.legacy_initialization_seed(),
        )
        .await
    }

    async fn legacy_initialization_seed(&self) -> Result<SnapshotSeed> {
        let watch = self.watch("")?;
        collect_snapshot_seed(
            &self.scope,
            self.client.restore_identity(),
            None,
            1,
            |cursor| {
                let mut watch = watch.clone();
                watch.cursor = cursor;
                async move { self.scoped_events(&watch).await }
            },
        )
        .await
    }
}

/// The verified parent-list lookup binds both numeric node and file identity.
/// Only `/` or the known parent-list permission response uses historical seed.
/// Missing/replaced metadata and transport failures never silently downgrade.
async fn read_root<Q, QF, L, LF, C>(
    scope: &Scope,
    restore: &str,
    query: Q,
    legacy: L,
    convert: C,
) -> Result<Node>
where
    Q: FnOnce() -> QF,
    QF: std::future::Future<Output = sproto::Result<Option<NodeInfo>>>,
    L: FnOnce() -> LF,
    LF: std::future::Future<Output = Result<Node>>,
    C: FnOnce(&NodeInfo) -> Result<Node>,
{
    validate_session(scope, restore)?;
    if scope.path == "/" {
        return legacy().await;
    }
    let info = match query().await {
        Ok(Some(info)) => info,
        Ok(None) => return Err(Error::RootUnavailable),
        Err(sproto::Error::Server { code: 12290, .. }) => return legacy().await,
        Err(error) => return Err(Error::Native(error)),
    };
    if info.node_id != scope.node_id
        || info.file_id != scope.file_id
        || info.file_type != FileType::Dir
        || info.is_removed
        || scope.path.rsplit('/').next() != Some(info.name.as_str())
        || info.sync_id == 0
        || info.max_id == 0
    {
        return Err(Error::RootUnavailable);
    }
    let node = convert(&info)?;
    validate_directory_request(scope, restore, &node, &scope.path)?;
    Ok(node)
}

async fn select_seed<C, CF, L, LF>(scope: &Scope, current: C, legacy: L) -> Result<SnapshotSeed>
where
    C: FnOnce() -> CF,
    CF: std::future::Future<Output = Result<SnapshotSeed>>,
    L: FnOnce() -> LF,
    LF: std::future::Future<Output = Result<SnapshotSeed>>,
{
    if scope.path != "/" {
        match current().await {
            Ok(seed) => return Ok(seed),
            Err(Error::Unsupported(watermark_seed::PARENT_QUERY_UNAVAILABLE)) => {}
            Err(error) => return Err(error),
        }
    }
    legacy().await
}

fn validate_session(scope: &Scope, restore: &str) -> Result<()> {
    scope.validate()?;
    if scope.view_id == 0
        || scope.node_id == 0
        || !native_file_id(&scope.file_id)
        || restore.is_empty()
    {
        return Err(Error::State(
            "initialization session identity invalid".into(),
        ));
    }
    Ok(())
}

fn validate_initialization_cursor(cursor: &str) -> Result<()> {
    let raw = cursor
        .strip_prefix("native-v1:")
        .ok_or_else(|| Error::State("invalid initialization cursor".into()))?;
    let value = raw
        .parse::<u64>()
        .map_err(|_| Error::State("invalid initialization cursor".into()))?;
    if value.to_string() != raw {
        return Err(Error::State("noncanonical initialization cursor".into()));
    }
    Ok(())
}

fn validate_directory_request(scope: &Scope, restore: &str, node: &Node, path: &str) -> Result<()> {
    validate_session(scope, restore)?;
    crate::state::valid_node(node)?;
    let token: VersionToken = serde_json::from_str(&node.version)?;
    if !node.directory
        || !token.directory
        || token.schema != 1
        || token.restore != restore
        || token.view != scope.view_id
        || token.root != scope.file_id
        || token.file != node.id
        || !native_file_id(&node.id)
        || token.sync == 0
        || token.max == 0
        || node.name.len() > 255
    {
        return Err(Error::State(
            "initialization directory token mismatch".into(),
        ));
    }
    if path.len() > 16384
        || !path.starts_with('/')
        || path.contains('\0')
        || !scope.contains(path)
        || (path != "/"
            && path[1..]
                .split('/')
                .any(|p| p.is_empty() || matches!(p, "." | "..") || p.len() > 255))
    {
        return Err(Error::State("invalid initialization directory path".into()));
    }
    if node.id == scope.file_id {
        if path != scope.path
            || node.parent.is_some()
            || (path != "/" && path.rsplit('/').next() != Some(node.name.as_str()))
        {
            return Err(Error::RootUnavailable);
        }
    } else {
        let (parent_path, name) = path
            .rsplit_once('/')
            .ok_or_else(|| Error::State("invalid initialization path parent".into()))?;
        let parent_path = if parent_path.is_empty() {
            "/"
        } else {
            parent_path
        };
        let relative = path
            .strip_prefix(scope.path.trim_end_matches('/'))
            .unwrap_or(path);
        if path == scope.path
            || name != node.name
            || !scope.contains(parent_path)
            || node
                .parent
                .as_deref()
                .is_none_or(|id| !native_file_id(id) || id == node.id)
            || (parent_path == scope.path)
                != (node.parent.as_deref() == Some(scope.file_id.as_str()))
            || relative.split('/').filter(|part| !part.is_empty()).count() > 128
        {
            return Err(Error::State(
                "initialization directory parent or path mismatch".into(),
            ));
        }
        // NodeInfo carries no parent identity and the path API is not an
        // atomic identity lookup. The owner must construct deeper paths from
        // its current candidate tree and replay/fence concurrent moves.
    }
    Ok(())
}

fn directory_reply<F>(
    scope: &Scope,
    node: &Node,
    result: sproto::Result<Vec<NodeInfo>>,
    mut convert: F,
) -> Result<InitializationBody>
where
    F: FnMut(&NodeInfo) -> Result<Node>,
{
    let infos = match result {
        Ok(infos) => infos,
        Err(sproto::Error::Server { code: 12290, .. }) if node.id != scope.file_id => {
            return Ok(InitializationBody::Directory {
                identity: node.id.clone(),
                children: vec![],
                denied: true,
            });
        }
        Err(sproto::Error::Server { code: 12290, .. }) => return Err(Error::RootUnavailable),
        Err(error) => return Err(Error::Native(error)),
    };
    if infos.len() > SNAPSHOT_NODE_LIMIT {
        return Err(Error::State("snapshot node budget exceeded".into()));
    }
    let mut children = Vec::new();
    let mut ids = HashSet::new();
    let mut names = HashSet::new();
    for info in infos {
        if info.is_removed {
            continue;
        }
        if info.node_id == 0 || !native_file_id(&info.file_id) {
            return Err(Error::State(
                "initialization listing identity missing".into(),
            ));
        }
        if info.file_id == node.id
            || info.file_id == scope.file_id
            || !ids.insert(info.file_id.clone())
            || !names.insert(info.name.clone())
        {
            return Err(Error::SnapshotChanged);
        }
        // Follow the existing snapshot policy: never traverse a symlink or
        // manufacture an empty regular file for it.
        if info.file_type == FileType::Symlink {
            continue;
        }
        let child = convert(&info)?;
        if child.id != info.file_id || child.parent.as_deref() != Some(node.id.as_str()) {
            return Err(Error::State(
                "initialization child identity mismatch".into(),
            ));
        }
        children.push(InitializationDirectoryEntry {
            node: child,
            native_node_id: info.node_id,
        });
    }
    Ok(InitializationBody::Directory {
        identity: node.id.clone(),
        children,
        denied: false,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    fn scope() -> Scope {
        Scope {
            view_id: 5,
            node_id: 7,
            file_id: "10".into(),
            path: "/selected".into(),
        }
    }
    fn node(id: &str, parent: Option<&str>, name: &str) -> Node {
        Node {
            id: id.into(),
            parent: parent.map(str::to_owned),
            name: name.into(),
            directory: true,
            metadata: None,
            version: serde_json::to_string(&VersionToken {
                schema: 1,
                restore: "server".into(),
                view: 5,
                root: "10".into(),
                file: id.into(),
                sync: 1,
                max: 2,
                size: 0,
                hash: String::new(),
                directory: true,
            })
            .unwrap(),
        }
    }
    fn seed() -> SnapshotSeed {
        SnapshotSeed {
            scope: scope(),
            restore: "server".into(),
            cursor: EventCursor(20),
            root: node("10", None, "selected"),
        }
    }
    #[tokio::test]
    async fn seed_uses_current_contract_and_only_known_parent_denial_falls_back() {
        assert_eq!(
            select_seed(
                &scope(),
                || async { Ok(seed()) },
                || async { panic!("unexpected legacy") }
            )
            .await
            .unwrap()
            .cursor,
            EventCursor(20)
        );
        let got = select_seed(
            &scope(),
            || async { Err(Error::Unsupported(watermark_seed::PARENT_QUERY_UNAVAILABLE)) },
            || async { Ok(seed()) },
        )
        .await
        .unwrap();
        assert_eq!(got.root.id, "10");
        for error in [
            Error::RootUnavailable,
            Error::Native(sproto::Error::ConnectionClosed),
            Error::Unsupported("other contract"),
        ] {
            assert!(
                select_seed(
                    &scope(),
                    || async { Err(error) },
                    || async { panic!("must not hide failure") }
                )
                .await
                .is_err()
            );
        }
        let mut root = scope();
        root.path = "/".into();
        assert!(
            select_seed(
                &root,
                || async { panic!("no current root contract") },
                || async { Ok(seed()) }
            )
            .await
            .is_ok()
        );
    }
    #[test]
    fn directory_requests_bind_token_path_root_and_parent_before_network() {
        let scope = scope();
        let root = node("10", None, "selected");
        validate_directory_request(&scope, "server", &root, "/selected").unwrap();
        let child = node("11", Some("10"), "child");
        validate_directory_request(&scope, "server", &child, "/selected/child").unwrap();
        for path in [
            "/selected2/child",
            "/selected/../child",
            "/selected//child",
            "/selected/child/",
            "relative",
            "/selected/renamed",
            "/selected/deeper/child",
        ] {
            assert!(
                validate_directory_request(&scope, "server", &child, path).is_err(),
                "{path}"
            );
        }
        for field in [
            "schema",
            "restore",
            "view",
            "root",
            "file",
            "directory",
            "sync",
            "max",
        ] {
            let mut bad = child.clone();
            let mut token: serde_json::Value = serde_json::from_str(&bad.version).unwrap();
            token[field] = match field {
                "schema" => 2.into(),
                "restore" | "root" | "file" => "999".into(),
                "view" => 6.into(),
                "directory" => false.into(),
                _ => 0.into(),
            };
            bad.version = serde_json::to_string(&token).unwrap();
            assert!(
                validate_directory_request(&scope, "server", &bad, "/selected/child").is_err(),
                "{field}"
            );
        }
        for parent in [None, Some("11"), Some("outside"), Some("12")] {
            let mut bad = child.clone();
            bad.parent = parent.map(str::to_owned);
            assert!(validate_directory_request(&scope, "server", &bad, "/selected/child").is_err());
        }
        let deep = node("12", Some("11"), "deep");
        validate_directory_request(&scope, "server", &deep, "/selected/child/deep").unwrap();
        assert!(validate_directory_request(&scope, "server", &deep, "/selected/deep").is_err());
        assert!(validate_directory_request(&scope, "other", &root, "/selected").is_err());
    }
    fn info(id: &str, name: &str, kind: FileType) -> NodeInfo {
        NodeInfo {
            node_id: 30,
            sync_id: 1,
            max_id: 2,
            name: name.into(),
            file_size: 0,
            mtime: 0,
            hash: String::new(),
            file_id: id.into(),
            file_type: kind,
            is_removed: false,
        }
    }
    #[tokio::test]
    async fn root_refresh_queries_current_metadata_once_without_watermark_or_legacy() {
        let calls = std::sync::atomic::AtomicUsize::new(0);
        let mut current = info("10", "selected", FileType::Dir);
        current.node_id = 7;
        let got = read_root(
            &scope(),
            "server",
            || async {
                calls.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
                Ok(Some(current))
            },
            || async { panic!("verified current root must not replay history") },
            |_| Ok(node("10", None, "selected")),
        )
        .await
        .unwrap();
        assert_eq!(got.id, "10");
        assert_eq!(calls.load(std::sync::atomic::Ordering::Relaxed), 1);
    }
    #[tokio::test]
    async fn root_refresh_rejects_every_identity_field_and_does_not_hide_transport_errors() {
        for field in 0..7 {
            let mut current = info("10", "selected", FileType::Dir);
            current.node_id = 7;
            match field {
                0 => current.node_id = 8,
                1 => current.file_id = "11".into(),
                2 => current.name = "renamed".into(),
                3 => current.file_type = FileType::File,
                4 => current.is_removed = true,
                5 => current.sync_id = 0,
                _ => current.max_id = 0,
            }
            assert!(matches!(
                read_root(
                    &scope(),
                    "server",
                    || async { Ok(Some(current)) },
                    || async { panic!("invalid identity is not a capability fallback") },
                    |_| panic!("invalid root must not convert")
                )
                .await,
                Err(Error::RootUnavailable)
            ));
        }
        assert!(matches!(
            read_root(
                &scope(),
                "server",
                || async { Ok(None) },
                || async { panic!("missing root is not fallback") },
                |_| panic!()
            )
            .await,
            Err(Error::RootUnavailable)
        ));
        assert!(matches!(
            read_root(
                &scope(),
                "server",
                || async { Err(sproto::Error::ConnectionClosed) },
                || async { panic!("transport is not fallback") },
                |_| panic!()
            )
            .await,
            Err(Error::Native(sproto::Error::ConnectionClosed))
        ));
    }
    #[tokio::test]
    async fn root_refresh_retains_only_verified_legacy_fallbacks() {
        let expected = node("10", None, "selected");
        let got = read_root(
            &scope(),
            "server",
            || async {
                Err(sproto::Error::Server {
                    code: 12290,
                    reason: "private".into(),
                })
            },
            || async { Ok(expected.clone()) },
            |_| panic!(),
        )
        .await
        .unwrap();
        assert_eq!(got, expected);
        let mut slash = scope();
        slash.path = "/".into();
        assert_eq!(
            read_root(
                &slash,
                "server",
                || async { panic!("no verified current root lookup") },
                || async { Ok(expected.clone()) },
                |_| panic!()
            )
            .await
            .unwrap(),
            expected
        );
        assert!(matches!(
            read_root(
                &scope(),
                "server",
                || async {
                    Err(sproto::Error::Server {
                        code: 12291,
                        reason: "private".into(),
                    })
                },
                || async { panic!("other server errors are not parent denial") },
                |_| panic!()
            )
            .await,
            Err(Error::Native(sproto::Error::Server { code: 12291, .. }))
        ));
    }
    #[test]
    fn listing_filters_links_but_never_hides_identity_collisions() {
        let root = node("10", None, "selected");
        let mut removed = info("14", "removed", FileType::File);
        removed.is_removed = true;
        let result = directory_reply(
            &scope(),
            &root,
            Ok(vec![
                info("11", "child", FileType::Dir),
                info("12", "link", FileType::Symlink),
                removed,
            ]),
            |n| Ok(node(&n.file_id, Some("10"), &n.name)),
        )
        .unwrap();
        assert!(
            matches!(result,InitializationBody::Directory{children,denied:false,..} if children.len()==1 && children[0].node.id=="11" && children[0].native_node_id==30)
        );
        for infos in [
            vec![info("10", "root", FileType::Dir)],
            vec![
                info("11", "same", FileType::File),
                info("12", "same", FileType::Symlink),
            ],
            vec![
                info("11", "a", FileType::Dir),
                info("11", "b", FileType::Dir),
            ],
        ] {
            assert!(matches!(
                directory_reply(&scope(), &root, Ok(infos), |n| Ok(node(
                    &n.file_id,
                    Some("10"),
                    &n.name
                ))),
                Err(Error::SnapshotChanged)
            ));
        }
        let mut invalid = info("11", "child", FileType::Dir);
        invalid.node_id = 0;
        assert!(matches!(
            directory_reply(&scope(), &root, Ok(vec![invalid]), |_| panic!()),
            Err(Error::State(_))
        ));
    }
    #[test]
    fn directory_serialization_preserves_same_list_row_numeric_identity() {
        let root = node("10", None, "selected");
        let mut row = info("11", "child", FileType::Dir);
        row.node_id = u64::MAX;
        let reply = directory_reply(&scope(), &root, Ok(vec![row]), |n| {
            Ok(node(&n.file_id, Some("10"), &n.name))
        })
        .unwrap();
        let decoded: InitializationBody =
            serde_json::from_slice(&serde_json::to_vec(&reply).unwrap()).unwrap();
        let InitializationBody::Directory {
            identity,
            children,
            denied,
        } = decoded
        else {
            panic!()
        };
        assert_eq!(identity, "10"); // still requested identity, NOT a server lookup proof.
        assert!(!denied);
        assert_eq!(children[0].native_node_id, u64::MAX);
        assert_eq!(children[0].node.id, "11");
        let mut replaced = info("12", "same-name", FileType::Dir);
        replaced.node_id = 45;
        assert!(
            directory_reply(&scope(), &root, Ok(vec![replaced]), |n| Ok(node(
                "11",
                Some("10"),
                &n.name
            )))
            .is_err()
        );
    }
    #[test]
    fn only_child_12290_is_denied_not_root_or_transport_failure() {
        let denied = || sproto::Error::Server {
            code: 12290,
            reason: "private server detail".into(),
        };
        let child = node("11", Some("10"), "child");
        assert!(
            matches!(directory_reply(&scope(),&child,Err(denied()),|_|panic!()),Ok(InitializationBody::Directory{denied:true,children,..}) if children.is_empty())
        );
        assert!(matches!(
            directory_reply(
                &scope(),
                &node("10", None, "selected"),
                Err(denied()),
                |_| panic!()
            ),
            Err(Error::RootUnavailable)
        ));
        assert!(matches!(
            directory_reply(
                &scope(),
                &child,
                Err(sproto::Error::ConnectionClosed),
                |_| panic!()
            ),
            Err(Error::Native(sproto::Error::ConnectionClosed))
        ));
    }
    #[test]
    fn pull_cursor_and_serialized_requests_are_closed() {
        for value in ["native-v1:0", "native-v1:20"] {
            validate_initialization_cursor(value).unwrap();
        }
        for value in [
            "",
            "20",
            "native-v1:01",
            "native-v1:+1",
            "native-v1:18446744073709551616",
        ] {
            assert!(validate_initialization_cursor(value).is_err());
        }
        assert!(
            serde_json::from_str::<InitializationRequest>(
                r#"{"Pull":{"cursor":"native-v1:1","extra":true}}"#
            )
            .is_err()
        );
        let reply = InitializationReply {
            scope: scope(),
            restore: "server".into(),
            body: InitializationBody::Events(None),
        };
        let round: InitializationReply =
            serde_json::from_slice(&serde_json::to_vec(&reply).unwrap()).unwrap();
        assert_eq!(reply, round);
    }
}
