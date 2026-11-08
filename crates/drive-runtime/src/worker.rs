use crate::{transport, wire::*};
use drive_core::native::NativeReadSession;
use std::{
    collections::HashMap,
    io,
    os::{fd::OwnedFd, unix::net::UnixStream},
    sync::Arc,
};
/// Job bodies concurrently active per worker session. Bursts above this limit
/// queue instead of ending the session (#31).
pub(crate) const MAX_IN_FLIGHT: usize = 32;
/// Hard bound on accepted commands (running + queued) per worker session.
/// Beyond it the worker refuses the individual command with a retryable
/// `worker_overloaded` category instead of dropping the whole session.
pub(crate) const MAX_ACCEPTED: usize = 160;
/// Fixed IPC category for a retryable overload refusal. Never a transport
/// loss: clients keep the session and retry the command.
pub(crate) const WORKER_OVERLOADED_CATEGORY: &str = "worker_overloaded";
/// A malformed or replayed command id is refused and dropped; unrelated jobs
/// and the session survive.
pub(crate) const INVALID_REQUEST_CATEGORY: &str = "invalid_worker_request";

fn failed_envelope(id: u64, category: &str) -> Envelope<Response> {
    Envelope {
        version: VERSION,
        id,
        body: Response::Failed {
            category: category.into(),
        },
    }
}

