use super::*;
use crate::trust::{Publication, TrustSettings};
use drive_core::{Node, NodeMetadata};
use std::os::unix::fs::PermissionsExt;
const CA: &str = include_str!("../../sproto/tests/fixtures/trust/ca.pem");
fn profile() -> Profile {
    Profile {
        schema: 1,
        task_id: uuid::Uuid::new_v4().to_string(),
        host: "unused.invalid".into(),
        tls_name: "unused.invalid".into(),
        trusted_ca_pem: vec![CA.into()],
        username: "synthetic".into(),
        device_id: uuid::Uuid::new_v4().to_string(),
        scope: drive_core::native::Scope {
            view_id: 1,
            node_id: 1,
            file_id: "root".into(),
            path: "/test".into(),
        },
        mount: PathBuf::from("/tmp/trust-fault-test-unused"),
        writable: false,
    }
}
fn fixture() -> (tempfile::TempDir, Arc<Service>) {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let mut store = Store::open(d.path()).unwrap();
    let nodes = [
        Node {
            id: "root".into(),
            parent: None,
            name: "root".into(),
            version: "v1".into(),
            directory: true,
            metadata: Some(NodeMetadata {
                size: 0,
                mtime: 1,
                mode: 0o755,
            }),
        },
        Node {
            id: "one".into(),
            parent: Some("root".into()),
            name: "one".into(),
            version: "v1".into(),
            directory: false,
            metadata: Some(NodeMetadata {
                size: 3,
                mtime: 1,
                mode: 0o644,
            }),
        },
    ];
    store
        .install_remote_snapshot(&nodes, "1", "server")
        .unwrap();
    store.ensure_fs_root("root").unwrap();
    store
        .publish_download(
            &drive_core::DownloadTicket {
                id: "one".into(),
                version: "v1".into(),
            },
            b"old",
            &drive_core::sha256(b"old"),
        )
        .unwrap();
    let service = Service::new(profile(), store, PathBuf::from("/unused-worker")).unwrap();
    let settings = service.trust_settings();
    settings.save(d.path()).unwrap();
    (d, service)
}
fn changed(s: &Service) -> TrustSettings {
    TrustSettings {
        schema: 1,
        task_id: s.profile.task_id.clone(),
        trusted_ca_pem: vec![],
    }
}
fn fail_after_rename(settings: &TrustSettings, directory: &Path) -> io::Result<Publication> {
    settings.publish_with(directory, |file, after| {
        if after {
            Err(io::Error::from_raw_os_error(libc::EIO))
        } else {
            file.sync_all()
        }
    })
}
fn assert_cached(s: &Service) {
    let store = s.bridge.store.lock().unwrap();
    let handle = store.open_cached("one").unwrap();
    let mut bytes = [0u8; 3];
    assert_eq!(handle.read_at(&mut bytes, 0).unwrap(), 3);
    assert_eq!(&bytes, b"old");
    assert_eq!(store.pending_operations().unwrap(), 0);
}
#[test]
fn pre_rename_sync_failure_preserves_old_settings_and_connection_generation() {
    let (d, s) = fixture();
    let old = s.trust_settings();
    let generation = s.control.lock().unwrap().generation;
    let error = s
        .set_trust_settings_with(&old.fingerprint(), changed(&s), |settings, dir| {
            settings.publish_with(dir, |_, _| Err(io::Error::from_raw_os_error(libc::ENOSPC)))
        })
        .unwrap_err();
    assert_eq!(error.raw_os_error(), Some(libc::ENOSPC));
    assert_eq!(s.trust_settings().fingerprint(), old.fingerprint());
    assert_eq!(
        TrustSettings::read(d.path(), &s.profile.task_id, &[])
            .unwrap()
            .fingerprint(),
        old.fingerprint()
    );
    assert_eq!(s.control.lock().unwrap().generation, generation);
    assert!(!s.control.lock().unwrap().trust_pending_durability);
    assert_cached(&s);
}
#[test]
fn post_rename_sync_failure_adopts_visible_removal_blocks_unlock_and_requires_current_save() {
    let (d, s) = fixture();
    s.unlock("synthetic-not-sent".into()).unwrap();
    let old = s.trust_settings();
    let generation = s.control.lock().unwrap().generation;
    assert!(
        s.set_trust_settings_with(&old.fingerprint(), changed(&s), fail_after_rename)
            .is_err()
    );
    let visible = s.trust_settings();
    assert!(visible.trusted_ca_pem.is_empty());
    assert_eq!(
        TrustSettings::read(d.path(), &s.profile.task_id, &[CA.into()])
            .unwrap()
            .fingerprint(),
        visible.fingerprint()
    );
    {
        let c = s.control.lock().unwrap();
        assert!(c.attention && c.trust_pending_durability);
        assert_eq!(c.state, ConnectionState::NeedsAttention);
        assert_eq!(c.error, Some("trust_settings_not_durable"));
        assert!(c.generation > generation);
    }
    s.pause().unwrap();
    s.resume().unwrap();
    assert_eq!(
        s.control.lock().unwrap().state,
        ConnectionState::NeedsAttention
    );
    assert!(s.unlock("synthetic-not-sent".into()).is_err());
    assert!(
        s.set_trust_settings(&old.fingerprint(), visible.clone())
            .is_err()
    );
    assert_cached(&s);
    s.set_trust_settings(&visible.fingerprint(), visible.clone())
        .unwrap();
    let c = s.control.lock().unwrap();
    assert!(!c.attention && !c.trust_pending_durability);
    assert_eq!(c.state, ConnectionState::Connecting);
    assert_eq!(c.error, None);
}
#[test]
fn failed_save_and_recovery_preserve_pause() {
    let (_d, s) = fixture();
    s.pause().unwrap();
    let old = s.trust_settings();
    assert!(
        s.set_trust_settings_with(&old.fingerprint(), changed(&s), fail_after_rename)
            .is_err()
    );
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Paused);
    let current = s.trust_settings();
    s.set_trust_settings(&current.fingerprint(), current)
        .unwrap();
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Paused);
    assert_cached(&s);
}
#[test]
fn restart_confirms_visible_settings_and_startup_sync_failure_keeps_cache_available() {
    let (d, s) = fixture();
    let old = s.trust_settings();
    assert!(
        s.set_trust_settings_with(&old.fingerprint(), changed(&s), fail_after_rename)
            .is_err()
    );
    let profile = s.profile.clone();
    drop(s);
    let restarted = Service::new_with_trust_sync(
        profile.clone(),
        Store::open(d.path()).unwrap(),
        PathBuf::from("/unused-worker"),
        |_| Err(io::Error::from_raw_os_error(libc::EIO)),
    )
    .unwrap();
    assert!(restarted.trust_settings().trusted_ca_pem.is_empty());
    assert_eq!(
        restarted.control.lock().unwrap().state,
        ConnectionState::NeedsAttention
    );
    assert!(restarted.unlock("synthetic-not-sent".into()).is_err());
    assert_cached(&restarted);
    drop(restarted);
    let recovered = Service::new(
        profile,
        Store::open(d.path()).unwrap(),
        PathBuf::from("/unused-worker"),
    )
    .unwrap();
    assert!(recovered.trust_settings().trusted_ca_pem.is_empty());
    assert_eq!(
        recovered.control.lock().unwrap().state,
        ConnectionState::Locked
    );
    recovered.unlock("synthetic-not-sent".into()).unwrap();
    assert_cached(&recovered);
}

