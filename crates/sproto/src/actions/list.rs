use std::collections::BTreeMap;

use crate::{
    channel::Channel,
    client::Client,
    error::{Error, Result},
    frame,
    pstream::PObject,
};

/// Metadata for a shared folder (team folder / view).
#[derive(Debug, Clone)]
pub struct ShareInfo {
    /// Present only when the server supplies the root node identity.
    pub node_id: Option<u64>,
    /// Unknown privilege is never promoted to writable.
    pub writable: bool,
    /// Server-assigned numeric identifier for this view.
    pub view_id: u64,

    /// Display name of the shared folder.
    pub name: String,

    /// Unique file identifier on the server.
    pub file_id: String,

    /// Whether the current user can download files from this share.
    pub can_download: bool,

    /// Whether the current user can sync this share to a device.
    pub can_sync: bool,
}

/// Metadata for a file or directory within a share.
#[derive(Debug, Clone)]
pub struct NodeInfo {
    /// Server-assigned numeric node identifier.
    pub node_id: u64,

    /// Sync identifier used for change tracking.
    pub sync_id: u64,

    /// Content revision, distinct from the event cursor.
    pub max_id: u64,

    /// File or directory name (leaf name, not full path).
    pub name: String,

    /// Size in bytes (0 for directories).
    pub file_size: u64,

    /// Last-modified time as a Unix timestamp.
    pub mtime: u64,

    /// Content hash for integrity verification.
    pub hash: String,

    /// Unique file identifier on the server.
    pub file_id: String,

    /// Whether this node is a file, directory, or symlink.
    pub file_type: FileType,

    /// Whether this node has been deleted (tombstone).
    pub is_removed: bool,
}

/// The kind of filesystem entry a node represents.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FileType {
    /// Regular file.
    File,
    /// Directory.
    Dir,
    /// Symbolic link.
    Symlink,
}

/// An unchanged response must never be interpreted as an empty directory.
#[derive(Debug)]
pub enum DirectoryListing {
    Unchanged {
        cursor: String,
    },
    Snapshot {
        nodes: Vec<NodeInfo>,
        cursor: String,
    },
}

/// List all available shares/views.
pub(crate) async fn list_team_folder(ch: &mut Channel, client: &Client) -> Result<Vec<ShareInfo>> {
    // Official CloudStation::ListTeamFolder omits zero-valued offset/limit.
    // On Drive 4.0.3, an explicit limit=0 returns no entries, not all entries.
    let request = client.build_request("list_team_folder", team_catalog_fields());
    let response = ch.request(frame::SCMD_REQUEST, &request).await?;
    // Team capabilities/file IDs and sync root node IDs are separate contracts.
    // Join by view identity, never by display name or a guessed node_id=1.
    let request = client.build_request("query_user_info", pmap! {"get_view" => true});
    let user = ch.request(frame::SCMD_REQUEST, &request).await?;
    parse_team_catalog(&response, &user)
}

fn team_catalog_fields() -> PObject {
    pmap! {"sort_by" => "name", "sort_direction" => "asc"}
}

fn parse_team_catalog(response: &PObject, user: &PObject) -> Result<Vec<ShareInfo>> {
    let teams = response.get("view_list").and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing team view_list".into()))?;
    let views = user.get("view_list").and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing sync view_list".into()))?;
    let mut identities = BTreeMap::new();
    for view in views {
        let map = view.as_map().ok_or_else(|| Error::Decode("invalid sync view".into()))?;
        let id = required_int(map, "view_id")?;
        if identities.insert(id, map).is_some() {
            return Err(Error::Decode("duplicate sync view identity".into()));
        }
    }
    let mut shares = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for team in teams {
        let mut share = parse_share_info(team)?;
        if !seen.insert(share.view_id) {
            return Err(Error::Decode("duplicate team view identity".into()));
        }
        if let Some(view) = identities.get(&share.view_id) {
            let privilege = view.get("privilege").and_then(PObject::as_str);
            share.node_id = view.get("node_id").and_then(PObject::as_int);
            share.writable &= privilege == Some("read-write");
            share.can_sync &= matches!(privilege, Some("read-write" | "read-only"));
            share.can_download &= view.get("disable_download").and_then(PObject::as_int) == Some(0);
        } else {
            // Keep the visible entry, but do not offer an unverified sync root.
            share.node_id = None;
            share.writable = false;
            share.can_sync = false;
        }
        shares.push(share);
    }
    Ok(shares)
}


