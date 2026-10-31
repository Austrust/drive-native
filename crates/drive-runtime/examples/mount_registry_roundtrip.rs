//! Explicit ordinary-user kernel/GIO lifecycle test; no NAS connection.
use drive_core::*;
use drive_runtime::mount_registry::MountRegistration;
use std::{
    fs,
    io::{BufRead, BufReader, Write},
    os::unix::fs::{PermissionsExt, symlink},
    path::{Path, PathBuf},
    process::{Child, ChildStdin, ChildStdout, Command, Stdio},
    sync::Arc,
};
type Result<T> = std::result::Result<T, Box<dyn std::error::Error>>;
struct OwnedChild(Child);
impl Drop for OwnedChild {
    fn drop(&mut self) {
        let _ = self.0.kill();
        let _ = self.0.wait();
    }
}
struct Observer {
    process: OwnedChild,
    input: ChildStdin,
    output: BufReader<ChildStdout>,
}
impl Observer {
    fn ask(&mut self, command: &str) -> Result<serde_json::Value> {
        writeln!(self.input, "{command}")?;
        self.input.flush()?;
        let mut line = String::new();
        if self.output.read_line(&mut line)? == 0 {
            return Err("GIO observer exited".into());
        }
        Ok(serde_json::from_str(&line)?)
    }
    fn check(&mut self, first: bool, second: bool, old: bool) -> Result<()> {
        let state = self.ask("inspect")?;
        assert_eq!(state["first"], first, "{state}");
        assert_eq!(state["second"], second, "{state}");
        assert_eq!(state["old_preview_disabled"], old, "{state}");
        Ok(())
    }
}
fn mount(path: &Path, state: &Path) -> Result<fuser::BackgroundSession> {
    fs::create_dir_all(path)?;
    let mut store = Store::open(state)?;
    let root = Node {
        id: "root".into(),
        parent: None,
        name: "root".into(),
        version: "1".into(),
        directory: true,
        metadata: None,
    };
    let file = Node {
        id: "file".into(),
        parent: Some("root".into()),
        name: "opaque".into(),
        version: "1".into(),
        directory: false,
        metadata: Some(NodeMetadata {
            size: 4,
            mtime: 1700000000,
            mode: 0o644,
        }),
    };
    store.install_remote_snapshot(&[root, file], "1", "test")?;
    store.publish_download(
        &DownloadTicket {
            id: "file".into(),
            version: "1".into(),
        },
        b"body",
        &sha256(b"body"),
    )?;
    let fs = drive_fs::CacheFilesystem::new(
        Arc::new(drive_core::SharedStore::new(store)),
        "root",
        unsafe { libc::geteuid() },
        unsafe { libc::getegid() },
    )?;
    Ok(fuser::spawn_mount2(
        fs,
        path,
        &[
            fuser::MountOption::RO,
            fuser::MountOption::NoDev,
            fuser::MountOption::NoSuid,
            fuser::MountOption::DefaultPermissions,
            fuser::MountOption::FSName("drive-native".into()),
        ],
    )?)
}
fn replace_field(body: &str, key: &str, value: &str) -> String {
    body.lines()
        .map(|line| {
            if line.starts_with(&format!("{key}=")) {
                format!("{key}={value}\n")
            } else {
                format!("{line}\n")
            }
        })
        .collect()
}
fn main() -> Result<()> {
    let args: Vec<_> = std::env::args_os().collect();
    if args.get(1).is_some_and(|a| a == "--publisher") {
        let guard = MountRegistration::publish_in(
            Path::new(&args[2]),
            args[3].to_str().ok_or("task")?,
            Path::new(&args[4]),
        )?;
        println!("ready");
        std::io::stdout().flush()?;
        let mut line = String::new();
        std::io::stdin().read_line(&mut line)?;
        drop(guard);
        return Ok(());
    }
    assert_ne!(unsafe { libc::geteuid() }, 0);
    let helper = PathBuf::from(args.get(1).ok_or("observer path required")?).canonicalize()?;
    let modules = PathBuf::from(args.get(2).ok_or("production module required")?).canonicalize()?;
    assert_eq!(
        fs::read_to_string(modules.join("scope-mode.txt"))?.trim(),
        "registry"
    );
    let output = PathBuf::from(args.get(3).ok_or("report path required")?);
    fs::write(&output, b"{\"passed\":false,\"stage\":\"starting\"}\n")?;
    let tmp = tempfile::tempdir()?;
    let base = tmp.path();
    let runtime = base.join("runtime");
    fs::create_dir(&runtime)?;
    fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700))?;
    let first = base.join("挂载 A\\B");
    let second = base.join("second");
    let first_mount = mount(&first, &base.join("state1"))?;
    let second_mount = mount(&second, &base.join("state2"))?;
    let task1 = uuid::Uuid::new_v4().to_string();
    let task2 = uuid::Uuid::new_v4().to_string();
    let registration1 = MountRegistration::publish_in(&runtime, &task1, &first)?;
    let registration2 = MountRegistration::publish_in(&runtime, &task2, &second)?;
    assert!(MountRegistration::publish_in(&runtime, &task1, &first).is_err());
    let path = registration1.path().to_owned();
    let original = fs::read_to_string(&path)?;
    let mut process = Command::new("/usr/bin/python3")
        .arg(&helper)
        .arg(&first)
        .arg(&second)
        .env_remove("GIO_USE_VFS")
        .env("GIO_EXTRA_MODULES", &modules)
        .env("XDG_RUNTIME_DIR", &runtime)
        .env_remove("DRIVE_NATIVE_METADATA_ROOT")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let input = process.stdin.take().unwrap();
    let output_pipe = BufReader::new(process.stdout.take().unwrap());
    let mut observer = Observer {
        process: OwnedChild(process),
        input,
        output: output_pipe,
    };
    observer.check(true, true, true)?;
    assert_eq!(
        observer.ask("scope-environment")?["ordinary_wrapped"],
        false
    );
    let benchmark = observer.ask("benchmark")?;
    println!("benchmark={benchmark}");
    for (key, value) in [
        ("schema", "2"),
        ("uid", "0"),
        ("pid_start", "0"),
        ("mount_id", "0"),
        ("mount_unique_id", "0"),
    ] {
        println!("reject_{key}");
        fs::write(&path, replace_field(&original, key, value))?;
        observer.check(false, true, false)?;
        fs::write(&path, &original)?;
        observer.check(true, true, true)?;
    }
    println!("reject_public_mode");
    fs::set_permissions(&path, fs::Permissions::from_mode(0o644))?;
    observer.check(false, true, false)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    println!("reject_symlink");
    let backup = base.join("outside-record");
    fs::write(&backup, &original)?;
    fs::set_permissions(&backup, fs::Permissions::from_mode(0o600))?;
    fs::remove_file(&path)?;
    symlink(&backup, &path)?;
    observer.check(false, true, false)?;
    fs::remove_file(&path)?;
    fs::write(&path, &original)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    println!("reject_fifo_and_hardlink");
    let hardlink = base.join("record-hardlink");
    fs::hard_link(&path, &hardlink)?;
    observer.check(false, true, false)?;
    fs::remove_file(hardlink)?;
    fs::remove_file(&path)?;
    use std::os::unix::ffi::OsStrExt;
    let cpath = std::ffi::CString::new(path.as_os_str().as_bytes())?;
    assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
    observer.check(false, true, false)?;
    assert!(MountRegistration::publish_in(&runtime, &task1, &first).is_err());
    fs::remove_file(&path)?;
    fs::write(&path, &original)?;
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600))?;
    println!("reject_directory_mode");
    let app = runtime.join("drive-native");
    fs::set_permissions(&app, fs::Permissions::from_mode(0o755))?;
    observer.check(false, false, false)?;
    fs::set_permissions(&app, fs::Permissions::from_mode(0o700))?;
    observer.check(true, true, true)?;
    println!("unmount_first");
    first_mount.join();
    fs::write(first.join("opaque"), b"ordinary")?;
    observer.check(false, true, false)?; // Old GFile also stops suppressing normal metadata.
    fs::remove_file(first.join("opaque"))?;
    println!("remount_first");
    let replacement = mount(&first, &base.join("state3"))?;
    observer.check(false, true, false)?; // Same pathname, new kernel mount ID.
    let replacement_registration = MountRegistration::publish_in(&runtime, &task1, &first)?;
    observer.check(true, true, true)?;
    drop(registration1);
    assert!(path.exists());
    observer.check(true, true, true)?;
    drop(replacement_registration);
    observer.check(false, true, false)?;
    replacement.join();
    fs::write(first.join("opaque"), b"ordinary")?;
    drop(registration2);
    observer.check(false, false, false)?;
    println!("publisher_crash");
    let task3 = uuid::Uuid::new_v4().to_string();
    let mut publisher = Command::new(std::env::current_exe()?)
        .arg("--publisher")
        .arg(&runtime)
        .arg(&task3)
        .arg(&second)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()?;
    let mut ready = String::new();
    BufReader::new(publisher.stdout.take().unwrap()).read_line(&mut ready)?;
    let mut publisher = OwnedChild(publisher);
    assert_eq!(ready.trim(), "ready");
    observer.check(false, true, false)?;
    publisher.0.kill()?;
    publisher.0.wait()?;
    observer.check(false, false, false)?; // Crashed publisher, mount still alive.
    let stale = runtime
        .join("drive-native/mounts")
        .join(format!("{task3}.mount"));
    assert!(stale.exists());
    fs::remove_file(stale)?;
    writeln!(observer.input, "quit")?;
    observer.input.flush()?;
    assert!(observer.process.0.wait()?.success());
    second_mount.join();
    let report = serde_json::json!({"passed":true,"ordinary_user":true,"real_kernel_fuse":true,
        "two_mounts_recognized_without_scope_environment":true,"unicode_space_backslash_path":true,
        "invalid_schema_uid_start_time_mount_id_rejected":true,"insecure_modes_and_symlinks_rejected":true,
        "retained_gfile_revalidates_scope":true,"unmount_and_same_path_remount_checked":true,
        "old_guard_preserves_new_generation":true,"crashed_publisher_record_rejected":true,
        "metadata_query_benchmark":benchmark,"nas_contacted":false,
        "automatic_gio_selection":true,"production_ignores_scope_environment":true,"fifo_and_hardlink_records_rejected":true});
    fs::write(output, serde_json::to_vec_pretty(&report)?)?;
    println!(
        "PASS real mount registry lifecycle, stale identity rejection and persistent GIO observer"
    );
    Ok(())
}
