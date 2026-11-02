//! Bounded, content-free observations. Timings never alter protocol deadlines.
//! Task-local scopes are not inherited by spawned tasks: scope each worker request.
use crate::{Error, Result, pstream::PObject};
use serde::{Deserialize, Serialize};
use std::{
    future::Future,
    sync::{Arc, Mutex},
    time::Instant,
};

#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum RequestKind {
    Auth,
    ServerInfo,
    Connect,
    List,
    ListSync,
    TeamCatalog,
    UserInfo,
    Watermark,
    PullEvents,
    LongPoll,
    Download,
    Other,
}
impl RequestKind {
    pub(crate) fn of(object: &PObject) -> Self {
        match object.get("_action").and_then(PObject::as_str) {
            Some("auth") => Self::Auth,
            Some("query_server_info") => Self::ServerInfo,
            Some("connect") => Self::Connect,
            Some("list") => Self::List,
            Some("list_sync_to_device") => Self::ListSync,
            Some("list_team_folder") => Self::TeamCatalog,
            Some("query_user_info") => Self::UserInfo,
            Some("get_max_sync_id") => Self::Watermark,
            Some("pull_event") => Self::PullEvents,
            Some("long_poll") => Self::LongPoll,
            Some("download") => Self::Download,
            _ => Self::Other,
        }
    }
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestPhase {
    Starting,
    Send,
    Receive,
}
#[derive(Clone, Copy, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RequestOutcome {
    Success,
    Cancelled,
    ResponseTimeout,
    ReceiveIdleTimeout,
    SendTimeout,
    SetupTimeout,
    OtherTimeout,
    Transport,
    Decode,
    ServerRejected,
    SessionInvalid,
    Configuration,
    CursorReset,
}
impl RequestOutcome {
    fn of<T>(result: &Result<T>) -> Self {
        match result {
            Ok(_) => Self::Success,
            Err(Error::Timeout("native response")) => Self::ResponseTimeout,
            Err(Error::Timeout("native receive idle")) => Self::ReceiveIdleTimeout,
            Err(Error::Timeout("native request send")) => Self::SendTimeout,
            Err(Error::Timeout(
                "TCP connect" | "TLS upgrade" | "TLS handshake" | "native channel setup",
            )) => Self::SetupTimeout,
            Err(Error::Timeout(_)) => Self::OtherTimeout,
            Err(Error::Io(_) | Error::ConnectionClosed | Error::Tls(_)) => Self::Transport,
            Err(Error::Server { .. }) => Self::ServerRejected,
            Err(Error::SessionInvalid) => Self::SessionInvalid,
            Err(Error::InvalidConfig(_) | Error::UnsupportedServer { .. }) => Self::Configuration,
            Err(Error::CursorReset { .. }) => Self::CursorReset,
            Err(_) => Self::Decode,
        }
    }
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RequestSample {
    pub kind: RequestKind,
    pub phase: RequestPhase,
    pub outcome: RequestOutcome,
    pub elapsed_us: u64,
    pub send_us: u64,
    pub receive_us: u64,
    /// Sum of synchronous Future::poll work, including transport polls; not pure parser CPU.
    pub decode_active_poll_us: u64,
    pub received_bytes: u64,
    pub frames: u64,
    pub continuation_frames: u64,
    pub keepalive_frames: u64,
}
#[derive(Clone, Debug, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RequestTotals {
    pub kind: RequestKind,
    pub count: u64,
    pub failed: u64,
    pub cancelled: u64,
    pub elapsed_us: u64,
    pub send_us: u64,
    pub receive_us: u64,
    pub decode_active_poll_us: u64,
    pub received_bytes: u64,
    pub frames: u64,
    pub continuation_frames: u64,
    pub keepalive_frames: u64,
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct Snapshot {
    pub pull: PullDiagnostics,
    /// At most one entry per closed RequestKind, independent of page count.
    pub requests: Vec<RequestTotals>,
    pub last_request: Option<RequestSample>,
    pub last_failure: Option<RequestSample>,
    pub pool_wait_us: u64,
    pub pool_waits: u64,
    pub pool_wait_cancelled: u64,
    pub channel_setup_us: u64,
    pub channels_new: u64,
    pub channels_reused: u64,
    pub channel_setup_failures: u64,
    pub channel_setup_cancelled: u64,
    pub max_pool_inflight: u64,
    pub list_pages: u64,
    pub list_items: u64,
    pub node_parse_us: u64,
    pub list_calls: u64,
    pub list_failures: u64,
    pub list_cancelled: u64,
    pub last_list_outcome: Option<RequestOutcome>,
}
/// Counts of validated response observations, never cursor values or event IDs.
/// Raw counts precede core root-roundtrip filtering; projection counts follow
/// successful initialization envelope validation. A no-advance response can
/// contain raw events and must not be labelled an empty native response.
#[derive(Clone, Debug, Default, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct PullDiagnostics {
    pub responses: u64,
    pub raw_event_count: u64,
    pub raw_empty_responses: u64,
    pub cursor_advanced_responses: u64,
    pub cursor_unchanged_responses: u64,
    pub recovery_required_responses: u64,
    pub root_roundtrip_filtered_count: u64,
    pub projected_pages: u64,
    pub projected_event_count: u64,
    pub projected_empty_pages: u64,
    pub no_advance_results: u64,
}
impl PullDiagnostics {
    fn valid(&self) -> bool {
        u128::from(self.cursor_advanced_responses) + u128::from(self.cursor_unchanged_responses)
            == u128::from(self.responses)
            && self.raw_empty_responses <= self.responses
            && self.recovery_required_responses <= self.responses
            && self.projected_empty_pages <= self.projected_pages
            && self.no_advance_results <= self.cursor_unchanged_responses
            && self.root_roundtrip_filtered_count <= self.raw_event_count
            && self.projected_event_count <= self.raw_event_count
    }
    fn combined(&self, other: &Self) -> Option<Self> {
        let mut next = self.clone();
        macro_rules! add {($($field:ident),+)=>{$(next.$field=next.$field.checked_add(other.$field)?;)+};}
        add!(
            responses,
            raw_event_count,
            raw_empty_responses,
            cursor_advanced_responses,
            cursor_unchanged_responses,
            recovery_required_responses,
            root_roundtrip_filtered_count,
            projected_pages,
            projected_event_count,
            projected_empty_pages,
            no_advance_results
        );
        Some(next)
    }
}
impl Snapshot {
    /// Validate the bounded public shape before accepting a worker message.
    pub fn valid(&self) -> bool {
        if !self.pull.valid() {
            return false;
        }
        if self.requests.len() > 12 {
            return false;
        }
        let mut kinds = std::collections::BTreeSet::new();
        for t in &self.requests {
            if t.count == 0
                || !kinds.insert(t.kind)
                || u128::from(t.failed) + u128::from(t.cancelled) > u128::from(t.count)
            {
                return false;
            }
            if u128::from(t.keepalive_frames) + u128::from(t.continuation_frames)
                > u128::from(t.frames)
            {
                return false;
            }
        }
        if u128::from(self.list_failures) + u128::from(self.list_cancelled)
            > u128::from(self.list_calls)
        {
            return false;
        }
        if self
            .last_request
            .as_ref()
            .is_some_and(|r| !kinds.contains(&r.kind))
        {
            return false;
        }
        if self
            .last_failure
            .as_ref()
            .is_some_and(|r| r.outcome == RequestOutcome::Success || !kinds.contains(&r.kind))
        {
            return false;
        }
        true
    }
    /// Merge each completed command exactly once. Refuses malformed inputs
    /// without publication; pool concurrency is a high-water mark, never a sum.
    pub fn merge(&mut self, other: &Self) -> bool {
        if !self.valid() || !other.valid() {
            return false;
        }
        let Some(pull) = self.pull.combined(&other.pull) else {
            return false;
        };
        if self.list_calls.checked_add(other.list_calls).is_none() {
            return false;
        }
        for incoming in &other.requests {
            if self
                .requests
                .iter()
                .find(|t| t.kind == incoming.kind)
                .is_some_and(|t| t.count.checked_add(incoming.count).is_none())
            {
                return false;
            }
        }
        for incoming in &other.requests {
            if let Some(total) = self.requests.iter_mut().find(|t| t.kind == incoming.kind) {
                macro_rules! add {($($field:ident),+)=>{$(total.$field=total.$field.saturating_add(incoming.$field);)+};}
                add!(
                    count,
                    failed,
                    cancelled,
                    elapsed_us,
                    send_us,
                    receive_us,
                    decode_active_poll_us,
                    received_bytes,
                    frames,
                    continuation_frames,
                    keepalive_frames
                );
            } else {
                self.requests.push(incoming.clone());
            }
        }
        self.requests.sort_by_key(|t| t.kind);
        macro_rules! add {($($field:ident),+)=>{$(self.$field=self.$field.saturating_add(other.$field);)+};}
        add!(
            pool_wait_us,
            pool_waits,
            pool_wait_cancelled,
            channel_setup_us,
            channels_new,
            channels_reused,
            channel_setup_failures,
            channel_setup_cancelled,
            list_pages,
            list_items,
            node_parse_us,
            list_calls,
            list_failures,
            list_cancelled
        );
        self.max_pool_inflight = self.max_pool_inflight.max(other.max_pool_inflight);
        self.pull = pull;
        if other.last_request.is_some() {
            self.last_request = other.last_request.clone();
        }
        if other.last_failure.is_some() {
            self.last_failure = other.last_failure.clone();
        }
        if other.last_list_outcome.is_some() {
            self.last_list_outcome = other.last_list_outcome;
        }
        true
    }
}
pub type Observer = Arc<dyn Fn(RequestSample) + Send + Sync>;
#[derive(Clone, Default)]
pub struct Capture {
    state: Arc<Mutex<Snapshot>>,
    observer: Option<Observer>,
}
impl Capture {
    pub fn new() -> Self {
        Self::default()
    }
    pub fn with_observer(observer: Observer) -> Self {
        Self {
            observer: Some(observer),
            ..Self::default()
        }
    }
    pub fn snapshot(&self) -> Snapshot {
        self.state.lock().unwrap().clone()
    }
    fn complete(&self, sample: RequestSample) {
        {
            let mut state = self.state.lock().unwrap();
            if !state.requests.iter().any(|t| t.kind == sample.kind) {
                state.requests.push(RequestTotals {
                    kind: sample.kind,
                    count: 0,
                    failed: 0,
                    cancelled: 0,
                    elapsed_us: 0,
                    send_us: 0,
                    receive_us: 0,
                    decode_active_poll_us: 0,
                    received_bytes: 0,
                    frames: 0,
                    continuation_frames: 0,
                    keepalive_frames: 0,
                });
                state.requests.sort_by_key(|t| t.kind);
            }
            let total = state
                .requests
                .iter_mut()
                .find(|t| t.kind == sample.kind)
                .unwrap();
            macro_rules! add { ($($field:ident),+) => { $(total.$field=total.$field.saturating_add(sample.$field);)+ }; }
            total.count = total.count.saturating_add(1);
            total.failed = total.failed.saturating_add(u64::from(!matches!(
                sample.outcome,
                RequestOutcome::Success | RequestOutcome::Cancelled
            )));
            total.cancelled = total
                .cancelled
                .saturating_add(u64::from(sample.outcome == RequestOutcome::Cancelled));
            add!(
                elapsed_us,
                send_us,
                receive_us,
                decode_active_poll_us,
                received_bytes,
                frames,
                continuation_frames,
                keepalive_frames
            );
            if sample.outcome != RequestOutcome::Success {
                state.last_failure = Some(sample.clone());
            }
            state.last_request = Some(sample.clone());
        }
        if let Some(observer) = &self.observer {
            observer(sample);
        }
    }
}
tokio::task_local! { static CAPTURE: Capture; static REQUEST: Arc<Mutex<RequestSample>>; }
pub async fn scope<F: Future>(capture: Capture, future: F) -> F::Output {
    CAPTURE.scope(capture, future).await
}
/// Exactly once after a native pull response has been fully parsed and validated.
pub fn record_pull_response(
    raw_event_count: usize,
    cursor_advanced: bool,
    recovery_required: bool,
) {
    let _ = CAPTURE.try_with(|c| {
        let mut state = c.state.lock().unwrap();
        let s = &mut state.pull;
        s.responses = s.responses.saturating_add(1);
        s.raw_event_count = s.raw_event_count.saturating_add(raw_event_count as u64);
        s.raw_empty_responses = s
            .raw_empty_responses
            .saturating_add(u64::from(raw_event_count == 0));
        s.cursor_advanced_responses = s
            .cursor_advanced_responses
            .saturating_add(u64::from(cursor_advanced));
        s.cursor_unchanged_responses = s
            .cursor_unchanged_responses
            .saturating_add(u64::from(!cursor_advanced));
        s.recovery_required_responses = s
            .recovery_required_responses
            .saturating_add(u64::from(recovery_required));
    });
}
/// Only the verified root move-away/return normalizer may report discarded events.
pub fn record_root_roundtrip_filtered(count: usize) {
    let _ = CAPTURE.try_with(|c| {
        let mut state = c.state.lock().unwrap();
        state.pull.root_roundtrip_filtered_count = state
            .pull
            .root_roundtrip_filtered_count
            .saturating_add(count as u64);
    });
}
/// Once per successful initialization translation: None is no cursor advance,
/// Some(0) is an advancing empty projected page, not a missing subscription.
pub fn record_pull_projection(projected_event_count: Option<usize>) {
    let _ = CAPTURE.try_with(|c| {
        let mut state = c.state.lock().unwrap();
        let s = &mut state.pull;
        if let Some(count) = projected_event_count {
            s.projected_pages = s.projected_pages.saturating_add(1);
            s.projected_event_count = s.projected_event_count.saturating_add(count as u64);
            s.projected_empty_pages = s
                .projected_empty_pages
                .saturating_add(u64::from(count == 0));
        } else {
            s.no_advance_results = s.no_advance_results.saturating_add(1);
        }
    });
}
pub(crate) async fn directory<F, T>(future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let Ok(capture) = CAPTURE.try_with(Clone::clone) else {
        return future.await;
    };
    let mut guard = DirectoryCompletion {
        capture,
        outcome: RequestOutcome::Cancelled,
    };
    let result = future.await;
    guard.outcome = RequestOutcome::of(&result);
    result
}
struct DirectoryCompletion {
    capture: Capture,
    outcome: RequestOutcome,
}
impl Drop for DirectoryCompletion {
    fn drop(&mut self) {
        let mut s = self.capture.state.lock().unwrap();
        s.list_calls = s.list_calls.saturating_add(1);
        s.list_failures = s.list_failures.saturating_add(u64::from(!matches!(
            self.outcome,
            RequestOutcome::Success | RequestOutcome::Cancelled
        )));
        s.list_cancelled = s
            .list_cancelled
            .saturating_add(u64::from(self.outcome == RequestOutcome::Cancelled));
        s.last_list_outcome = Some(self.outcome);
    }
}
fn micros(start: Instant) -> u64 {
    start.elapsed().as_micros().min(u128::from(u64::MAX)) as u64
}
pub(crate) async fn request<F, T>(kind: RequestKind, future: F) -> Result<T>
where
    F: Future<Output = Result<T>>,
{
    let Ok(capture) = CAPTURE.try_with(Clone::clone) else {
        return future.await;
    };
    let sample = Arc::new(Mutex::new(RequestSample {
        kind,
        phase: RequestPhase::Starting,
        outcome: RequestOutcome::Cancelled,
        elapsed_us: 0,
        send_us: 0,
        receive_us: 0,
        decode_active_poll_us: 0,
        received_bytes: 0,
        frames: 0,
        continuation_frames: 0,
        keepalive_frames: 0,
    }));
    let mut guard = Completion {
        capture,
        sample: sample.clone(),
        start: Instant::now(),
        outcome: RequestOutcome::Cancelled,
    };
    let result = REQUEST.scope(sample, future).await;
    guard.outcome = RequestOutcome::of(&result);
    result
}
struct Completion {
    capture: Capture,
    sample: Arc<Mutex<RequestSample>>,
    start: Instant,
    outcome: RequestOutcome,
}
impl Drop for Completion {
    fn drop(&mut self) {
        let mut sample = self.sample.lock().unwrap().clone();
        sample.elapsed_us = micros(self.start);
        sample.outcome = self.outcome;
        self.capture.complete(sample);
    }
}
pub(crate) struct PhaseSpan {
    sample: Option<Arc<Mutex<RequestSample>>>,
    phase: RequestPhase,
    start: Instant,
}
pub(crate) fn phase(phase: RequestPhase) -> PhaseSpan {
    let sample = REQUEST.try_with(Clone::clone).ok();
    if let Some(sample) = &sample {
        sample.lock().unwrap().phase = phase;
    }
    PhaseSpan {
        sample,
        phase,
        start: Instant::now(),
    }
}
impl Drop for PhaseSpan {
    fn drop(&mut self) {
        if let Some(sample) = &self.sample {
            let mut s = sample.lock().unwrap();
            let elapsed = micros(self.start);
            match self.phase {
                RequestPhase::Send => s.send_us = s.send_us.saturating_add(elapsed),
                RequestPhase::Receive => s.receive_us = s.receive_us.saturating_add(elapsed),
                RequestPhase::Starting => {}
            }
        }
    }
}
pub(crate) async fn decode<F: Future>(future: F) -> F::Output {
    if REQUEST.try_with(|_| ()).is_err() {
        return future.await;
    }
    tokio::pin!(future);
    std::future::poll_fn(|cx| {
        let start = Instant::now();
        let result = future.as_mut().poll(cx);
        let elapsed = micros(start);
        let _ = REQUEST.try_with(|s| {
            let mut s = s.lock().unwrap();
            s.decode_active_poll_us = s.decode_active_poll_us.saturating_add(elapsed);
        });
        result
    })
    .await
}
pub(crate) fn received(bytes: usize) {
    let _ = REQUEST.try_with(|s| {
        let mut s = s.lock().unwrap();
        s.received_bytes = s.received_bytes.saturating_add(bytes as u64);
    });
}
pub(crate) fn frame(keepalive: bool, continuation: bool) {
    let _ = REQUEST.try_with(|s| {
        let mut s = s.lock().unwrap();
        s.frames = s.frames.saturating_add(1);
        s.keepalive_frames = s.keepalive_frames.saturating_add(u64::from(keepalive));
        s.continuation_frames = s
            .continuation_frames
            .saturating_add(u64::from(continuation));
    });
}
pub(crate) fn page(items: usize) {
    let _ = CAPTURE.try_with(|c| {
        let mut s = c.state.lock().unwrap();
        s.list_pages = s.list_pages.saturating_add(1);
        s.list_items = s.list_items.saturating_add(items as u64);
    });
}
pub(crate) struct ParseSpan {
    capture: Option<Capture>,
    start: Instant,
}
pub(crate) fn parsing() -> ParseSpan {
    ParseSpan {
        capture: CAPTURE.try_with(Clone::clone).ok(),
        start: Instant::now(),
    }
}
impl Drop for ParseSpan {
    fn drop(&mut self) {
        if let Some(c) = &self.capture {
            let mut s = c.state.lock().unwrap();
            s.node_parse_us = s.node_parse_us.saturating_add(micros(self.start));
        }
    }
}
pub(crate) fn pool_acquired(new: bool, inflight: usize) {
    let _ = CAPTURE.try_with(|c| {
        let mut s = c.state.lock().unwrap();
        s.max_pool_inflight = s.max_pool_inflight.max(inflight as u64);
        if new {
            s.channels_new = s.channels_new.saturating_add(1);
        } else {
            s.channels_reused = s.channels_reused.saturating_add(1);
        }
    });
}
pub(crate) struct PoolWait {
    capture: Option<Capture>,
    start: Instant,
    pub acquired: bool,
}
pub(crate) fn pool_wait() -> PoolWait {
    PoolWait {
        capture: CAPTURE.try_with(Clone::clone).ok(),
        start: Instant::now(),
        acquired: false,
    }
}
impl Drop for PoolWait {
    fn drop(&mut self) {
        if let Some(c) = &self.capture {
            let mut s = c.state.lock().unwrap();
            s.pool_wait_us = s.pool_wait_us.saturating_add(micros(self.start));
            s.pool_waits = s.pool_waits.saturating_add(1);
            s.pool_wait_cancelled = s
                .pool_wait_cancelled
                .saturating_add(u64::from(!self.acquired));
        }
    }
}
pub(crate) struct SetupSpan {
    capture: Option<Capture>,
    start: Instant,
    pub outcome: Option<bool>,
}
pub(crate) fn setup() -> SetupSpan {
    SetupSpan {
        capture: CAPTURE.try_with(Clone::clone).ok(),
        start: Instant::now(),
        outcome: None,
    }
}
impl Drop for SetupSpan {
    fn drop(&mut self) {
        if let Some(c) = &self.capture {
            let mut s = c.state.lock().unwrap();
            s.channel_setup_us = s.channel_setup_us.saturating_add(micros(self.start));
            if self.outcome == Some(false) {
                s.channel_setup_failures = s.channel_setup_failures.saturating_add(1);
            }
            if self.outcome.is_none() {
                s.channel_setup_cancelled = s.channel_setup_cancelled.saturating_add(1);
            }
        }
    }
}

#[cfg(test)]
#[path = "diagnostics_tests.rs"]
mod tests;
