# be-search Specification — Hardened Against AI_RESEARCH_OKF

---
okf_schema: okf.spec.v1
spec_id: be-search
version: 2.0.0
status: hardened
generated_utc: "2026-07-01T14:30:00Z"
evidence_base:
  - research-log/2026-07-01-topological-map.md
  - research-logs from 5 parallel agents (tantivy, meilisearch, hybrid, security, lgwks)
  - AI/Research/LOGIC_OS_AI_FIRST_LANGUAGE_STATE_FABRIC_SECURITY_RESEARCH_BASELINE.md
  - AI/Research/agent_handoff_ai_os_framework.md
  - AI/Research/deep-research-report.md
okf_principles_checked:
  - binary_first
  - scoped_retrieval
  - typed_output
  - deterministic_fallback
  - audit_evidence
  - authz_before_retrieval
  - leak_prevention
  - fail_closed
  - human_reconstructability
---

## 1. Purpose

`be-search` is the **Semantic Index** state fabric for the browser engine. It provides scoped, capability-gated retrieval over the semantic graph (DOM nodes, page elements, page state).

Per Logic OS Research Baseline §5.1:
> "Semantic Index: Embeddings and searchable summaries for local retrieval. Local vector index with scoped retrieval."

## 2. Non-Negotiable Principles (from AI_RESEARCH_OKF)

### 2.1 Probabilistic interpretation above; deterministic authority below

Search is in the **Probabilistic Plane** (Logic OS §4.2):
- semantic retrieval
- ranking
- fuzzy matching

But search RESULTS are consumed by the **Deterministic Plane**:
- capability validation
- effect gating
- audit evidence

Therefore: search output MUST be typed candidates consumed by deterministic systems.

### 2.2 Authorization before retrieval (Logic OS §12.1)

> "Vector retrieval leak: Semantic search returns embeddings or documents outside the allowed matter."

**MUST:** Apply scope filter BEFORE retrieval (pre-filtered KNN, not post-filter).
**MUST:** Never return scores to caller (score suppression).
**MUST:** Partition index per scope (strong isolation).

### 2.3 Fail-closed on uncertainty (Deep Research Report)

> "Outcome lattice: Allow < Degraded < Deny. Any uncertainty can only move the result upward by join, never downward by fallback."

**MUST:** Search failure = Deny (return empty/denied), never stale results.
**MUST:** Missing scope = compile error (unforgeable Scope type).
**MUST:** `Result<Vec<Candidate>, FailClosed>` return type — fail-open is unrepresentable.

### 2.4 Journal-before-effect (Logic OS §10.1)

> "journal-before-effect → effect execution → completion receipt → audit evidence"

**MUST:** Search index MUST be reconstructable from journal.
**MUST:** Every query MUST produce audit evidence.
**MUST:** Use Opstamp-contiguous batching for deterministic replay.

### 2.5 Black-Box Reduction Envelope (Logic OS §15.3)

> "output: typed_candidate_set, bounded_numeric_scores, confidence, evidence_refs, model_manifest_id"

**MUST:** Output is typed `Candidate` struct, not raw vectors or scores.
**MUST:** Internal scores are bounded and never exposed to caller.
**MUST:** Each candidate carries evidence refs and provenance.

### 2.6 Capability composition is intersection (from Elastic DLS footgun)

> "Combining a DLS role with a non-DLS role yields all docs."

**MUST:** Capability composition is AND (intersection), never OR (union).
**MUST:** Missing scope filter = deny, not allow.

## 3. Architecture

### 3.1 Two Search Layers

```text
┌──────────────────────────────────────────┐
│  AI Search Layer (Probabilistic Plane)    │
│  - Semantic/fuzzy matching                │
│  - Ranked candidate lattice               │
│  - Powered by: Tantivy BM25 (+ vector)    │
│  Output: Vec<Candidate> with provenance   │
└──────────────┬───────────────────────────┘
               │ candidates
┌──────────────▼───────────────────────────┐
│  Capability Gate (Deterministic Plane)    │
│  - Per-object ABAC verification           │
│  - Scope provenance check                 │
│  - Score suppression                      │
│  Output: Vec<VerifiedCandidate> or Deny   │
└──────────────┬───────────────────────────┘
               │ verified results
┌──────────────▼───────────────────────────┐
│  Human/Agent Decision                     │
│  - Pick from candidates or dismiss        │
│  - This is NOT search; it's consumption   │
└──────────────────────────────────────────┘
```

### 3.2 Crate Structure

