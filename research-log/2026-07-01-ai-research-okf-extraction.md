# 2026-07-01 — AI Research OKF Extraction: Search Infrastructure for Browser Engine

## Date

2026-07-01

## Hypothesis

The Logic OS research corpus defines a precise architecture for search/retrieval that the browser engine must follow. The search infrastructure is not just "a search bar" — it is a state fabric component with specific security, scoping, and deterministic requirements.

## Design

Deeply extract search-relevant patterns from three primary research documents:
1. Logic OS Research Baseline (1751 lines) — state fabric, semantic index, security posture
2. Agent Handoff Framework (1138 lines) — state engine, declarative schemas, production layers
3. Deep Research Report (73 lines) — WORM sink, journal architecture, fail-closed patterns

Map extracted patterns to browser engine implementation requirements.

## Method

1. Read all three documents in full
2. Extract search/retrieval-specific patterns
3. Map patterns to browser engine architecture
4. Identify gaps and requirements

## Raw Data

### Pattern 1: Semantic Index as State Fabric (Logic OS §5.1)

The research defines **eight canonical state fabrics**. The Semantic Index is one:

```yaml
semantic_index:
  purpose: "Embeddings and searchable summaries for local retrieval"
  storage_form: "Local vector index with scoped retrieval"
```

**Browser engine mapping:** `be-search` IS the Semantic Index for the browser engine. It indexes:
- DOM nodes (role, label, action, position)
- Semantic graph nodes (type, relationships)
- Page state (URL, title, metadata)

### Pattern 2: Scoped Retrieval (Logic OS §12.1)

The research explicitly warns about search security:

```yaml
vector_retrieval_leak:
  risk: "Semantic search returns embeddings or documents outside the allowed matter"
  controls:
    - "Authorization before retrieval"
    - "Metadata scope filters"
    - "Post-filter verification"
```

**Browser engine mapping:** Search queries MUST be scoped:
- Per-page (only search nodes from the current page)
- Per-session (only search within the active session)
- Per-tenant (if multi-tenant, only search within the tenant)

### Pattern 3: Deterministic vs Probabilistic Split (Logic OS §4.2)

The research defines two runtime planes:

```yaml
probabilistic_plane:
  - summarization
  - intent extraction
  - recommendation ranking
  - resource tuning
  - semantic retrieval  # <-- SEARCH LIVES HERE
  - local model inference

deterministic_plane:
  - identity verification
  - capability validation
  - tenant isolation
  - effect gating
  - journal-before-effect
  - replay
  - key custody
  - artifact admission
  - audit evidence
```

**Browser engine mapping:** 
- `be-search` is probabilistic (ranking, relevance, fuzzy matching)
- But search results are consumed by deterministic systems (capability checks, effect gating)
- Therefore: search results MUST be typed candidates with bounded scores, not raw vectors

### Pattern 4: Black-Box Reduction Envelope (Logic OS §15.3)

ML/search output must be wrapped:

```yaml
black_box_reduction_envelope:
  input:
    - schema_versioned_features_only
    - tenant_scoped_context_only
    - minimized_data
    - redacted_secrets
  output:
    - typed_candidate_set
    - bounded_numeric_scores
    - confidence
    - evidence_refs
    - model_manifest_id
  runtime_controls:
    - calibration_threshold
    - out_of_distribution_detection
    - deterministic_fallback
    - reversible_action_only_by_default
    - resource_budget
    - kill_switch
    - rollback
    - audit_record
  prohibited:
    - raw_shell
    - raw_sql
    - ambient_credentials
    - direct_irreversible_effect
    - self_authorized_capability_expansion
    - unsigned_runtime_component
```

**Browser engine mapping:** `be-search` output MUST be:
```rust
struct SearchResult {
    node_id: NodeId,           // typed candidate
    score: f64,                // bounded numeric score
    confidence: f64,           // confidence level
    evidence_refs: Vec<Ref>,   // links to source nodes
    scope: Scope,              // page/session/tenant scope
}
```

NOT raw vectors or untyped results.

### Pattern 5: Intent Lattice (Logic OS §6.3)

The research defines a lattice of competing intents:

```yaml
intent_lattice:
  trajectory_id: trajectory-123
  candidates:
    - intent: draft_followup_email
      confidence: 0.61
      evidence:
        - meeting_ended
        - commitment_detected
        - client_record_open
    - intent: update_ticket
      confidence: 0.25
      evidence:
        - ticket_reference
        - unresolved_state
    - intent: schedule_review
      confidence: 0.14
      evidence:
        - followup_date_mentioned
  model_manifest: local-intent-8b-v4
  policy_snapshot: policy-882
```

**Browser engine mapping:** Search results ARE an intent lattice:
```rust
struct SearchLattice {
    query_id: QueryId,
    candidates: Vec<SearchResult>,  // ranked by score
    scope: Scope,
    timestamp: TrustedTime,
}
```

### Pattern 6: State Engine Ownership (Agent Handoff §7)

The state engine owns:

```yaml
state_engine:
  owns:
    - StateGraph
    - MutationLog
    - Snapshot
  receives:
    - Command
    - Event
  emits:
    - StateDiff
    - RenderInvalidation
    - MotionTrigger
```

**Browser engine mapping:** `be-state` manages page state, `be-search` queries it:
- `be-state` owns the state graph (DOM, semantic graph, page metadata)
- `be-search` indexes the state graph for retrieval
- Search queries receive Commands, return SearchResult sets

### Pattern 7: Eleven Canonical Kernel Engines (Logic OS §10)

The research defines eleven engines. Three are search-relevant:

