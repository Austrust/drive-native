//! Narrow local scheduling for known directories and page-local unknown-child
//! dependencies under verified existing parents. Native identities are prior
//! list observations, not proof of the current path. Re-list affected parents
//! and subtrees; never copy event metadata into entries.
//!
//! Unknown Modify directories may form a page-local unique-ID/name DAG. Only
//! edges connected to an existing, non-denied native anchor are admitted; the
//! first edge must be absent, not a same-name replacement. Real parent listings
//! create identities and pending children. Planning is bounded to 4096 events,
//! 64 directory events and depth 128, and commits with the applied cursor.
//! Uncovered live-file parents, cycles, ambiguous names/IDs and unsupported
//! event kinds retain conservative recovery. Unknown Remove is deliberately
//! NOT localized: tombstones currently lack verified historical parent identity
//! or proof that the old parent's own metadata is current at the event fence.
use super::*;
use crate::Event;

pub(super) struct LocalDirectoryPlan {
    pub directory: String,
    pub parents: HashSet<String>,
}

/// A page-local dependency, NOT a candidate identity. Only the already observed
/// parent is scheduled; its native listing must discover (or disprove) this ID.
pub(super) struct UnknownDirectoryPlan {
    /// The real candidate anchor whose listing is the durable obligation.
    pub parent: String,
    path: String,
    depth: usize,
}

pub(super) fn plan_unknown_directory(
    db: &Connection,
    s: &InitializationStatus,
    node: &Node,
    hint: Option<&InitializationEventHint>,
) -> Result<Option<UnknownDirectoryPlan>> {
    let Some(hint) = hint else {
        return Ok(None);
    };
    if !node.directory
        || node.id == root_id(&s.binding)?
        || hint.kind != InitializationEventKind::Modify
        || hint.metadata_removed
        || hint.destination_path.is_some()
        || s.root_needs_refresh
    {
        return Ok(None);
    }
    let Some(parent) = node.parent.as_ref() else {
        return Ok(None);
    };
    if hint.metadata_parent_id.as_ref() != Some(parent) {
        return Ok(None);
    }
    let parent_node = match load_node(db, &s.run.run_id, parent) {
        Ok(node) if node.directory => node,
        Ok(_) => return Ok(None),
        Err(e) if e.is_not_found() => return Ok(None),
        Err(e) => return Err(e),
    };
    let Some((parent_path, chain)) = observed_path(db, s, &parent_node.id)? else {
        return Ok(None);
    };
    for ancestor in &chain {
        let denied: bool = db.query_row(
            "SELECT EXISTS(SELECT 1 FROM preview_denied WHERE run=?1 AND id=?2)",
            params![s.run.run_id, ancestor.id],
            |row| row.get(0),
        )?;
        if denied {
            return Ok(None);
        }
    }
    let root_invalid: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM preview_visibility WHERE run=?1 AND id=?2 AND root_invalid!=0)",
        params![s.run.run_id, root_id(&s.binding)?], |row| row.get(0))?;
    if root_invalid {
        return Ok(None);
    }
    let expected = format!("{}/{}", parent_path.trim_end_matches('/'), node.name);
    if hint.event_path != expected {
        return Ok(None);
    }
    let collision: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM initialization_entries WHERE run=?1 AND parent=?2 AND name=?3)",
        params![s.run.run_id, parent, node.name], |row| row.get(0))?;
    if collision {
        return Ok(None);
    }
    let orphan: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM initialization_entries WHERE run=?1 AND parent=?2)",
        params![s.run.run_id, node.id],
        |row| row.get(0),
    )?;
    if orphan || chain.len() > 128 {
        return Ok(None);
    }
    Ok(Some(UnknownDirectoryPlan {
        parent: parent.clone(),
        path: expected,
        depth: chain.len(),
    }))
}

/// Extend only a page-local dependency whose ancestor was already proved.
/// This never supplies a Node/native ID to the candidate. The first real parent
/// listing creates a pending directory; later real listings discover this edge.
/// An unresolved cycle or missing edge cannot bootstrap itself into a plan.
pub(super) fn plan_nested_unknown_directory(
    db: &Connection,
    s: &InitializationStatus,
    node: &Node,
    hint: Option<&InitializationEventHint>,
    parent_plan: &UnknownDirectoryPlan,
) -> Result<Option<UnknownDirectoryPlan>> {
    let Some(hint) = hint else {
        return Ok(None);
    };
    if !node.directory
        || node.id == root_id(&s.binding)?
        || hint.kind != InitializationEventKind::Modify
        || hint.metadata_removed
        || hint.destination_path.is_some()
        || node.parent.is_none()
        || hint.metadata_parent_id != node.parent
        || s.root_needs_refresh
        || parent_plan.depth >= 128
    {
        return Ok(None);
    }
    let expected = format!("{}/{}", parent_plan.path.trim_end_matches('/'), node.name);
    if hint.event_path != expected {
        return Ok(None);
    }
    // A missing ID with existing children would violate the closed-tree proof.
    // Likewise never walk through an existing same-name, different-ID object.
    let conflict: bool = db.query_row(
        "SELECT EXISTS(SELECT 1 FROM initialization_entries WHERE run=?1 AND parent=?2) OR EXISTS(SELECT 1 FROM initialization_entries WHERE run=?1 AND parent=?3 AND name=?4)",
        params![s.run.run_id,node.id,node.parent,node.name],|row|row.get(0))?;
    if conflict {
        return Ok(None);
    }
    Ok(Some(UnknownDirectoryPlan {
        parent: parent_plan.parent.clone(),
        path: expected,
        depth: parent_plan.depth + 1,
    }))
}

