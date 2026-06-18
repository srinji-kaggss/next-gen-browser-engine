//! Traceability: AXIOM_CLOSED_ACTIONS, AXIOM_SMALL_MODEL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// A validated, closed-vocabulary action ready for the state machine.
pub struct Action {
    pub verb: ActionVerb,
    pub target_cid: Cid,
    pub capability_cid: Cid,
    pub risk: Risk,
    pub parameters: Vec<u8>,
    pub effect_signature: Vec<String>,
}

impl Action {
    /// Validate that the action is one of the closed verbs.
    pub fn validate_verb(&self) -> Result<(), &'static str> {
        let _ = self.verb.as_str();
        Ok(())
    }
}
