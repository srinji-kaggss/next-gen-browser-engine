//! Traceability: AXIOM_DETERMINISTIC_QUIESCENCE, AXIOM_POLICY_AUTHORITY.
use crate::{action::Action, browser_types::*};

/// Deterministic state-machine transition table.
pub struct TransitionTable;

impl TransitionTable {
    pub fn new() -> Self {
        Self
    }

    /// Given current state and a validated action, produce the next state.
    ///
    /// Verdict is the policy-broker output. The state machine only encodes
    /// *structural* transitions: whether the engine is idle, busy, or finished.
    /// Policy decisions (Allow/Confirm/Deny) are consumed by the caller before
    /// the transition is applied.
    pub fn transition(
        &self,
        state: BrowserState,
        action: &Action,
        verdict: Verdict,
    ) -> BrowserState {
        // Denied actions leave the state unchanged.
        if verdict == Verdict::Deny {
            return state;
        }

        match (state, action.verb) {
            // Navigation starts from Idle, Quiescent, or Executing.
            (BrowserState::Idle, ActionVerb::Navigate)
            | (BrowserState::Quiescent, ActionVerb::Navigate)
            | (BrowserState::Executing, ActionVerb::Navigate) => BrowserState::Navigating,

            // Observation happens once a page is quiescent, or as a side effect.
            (BrowserState::Quiescent, ActionVerb::Observe)
            | (BrowserState::Executing, ActionVerb::Observe) => BrowserState::Observing,

            // Click / Type / Scroll / ExecuteJs / ExecuteWasm are executing states.
            (BrowserState::Quiescent, ActionVerb::Click)
            | (BrowserState::Quiescent, ActionVerb::Type)
            | (BrowserState::Quiescent, ActionVerb::Scroll)
            | (BrowserState::Quiescent, ActionVerb::ExecuteJs)
            | (BrowserState::Quiescent, ActionVerb::ExecuteWasm)
            | (BrowserState::Navigating, ActionVerb::Wait) => BrowserState::Executing,

            // Wait lets the engine remain in the current active state.
            (s, ActionVerb::Wait) => s,

            // Download from any stable state goes through Executing.
            (BrowserState::Quiescent, ActionVerb::Download)
            | (BrowserState::Executing, ActionVerb::Download) => BrowserState::Executing,

            // Terminal: explicit terminate from any state.
            (_, ActionVerb::Navigate) if action.risk == Risk::Denied => BrowserState::Terminated,

            // Anything else that is not valid in the current state returns the
            // current state unchanged, preserving determinism.
            (s, _) => s,
        }
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

#[cfg(test)]
mod tests {
    use super::*;
    use alloc::vec::Vec;
    use alloc::string::ToString;

    fn action(verb: ActionVerb, risk: Risk) -> Action {
        Action {
            verb,
            target_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"example.com"),
            origin: "example.com".to_string(),
            capability_cid: Cid::compute(WEB_ELEMENT_DOMAIN, b"cap-1"),
            risk,
            parameters: Vec::new(),
            effect_signature: Vec::new(),
        }
    }

    #[test]
    fn navigate_from_idle() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Idle,
                &action(ActionVerb::Navigate, Risk::Low),
                Verdict::Allow
            ),
            BrowserState::Navigating
        );
    }

    #[test]
    fn deny_preserves_state() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Navigating,
                &action(ActionVerb::Click, Risk::Low),
                Verdict::Deny
            ),
            BrowserState::Navigating
        );
    }

    #[test]
    fn click_from_quiescent() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Quiescent,
                &action(ActionVerb::Click, Risk::Low),
                Verdict::Allow
            ),
            BrowserState::Executing
        );
    }

    #[test]
    fn wait_preserves_active_state() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Executing,
                &action(ActionVerb::Wait, Risk::Low),
                Verdict::Allow
            ),
            BrowserState::Executing
        );
    }

    #[test]
    fn observe_from_quiescent() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Quiescent,
                &action(ActionVerb::Observe, Risk::Low),
                Verdict::Allow
            ),
            BrowserState::Observing
        );
    }

    #[test]
    fn invalid_transition_preserves_state() {
        let t = TransitionTable::new();
        assert_eq!(
            t.transition(
                BrowserState::Idle,
                &action(ActionVerb::Click, Risk::Low),
                Verdict::Allow
            ),
            BrowserState::Idle
        );
    }
}