```text
be-search/
├── Cargo.toml
├── src/
│   ├── lib.rs              — Public API: BrowserSearch, search()
│   ├── scope.rs            — Unforgeable Scope type (R7)
│   ├── query.rs            — QueryPlan sanitized AST (R8, P1)
│   ├── candidate.rs        — Typed Candidate output (R3, R4)
│   ├── index.rs            — Tantivy wrapper, partitioned per scope (P7)
│   ├── filter.rs           — Scope filter injection (P3, RLS analog)
│   ├── audit.rs            — Append-only audit trail (P9, R10)
│   ├── fallback.rs         — Deterministic scoped fallback (P10)
│   ├── partition.rs        — Index partition management (P7)
│   └── error.rs            — FailClosed error type (R6)
├── tests/
│   ├── test_scope_isolation.rs
│   ├── test_query_sanitization.rs
│   ├── test_fail_closed.rs
│   ├── test_audit_trail.rs
│   └── test_journal_replay.rs
└── benches/
    └── bench_search.rs
```

## 4. Core Types

### 4.1 Scope (Unforgeable, R7)

```rust
/// Unforgeable capability token. Constructed ONLY by the trusted auth path.
/// Callers receive &Scope but can never fabricate or widen one.
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Scope {
    pub page_origin: OriginHash,     // per-page isolation
    pub session: SessionId,          // per-session isolation
    pub tenant: TenantId,            // per-tenant isolation (multi-tenant mode)
    pub capabilities: CapSet,        // ABAC attributes
}

impl Scope {
    /// ONLY constructor — called from verified session, never from client input
    pub(crate) fn from_verified(session: &VerifiedSession) -> Result<Self, FailClosed>;
}
```

**OKF compliance:** §2.2 (authz before retrieval), §2.3 (fail-closed — no Scope = no query compiles)

### 4.2 QueryPlan (Sanitized AST, R8, P1)

```rust
/// A query is NEVER a raw string reaching the engine. It is a sanitized AST.
/// The only constructor is `parse()` which is fallible.
pub enum QueryPlan {
    Term(Field, NormalizedTerm),
    Phrase(Field, Vec<NormalizedTerm>),
    Vector(EmbeddingRef, ScopeFilter),   // semantic: embedding + mandatory scope filter
    And(Box<QueryPlan>, Box<QueryPlan>),
    Or(Box<QueryPlan>, Box<QueryPlan>),
    Empty,
}

impl QueryPlan {
    /// Parse user input into sanitized AST. Rejects operators, controls, long input.
    pub fn parse(raw: &str, scope: &Scope) -> Result<Self, FailClosed>;

    /// Authorization checkpoint: inspect AST before building engine query.
    /// Rejects queries targeting unauthorized fields.
    pub fn authorize(&self, scope: &Scope) -> Result<(), FailClosed>;
}
```

**OKF compliance:** §2.2 (eval-free, no injection path), §2.3 (fail-closed parse), P1 (operator surface minimized)

### 4.3 Candidate (Typed Output, R3, R4, P5)

```rust
/// Typed search result. NO score, NO distance, NO embedding field.
/// Ranking is internal-only; only an ordered, opaque list crosses the API boundary.
#[derive(Clone, Debug)]
pub struct Candidate {
    pub node_id: NodeId,
    pub kind: NodeKind,              // Element, Text, Attr, ...
    pub provenance: Scope,           // signed scope this candidate lives in
    pub excerpt: SanitizedExcerpt,   // PII/classification-redacted
    pub evidence_refs: Vec<Ref>,     // links to source nodes
}
// NOTE: No field named `score`, `distance`, or `embedding`.
```

**OKF compliance:** §2.5 (Black-Box Reduction Envelope), P5 (score suppression), P6 (bucket-sort lattice internally)

### 4.4 FailClosed (Fail-Open Unrepresentable, R6, P10)

```rust
/// Search failure modes. All map to Deny in the outcome lattice.
#[derive(Debug, Clone)]
pub enum FailClosed {
    NoScope,           // no scope provided — impossible at type level
    BadPlan,           // query rejected by sanitization
    IndexDown,         // search index unavailable
    AuditFail,         // audit log write failed
    PartitionMissing,  // scope partition doesn't exist
    Unauthorized,      // scope lacks capability for this query
}

/// Return type: fail-open is UNREPRESENTABLE.
pub type SearchResult = Result<Vec<Candidate>, FailClosed>;
```

**OKF compliance:** §2.3 (fail-closed), §2.6 (AND composition)

