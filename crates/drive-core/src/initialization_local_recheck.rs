//! Narrow local scheduling for an already known non-root directory. Native
//! identities are prior list observations, not proof of the current path.
//! Re-list the affected parents/subtree; never copy event metadata into entries.
use super::*;
use crate::Event;

pub(super) struct LocalDirectoryPlan {
    pub directory: String,
    pub parents: HashSet<String>,
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