/// Bounded command admission shared by the worker loop and its tests.
///
/// The loop owns at most `MAX_IN_FLIGHT` executing bodies and at most
/// `max_accepted` accepted commands. Excess commands receive the retryable
/// `worker_overloaded` response; cancellation reaches queued jobs through the
/// stored abort handle, and shutdown aborts every accepted command.
struct JobQueue {
    in_flight: Arc<tokio::sync::Semaphore>,
    max_accepted: usize,
    handles: HashMap<u64, tokio::task::AbortHandle>,
    tasks: tokio::task::JoinSet<u64>,
}
impl JobQueue {
    fn new(max_in_flight: usize, max_accepted: usize) -> Self {
        Self {
            in_flight: Arc::new(tokio::sync::Semaphore::new(max_in_flight)),
            max_accepted,
            handles: HashMap::new(),
            tasks: tokio::task::JoinSet::new(),
        }
    }
    fn accepted(&self) -> usize {
        self.handles.len()
    }
    fn can_accept(&self) -> bool {
        self.accepted() < self.max_accepted
    }
    fn has_jobs(&self) -> bool {
        !self.handles.is_empty()
    }
    fn contains(&self, id: u64) -> bool {
        self.handles.contains_key(&id)
    }
    fn spawn<F>(&mut self, id: u64, job: F)
    where
        F: std::future::Future<Output = ()> + Send + 'static,
    {
        let in_flight = self.in_flight.clone();
        let abort = self.tasks.spawn(async move {
            let Ok(_permit) = in_flight.acquire_owned().await else {
                return id;
            };
            job.await;
            id
        });
        self.handles.insert(id, abort);
    }
    fn cancel(&mut self, id: u64) -> bool {
        let Some(abort) = self.handles.remove(&id) else {
            return false;
        };
        abort.abort();
        true
    }
    async fn next_completed(&mut self) -> Option<u64> {
        loop {
            match self.tasks.join_next().await {
                Some(Ok(id)) => {
                    self.handles.remove(&id);
                    return Some(id);
                }
                // Aborted tasks were removed from `handles` by `cancel`; a
                // panicked task must not take the session down with it.
                Some(Err(_)) => continue,
                None => return None,
            }
        }
    }
    async fn shutdown(mut self) {
        self.tasks.abort_all();
        while self.tasks.join_next().await.is_some() {}
        self.handles.clear();
    }
}
pub async fn serve(socket: UnixStream) -> io::Result<()> {
    transport::check_peer(&socket)?;
    let mut reader = socket.try_clone()?;
    let mut writer = socket;
    let (tx, mut rx) = tokio::sync::mpsc::channel(64);
    std::thread::spawn(move || {
        while let Ok(packet) = transport::receive::<Envelope<Command>>(&mut reader) {
            if tx.blocking_send(packet).is_err() {
                break;
            }
        }
    });
    let (out_tx, out_rx) =
        std::sync::mpsc::sync_channel::<(Envelope<Response>, Option<OwnedFd>)>(64);
    let writer_thread = std::thread::spawn(move || {
        for (env, fd) in out_rx {
            if transport::send(&mut writer, &env, fd.as_ref()).is_err() {
                break;
            }
        }
    });
    let Some((hello, fd)) = rx.recv().await else {
        return Ok(());
    };
    if fd.is_some() || hello.version != VERSION {
        return Err(io::Error::other("invalid worker handshake"));
    }
    let Command::Connect(login) = hello.body else {
        return Err(io::Error::other("worker requires login handshake"));
    };
    let view = login.scope.view_id;
    let trusted_ca_der = match sproto::trust::parse_trusted_pem(&login.trusted_ca_pem) {
        Ok(certs) => certs,
        Err(_) => {
            let _ = out_tx.send((
                Envelope {
                    version: VERSION,
                    id: hello.id,
                    body: Response::Failed {
                        category: "tls_or_configuration".into(),
                    },
                },
                None,
            ));
            drop(out_tx);
            let _ = writer_thread.join();
            return Ok(());
        }
    };
    let config = sproto::client::Config::builder()
        .host(login.host)
        .tls_server_name(login.tls_name)
        .trusted_ca_der(trusted_ca_der)
        .device_uuid(login.device_id)
        .max_channels(6usize)
        .credentials(sproto::client::Credentials::Password {
            username: login.username,
            password: login.password,
            otp: None,
        })
        .build();
    let client = match sproto::Client::connect(config).await {
        Ok(c) => c,
        Err(error) => {
            let category = login_error_category(&error);
            let _ = out_tx.send((
                Envelope {
                    version: VERSION,
                    id: hello.id,
                    body: Response::Failed {
                        category: category.into(),
                    },
                },
                None,
            ));
            drop(out_tx);
            let _ = writer_thread.join();
            return Ok(());
        }
    };
    let restore = client.restore_identity().to_string();
    let session = Arc::new(NativeReadSession::new(client, login.scope).map_err(io::Error::other)?);
    out_tx
        .send((
            Envelope {
                version: VERSION,
                id: hello.id,
                body: Response::Connected { restore },
            },
            None,
        ))
        .map_err(|_| io::Error::other("IPC output closed"))?;
    let flow_control = sproto::flow::TransferControl::new(0, 0);
    let staging = Arc::new(tempfile::tempdir()?);
    let mut queue = JobQueue::new(MAX_IN_FLIGHT, MAX_ACCEPTED);
    loop {
        tokio::select! {
            incoming = rx.recv() => {
                let Some((env, fd)) = incoming else {
                    break;
                };
                if env.version != VERSION || !valid_command_descriptor(&env.body, fd.is_some()) {
                    break;
                }
                if let Command::ConfigureTransfers(limits) = &env.body {
                    if env.id == 0 || limits.validate().is_err() {
                        break;
                    }
                    flow_control.set_limits(limits.download_bps, limits.upload_bps);
                    let response = Envelope {
                        version: VERSION,
                        id: env.id,
                        body: Response::TransfersConfigured(*limits),
                    };
                    if out_tx.try_send((response, None)).is_err() {
                        break;
                    }
                    continue;
                }
                if let Command::Cancel { request } = env.body {
                    queue.cancel(request);
                    continue;
                }
                if env.id == 0 || queue.contains(env.id) {
                    // A replayed or malformed id is refused per request; it
                    // never takes unrelated jobs or the session down.
                    if out_tx
                        .try_send((failed_envelope(env.id, INVALID_REQUEST_CATEGORY), None))
                        .is_err()
                    {
                        break;
                    }
                    continue;
                }
                if !queue.can_accept() {
                    // Bounded backpressure: refuse this command with a
                    // retryable category and keep the session and its jobs.
                    if out_tx
                        .try_send((failed_envelope(env.id, WORKER_OVERLOADED_CATEGORY), None))
                        .is_err()
                    {
                        break;
                    }
                    continue;
                }
                let session = session.clone();
                let output = out_tx.clone();
                let staging = staging.clone();
                let control = flow_control.clone();
                let progress_output = output.clone();
                let request_id = env.id;
                let observer = Arc::new(move |progress: sproto::flow::Progress| {
                    let _ = progress_output.try_send((
                        Envelope {
                            version: VERSION,
                            id: request_id,
                            body: Response::TransferProgress(progress.into()),
                        },
                        None,
                    ));
                });
                queue.spawn(request_id, async move {
            let result: drive_core::Result<(Response, Option<OwnedFd>)> =
                sproto::flow::scope(control, observer, async {
                    match env.body {
                        #[cfg(feature = "native-research")]
                        Command::Mkdir { job, reconcile } => {
                            let response = match session.create_directory(&job, reconcile).await {
                                Ok(drive_core::native::NativeDirectoryOutcome::Prepared(
                                    origin,
                                )) => Response::DirectoryPrepared { origin },
                                Ok(drive_core::native::NativeDirectoryOutcome::Created {
                                    node,
                                    remote,
                                }) => Response::Created { node, remote },
                                Ok(drive_core::native::NativeDirectoryOutcome::Conflict) => {
                                    Response::Conflict
                                }
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionUncertain { code })) => mutation_permission_response("mkdir", Some(&job.node), code),
                                // Directory creation is addressed to a child that
                                // does not exist yet; never a definite denial.
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionDenied { code })) => mutation_permission_response("mkdir", Some(&job.node), code),
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }

                        #[cfg(feature = "native-research")]
                        Command::Replace {
                            job,
                            sha256,
                            reconcile,
                        } => {
                            use std::os::fd::AsRawFd;
                            let descriptor = fd.ok_or_else(|| {
                                drive_core::Error::State("replacement descriptor missing".into())
                            })?;
                            let source = std::path::PathBuf::from(format!(
                                "/proc/self/fd/{}",
                                descriptor.as_raw_fd()
                            ));
                            let response = match session
                                .replace_target(&job, &source, &sha256, staging.path(), reconcile)
                                .await
                            {
                                Ok(drive_core::native::NativeWriteOutcome::Created {
                                    node,
                                    remote,
                                }) => Response::Created { node, remote },
                                Ok(drive_core::native::NativeWriteOutcome::Confirmed(node)) => {
                                    Response::Committed { node }
                                }
                                Ok(drive_core::native::NativeWriteOutcome::Conflict) => {
                                    Response::Conflict
                                }
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionUncertain { code })) => mutation_permission_response("replace", Some(&job.target), code),
                                // A compound replace mixes upload and cleanup phases; the
                                // client target set has no single target write_content pair.
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionDenied { code })) => mutation_permission_response("replace", Some(&job.target), code),
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }
                        #[cfg(feature = "native-research")]
                        Command::Remove { job, reconcile } => {
                            use drive_core::native::NativeRemoveOutcome;
                            let response = match session.remove(&job, reconcile).await {
                                Ok(NativeRemoveOutcome::Confirmed { identity }) => {
                                    Response::Removed { identity }
                                }
                                Ok(NativeRemoveOutcome::Conflict) => Response::Conflict,
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionUncertain { code })) => mutation_permission_response("remove", Some(&job.node), code),
                                // Delete target codes keep the existing hint: 0xa004 is
                                // also profile read-only filtering and 0xa003 has no
                                // proven producer.
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionDenied { code })) => mutation_permission_response("remove", Some(&job.node), code),
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }
                        #[cfg(feature = "native-research")]
                        Command::Rename { job, reconcile } => {
                            use drive_core::native::NativeRenameOutcome;
                            let response = match session.rename(&job, reconcile).await {
                                Ok(NativeRenameOutcome::Confirmed { node, same_content }) => {
                                    Response::Renamed { node, same_content }
                                }
                                Ok(NativeRenameOutcome::Refresh { node }) => {
                                    Response::DirectoryRenameRefresh { node }
                                }
                                Ok(NativeRenameOutcome::Conflict) => Response::Conflict,
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionUncertain { code })) => mutation_permission_response("rename", Some(&job.node), code),
                                // Rename source and destination tests share one reply with
                                // no participant field; never learn which side was denied.
                                Err(drive_core::Error::Native(sproto::Error::MutationPermissionDenied { code })) => mutation_permission_response("rename", Some(&job.node), code),
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }

                        #[cfg(feature = "native-research")]
                        Command::PrepareUpload {
                            node,
                            parent,
                            path,
                            sha256,
                            cursor,
                        } => {
                            if fd.is_some() {
                                return Err(drive_core::Error::State(
                                    "preparation received content".into(),
                                ));
                            }
                            let response = match session
                                .prepare_upload_creation(&node, &parent, &path, &sha256, &cursor)
                                .await?
                            {
                                Some(origin) => Response::UploadPrepared { origin },
                                None => Response::Conflict,
                            };
                            Ok((response, None))
                        }
                        #[cfg(feature = "native-research")]
                        Command::Upload {
                            origin,
                            node,
                            path,
                            sha256,
                            reconcile,
                            mtime,
                        } => {
                            use std::os::fd::AsRawFd;
                            let descriptor = fd.ok_or_else(|| {
                                drive_core::Error::State("upload descriptor absent".into())
                            })?;
                            let source = std::path::PathBuf::from(format!(
                                "/proc/self/fd/{}",
                                descriptor.as_raw_fd()
                            ));
                            let result = if reconcile {
                                session
                                    .reconcile_with_origin_mtime(
                                        &node,
                                        &path,
                                        &source,
                                        &sha256,
                                        staging.path(),
                                        origin.as_ref(),
                                        mtime,
                                    )
                                    .await
                            } else {
                                session
                                    .upload_with_origin_mtime(
                                        &node,
                                        &path,
                                        &source,
                                        &sha256,
                                        staging.path(),
                                        origin.as_ref(),
                                        mtime,
                                    )
                                    .await
                            };
                            let response = match result {
                                Ok(drive_core::native::NativeWriteOutcome::Created {
                                    node,
                                    remote,
                                }) => Response::Created { node, remote },
                                Ok(drive_core::native::NativeWriteOutcome::Confirmed(node)) => {
                                    Response::Committed { node }
                                }
                                Ok(drive_core::native::NativeWriteOutcome::Conflict) => {
                                    Response::Conflict
                                }
                                Err(drive_core::Error::Native(
                                    sproto::Error::MutationPermissionUncertain { code },
                                )) => Response::MutationPermissionUncertain { code },
                                // A typed pre-commit ACL refusal is published as
                                // a definite denial only for the identity-addressed
                                // upload of an existing node. A creation is
                                // addressed to a child that does not exist yet, so
                                // its capability check is not a CreateFile
                                // primitive and keeps the uncertain/hint path.
                                Err(drive_core::Error::Native(
                                    sproto::Error::MutationPermissionDenied { code },
                                )) => mutation_permission_response("upload", Some(&node), code),
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }
                        Command::Initialization(request) => {
                            // Metadata travels as one bounded, checksummed FD
                            // payload, never as lossy IndexProgress messages.
                            // The common command gate rejects incoming FDs.
                            let capture = sproto::diagnostics::Capture::new();
                            let reply = sproto::diagnostics::scope(
                                capture.clone(),
                                session.initialization(request),
                            )
                            .await;
                            // Reliable before success OR failure. No payload, IDs
                            // or credentials; this message is not a checkpoint.
                            let diagnostics = capture.snapshot();
                            let diagnostics_output = output.clone();
                            tokio::task::spawn_blocking(move || {
                                diagnostics_output.send((
                                    Envelope {
                                        version: VERSION,
                                        id: request_id,
                                        body: Response::InitializationDiagnostics(diagnostics),
                                    },
                                    None,
                                ))
                            })
                            .await
                            .map_err(|_| {
                                drive_core::Error::State("diagnostics task failed".into())
                            })?
                            .map_err(|_| drive_core::Error::State("IPC output closed".into()))?;
                            let reply = reply?;
                            let payload = tokio::task::spawn_blocking(move || {
                                initialization_payload(&reply, staging.path(), 512 * 1024 * 1024)
                            })
                            .await
                            .map_err(|_| {
                                drive_core::Error::State(
                                    "initialization payload task failed".into(),
                                )
                            })??;
                            Ok((
                                Response::Initialization {
                                    size: payload.0,
                                    sha256: payload.1,
                                },
                                Some(payload.2),
                            ))
                        }
                        Command::RecoverySnapshot => {
                            use std::io::{Seek, Write};
                            let progress =
                                crate::index_progress::Reporter::new(request_id, output.clone());
                            let snapshot = drive_core::snapshot_progress::scope(
                                progress.observer(),
                                session.recovery_snapshot(),
                            )
                            .await;
                            progress.flush();
                            let snapshot = snapshot?;
                            let bytes = serde_json::to_vec(&snapshot)?;
                            if bytes.len() > 512 * 1024 * 1024 {
                                return Err(drive_core::Error::State(
                                    "snapshot transfer budget exceeded".into(),
                                ));
                            }
                            let mut file = tempfile::tempfile_in(staging.path())?;
                            file.write_all(&bytes)?;
                            file.sync_all()?;
                            file.rewind()?;
                            Ok((
                                Response::RecoverySnapshot {
                                    size: bytes.len() as u64,
                                    sha256: drive_core::sha256(&bytes),
                                },
                                Some(file.into()),
                            ))
                        }
                        Command::Pull { cursor } => {
                            Ok((Response::Events(session.pull(&cursor).await?), None))
                        }
                        Command::Wait { cursor, digest } => {
                            let n = sproto::receive_activity::scope(
                                crate::wait_activity::observer(env.id, output.clone()),
                                session.wait(&cursor, &digest),
                            )
                            .await?;
                            Ok((
                                notification_response(view, n),
                                None,
                            ))
                        }
                        Command::Download { node } => {
                            let d = match session.download(&node, staging.path()).await {
                                Ok(d) => d,
                                Err(error) => return download_error_response(&node.id, error),
                            };
                            Ok((
                                Response::Content {
                                    identity: d.identity,
                                    version: d.version,
                                    size: d.size,
                                    sha256: d.sha256,
                                },
                                Some(d.file.into()),
                            ))
                        }
                        _ => Err(drive_core::Error::Unsupported("invalid worker command")),
                    }
                })
                .await;
            let (body, fd) = match result {
                Ok(r) => r,
                Err(error) => (
                    Response::Failed {
                        category: worker_error_category(&error),
                    },
                    None,
                ),
            };
            let _ = tokio::task::spawn_blocking(move || {
                output.send((
                    Envelope {
                        version: VERSION,
                        id: env.id,
                        body,
                    },
                    fd,
                ))
            })
            .await;
                });
            }
            _ = queue.next_completed(), if queue.has_jobs() => {}
        }
    }
    queue.shutdown().await;
    Ok(())
}

