//! Durable, version-bound download retries. These records never replay writes.
use crate::{DownloadTicket, Error, Result, Store};
use rusqlite::{OptionalExtension, params};
use serde::{Deserialize, Serialize};

pub const MAX_AUTOMATIC_DOWNLOAD_ATTEMPTS: u32 = 5;

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DownloadFailureKind {
    Transport,
    SessionExpired,
    ServerRejected,
    InvalidContent,
    LocalStorage,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct DownloadFailure {
    pub schema: u8,
    pub kind: DownloadFailureKind,
    pub server_code: Option<u32>,
    pub attempts: u32,
    pub retry_at: Option<u64>,
}

impl Store {
    pub fn download_failure(&self, id: &str) -> Result<Option<DownloadFailure>> {
        let raw: Option<String> = self.db.query_row(
            "SELECT t.error FROM transfers t JOIN nodes n ON n.id=t.node AND n.remote_version=t.version
             WHERE t.node=?1 AND t.direction='download' AND t.error IS NOT NULL AND n.deleted=0
             AND (n.blob IS NULL OR n.local_version!=n.remote_version)",
            [id], |r| r.get(0),
        ).optional()?;
        raw.map(|raw| {
            let value: DownloadFailure = serde_json::from_str(&raw)?;
            if value.schema != 1 || value.attempts == 0 {
                return Err(Error::State("unsupported download failure record".into()));
            }
            Ok(value)
        })
        .transpose()
    }

    pub fn download_ready(&self, ticket: &DownloadTicket, now: u64) -> Result<bool> {
        let node = self.node(&ticket.id)?;
        if node.directory || node.version != ticket.version || self.dirty_draft(&node.id)? {
            return Ok(false);
        }
        Ok(self
            .download_failure(&ticket.id)?
            .is_none_or(|f| f.retry_at.is_some_and(|at| at <= now)))
    }

    pub fn wants_background_download(&self, ticket: &DownloadTicket) -> Result<bool> {
        let n = self.node(&ticket.id)?;
        if n.directory
            || n.version != ticket.version
            || self.dirty_draft(&n.id)?
            || crate::namespace::blocks_upload(&self.db, &n.id)?
        {
            return Ok(false);
        }
        Ok(self.open_cached(&n.id).is_ok()
            || self.is_pinned(&n.id)?
            || self.has_download_intent(ticket)?)
    }
    pub fn begin_download(&mut self, ticket: &DownloadTicket) -> Result<()> {
        let node = self.node(&ticket.id)?;
        if node.directory || node.version != ticket.version {
            return Err(Error::State("download identity/version changed".into()));
        }
        let id = self.queue_download(ticket)?;
        self.db
            .execute("UPDATE transfers SET state='inflight' WHERE id=?1", [id])?;
        Ok(())
    }

    /// Store only a typed category and numeric code, never server reason strings.
    pub fn fail_download(
        &mut self,
        ticket: &DownloadTicket,
        kind: DownloadFailureKind,
        server_code: Option<u32>,
        now: u64,
    ) -> Result<()> {
        let node = match self.node(&ticket.id) {
            Ok(n) if n.version == ticket.version && !n.directory => n,
            Ok(_) => return Ok(()),
            Err(e) if e.is_not_found() => return Ok(()),
            Err(e) => return Err(e),
        };
        let attempts = self
            .download_failure(&node.id)?
            .map_or(1, |f| f.attempts.saturating_add(1));
        let retry_at = if attempts < MAX_AUTOMATIC_DOWNLOAD_ATTEMPTS {
            Some(now.saturating_add(2u64 << (attempts - 1)))
        } else {
            None
        };
        let failure = DownloadFailure {
            schema: 1,
            kind,
            server_code,
            attempts,
            retry_at,
        };
        let id = self.queue_download(ticket)?;
        self.db.execute(
            "UPDATE transfers SET state=?2,error=?3 WHERE id=?1",
            params![
                id,
                if retry_at.is_some() {
                    "retry"
                } else {
                    "failed"
                },
                serde_json::to_string(&failure)?
            ],
        )?;
        Ok(())
    }

    pub fn finish_download(&mut self, ticket: &DownloadTicket) -> Result<()> {
        self.db.execute(
            "DELETE FROM transfers WHERE node=?1 AND version=?2 AND direction='download'",
            params![ticket.id, ticket.version],
        )?;
        Ok(())
    }

    pub fn retry_download(&mut self, id: &str) -> Result<bool> {
        let n = self.node(id)?;
        Ok(self.db.execute(
            "UPDATE transfers SET state='pending',error=NULL WHERE node=?1
            AND version=?2 AND direction='download' AND error IS NOT NULL",
            params![id, n.version],
        )? > 0)
    }

    pub fn next_download_retry(&self) -> Result<Option<u64>> {
        let mut q=self.db.prepare("SELECT t.node FROM transfers t JOIN nodes n ON n.id=t.node
            AND n.remote_version=t.version WHERE t.direction='download' AND t.state='retry'
            AND n.deleted=0 AND n.dirty_blob IS NULL AND (n.blob IS NULL OR n.local_version!=n.remote_version)")?;
        let ids = q
            .query_map([], |r| r.get::<_, String>(0))?
            .collect::<std::result::Result<Vec<_>, _>>()?;
        let mut next = None;
        for id in ids {
            if self.dirty_draft(&id)? || crate::namespace::blocks_upload(&self.db, &id)? {
                continue;
            }
            if let Some(at) = self.download_failure(&id)?.and_then(|f| f.retry_at) {
                next = Some(next.map_or(at, |old: u64| old.min(at)));
            }
        }
        Ok(next)
    }

    pub fn download_failure_count(&self) -> Result<u64> {
        let count: i64 = self.db.query_row(
            "SELECT count(*) FROM transfers t JOIN nodes n ON n.id=t.node
            AND n.remote_version=t.version WHERE t.direction='download' AND t.error IS NOT NULL
            AND n.deleted=0 AND (n.blob IS NULL OR n.local_version!=n.remote_version)",
            [],
            |r| r.get(0),
        )?;
        count
            .try_into()
            .map_err(|_| Error::State("negative download error count".into()))
    }

    pub(crate) fn has_download_intent(&self, ticket: &DownloadTicket) -> Result<bool> {
        Ok(self
            .db
            .query_row(
                "SELECT 1 FROM transfers WHERE node=?1 AND version=?2 AND direction='download'
            AND state IN ('pending','inflight','retry','failed')",
                params![ticket.id, ticket.version],
                |r| r.get::<_, u8>(0),
            )
            .optional()?
            .is_some())
    }
}
