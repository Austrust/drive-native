//! Durable initialization-only location hints, decoded beside the normalized
//! change. Hints are observations, not independently verified ancestry or ACL.
use crate::{Batch, Change, Error, Result};
use serde::{Deserialize, Serialize};

/// Numeric identity returned in the SAME native list row as `node`.
/// The surrounding directory reply still echoes its requested identity; this
/// entry does not turn a path list into an atomic identity lookup.
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationDirectoryEntry {
    pub node: crate::Node,
    pub native_node_id: u64,
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
pub enum InitializationEventKind {
    Modify,
    Remove,
    Move,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationEventHint {
    pub ordinal: u64,
    pub event_id: String,
    pub event_sync_id: u64,
    pub file_id: String,
    pub kind: InitializationEventKind,
    pub event_path: String,
    pub destination_path: Option<String>,
    /// Current metadata observation; NEVER an asserted historical old parent.
    pub metadata_parent_id: Option<String>,
    pub destination_parent_id: Option<String>,
    pub metadata_removed: bool,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct InitializationEventPage {
    pub batch: Batch,
    pub scope: serde_json::Value,
    pub restore: String,
    pub hints: Vec<InitializationEventHint>,
}

impl InitializationEventPage {
    /// Structural correspondence only. The receiver must separately bind the
    /// header to its authenticated session and durable run/epoch, then validate
    /// the existing Batch/Node contracts. No path here authorizes a network IO.
    pub fn validate(&self) -> Result<()> {
        validate_event_page_parts(&self.batch, &self.scope, &self.restore, &self.hints)
    }
}

/// Validate a borrowed decoded receipt without copying or decoding its Batch again.
pub(crate) fn validate_event_page_parts(
    batch: &Batch,
    scope: &serde_json::Value,
    restore: &str,
    hints: &[InitializationEventHint],
) -> Result<()> {
    let invalid = || Error::State("initialization event hint mismatch".into());
    let path = |s: &str| {
        s.starts_with('/')
            && !s.contains('\0')
            && (s == "/" || s[1..].split('/').all(|p| !matches!(p, "" | "." | "..")))
    };
    if !scope.is_object() || restore.is_empty() || hints.len() != batch.events.len() {
        return Err(invalid());
    }
    // Match the native parser's event sync_id <= next_sync_id check. This is
    // not current file.max_id, and historical events may precede `from` or be 0.
    let through = batch
        .to
        .strip_prefix("native-v1:")
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse::<u64>().ok())
        .ok_or_else(invalid)?;
    let scope_path = scope
        .get("path")
        .and_then(serde_json::Value::as_str)
        .filter(|s| path(s))
        .ok_or_else(invalid)?;
    let root_id = scope
        .get("file_id")
        .and_then(serde_json::Value::as_str)
        .filter(|s| !s.is_empty())
        .ok_or_else(invalid)?;
    let contains = |s: &str| {
        scope_path == "/"
            || s == scope_path
            || s.strip_prefix(scope_path)
                .is_some_and(|rest| rest.starts_with('/'))
    };
    for (ordinal, (hint, event)) in hints.iter().zip(&batch.events).enumerate() {
        let id = match &event.change {
            Change::Upsert(node) => &node.id,
            Change::Delete { id } => id,
        };
        if hint.ordinal != ordinal as u64
            || hint.event_sync_id > through
            || hint.event_id != event.id
            || hint.event_id.is_empty()
            || hint.file_id.is_empty()
            || &hint.file_id != id
            || !path(&hint.event_path)
            || hint
                .metadata_parent_id
                .as_ref()
                .is_some_and(|p| p.is_empty())
            || hint
                .destination_parent_id
                .as_ref()
                .is_some_and(|p| p.is_empty())
            || (hint.metadata_removed && hint.metadata_parent_id.is_some())
        {
            return Err(invalid());
        }
        match hint.kind {
            InitializationEventKind::Move => {
                if !hint.destination_path.as_deref().is_some_and(path)
                    || hint.destination_parent_id.is_none()
                {
                    return Err(invalid());
                }
            }
            _ if hint.destination_path.is_some() || hint.destination_parent_id.is_some() => {
                return Err(invalid());
            }
            _ => {}
        }
        let destination_in_scope = hint.destination_path.as_deref().is_some_and(contains);
        if !contains(&hint.event_path) && !destination_in_scope {
            return Err(invalid());
        }
        if (hint.metadata_removed || hint.kind == InitializationEventKind::Remove)
            && !matches!(event.change, Change::Delete { .. })
        {
            return Err(invalid());
        }
        if hint.kind == InitializationEventKind::Remove && !hint.metadata_removed {
            return Err(invalid());
        }
        if let Change::Upsert(node) = &event.change {
            let (location, parent) = if hint.kind == InitializationEventKind::Move {
                (
                    hint.destination_path.as_deref().unwrap(),
                    &hint.destination_parent_id,
                )
            } else {
                (hint.event_path.as_str(), &hint.metadata_parent_id)
            };
            let name = if location == "/" {
                "My Drive"
            } else {
                location.rsplit('/').next().unwrap()
            };
            let root = node.id == root_id;
            if !contains(location)
                || root != (location == scope_path)
                || root != node.parent.is_none()
                || node.name != name
                || (node.parent.is_some() && &node.parent != parent)
            {
                return Err(invalid());
            }
        }
        if matches!(event.change, Change::Delete { .. })
            && !hint.metadata_removed
            && !(hint.kind == InitializationEventKind::Move && !destination_in_scope)
        {
            return Err(invalid());
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn page() -> InitializationEventPage {
        InitializationEventPage {
            batch: Batch {
                from: "native-v1:7".into(),
                to: "native-v1:8".into(),
                events: vec![crate::Event {
                    id: "synthetic-event".into(),
                    change: Change::Upsert(crate::Node {
                        id: "file".into(),
                        parent: Some("parent".into()),
                        name: "A".into(),
                        version: "opaque".into(),
                        directory: false,
                        metadata: None,
                    }),
                }],
            },
            scope: serde_json::json!({"path":"/Lab","file_id":"root"}),
            restore: "server".into(),
            hints: vec![InitializationEventHint {
                ordinal: 0,
                event_id: "synthetic-event".into(),
                event_sync_id: 8,
                file_id: "file".into(),
                kind: InitializationEventKind::Modify,
                event_path: "/Lab/sub/A".into(),
                destination_path: None,
                metadata_parent_id: Some("parent".into()),
                destination_parent_id: None,
                metadata_removed: false,
            }],
        }
    }

    #[test]
    fn pure_page_roundtrip_rejects_unknown_envelope_or_hint_fields() {
        let p = page();
        p.validate().unwrap();
        let mut value = serde_json::to_value(&p).unwrap();
        assert_eq!(
            serde_json::from_value::<InitializationEventPage>(value.clone()).unwrap(),
            p
        );
        value["old_parent"] = "invented".into();
        assert!(serde_json::from_value::<InitializationEventPage>(value).is_err());
        let mut value = serde_json::to_value(p).unwrap();
        value["hints"][0]["old_parent"] = "invented".into();
        assert!(serde_json::from_value::<InitializationEventPage>(value).is_err());
    }

    #[test]
    fn page_correspondence_paths_and_selected_change_are_not_advisory() {
        let mutations: Vec<Box<dyn Fn(&mut InitializationEventPage)>> = vec![
            Box::new(|p| p.hints.clear()),
            Box::new(|p| p.hints[0].ordinal = 1),
            Box::new(|p| p.hints[0].event_id = "another".into()),
            Box::new(|p| p.hints[0].file_id = "another".into()),
            Box::new(|p| p.hints[0].event_path = "/Lab/../outside/A".into()),
            Box::new(|p| p.hints[0].event_path = "/Lab-other/A".into()),
            Box::new(|p| p.hints[0].event_path = "/Lab/sub/B".into()),
            Box::new(|p| p.hints[0].metadata_parent_id = Some("another".into())),
            Box::new(|p| p.hints[0].destination_path = Some("/Lab/B".into())),
            Box::new(|p| p.scope["path"] = "/Lab//bad".into()),
            Box::new(|p| p.hints[0].metadata_removed = true),
        ];
        for mutate in mutations {
            let mut p = page();
            mutate(&mut p);
            assert!(p.validate().is_err());
        }
    }

    #[test]
    fn outgoing_move_retains_source_and_destination_while_normalized_delete_is_valid() {
        let mut p = page();
        p.hints[0].kind = InitializationEventKind::Move;
        p.hints[0].destination_path = Some("/Elsewhere/A".into());
        p.hints[0].destination_parent_id = Some("elsewhere-parent".into());
        p.batch.events[0].change = Change::Delete { id: "file".into() };
        p.validate().unwrap();
        p.hints[0].event_path = "/Other/A".into();
        assert!(p.validate().is_err());
    }

    #[test]
    fn event_sync_upper_bound_matches_parser_without_a_lower_bound() {
        let mut p = page();
        for sync in [0, 1, 7, 8] {
            p.hints[0].event_sync_id = sync;
            p.validate().unwrap();
        }
        p.hints[0].event_sync_id = 9;
        assert!(p.validate().is_err());
        p.hints[0].event_sync_id = 0;
        for cursor in [
            "",
            "other:8",
            "native-v1:",
            "native-v1:-1",
            "native-v1:+8",
            "native-v1:18446744073709551616",
        ] {
            p.batch.to = cursor.into();
            assert!(p.validate().is_err());
        }
    }

    #[test]
    fn directory_entry_keeps_u64_identity_and_rejects_wrong_json_type() {
        let Change::Upsert(node) = page().batch.events.remove(0).change else {
            panic!()
        };
        let entry = InitializationDirectoryEntry {
            node,
            native_node_id: u64::MAX,
        };
        let mut value = serde_json::to_value(&entry).unwrap();
        assert_eq!(
            serde_json::from_value::<InitializationDirectoryEntry>(value.clone()).unwrap(),
            entry
        );
        value["native_node_id"] = "123".into();
        assert!(serde_json::from_value::<InitializationDirectoryEntry>(value).is_err());
    }
}
