//! Native read contracts validated against Drive Server protocol build 17892.
//! Event cursors and content versions remain separate: a deletion advances sync_id
//! but its file.max_id can remain unchanged. Notifications are view IDs, not events.
use crate::{Client, Error, Result, frame, pstream::PObject};
use std::time::Duration;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EventCursor(pub u64);

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct NativeVersion {
    pub sync_id: u64,
    pub max_id: u64,
}

#[derive(Clone, Debug)]
pub struct PersonalRoot {
    pub view_id: u64,
    pub node_id: u64,
    pub writable: bool,
    pub download_allowed: bool,
}

#[derive(Clone, Debug)]
pub struct Watch {
    pub view_id: u64,
    pub node_id: u64,
    pub path: String,
    pub cursor: EventCursor,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum EventKind {
    Modify,
    Move {
        destination: String,
        parent_id: String,
    },
    Remove,
}

#[derive(Clone, Debug)]
pub struct NativeEvent {
    /// One-way fingerprint only; raw event session credentials never escape decoding.
    pub committer: Option<String>,
    pub kind: EventKind,
    pub path: String,
    pub file_id: String,
    pub parent_id: String,
    pub version: NativeVersion,
    pub file_type: super::list::FileType,
    pub size: u64,
    pub hash: String,
    pub mtime: u64,
    pub removed: bool,
}

#[derive(Debug)]
pub struct EventBatch {
    pub events: Vec<NativeEvent>,
    pub next: EventCursor,
    /// Rescan/merge signals must be handled before advancing any durable cursor.
    pub recovery_required: bool,
}

#[derive(Debug)]
pub struct Notification {
    pub changed_views: Vec<u64>,
    pub profile_changed: bool,
    pub profile_digest: String,
}

fn integer(o: &PObject, key: &str) -> Result<u64> {
    o.get(key)
        .and_then(PObject::as_int)
        .ok_or_else(|| Error::Decode(format!("missing native integer: {key}")))
}
fn string(o: &PObject, key: &str) -> Result<String> {
    o.get(key)
        .and_then(PObject::as_str)
        .map(str::to_owned)
        .ok_or_else(|| Error::Decode(format!("missing native string: {key}")))
}
fn identity(o: &PObject, key: &str) -> Result<String> {
    match o.get(key) {
        Some(PObject::Integer(n)) => Ok(n.to_string()),
        Some(PObject::Str(s)) if !s.is_empty() => Ok(s.clone()),
        _ => Err(Error::Decode(format!("missing native identity: {key}"))),
    }
}
pub(crate) fn validate_path(path: &str) -> Result<()> {
    if path == "/" {
        return Ok(());
    }
    if !path.starts_with('/')
        || path.contains('\0')
        || path[1..].split('/').any(|s| matches!(s, "" | "." | ".."))
    {
        return Err(Error::InvalidConfig("invalid absolute native path".into()));
    }
    Ok(())
}

impl Client {
    /// Query the personal root explicitly; team-folder lists may omit My Drive.
    pub async fn personal_root(&self) -> Result<PersonalRoot> {
        let mut guard = self.pool.acquire().await?;
        let req = self.build_request("query_user_info", pmap! {"get_view" => true});
        let result = async {
            let r = guard.channel().request(frame::SCMD_REQUEST, &req).await?;
            parse_personal_root(&r)
        }
        .await;
        guard.poison_on_err(result)
    }

    /// Pull incremental changes. Persist the whole batch before committing `next`.
    /// Initial cursor zero is observed to return current nodes including the root;
    /// concurrent snapshot/bootstrap and cursor-reset cases still require validation.
    pub async fn pull_events(&self, watch: &Watch) -> Result<EventBatch> {
        validate_path(&watch.path)?;
        let req = self.build_request(
            "pull_event",
            pmap! {
                "view_id" => watch.view_id, "node_id" => watch.node_id,
                "path" => watch.path.as_str(), "sync_id" => watch.cursor.0,
                "recursive" => true, "offset" => 0u64,
                "file" => true, "mtime" => true, "execbit" => true,
            },
        );
        let mut guard = self.pool.acquire().await?;
        let result = async {
            let response = guard.channel().request(frame::SCMD_REQUEST, &req).await?;
            parse_events(&response, watch.cursor)
        }
        .await;
        guard.poison_on_err(result)
    }

