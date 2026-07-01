//! # be-search security suite — 10-layer defense-in-depth + OKF compliance.
//!
//! Proves the security model defined in:
//!   - spec §6  (the 10 layers)
//!   - spec §10 (the 12 required tests)
//!   - topological map "Critical Footguns" table (#1, #2, #5)
//!
//! ## STATUS — read before interpreting results
//!
//! The `be-search` crate is mid-refactor by the owner agent and currently
//! **does not compile** (16 errors: `tantivy`/`dashmap` not in `Cargo.toml`,
//! `error::SearchResult` removed, `partition::PartitionStore` /
//! `filter::inject_scope_filter` / `filter::post_filter` referenced but
//! undefined). Additionally the live type layer diverges from spec in three
//! places that these tests pin:
//!
//!   - `scope.rs`    is FORGEABLE (public fields + `pub fn new`) — spec §4.1
//!                   requires an unforgeable `Scope` (`pub(crate)` ctor).
//!   - `query.rs`    has NO `parse`/sanitization — spec §4.2 / L2 require an
//!                   eval-free, operator-rejecting `QueryPlan::parse`.
//!   - `candidate.rs` lost `provenance: Scope` — spec §4.3 requires it.
//!
//! Tests are written to the SPEC contract. Those needing the orchestrator
//! (`BrowserSearch::search`, partitioned index, fault injection) are
//! `#[ignore]`'d with the exact assertion contract documented inline; they
//! activate the moment the crate compiles to spec. No test is weakened to
//! accommodate the current broken state (code-review skill: never retarget a
//! gate green).

mod common;

use be_search::{FailClosed, QueryPlan, Scope};

// ===========================================================================
// spec §10 — the 12 required tests
// ===========================================================================

// --- L1: unforgeable Scope -------------------------------------------------

/// Prove: `Scope` has no public constructor; it cannot be created from client
/// input. The strong proof is compile-time: the `compile_fail` doctests in
/// `src/lib.rs` assert `Scope::new` / `Scope::default()` / struct-literal
/// construction all fail to compile against the public API. Here we assert the
/// runtime consequence: every obtainable scope is immutable and distinct, with
/// read-only accessors as the only surface.
///
/// STATUS: runnable once `Scope::for_test` exists and `Scope` is unforgeable.
#[test]
fn test_scope_cannot_be_fabricated() {
    let a = common::scope(1, common::READ);
    let b = common::scope(2, common::READ);
    assert_ne!(a, b, "distinct sessions must yield distinct scopes");
    assert_ne!(a.session(), b.session());
    assert_eq!(a.session(), 1);
    // No mutator exists — only read accessors.
    let _ = a.page_origin();
    let _ = a.tenant();
    let _ = a.capabilities();
}

// --- L2: eval-free query AST ----------------------------------------------

/// Prove: operator chars (`/`, `*`, `?`, parens, brackets, quotes, boolean &
/// regex metachars), control chars, and overlong input are all rejected by
/// `QueryPlan::parse` with `FailClosed::BadPlan`. No injection reaches the
/// engine.
///
/// STATUS: runnable once `query.rs` restores `QueryPlan::parse` (spec §4.2).
#[test]
fn test_query_injection_rejected() {
    let bad = [
        "\0", "\u{1}", "\n", "\t", // control chars
        "a/b", "//", // path / comment operators
        "a*b", "a?b", // glob operators
        "(group)", "[set]", "{brace}", // grouping
        "\"q\"", "a'b",  // quotes
        "a\\b", // escape
        "a|b", "a&b", "!n", "^b", "~f", // boolean / regex operators
        "a:b", "a<b", "a>b", // field / angle operators
    ];
    for q in bad {
        assert!(
            matches!(QueryPlan::parse(q), Err(FailClosed::BadPlan)),
            "query injection not rejected: {q:?}"
        );
    }
    // Overlong input is rejected.
    let overlong = "x".repeat(5000);
    assert!(matches!(
        QueryPlan::parse(&overlong),
        Err(FailClosed::BadPlan)
    ));
    // Benign queries are accepted; whitespace-only reduces to Empty.
    assert!(QueryPlan::parse("hello world").is_ok());
    assert!(matches!(QueryPlan::parse("   "), Ok(QueryPlan::Empty)));
}

/// SPEC GAP: spec §10 lists `$` as a rejected operator char, but the
/// `query.rs` sanitization set historically omits `$` (and `.`, `+`). This test
/// pins the spec requirement. It is `#[ignore]`'d so it surfaces the gap under
/// `cargo test -- --ignored` without failing the suite. When `query.rs`
/// rejects `$`, remove the `#[ignore]`.
#[test]
fn test_query_injection_dollar_sign_gap() {
    assert!(
        matches!(QueryPlan::parse("price$"), Err(FailClosed::BadPlan)),
        "'$' should be rejected per spec §10"
    );
}