pub(super) fn covers_unknown_parent_file(
    plan: &UnknownDirectoryPlan,
    node: &Node,
    hint: Option<&InitializationEventHint>,
) -> bool {
    hint.is_some_and(|hint| {
        !node.directory
            && hint.kind == InitializationEventKind::Modify
            && !hint.metadata_removed
            && hint.destination_path.is_none()
            && hint.metadata_parent_id == node.parent
            && hint.event_path == format!("{}/{}", plan.path.trim_end_matches('/'), node.name)
    })
}

/// Only known, non-denied ancestry with same-row native identities is eligible.
/// A missing identity remains conservative; corrupt evidence is an error.
fn observed_path(
    db: &Connection,
    s: &InitializationStatus,
    id: &str,
) -> Result<Option<(String, Vec<Node>)>> {
    let chain = directory_chain(db, &s.run.run_id, &s.binding, id)?;
    let root = root_id(&s.binding)?;
    for node in &chain {
        let coverage: String = db.query_row(
            "SELECT state FROM initialization_directories WHERE run=?1 AND id=?2",
            params![s.run.run_id, node.id],
            |row| row.get(0),
        )?;
        if !matches!(
            coverage.as_str(),
            "pending" | "failed" | "loading" | "received"
        ) {
            return Ok(None);
        }
        let identity = if node.id == root {
            s.binding
                .scope
                .get("node_id")
                .and_then(|v| v.as_u64())
                .filter(|n| *n != 0)
        } else {
            read_native_identity(db, &s.run.run_id, &node.id)?
        };
        if identity.is_none() {
            return Ok(None);
        }
    }
    let selected = s
        .binding
        .scope
        .get("path")
        .and_then(|v| v.as_str())
        .ok_or_else(|| invalid("candidate scope path missing"))?;
    let mut path = selected.trim_end_matches('/').to_owned();
    for node in chain.iter().rev().skip(1) {
        path.push('/');
        path.push_str(&node.name);
    }
    if path.is_empty() {
        path.push('/');
    }
    Ok(Some((path, chain)))
}

pub(super) fn plan_known_directory(
    db: &Connection,
    s: &InitializationStatus,
    event: &Event,
    hint: Option<&InitializationEventHint>,
    old: Option<&Node>,
) -> Result<Option<LocalDirectoryPlan>> {
    let (Some(hint), Some(old)) = (hint, old) else {
        return Ok(None);
    };
    if !old.directory || old.id == root_id(&s.binding)? {
        return Ok(None);
    }
    if matches!(&event.change, Change::Upsert(node) if !node.directory) {
        return Ok(None);
    }
    let Some((path, _)) = observed_path(db, s, &old.id)? else {
        return Ok(None);
    };
    // In particular, a historical move already reflected at destination does
    // not let current metadata masquerade as the historical source parent.
    if path != hint.event_path {
        return Ok(None);
    }
    let source_parent = old
        .parent
        .as_ref()
        .ok_or_else(|| invalid("non-root directory lacks parent"))?;
    let mut parents = HashSet::from([source_parent.clone()]);
    if hint.kind == InitializationEventKind::Move {
        let destination = hint
            .destination_path
            .as_deref()
            .ok_or_else(|| invalid("move path missing"))?;
        let selected = s
            .binding
            .scope
            .get("path")
            .and_then(|v| v.as_str())
            .unwrap();
        let inside = selected == "/"
            || destination == selected
            || destination
                .strip_prefix(selected)
                .is_some_and(|rest| rest.starts_with('/'));
        if inside {
            let target = hint
                .destination_parent_id
                .as_deref()
                .ok_or_else(|| invalid("move parent missing"))?;
            let parent = match load_node(db, &s.run.run_id, target) {
                Ok(parent) if parent.directory => parent,
                Ok(_) => return Ok(None),
                Err(e) if e.is_not_found() => return Ok(None),
                Err(e) => return Err(e),
            };
            let Some((target_path, chain)) = observed_path(db, s, &parent.id)? else {
                return Ok(None);
            };
            let expected = destination
                .rsplit_once('/')
                .map(|(p, _)| if p.is_empty() { "/" } else { p })
                .ok_or_else(|| invalid("move destination lacks parent"))?;
            if target_path != expected || chain.iter().any(|node| node.id == old.id) {
                return Ok(None);
            }
            parents.insert(parent.id);
        }
    } else if let Change::Upsert(node) = &event.change {
        if node.parent.as_ref() != Some(source_parent)
            || hint.metadata_parent_id.as_ref() != Some(source_parent)
        {
            return Ok(None);
        }
    }
    Ok(Some(LocalDirectoryPlan {
        directory: old.id.clone(),
        parents,
    }))
}

/// Avoid repeatedly walking overlapping subtrees from a large event page.
pub(super) fn invalidate_local_subtrees(
    db: &Connection,
    s: &InitializationStatus,
    roots: &HashSet<String>,
) -> Result<()> {
    for id in roots {
        let chain = directory_chain(db, &s.run.run_id, &s.binding, id)?;
        if !chain
            .iter()
            .skip(1)
            .any(|parent| roots.contains(&parent.id))
        {
            invalidate(db, &s.run.run_id, id, true)?;
        }
    }
    Ok(())
}
