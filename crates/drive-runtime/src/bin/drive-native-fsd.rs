use drive_core::status_changes::{ChangePage, StatusCursor};
use drive_runtime::{
    desktop::{Desktop, OBJECT_PATH},
    service::{Profile, Service, prepare_paths},
    status_events,
};
use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};

/// One revisioned delta of the committed status journal (#31).
struct StatusDelta {
    cursor: Option<StatusCursor>,
    epoch: String,
    document: String,
    reset: bool,
}

/// Read the committed journal under the serialized writer connection and turn
/// the changed node ids into absolute mount paths. Journal tables are
/// connection-local TEMP tables, so this must not use the read-only handle.
fn status_delta(
    service: &Service,
    from_revision: u64,
    to_revision: u64,
    last: Option<&StatusCursor>,
    last_epoch: &str,
    overview_changed: bool,
) -> StatusDelta {
    let mount = service.profile.mount.to_string_lossy().into_owned();
    let Ok(guard) = service.bridge.store.lock() else {
        let (reset, document) = status_events::change_document(
            from_revision,
            to_revision,
            last_epoch,
            &mount,
            true,
            overview_changed,
            Vec::new(),
        );
        return StatusDelta {
            cursor: None,
            epoch: last_epoch.to_owned(),
            document,
            reset,
        };
    };
    let watermark = guard.status_change_watermark().ok();
    let epoch = watermark
        .as_ref()
        .map(|watermark| watermark.cursor.epoch.clone())
        .unwrap_or_else(|| last_epoch.to_owned());
    let mut reset = watermark.is_none();
    let mut cursor = watermark.as_ref().map(|watermark| watermark.cursor.clone());
    let mut collected = Vec::new();
    if let Some(watermark) = watermark.as_ref() {
        match last {
            // The first observation only records the committed boundary; the
            // consumer's first delta is a full refresh anyway. A later tick
            // without a cursor (for example the previous delta returned reset)
            // must keep the reset sticky instead of emitting an empty
            // continuous delta that hides the missed window (verify-31 c1).
            None => reset = status_events::resets_without_cursor(from_revision),
            Some(last) if last.epoch != watermark.cursor.epoch => reset = true,
            Some(last) => {
                let mut current = last.clone();
                let mut rows = 0usize;
                while !reset {
                    match guard.read_status_changes(&current, status_events::MAX_CHANGES_PER_READ) {
                        Ok(ChangePage::Rebuild {
                            watermark: rebuilt, ..
                        }) => {
                            reset = true;
                            cursor = Some(rebuilt.cursor.clone());
                        }
                        Ok(ChangePage::Ready {
                            changes,
                            next,
                            watermark: page,
                            has_more,
                        }) => {
                            rows += changes.len();
                            collected.extend(changes);
                            current = next;
                            cursor = Some(page.cursor.clone());
                            if !has_more {
                                break;
                            }
                            if rows >= status_events::MAX_CHANGES_TOTAL {
                                reset = true;
                            }
                        }
                        Err(_) => reset = true,
                    }
                }
            }
        }
    }
    let mut change_paths = Vec::new();
    if !reset && !collected.is_empty() {
        let mut ids = Vec::new();
        for change in &collected {
            for id in [
                &change.new_node,
                &change.old_node,
                &change.new_parent,
                &change.old_parent,
            ]
            .into_iter()
            .flatten()
            {
                ids.push(id.clone());
            }
        }
        ids.sort();
        ids.dedup();
        match drive_core::desktop_status::desktop_status_ancestry_paths(&guard, &ids) {
            Ok(paths) => change_paths = status_events::change_paths(&collected, &paths),
            Err(_) => reset = true,
        }
    }
    drop(guard);
    let (reset, document) = status_events::change_document(
        from_revision,
        to_revision,
        &epoch,
        &mount,
        reset,
        overview_changed,
        change_paths,
    );
    StatusDelta {
        cursor,
        epoch,
        document,
        reset,
    }
}

