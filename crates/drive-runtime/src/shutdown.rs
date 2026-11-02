//! Local shutdown diagnostics. Never substitutes a deadline for data durability.
use std::{
    sync::Mutex,
    time::{Duration, Instant},
};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Phase {
    Idle,
    Requested,
    StoppingWorkers,
    DetachingMount,
    DrainingFilesystem,
    CheckpointingLocal,
    Complete,
    Failed,
}
impl Phase {
    fn name(self) -> &'static str {
        match self {
            Self::Idle => "idle",
            Self::Requested => "requested",
            Self::StoppingWorkers => "stopping_workers",
            Self::DetachingMount => "detaching_mount",
            Self::DrainingFilesystem => "draining_filesystem",
            Self::CheckpointingLocal => "checkpointing_local",
            Self::Complete => "complete",
            Self::Failed => "failed",
        }
    }
}
struct State {
    phase: Phase,
    start: Instant,
    stage: Instant,
}
pub struct Diagnostics(Mutex<State>);
impl Default for Diagnostics {
    fn default() -> Self {
        let now = Instant::now();
        Self(Mutex::new(State {
            phase: Phase::Idle,
            start: now,
            stage: now,
        }))
    }
}
impl Diagnostics {
    pub fn phase(&self, phase: Phase) {
        let mut s = self.0.lock().unwrap();
        if s.phase == phase || (phase == Phase::Requested && s.phase != Phase::Idle) {
            return;
        }
        let now = Instant::now();
        if s.phase == Phase::Idle {
            s.start = now;
        }
        s.phase = phase;
        s.stage = now;
        eprintln!(
            "drive-native-fsd: shutdown phase={} elapsed_ms={}",
            phase.name(),
            s.start.elapsed().as_millis()
        );
    }
    pub fn snapshot(&self) -> serde_json::Value {
        let s = self.0.lock().unwrap();
        let active = !matches!(s.phase, Phase::Idle | Phase::Complete | Phase::Failed);
        serde_json::json!({"phase":s.phase.name(),"elapsed_ms":if s.phase==Phase::Idle{0}else{s.start.elapsed().as_millis() as u64},
            "stage_elapsed_ms":if s.phase==Phase::Idle{0}else{s.stage.elapsed().as_millis() as u64},
            "overdue":active && s.stage.elapsed()>=Duration::from_secs(30)})
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn repeated_stop_does_not_restart_stage_or_deadline() {
        let d = Diagnostics::default();
        assert_eq!(d.snapshot()["phase"], "idle");
        d.phase(Phase::Requested);
        d.phase(Phase::DrainingFilesystem);
        d.0.lock().unwrap().stage = Instant::now() - Duration::from_secs(31);
        d.phase(Phase::Requested);
        assert_eq!(d.snapshot()["phase"], "draining_filesystem");
        assert_eq!(d.snapshot()["overdue"], true);
        d.phase(Phase::Complete);
        assert_eq!(d.snapshot()["overdue"], false);
    }
}