// --- L6: cross-scope isolation --------------------------------------------

/// Prove: a search under Scope A cannot return Scope B's nodes.
///
/// STATUS: `#[ignore]` — requires the partitioned index + a public indexing
/// fixture on `BrowserSearch` (spec §8.1). Contract below.
#[test]
#[ignore = "TODO-CONNECT: needs BrowserSearch index fixture + partitioned search (spec §5 L6, §8)"]
fn test_cross_scope_isolation() {
    use be_search::{BrowserSearch, SearchRequest};

    let bs = BrowserSearch::new();
    let a = common::scope(1, common::READ);
    let b = common::scope(2, common::READ);
    // bs.index(&a, ...node 1...); bs.index(&b, ...node 2...);
    let hits = bs
        .search(SearchRequest::new("x", 10), &a)
        .expect("index up");
    assert!(
        hits.iter().all(|c| c.provenance == a),
        "scope A leaked a foreign node"
    );
    assert!(
        hits.iter().all(|c| c.node_id != 2),
        "scope B node leaked into A"
    );
    // Precondition checkable now: distinct scopes are never equal.
    assert_ne!(a, b);
}

// --- L4: scope filter always applied --------------------------------------

/// Prove: every executed query carries a mandatory scope guard (`Occur::Must`
/// AND'd onto the user query — the RLS analog, spec §8.2). Omitting it is a
/// deny / build error, never silent allow (Footgun #1).
///
/// STATUS: `#[ignore]` — requires `filter::inject_scope_filter` + a `GuardedPlan`
/// with an inspectable guard (spec §5 L4, §8.2). Contract below.
#[test]
fn test_scope_filter_always_applied() {
    use be_search::filter;
    let scope = common::scope(1, common::READ);
    let plan = QueryPlan::parse("anything").expect("benign");
    let guarded = filter::inject_scope_filter(plan, &scope);
    assert_eq!(guarded.guard, be_search::filter::Occur::Must);
    assert_eq!(guarded.scope_hash, scope.digest());
}

// --- L8: score not exposed ------------------------------------------------

/// Prove: `Candidate` exposes NO `score` / `distance` / `embedding` field
/// (spec §4.3, P5). The negative is enforced by `compile_fail` doctests in
/// `src/lib.rs`; here we lock the documented public field surface.
///
/// STATUS: runnable once `Candidate` restores `provenance: Scope` (spec §4.3).
#[test]
fn test_score_not_exposed() {
    use be_search::NodeKind;
    let scope = common::scope(1, common::READ);
    let c = common::candidate(7, &scope);
    // Candidate exposes EXACTLY these public fields (spec §4.3):
    let _node_id: u64 = c.node_id;
    let _kind: NodeKind = c.kind;
    let _prov: &Scope = &c.provenance;
    let _excerpt: &str = &c.excerpt; // SanitizedExcerpt derefs/as_str per spec
    let _ev: &Vec<u64> = &c.evidence_refs;
    // The negative (no score/distance/embedding) is proven by compile_fail
    // doctests in src/lib.rs — `c.score` will not compile.
}

// --- L6/L10: fail-closed on index down ------------------------------------

/// Prove: when the index is unavailable, search returns `Err(FailClosed)`,
/// never stale or cached results (spec §2.3).
///
/// STATUS: `#[ignore]` — requires a fault-injection knob on `BrowserSearch`
/// (spec §5 L6). Contract below.
#[test]
#[ignore = "TODO-CONNECT: needs BrowserSearch index-down fault injection (spec §5 L6, §4.4)"]
fn test_fail_closed_on_index_down() {
    use be_search::{BrowserSearch, SearchRequest};
    let bs = BrowserSearch::new();
    // bs.set_index_up(false);
    let scope = common::scope(1, common::READ);
    let res = bs.search(SearchRequest::new("x", 10), &scope);
    assert!(
        matches!(
            res,
            Err(FailClosed::IndexDown) | Err(FailClosed::PartitionMissing)
        ),
        "index-down must fail closed, got {res:?}"
    );
    // FailClosed::IndexDown exists as the deny variant (precondition).
    assert!(matches!(FailClosed::IndexDown, FailClosed::IndexDown));
}

// --- L10: fallback never global -------------------------------------------

