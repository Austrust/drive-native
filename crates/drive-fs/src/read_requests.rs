//! Kernel read ownership: one reply, cancellation independent of shared content.
use fuser::ReplyData;
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex, Weak,
        atomic::{AtomicBool, Ordering},
    },
};

#[derive(Clone, Default)]
pub struct ReadCancellation(Arc<AtomicBool>);
impl ReadCancellation {
    pub fn is_cancelled(&self) -> bool {
        self.0.load(Ordering::Acquire)
    }
    pub fn cancel(&self) {
        self.0.store(true, Ordering::Release);
    }
}
type Registry = Mutex<HashMap<u64, Weak<Pending>>>;
#[derive(Clone, Default)]
pub(super) struct ReadRequests(Arc<Registry>);
struct Pending {
    id: u64,
    registry: Weak<Registry>,
    cancel: ReadCancellation,
    reply: Mutex<Option<ReplyData>>,
}
impl Drop for Pending {
    fn drop(&mut self) {
        if let Some(registry) = self.registry.upgrade() {
            registry.lock().unwrap().remove(&self.id);
        }
        // A queue rejection drops the remaining ReplyData, which replies EIO.
    }
}
pub(super) struct ReadReply(Arc<Pending>);
impl ReadRequests {
    pub fn register(&self, id: u64, reply: ReplyData) -> ReadReply {
        let pending = Arc::new(Pending {
            id,
            registry: Arc::downgrade(&self.0),
            cancel: ReadCancellation::default(),
            reply: Mutex::new(Some(reply)),
        });
        self.0.lock().unwrap().insert(id, Arc::downgrade(&pending));
        ReadReply(pending)
    }
    pub fn interrupt(&self, id: u64) -> bool {
        let pending = self.0.lock().unwrap().get(&id).and_then(Weak::upgrade);
        if let Some(pending) = pending {
            let mut reply = pending.reply.lock().unwrap();
            if let Some(reply) = reply.take() {
                pending.cancel.cancel();
                reply.error(libc::EINTR);
            }
            true
        } else {
            false
        }
    }
}
impl ReadReply {
    pub fn cancellation(&self) -> ReadCancellation {
        self.0.cancel.clone()
    }
    pub fn data(self, data: &[u8]) {
        if let Some(reply) = self.0.reply.lock().unwrap().take() {
            reply.data(data)
        }
    }
    pub fn error(self, error: i32) {
        if let Some(reply) = self.0.reply.lock().unwrap().take() {
            reply.error(error)
        }
    }
}
