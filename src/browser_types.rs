//! Traceability: AXIOM_BRAID_CANONICAL.
use alloc::string::String;
use alloc::vec::Vec;

pub type Cid = String;
pub type Origin = String;
pub type Url = String;

/// A content-addressed fact in the Braid fabric.
#[derive(Debug, Clone, PartialEq)]
pub struct WebAnchor {
    pub cid: Cid,
    pub term_family: TermFamily,
    pub created_at: String,
    pub provenance: Provenance,
    pub payload: Vec<u8>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TermFamily {
    Element,
    Observation,
    Action,
    Capability,
    Verdict,
    Transition,
}

impl TermFamily {
    pub fn as_str(&self) -> &'static str {
        match self {
            TermFamily::Element => "web.element",
            TermFamily::Observation => "web.observation",
            TermFamily::Action => "web.action",
            TermFamily::Capability => "web.capability",
            TermFamily::Verdict => "web.verdict",
            TermFamily::Transition => "web.transition",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Provenance {
    pub source: String,
    pub input_cids: Vec<Cid>,
}

/// Closed-vocabulary action verbs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActionVerb {
    Navigate,
    Observe,
    Click,
    Type,
    Scroll,
    Download,
    Wait,
    ExecuteJs,
    ExecuteWasm,
}

impl ActionVerb {
    pub fn as_str(&self) -> &'static str {
        match self {
            ActionVerb::Navigate => "web.navigate",
            ActionVerb::Observe => "web.observe",
            ActionVerb::Click => "web.click",
            ActionVerb::Type => "web.type",
            ActionVerb::Scroll => "web.scroll",
            ActionVerb::Download => "web.download",
            ActionVerb::Wait => "web.wait",
            ActionVerb::ExecuteJs => "web.execute_js",
            ActionVerb::ExecuteWasm => "web.execute_wasm",
        }
    }
}

/// Verdict from the policy broker.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    Allow,
    Deny,
    Confirm,
}
