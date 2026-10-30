//! Persisted payload bandwidth policy; no credentials or remote mutations.
use crate::{Error, Result, Store};
use rusqlite::OptionalExtension;
use serde::{Deserialize, Serialize};
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TransferLimits {
    pub download_bps: u64,
    pub upload_bps: u64,
}
impl TransferLimits {
    pub fn validate(&self) -> Result<()> {
        for rate in [self.download_bps, self.upload_bps] {
            if rate > 1024 * 1024 * 1024 || rate % 1024 != 0 {
                return Err(Error::State(
                    "bandwidth must be whole KiB/s, at most 1 GiB/s".into(),
                ));
            }
        }
        Ok(())
    }
    pub fn fingerprint(&self) -> String {
        crate::sha256(&serde_json::to_vec(self).unwrap())
    }
}
impl Store {
    pub fn transfer_limits(&self) -> Result<TransferLimits> {
        let value: Option<String> = self
            .db
            .query_row(
                "SELECT value FROM metadata WHERE key='transfer_limits_v1'",
                [],
                |r| r.get(0),
            )
            .optional()?;
        let limits = value
            .map(|v| serde_json::from_str::<TransferLimits>(&v))
            .transpose()?
            .unwrap_or_default();
        limits.validate()?;
        Ok(limits)
    }
    pub fn set_transfer_limits(&mut self, limits: TransferLimits) -> Result<()> {
        limits.validate()?;
        self.db.execute("INSERT INTO metadata(key,value) VALUES('transfer_limits_v1',?1) ON CONFLICT(key) DO UPDATE SET value=excluded.value",[serde_json::to_string(&limits)?])?;
        Ok(())
    }
    pub fn transfer_path(&self, id: &str) -> Result<String> {
        crate::namespace::node_path(&self.db, id)
    }
}
#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn limits_validate_and_persist_without_creating_remote_operations() {
        let d = tempfile::tempdir().unwrap();
        let mut store = Store::open(d.path()).unwrap();
        assert_eq!(store.transfer_limits().unwrap(), TransferLimits::default());
        let limits = TransferLimits {
            download_bps: 65536,
            upload_bps: 131072,
        };
        store.set_transfer_limits(limits).unwrap();
        assert!(
            store
                .set_transfer_limits(TransferLimits {
                    download_bps: 1,
                    upload_bps: 0
                })
                .is_err()
        );
        assert_eq!(store.transfer_limits().unwrap(), limits);
        assert_eq!(store.pending_operations().unwrap(), 0);
        drop(store);
        assert_eq!(
            Store::open(d.path()).unwrap().transfer_limits().unwrap(),
            limits
        );
    }
}
