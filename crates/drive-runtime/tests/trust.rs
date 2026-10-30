use drive_runtime::trust::TrustSettings;
use std::os::unix::fs::{PermissionsExt, symlink};
const CA: &str = include_str!("../../sproto/tests/fixtures/trust/ca.pem");
#[test]
fn task_trust_is_private_persistent_and_empty_override_removes_profile_defaults() {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let original = TrustSettings::read(d.path(), "task", &[CA.into()]).unwrap();
    assert_eq!(original.trusted_ca_pem.len(), 1);
    original.save(d.path()).unwrap();
    let path = d.path().join("connection-trust.json");
    assert_eq!(path.metadata().unwrap().permissions().mode() & 0o777, 0o600);
    let loaded = TrustSettings::read(d.path(), "task", &[]).unwrap();
    assert_eq!(loaded.fingerprint(), original.fingerprint());
    TrustSettings {
        schema: 1,
        task_id: "task".into(),
        trusted_ca_pem: vec![],
    }
    .save(d.path())
    .unwrap();
    assert!(
        TrustSettings::read(d.path(), "task", &[CA.into()])
            .unwrap()
            .trusted_ca_pem
            .is_empty()
    );
    assert!(TrustSettings::read(d.path(), "different-task", &[]).is_err());
}
#[test]
fn invalid_settings_are_never_published_and_existing_trust_remains() {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let original = TrustSettings {
        schema: 1,
        task_id: "task".into(),
        trusted_ca_pem: vec![CA.into()],
    };
    original.save(d.path()).unwrap();
    let invalid = TrustSettings {
        schema: 1,
        task_id: "task".into(),
        trusted_ca_pem: vec!["not a certificate".into()],
    };
    assert!(invalid.save(d.path()).is_err());
    assert_eq!(
        TrustSettings::read(d.path(), "task", &[])
            .unwrap()
            .fingerprint(),
        original.fingerprint()
    );
}
#[test]
fn shared_file_symlink_fifo_unknown_fields_and_bad_version_fail_closed() {
    let d = tempfile::tempdir().unwrap();
    std::fs::set_permissions(d.path(), std::fs::Permissions::from_mode(0o700)).unwrap();
    let path = d.path().join("connection-trust.json");
    let settings = TrustSettings {
        schema: 1,
        task_id: "task".into(),
        trusted_ca_pem: vec![],
    };
    settings.save(d.path()).unwrap();
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).unwrap();
    assert!(TrustSettings::read(d.path(), "task", &[]).is_err());
    std::fs::remove_file(&path).unwrap();
    let target = d.path().join("target");
    std::fs::write(&target, serde_json::to_vec(&settings).unwrap()).unwrap();
    symlink(&target, &path).unwrap();
    assert!(TrustSettings::read(d.path(), "task", &[]).is_err());
    std::fs::remove_file(&path).unwrap();
    use std::os::unix::ffi::OsStrExt;
    let name = std::ffi::CString::new(path.as_os_str().as_bytes()).unwrap();
    assert_eq!(unsafe { libc::mkfifo(name.as_ptr(), 0o600) }, 0);
    assert!(TrustSettings::read(d.path(), "task", &[]).is_err());
    std::fs::remove_file(&path).unwrap();
    for text in [
        r#"{"schema":2,"task_id":"task","trusted_ca_pem":[]}"#,
        r#"{"schema":1,"task_id":"task","trusted_ca_pem":[],"skip_verify":true}"#,
    ] {
        std::fs::write(&path, text).unwrap();
        std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o600)).unwrap();
        assert!(TrustSettings::read(d.path(), "task", &[]).is_err());
    }
}