/// Bound serialization as it is produced, rather than allocating an unbounded
/// JSON Vec and checking only afterwards. A failed or cancelled reply leaves no
/// named file and cannot advance any durable initialization receipt.
fn initialization_payload(
    reply: &drive_core::native::InitializationReply,
    staging: &std::path::Path,
    limit: u64,
) -> drive_core::Result<(u64, String, OwnedFd)> {
    use sha2::{Digest, Sha256};
    use std::io::{Seek, Write};
    struct Payload {
        file: std::fs::File,
        hash: Sha256,
        size: u64,
        limit: u64,
    }
    impl Write for Payload {
        fn write(&mut self, bytes: &[u8]) -> io::Result<usize> {
            let next = self
                .size
                .checked_add(bytes.len() as u64)
                .filter(|next| *next <= self.limit)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidData,
                        "initialization transfer budget exceeded",
                    )
                })?;
            self.file.write_all(bytes)?;
            self.hash.update(bytes);
            self.size = next;
            Ok(bytes.len())
        }
        fn flush(&mut self) -> io::Result<()> {
            self.file.flush()
        }
    }
    let mut payload = Payload {
        file: tempfile::tempfile_in(staging)?,
        hash: Sha256::new(),
        size: 0,
        limit,
    };
    // Serde writes tiny JSON fragments; buffer them before file syscalls and
    // hashing so a wide directory does not cause millions of short writes.
    {
        let mut buffered = std::io::BufWriter::with_capacity(64 * 1024, &mut payload);
        serde_json::to_writer(&mut buffered, reply)?;
        buffered.flush()?;
    }
    payload.file.sync_all()?;
    payload.file.rewind()?;
    Ok((
        payload.size,
        format!("{:x}", payload.hash.finalize()),
        payload.file.into(),
    ))
}

