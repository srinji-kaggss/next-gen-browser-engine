//! Traceability: AXIOM_CLOSED_ACTIONS, AXIOM_SMALL_MODEL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// A validated, closed-vocabulary action ready for the state machine.
pub struct Action {
    pub verb: ActionVerb,
    pub target_cid: Cid,
    /// The target's origin — a first-class capability boundary (A3), carried
    /// explicitly. It is NOT divined from `target_cid`: a content-address
    /// commits to bytes, not to provenance. A full impl resolves it from an
    /// origin fact; until then the producer supplies it.
    pub origin: Origin,
    pub capability_cid: Cid,
    pub risk: Risk,
    pub parameters: Vec<u8>,
    pub effect_signature: Vec<String>,
}

impl Action {
    /// Validate that the action is one of the closed verbs.
    pub fn validate_verb(&self) -> Result<(), &'static str> {
        if braid_vocab_web::registry_v0()
            .get(self.verb.as_str())
            .is_some()
        {
            Ok(())
        } else {
            Err("action verb is not registered in braid-vocab-web")
        }
    }
}