#[test]
fn transfer_settings_are_task_bound_persisted_and_preserve_pause() {
    use crate::desktop;
    let (d, s) = fixture();
    s.pause().unwrap();
    let old: serde_json::Value =
        serde_json::from_str(&desktop::transfer_settings(&s).unwrap()).unwrap();
    let mut request = serde_json::json!({"schema":1,"task_id":"wrong", "expected_version":old["version"],"limits":{"download_bps":65536,"upload_bps":32768}});
    assert!(desktop::set_transfer_settings(&s, &request.to_string()).is_err());
    request["task_id"] = s.profile.task_id.clone().into();
    let updated = desktop::set_transfer_settings(&s, &request.to_string()).unwrap();
    assert_ne!(
        serde_json::from_str::<serde_json::Value>(&updated).unwrap()["version"],
        old["version"]
    );
    assert!(desktop::set_transfer_settings(&s, &request.to_string()).is_err());
    assert_eq!(s.control.lock().unwrap().state, ConnectionState::Paused);
    assert_cached(&s);
    drop(s);
    let store = Store::open(d.path()).unwrap();
    assert_eq!(store.transfer_limits().unwrap().download_bps, 65536);
    assert_eq!(store.transfer_limits().unwrap().upload_bps, 32768);
}

#[test]
fn live_transfer_setting_acknowledgement_is_required_without_losing_saved_policy() {
    use crate::{rpc::Rpc, transport, wire::*};
    use drive_core::transfer_policy::TransferLimits;
    use std::os::unix::net::UnixStream;
    for acknowledged in [false, true] {
        let (_d, s) = fixture();
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(a).unwrap());
        let (done, rx) = std::sync::mpsc::channel();
        let peer = std::thread::spawn(move || {
            let (request, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            let Command::ConfigureTransfers(limits) = request.body else {
                panic!("unexpected remote operation")
            };
            let body = if acknowledged {
                Response::TransfersConfigured(limits)
            } else {
                Response::Failed {
                    category: "synthetic_policy_rejected".into(),
                }
            };
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: request.id,
                    body,
                },
                None,
            )
            .unwrap();
            rx.recv().unwrap();
        });
        {
            let mut c = s.control.lock().unwrap();
            c.state = ConnectionState::Online;
            c.active = Some(rpc.clone());
        }
        let limits = TransferLimits {
            download_bps: 65536,
            upload_bps: 32768,
        };
        let result = s.set_transfer_limits(&TransferLimits::default().fingerprint(), limits);
        assert_eq!(result.is_ok(), acknowledged);
        assert_eq!(
            s.bridge.store.lock().unwrap().transfer_limits().unwrap(),
            limits
        );
        assert_eq!(
            s.control.lock().unwrap().state,
            if acknowledged {
                ConnectionState::Online
            } else {
                ConnectionState::Offline
            }
        );
        assert_cached(&s);
        done.send(()).unwrap();
        peer.join().unwrap();
        rpc.close();
    }
}

