//! Traceability: AXIOM_OBSERVABILITY_TYPED, AXIOM_BRAID_CANONICAL.
use crate::browser_types::*;
use alloc::string::String;
use alloc::vec::Vec;

/// A typed observation fact derived from WebKit output.
pub struct ObservationAnchor {
    pub kind: ObservationKind,
    pub target_cid: Cid,
    pub observed_at: String,
    pub facts: Vec<Fact>,
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
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Fact {
    pub predicate: String,
    pub object: String,
}