pub(crate) fn notification_response(view: u64, n: sproto::actions::native::Notification) -> Response {
    Response::Notification {
        changed: n.changed_views.contains(&view),
        profile_changed: n.profile_changed,
        digest: n.profile_digest,
    }
}

/// Only used for the identity-addressed, read-only download request. These
/// native codes are verified in the official client's ProtoResp2ErrorCode;
/// see docs/validation/issue7-native-001/REPORT.md. Neither text messages nor
/// local PermissionDenied I/O errors are evidence of a remote ACL decision.
pub(crate) fn download_error_response(
    node: &str,
    error: drive_core::Error,
) -> drive_core::Result<(Response, Option<OwnedFd>)> {
    match error {
        drive_core::Error::Native(sproto::Error::DownloadPermissionDenied { .. }) =>
            Ok((Response::PermissionDenied {
                node: node.to_owned(),
                action: drive_core::write_permissions::Action::Read,
            }, None)),
        error => Err(error),
    }
}

fn valid_command_descriptor(command: &Command, has_descriptor: bool) -> bool {
    has_descriptor == matches!(command, Command::Upload { .. } | Command::Replace { .. })
}

/// Bind a typed pre-commit ACL refusal to this request. A definite denial is
/// published only for an identity-addressed upload of an existing node: that
/// request names exactly one `(node, write_content)` pair, and the server's
/// Create ACL test runs after the header is read and before `ReadBody`, the
/// handler and any commit (server-static/REPORT.md, function anchors 0x223710 /
/// 0x223140 / 0x220610). A creation is addressed to a child that does not exist
/// yet, so its capability check is not a CreateFile/CreateDir primitive. Other
/// mutation kinds never qualify and keep the existing pending hint, so the
/// diagnostic is not lost.
fn mutation_permission_response(
    kind: &str,
    node: Option<&drive_core::Node>,
    code: u32,
) -> Response {
    if kind == "upload" {
        if let Some(node) = node {
            if node.version != drive_core::creations::CREATE_VERSION {
                // Defence in depth (T4 review): sproto already guarantees this
                // typed variant only for 0x9003, but the learning path must not
                // publish a definite denial for any other numeric code.
                debug_assert_eq!(
                    code, 0x9003,
                    "typed ACL denial must be the Create ACL code"
                );
                return Response::PermissionDenied {
                    node: node.id.clone(),
                    action: drive_core::write_permissions::Action::WriteContent,
                };
            }
        }
    }
    Response::MutationPermissionUncertain { code }
}

