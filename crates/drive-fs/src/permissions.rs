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
    match store.posix_record_by_inode(ino) {
        Ok(record) => node(store, &record.id, action),
        Err(error) if error.is_not_found() => {
            // Preview has its own current identity/denial checks and no body.
            // Do not turn its EAGAIN/EROFS contract into authoritative ENOENT.
            store.preview_record_by_inode(ino)?;
            match action {
                Action::Traverse | Action::List => Ok(()),
                _ => Err(Error::Io(std::io::Error::from_raw_os_error(
                    if action.is_write() { libc::EROFS } else { libc::EAGAIN },
                ))),
            }
        }
        Err(error) => Err(error),
    }
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
/// Body reads call this only on a worker. Wait for the current Store decision
/// instead of turning ordinary metadata contention into a spurious EAGAIN.
/// Dispatcher metadata handlers use their normal inline-or-queue routing.
pub(super) fn read_on_worker(store: &drive_core::SharedStore, ino: u64, action: Action) -> Result<(), i32> {
    if !store.has_permission_denials() {
        return Ok(());
    }
    let guard = store.lock().map_err(|_| libc::EIO)?;
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
    // Whole-task policy is EROFS at the actual operation (and at mount time).
    // Removing write bits here would make DefaultPermissions return EACCES
    // before that hook, particularly after a live user-policy change.
    if !drive_core::write_permissions::mount_writable(store, &root)? {
        return Ok(());
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