```yaml
engines:
  2_capability_vocabulary:
    question: "Is the requested action nameable in the closed registry?"
    browser_mapping: "be-search validates that search targets exist in the capability registry"
  
  3_authority_rights_calculus:
    question: "Does this principal hold the scoped right now?"
    browser_mapping: "be-search results are filtered by capability scope"
  
  8_durable_journal_replay:
    question: "Can the state recover deterministically after interruption?"
    browser_mapping: "be-search index must be reconstructable from the journal"
```

### Pattern 8: Database Vulnerability Posture (Logic OS §12)

Search-specific vulnerabilities:

```yaml
search_vulnerabilities:
  injection:
    risk: "Raw string concatenation reaches search filters"
    control: "Prepared statements, typed query builders, no raw dynamic query execution"
  
  graph_traversal_leak:
    risk: "A traversal crosses tenant or policy boundaries through an edge"
    control: "Scoped graph traversal, edge-level policy, tenant-stamped nodes and edges"
  
  vector_retrieval_leak:
    risk: "Semantic search returns embeddings or documents outside the allowed matter"
    control: "Authorization before retrieval, metadata scope filters, post-filter verification"
  
  log_leakage:
    risk: "Secrets, tokens, PII, prompts, or model context enter logs"
    control: "Structured redaction, secret types, log schema linting, retention controls"
```

**Browser engine mapping:** `be-search` MUST:
- Use typed query builders (no raw string interpolation)
- Scope traversal to current page/session
- Filter results by capability scope
- Redact sensitive data from search logs

### Pattern 9: Fail-Closed Architecture (Deep Research Report)

The research defines outcome lattice:

```yaml
outcome_lattice:
  ordering: "Allow < Degraded < Deny"
  rule: "Any uncertainty, integrity failure, or policy failure can only move the result upward by join, never downward by fallback"
```

**Browser engine mapping:** Search failure modes:
- Search unavailable → Deny (don't return stale results)
- Search timeout → Degraded (return partial results with warning)
- Search scope violation → Deny (never return out-of-scope results)

### Pattern 10: Journal-Before-Effect (Logic OS §10.1)

Required order for governed action:

```text
identity + trusted time
→ closed capability vocabulary
→ scoped authority
→ tenant isolation
→ IPC re-stamp
→ resource arbiter
→ projected outcome simulation
→ irreversibility gate
→ journal-before-effect
→ effect execution
→ completion receipt
→ audit evidence
```

**Browser engine mapping:** Search is a "read" effect, but still needs:
- Identity (who is searching)
- Trusted time (when)
- Scope (what is searchable)
- Audit evidence (what was searched, what was returned)

## Analysis

### What the research tells us about be-search

1. **be-search is a state fabric component**, not just a search bar
2. **Search results MUST be typed candidates** with bounded scores, not raw vectors
3. **Search MUST be scoped** to page/session/tenant
4. **Search MUST have deterministic fallback** when unavailable
5. **Search MUST produce audit evidence** for every query
6. **Search MUST use typed query builders** (no raw string interpolation)
7. **Search index MUST be reconstructable** from the journal
8. **Search output follows the Black-Box Reduction Envelope** pattern

### What this means for library choice

The search library must support:
- **Scoped indexing** (partition by page, session, tenant)
- **Typed output** (not just raw scores, but structured results)
- **BM25 + vector hybrid** (for both exact and semantic search)
- **Embedded usage** (no external server)
- **Deterministic fallback** (can reconstruct index from scratch)

**Laurus** fits all these requirements:
- Hybrid BM25 + vector search
- Embedded (no server)
- Pure Rust
- Good API for typed output

**Foxstash** also fits:
- SIMD-accelerated
- WASM support (could run in browser-side)
- HNSW + BM25

### Implementation requirements

```rust
// be-search crate structure
pub struct BrowserSearch {
    // Laurus or Foxstash index
    index: SearchIndex,
    
    // Scope enforcement
    scope: Scope,
    
    // Audit trail
    audit_log: AuditLog,
}

pub struct SearchQuery {
    text: String,
    scope: Scope,
    max_results: usize,
    min_score: f64,
}

pub struct SearchResult {
    node_id: NodeId,
    score: f64,
    confidence: f64,
    evidence_refs: Vec<Ref>,
    scope: Scope,
}

pub struct SearchLattice {
    query_id: QueryId,
    candidates: Vec<SearchResult>,
    scope: Scope,
    timestamp: TrustedTime,
}
```

## Observation

- **The research is extremely specific** about search architecture — it's not optional, it's a state fabric component
- **Search security is explicitly addressed** — scope, authorization, post-filter verification
- **Search output must be typed** — not raw vectors, but structured candidates with evidence
- **Search must be deterministic** — reconstructable from journal, fallback when unavailable
- **The research maps directly to browser engine** — Semantic Index, Intent Lattice, Black-Box Reduction Envelope all apply

## Next Steps

1. **Add Laurus or Foxstash** to workspace dependencies
2. **Create be-search crate** with scoped indexing
3. **Implement SearchLattice** pattern (typed candidates with evidence)
4. **Add scope enforcement** (per-page, per-session)
5. **Add audit logging** (what was searched, what was returned)
6. **Add deterministic fallback** (reconstruct index from journal)
7. **Wire be-search into be-api** (new endpoint: `/search`)

## Tags

`research` `ai-research-okf` `search-infrastructure` `state-fabric` `2026`

## Claim Links

- [claim:SI-001] — Semantic Index is a state fabric component, not just a search bar
- [claim:SI-002] — Search results MUST be typed candidates with bounded scores
- [claim:SI-003] — Search MUST be scoped to page/session/tenant
- [claim:SI-004] — Search MUST have deterministic fallback when unavailable
- [claim:SI-005] — Search MUST produce audit evidence for every query
- [claim:SI-006] — Search index MUST be reconstructable from the journal
- [claim:SI-007] — Laurus or Foxstash are suitable libraries for be-search