#[cfg(test)]
mod mutation_permission_tests {
    use super::*;
    fn node(version: &str) -> drive_core::Node {
        drive_core::Node {
            id: "42".into(),
            parent: Some("1".into()),
            name: "A.doc".into(),
            version: version.into(),
            directory: false,
            metadata: None,
        }
    }
    #[test]
    fn only_an_existing_node_upload_binds_the_definite_refusal() {
        match mutation_permission_response("upload", Some(&node("v7")), 0x9003) {
            Response::PermissionDenied { node, action } => {
                assert_eq!(node, "42");
                assert_eq!(action, drive_core::write_permissions::Action::WriteContent);
            }
            _ => panic!("expected a definite upload denial"),
        }
    }
    #[test]
    fn creation_and_every_other_mutation_keep_the_uncertain_hint() {
        // A creation has no target node yet: the ACL Create test there is not a
        // CreateFile primitive, so the code must stay a pending hint.
        for (kind, version) in [
            ("upload", drive_core::creations::CREATE_VERSION),
            ("mkdir", drive_core::creations::CREATE_VERSION),
            ("replace", "v7"),
            ("remove", "v7"),
            ("rename", "v7"),
        ] {
            match mutation_permission_response(kind, Some(&node(version)), 0x9003) {
                Response::MutationPermissionUncertain { code } => assert_eq!(code, 0x9003),
                _ => panic!("{kind} must stay uncertain/hint"),
            }
        }
        match mutation_permission_response("upload", None, 0x9003) {
            Response::MutationPermissionUncertain { code } => assert_eq!(code, 0x9003),
            _ => panic!("missing request identity must stay uncertain"),
        }
    }
}

