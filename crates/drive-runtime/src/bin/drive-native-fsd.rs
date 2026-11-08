use drive_runtime::{
    desktop::{Desktop, OBJECT_PATH},
    service::{Profile, Service, prepare_paths},
};
use std::{path::PathBuf, sync::atomic::Ordering, time::Duration};

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
    let mut revision = 0;
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
                let local=s.bridge.store.lock().unwrap().local_revision();
                let downloads=s.bridge.downloads.load(Ordering::Relaxed);
                let active=s.bridge.active_downloads();
                let error=s.bridge.last_error();
                (control,local,downloads,active.len(),error,s.bridge.transfer_revision())
            }).await?;
            if last.as_ref()!=Some(&current) {
                revision+=1;
                Desktop::status_changed(&zbus::object_server::SignalEmitter::new(&conn,OBJECT_PATH)?,revision).await?;
                last=Some(current);
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
