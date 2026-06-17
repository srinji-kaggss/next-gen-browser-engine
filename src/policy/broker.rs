//! Traceability: AXIOM_POLICY_AUTHORITY, AXIOM_CLOSED_ACTIONS, AXIOM_LLMS_SENSOR_ONLY.
use crate::{browser_types::*, capability::WebCapability};
use alloc::string::String;
use alloc::vec::Vec;

/// DAL-A policy broker.
/// (facts, proposed_action, caller_caps) -> Verdict
pub struct PolicyBroker;

impl PolicyBroker {
    pub fn new() -> Self {
        Self
    }

    /// Deterministic verdict. No LLM, no ambient authority.
    pub fn decide(
        &self,
        _facts: &[WebAnchor],
        _action: &WebAction,
        _caps: &[WebCapability],
    ) -> Verdict {
        todo!("deterministic policy decision")
    }
}

impl Default for PolicyBroker {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebAction {
    pub verb: ActionVerb,
    pub target_cid: Cid,
    pub capability: String,
    pub risk: Risk,
    pub parameters: Vec<u8>,
    pub effect_signature: Vec<String>,
}
