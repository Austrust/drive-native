use crate::{transport, wire::*};
use std::{
    collections::HashMap,
    io,
    net::Shutdown,
    os::{fd::OwnedFd, unix::net::UnixStream},
    sync::{
        Arc, Mutex,
        atomic::{AtomicBool, AtomicU64, Ordering},
        mpsc::{self, SyncSender},
    },
    time::Duration,
};
pub struct Reply {
    pub body: Response,
    pub descriptor: Option<OwnedFd>,
}
struct PendingRequest {
    reply: mpsc::Sender<io::Result<Reply>>,
    wait: Option<Arc<Mutex<crate::wait_activity::WaitClock>>>,
}
type Pending = Arc<Mutex<HashMap<u64, PendingRequest>>>;
pub struct Rpc {
    sender: SyncSender<(Envelope<Command>, Option<OwnedFd>)>,
    pending: Pending,
    pub transfers: Arc<crate::transfers::TransferRegistry>,
    next: AtomicU64,
    connected: Arc<AtomicBool>,
    shutdown: UnixStream,
}
fn disconnected() -> io::Error {
    io::Error::new(io::ErrorKind::NotConnected, "native worker disconnected")
}
fn fail_all(pending: &Pending) {
    for (_, tx) in pending.lock().unwrap().drain() {
        let _ = tx.reply.send(Err(disconnected()));
    }
}
impl Rpc {
    pub fn new(socket: UnixStream) -> io::Result<Self> {
        transport::check_peer(&socket)?;
        let shutdown = socket.try_clone()?;
        let mut reader = socket.try_clone()?;
        let mut writer = socket;
        let pending: Pending = Arc::new(Mutex::new(HashMap::new()));
        let transfers = Arc::new(crate::transfers::TransferRegistry::default());
        let connected = Arc::new(AtomicBool::new(true));
        let online = connected.clone();
        let p = pending.clone();
        let t = transfers.clone();
        std::thread::spawn(move || {
            loop {
                let response = transport::receive::<Envelope<Response>>(&mut reader);
                match response {
                    Ok((env, fd)) if env.version == VERSION => {
                        let expect_fd = matches!(
                            &env.body,
                            Response::Content { .. } | Response::RecoverySnapshot { .. }
                        );
                        if expect_fd != fd.is_some() {
                            break;
                        }
                        if let Response::TransferProgress(progress) = env.body {
                            if t.update(env.id, progress).is_err() {
                                break;
                            }
                            continue;
                        }
                        if let Response::WaitProgress {
                            bytes,
                            received_at_ns,
                        } = env.body
                        {
                            let requests = p.lock().unwrap();
                            if let Some(request) = requests.get(&env.id) {
                                let Some(clock) = &request.wait else { break };
                                if clock.lock().unwrap().update(bytes, received_at_ns).is_err() {
                                    break;
                                }
                            }
                            // Late progress after timeout/cancellation cannot revive a request.
                            continue;
                        }
                        if let Some(tx) = p.lock().unwrap().remove(&env.id) {
                            let _ = tx.reply.send(Ok(Reply {
                                body: env.body,
                                descriptor: fd,
                            }));
                        }
                    }
                    _ => break,
                }
            }
            online.store(false, Ordering::Release);
            let _ = reader.shutdown(Shutdown::Both);
            fail_all(&p);
            t.clear();
        });
        let (sender, rx) = mpsc::sync_channel::<(Envelope<Command>, Option<OwnedFd>)>(128);
        let online = connected.clone();
        let p = pending.clone();
        let t = transfers.clone();
        std::thread::spawn(move || {
            for (env, fd) in rx {
                if transport::send(&mut writer, &env, fd.as_ref()).is_err() {
                    break;
                }
            }
            online.store(false, Ordering::Release);
            let _ = writer.shutdown(Shutdown::Both);
            fail_all(&p);
            t.clear();
        });
        Ok(Self {
            sender,
            pending,
            transfers,
            next: AtomicU64::new(1),
            connected,
            shutdown,
        })
    }
    pub fn call(&self, body: Command, timeout: Duration) -> io::Result<Reply> {
        self.call_with_descriptor(body, timeout, None)
    }
    pub fn call_with_descriptor(
        &self,
        body: Command,
        timeout: Duration,
        descriptor: Option<OwnedFd>,
    ) -> io::Result<Reply> {
        self.call_controlled(body, timeout, descriptor, None)
    }
    pub fn call_cancellable(
        &self,
        body: Command,
        timeout: Duration,
        cancelled: &dyn Fn() -> bool,
    ) -> io::Result<Reply> {
        self.call_controlled(body, timeout, None, Some(cancelled))
    }
    fn call_controlled(
        &self,
        body: Command,
        timeout: Duration,
        descriptor: Option<OwnedFd>,
        cancelled: Option<&dyn Fn() -> bool>,
    ) -> io::Result<Reply> {
        if cancelled.is_some_and(|f| f()) {
            return Err(io::ErrorKind::Interrupted.into());
        }
        if !self.is_connected() {
            return Err(disconnected());
        }
        let id = self.next.fetch_add(1, Ordering::Relaxed);
        if id == u64::MAX {
            return Err(io::Error::other("IPC request ID exhausted"));
        }
        let wait_clock = if matches!(&body, Command::Wait { .. }) {
            Some(Arc::new(
                Mutex::new(crate::wait_activity::WaitClock::new()?),
            ))
        } else {
            None
        };
        self.transfers.register(id, &body);
        let (tx, rx) = mpsc::channel();
        self.pending.lock().unwrap().insert(
            id,
            PendingRequest {
                reply: tx,
                wait: wait_clock.clone(),
            },
        );
        if self
            .sender
            .try_send((
                Envelope {
                    version: VERSION,
                    id,
                    body,
                },
                descriptor,
            ))
            .is_err()
        {
            self.pending.lock().unwrap().remove(&id);
            self.transfers.remove(id);
            return Err(disconnected());
        }
        let started = std::time::Instant::now();
        let remaining_budget = || {
            wait_clock
                .as_ref()
                .map(|clock| clock.lock().unwrap().remaining(timeout))
                .or_else(|| self.transfers.remaining(id, timeout))
                .unwrap_or_else(|| timeout.saturating_sub(started.elapsed()))
        };
        loop {
            if cancelled.is_some_and(|f| f()) {
                self.pending.lock().unwrap().remove(&id);
                self.transfers.remove(id);
                if self
                    .sender
                    .try_send((
                        Envelope {
                            version: VERSION,
                            id: 0,
                            body: Command::Cancel { request: id },
                        },
                        None,
                    ))
                    .is_err()
                {
                    self.close();
                }
                return Err(io::ErrorKind::Interrupted.into());
            }
            let remaining = remaining_budget();
            let wait = if cancelled.is_some() {
                remaining.min(Duration::from_millis(25))
            } else {
                remaining
            };
            match rx.recv_timeout(wait) {
                Ok(result) => {
                    self.transfers.remove(id);
                    return result;
                }
                Err(mpsc::RecvTimeoutError::Timeout) if wait < remaining => continue,
                Err(mpsc::RecvTimeoutError::Timeout) if !remaining_budget().is_zero() => {
                    continue;
                }
                Err(error) => {
                    self.pending.lock().unwrap().remove(&id);
                    self.transfers.remove(id);
                    if self
                        .sender
                        .try_send((
                            Envelope {
                                version: VERSION,
                                id: 0,
                                body: Command::Cancel { request: id },
                            },
                            None,
                        ))
                        .is_err()
                    {
                        self.close();
                    }
                    return Err(if matches!(error, mpsc::RecvTimeoutError::Disconnected) {
                        disconnected()
                    } else {
                        io::Error::new(io::ErrorKind::TimedOut, "native request stalled")
                    });
                }
            }
        }
    }
    pub fn is_connected(&self) -> bool {
        self.connected.load(Ordering::Acquire)
    }
    pub fn close(&self) {
        self.connected.store(false, Ordering::Release);
        let _ = self.shutdown.shutdown(Shutdown::Both);
        fail_all(&self.pending);
        self.transfers.clear();
    }
}
impl Drop for Rpc {
    fn drop(&mut self) {
        self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn out_of_order_responses_are_matched_without_head_of_line_blocking() {
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Arc::new(Rpc::new(a).unwrap());
        let server = std::thread::spawn(move || {
            let (first, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            let (second, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            for request in [second, first] {
                let Command::Pull { cursor } = request.body else {
                    panic!("wrong command")
                };
                transport::send(
                    &mut b,
                    &Envelope {
                        version: VERSION,
                        id: request.id,
                        body: Response::Connected { restore: cursor },
                    },
                    None,
                )
                .unwrap();
            }
        });
        let clients = ["slow", "fast"].map(|cursor| {
            let rpc = rpc.clone();
            std::thread::spawn(move || {
                let reply = rpc
                    .call(
                        Command::Pull {
                            cursor: cursor.into(),
                        },
                        Duration::from_secs(2),
                    )
                    .unwrap();
                let Response::Connected { restore } = reply.body else {
                    panic!("wrong response")
                };
                assert_eq!(restore, cursor)
            })
        });
        for c in clients {
            c.join().unwrap()
        }
        server.join().unwrap();
    }
    #[test]
    fn timeout_sends_cancellation_and_late_response_cannot_poison_next_request() {
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Rpc::new(a).unwrap();
        let server = std::thread::spawn(move || {
            let (old, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            let (cancel, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            assert!(matches!(cancel.body,Command::Cancel{request} if request==old.id));
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: old.id,
                    body: Response::Events(None),
                },
                None,
            )
            .unwrap();
            let (next, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION,
                    id: next.id,
                    body: Response::Connected {
                        restore: "new".into(),
                    },
                },
                None,
            )
            .unwrap();
        });
        assert!(
            rpc.call(
                Command::Pull {
                    cursor: "old".into()
                },
                Duration::from_millis(20)
            )
            .is_err()
        );
        let reply = rpc
            .call(
                Command::Pull {
                    cursor: "next".into(),
                },
                Duration::from_secs(2),
            )
            .unwrap();
        assert!(matches!(reply.body,Response::Connected{restore} if restore=="new"));
        server.join().unwrap();
    }
    #[test]
    fn protocol_version_mismatch_disconnects_all_waiters() {
        let (a, mut b) = UnixStream::pair().unwrap();
        let rpc = Rpc::new(a).unwrap();
        let server = std::thread::spawn(move || {
            let (req, _) = transport::receive::<Envelope<Command>>(&mut b).unwrap();
            transport::send(
                &mut b,
                &Envelope {
                    version: VERSION + 1,
                    id: req.id,
                    body: Response::Events(None),
                },
                None,
            )
            .unwrap();
        });
        assert!(
            rpc.call(
                Command::Pull {
                    cursor: String::new()
                },
                Duration::from_secs(2)
            )
            .is_err()
        );
        server.join().unwrap();
    }
}

#[cfg(test)]
#[path = "transfer_rpc_tests.rs"]
mod transfer_rpc_tests;

#[cfg(test)]
#[path = "wait_rpc_tests.rs"]
mod wait_rpc_tests;
