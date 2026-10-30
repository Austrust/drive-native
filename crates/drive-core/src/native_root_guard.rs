//! A watch validates the selected (view, node, path) tuple on the server.
//! Its hint is not a committed cursor: the fsd still consumes every event from
//! its durable cursor. No directory scan and no local deletion happen here.
use super::*;
use sproto::actions::native::NativeEvent;
use std::sync::atomic::Ordering;

pub(super) fn root_event(scope: &Scope, event: &NativeEvent) -> Result<()> {
    let path = match &event.kind {
        EventKind::Move { destination, .. } => destination,
        _ => &event.path,
    };
    if event.file_id == scope.file_id {
        if event.removed
            || matches!(event.kind, EventKind::Remove)
            || path != &scope.path
            || event.file_type != FileType::Dir
        {
            return Err(Error::RootUnavailable);
        }
    } else if path == &scope.path || event.path == scope.path {
        return Err(Error::RootUnavailable);
    }
    Ok(())
}
impl NativeReadSession {
    pub(super) fn root_guard_result<T>(&self, result: Result<T>) -> Result<T> {
        if matches!(result, Err(Error::RootUnavailable)) {
            self.root_lost.store(true, Ordering::Release);
        }
        result
    }
    pub(super) async fn scoped_events(&self, watch: &Watch) -> Result<EventBatch> {
        if self.root_lost.load(Ordering::Acquire) {
            return Err(Error::RootUnavailable);
        }
        let result = self
            .client
            .pull_events(watch)
            .await
            .map_err(|error| match error {
                sproto::Error::Server { code: 12291, .. } => Error::RootUnavailable,
                error => Error::Native(error),
            });
        let mut batch = self.root_guard_result(result)?;
        if batch
            .events
            .iter()
            .any(|e| root_event(&self.scope, e).is_err())
        {
            // A restored directory still has its move-away in historical events.
            // Verify the current file AND node identity before discarding only
            // that intermediate move. A tombstone/replacement is never hidden.
            let current = if root_roundtrip(&self.scope, &batch) {
                self.current_root_matches().await?
            } else {
                false
            };
            self.root_guard_result(normalize_root_roundtrip(&self.scope, &mut batch, current))?;
        }
        if !batch.recovery_required {
            self.root_hint.fetch_max(batch.next.0, Ordering::AcqRel);
        }
        // Another in-flight request can discover root loss while this one awaits.
        if self.root_lost.load(Ordering::Acquire) {
            return Err(Error::RootUnavailable);
        }
        Ok(batch)
    }
    async fn current_root_matches(&self) -> Result<bool> {
        if self.scope.path == "/" {
            return Ok(false);
        }
        let (parent, name) = self.scope.path.rsplit_once('/').unwrap();
        let entries = self
            .client
            .list_dir(
                self.scope.view_id,
                if parent.is_empty() { "/" } else { parent },
            )
            .await?;
        let mut matched = entries.iter().filter(|n| n.name == name && !n.is_removed);
        Ok(matches!((matched.next(),matched.next()),(Some(n),None)
            if n.file_id==self.scope.file_id && n.node_id==self.scope.node_id && n.file_type==FileType::Dir))
    }
    pub(super) async fn verify_root(&self) -> Result<()> {
        let mut watch = self.watch("")?;
        watch.cursor = EventCursor(self.root_hint.load(Ordering::Acquire));
        let batch = self.scoped_events(&watch).await?;
        if batch.recovery_required {
            return Err(Error::Unsupported(
                "native root verification needs event recovery",
            ));
        }
        Ok(())
    }
}

