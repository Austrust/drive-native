//! Volatile IPC progress. Never substitutes for durable server commit state.
use crate::wire::{Command, TransferProgress};
use serde::Serialize;
use std::{
    collections::HashMap,
    io,
    sync::{
        Mutex,
        atomic::{AtomicU64, Ordering},
    },
    time::{Duration, Instant},
};
struct Active {
    node: String,
    progress: Option<TransferProgress>,
    started: Instant,
    last: Instant,
}
#[derive(Serialize)]
pub struct TransferEntry {
    pub request: u64,
    pub node: String,
    pub progress: Option<TransferProgress>,
    pub elapsed_ms: u64,
}
#[derive(Default)]
pub struct TransferRegistry {
    items: Mutex<HashMap<u64, Active>>,
    revision: AtomicU64,
}
impl TransferRegistry {
    pub fn register(&self, id: u64, body: &Command) {
        let node = match body {
            Command::Download { node } | Command::Upload { node, .. } => Some(&node.id),
            Command::Replace { job, .. } => Some(&job.target.id),
            _ => None,
        };
        if let Some(node) = node {
            let now = Instant::now();
            self.items.lock().unwrap().insert(
                id,
                Active {
                    node: node.clone(),
                    progress: None,
                    started: now,
                    last: now,
                },
            );
            self.revision.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn update(&self, id: u64, p: TransferProgress) -> io::Result<()> {
        let mut items = self.items.lock().unwrap();
        let Some(active) = items.get_mut(&id) else {
            return Ok(());
        };
        if p.epoch == 0
            || p.bytes > p.total
            || !matches!(p.direction.as_str(), "download" | "upload")
            || !matches!(
                p.phase.as_str(),
                "queued" | "preparing" | "transferring" | "verifying" | "awaiting_confirmation"
            )
        {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "invalid transfer progress",
            ));
        }
        if let Some(old) = &active.progress {
            if p.epoch < old.epoch {
                return Ok(());
            }
            if p.epoch == old.epoch {
                if p.total != old.total || p.direction != old.direction || p.bytes < old.bytes {
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidData,
                        "nonmonotonic transfer progress",
                    ));
                }
                if p.bytes == old.bytes && p.phase == old.phase {
                    return Ok(());
                }
            } else {
                active.started = Instant::now();
            }
        }
        active.last = Instant::now();
        active.progress = Some(p);
        self.revision.fetch_add(1, Ordering::Relaxed);
        Ok(())
    }
    pub fn remove(&self, id: u64) {
        if self.items.lock().unwrap().remove(&id).is_some() {
            self.revision.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn clear(&self) {
        let mut items = self.items.lock().unwrap();
        if !items.is_empty() {
            items.clear();
            self.revision.fetch_add(1, Ordering::Relaxed);
        }
    }
    pub fn remaining(&self, id: u64, idle: Duration) -> Option<Duration> {
        self.items
            .lock()
            .unwrap()
            .get(&id)
            .map(|active| idle.saturating_sub(active.last.elapsed()))
    }
    pub fn revision(&self) -> u64 {
        self.revision.load(Ordering::Relaxed)
    }
    pub fn entries(&self) -> Vec<TransferEntry> {
        let items = self.items.lock().unwrap();
        let mut values: Vec<_> = items
            .iter()
            .map(|(id, a)| TransferEntry {
                request: *id,
                node: a.node.clone(),
                progress: a.progress.clone(),
                elapsed_ms: a.started.elapsed().as_millis().min(u64::MAX as u128) as u64,
            })
            .collect();
        values.sort_by_key(|e| e.request);
        values
    }
}
