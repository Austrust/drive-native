//! Per-user, versioned records of live FUSE mounts for desktop metadata adapters.
//! Records contain no account credentials or remote paths. Readers must also
//! validate the kernel mount ID and the publisher's process start time.
use std::{
    collections::HashMap,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::unix::{
        ffi::OsStrExt,
        fs::{DirBuilderExt, MetadataExt, OpenOptionsExt},
    },
    path::{Path, PathBuf},
};

pub struct MountRegistration {
    path: PathBuf,
    generation: String,
}
impl MountRegistration {
    pub fn publish(task: &str, mount: &Path) -> io::Result<Self> {
        let runtime = std::env::var_os("XDG_RUNTIME_DIR")
            .map(PathBuf::from)
            .ok_or_else(|| io::Error::other("private user runtime directory unavailable"))?;
        Self::publish_in(&runtime, task, mount)
    }
    pub fn publish_in(runtime: &Path, task: &str, mount: &Path) -> io::Result<Self> {
        let task =
            uuid::Uuid::parse_str(task).map_err(|_| io::Error::other("invalid task identity"))?;
        let uid = unsafe { libc::geteuid() };
        if uid == 0 {
            return Err(io::Error::other(
                "mount registrations require ordinary user",
            ));
        }
        let mount_id = mounted_id(&fs::read("/proc/self/mountinfo")?, mount)?;
        let mount_unique_id = unique_mount_id(mount)?;
        let pid = std::process::id();
        let pid_start = process_start(pid)?;
        let directory = registry_directory(runtime)?;
        let path = directory.join(format!("{task}.mount"));
        match read_record(&path) {
            Ok(record) => {
                let fields = fields(&record);
                if let (Some(pid), Some(start), Some(id), Some(hex)) = (
                    number(&fields, "pid"),
                    number(&fields, "pid_start"),
                    number(&fields, "mount_unique_id"),
                    fields.get("path_hex"),
                ) {
                    if pid <= u32::MAX as u64 && process_start(pid as u32).ok() == Some(start) {
                        if let Some(bytes) = unhex(hex) {
                            let old = Path::new(std::ffi::OsStr::from_bytes(&bytes));
                            if unique_mount_id(old).ok() == Some(id) {
                                return Err(io::Error::new(
                                    io::ErrorKind::AlreadyExists,
                                    "task mount is still active",
                                ));
                            }
                        }
                    }
                }
            }
            Err(e) if e.kind() == io::ErrorKind::NotFound => {}
            Err(e) => return Err(e),
        }
        let generation = uuid::Uuid::new_v4().to_string();
        let path_hex: String = mount
            .as_os_str()
            .as_bytes()
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect();
        let body = format!(
            "[Mount]\nschema=1\nuid={uid}\npid={pid}\npid_start={pid_start}\nmount_id={mount_id}\nmount_unique_id={mount_unique_id}\npath_hex={path_hex}\ngeneration={generation}\n"
        );
        if body.len() > 8192 {
            return Err(io::Error::other("mount record exceeds limit"));
        }
        let mut temp = tempfile::NamedTempFile::new_in(&directory)?;
        temp.write_all(body.as_bytes())?;
        temp.as_file().sync_all()?;
        temp.persist(&path).map_err(|e| e.error)?;
        File::open(&directory)?.sync_all()?;
        Ok(Self { path, generation })
    }
    pub fn path(&self) -> &Path {
        &self.path
    }
}
impl Drop for MountRegistration {
    fn drop(&mut self) {
        if let Ok(record) = read_record(&self.path) {
            if fields(&record).get("generation").copied() == Some(self.generation.as_str()) {
                let _ = fs::remove_file(&self.path);
            }
        }
    }
}
fn private_directory(path: &Path) -> io::Result<()> {
    let meta = fs::symlink_metadata(path)?;
    if !path.is_absolute()
        || !meta.is_dir()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
        || path.canonicalize()? != path
    {
        return Err(io::Error::other(
            "runtime directory must be canonical, owned and private",
        ));
    }
    Ok(())
}
fn registry_directory(runtime: &Path) -> io::Result<PathBuf> {
    private_directory(runtime)?;
    let mut path = runtime.to_owned();
    for component in ["drive-native", "mounts"] {
        path.push(component);
        match fs::DirBuilder::new().mode(0o700).create(&path) {
            Ok(()) => {}
            Err(e) if e.kind() == io::ErrorKind::AlreadyExists => {}
            Err(e) => return Err(e),
        }
        private_directory(&path)?;
    }
    Ok(path)
}
fn read_record(path: &Path) -> io::Result<String> {
    let file = OpenOptions::new()
        .read(true)
        .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
        .open(path)?;
    let meta = file.metadata()?;
    if !meta.is_file()
        || meta.uid() != unsafe { libc::geteuid() }
        || meta.mode() & 0o077 != 0
        || meta.len() > 8192
        || meta.nlink() != 1
    {
        return Err(io::Error::other(
            "invalid mount record ownership, mode or size",
        ));
    }
    let mut text = String::new();
    file.take(8193).read_to_string(&mut text)?;
    if text.len() > 8192 {
        return Err(io::Error::other("mount record exceeds limit"));
    }
    Ok(text)
}
fn fields(text: &str) -> HashMap<&str, &str> {
    text.lines()
        .filter_map(|line| line.split_once('='))
        .collect()
}
fn number(fields: &HashMap<&str, &str>, key: &str) -> Option<u64> {
    fields.get(key)?.parse().ok()
}
fn unhex(text: &str) -> Option<Vec<u8>> {
    if text.len() % 2 != 0 {
        return None;
    }
    text.as_bytes()
        .chunks_exact(2)
        .map(|pair| {
            let a = (pair[0] as char).to_digit(16)?;
            let b = (pair[1] as char).to_digit(16)?;
            let byte = (a * 16 + b) as u8;
            if byte == 0 { None } else { Some(byte) }
        })
        .collect()
}
// Linux 6.8+ UAPI. libc exposes statx but not yet this mask constant.
fn unique_mount_id(path: &Path) -> io::Result<u64> {
    const STATX_MNT_ID_UNIQUE: u32 = 0x00004000;
    let path = std::ffi::CString::new(path.as_os_str().as_bytes())
        .map_err(|_| io::Error::other("invalid mount path"))?;
    let mut value: libc::statx = unsafe { std::mem::zeroed() };
    let result = unsafe {
        libc::statx(
            libc::AT_FDCWD,
            path.as_ptr(),
            libc::AT_NO_AUTOMOUNT | libc::AT_SYMLINK_NOFOLLOW | libc::AT_STATX_DONT_SYNC,
            STATX_MNT_ID_UNIQUE,
            &mut value,
        )
    };
    if result != 0 {
        return Err(io::Error::last_os_error());
    }
    if value.stx_mask & STATX_MNT_ID_UNIQUE == 0 {
        return Err(io::Error::new(
            io::ErrorKind::Unsupported,
            "unique mount identity requires Linux 6.8 or newer",
        ));
    }
    Ok(value.stx_mnt_id)
}
fn process_start(pid: u32) -> io::Result<u64> {
    let data = fs::read(format!("/proc/{pid}/stat"))?;
    process_start_from(&data).ok_or_else(|| io::Error::other("invalid process identity"))
}
fn process_start_from(data: &[u8]) -> Option<u64> {
    let end = data.iter().rposition(|b| *b == b')')?;
    std::str::from_utf8(data.get(end + 1..)?)
        .ok()?
        .split_whitespace()
        .nth(19)?
        .parse()
        .ok()
}
fn unescape(data: &[u8]) -> Option<Vec<u8>> {
    let mut out = Vec::new();
    let mut i = 0;
    while i < data.len() {
        if data[i] == b'\\' {
            let digits = data.get(i + 1..i + 4)?;
            if !digits.iter().all(|d| (b'0'..=b'7').contains(d)) {
                return None;
            }
            let n = ((digits[0] - b'0') as u16) * 64
                + ((digits[1] - b'0') as u16) * 8
                + (digits[2] - b'0') as u16;
            if n == 0 || n > 255 {
                return None;
            }
            out.push(n as u8);
            i += 4;
        } else {
            out.push(data[i]);
            i += 1;
        }
    }
    Some(out)
}
fn mounted_id(data: &[u8], path: &Path) -> io::Result<u64> {
    let mut found = None;
    for line in data.split(|b| *b == b'\n') {
        let Some(separator) = line.windows(3).position(|s| s == b" - ") else {
            continue;
        };
        let head: Vec<_> = line[..separator].split(|b| *b == b' ').collect();
        let tail: Vec<_> = line[separator + 3..].split(|b| *b == b' ').collect();
        if head.len() < 6
            || tail.len() < 2
            || unescape(head[4]).as_deref() != Some(path.as_os_str().as_bytes())
        {
            continue;
        }
        if found.is_some()
            || !matches!(tail[0], b"fuse" | b"fuse.drive-native")
            || tail[1] != b"drive-native"
        {
            return Err(io::Error::other("mount is overlaid or not Drive Native"));
        }
        found = std::str::from_utf8(head[0])
            .ok()
            .and_then(|s| s.parse().ok());
    }
    found.ok_or_else(|| io::Error::other("Drive Native mount is absent"))
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::os::unix::fs::{PermissionsExt, symlink};
    #[test]
    fn mount_identity_handles_escaped_names_and_rejects_overmounts() {
        let mount = b"45 1 0:50 / /tmp/a\\040b\\134c rw - fuse drive-native rw\n";
        let path = Path::new("/tmp/a b\\c");
        assert_eq!(mounted_id(mount, path).unwrap(), 45);
        let mut overlay = mount.to_vec();
        overlay.extend_from_slice(b"46 1 0:51 / /tmp/a\\040b\\134c rw - tmpfs tmpfs rw\n");
        assert!(mounted_id(&overlay, path).is_err());
        assert!(
            mounted_id(
                b"45 1 0:50 / /tmp/ordinary rw - fuse other rw\n",
                Path::new("/tmp/ordinary")
            )
            .is_err()
        );
        assert!(unescape(b"bad\\000").is_none());
    }
    #[test]
    fn process_identity_handles_parentheses_and_uses_start_time() {
        let data =
            b"123 (name with ) bracket) S 1 2 3 4 5 6 7 8 9 10 11 12 13 14 15 16 17 18 98765 20";
        assert_eq!(process_start_from(data), Some(98765));
        assert!(process_start(std::process::id()).unwrap() > 0);
    }
    #[test]
    fn registry_rejects_shared_or_symlinked_runtime_and_unrelated_files() {
        let base = tempfile::tempdir().unwrap();
        let runtime = base.path().join("runtime");
        fs::create_dir(&runtime).unwrap();
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700)).unwrap();
        let alias = base.path().join("alias");
        symlink(&runtime, &alias).unwrap();
        assert!(registry_directory(&alias).is_err());
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o755)).unwrap();
        assert!(registry_directory(&runtime).is_err());
        fs::set_permissions(&runtime, fs::Permissions::from_mode(0o700)).unwrap();
        let directory = registry_directory(&runtime).unwrap();
        let file = directory.join("test.mount");
        fs::write(&file, b"[Mount]\nschema=1\n").unwrap();
        fs::set_permissions(&file, fs::Permissions::from_mode(0o644)).unwrap();
        assert!(read_record(&file).is_err());
        fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).unwrap();
        assert!(read_record(&file).is_ok());
        let link = directory.join("link.mount");
        symlink(&file, &link).unwrap();
        assert!(read_record(&link).is_err());
        let fifo = directory.join("fifo.mount");
        let cpath = std::ffi::CString::new(fifo.as_os_str().as_bytes()).unwrap();
        assert_eq!(unsafe { libc::mkfifo(cpath.as_ptr(), 0o600) }, 0);
        assert!(read_record(&fifo).is_err());
        let hard = directory.join("hard.mount");
        fs::hard_link(&file, &hard).unwrap();
        assert!(read_record(&hard).is_err());
    }
    #[test]
    fn teardown_never_removes_another_generation() {
        let base = tempfile::tempdir().unwrap();
        let path = base.path().join("test.mount");
        fs::write(&path, "[Mount]\ngeneration=new\n").unwrap();
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
        drop(MountRegistration {
            path: path.clone(),
            generation: "old".into(),
        });
        assert!(path.exists());
        drop(MountRegistration {
            path: path.clone(),
            generation: "new".into(),
        });
        assert!(!path.exists());
    }
}
