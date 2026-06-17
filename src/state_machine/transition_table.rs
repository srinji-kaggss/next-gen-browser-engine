//! Traceability: AXIOM_DETERMINISTIC_QUIESCENCE, AXIOM_POLICY_AUTHORITY.
use crate::{browser_types::*, policy::WebAction};

/// Deterministic state-machine transition table.
pub struct TransitionTable;

impl TransitionTable {
    pub fn new() -> Self {
        Self
    }

    /// Given current state and a validated action, produce the next state and verdict.
    pub fn transition(
        &self,
        _state: BrowserState,
        _action: &WebAction,
        _verdict: Verdict,
    ) -> BrowserState {
        todo!("state machine transition")
    }
}

impl Default for TransitionTable {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrowserState {
    Idle,
    Navigating,
    Observing,
    Quiescent,
    Executing,
    Terminated,
}
