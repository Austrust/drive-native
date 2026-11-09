use super::*;
impl Bridge {
    /// Namespace failures describe unresolved durable intents. They are cleared
    /// only when this journal drains; an upload success cannot clear them.
    pub(crate) fn clear_namespace_failure_if_drained(&self) -> io::Result<()> {
        let drained = self
            .store
            .lock()
            .unwrap()
            .pending_namespace_operations()
            .map_err(io::Error::other)?
            == 0;
        if drained {
            self.clear_failure(crate::background::BackgroundSource::NamespaceUnconfirmed);
        }
        Ok(())
    }
    /// Process the durable sequence. An uncertain head is queried, never resent.
    pub fn rename_pending(&self) -> io::Result<usize> {
        let _mutation = self.recovery_gate.read().unwrap();
        if self.session_pending.load(Ordering::Acquire) {
            return Ok(0);
        }
        if self
            .store
            .lock()
            .unwrap()
            .initialization_writeback_blocked()
            .map_err(io::Error::other)?
        {
            return Ok(0);
        }
        self.rename_pending_under_gate(usize::MAX)
    }
    // namespace_pending already holds the guard. Never recursively acquire an
    // RwLock read guard: a queued recovery writer could otherwise deadlock us.
    fn rename_pending_under_gate(&self, limit: usize) -> io::Result<usize> {
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; namespace queue retained",
            ));
        }
        let mut done = 0;
        loop {
            let Some(queued) = self
                .store
                .lock()
                .unwrap()
                .scheduled_rename()
                .map_err(io::Error::other)?
            else {
                return Ok(done);
            };
            let reconcile = match queued.state.as_str() {
                "pending" => false,
                "uncertain" => true,
                _ => return Ok(done),
            };
            if !reconcile
                && self
                    .store
                    .lock()
                    .unwrap()
                    .mark_rename_started(&queued.job.operation)
                    .is_err()
            {
                return Ok(done);
            }
            let permission_targets=drive_core::write_permissions::namespace_targets("rename",&serde_json::to_string(&queued.job)?).map_err(io::Error::other)?;
            let op = queued.job.operation.clone();
            let fresh = self
                .store
                .lock()
                .unwrap()
                .scheduled_rename()
                .map_err(io::Error::other)?
                .ok_or_else(|| io::Error::other("rename disappeared"))?;
            if fresh.job.operation != op {
                return Err(io::Error::other("rename order changed"));
            }
            let outcome = rpc.call(
                Command::Rename {
                    job: fresh.job,
                    reconcile,
                },
                Duration::from_secs(120),
            );
            let mut store = self.store.lock().unwrap();
            match outcome {
                Ok(reply) => match reply.body {
                    Response::DirectoryRenameRefresh { node } if !reconcile => {
                        if store.refresh_directory_rename(&op, &node).is_err() {
                            store.conflict_rename(&op).map_err(io::Error::other)?;
                        } else {
                            self.signal_upload();
                        }
                        return Ok(done);
                    }
                    Response::Renamed { node, same_content } => {
                        store
                            .confirm_rename(&op, &node, same_content)
                            .map_err(io::Error::other)?;
                        if let Ok(record) = store.record(&node.id) {
                            self.enqueue_invalidation(Invalidation::Inode(record.ino));
                        }
                        drive_core::write_permissions::observe_targets(&store,&permission_targets,true).map_err(io::Error::other)?;
                        done += 1;
                        if done >= limit {
                            return Ok(done);
                        }
                    }
                    Response::PermissionDenied{node,action} if !reconcile => {
                        drive_core::write_permissions::refuse_operation(&store,&op,true,&permission_targets,&node,action).map_err(io::Error::other)?;
                        return Ok(done);
                    }
                    Response::MutationPermissionUncertain { code } if !reconcile => {
                        drive_core::write_permissions::mutation_permission_uncertain(&store,&op,true,code).map_err(io::Error::other)?;
                        return Ok(done);
                    }
                    Response::Conflict => {
                        store.conflict_rename(&op).map_err(io::Error::other)?;
                        return Ok(done);
                    }
                    _ => {
                        if !reconcile {
                            store.mark_rename_uncertain_diagnosed(
                                &op,
                                CommitCategory::Protocol,
                                Some("server returned an unexpected rename reply"),
                            ).map_err(io::Error::other)?;
                        } else {
                            store.record_namespace_commit_diagnostic(
                                &op,
                                CommitCategory::Protocol,
                                Some("server returned an unexpected reply during rename reconcile"),
                                0,
                            ).map_err(io::Error::other)?;
                        }
                        return Ok(done);
                    }
                },
                Err(_) => {
                    if !reconcile {
                        store.mark_rename_uncertain_diagnosed(
                            &op,
                            CommitCategory::Transport,
                            Some("rename transport lost before the commit was observed"),
                        ).map_err(io::Error::other)?;
                    } else {
                        store.record_namespace_commit_diagnostic(
                            &op,
                            CommitCategory::Transport,
                            Some("rename transport lost during reconcile"),
                            0,
                        ).map_err(io::Error::other)?;
                    }
                    return Err(io::Error::other(
                        "rename commit unknown; durable intent retained",
                    ));
                }
            }
        }
    }
}

