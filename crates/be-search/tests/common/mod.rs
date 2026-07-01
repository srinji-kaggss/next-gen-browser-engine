//! Test helpers for the be-search security suite.
//!
//! STATUS: spec-faithful. These helpers target the spec API (§4.1–§4.3, §5).
//! They activate once the crate compiles to that API. Two connection points
//! required from the implementer:
//!
//!   (1) `Scope` MUST be made unforgeable per spec §4.1 (private fields,
//!       `pub(crate)` constructor). It is currently FORGEABLE in scope.rs
//!       (public fields + `pub fn new`) — a spec violation to fix.
//!   (2) A test-only `Scope::for_test` constructor behind
//!       `#[cfg(feature = "test-fixtures")]` (mirrors the trusted auth path;
//!       production unforgeability is unaffected). audit.rs already anticipates
//!       a `Scope::__test()`, so this is intended testability infra.
//!
//! Capability encoding: `CapSet` bit indices used in tests.
//!   READ  = 1 << 0   WRITE = 1 << 1   ACT   = 1 << 2

#![allow(dead_code)]

use be_search::{Candidate, CapSet, NodeKind, Scope};

pub const READ: u64 = 1 << 0;
pub const WRITE: u64 = 1 << 1;
pub const ACT: u64 = 1 << 2;

/// Build a `Scope` for a single session within one page/tenant, with the given
/// capability bits.
pub fn scope(session: u64, cap_bits: u64) -> Scope {
    Scope::new(1, session, 1, CapSet::from_bits(cap_bits))
}

/// Build a `Scope` with full control over the isolation axes.
pub fn scope_full(page: u64, session: u64, tenant: u64, cap_bits: u64) -> Scope {
    Scope::new(page, session, tenant, CapSet::from_bits(cap_bits))
}

/// Build a typed `Candidate` (spec §4.3) stamped with the querying scope as its
/// provenance. Used to assert cross-scope isolation and IDOR post-filtering.
pub fn candidate(node_id: u64, scope: &Scope) -> Candidate {
    Candidate {
        node_id,
        kind: NodeKind::ELEMENT,
        provenance: scope.clone(),
        excerpt: be_search::SanitizedExcerpt(format!("node-{node_id}")),
        evidence_refs: vec![node_id],
    }
}
