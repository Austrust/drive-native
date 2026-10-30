//! Task-private certificate settings, atomically persisted before network reconnection.
use serde::{Deserialize, Serialize};
use std::{
    fs,
    io::{self, Read, Write},
    os::unix::fs::{MetadataExt, OpenOptionsExt},
    path::Path,
};
#[derive(Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TrustSettings {
    pub schema: u8,
    pub task_id: String,
    pub trusted_ca_pem: Vec<String>,
}
/// Visibility and crash durability are separate after atomic rename.
#[derive(Debug, PartialEq, Eq)]
pub(crate) enum Publication {
    Durable,
    VisibleNotDurable,
}
impl TrustSettings {
    pub fn validate(&self, task: &str) -> io::Result<()> {
        if self.schema != 1 || self.task_id != task {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "trust settings task/version mismatch",
            ));
        }
        sproto::trust::parse_trusted_pem(&self.trusted_ca_pem).map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "invalid extra trust certificates",
            )
        })?;
        Ok(())
    }
    pub fn fingerprint(&self) -> String {
        drive_core::sha256(&serde_json::to_vec(self).unwrap())
    }
    pub fn read(directory: &Path, task: &str, defaults: &[String]) -> io::Result<Self> {
        let file = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(directory.join("connection-trust.json"))
        {
            Ok(f) => f,
            Err(e) if e.kind() == io::ErrorKind::NotFound => {
                let settings = Self {
                    schema: 1,
                    task_id: task.into(),
                    trusted_ca_pem: defaults.into(),
                };
                settings.validate(task)?;
                return Ok(settings);
            }
            Err(e) => return Err(e),
        };
        let meta = file.metadata()?;
        if !meta.is_file()
            || meta.uid() != unsafe { libc::geteuid() }
            || meta.mode() & 0o077 != 0
            || meta.len() > 65536
        {
            return Err(io::Error::other(
                "trust settings require a private owned regular file",
            ));
        }
        let mut bytes = Vec::new();
        file.take(65537).read_to_end(&mut bytes)?;
        if bytes.len() > 65536 {
            return Err(io::Error::other("trust settings too large"));
        }
        let settings: Self = serde_json::from_slice(&bytes)
            .map_err(|_| io::Error::other("invalid trust settings"))?;
        settings.validate(task)?;
        Ok(settings)
    }
    pub fn save(&self, directory: &Path) -> io::Result<()> {
        match self.publish(directory)? {
            Publication::Durable => Ok(()),
            Publication::VisibleNotDurable => Err(io::Error::other(
                "trust settings visible but durability unconfirmed",
            )),
        }
    }
    pub(crate) fn publish(&self, directory: &Path) -> io::Result<Publication> {
        self.publish_with(directory, |file, _after_rename| file.sync_all())
    }
    /// The injected sync boundary is used only by in-crate fault tests.
    pub(crate) fn publish_with(
        &self,
        directory: &Path,
        sync: impl Fn(&fs::File, bool) -> io::Result<()>,
    ) -> io::Result<Publication> {
        self.validate(&self.task_id)?;
        let meta = directory.symlink_metadata()?;
        if !meta.is_dir() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
            return Err(io::Error::other(
                "trust settings directory must be private and owned",
            ));
        }
        let mut temp = tempfile::NamedTempFile::new_in(directory)?;
        serde_json::to_writer(&mut temp, self).map_err(io::Error::other)?;
        temp.flush()?;
        sync(temp.as_file(), false)?;
        temp.persist(directory.join("connection-trust.json"))
            .map_err(|e| e.error)?;
        // An error here must not pretend the previous file is still visible.
        // The caller adopts the new visible settings and blocks network use.
        match fs::File::open(directory).and_then(|dir| sync(&dir, true)) {
            Ok(()) => Ok(Publication::Durable),
            Err(_) => Ok(Publication::VisibleNotDurable),
        }
    }
    /// A new process establishes durability of the visible trust file before
    /// allowing authentication. Failure leaves cached filesystem access intact.
    pub(crate) fn confirm_durable(directory: &Path) -> io::Result<()> {
        let file = match fs::OpenOptions::new()
            .read(true)
            .custom_flags(libc::O_NOFOLLOW | libc::O_NONBLOCK)
            .open(directory.join("connection-trust.json"))
        {
            Ok(file) => file,
            Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(()),
            Err(error) => return Err(error),
        };
        let meta = file.metadata()?;
        if !meta.is_file() || meta.uid() != unsafe { libc::geteuid() } || meta.mode() & 0o077 != 0 {
            return Err(io::Error::other(
                "trust durability requires private regular file",
            ));
        }
        file.sync_all()?;
        fs::File::open(directory)?.sync_all()
    }
}