impl Bridge {
    /// Preserve dependency order, while independent operations may pass a
    /// retained uncertain/conflicting intent. Network mutation remains serial.
    pub fn namespace_pending(&self) -> io::Result<usize> {
        let _mutation = self.recovery_gate.read().unwrap();
        if self.session_pending.load(Ordering::Acquire) {
            return Ok(0);
        }
        // Includes uncertain rename/remove/replace/mkdir reconciliation: its
        // local journal effects must not race an incomplete candidate either.
        if self
            .store
            .lock()
            .unwrap()
            .initialization_writeback_blocked()
            .map_err(io::Error::other)?
        {
            return Ok(0);
        }
        let mut done = 0;
        for _ in 0..64 {
            let selected = self
                .store
                .lock()
                .unwrap()
                .next_namespace_operation()
                .map_err(io::Error::other)?;
            let Some(selected) = selected else {
                self.clear_namespace_failure_if_drained()?;
                return Ok(done);
            };
            let outcome = match selected.kind.as_str() {
                "rename" => self.rename_pending_under_gate(1),
                "remove" => self.remove_pending(),
                "replace" => self.replace_pending(),
                "mkdir" => self.directory_pending(),
                _ => return Err(io::Error::other("unknown namespace journal operation")),
            };
            let observation = if outcome.is_err() {
                drive_core::namespace_schedule::NamespaceRecoveryObservation::TransportUnavailable
            } else {
                drive_core::namespace_schedule::NamespaceRecoveryObservation::CommitUnconfirmed
            };
            let count = match outcome {
                Ok(count) => count,
                Err(error) => {
                    let retained = self
                        .store
                        .lock()
                        .unwrap()
                        .observe_namespace_unresolved(&selected.operation, observation)
                        .map_err(io::Error::other)?;
                    if !retained
                        || !self.connection().is_connected()
                        || error.kind() == io::ErrorKind::NotConnected
                    {
                        return Err(error);
                    }
                    self.note_failure(crate::background::BackgroundSource::NamespaceUnconfirmed);
                    // The read-only reconciliation failed, but unrelated native
                    // operations can still use the connected worker.
                    0
                }
            };
            if count > 0 {
                self.store
                    .lock()
                    .unwrap()
                    .clear_namespace_observation(&selected.operation);
                done += count;
            } else {
                // Double observation after an error would distort retry counts.
                let mut store = self.store.lock().unwrap();
                if matches!(
                    observation,
                    drive_core::namespace_schedule::NamespaceRecoveryObservation::CommitUnconfirmed
                ) {
                    store
                        .observe_namespace_unresolved(&selected.operation, observation)
                        .map_err(io::Error::other)?;
                }
                let next = store.next_namespace_operation().map_err(io::Error::other)?;
                if next.is_none_or(|next| next.operation == selected.operation) {
                    drop(store);
                    self.clear_namespace_failure_if_drained()?;
                    return Ok(done);
                }
            }
        }
        self.signal_upload();
        self.clear_namespace_failure_if_drained()?;
        Ok(done)
    }
    fn remove_pending(&self) -> io::Result<usize> {
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; delete retained",
            ));
        }
        let Some(queued) = self
            .store
            .lock()
            .unwrap()
            .scheduled_remove()
            .map_err(io::Error::other)?
        else {
            return Ok(0);
        };
        let reconcile = match queued.state.as_str() {
            "pending" => false,
            "uncertain" => true,
            _ => return Ok(0),
        };
        if !reconcile
            && self
                .store
                .lock()
                .unwrap()
                .mark_remove_started(&queued.job.operation)
                .is_err()
        {
            return Ok(0);
        }
        let permission_targets=drive_core::write_permissions::namespace_targets("remove",&serde_json::to_string(&queued.job)?).map_err(io::Error::other)?;
        let op = queued.job.operation.clone();
        let outcome = rpc.call(
            Command::Remove {
                job: queued.job,
                reconcile,
            },
            Duration::from_secs(120),
        );
        let mut store = self.store.lock().unwrap();
        match outcome {
            Ok(reply) => match reply.body {
                Response::Removed { identity } => {
                    store
                        .confirm_remove(&op, &identity)
                        .map_err(io::Error::other)?;
                    drive_core::write_permissions::observe_targets(&store,&permission_targets,true).map_err(io::Error::other)?;
                    Ok(1)
                }
                Response::PermissionDenied{node,action} if !reconcile => {
                    drive_core::write_permissions::refuse_operation(&store,&op,true,&permission_targets,&node,action).map_err(io::Error::other)?;Ok(0)
                }
                Response::MutationPermissionUncertain { code } if !reconcile => {
                    drive_core::write_permissions::mutation_permission_uncertain(&store,&op,true,code).map_err(io::Error::other)?;Ok(0)
                }
                Response::Conflict => {
                    store.conflict_remove(&op).map_err(io::Error::other)?;
                    Ok(0)
                }
                _ => {
                    if !reconcile {
                        store.mark_remove_uncertain_diagnosed(
                            &op,
                            CommitCategory::Protocol,
                            Some("server returned an unexpected remove reply"),
                        ).map_err(io::Error::other)?;
                    } else {
                        store.record_namespace_commit_diagnostic(
                            &op,
                            CommitCategory::Protocol,
                            Some("server returned an unexpected reply during remove reconcile"),
                            0,
                        ).map_err(io::Error::other)?;
                    }
                    Ok(0)
                }
            },
            Err(_) => {
                if !reconcile {
                    store.mark_remove_uncertain_diagnosed(
                        &op,
                        CommitCategory::Transport,
                        Some("remove transport lost before the commit was observed"),
                    ).map_err(io::Error::other)?;
                } else {
                    store.record_namespace_commit_diagnostic(
                        &op,
                        CommitCategory::Transport,
                        Some("remove transport lost during reconcile"),
                        0,
                    ).map_err(io::Error::other)?;
                }
                Err(io::Error::other(
                    "remove commit unknown; reconcile required",
                ))
            }
        }
    }
}