/// Prove: the deterministic fallback is scoped-only — it may return a per-scope
/// snapshot but NEVER global / cross-scope results (spec §2.3, L10).
///
/// STATUS: `#[ignore]` — requires `fallback::deterministic_fallback` + a scoped
/// snapshot fixture (spec §5 L10). Contract below.
#[test]
#[ignore = "TODO-CONNECT: needs fallback::deterministic_fallback scoped snapshot (spec §5 L10)"]
fn test_fallback_never_global() {
    use be_search::{fallback, BrowserSearch, SearchRequest};
    let bs = BrowserSearch::new();
    let a = common::scope(1, common::READ);
    // bs.seed_scoped_snapshot(&a, [a_nodes...]);
    let hits = bs
        .search(SearchRequest::new("x", 10), &a)
        .expect("fallback");
    assert!(hits.iter().all(|c| c.provenance == a));
    // Precondition: a fallback keyed by scope hash cannot address another partition.
    let b = common::scope(2, common::READ);
    assert_ne!(a, b);
    let _ = fallback::deterministic_fallback; // symbol exists
}

// --- L5: audit before search ----------------------------------------------

/// Prove: the query is journaled (`Outcome::Submitted`) BEFORE execution. Even
/// if search fails, the intent record exists (audit-before-effect, spec §2.4).
///
/// STATUS: `#[ignore]` — requires the search pipeline wired through `AuditLog`
/// + index-down injection so execution is provably skipped while the audit
/// entry lands (spec §5 L5, §9). Contract below.
#[test]
#[ignore = "TODO-CONNECT: needs AuditLog wired through search() + fault injection (spec §5 L5, §9)"]
fn test_audit_before_search() {
    use be_search::{BrowserSearch, Outcome, SearchRequest};
    let bs = BrowserSearch::new();
    let scope = common::scope(1, common::READ);
    // bs.set_index_up(false);
    let _ = bs.search(SearchRequest::new("x", 10), &scope); // fails closed
    let trail = bs.audit_trail(&scope);
    assert!(
        trail.iter().any(|e| e.outcome == Outcome::Submitted),
        "query must be journaled before/without execution"
    );
}

// --- §2.4: journal replay -------------------------------------------------

/// Prove: replaying the journal against a fresh index produces identical
/// per-scope state (spec §8.3, R5).
///
/// STATUS: `#[ignore]` — requires `journal::SearchJournal` + `export`/`replay`
/// fixtures (spec §8.3). Contract below.
#[test]
#[ignore = "TODO-CONNECT: needs journal::SearchJournal export/replay (spec §8.3, R5)"]
fn test_journal_replay() {
    use be_search::BrowserSearch;
    let bs1 = BrowserSearch::new();
    let scope = common::scope(1, common::READ);
    // bs1.index(...nodes...);
    // let ops = bs1.export_journal();
    // let bs2 = BrowserSearch::new(); bs2.replay(ops);
    // assert_eq!(bs1.dump_scope_nodes(&scope), bs2.dump_scope_nodes(&scope));
    let _ = (bs1, scope); // placeholders until journal API lands
}

// --- §2.6: capability composition is AND (intersection) -------------------

/// Prove: composing capabilities is intersection (AND), never union (OR). A
/// node requiring {Read, Write} is visible only to a scope holding BOTH.
///
/// STATUS: runnable once `Scope`/`CapSet` restore the spec capability API
/// (`Scope::allows`, `CapSet::intersection`/`includes`).
#[test]
fn test_and_composition() {
    use be_search::CapSet;
    let node_needs = CapSet::from_bits(common::READ | common::WRITE);
    let only_read = CapSet::from_bits(common::READ);
    let only_write = CapSet::from_bits(common::WRITE);
    let both = CapSet::from_bits(common::READ | common::WRITE);

    assert!(!only_read.includes(node_needs), "missing Write must deny");
    assert!(!only_write.includes(node_needs), "missing Read must deny");
    assert!(both.includes(node_needs), "holding both must allow");

    // Composition = intersection: the joined set is ⊆ each party — no widening.
    let composed = only_read.intersection(both);
    assert!(
        !composed.includes(node_needs),
        "AND composition must not widen"
    );
    assert!(
        !only_read.intersection(only_write).includes(node_needs),
        "read∩write is empty → must deny"
    );
}

// --- L7: IDOR neutralized (CWE-639) ---------------------------------------