    /// A server-side blocking notification request; no directory polling.
    /// Cancellation drops its channel so a delayed response cannot pollute the pool.
    pub async fn wait_changes(&self, watches: &[Watch], digest: &str) -> Result<Notification> {
        if watches.is_empty() {
            return Err(Error::InvalidConfig("empty watch set".into()));
        }
        let mut subscriptions = Vec::new();
        for w in watches {
            validate_path(&w.path)?;
            subscriptions.push(pmap! {"view_id" => w.view_id,"node_id" => w.node_id,
            "path" => w.path.as_str(),"sync_id" => w.cursor.0,"recursive" => true});
        }
        let request = self.build_request(
            "long_poll",
            pmap! {
                "timeout" => 400u64,"subscribe" => subscriptions,"profile_digest" => digest,
            },
        );
        let mut guard = self.pool.acquire().await?;
        let result = async {
            let r = guard
                .channel()
                .request_with_timeout(frame::SCMD_REQUEST, &request, Duration::from_secs(450))
                .await?;
            parse_notification(&r, digest)
        }
        .await;
        guard.poison_on_err(result)
    }
}

fn parse_personal_root(r: &PObject) -> Result<PersonalRoot> {
    if integer(r, "enable_mydrive")? != 1 {
        return Err(Error::InvalidConfig("personal Drive disabled".into()));
    }
    let views = r
        .get("view_list")
        .and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing personal view list".into()))?;
    let mut homes = views
        .iter()
        .filter(|v| v.get("name").and_then(PObject::as_str) == Some("home"));
    let home = homes
        .next()
        .ok_or_else(|| Error::Decode("personal home view missing".into()))?;
    if homes.next().is_some() {
        return Err(Error::Decode("ambiguous personal home view".into()));
    }
    let privilege = string(home, "privilege")?;
    if !matches!(privilege.as_str(), "read-write" | "read-only") {
        return Err(Error::InvalidConfig(
            "personal Drive permission denied".into(),
        ));
    }
    Ok(PersonalRoot {
        view_id: integer(home, "view_id")?,
        node_id: integer(home, "node_id")?,
        writable: privilege == "read-write",
        download_allowed: integer(home, "disable_download")? == 0,
    })
}

fn parse_notification(r: &PObject, previous_digest: &str) -> Result<Notification> {
    let changes = r
        .get("changes")
        .and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing notification changes".into()))?;
    let profile_changed = match r.get("profile_changed") {
        None => false,
        Some(PObject::Integer(0)) => false,
        Some(PObject::Integer(1)) => true,
        _ => return Err(Error::Decode("invalid profile_changed".into())),
    };
    let profile_digest = match r.get("profile_digest") {
        Some(PObject::Str(s)) => s.clone(),
        None if !profile_changed => previous_digest.to_owned(),
        _ => return Err(Error::Decode("missing changed profile digest".into())),
    };
    Ok(Notification {
        changed_views: changes
            .iter()
            .map(|c| {
                c.as_int()
                    .ok_or_else(|| Error::Decode("invalid changed view".into()))
            })
            .collect::<Result<_>>()?,
        profile_changed,
        profile_digest,
    })
}

fn parse_events(r: &PObject, cursor: EventCursor) -> Result<EventBatch> {
    let next = integer(r, "next_sync_id")?;
    if next < cursor.0 {
        return Err(Error::CursorReset {
            requested: cursor.0,
            available: next,
        });
    }
    let list = r
        .get("event_list")
        .and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing event list".into()))?;
    let mut events = Vec::new();
    for e in list {
        let path = string(e, "path")?;
        validate_path(&path)?;
        let kind = match string(e, "type")?.as_str() {
            "modify" => EventKind::Modify,
            "remove" => EventKind::Remove,
            "move" => {
                let d = string(
                    e.get("destination")
                        .ok_or_else(|| Error::Decode("move destination missing".into()))?,
                    "path",
                )?;
                validate_path(&d)?;
                let parents = e
                    .get("destination")
                    .and_then(|d| d.get("parent_ids"))
                    .and_then(PObject::as_array)
                    .ok_or_else(|| Error::Decode("move destination parents missing".into()))?;
                let parent_id = match parents.last() {
                    Some(PObject::Integer(n)) => n.to_string(),
                    Some(PObject::Str(s)) if !s.is_empty() => s.clone(),
                    _ => return Err(Error::Decode("move destination parent missing".into())),
                };
                EventKind::Move {
                    destination: d,
                    parent_id,
                }
            }
            _ => {
                return Err(Error::Decode(
                    "unknown event kind; cursor not advanced".into(),
                ));
            }
        };
        let f = e
            .get("file")
            .ok_or_else(|| Error::Decode("event file metadata missing".into()))?;
        let sync_id = integer(e, "sync_id")?;
        if sync_id > next {
            return Err(Error::Decode("event beyond returned cursor".into()));
        }
        let file_type = match string(f, "file_type")?.as_str() {
            "dir" => super::list::FileType::Dir,
            "file" => super::list::FileType::File,
            "symlink" => super::list::FileType::Symlink,
            _ => return Err(Error::Decode("unsupported event file type".into())),
        };
        let removed = integer(f, "is_removed")? != 0;
        // Historical move/modify events can carry CURRENT tombstone metadata.
        // Live replay after delete showed a move at 11 with is_removed=1 followed
        // by remove at 12. Do not assume event payloads are immutable by sync_id.
        if matches!(kind, EventKind::Remove) && !removed {
            return Err(Error::Decode("remove event lacks tombstone".into()));
        }
        events.push(NativeEvent {
            committer: parse_committer(e)?,
            kind,
            path,
            file_id: identity(e, "file_id")?,
            parent_id: if removed { String::new() } else { identity(f, "parent_id")? },
            version: NativeVersion {
                sync_id,
                max_id: integer(f, "max_id")?,
            },
            file_type,
            size: if removed { 0 } else { integer(f, "file_size")? },
            hash: if removed { String::new() } else { string(f, "file_hash")? },
            mtime: if removed { 0 } else { integer(f, "mtime")? },
            removed,
        });
    }
    Ok(EventBatch {
        events,
        next: EventCursor(next),
        recovery_required: ["rescan_later", "merge_mode"]
            .iter()
            .any(|key| !matches!(r.get(key), None | Some(PObject::Integer(0)))),
    })
}

fn parse_committer(event: &PObject) -> Result<Option<String>> {
    fn parse(value: Option<&PObject>) -> Result<Option<&str>> {
        match value {
            None | Some(PObject::Null) => Ok(None),
            Some(PObject::Str(s)) if s.is_empty() => Ok(None),
            Some(PObject::Str(s)) if s.len() <= 8192 => Ok(Some(s)),
            _ => Err(Error::Decode("invalid event committer identity".into())),
        }
    }
    let flat = parse(event.get("committer_session_id"))?;
    let nested = parse(event.get("committer").and_then(|x| x.get("session_id")))?;
    if let (Some(a), Some(b)) = (flat, nested) {
        if a != b {
            return Err(Error::Decode(
                "conflicting event committer identities".into(),
            ));
        }
    }
    // Server/filesystem-originated events can be anonymous. They still update
    // ordinary metadata, but can never confirm this client's uncertain mkdir.
    Ok(flat.or(nested).map(crate::client::session_fingerprint))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn committer_fingerprint_never_exposes_token_and_rejects_disagreement() {
        let token = "test-only-random-looking-session";
        let flat = pmap! {"committer_session_id"=>token};
        let nested = pmap! {"committer"=>pmap! {"session_id"=>token}};
        let fingerprint = parse_committer(&flat).unwrap().unwrap();
        assert_eq!(fingerprint.len(), 64);
        assert!(!fingerprint.contains(token));
        assert_eq!(parse_committer(&nested).unwrap(), Some(fingerprint.clone()));
        assert!(
            parse_committer(
                &pmap! {"committer_session_id"=>token,"committer"=>pmap! {"session_id"=>"other"}}
            )
            .is_err()
        );
        assert!(parse_committer(&pmap! {"committer_session_id"=>0u64}).is_err());
        assert_eq!(parse_committer(&pmap! {}).unwrap(), None);
        assert_eq!(
            parse_committer(&pmap! {"committer_session_id"=>""}).unwrap(),
            None
        );
        assert_eq!(
            parse_committer(&pmap! {"committer_session_id"=>PObject::Null}).unwrap(),
            None
        );
    }
    fn object(v: serde_json::Value) -> PObject {
        match v {
            serde_json::Value::Null => PObject::Null,
            serde_json::Value::Bool(b) => b.into(),
            serde_json::Value::Number(n) => n.as_u64().unwrap().into(),
            serde_json::Value::String(s) => s.into(),
            serde_json::Value::Array(a) => PObject::Array(a.into_iter().map(object).collect()),
            serde_json::Value::Object(m) => {
                PObject::Map(m.into_iter().map(|(k, v)| (k, object(v))).collect())
            }
        }
    }
    #[test]
    fn replay_recorded_native_events_and_notifications() {
        let mut kinds = [false; 3];
        let mut batches = 0;
        let mut notifications = 0;
        for line in include_str!("../../../../evidence/native-live.jsonl").lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            if !v["response"]["error"].is_null() {
                continue;
            }
            let r = object(v["response"].clone());
            match v["stage"].as_str().unwrap() {
                "pull_event" => {
                    let b =
                        parse_events(&r, EventCursor(v["request"]["sync_id"].as_u64().unwrap()))
                            .unwrap();
                    for e in b.events {
                        kinds[match e.kind {
                            EventKind::Modify => 0,
                            EventKind::Move { .. } => 1,
                            EventKind::Remove => 2,
                        }] = true;
                    }
                    batches += 1;
                }
                "long_poll" => {
                    let n =
                        parse_notification(&r, v["request"]["profile_digest"].as_str().unwrap())
                            .unwrap();
                    assert!(n.changed_views.iter().all(|id| *id == 54));
                    notifications += 1;
                }
                _ => {}
            }
        }
        assert!(batches >= 5 && notifications >= 1);
        assert_eq!(kinds, [true; 3]);
    }
    #[test]
    fn fail_closed_on_missing_or_regressing_cursor() {
        assert!(
            parse_events(
                &pmap! {"event_list"=>PObject::Array(vec![])},
                EventCursor(0)
            )
            .is_err()
        );
        assert!(
            parse_events(
                &pmap! {"event_list"=>PObject::Array(vec![]),"next_sync_id"=>3u64},
                EventCursor(4)
            )
            .is_err()
        );
    }
    #[test]
    fn replay_live_future_cursor_response_requires_explicit_recovery() {
        let mut checked = false;
        for line in include_str!("../../../../evidence/native-cursor-reset-live.jsonl").lines() {
            let v: serde_json::Value = serde_json::from_str(line).unwrap();
            if v["stage"] == "pull_event" && v["request"]["sync_id"] == 999999 {
                assert!(matches!(
                    parse_events(&object(v["response"].clone()), EventCursor(999999)),
                    Err(Error::CursorReset {
                        requested: 999999,
                        available: 29
                    })
                ));
                checked = true;
            }
        }
        assert!(checked);
    }
    #[test]
    fn scope_paths_are_component_checked() {
        for p in ["", "relative", "/x/../y", "/x//y", "/x/", "/x\0"] {
            assert!(validate_path(p).is_err());
        }
        for p in ["/", "/home", "/中文/a.doc"] {
            validate_path(p).unwrap();
        }
    }
    #[test]
    fn unknown_recovery_signal_does_not_advance_normally() {
        for value in [PObject::from(1u64), PObject::from("unknown"), PObject::Null] {
            let r = pmap! {"event_list"=>PObject::Array(vec![]),"next_sync_id"=>1u64,"merge_mode"=>value};
            assert!(parse_events(&r, EventCursor(0)).unwrap().recovery_required);
        }
    }
}

#[cfg(test)]mod minimal_tombstones {
 use super::*;
 #[test]fn deleted_content_may_be_pruned_but_live_metadata_remains_mandatory(){
  for removed in [0u64,1u64]{
   let r=pmap!{"next_sync_id"=>22675u64,"event_list"=>PObject::Array(vec![pmap!{"type"=>"remove","path"=>"/old.txt","file_id"=>"42","sync_id"=>22675u64,"file"=>pmap!{"file_type"=>"file","is_removed"=>removed,"max_id"=>7u64}}])};
   let v=parse_events(&r,EventCursor(22000));
   if removed==1 {let e=v.unwrap().events.remove(0);assert!(e.removed);assert_eq!(e.file_id,"42");}
   else {assert!(v.is_err());}
  }
  let r=pmap!{"next_sync_id"=>22675u64,"event_list"=>PObject::Array(vec![pmap!{"type"=>"modify","path"=>"/live.txt","file_id"=>"42","sync_id"=>22675u64,"file"=>pmap!{"file_type"=>"file","is_removed"=>0u64,"max_id"=>7u64}}])};
  assert!(parse_events(&r,EventCursor(22000)).is_err());
 }
}
