use crate::{transport, wire::*};
use drive_core::native::NativeReadSession;
use std::{
    collections::HashMap,
    io,
    os::{fd::OwnedFd, unix::net::UnixStream},
    sync::Arc,
};
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
    let mut jobs: HashMap<u64, tokio::task::JoinHandle<()>> = HashMap::new();
    while let Some((env, fd)) = rx.recv().await {
        jobs.retain(|_, job| !job.is_finished());
        if env.version != VERSION
            || fd.is_some() != matches!(&env.body, Command::Upload { .. } | Command::Replace { .. })
        {
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
            if let Some(job) = jobs.remove(&request) {
                job.abort()
            }
            continue;
        }
        if env.id == 0 || jobs.contains_key(&env.id) || jobs.len() >= 32 {
            break;
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
        let job = tokio::spawn(async move {
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
                                    .reconcile_with_origin(
                                        &node,
                                        &path,
                                        &source,
                                        &sha256,
                                        staging.path(),
                                        origin.as_ref(),
                                    )
                                    .await
                            } else {
                                session
                                    .upload_with_origin(
                                        &node,
                                        &path,
                                        &source,
                                        &sha256,
                                        staging.path(),
                                        origin.as_ref(),
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
                                _ => Response::Uncertain,
                            };
                            Ok((response, None))
                        }
                        Command::RecoverySnapshot => {
                            use std::io::{Seek, Write};
                            let progress = crate::index_progress::Reporter::new(request_id, output.clone());
                            let snapshot = drive_core::snapshot_progress::scope(progress.observer(), session.recovery_snapshot()).await;
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
                            ).await?;
                            Ok((
                                Response::Notification {
                                    changed: n.changed_views.contains(&view),
                                    digest: n.profile_digest,
                                },
                                None,
                            ))
                        }
                        Command::Download { node } => {
                            let d = session.download(&node, staging.path()).await?;
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
        jobs.insert(env.id, job);
    }
    for (_, job) in jobs {
        job.abort()
    }
    Ok(())
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
        drive_core::Error::Native(sproto::Error::Timeout("native receive idle")) => "native_receive_idle".into(),
        drive_core::Error::Native(sproto::Error::Timeout("notification channel acquisition")) => "native_notification_setup_timeout".into(),
        drive_core::Error::Native(sproto::Error::Timeout("native request send")) => "native_send_timeout".into(),
        drive_core::Error::Native(sproto::Error::Timeout("native response")) => "native_response_timeout".into(),
        drive_core::Error::RootUnavailable => "native_root_unavailable".into(),
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
        drive_core::Error::State(message) if matches!(message.as_str(),
            "snapshot seed did not reach boundary" | "snapshot seed budget exceeded" |
            "snapshot seed time budget exceeded" | "snapshot node budget exceeded") =>
            "native_index_capacity".into(),
        drive_core::Error::Native(sproto::Error::Decode(message)) if matches!(message.as_str(),
            "metadata node budget exceeded" | "metadata string budget exceeded") =>
            "native_response_capacity".into(),
        drive_core::Error::Unsupported("native symlink target contract") => "native_filtered_type_changed".into(),
        _ => "native_recovery_required".into(),
    }
}
#[cfg(test)]
mod download_diagnostics_tests {
    #[test]
    fn timeout_stages_have_fixed_content_free_categories() {
        for (stage, category) in [
            ("native receive idle", "native_receive_idle"),
            ("notification channel acquisition", "native_notification_setup_timeout"),
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

#[cfg(test)]mod capacity_categories {
 #[test]fn fixed_categories_do_not_leak_messages(){
  use drive_core::Error;
  assert_eq!(super::worker_error_category(&Error::State("snapshot seed did not reach boundary".into())),"native_index_capacity");
  assert_eq!(super::worker_error_category(&Error::Native(sproto::Error::Decode("metadata node budget exceeded".into()))),"native_response_capacity");
  assert_eq!(super::worker_error_category(&Error::State("private server detail".into())),"native_recovery_required");
  assert_eq!(super::worker_error_category(&Error::Unsupported("native symlink target contract")),"native_filtered_type_changed");
 }
}