#[cfg(test)]
mod initialization_payload_tests {
    use super::*;
    fn reply() -> drive_core::native::InitializationReply {
        drive_core::native::InitializationReply {
            scope: drive_core::native::Scope {
                view_id: 5,
                node_id: 7,
                file_id: "10".into(),
                path: "/selected".into(),
            },
            restore: "server".into(),
            body: drive_core::native::InitializationBody::Events(None),
        }
    }
    #[test]
    fn payload_fd_roundtrips_at_exact_budget_and_starts_at_zero() {
        use std::io::Read;
        let temp = tempfile::tempdir().unwrap();
        let reply = reply();
        let expected = serde_json::to_vec(&reply).unwrap();
        let (size, hash, fd) =
            initialization_payload(&reply, temp.path(), expected.len() as u64).unwrap();
        let mut bytes = Vec::new();
        std::fs::File::from(fd).read_to_end(&mut bytes).unwrap();
        assert_eq!(size, expected.len() as u64);
        assert_eq!(hash, drive_core::sha256(&bytes));
        assert_eq!(bytes, expected);
        assert_eq!(
            serde_json::from_slice::<drive_core::native::InitializationReply>(&bytes).unwrap(),
            reply
        );
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }
    #[test]
    fn budget_failure_leaves_no_named_partial_payload() {
        let temp = tempfile::tempdir().unwrap();
        let reply = reply();
        let size = serde_json::to_vec(&reply).unwrap().len() as u64;
        assert!(initialization_payload(&reply, temp.path(), size - 1).is_err());
        assert_eq!(std::fs::read_dir(temp.path()).unwrap().count(), 0);
    }
    #[test]
    fn initialization_commands_never_accept_incoming_descriptors() {
        use drive_core::native::InitializationRequest;
        for request in [
            InitializationRequest::Seed,
            InitializationRequest::Root,
            InitializationRequest::Pull {
                cursor: "native-v1:1".into(),
            },
        ] {
            let command = Command::Initialization(request);
            assert!(valid_command_descriptor(&command, false));
            assert!(!valid_command_descriptor(&command, true));
        }
        assert!(valid_command_descriptor(&Command::RecoverySnapshot, false));
        assert!(!valid_command_descriptor(&Command::RecoverySnapshot, true));
    }
}