#[test]
fn durable_pause_controls_handover_and_unlock() {
    let (d, service) = fixture();
    service.pause().unwrap();
    let profile = service.profile.clone();
    drop(service);
    let store = Store::open(d.path()).unwrap();
    let service = Service::new(profile, store, PathBuf::from("/unused-worker")).unwrap();
    assert_eq!(service.control.lock().unwrap().state, ConnectionState::Paused);
    service.unlock("synthetic-unused".into()).unwrap();
    assert_eq!(service.control.lock().unwrap().state, ConnectionState::Paused);
    service.resume().unwrap();
    assert!(!service.control.lock().unwrap().paused);
    assert!(!service.bridge.store.lock().unwrap().sync_paused().unwrap());
}

#[test]
fn installed_worker_replacement_keeps_old_executable_inode() {
    let (d, service) = fixture();
    let profile = service.profile.clone();
    drop(service);
    let worker = d.path().join("worker");
    fs::copy("/bin/true", &worker).unwrap();
    let service = Service::new(profile, Store::open(d.path()).unwrap(), worker.clone()).unwrap();
    let replacement = d.path().join("replacement");
    fs::copy("/bin/false", &replacement).unwrap();
    fs::rename(replacement, worker).unwrap();
    let pinned = service.worker_executable().unwrap();
    let original = fs::metadata(&pinned).unwrap();
    assert!(std::process::Command::new(&pinned).status().unwrap().success());
    drop(service);
    assert!(fs::metadata(pinned).map_or(true, |m| m.ino() != original.ino() || m.dev() != original.dev()));
}

#[test]
fn missing_worker_image_cannot_fall_back_to_replaced_path() {
    let (_d, service) = fixture();
    assert_eq!(service.worker_executable().unwrap_err().kind(), io::ErrorKind::InvalidInput);
}