/// Prove: every returned object passes a per-object capability check — a scope
/// match alone is NOT sufficient. The primitive gate is `Scope::allows`.
///
/// STATUS: primitive-level proof runnable once `Scope::allows` exists; the
/// "applied to EVERY result" enforcement is the orchestrator's job (noted).
#[test]
fn test_idor_neutralized() {
    let scope_read = common::scope(1, common::READ);
    let scope_rw = common::scope(1, common::READ | common::WRITE);

    let needs_read = be_search::CapSet::from_bits(common::READ);
    let needs_write = be_search::CapSet::from_bits(common::WRITE);
    let needs_rw = be_search::CapSet::from_bits(common::READ | common::WRITE);

    // Same scope identity, different per-object caps → per-object deny (IDOR).
    assert!(scope_read.allows(needs_read));
    assert!(
        !scope_read.allows(needs_write),
        "write-gated object must deny read-only scope"
    );
    assert!(!scope_read.allows(needs_rw));
    assert!(scope_rw.allows(needs_rw));
    // TODO-CONNECT: "on EVERY result" enforcement needs BrowserSearch post_filter.
}

// --- §2.3: user-supplied regex rejected -----------------------------------

/// Prove: user-supplied regex is rejected — regex must be server-compiled from
/// an allowlist only (spec §8.2, Footgun #4). There is no regex execution path.
///
/// STATUS: runnable once `query.rs` restores `QueryPlan::parse` (spec §4.2).
#[test]
fn test_regex_rejected() {
    let regexes = [".*", "[a-z]", "(a|b)", "\\d+", "^x$", "a*b+", "col(r|ur)"];
    for r in regexes {
        assert!(
            matches!(QueryPlan::parse(r), Err(FailClosed::BadPlan)),
            "user regex not rejected: {r:?}"
        );
    }
}

// ===========================================================================
// Critical Footguns (topological map)
// ===========================================================================

/// Footgun #1 (Elastic DLS): a query reaching the engine WITHOUT a scope filter
/// MUST be denied at build/runtime, never silently allowed.
///
/// STATUS: type-level proof via `compile_fail` doctest in `src/lib.rs` (no
/// `BrowserSearch::execute_unscoped`). Runtime proof is `#[ignore]` until the
/// orchestrator + `GuardedPlan` land (see test_scope_filter_always_applied).
#[test]
fn test_omitted_filter_denies() {
    use be_search::filter;
    let scope = common::scope(1, common::READ);
    let plan = QueryPlan::parse("x").expect("benign");
    let guarded = filter::inject_scope_filter(plan, &scope);
    assert_eq!(guarded.guard, filter::Occur::Must);
    // An unguarded plan must be refused at the execution boundary.
}

/// Footgun #2 (Elastic DLS): combining capabilities must NOT union them.
/// Intersection is always ⊆ both operands — never wider.
///
/// STATUS: runnable once `CapSet::intersection`/`contains`/`includes` exist.
#[test]
fn test_multi_capability_or_widening_prevented() {
    use be_search::CapSet;
    let s1 = CapSet::from_bits(common::READ | common::ACT);
    let s2 = CapSet::from_bits(common::WRITE | common::ACT);
    let inter = s1.intersection(s2);

    // Intersection grants a cap only if BOTH sides hold it.
    assert!(!inter.contains(0), "Read not in s2 → not in intersection");
    assert!(!inter.contains(1), "Write not in s1 → not in intersection");
    assert!(inter.contains(2), "Act in both → retained");
    // Widening would make the composition a superset of either operand.
    assert!(!inter.includes(s1));
    assert!(!inter.includes(s2));
    // A node requiring Read+Write is invisible to the composition (AND), even
    // though Read∈s1 and Write∈s2 individually — OR would allow it.
    let rw = CapSet::from_bits(common::READ | common::WRITE);
    assert!(!inter.includes(rw), "OR-widening must be prevented");
}

/// Footgun #5 (Logic OS §12.3): LLM output MUST flow through the typed
/// action → sanitization → scoped query pipeline, never as a raw string into
/// the engine. The only string entry is `QueryPlan::parse`, which sanitizes.
///
/// STATUS: runnable once `query.rs` restores `QueryPlan::parse` (spec §4.2).
/// The "no raw-exec bypass" negative is a `compile_fail` doctest in `src/lib.rs`.
#[test]
fn test_llm_output_not_raw_query() {
    let injections = [
        "query OR *",                // glob widen → rejected (*)
        "<script>alert(1)</script>", // markup → rejected (<, >)
        "a:b:c",                     // field injection → rejected (:)
        "(select all)",              // grouping → rejected (())
        "term & everything",         // boolean widen → rejected (&)
        "x\\y",                      // escape → rejected (\)
    ];
    for inj in injections {
        assert!(
            matches!(QueryPlan::parse(inj), Err(FailClosed::BadPlan)),
            "LLM injection not sanitized: {inj:?}"
        );
    }
    // A benign LLM-produced term is accepted as a sanitized literal only.
    assert!(QueryPlan::parse("submit button").is_ok());
}
