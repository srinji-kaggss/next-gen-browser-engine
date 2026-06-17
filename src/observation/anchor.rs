//! Traceability: AXIOM_OBSERVABILITY_TYPED, AXIOM_BRAID_CANONICAL, AXIOM_PRIVACY_TIER.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// A typed observation fact derived from WebKit output.
pub struct ObservationAnchor {
    pub kind: ObservationKind,
    pub target_cid: Cid,
    pub observed_at: String,
    pub facts: Vec<Fact>,
    pub sensitivity: Option<SensitivityClass>,
    pub privacy_tier: PrivacyTier,
    pub trust_class: TrustClass,
    pub raw_source: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    Load,
    Layout,
    Paint,
    Element,
    Network,
    Console,
    A11y,
    Semantic,
    AipState,
    AipPolicy,
}

impl ObservationKind {
    pub fn as_str(&self) -> &'static str {
        match self {
            ObservationKind::Load => "load",
            ObservationKind::Layout => "layout",
            ObservationKind::Paint => "paint",
            ObservationKind::Element => "element",
            ObservationKind::Network => "network",
            ObservationKind::Console => "console",
            ObservationKind::A11y => "a11y",
            ObservationKind::Semantic => "semantic",
            ObservationKind::AipState => "aip_state",
            ObservationKind::AipPolicy => "aip_policy",
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub predicate: String,
    pub object: String,
    pub sensitivity: Option<SensitivityClass>,
}
