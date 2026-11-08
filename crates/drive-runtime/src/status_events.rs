//! Issue #31: turn a committed status-journal page into the bounded change set
//! a desktop consumer can apply without re-reading every tracked path.
//!
//! A change set is expressed in mount-relative paths plus a kind:
//! - `status`: the node itself changed (state, pin, content identity);
//! - `listing`: a directory's children changed;
//! - `subtree`: many descendants changed (inherited pin/overlay/ACL), so a
//!   consumer may refresh every tracked path below it.
//!
//! Ancestors of a node change its aggregate status too, so they are emitted as
//! `status` paths with `directory = true`. If a page cannot be represented in
//! [`MAX_CHANGE_PATHS`] entries, the document says `reset = true` and consumers
//! fall back to their existing full refresh.
use drive_core::desktop_status::StatusPath;
use drive_core::status_changes::{ChangeScope, StatusChange};
use serde::Serialize;
use std::collections::{BTreeMap, HashMap};

/// Upper bound on paths carried by one change set. A larger delta resets.
pub const MAX_CHANGE_PATHS: usize = 256;
/// Page size used while draining the status journal.
pub const MAX_CHANGES_PER_READ: usize = 512;
/// Total journal rows a single signal may drain before it must reset.
pub const MAX_CHANGES_TOTAL: usize = 1024;

#[derive(Clone, Debug, PartialEq, Eq, Serialize)]
pub struct ChangePath {
    pub path: String,
    pub directory: bool,
    pub kind: &'static str,
}

fn rank(kind: &str) -> u8 {
    match kind {
        "status" => 0,
        "listing" => 1,
        "subtree" => 2,
        _ => 0,
    }
}

fn note(map: &mut BTreeMap<String, ChangePath>, path: &str, directory: bool, kind: &'static str) {
    use std::collections::btree_map::Entry;
    match map.entry(path.to_owned()) {
        Entry::Vacant(entry) => {
            entry.insert(ChangePath {
                path: path.to_owned(),
                directory,
                kind,
            });
        }
        Entry::Occupied(mut entry) => {
            let existing = entry.get_mut();
            existing.directory |= directory;
            if rank(kind) > rank(existing.kind) {
                existing.kind = kind;
            }
        }
    }
}

fn note_resolved(
    map: &mut BTreeMap<String, ChangePath>,
    paths: &HashMap<String, StatusPath>,
    id: &str,
    kind: &'static str,
) {
    let Some(resolved) = paths.get(id) else {
        return;
    };
    note(map, &resolved.path, resolved.directory, kind);
    // A node change also moves the aggregate status of every ancestor.
    for ancestor in &resolved.ancestors {
        note(map, ancestor, true, "status");
    }
}

fn note_listing(
    map: &mut BTreeMap<String, ChangePath>,
    paths: &HashMap<String, StatusPath>,
    parent: &str,
) {
    if let Some(resolved) = paths.get(parent) {
        note(map, &resolved.path, true, "listing");
    }
}

/// Build the bounded, deterministic change set for one journal delta.
pub fn change_paths(changes: &[StatusChange], paths: &HashMap<String, StatusPath>) -> Vec<ChangePath> {
    let mut map = BTreeMap::new();
    for change in changes {
        if let Some(id) = change.new_node.as_deref() {
            note_resolved(&mut map, paths, id, "status");
        }
        if let Some(id) = change.old_node.as_deref() {
            note_resolved(&mut map, paths, id, "status");
            // A delete/rename may have removed the row already; the journal
            // still records where it used to be, so name that path too.
            if !paths.contains_key(id) {
                if let (Some(parent), Some(name)) =
                    (change.old_parent.as_deref(), change.old_name.as_deref())
                {
                    if let Some(base) = paths.get(parent) {
                        let path = if base.path == "." {
                            name.to_owned()
                        } else {
                            format!("{}/{}", base.path, name)
                        };
                        note(&mut map, &path, false, "status");
                    }
                }
            }
        }
        if let Some(parent) = change.new_parent.as_deref() {
            note_listing(&mut map, paths, parent);
        }
        if let Some(parent) = change.old_parent.as_deref() {
            note_listing(&mut map, paths, parent);
        }
        if change.scope == ChangeScope::SubtreeAndAncestors {
            if let Some(id) = change.new_node.as_deref().or(change.old_node.as_deref()) {
                if let Some(resolved) = paths.get(id) {
                    note(&mut map, &resolved.path, true, "subtree");
                }
            }
        }
    }
    map.into_values().collect()
}

