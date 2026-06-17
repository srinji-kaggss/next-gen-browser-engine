//! Traceability: AXIOM_BRAID_CANONICAL, AXIOM_DERIVED_LENS.
use crate::{braid_bridge::term::BraidTerm, browser_types::*};

/// Bidirectional adapter between WebKit observations and Braid IR.
pub struct BraidAdapter;

impl BraidAdapter {
    pub fn new() -> Self {
        Self
    }

    pub fn to_braid(_anchor: &WebAnchor) -> Result<BraidTerm, &'static str> {
        todo!("project WebAnchor to BraidTerm")
    }

    pub fn from_braid(_term: &BraidTerm) -> Result<WebAnchor, &'static str> {
        todo!("project BraidTerm to WebAnchor")
    }
}

impl Default for BraidAdapter {
    fn default() -> Self {
        Self::new()
    }
}