/// List directory contents using `list_sync_to_device`.
/// Returns all children (no pagination) + a cursor for change detection.
/// Only works with team folder view IDs from `list_team_folder`.
///
/// Pass a `cursor` from a previous call to enable the server's fast path:
/// if nothing changed, the server returns the same cursor with no `node_list`,
/// avoiding a full directory transfer.
pub(crate) async fn list_sync_to_device(
    ch: &mut Channel,
    client: &Client,
    view_id: u64,
    path: &str,
    cursor: Option<&str>,
) -> Result<DirectoryListing> {
    let mut fields = pmap! {
        "path" => path,
        "view_id" => view_id,
        "merge_local" => 0u64,
        "list_dir_only" => 0u64,
        "include_node_locking" => 0u64,
    };
    if let Some(c) = cursor {
        fields
            .as_map_mut()
            .expect("pmap! always produces a Map")
            .insert("cursor".to_string(), PObject::from(c));
    }
    let request = client.build_request("list_sync_to_device", fields);
    let resp = ch.request(frame::SCMD_LIST, &request).await?;

    let new_cursor = resp
        .get("cursor")
        .and_then(|v| v.as_str())
        .ok_or_else(|| Error::Decode("missing directory cursor".into()))?
        .to_string();

    let mut nodes = Vec::new();
    if let Some(node_list) = resp.get("node_list").and_then(|v| v.as_array()) {
        for node in node_list {
            nodes.push(parse_node_info(node)?);
        }
    } else if resp.get("node_list").is_none() && cursor == Some(new_cursor.as_str()) {
        return Ok(DirectoryListing::Unchanged { cursor: new_cursor });
    } else {
        return Err(Error::Decode(
            "missing node_list for a changed directory".into(),
        ));
    }
    Ok(DirectoryListing::Snapshot {
        nodes,
        cursor: new_cursor,
    })
}

/// List directory contents using `list` action (paginated).
pub(crate) async fn list(
    ch: &mut Channel,
    client: &Client,
    view_id: u64,
    path: &str,
) -> Result<Vec<NodeInfo>> {
    let mut all_nodes = Vec::new();
    let mut offset = 0u64;
    // Official sync enumeration excludes merged filesystem-only entries. On
    // 4.0.3 merge_local=1 returns the full directory with zero identities outside
    // the requested page. Never treat those synthetic entries as sync objects.
    let mut expected_total = None;
    let mut identities = std::collections::HashSet::new();
    let mut names = std::collections::HashSet::new();
    let limit = 1000u64;

    loop {
        let request = client.build_request(
            "list",
            pmap! {
                "path" => path,
                "view_id" => view_id,
                "merge_local" => 0u64,
                "list_dir_only" => 0u64,
                "search_criteria" => pmap! {
                    "limit" => limit,
                    "offset" => offset,
                    "sort_by" => "name",
                    "sort_direction" => "asc",
                },
            },
        );
        let resp = ch.request(frame::SCMD_LIST, &request).await?;

        let node_list = resp
            .get("node_list")
            .and_then(|v| v.as_array())
            .ok_or_else(|| Error::Decode("missing node_list".into()))?;

        let total_count = resp
            .get("total_count")
            .and_then(PObject::as_int)
            .ok_or_else(|| Error::Decode("missing total_count".into()))?;
        if expected_total.is_some_and(|previous| previous != total_count)
            || node_list.len() as u64 > limit
            || offset + node_list.len() as u64 > total_count
        {
            return Err(Error::Decode(
                "directory changed or violated page bounds".into(),
            ));
        }
        expected_total = Some(total_count);
        if total_count > 1_000_000 || all_nodes.len() + node_list.len() > 1_000_000 {
            return Err(Error::Decode(
                "directory exceeds configured entry limit".into(),
            ));
        }
        if node_list.is_empty() && offset < total_count {
            return Err(Error::Decode(
                "directory pagination made no progress".into(),
            ));
        }

        for node in node_list {
            let node = parse_node_info(node)?;
            if !identities.insert(node.file_id.clone()) || !names.insert(node.name.clone()) {
                return Err(Error::Decode(
                    "directory repeated an identity or name across pages".into(),
                ));
            }
            all_nodes.push(node);
        }

        #[cfg(feature = "protocol-research")]
        super::list_observer::at_page(super::list_observer::PageBoundary {
            path: path.into(),
            offset,
            count: node_list.len() as u64,
            total: total_count,
            first_name: all_nodes
                .get(offset as usize)
                .map(|n| n.name.clone())
                .unwrap_or_default(),
            last_name: all_nodes.last().map(|n| n.name.clone()).unwrap_or_default(),
        })
        .await?;
        offset += node_list.len() as u64;
        if offset >= total_count || node_list.is_empty() {
            break;
        }
    }

    Ok(all_nodes)
}