impl Bridge {
    fn replace_pending(&self) -> io::Result<usize> {
        use sha2::Digest;
        use std::io::{Read, Seek, SeekFrom};
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; replacement retained",
            ));
        }
        let Some(queued) = self
            .store
            .lock()
            .unwrap()
            .scheduled_replace()
            .map_err(io::Error::other)?
        else {
            return Ok(0);
        };
        let permission_targets=drive_core::write_permissions::namespace_targets("replace",&serde_json::to_string(&queued.job)?).map_err(io::Error::other)?;
        let op = queued.job.operation.clone();
        let mut target_remote = queued.job.target_remote.clone();
        let cleanup = matches!(
            queued.state.as_str(),
            "cleanup_pending" | "cleanup_uncertain"
        );
        let reconcile = matches!(queued.state.as_str(), "uncertain" | "cleanup_uncertain");
        if !matches!(
            queued.state.as_str(),
            "pending" | "uncertain" | "cleanup_pending" | "cleanup_uncertain"
        ) {
            return Ok(0);
        }
        let outcome =
            if cleanup {
                let job = {
                    let mut s = self.store.lock().unwrap();
                    if !reconcile && s.mark_replace_cleanup_started(&op).is_err() {
                        return Ok(0);
                    }
                    s.replacement_cleanup(&queued.job)
                        .map_err(io::Error::other)?
                };
                rpc.call(Command::Remove { job, reconcile }, Duration::from_secs(120))
            } else {
                let mut file = self
                    .store
                    .lock()
                    .unwrap()
                    .replacement_file(&queued.job)
                    .map_err(io::Error::other)?;
                let mut hash = sha2::Sha256::new();
                let mut buf = [0; 65536];
                loop {
                    let n = file.read(&mut buf)?;
                    if n == 0 {
                        break;
                    }
                    hash.update(&buf[..n]);
                }
                file.seek(SeekFrom::Start(0))?;
                let sha256 = format!("{:x}", hash.finalize());
                if !reconcile && queued.job.target.version == drive_core::creations::CREATE_VERSION
                {
                    let (parent, path, cursor) =
                        {
                            let s = self.store.lock().unwrap();
                            let parent =
                                s.node(queued.job.target.parent.as_deref().ok_or_else(|| {
                                    io::Error::other("replacement parent absent")
                                })?)
                                .map_err(io::Error::other)?;
                            (
                                parent,
                                format!(
                                    "{}/{}",
                                    s.scope_path()
                                        .map_err(io::Error::other)?
                                        .trim_end_matches('/'),
                                    queued.job.destination
                                ),
                                s.cursor().map_err(io::Error::other)?,
                            )
                        };
                    if parent.version == drive_core::creations::CREATE_VERSION {
                        return Ok(0);
                    }
                    let prepared = rpc.call(
                        Command::PrepareUpload {
                            node: queued.job.target.clone(),
                            parent,
                            path,
                            sha256: sha256.clone(),
                            cursor,
                        },
                        Duration::from_secs(120),
                    )?;
                    match prepared.body {
                        Response::UploadPrepared { origin } => self
                            .store
                            .lock()
                            .unwrap()
                            .prepare_replace_origin(&op, origin, &sha256)
                            .map_err(io::Error::other)?,
                        Response::Conflict => {
                            self.store
                                .lock()
                                .unwrap()
                                .conflict_replace(&op)
                                .map_err(io::Error::other)?;
                            return Ok(0);
                        }
                        _ => return Ok(0),
                    }
                }
                let job = {
                    let mut s = self.store.lock().unwrap();
                    if !reconcile && s.mark_replace_started(&op).is_err() {
                        return Ok(0);
                    }
                    let q = s
                        .scheduled_replace()
                        .map_err(io::Error::other)?
                        .ok_or_else(|| io::Error::other("replacement journal disappeared"))?;
                    if q.job.operation != op {
                        return Err(io::Error::other("replacement order changed"));
                    }
                    q.job
                };
                target_remote = job.target_remote.clone();
                rpc.call_with_descriptor(
                    Command::Replace {
                        job,
                        sha256,
                        reconcile,
                    },
                    Duration::from_secs(1800),
                    Some(file.into()),
                )
            };
        let mut s = self.store.lock().unwrap();
        let mut unknown_category = CommitCategory::Protocol;
        let mut unknown_error = "server returned an unexpected replacement reply";
        let result = match outcome {
            Ok(reply) => match reply.body {
                Response::PermissionDenied{node,action} if !reconcile => {
                    drive_core::write_permissions::refuse_operation(&s,&op,true,&permission_targets,&node,action).map_err(io::Error::other)?;return Ok(0);
                }
                Response::MutationPermissionUncertain { code } if !reconcile => {
                    drive_core::write_permissions::mutation_permission_uncertain(&s,&op,true,code).map_err(io::Error::other)?;return Ok(0);
                }
                Response::Removed { identity } if cleanup => {
                    Some(s.confirm_replace_cleanup(&op, &identity))
                }
                Response::Created { node, remote } if !cleanup => {
                    Some(s.confirm_replace_upload(&op, &node, &remote))
                }
                Response::Committed { node } if !cleanup => {
                    let remote = target_remote.as_deref().ok_or_else(|| {
                        io::Error::other("existing replacement target identity absent")
                    })?;
                    Some(s.confirm_replace_upload(&op, &node, remote))
                }
                Response::Conflict => {
                    s.conflict_replace(&op).map_err(io::Error::other)?;
                    return Ok(0);
                }
                _ => None,
            },
            Err(_) => {
                unknown_category = CommitCategory::Transport;
                unknown_error = "replacement transport lost before the commit was observed";
                None
            }
        };
        match result {
            Some(Ok(())) => {drive_core::write_permissions::observe_targets(&s,&permission_targets,true).map_err(io::Error::other)?;Ok(1)},
            _ => {
                if !reconcile {
                    s.mark_replace_uncertain_diagnosed(&op, unknown_category, Some(unknown_error)).map_err(io::Error::other)?;
                } else {
                    s.record_namespace_commit_diagnostic(&op, unknown_category, Some(unknown_error), 0).map_err(io::Error::other)?;
                }
                Ok(0)
            }
        }
    }
}

