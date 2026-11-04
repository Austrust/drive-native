use super::*;
pub(super) use drive_core::write_permissions::Action;
pub(super) fn node(store: &Store, id: &str, action: Action) -> drive_core::Result<()> {
    let Some(root) = drive_core::write_permissions::filesystem_root(store)? else {
        return Ok(());
    };
    let result = drive_core::write_permissions::enforce(store, &root, id, action);
    if let Err(Error::Io(error)) = &result {
        if error.raw_os_error() == Some(libc::EACCES) {
            diagnostics::count(diagnostics::Stage::PermissionDenied);
        }
        if error.raw_os_error() == Some(libc::EROFS) {
            diagnostics::count(diagnostics::Stage::SpaceForbidden);
        }
    }
    result
}
pub(super) fn inode(store: &Store, ino: u64, action: Action) -> drive_core::Result<()> {
    let record = store.posix_record_by_inode(ino)?;
    node(store, &record.id, action)
}
pub(super) fn remove(
    store: &Store,
    parent: &str,
    id: &str,
    directory: bool,
) -> drive_core::Result<()> {
    node(store, parent, Action::Delete)?;
    node(store, id, Action::Delete)?;
    if directory {
        node(store, parent, Action::CreateDir)?;
    }
    Ok(())
}
pub(super) fn rename(
    store: &Store,
    parent: &str,
    id: &str,
    target_parent: &str,
    target: Option<&str>,
) -> drive_core::Result<()> {
    node(store, parent, Action::RenameSource)?;
    node(store, id, Action::RenameSource)?;
    node(store, target_parent, Action::RenameTarget)?;
    if let Some(target) = target {
        node(store, target_parent, Action::Delete)?;
        node(store, target, Action::Delete)?;
    }
    Ok(())
}
/// No-denial metadata scans pay one atomic read. Once a denial is published,
/// current Store decisions are required; a busy writer yields EAGAIN, never
/// a stale permission grant. Existing access_denied/preview gates still apply.
pub(super) fn read(store: &drive_core::SharedStore, ino: u64, action: Action) -> Result<(), i32> {
    if !store.has_permission_denials() {
        return Ok(());
    }
    let guard = store.try_lock().map_err(|_| libc::EAGAIN)?;
    inode(&guard, ino, action).map_err(|e| errno(&e))
}
pub(super) fn publish_read(
    store: &drive_core::SharedStore,
    ino: u64,
    send: impl FnOnce(Result<(), i32>),
) {
    if let Some(lease) = store.permission_clear_lease() {
        send(Ok(()));
        drop(lease);
        return;
    }
    let guard = store.lock().unwrap();
    send(inode(&guard, ino, Action::Read).map_err(|e| errno(&e)));
    drop(guard);
}
pub(super) fn project(
    store: &Store,
    record: &mut drive_core::filesystem::PosixRecord,
) -> drive_core::Result<()> {
    node(store, &record.id, Action::Traverse)?;
    let Some(root) = drive_core::write_permissions::filesystem_root(store)? else {
        return Ok(());
    };
    use drive_core::write_permissions::{Decision, decide};
    if decide(
        store,
        &root,
        &record.id,
        if record.directory {
            Action::List
        } else {
            Action::Read
        },
    )? == Decision::Deny
    {
        record.mode &= !0o444;
    }
    // A single denied directory operation must not make DefaultPermissions
    // reject a different allowed operation. Unknown access() stays optimistic.
    let actions: &[Action] = if record.directory {
        &[
            Action::CreateFile,
            Action::CreateDir,
            Action::Delete,
            Action::RenameSource,
            Action::RenameTarget,
            Action::Setattr,
        ]
    } else {
        &[Action::WriteContent, Action::Setattr]
    };
    let mut all_denied = true;
    for action in actions {
        all_denied &= decide(store, &root, &record.id, *action)? == Decision::Deny;
    }
    if all_denied {
        record.mode &= !0o222;
    }
    Ok(())
}