fn get_int(map: &BTreeMap<String, PObject>, key: &str) -> u64 {
    map.get(key).and_then(PObject::as_int).unwrap_or(0)
}

fn parse_share_info(obj: &PObject) -> Result<ShareInfo> {
    let map = obj
        .as_map()
        .ok_or_else(|| Error::Decode("view is not a map".into()))?;

    let caps = |key: &str| -> bool {
        map.get("capabilities")
            .and_then(|c| c.get(key))
            .and_then(PObject::as_int)
            .is_some_and(|v| v != 0)
    };

    Ok(ShareInfo {
        node_id: map.get("node_id").and_then(PObject::as_int),
        writable: caps("can_write"),
        name: required_str(map, "name")?,
        can_sync: caps("can_sync"),
        view_id: required_int(map, "view_id")?,
        file_id: required_str(map, "file_id")?,
        can_download: caps("can_download") && get_int(map, "disable_download") == 0,
    })
}

pub(crate) fn parse_node_info(obj: &PObject) -> Result<NodeInfo> {
    let map = obj
        .as_map()
        .ok_or_else(|| Error::Decode("node is not a map".into()))?;

    let name = required_str(map, "name")?;
    if name.is_empty() || name == "." || name == ".." || name.contains('/') || name.contains('\0') {
        return Err(Error::Decode("invalid remote leaf name".into()));
    }
    if required_str(map, "file_id")?.is_empty()
        || required_str(map, "file_id")? == "0"
        || required_int(map, "sync_id")? == 0
        || required_int(map, "max_id")? == 0
    {
        return Err(Error::Decode(
            "directory contains an unversioned synthetic entry".into(),
        ));
    }
    Ok(NodeInfo {
        name,
        hash: required_str(map, "file_hash")?,
        mtime: required_int(map, "mtime")?,
        node_id: required_int(map, "node_id")?,
        sync_id: required_int(map, "sync_id")?,
        max_id: required_int(map, "max_id")?,
        file_id: required_str(map, "file_id")?,
        file_size: required_int(map, "file_size")?,
        is_removed: get_int(map, "is_removed") != 0,
        file_type: match map.get("file_type").and_then(|v| v.as_str()) {
            Some("dir") => FileType::Dir,
            Some("symlink") => FileType::Symlink,
            Some("file") => FileType::File,
            _ => return Err(Error::Decode("missing or unsupported file_type".into())),
        },
    })
}

fn required_int(map: &BTreeMap<String, PObject>, key: &str) -> Result<u64> {
    map.get(key)
        .and_then(PObject::as_int)
        .ok_or_else(|| Error::Decode(format!("missing integer field: {key}")))
}
fn required_str(map: &BTreeMap<String, PObject>, key: &str) -> Result<String> {
    map.get(key)
        .and_then(PObject::as_str)
        .map(str::to_string)
        .ok_or_else(|| Error::Decode(format!("missing string field: {key}")))
}

