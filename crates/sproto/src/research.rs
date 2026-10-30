//! Static reconstructions for offline replay. These do not open a channel.
//! Binary sample: ea01d634762060424f4a5fd3fccb3f5c0ba7c10a81e90e144706fb2b8406314a.
//! Source: LongPoller::LongPoll (0x5cc70), PullEventHandler::PullEventFromServer (0x14cb10).
//! Neither response cursors nor write-confirmation semantics are complete.
use crate::{Error, pstream::PObject};

pub struct Subscription<'a> {
    pub sync_id: u64,
    pub path: &'a str,
    pub view_id: u64,
    pub node_id: u64,
    pub recursive: bool,
    pub locks: Option<(u64, u64)>,
}

/// Constructs only the recovered fields. The native envelope/SCMD and caller's
/// interpretation of `changes` still need independent recorded-message validation.
pub fn long_poll_body(subscriptions: &[Subscription<'_>], profile_digest: &str) -> PObject {
    let subscriptions = subscriptions
        .iter()
        .map(|s| {
            let mut o = pmap! {"sync_id"=>s.sync_id,"path"=>s.path,"view_id"=>s.view_id,
            "node_id"=>s.node_id,"recursive"=>s.recursive};
            if let Some((lock, options)) = s.locks {
                let map = o.as_map_mut().unwrap();
                map.insert("lock_id".into(), PObject::Integer(lock));
                map.insert("option_change_lock_id".into(), PObject::Integer(options));
            }
            o
        })
        .collect();
    // 0x5cd9e: mov esi,0x190; assignment to timeout. Units are not asserted here.
    pmap! {"_action"=>"long_poll","timeout"=>400u64,"subscribe"=>PObject::Array(subscriptions),"profile_digest"=>profile_digest}
}

/// `changes` is an array of unsigned tokens in the observed implementation.
/// Do not interpret these tokens as per-file IDs or durable event cursors yet.
pub fn changed_subscription_tokens(response: &PObject) -> Result<Vec<u64>, Error> {
    let changes = response
        .get("changes")
        .and_then(PObject::as_array)
        .ok_or_else(|| Error::Decode("missing long_poll changes array".into()))?;
    changes
        .iter()
        .map(|v| {
            v.as_int()
                .ok_or_else(|| Error::Decode("invalid long_poll change token".into()))
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn reconstructed_body_preserves_subscription_identity() {
        let p = long_poll_body(
            &[Subscription {
                sync_id: 8,
                path: "",
                view_id: 7,
                node_id: 19,
                recursive: true,
                locks: None,
            }],
            "digest",
        );
        assert_eq!(p.get("timeout").and_then(PObject::as_int), Some(400));
        let s = &p.get("subscribe").unwrap().as_array().unwrap()[0];
        assert_eq!(s.get("sync_id").and_then(PObject::as_int), Some(8));
        assert!(s.get("lock_id").is_none());
    }
    #[test]
    fn notification_is_not_accepted_as_a_file_event_batch() {
        assert_eq!(
            changed_subscription_tokens(
                &pmap! {"changes"=>PObject::Array(vec![PObject::Integer(7)])}
            )
            .unwrap(),
            vec![7]
        );
        assert!(
            changed_subscription_tokens(
                &pmap! {"changes"=>PObject::Array(vec![pmap!{"file_id"=>"fake"}])}
            )
            .is_err()
        );
    }
}