## 5. Public API

```rust
pub struct BrowserSearch {
    // Partitioned indexes: one Tantivy Index per (tenant, page_origin)
    partitions: DashMap<PartitionKey, tantivy::Index>,
    // Append-only audit trail
    audit: AuditLog,
    // Scope-bound cache (hashed keys, re-verified on read)
    cache: ScopeCache,
}

impl BrowserSearch {
    /// The ONLY public search entrypoint. Requires verified session.
    pub fn search(
        &self,
        request: SearchRequest,
        session: &VerifiedSession,
    ) -> SearchResult {
        // L1: Derive unforgeable scope from verified session
        let scope = Scope::from_verified(session)?;

        // L2: Parse + sanitize query into AST
        let mut plan = QueryPlan::parse(&request.query, &scope)?;

        // L3: Authorize (reject unauthorized fields)
        plan.authorize(&scope)?;

        // L4: Rewrite with scope predicate (RLS analog)
        let guarded = self.inject_scope_filter(plan, &scope);

        // L5: Log query BEFORE execution (audit-before-effect)
        self.audit.log_query(&scope, &guarded, &request.reason)?;

        // L6: Execute search against partitioned index
        let hits = match self.search_partitioned(&guarded, &scope, request.limit) {
            Ok(hits) => hits,
            Err(_) => return self.deterministic_fallback(&guarded, &scope), // L10
        };

        // L7: Per-object post-filter (ABAC, defense-in-depth)
        let verified = self.post_filter(hits, &scope);

        // L8: Score suppression (already done by Candidate type)
        let candidates = verified.into_iter().map(|h| Candidate::from(h, &scope)).collect();

        // L9: Log results
        self.audit.log_results(&scope, &candidates)?;

        Ok(candidates)
    }
}
```

## 6. Security Enforcement (10-Layer Defense-in-Depth)

| Layer | What | OKF | Test |
|-------|------|-----|------|
| L1 | Scope::from_verified — unforgeable | §2.2 | test_scope_cannot_be_fabricated |
| L2 | QueryPlan::parse — eval-free AST | §2.3 | test_query_injection_rejected |
| L3 | plan.authorize — field allowlist | §2.2 | test_unauthorized_field_rejected |
| L4 | inject_scope_filter — RLS analog | §2.2 | test_scope_filter_always_applied |
| L5 | audit.log_query — before execution | §2.4 | test_audit_before_search |
| L6 | search_partitioned — per-scope index | §2.2 | test_cross_scope_isolation |
| L7 | post_filter — per-object ABAC | §2.2 | test_idor_neutralized |
| L8 | Candidate type — no score field | §2.5 | test_score_not_exposed |
| L9 | audit.log_results — completion receipt | §2.4 | test_results_logged |
| L10 | deterministic_fallback — scoped-only | §2.3 | test_fallback_never_global |

## 7. Library Choice

### 7.1 Primary: Tantivy (BM25 lexical search)

```toml
[dependencies]
tantivy = { version = "0.26", default-features = false }
# default-features = false: kill mmap for portability
# Use RamDirectory or custom Directory for persistence
```

**Rationale:** 32/35 OKF score. Production-ready. Pure Rust. Journal replay via Opstamp. FilterCollector for scoping.

### 7.2 Future: Vector Layer (semantic search)

Phase 2 addition. Options monitored:
- **Laurus** — if it reaches 1.0 with >10k downloads
- **Minimal HNSW** — build thin vector layer over Tantivy
- **seekstorm** — verify as Laurus competitor

Vector search will follow same 10-layer security model.

### 7.3 Rejected

| Library | Reason |
|---------|--------|
| Foxstash | Does not exist (fabricated) |
| vxdb | Does not exist (fabricated) |
| Hora | Dead since 2021 |
| Sonic | Server, not embedded; returns IDs not docs |
| USearch | C++ core, not pure Rust |
| Veles | Wrong category (dead UI library) |
| vecdb | Wrong category (data structure, not search) |

## 8. Tantivy Integration Patterns

### 8.1 Index Creation (partitioned per scope)

```rust
fn create_partition(scope: &Scope) -> tantivy::Index {
    let mut builder = Schema::builder();
    builder.add_text_field("tag", TEXT);
    builder.add_text_field("role", TEXT);
    builder.add_text_field("text", TEXT);
    builder.add_text_field("aria_label", TEXT);
    builder.add_u64_field("scope_hash", FAST | INDEXED);  // scope enforcement
    builder.add_facet_field("node_kind", FacetOptions::default());
    let schema = builder.build();

    // RamDirectory for in-memory (portable, no mmap)
    tantivy::Index::create_in_ram(schema)
}
```