/// Serialize one `StatusChanged2` payload. Paths are emitted as absolute mount
/// paths (desktop consumers match them against tracked filesystem paths). The
/// returned flag is the final reset state: true when the caller already knew of
/// a gap or the change set overflowed.
pub fn change_document(
    from_revision: u64,
    to_revision: u64,
    epoch: &str,
    mount: &str,
    reset: bool,
    overview_changed: bool,
    mut changes: Vec<ChangePath>,
) -> (bool, String) {
    let mut reset = reset;
    let truncated = changes.len() > MAX_CHANGE_PATHS;
    if truncated {
        changes.truncate(MAX_CHANGE_PATHS);
        reset = true;
    }
    let mount = mount.trim_end_matches('/');
    for change in &mut changes {
        change.path = if change.path == "." {
            if mount.is_empty() {
                "/".to_owned()
            } else {
                mount.to_owned()
            }
        } else if mount.is_empty() {
            format!("/{}", change.path)
        } else {
            format!("{mount}/{}", change.path)
        };
    }
    let document = serde_json::json!({
        "schema": 1,
        "epoch": epoch,
        "from_revision": from_revision,
        "to_revision": to_revision,
        "reset": reset,
        "truncated": truncated,
        "overview_changed": overview_changed,
        "changes": changes,
    });
    (reset, document.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn change(sequence: u64, scope: ChangeScope, old: Option<&str>, new: Option<&str>) -> StatusChange {
        StatusChange {
            sequence,
            source: "nodes".into(),
            operation: "update".into(),
            scope,
            old_node: old.map(str::to_owned),
            new_node: new.map(str::to_owned),
            old_parent: old.map(|_| "dir".to_owned()),
            new_parent: new.map(|_| "dir".to_owned()),
            old_name: old.map(|_| "file".to_owned()),
            new_name: new.map(|_| "file".to_owned()),
            bytes: 0,
        }
    }

    fn paths() -> HashMap<String, StatusPath> {
        let entries = [
            ("root", ".", true, vec![]),
            ("dir", "dir", true, vec!["."]),
            ("file", "dir/file", false, vec!["dir", "."]),
            ("deep", "dir/sub/deep", false, vec!["dir/sub", "dir", "."]),
            ("sub", "dir/sub", true, vec!["dir", "."]),
        ];
        entries
            .into_iter()
            .map(|(id, path, directory, ancestors)| {
                (
                    id.to_owned(),
                    StatusPath {
                        path: path.to_owned(),
                        directory,
                        ancestors: ancestors.into_iter().map(str::to_owned).collect(),
                    },
                )
            })
            .collect()
    }

    #[test]
    fn node_changes_include_ancestors_and_listing_parents() {
        let changes = vec![change(1, ChangeScope::NodeAndAncestors, Some("file"), Some("file"))];
        let set = change_paths(&changes, &paths());
        let by_kind = |kind: &str| {
            set.iter()
                .filter(|entry| entry.kind == kind)
                .map(|entry| entry.path.as_str())
                .collect::<Vec<_>>()
        };
        // The node itself plus the root ancestor that aggregates its status;
        // `dir` is also its parent, so the stronger `listing` kind wins there.
        assert_eq!(by_kind("status"), vec![".", "dir/file"]);
        assert_eq!(by_kind("listing"), vec!["dir"]);
        assert!(by_kind("subtree").is_empty());
        assert!(
            !set.iter().any(|entry| entry.path == "dir/sub/deep"),
            "siblings must not enter the change set"
        );
    }

    #[test]
    fn subtree_scope_wins_over_other_kinds_for_the_same_path() {
        let changes = vec![change(1, ChangeScope::SubtreeAndAncestors, None, Some("sub"))];
        let set = change_paths(&changes, &paths());
        let sub = set.iter().find(|entry| entry.path == "dir/sub").unwrap();
        assert_eq!(sub.kind, "subtree");
        assert!(sub.directory);
        assert!(set.iter().all(|entry| entry.path != "dir/sub/deep"));
    }

    #[test]
    fn a_removed_row_keeps_its_recorded_old_path() {
        // The node row is already gone from the store, so only the journal's
        // recorded parent/name can name the vanished path.
        let mut change = change(1, ChangeScope::NodeAndAncestors, Some("gone"), None);
        change.old_name = Some("removed".into());
        let set = change_paths(&[change], &paths());
        assert!(set.iter().any(|entry| entry.path == "dir/removed" && entry.kind == "status"));
        assert!(set.iter().any(|entry| entry.path == "dir" && entry.kind == "listing"));
    }

    #[test]
    fn too_many_paths_forces_a_reset_and_truncates() {
        let changes: Vec<_> = (0..300)
            .map(|index| ChangePath {
                path: format!("dir/f{index:03}"),
                directory: false,
                kind: "status",
            })
            .collect();
        let (reset, document) =
            change_document(1, 2, "epoch", "/mnt/drive", false, true, changes);
        assert!(reset, "an over-budget change set must reset consumers");
        let value: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(value["schema"], 1);
        assert_eq!(value["reset"], true);
        assert_eq!(value["truncated"], true);
        assert_eq!(value["overview_changed"], true);
        assert_eq!(value["changes"].as_array().unwrap().len(), MAX_CHANGE_PATHS);
    }

    #[test]
    fn documents_carry_epoch_and_absolute_mount_paths() {
        let changes = change_paths(
            &[change(1, ChangeScope::NodeAndAncestors, Some("file"), Some("file"))],
            &paths(),
        );
        let (reset, document) =
            change_document(7, 9, "writer-epoch", "/mnt/drive/", false, false, changes);
        assert!(!reset);
        let value: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(value["schema"], 1);
        assert_eq!(value["epoch"], "writer-epoch");
        assert_eq!(value["from_revision"], 7);
        assert_eq!(value["to_revision"], 9);
        assert_eq!(value["reset"], false);
        assert_eq!(value["truncated"], false);
        assert_eq!(value["overview_changed"], false);
        let paths: Vec<&str> = value["changes"]
            .as_array()
            .unwrap()
            .iter()
            .map(|entry| entry["path"].as_str().unwrap())
            .collect();
        assert!(paths.contains(&"/mnt/drive"));
        assert!(paths.contains(&"/mnt/drive/dir/file"));
        assert!(
            paths.iter().all(|path| path.starts_with("/mnt/drive")),
            "all change paths must be absolute mount paths: {paths:?}"
        );
    }

    #[test]
    fn server_side_reset_keeps_the_flag_and_empty_changes() {
        let (reset, document) =
            change_document(7, 9, "writer-epoch", "/mnt/drive", true, false, Vec::new());
        assert!(reset);
        let value: serde_json::Value = serde_json::from_str(&document).unwrap();
        assert_eq!(value["reset"], true);
        assert_eq!(value["truncated"], false);
        assert!(value["changes"].as_array().unwrap().is_empty());
    }
}
