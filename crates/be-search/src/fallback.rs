//! Deterministic scoped fallback (P10, §2.3).
//!
//! Invoked at L10 when the partitioned search index is unavailable. It is the
//! guaranteed-scoped floor: with no partition reachable from a pure function
//! it can only ever return a scoped result set or `FailClosed`. It can NEVER
//! return global/unscoped results — there is no unscoped pool to draw from,
//! and the `SearchResult` type makes fail-open unrepresentable.
//!
//! `Ok(vec![])` here is deny-by-emptiness (§2.3 lists empty as a Deny form):
//! scoped, leak-free, and never stale. A richer scoped exact-term lookup is
//! wired at the partition layer (which owns the per-scope index); this
//! function is the structural safety net when that layer is unreachable.

use crate::query::QueryPlan;
use crate::scope::Scope;
use crate::SearchResult;

pub fn deterministic_fallback(_plan: &QueryPlan, scope: &Scope) -> SearchResult {
    // `scope` is accepted to make the scope-aware contract explicit at the
    // type level; the empty result is trivially a subset of `scope`.
    let _ = scope;
    Ok(Vec::new())
}