/// Keep server messages (which may echo credentials) out of IPC diagnostics.
fn login_error_category(error: &sproto::Error) -> &'static str {
    match error {
        sproto::Error::Io(e) if e.kind() == io::ErrorKind::InvalidData => "tls_or_configuration",
        sproto::Error::Io(_) | sproto::Error::Timeout(_) | sproto::Error::ConnectionClosed => {
            "transport"
        }
        sproto::Error::Tls(_) | sproto::Error::InvalidConfig(_) => "tls_or_configuration",
        _ => "authentication_or_protocol",
    }
}

fn worker_error_category(error: &drive_core::Error) -> String {
    match error {
        drive_core::Error::Native(sproto::Error::Timeout("native receive idle")) => {
            "native_receive_idle".into()
        }
        drive_core::Error::Native(sproto::Error::Timeout("notification channel acquisition")) => {
            "native_notification_setup_timeout".into()
        }
        drive_core::Error::Native(sproto::Error::Timeout("native request send")) => {
            "native_send_timeout".into()
        }
        drive_core::Error::Native(sproto::Error::Timeout("native response")) => {
            "native_response_timeout".into()
        }
        drive_core::Error::RootUnavailable => "native_root_unavailable".into(),
        drive_core::Error::SnapshotChanged => "native_snapshot_changed".into(),
        drive_core::Error::Native(sproto::Error::CursorReset { .. }) => {
            "native_cursor_reset".into()
        }
        drive_core::Error::Native(sproto::Error::Server { code, .. }) => {
            format!("native_server_code:{code}")
        }
        drive_core::Error::Native(sproto::Error::SessionInvalid) => "session_expired".into(),
        drive_core::Error::Native(e) if login_error_category(e) == "transport" => {
            "transport".into()
        }
        drive_core::Error::State(message)
            if matches!(
                message.as_str(),
                "snapshot seed did not reach boundary"
                    | "snapshot seed budget exceeded"
                    | "snapshot seed time budget exceeded"
                    | "snapshot node budget exceeded"
            ) =>
        {
            "native_index_capacity".into()
        }
        drive_core::Error::Native(sproto::Error::Decode(message))
            if matches!(
                message.as_str(),
                "metadata node budget exceeded" | "metadata string budget exceeded"
            ) =>
        {
            "native_response_capacity".into()
        }
        drive_core::Error::Unsupported("native symlink target contract") => {
            "native_filtered_type_changed".into()
        }
        _ => "native_recovery_required".into(),
    }
}
#[cfg(test)]
mod download_diagnostics_tests {
    #[test]
    fn timeout_stages_have_fixed_content_free_categories() {
        for (stage, category) in [
            ("native receive idle", "native_receive_idle"),
            (
                "notification channel acquisition",
                "native_notification_setup_timeout",
            ),
            ("native request send", "native_send_timeout"),
            ("native response", "native_response_timeout"),
            ("synthetic-private-input", "transport"),
        ] {
            let error = drive_core::Error::Native(sproto::Error::Timeout(stage));
            assert_eq!(super::worker_error_category(&error), category);
        }
    }
    #[test]
    fn numeric_server_code_never_includes_reason() {
        assert_eq!(
            super::worker_error_category(&drive_core::Error::SnapshotChanged),
            "native_snapshot_changed"
        );
        assert_eq!(
            super::worker_error_category(&drive_core::Error::RootUnavailable),
            "native_root_unavailable"
        );
        let error = drive_core::Error::Native(sproto::Error::Server {
            code: 32772,
            reason: "synthetic-private-input".into(),
        });
        assert_eq!(
            super::worker_error_category(&error),
            "native_server_code:32772"
        );
    }
}

#[cfg(test)]
#[path = "worker_queue_tests.rs"]
mod worker_queue_tests;

#[cfg(test)]
mod capacity_categories {
    #[test]
    fn fixed_categories_do_not_leak_messages() {
        use drive_core::Error;
        assert_eq!(
            super::worker_error_category(&Error::State(
                "snapshot seed did not reach boundary".into()
            )),
            "native_index_capacity"
        );
        assert_eq!(
            super::worker_error_category(&Error::Native(sproto::Error::Decode(
                "metadata node budget exceeded".into()
            ))),
            "native_response_capacity"
        );
        assert_eq!(
            super::worker_error_category(&Error::State("private server detail".into())),
            "native_recovery_required"
        );
        assert_eq!(
            super::worker_error_category(&Error::Unsupported("native symlink target contract")),
            "native_filtered_type_changed"
        );
    }
}
