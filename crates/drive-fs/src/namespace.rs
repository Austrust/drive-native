use super::*;
pub(super) fn rename(
    fs: &mut CacheFilesystem,
    parent: u64,
    name: &OsStr,
    new_parent: u64,
    new_name: &OsStr,
    flags: u32,
    reply: ReplyEmpty,
) {
    if !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    if flags & !(libc::RENAME_NOREPLACE as u32) != 0 {
        reply.error(libc::EOPNOTSUPP);
        return;
    }
    let (Some(name), Some(new_name)) = (
        name.to_str().map(str::to_owned),
        new_name.to_str().map(str::to_owned),
    ) else {
        reply.error(libc::EINVAL);
        return;
    };
    // Resolve under the state lock before choosing the operation. Replacement
    // hydration runs on its own queue, never on the FUSE dispatch thread.
    let store = fs.store.clone();
    let hydrator = fs.hydrator.clone();
    let handles = fs.handles.clone();
    let updates = fs.handle_updates.clone();
    let network = fs.network_queue.clone();
    fs.queue.submit(Box::new(move || {
        let result = (|| -> drive_core::Result<Option<(String, String, String, String)>> {
            let mut s = store.lock().unwrap();
            let p = s.record_by_inode(parent)?.node.id;
            let np = s.record_by_inode(new_parent)?.node.id;
            let source = s.lookup(&p, &name)?.node.id;
            match s.lookup(&np, &new_name) {
                Ok(target) if target.node.id != source && flags == 0 => {
                    Ok(Some((p, np, source, target.node.id)))
                }
                _ => {
                    s.rename_local(&p, &name, &np, &new_name, flags != 0)?;
                    Ok(None)
                }
            }
        })();
        match result {
            Ok(Some((p, np, source, target))) => replace_prepared(
                store, handles, updates, hydrator, network, p, name, np, new_name, source, target,
                reply,
            ),
            Ok(None) => {
                if let Some(h) = hydrator {
                    h.local_saved();
                }
                reply.ok()
            }
            Err(e) => reply.error(errno(&e)),
        }
    }));
}

pub(super) fn remove(
    fs: &mut CacheFilesystem,
    parent: u64,
    name: &OsStr,
    directory: bool,
    reply: ReplyEmpty,
) {
    if !fs.writable {
        reply.error(libc::EROFS);
        return;
    }
    let Some(name) = name.to_str().map(str::to_owned) else {
        reply.error(libc::EINVAL);
        return;
    };
    let store = fs.store.clone();
    let handles = fs.handles.clone();
    let updates = fs.handle_updates.clone();
    let hydrator = fs.hydrator.clone();
    let network = fs.network_queue.clone();
    fs.queue.submit(Box::new(move|| {
        let identity=(||->drive_core::Result<(String,String)>{
            let s=store.lock().unwrap();let parent=s.record_by_inode(parent)?.node.id;let id=s.lookup(&parent,&name)?.node.id;Ok((parent,id))
        })();
        let (parent,id)=match identity {Ok(v)=>v,Err(e)=>{reply.error(errno(&e));return;}};
        network.submit(Box::new(move|| {
            let result=(||->Result<(),i32>{
                loop {
                    // Freeze handle registration/promotion without obstructing
                    // cached reads. Never hold the table while entering Store.
                    let update=updates.lock().unwrap();
                    let current=store.lock().unwrap().lookup(&parent,&name).map_err(|e|errno(&e))?;
                    if current.node.id!=id {return Err(libc::EAGAIN);}
                    let missing=handles.lock().unwrap().entries.values().filter(|h|h.node==id).find_map(|h|match &h.content{OpenContent::Missing(n)=>Some(n.clone()),_=>None});
                    let Some(missing)=missing else {
                        store.lock().unwrap().unlink_local(&parent,&name,&id,directory).map_err(|e|errno(&e))?;
                        return Ok(());
                    };
                    drop(update);
                    // Only an already-open missing handle needs hydration before
                    // deletion. Removing an unopened file downloads no body.
                    let content=hydrator.as_ref().ok_or(libc::ENETDOWN)?.hydrate(&missing)?;
                    let mut h=handles.lock().unwrap();
                    for entry in h.entries.values_mut().filter(|h|h.node==id) {
                        if matches!(&entry.content,OpenContent::Missing(n) if n.version==missing.version) {
                            entry.content=OpenContent::Cached(content.clone());
                        }
                    }
                }
            })();
            match result {Ok(())=>{if let Some(h)=hydrator{h.local_saved();}reply.ok()},Err(e)=>reply.error(e)}
        }));
    }));
}

fn replace_prepared(
    store: Arc<drive_core::SharedStore>,
    handles: Arc<Mutex<Handles>>,
    updates: Arc<Mutex<()>>,
    hydrator: Option<Arc<dyn Hydrator>>,
    network: Arc<LocalQueue>,
    parent: String,
    name: String,
    new_parent: String,
    new_name: String,
    source: String,
    target: String,
    reply: ReplyEmpty,
) {
    network.submit(Box::new(move||{
        let result=(||->Result<(),i32>{loop{
            let update=updates.lock().unwrap();
            let (src,dst)={let s=store.lock().unwrap();(s.lookup(&parent,&name).map_err(|e|errno(&e))?,s.lookup(&new_parent,&new_name).map_err(|e|errno(&e))?)};
            if src.node.id!=source || dst.node.id!=target{return Err(libc::EAGAIN);}
            if src.node.directory || dst.node.directory{return Err(if src.node.directory&&!dst.node.directory{libc::ENOTDIR}else if !src.node.directory&&dst.node.directory{libc::EISDIR}else{libc::EOPNOTSUPP});}
            let missing=handles.lock().unwrap().entries.values().filter(|h|h.node==source||h.node==target).find_map(|h|match &h.content{OpenContent::Missing(n)=>Some(n.clone()),_=>None});
            let mut s=store.lock().unwrap();
            let missing=match missing {
                Some(n)=>Some(n),
                None if s.open_dirty_draft(&source).map_err(|e|errno(&e))?.is_some()=>None,
                None=>match s.open_cached(&source){Ok(_)=>None,Err(Error::Offline)=>Some(src.node.clone()),Err(e)=>return Err(errno(&e))},
            };
            let Some(missing)=missing else {
                s.replace_local(&parent,&name,&new_parent,&new_name,&source,&target).map_err(|e|errno(&e))?;return Ok(());
            };
            drop(s);drop(update);
            let body=hydrator.as_ref().ok_or(libc::ENETDOWN)?.hydrate(&missing)?;
            for entry in handles.lock().unwrap().entries.values_mut(){
                if entry.node==missing.id && matches!(&entry.content,OpenContent::Missing(n) if n.version==missing.version){entry.content=OpenContent::Cached(body.clone());}
            }
        }})();
        match result{Ok(())=>{if let Some(h)=hydrator{h.local_saved();}reply.ok()},Err(e)=>reply.error(e)}
    }));
}
