//! Native event boundary and authenticated author fingerprint; no raw credentials.
use crate::{Error, Result};
use serde::{Deserialize, Serialize};
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct CommitOrigin {
    pub cursor: String,
    pub committer: String,
    pub restore: String,
}
impl CommitOrigin {
    pub fn validate(&self) -> Result<()> {
        if self
            .cursor
            .strip_prefix("native-v1:")
            .and_then(|s| s.parse::<u64>().ok())
            .is_none()
            || self.committer.len() != 64
            || !self
                .committer
                .bytes()
                .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c))
            || self.restore.is_empty()
            || self.restore.len() > 8192
        {
            return Err(Error::State("invalid commit provenance".into()));
        }
        Ok(())
    }
}