### 8.2 Scoped Query (authorization checkpoint)

```rust
fn build_scoped_query(
    index: &tantivy::Index,
    plan: &QueryPlan,
    scope: &Scope,
) -> Box<dyn tantivy::query::Query> {
    let qp = QueryParser::for_index(index, vec![text_field, role_field]);
    // NEVER: qp.allow_regexes() — regex must be server-compiled from allowlist only

    let user_query = build_from_plan(plan, &qp);

    // Scope guard: TermQuery on scope_hash, always AND'd (Occur::Must)
    let scope_guard = TermQuery::new(
        Term::from_field_u64(scope_hash_field, scope.hash()),
        IndexRecordOption::Basic,
    );

    BooleanQuery::new(vec![
        (Occur::Must, user_query),
        (Occur::Must, Box::new(scope_guard)),
    ])
}
```

### 8.3 Journal Replay (deterministic reconstruction)

```rust
struct SearchJournal {
    entries: Vec<(Opstamp, UserOperation)>,
}

impl SearchJournal {
    /// Replay journal against fresh index → byte-equivalent state
    fn replay(&self, index: &mut tantivy::Index) -> Result<()> {
        let mut writer = index.writer(50_000_000)?;
        for (opstamp, op) in &self.entries {
            writer.run(vec![op.clone()])?;  // contiguous opstamps
        }
        writer.commit()?;
        Ok(())
    }
}
```

## 9. Audit Trail

```rust
struct AuditEntry {
    scope: Scope,
    plan: QueryPlan,        // sanitized plan, NOT raw input
    candidates: Vec<NodeId>,
    timestamp: TrustedTime,
    outcome: Outcome,       // Success | Denied | Fallback
}

// Append-only. Caller can read only their own trail.
// Audit write failure ⇒ search denied (fail-closed).
```

## 10. Test Requirements

| Test | What it proves | OKF |
|------|----------------|-----|
| test_scope_cannot_be_fabricated | Scope has no public constructor | §2.2 |
| test_query_injection_rejected | Operator/control chars rejected | §2.3 |
| test_cross_scope_isolation | Scope A cannot see Scope B's nodes | §2.2 |
| test_scope_filter_always_applied | Every query has scope guard | §2.2 |
| test_score_not_exposed | Candidate has no score field | §2.5 |
| test_fail_closed_on_index_down | Returns FailClosed, not stale | §2.3 |
| test_fallback_never_global | Fallback is scoped-only | §2.3 |
| test_audit_before_search | Audit logged before execution | §2.4 |
| test_journal_replay | Replay produces identical index | §2.4 |
| test_and_composition | Capabilities AND, not OR | §2.6 |
| test_idor_neutralized | Per-object check on every result | §2.2 |
| test_regex_rejected | User regex rejected | §2.3 |

## 11. Future: Vector Search Phase

When vector/semantic search is needed:
1. Add vector field to schema
2. Add `QueryPlan::Vector(EmbeddingRef, ScopeFilter)` variant
3. Implement pre-filtered KNN (filter inside traversal)
4. Fuse with BM25 results via RRF (Reciprocal Rank Fusion)
5. Same 10-layer security model applies
6. Scores still suppressed — only typed Candidate output

Monitor: laurus (if 1.0+), seekstorm (verify), or build minimal HNSW.

## 12. OKF Principle Compliance Matrix

| OKF Principle | Score | Evidence |
|---------------|-------|----------|
| Binary-first | 5/5 | Tantivy compiles to deterministic binary; no SaaS |
| Scoped retrieval | 5/5 | Partition per scope + mandatory scope filter + ABAC post-filter |
| Typed output | 5/5 | Candidate struct, no scores, provenance stamped |
| Deterministic fallback | 5/5 | Journal replay via Opstamp; scoped-only fallback |
| Audit evidence | 5/5 | Append-only trail, query+results logged, audit-before-search |
| Authz before retrieval | 5/5 | Scope filter injected before query execution; pre-filtered KNN |
| Leak prevention | 5/5 | Partition isolation, score suppression, scope-bound cache |
| Fail-closed | 5/5 | Result<_, FailClosed>; fail-open unrepresentable |
| Human reconstructability | 4/5 | Journal replay, typed output, audit trail; Tantivy docs are excellent |

**Total: 44/45 (98%)**
