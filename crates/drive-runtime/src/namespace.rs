use super::*;
impl Bridge {
    /// Process the durable sequence. An uncertain head is queried, never resent.
    pub fn rename_pending(&self) -> io::Result<usize> {
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
                .next_rename()
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
            let op = queued.job.operation.clone();
            let fresh = self
                .store
                .lock()
                .unwrap()
                .next_rename()
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
                            let _ = self.notifications.try_send(Invalidation::Inode(record.ino));
                        }
                        done += 1;
                    }
                    Response::Conflict => {
                        store.conflict_rename(&op).map_err(io::Error::other)?;
                        return Ok(done);
                    }
                    _ => {
                        if !reconcile {
                            store.mark_rename_uncertain(&op).map_err(io::Error::other)?;
                        }
                        return Ok(done);
                    }
                },
                Err(_) => {
                    if !reconcile {
                        store.mark_rename_uncertain(&op).map_err(io::Error::other)?;
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
    /// Both operation kinds obey the same journal sequence. No uncertain head
    /// is retransmitted, and a conflict remains visible instead of being skipped.
    pub fn namespace_pending(&self) -> io::Result<usize> {
        let _mutation = self.recovery_gate.read().unwrap();
        let mut done = 0;
        loop {
            let kind = self
                .store
                .lock()
                .unwrap()
                .next_namespace_kind()
                .map_err(io::Error::other)?;
            let count = match kind.as_deref() {
                None => return Ok(done),
                Some("rename") => self.rename_pending()?,
                Some("remove") => self.remove_pending()?,
                Some("replace") => self.replace_pending()?,
                Some("mkdir") => self.directory_pending()?,
                _ => return Err(io::Error::other("unknown namespace journal operation")),
            };
            done += count;
            if count == 0 {
                return Ok(done);
            }
        }
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
            .next_remove()
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
                    Ok(1)
                }
                Response::Conflict => {
                    store.conflict_remove(&op).map_err(io::Error::other)?;
                    Ok(0)
                }
                _ => {
                    if !reconcile {
                        store.mark_remove_uncertain(&op).map_err(io::Error::other)?;
                    }
                    Ok(0)
                }
            },
            Err(_) => {
                if !reconcile {
                    store.mark_remove_uncertain(&op).map_err(io::Error::other)?;
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
            .next_replace()
            .map_err(io::Error::other)?
        else {
            return Ok(0);
        };
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
                        .next_replace()
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
        let result = match outcome {
            Ok(reply) => match reply.body {
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
            Err(_) => None,
        };
        match result {
            Some(Ok(())) => Ok(1),
            _ => {
                if !reconcile {
                    s.mark_replace_uncertain(&op).map_err(io::Error::other)?;
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
            .next_directory()
            .map_err(io::Error::other)?
        else {
            return Ok(0);
        };
        let reconcile = match q.state.as_str() {
            "pending" => false,
            "uncertain" => true,
            _ => return Ok(0),
        };
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
        match outcome {
            Ok(reply) => match reply.body {
                Response::DirectoryPrepared { origin } => {
                    s.prepare_directory(&op, origin).map_err(io::Error::other)?;
                    return Ok(1);
                }
                Response::Created { node, remote } => {
                    if s.confirm_directory(&op, &node, &remote).is_ok() {
                        return Ok(1);
                    }
                }
                Response::Conflict => {
                    s.conflict_directory(&op).map_err(io::Error::other)?;
                    return Ok(0);
                }
                _ => {}
            },
            Err(_) => {}
        }
        if !reconcile {
            s.mark_directory_uncertain(&op).map_err(io::Error::other)?;
        }
        Ok(0)
    }
}
