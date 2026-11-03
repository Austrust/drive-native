//! Writable-source migration and mount policy matrix. No NAS, mount or keyring.
use super::*;
use drive_core::write_permissions::WritePermission;
use drive_fs::WritePolicy;
use std::os::unix::fs::OpenOptionsExt;

fn raw_profile(writable: bool, extra: serde_json::Value) -> serde_json::Value {
    let mut value = serde_json::json!({
        "schema": 1,
        "task_id": uuid::Uuid::new_v4().to_string(),
        "host": "fixture.invalid",
        "tls_name": "fixture.invalid",
        "username": "fixture",
        "device_id": uuid::Uuid::new_v4().to_string(),
        "scope": {"view_id": 1, "node_id": 1, "file_id": "root", "path": "/"},
        "mount": "/tmp/drive-native-policy-test",
        "writable": writable,
    });
    if let (Some(target), Some(source)) = (value.as_object_mut(), extra.as_object()) {
        for (key, value) in source {
            target.insert(key.clone(), value.clone());
        }
    }
    value
}
fn parse(value: serde_json::Value) -> Profile {
    let mut profile: Profile = serde_json::from_value(value).unwrap();
    profile.normalize_writable_source();
    profile
}

#[test]
fn legacy_read_only_profile_stays_read_only_after_migration() {
    let profile = parse(raw_profile(false, serde_json::json!({})));
    assert_eq!(profile.writable_source, WritableSource::User);
    assert!(!profile.writable);
    assert_eq!(profile.write_policy(false), WritePolicy::ReadOnly);
    // Even a directory-level allow cannot override an explicit user RO choice.
    assert_eq!(profile.write_policy(true), WritePolicy::ReadOnly);
}

#[test]
fn legacy_writable_profile_attributes_capability_to_the_remote_space() {
    let profile = parse(raw_profile(true, serde_json::json!({})));
    assert_eq!(profile.writable_source, WritableSource::RemoteShare);
    assert_eq!(
        profile.write_policy(false),
        WritePolicy::PerDirectory {
            fallback: WritePermission::Allowed
        }
    );
}

#[test]
fn remote_read_only_scope_only_mounts_rw_with_an_explicit_directory_allow() {
    let explicit = parse(raw_profile(
        false,
        serde_json::json!({"writable_source": "remote_share"}),
    ));
    assert_eq!(explicit.write_policy(false), WritePolicy::ReadOnly);
    assert_eq!(
        explicit.write_policy(true),
        WritePolicy::PerDirectory {
            fallback: WritePermission::Denied
        }
    );
    let user_choice = parse(raw_profile(
        false,
        serde_json::json!({"writable_source": "user"}),
    ));
    assert_eq!(user_choice.write_policy(true), WritePolicy::ReadOnly);
    let writable = parse(raw_profile(
        true,
        serde_json::json!({"writable_source": "remote_share"}),
    ));
    assert_eq!(
        writable.write_policy(false),
        WritePolicy::PerDirectory {
            fallback: WritePermission::Allowed
        }
    );
}

#[test]
fn profile_read_normalizes_legacy_sidecar_without_promoting_read_only() {
    let temp = tempfile::tempdir().unwrap();
    let path = temp.path().join("profile.json");
    let value = raw_profile(false, serde_json::json!({}));
    let mut file = fs::OpenOptions::new()
        .create_new(true)
        .write(true)
        .mode(0o600)
        .open(&path)
        .unwrap();
    use std::io::Write;
    file.write_all(serde_json::to_string(&value).unwrap().as_bytes())
        .unwrap();
    drop(file);
    let profile = Profile::read(&path).unwrap();
    assert_eq!(profile.writable_source, WritableSource::User);
    assert!(!profile.writable);

    // Re-encoding a normalized profile records the source for other readers.
    let encoded = serde_json::to_value(&profile).unwrap();
    assert_eq!(encoded["writable_source"], "user");
}

#[test]
fn explicit_source_is_preserved_and_legacy_is_resolved_in_memory_only() {
    let explicit = parse(raw_profile(
        true,
        serde_json::json!({"writable_source": "remote_share"}),
    ));
    assert_eq!(
        explicit.effective_writable_source(),
        WritableSource::RemoteShare
    );
    let legacy = parse(raw_profile(false, serde_json::json!({})));
    // Original profile object is unchanged until re-encoded by the caller.
    assert_eq!(legacy.writable_source, WritableSource::User);
    assert_eq!(legacy.effective_writable_source(), WritableSource::User);
}

#[test]
fn validate_keeps_release_gate_closed_for_writable_native() {
    let mut profile = parse(raw_profile(true, serde_json::json!({})));
    if !cfg!(feature = "native-research") {
        assert!(profile.validate().is_err());
    }
    profile.writable = false;
    profile.writable_source = WritableSource::RemoteShare;
    assert!(profile.validate().is_ok());
}
