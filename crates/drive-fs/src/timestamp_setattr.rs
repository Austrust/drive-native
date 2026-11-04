//! File mtime uses the ordinary hydrated, durable upload path; atime is local.
use super::*;

fn times(
    atime: Option<TimeOrNow>,
    mtime: Option<TimeOrNow>,
    now: SystemTime,
) -> drive_core::Result<(Option<(u64, u32)>, Option<u32>)> {
    let resolve = |value| match value {
        TimeOrNow::SpecificTime(t) => t,
        TimeOrNow::Now => now,
    };
    Ok((
        atime
            .map(|t| drive_core::timestamps::local_time(resolve(t)))
            .transpose()?,
        mtime
            .map(|t| drive_core::timestamps::native_mtime(resolve(t)))
            .transpose()?,
    ))
}
pub(super) fn set_times(
    fs: &mut CacheFilesystem,
    ino: u64,
    fh: Option<u64>,
    atime: Option<TimeOrNow>,
    mtime: Option<TimeOrNow>,
    reply: ReplyAttr,
) {
    let (atime, mtime) = match times(atime, mtime, SystemTime::now()) {
        Ok(v) => v,
        Err(e) => {
            reply.error(errno(&e));
            return;
        }
    };
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let notify = fs.notify.clone();
    let updates = fs.handle_updates.clone();
    let hydrator = fs.hydrator.clone();
    let network = fs.network_queue.clone();
    let (uid, gid) = (fs.uid, fs.gid);
    fs.queue.submit(Box::new(move || {
        let held = if let Some(fh) = fh {
            match handles.lock().unwrap().entries.get(&fh) {
                Some(h) if h.ino == ino => Some((h.node.clone(), h.content.clone())),
                _ => {
                    reply.error(libc::EBADF);
                    return;
                }
            }
        } else {
            None
        };
        let resolved = (|| -> drive_core::Result<_> {
            let mut s = store.lock().unwrap();
            let record = preview::authoritative(&s, ino, libc::EROFS)?;
            permissions::node(&s,&record.node.id,permissions::Action::Setattr)?;
            if mtime.is_some() && record.node.directory {
                return Err(Error::Io(std::io::Error::from_raw_os_error(
                    libc::EOPNOTSUPP,
                )));
            }
            if mtime.is_none() {
                if let Some((sec, nsec)) = atime {
                    s.set_local_atime(&record.node.id, sec, nsec)?;
                }
                return Ok((record.node.id, None));
            }
            let (id, body) = match held {
                Some(h) => h,
                None => content(&s, ino, libc::EROFS)?,
            };
            if id != record.node.id {
                return Err(Error::Integrity);
            }
            Ok((id, Some(body)))
        })();
        let (id, body) = match resolved {
            Ok(v) => v,
            Err(e) => {
                reply.error(errno(&e));
                return;
            }
        };
        let Some(body) = body else {
            match store.lock().unwrap().record_for_handle(&id) {
                Ok(record) => {
                    notify.invalidate_inode(record.ino);
                    reply.attr(&TTL, &attr(&record, uid, gid))
                }
                Err(e) => reply.error(errno(&e)),
            }
            return;
        };
        let missing = matches!(body, OpenContent::Missing(_));
        let job: Job = Box::new(move || {
            let result =
                draft(&store, &handles, &updates, &hydrator, &id, body, false, permissions::Action::Setattr).and_then(|d| {
                    let mut s = store.lock().unwrap();
                    permissions::node(&s,&id,permissions::Action::Setattr).map_err(|e|errno(&e))?;
                    // Recheck access after network hydration and before mutation.
                    s.record_for_handle(&id).map_err(|e| errno(&e))?;
                    s.draft_set_mtime(&d, mtime.unwrap())
                        .map_err(|e| errno(&e))?;
                    s.checkpoint_draft(&d).map_err(|e| errno(&e))?;
                    s.creation_saved(&id).map_err(|e| errno(&e))?;
                    if let Some((sec, nsec)) = atime {
                        s.set_local_atime(&id, sec, nsec).map_err(|e| errno(&e))?;
                    }
                    s.record_for_handle(&id).map_err(|e| errno(&e))
                });
            match result {
                Ok(record) => {
                    if let Some(h) = hydrator {
                        h.local_saved();
                    }
                    notify.invalidate_inode(record.ino);
                    reply.attr(&TTL, &attr(&record, uid, gid));
                }
                Err(e) => reply.error(e),
            }
        });
        if missing { network.submit(job) } else { job() }
    }));
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn omitted_times_stay_absent_and_now_is_one_instant() {
        let now = UNIX_EPOCH + Duration::new(123, 456);
        assert_eq!(times(None, None, now).unwrap(), (None, None));
        assert_eq!(
            times(Some(TimeOrNow::Now), Some(TimeOrNow::Now), now).unwrap(),
            (Some((123, 456)), Some(123))
        );
    }
    #[test]
    fn invalid_mtime_rejects_combined_request_before_atime_mutation() {
        let now = UNIX_EPOCH + Duration::from_secs(123);
        assert!(
            times(
                Some(TimeOrNow::Now),
                Some(TimeOrNow::SpecificTime(UNIX_EPOCH - Duration::from_secs(1))),
                now
            )
            .is_err()
        );
        assert!(
            times(
                None,
                Some(TimeOrNow::SpecificTime(
                    UNIX_EPOCH + Duration::from_secs(u64::from(u32::MAX) + 1)
                )),
                now
            )
            .is_err()
        );
    }
}
