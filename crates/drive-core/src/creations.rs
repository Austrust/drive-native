//! Stable local identities for offline creation and native identity binding.
use crate::{Change, Error, Node, NodeMetadata, Result, Store};
use rusqlite::{Connection, OptionalExtension, params};
pub const CREATE_VERSION: &str = "local-create-v1";
pub fn is_local_identity(id: &str) -> bool {
    id.strip_prefix("local:")
        .is_some_and(|s| uuid::Uuid::parse_str(s).is_ok())
}
pub(crate) fn local_identity(db: &Connection, remote: &str) -> Result<String> {
    Ok(db
        .query_row(
            "SELECT node FROM remote_aliases WHERE remote=?1",
            [remote],
            |r| r.get(0),
        )
        .optional()?
        .unwrap_or_else(|| remote.to_owned()))
}
pub(crate) fn map_change(db: &Connection, c: &Change) -> Result<Change> {
    Ok(match c {
        Change::Upsert(n) => {
            let mut n = n.clone();
            n.id = local_identity(db, &n.id)?;
            n.parent = n
                .parent
                .as_deref()
                .map(|p| local_identity(db, p))
                .transpose()?;
            Change::Upsert(n)
        }
        Change::Delete { id } => Change::Delete {
            id: local_identity(db, id)?,
        },
    })
}
impl Store {
    pub fn local_identity(&self, remote: &str) -> Result<String> {
        local_identity(&self.db, remote)
    }
    pub fn create_local_file(&mut self, parent: &str, name: &str, mode: u32) -> Result<Node> {
        if !self.node(parent)?.directory {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::ENOTDIR)));
        }
        let id = format!("local:{}", uuid::Uuid::new_v4());
        let n = Node {
            id: id.clone(),
            parent: Some(parent.into()),
            name: name.into(),
            version: CREATE_VERSION.into(),
            directory: false,
            metadata: Some(NodeMetadata {
                size: 0,
                mtime: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap_or_default()
                    .as_secs(),
                mode: mode & 0o777,
            }),
        };
        crate::state::valid_node(&n)?;
        let exists: bool = self.db.query_row(
            "SELECT EXISTS(SELECT 1 FROM nodes WHERE parent=?1 AND name=?2 AND deleted=0)",
            params![parent, name],
            |r| r.get(0),
        )?;
        if exists {
            return Err(Error::Io(std::io::Error::from_raw_os_error(libc::EEXIST)));
        }
        let blob = self.write_blob(&[])?;
        let op = uuid::Uuid::new_v4().to_string();
        let tx = self.db.transaction()?;
        tx.execute("INSERT INTO nodes(id,parent,name,remote_version,directory,local_version,blob,dirty_blob,base_version,dirty_generation) VALUES(?1,?2,?3,?4,0,?4,?5,?5,?4,1)",params![id,parent,name,CREATE_VERSION,blob])?;
        crate::filesystem::write_metadata(&tx, &n)?;
        tx.execute(
            "INSERT INTO local_creations(node,ready) VALUES(?1,0)",
            [&id],
        )?;
        tx.execute("INSERT INTO operations(id,node,kind,base,blob,generation) VALUES(?1,?2,'upload',?3,?4,1)",params![op,id,CREATE_VERSION,blob])?;
        tx.commit()?;
        Ok(n)
    }
    pub fn creation_saved(&mut self, id: &str) -> Result<()> {
        self.db
            .execute("UPDATE local_creations SET ready=1 WHERE node=?1", [id])?;
        Ok(())
    }
    pub(crate) fn mapped_snapshot(&self, nodes: &[Node]) -> Result<Vec<Node>> {
        nodes
            .iter()
            .map(
                |n| match map_change(&self.db, &Change::Upsert(n.clone()))? {
                    Change::Upsert(n) => Ok(n),
                    _ => unreachable!(),
                },
            )
            .collect()
    }
}
/// Called inside the same transaction as version/blob confirmation.
pub(crate) fn bind_created_identity(
    tx: &rusqlite::Transaction<'_>,
    id: &str,
    remote: &str,
    version: &str,
) -> Result<()> {
    if !is_local_identity(id) || remote.is_empty() || !remote.bytes().all(|c| c.is_ascii_digit()) {
        return Err(Error::State("invalid new native identity".into()));
    }
    let created: bool = tx.query_row(
        "SELECT EXISTS(SELECT 1 FROM local_creations WHERE node=?1)",
        [id],
        |r| r.get(0),
    )?;
    if !created {
        return Err(Error::State("creation intent absent".into()));
    }
    let alias: Option<String> = tx
        .query_row(
            "SELECT node FROM remote_aliases WHERE remote=?1",
            [remote],
            |r| r.get(0),
        )
        .optional()?;
    if alias.as_deref().is_some_and(|n| n != id) {
        return Err(Error::State(
            "native identity already belongs to another file".into(),
        ));
    }
    let echo: Option<(String, bool)> = tx
        .query_row(
            "SELECT remote_version,dirty_blob IS NOT NULL FROM nodes WHERE id=?1",
            [remote],
            |r| Ok((r.get(0)?, r.get(1)?)),
        )
        .optional()?;
    if let Some((echo_version, dirty)) = echo {
        if echo_version != version || dirty {
            return Err(Error::State(
                "remote changed during creation identity binding".into(),
            ));
        }
    }
    tx.execute(
        "INSERT INTO remote_aliases(remote,node) VALUES(?1,?2)",
        params![remote, id],
    )?;
    // An event may precede its upload receipt. Its read-only mirror is folded into
    // the persistent local identity; the existing inode and open handles survive.
    for table in ["nodes", "node_metadata", "fs_inodes"] {
        let column = if table == "nodes" { "id" } else { "node" };
        tx.execute(&format!("DELETE FROM {table} WHERE {column}=?1"), [remote])?;
    }
    tx.execute("DELETE FROM local_creations WHERE node=?1", [id])?;
    Ok(())
}
