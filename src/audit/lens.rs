//! Traceability: AXIOM_TAPE_APPEND_ONLY, AXIOM_OBSERVABILITY_TYPED.
use crate::browser_types::*;
use alloc::vec::Vec;

/// Audit lens: project canonical facts into human/AI inspectable views.
pub struct AuditLens;

impl AuditLens {
    pub fn new() -> Self {
        Self
    }

    pub fn provenance(_anchor: &WebAnchor) -> Result<Vec<Cid>, &'static str> {
        todo!("walk provenance graph")
    }

    pub fn diff(_left: &[WebAnchor], _right: &[WebAnchor]) -> Vec<Diff> {
        todo!("semantic diff of two sessions")
    }
}

impl Default for AuditLens {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct Diff {
    pub cid: Cid,
    pub before: Option<Vec<u8>>,
    pub after: Option<Vec<u8>>,
}