impl Bridge {
    fn directory_pending(&self) -> io::Result<usize> {
        let rpc = self.connection();
        if !rpc.is_connected() {
            return Err(io::Error::new(
                io::ErrorKind::NotConnected,
                "worker offline; directory retained",
            ));
        }
        let Some(q) = self
            .store
            .lock()
            .unwrap()
            .scheduled_directory()
            .map_err(io::Error::other)?
        else {
            return Ok(0);
        };
        let reconcile = match q.state.as_str() {
            "pending" => false,
            "uncertain" => true,
            _ => return Ok(0),
        };
        let permission_targets=drive_core::write_permissions::namespace_targets("mkdir",&serde_json::to_string(&q.job)?).map_err(io::Error::other)?;
        let op = q.job.operation.clone();
        let job = if reconcile {
            q.job
        } else {
            match self.store.lock().unwrap().mark_directory_started(&op) {
                Ok(job) => job,
                Err(_) => return Ok(0),
            }
        };
        let outcome = rpc.call(Command::Mkdir { job, reconcile }, Duration::from_secs(120));
        let mut s = self.store.lock().unwrap();
        let mut unknown_category = CommitCategory::Protocol;
        let mut unknown_error = "server returned an unexpected directory reply";
        match outcome {
            Ok(reply) => match reply.body {
                Response::DirectoryPrepared { origin } => {
                    s.prepare_directory(&op, origin).map_err(io::Error::other)?;
                    return Ok(1);
                }
                Response::Created { node, remote } => {
                    if s.confirm_directory(&op, &node, &remote).is_ok() {
                        drive_core::write_permissions::observe_targets(&s,&permission_targets,true).map_err(io::Error::other)?;
                        return Ok(1);
                    }
                }
                Response::PermissionDenied{node,action} if !reconcile => {
                    drive_core::write_permissions::refuse_operation(&s,&op,true,&permission_targets,&node,action).map_err(io::Error::other)?;return Ok(0);
                }
                Response::MutationPermissionUncertain { code } if !reconcile => {
                    drive_core::write_permissions::mutation_permission_uncertain(&s,&op,true,code).map_err(io::Error::other)?;return Ok(0);
                }
                Response::Conflict => {
                    s.conflict_directory(&op).map_err(io::Error::other)?;
                    return Ok(0);
                }
                _ => {}
            },
            Err(_) => {
                unknown_category = CommitCategory::Transport;
                unknown_error = "directory transport lost before the commit was observed";
            }
        }
        if !reconcile {
            s.mark_directory_uncertain_diagnosed(&op, unknown_category, Some(unknown_error))
                .map_err(io::Error::other)?;
        } else {
            s.record_namespace_commit_diagnostic(
                &op,
                unknown_category,
                Some(unknown_error),
                0,
            )
            .map_err(io::Error::other)?;
        }
        Ok(0)
    }
}