#[tokio::main]
async fn main() {
    // No native error payloads, login material or file bodies go to stderr.
    if run().await.is_err() {
        eprintln!(
            "drive-native-fsd: startup or local service failed; inspect profile and local status"
        );
        std::process::exit(1);
    }
}
async fn run() -> Result<(), Box<dyn std::error::Error>> {
    drive_runtime::disable_core_dumps()?;
    let args: Vec<_> = std::env::args_os().collect();
    if args.len() != 3 || args[1] != "--config" {
        return Err("usage: drive-native-fsd --config <private-profile.json>".into());
    }
    let profile = Profile::read(&PathBuf::from(&args[2]))?;
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))
        .ok_or("no user data directory")?;
    let state = prepare_paths(&profile, &base)?;
    let worker = std::env::current_exe()?.with_file_name("drive-native-worker");
    let mut store = drive_core::Store::open(&state)?;
    store.consume_startup_pause(&profile.task_id)?;
    let service = Service::new(profile, store, worker)?;
    let conn = zbus::connection::Builder::session()
        .name(drive_runtime::instance::bus_name(
            std::env::var("DRIVE_NATIVE_INSTANCE").ok().as_deref(),
        )?)
        .serve_at(OBJECT_PATH, Desktop(service.clone()))
        .build()
        .await?;
    let mut terminate = tokio::signal::unix::signal(tokio::signal::unix::SignalKind::terminate())?;
    let tasks = service.start();
    // Separate signal path: no Store lock, no metadata/stat refresh on progress.
    let progress_service = service.clone();
    let progress_conn = conn.clone();
    let progress_task = tokio::spawn(async move {
        let mut tick = tokio::time::interval(Duration::from_millis(250));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
        let mut last = 0;
        while !progress_service.stop.load(Ordering::Acquire) {
            tick.tick().await;
            let revision = progress_service.bridge.initialization.revision();
            if revision != last {
                if let Ok(emitter) =
                    zbus::object_server::SignalEmitter::new(&progress_conn, OBJECT_PATH)
                {
                    let _ = Desktop::progress_changed(&emitter, revision).await;
                }
                last = revision;
            }
        }
    });
    let mut mounted: Option<fuser::BackgroundSession> = None;
    let mut registration: Option<drive_runtime::mount_registry::MountRegistration> = None;
    let mut last = None;
    let mut revision = 0u64;
    let mut semantic_revision = 0u64;
    let mut last_cursor: Option<StatusCursor> = None;
    let mut last_epoch = String::new();
    let mut tick = tokio::time::interval(Duration::from_millis(250));
    let result=async {
        loop {
            tokio::select!{_=tokio::signal::ctrl_c()=>{service.shutdown();},_=terminate.recv()=>{service.shutdown();},_=tick.tick()=>{}}
            if service.stop.load(Ordering::Acquire) {break;}
            if mounted.as_ref().is_some_and(|session|session.guard.is_finished()) {
                service.mounted.store(false,Ordering::Release);
                return Err("filesystem session ended externally".into());
            }
            if mounted.is_none() {
                let s=service.clone();mounted=tokio::task::spawn_blocking(move||s.mount_if_ready()).await??;
                if mounted.is_some() {
                    registration=Some(drive_runtime::mount_registry::MountRegistration::publish(
                        &service.profile.task_id,&service.profile.mount)?);
                }
            }
            let s=service.clone();
            let current=tokio::task::spawn_blocking(move||{
                let control=s.control.lock().unwrap().revision;
                // Semantic status watermark: only committed namespace/status
                // atoms move it, unlike the total_changes counter.
                let commit=s.bridge.store.change_state();
                let downloads=s.bridge.downloads.load(Ordering::Relaxed);
                let active=s.bridge.active_downloads();
                let error=s.bridge.last_error();
                (control,downloads,active.len(),error,s.bridge.transfer_revision(),commit.revision,commit.verified)
            }).await?;
            if last.as_ref()!=Some(&current) {
                revision+=1;
                Desktop::status_changed(&zbus::object_server::SignalEmitter::new(&conn,OBJECT_PATH)?,revision).await?;
                let to_revision=current.5;
                let overview_changed=last.as_ref().map_or(true,|previous|{
                    previous.0!=current.0||previous.1!=current.1||previous.2!=current.2
                        ||previous.3!=current.3||previous.4!=current.4||previous.6!=current.6
                });
                let semantic_changed=last.as_ref().map_or(true,|previous|{
                    previous.5!=current.5||previous.6!=current.6
                });
                let previous_epoch=last_epoch.clone();
                last=Some(current);
                if semantic_changed {
                    let s=service.clone();
                    let from_revision=semantic_revision;
                    let last_cursor_for=last_cursor.clone();
                    let last_epoch_for=previous_epoch;
                    let delta=tokio::task::spawn_blocking(move||{
                        status_delta(&s,from_revision,to_revision,last_cursor_for.as_ref(),&last_epoch_for,overview_changed)
                    }).await?;
                    semantic_revision=to_revision;
                    last_cursor=delta.cursor;
                    last_epoch=delta.epoch.clone();
                    let emitter=zbus::object_server::SignalEmitter::new(&conn,OBJECT_PATH)?;
                    Desktop::status_changed2(&emitter,from_revision,to_revision,&delta.epoch,delta.reset,overview_changed,&delta.document).await?;
                } else if overview_changed {
                    let (reset,document)=status_events::change_document(
                        semantic_revision,to_revision,&previous_epoch,
                        &service.profile.mount.to_string_lossy(),false,true,Vec::new());
                    let emitter=zbus::object_server::SignalEmitter::new(&conn,OBJECT_PATH)?;
                    Desktop::status_changed2(&emitter,semantic_revision,to_revision,&previous_epoch,reset,true,&document).await?;
                }
            }
        }
        Ok::<(),Box<dyn std::error::Error>>(())
    }.await;
    service.shutdown();
    progress_task.abort();
    let shutdown_service = service.clone();
    let mut shutdown = tokio::task::spawn_blocking(move || -> std::io::Result<()> {
        use drive_runtime::shutdown::Phase;
        let s = shutdown_service;
        s.shutdown_diagnostics.reason(Some("waiting_for_workers"));
        s.shutdown_diagnostics.phase(Phase::StoppingWorkers);
        tasks.join();
        s.bridge.clear_notifier();
        s.shutdown_diagnostics.reason(Some("unmounting_mount"));
        s.shutdown_diagnostics.phase(Phase::DetachingMount);
        let session_result = if let Some(session) = mounted {
            session.join_with_unmount(|| {
                if let Ok(present) = drive_runtime::mount_registry::mount_present(&s.profile.mount)
                {
                    s.mounted.store(present, Ordering::Release);
                }
                s.shutdown_diagnostics.reason(Some("draining_open_files"));
                s.shutdown_diagnostics.phase(Phase::DrainingFilesystem);
            })
        } else {
            Ok(())
        };
        // Session destruction has drained all accepted write/fsync callbacks.
        // Persist drafts even when applications kept their file descriptors open.
        // This is a local checkpoint, never a server upload or cache clear.
        s.shutdown_diagnostics.reason(Some("persisting_local_changes"));
        s.shutdown_diagnostics.phase(Phase::CheckpointingLocal);
        let checkpoint = s
            .bridge
            .store
            .lock()
            .unwrap()
            .checkpoint_recovered_drafts()
            .map_err(std::io::Error::other);
        let checkpoint_failed = checkpoint.is_err();
        let present = drive_runtime::mount_registry::mount_present(&s.profile.mount);
        if let Ok(value) = present.as_ref() {
            s.mounted.store(*value, Ordering::Release);
        }
        let mount_present = matches!(present, Ok(true));
        let outcome = session_result.and(checkpoint.map(|_| ())).and_then(|_| {
            if present? {
                Err(std::io::Error::other(
                    "mount remains after session shutdown",
                ))
            } else {
                Ok(())
            }
        });
        if outcome.is_ok() {
            s.shutdown_diagnostics.reason(None);
            s.shutdown_diagnostics.phase(Phase::Complete);
        } else {
            let reason = if mount_present {
                "mount_still_present"
            } else if checkpoint_failed {
                "checkpoint_failed"
            } else {
                "unmount_failed"
            };
            s.shutdown_diagnostics.reason(Some(reason));
            s.shutdown_diagnostics.phase(Phase::Failed);
        }
        outcome
    });
    // Keep the local bus responsive and report the precise slow phase; never
    // turn a timeout into SIGKILL or pretend an unfinished fsync succeeded.
    let mut warning = tokio::time::interval(Duration::from_millis(500));
    warning.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);
    let mut shutdown_observed = None;
    let mut warned_phase = None;
    let stopped = loop {
        tokio::select! {
            done=&mut shutdown => break done?,
            _=warning.tick()=>{
                let snapshot=service.shutdown_diagnostics.snapshot();
                let observed=(snapshot["phase"].clone(),snapshot["overdue"].clone(),service.mounted.load(Ordering::Acquire));
                if shutdown_observed.as_ref()!=Some(&observed) {
                    revision+=1;
                    if let Ok(emitter)=zbus::object_server::SignalEmitter::new(&conn,OBJECT_PATH) {
                        let _=Desktop::shutdown_changed(&emitter,revision).await;
                        let _=Desktop::status_changed(&emitter,revision).await;
                    }
                    shutdown_observed=Some(observed);
                }

                if snapshot["overdue"]==true && warned_phase.as_ref()!=snapshot["phase"].as_str().map(|s|s.to_owned()).as_ref() {
                    eprintln!("drive-native-fsd: shutdown overdue {}",snapshot);
                    warned_phase=snapshot["phase"].as_str().map(str::to_owned);
                }
            }
        }
    };
    drop(registration);
    stopped?;
    result
}