#[cfg(test)]
mod strict_metadata_tests {
    use super::*;
    fn node() -> PObject {
        pmap! {"name"=>"A.doc","file_hash"=>"hash","mtime"=>1u64,"node_id"=>2u64,"sync_id"=>3u64,"max_id"=>3u64,
        "file_id"=>"opaque","file_size"=>4u64,"file_type"=>"file"}
    }
    #[test]
    fn missing_identity_is_not_defaulted() {
        let mut p = node();
        p.as_map_mut().unwrap().remove("file_id");
        assert!(parse_node_info(&p).is_err());
    }
    #[test]
    fn malformed_remote_path_is_rejected() {
        let mut p = node();
        p.as_map_mut()
            .unwrap()
            .insert("name".into(), PObject::from("../escape"));
        assert!(parse_node_info(&p).is_err());
    }
    #[test]
    fn unknown_file_type_is_not_silently_a_regular_file() {
        let mut p = node();
        p.as_map_mut()
            .unwrap()
            .insert("file_type".into(), PObject::from("unknown"));
        assert!(parse_node_info(&p).is_err());
    }
}

#[cfg(test)]
mod recorded_snapshot_pages {
    use super::*;
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
    fn replay_real_page_accepts_verified_empty_files_and_rejects_merged_zero_identity() {
        let samples: serde_json::Value = serde_json::from_str(include_str!(
            "../../../../evidence/snapshot-page-samples.json"
        ))
        .unwrap();
        for node in samples["indexed"].as_array().unwrap() {
            let n = parse_node_info(&object(node.clone())).unwrap();
            assert_eq!(n.file_size, 0);
            assert_eq!(n.hash, "31d6cfe0d16ae931b73c59d7e0c089c0");
            assert_ne!(n.file_id, "0");
        }
        assert!(parse_node_info(&object(samples["synthetic"].clone())).is_err());
    }
}

#[cfg(test)]
mod team_catalog_tests {
    use super::*;
    fn team(id:u64, write:bool) -> PObject {
        pmap! {"view_id"=>id,"file_id"=>format!("file-{id}"),"name"=>"team",
            "disable_download"=>0u64,"capabilities"=>pmap!{
                "can_write"=>write,"can_sync"=>true,"can_download"=>true}}
    }
    fn view(id:u64, node:u64, privilege:&str) -> PObject {
        pmap! {"view_id"=>id,"node_id"=>node,"privilege"=>privilege,"disable_download"=>0u64}
    }
    fn list(items:Vec<PObject>)->PObject {pmap!{"view_list"=>PObject::Array(items)}}
    #[test]
    fn unlimited_catalog_omits_zero_limit_like_official_linux() {
        let fields=team_catalog_fields();
        assert!(fields.get("limit").is_none());
        assert!(fields.get("offset").is_none());
        assert_eq!(fields.get("sort_by").and_then(PObject::as_str),Some("name"));
    }
    #[test]
    fn separate_native_responses_join_by_view_not_order_or_name() {
        let teams=list(vec![team(14,false),team(12,true)]);
        let views=list(vec![view(12,71,"read-write"),view(999,9,"denied"),view(14,82,"read-write")]);
        let result=parse_team_catalog(&teams,&views).unwrap();
        assert_eq!(result.len(),2);
        assert_eq!(result[0].node_id,Some(82));assert!(!result[0].writable);
        assert_eq!(result[1].node_id,Some(71));assert!(result[1].writable);
        assert!(result.iter().all(|s|s.can_download&&s.can_sync));
    }
    #[test]
    fn missing_denied_and_readonly_views_never_gain_write_access() {
        let result=parse_team_catalog(&list(vec![team(1,true),team(2,true),team(3,true)]),
            &list(vec![view(2,7,"denied"),view(3,8,"read-only")])).unwrap();
        assert!(result.iter().all(|s|!s.writable));
        assert!(!result[0].can_sync);assert!(!result[1].can_sync);assert!(result[2].can_sync);
        assert_eq!(result[0].node_id,None);
    }
    #[test]
    fn ambiguous_view_identity_is_rejected() {
        assert!(parse_team_catalog(&list(vec![team(1,true)]),&list(vec![view(1,2,"read-write"),view(1,3,"read-write")])).is_err());
        assert!(parse_team_catalog(&list(vec![team(1,true),team(1,true)]),&list(vec![view(1,2,"read-write")])).is_err());
    }
}
