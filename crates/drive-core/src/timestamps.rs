//! Local access times and explicit, journaled native upload modification times.
//! Native mtime is u32 seconds. Fractional input is rounded down to the declared
//! one-second filesystem resolution; access time remains local nanoseconds.
use crate::{Error, Node, Result, Store};
use rusqlite::{OptionalExtension, params};
use std::time::{SystemTime, UNIX_EPOCH};

fn os(code: i32) -> Error {
    Error::Io(std::io::Error::from_raw_os_error(code))
}
pub fn native_mtime(time: SystemTime) -> Result<u32> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| os(libc::EINVAL))?;
    u32::try_from(duration.as_secs()).map_err(|_| os(libc::EOVERFLOW))
}
pub fn local_time(time: SystemTime) -> Result<(u64, u32)> {
    let duration = time
        .duration_since(UNIX_EPOCH)
        .map_err(|_| os(libc::EINVAL))?;
    if duration.as_secs() > i64::MAX as u64 {
        return Err(os(libc::EOVERFLOW));
    }
    Ok((duration.as_secs(), duration.subsec_nanos()))
}
impl Store {
    pub fn local_atime(&self, id: &str, fallback: u64) -> Result<(u64, u32)> {
        let row: Option<(i64, i64)> = self
            .db
            .query_row(
                "SELECT seconds,nanoseconds FROM local_atimes WHERE node=?1",
                [id],
                |r| Ok((r.get(0)?, r.get(1)?)),
            )
            .optional()?;
        match row {
            None => Ok((fallback, 0)),
            Some((seconds, nanoseconds)) => {
                let seconds = u64::try_from(seconds)
                    .map_err(|_| Error::State("invalid local access time".into()))?;
                let nanoseconds = u32::try_from(nanoseconds)
                    .ok()
                    .filter(|n| *n < 1_000_000_000)
                    .ok_or_else(|| Error::State("invalid local access time".into()))?;
                Ok((seconds, nanoseconds))
            }
        }
    }
    pub fn set_local_atime(&mut self, id: &str, seconds: u64, nanoseconds: u32) -> Result<()> {
        self.node_for_handle(id)?;
        if seconds > i64::MAX as u64 || nanoseconds >= 1_000_000_000 {
            return Err(os(libc::EINVAL));
        }
        self.db.execute("INSERT INTO local_atimes(node,seconds,nanoseconds) VALUES(?1,?2,?3) ON CONFLICT(node) DO UPDATE SET seconds=excluded.seconds,nanoseconds=excluded.nanoseconds",params![id,seconds as i64,nanoseconds])?;
        Ok(())
    }
    /// None identifies legacy jobs, whose request never promised remote mtime.
    pub fn upload_mtime(&self, operation: &str) -> Result<Option<u32>> {
        Ok(self
            .db
            .query_row(
                "SELECT mtime FROM upload_times WHERE operation=?1",
                [operation],
                |r| r.get(0),
            )
            .optional()?)
    }
    pub fn validate_upload_mtime(&self, operation: &str, confirmed: &Node) -> Result<()> {
        if let Some(expected) = self.upload_mtime(operation)? {
            if confirmed
                .metadata
                .as_ref()
                .is_none_or(|m| m.mtime != u64::from(expected))
            {
                return Err(Error::Integrity);
            }
        }
        Ok(())
    }
}
