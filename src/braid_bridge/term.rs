//! Traceability: AXIOM_BRAID_CANONICAL.
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