// Deliberately bounded to a complete root move-away/return chain in this batch.
// Split batches or tombstones remain a recovery gate, not guessed continuity.
fn root_roundtrip(scope: &Scope, batch: &EventBatch) -> bool {
    let last = batch
        .events
        .iter()
        .rev()
        .find(|e| e.file_id == scope.file_id);
    last.is_some_and(|e| root_event(scope, e).is_ok())
        && batch.events.iter().all(|e| {
            root_event(scope, e).is_ok()
                || (e.file_id == scope.file_id
                    && !e.removed
                    && e.file_type == FileType::Dir
                    && matches!(e.kind, EventKind::Move { .. }))
        })
}
fn normalize_root_roundtrip(
    scope: &Scope,
    batch: &mut EventBatch,
    current_matches: bool,
) -> Result<()> {
    if !current_matches || !root_roundtrip(scope, batch) {
        return Err(Error::RootUnavailable);
    }
    batch.events.retain(|e| root_event(scope, e).is_ok());
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn root_disappearance_is_never_translated_into_recursive_local_delete() {
        let scope = super::super::tests::scope();
        for kind in [
            EventKind::Remove,
            EventKind::Move {
                destination: "/lab/moved".into(),
                parent_id: "parent".into(),
            },
        ] {
            let mut event = super::super::tests::event();
            event.file_id = scope.file_id.clone();
            event.path = scope.path.clone();
            event.file_type = FileType::Dir;
            event.kind = kind;
            assert!(matches!(
                translate(
                    &scope,
                    "server",
                    "native-v1:4",
                    super::super::tests::batch(event, 5)
                ),
                Err(Error::RootUnavailable)
            ));
        }
        let mut event = super::super::tests::event();
        event.path = scope.path.clone();
        event.file_type = FileType::Dir;
        assert!(matches!(
            translate(
                &scope,
                "server",
                "native-v1:4",
                super::super::tests::batch(event, 5)
            ),
            Err(Error::RootUnavailable)
        ));
    }
    #[test]
    fn root_metadata_and_child_tombstones_remain_valid() {
        let scope = super::super::tests::scope();
        let mut event = super::super::tests::event();
        event.removed = true;
        event.kind = EventKind::Remove;
        assert!(root_event(&scope, &event).is_ok());
        event.file_id = scope.file_id.clone();
        event.path = scope.path.clone();
        event.file_type = FileType::Dir;
        event.removed = false;
        event.kind = EventKind::Modify;
        assert!(root_event(&scope, &event).is_ok());
        event.removed = true;
        assert!(matches!(
            root_event(&scope, &event),
            Err(Error::RootUnavailable)
        ));
    }
}

#[cfg(test)]
mod history_tests {
    use super::*;
    #[test]
    fn historical_move_requires_current_identity_and_a_return_event() {
        let scope = super::super::tests::scope();
        let mut away = super::super::tests::event();
        away.file_id = scope.file_id.clone();
        away.path = scope.path.clone();
        away.file_type = FileType::Dir;
        away.kind = EventKind::Move {
            destination: "/lab/moved".into(),
            parent_id: "parent".into(),
        };
        let mut back = away.clone();
        back.path = "/lab/moved".into();
        back.kind = EventKind::Move {
            destination: scope.path.clone(),
            parent_id: "parent".into(),
        };
        let mut batch = super::super::tests::batch(away.clone(), 6);
        batch.events.push(back.clone());
        assert!(normalize_root_roundtrip(&scope, &mut batch, false).is_err());
        normalize_root_roundtrip(&scope, &mut batch, true).unwrap();
        assert_eq!(batch.events.len(), 1);
        let translated = translate(&scope, "server", "native-v1:4", batch)
            .unwrap()
            .unwrap();
        assert!(
            matches!(&translated.events[0].change,Change::Upsert(n) if n.id==scope.file_id && n.parent.is_none())
        );
        let mut single = super::super::tests::batch(away.clone(), 5);
        assert!(normalize_root_roundtrip(&scope, &mut single, true).is_err());
        away.kind = EventKind::Remove;
        away.removed = true;
        let mut removed = super::super::tests::batch(away, 6);
        removed.events.push(back);
        assert!(normalize_root_roundtrip(&scope, &mut removed, true).is_err());
    }
}
