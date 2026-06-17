//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_AIP_WIRE_LENS, AXIOM_SMALL_MODEL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// Braid term families specific to the browser engine.
pub enum BraidTerm {
    Element(WebElement),
    Observation(WebObservation),
    Action(WebActionTerm),
    Capability(WebCapabilityTerm),
    Verdict(WebVerdict),
    AipState(AipState),
    AipPolicy(AipPolicy),
    AipAction(AipAction),
    AipDelegation(AipDelegation),
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebElement {
    pub tag: String,
    pub attrs: Vec<(String, String)>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebObservation {
    pub kind: String,
    pub target_cid: Cid,
    pub facts: Vec<(String, String)>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebActionTerm {
    pub verb: String,
    pub target_cid: Cid,
    pub parameters: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebCapabilityTerm {
    pub issuer: String,
    pub subject: String,
    pub scope: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct WebVerdict {
    pub decision: String,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AipState {
    pub version: String,
    pub surface_id: String,
    pub surface_type: String,
    pub url: Option<String>,
    pub state: Vec<(String, String)>,
    pub affordances: Vec<AipAffordance>,
    pub memory_scope: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AipAffordance {
    pub id: String,
    pub kind: String,
    pub label: String,
    pub risk: Risk,
    pub requires_human_confirmation: bool,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AipAction {
    pub id: String,
    pub kind: String,
    pub target_cid: Cid,
    pub risk: Risk,
    pub preconditions: Vec<String>,
    pub postconditions: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AipPolicy {
    pub version: String,
    pub site: String,
    pub observation_allowed: bool,
    pub observation_scope: String,
    pub training_allowed: bool,
    pub export_allowed: bool,
}

#[derive(Debug, Clone, PartialEq)]
pub struct AipDelegation {
    pub issuer_did: Did,
    pub holder_did: Did,
    pub audience: String,
    pub scope: Vec<String>,
    pub denied: Vec<String>,
    pub privacy_tier: PrivacyTier,
    pub ttl_seconds: u32,
}
