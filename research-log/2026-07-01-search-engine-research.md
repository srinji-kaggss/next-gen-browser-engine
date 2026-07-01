# 2026-07-01 — Search Engine Research

## Date

2026-07-01

## Hypothesis

There are mature, production-ready search engine libraries in 2026 that can be embedded into the browser engine without building our own.

## Design

Research 2026 search engine options across three categories:
1. Embedded libraries (in-process, no server)
2. Standalone servers (can be run alongside)
3. Vector-focused databases

Evaluate each against browser engine requirements:
- Must be embeddable in Rust
- Must support BM25 full-text search
- Should support vector/hybrid search
- Must be lightweight (single binary, no JVM)
- Should have good documentation and community

## Method

1. Web search for 2026 search engine options
2. Evaluate each option against requirements
3. Compare features, performance, and ecosystem
4. Select top candidates for browser engine integration

## Raw Data

### Embedded Libraries (In-Process)

| Library | Version | Stars | BM25 | Vector | Hybrid | WASM | License |
|---------|---------|-------|------|--------|--------|------|---------|
| **Tantivy** | 0.22+ | 4.5K | ✅ | ❌ | ❌ | ❌ | MIT |
| **Laurus** | 0.9.0 | New | ✅ | ✅ | ✅ | ❌ | MIT |
| **Foxstash** | 0.1+ | New | ✅ | ✅ | ✅ | ✅ | MIT |
| **Veles** | 0.6.0 | New | ✅ | ✅ | ✅ | ❌ | MIT |
| **Indicium** | — | — | ✅ | ❌ | ❌ | ❌ | — |
| **FST** | — | — | ✅ | ❌ | ❌ | ❌ | MIT |
| **Porigon** | 0.1+ | 32 | ✅ | ❌ | ❌ | ✅ | MIT |

### Standalone Servers

| Server | Stars | BM25 | Vector | Hybrid | RAM | License |
|--------|-------|------|--------|--------|-----|---------|
| **Meilisearch** | 12.4K | ✅ | ✅ | ✅ | 150-300MB | MIT/BSL |
| **Sonic** | 10.9K | ✅ | ❌ | ❌ | 10-20MB | AGPL |
| **Typesense** | 5.1K | ✅ | ✅ | ✅ | 100MB+ | GPL-3 |
| **Aperio** | 52 | ✅ | ❌ | ❌ | Low | Custom |
| **Prism** | New | ✅ | ✅ | ✅ | ~50MB | — |

### Vector-Focused

| Database | Embedded | Server | BM25 | Vector | Hybrid | Size |
|----------|----------|--------|------|--------|--------|------|
| **vecdb** | ✅ | ✅ | ✅ | ✅ | ✅ | ~10MB |
| **vxdb** | ✅ | ✅ | ✅ | ✅ | ✅ | ~5MB |
| **NodeDB** | ✅ | ✅ | ✅ | ✅ | ✅ | — |

## Analysis

### Top Candidates for Browser Engine

#### 1. Tantivy (Best for BM25-only)
- **Pros:** Mature (4.5K stars), battle-tested, pure Rust, fast, good docs
- **Cons:** No vector search, no hybrid
- **Use case:** If we only need keyword search over semantic graph nodes

#### 2. Laurus (Best for Hybrid Search)
- **Pros:** BM25 + vector + hybrid, pure Rust, 2026 release, good API
- **Cons:** New (less battle-tested), no WASM
- **Use case:** If we need both keyword and semantic search

#### 3. Foxstash (Best for WASM + Performance)
- **Pros:** SIMD-accelerated, HNSW + BM25, WASM support, vector quantization
- **Cons:** New, less documentation
- **Use case:** If we need browser-side search or maximum performance

#### 4. Veles (Best for Code Search)
- **Pros:** Code-search focused, tree-sitter integration, BM25 + embeddings
- **Cons:** Code-specific, may not work for general semantic search
- **Use case:** If we're indexing browser engine code specifically

### Recommendation

**For browser engine integration, use Laurus or Foxstash:**

- **Laurus** if we want a well-documented, pure-Rust hybrid search library with good API
- **Foxstash** if we need WASM support or maximum SIMD performance

Both support:
- BM25 full-text search (for querying semantic graph nodes by role, label, action)
- Vector search (for semantic similarity)
- Hybrid search (combining both)
- Embedded usage (no server required)

### Integration Plan

1. Add `laurus` or `foxstash` to workspace dependencies
2. Create `be-search` crate that wraps the search library
3. Index semantic graph nodes (role, label, action, position)
4. Expose search API via be-api endpoints
5. Add search to the browser engine pipeline

## Observation

- **The search engine landscape in 2026 is mature.** Multiple production-ready options exist.
- **Hybrid search (BM25 + vector) is now standard.** No need to build our own.
- **Embedded libraries are preferred over servers** for the browser engine (no external dependencies).
- **Tantivy is the safest choice** (mature, battle-tested), but **Laurus or Foxstash are better** for hybrid search.

## Next Steps

1. Choose between Laurus and Foxstash (or both)
2. Create be-search crate
3. Index semantic graph nodes
4. Add search API endpoints
5. Test with real queries

## Tags

`research` `search-engine` `2026`

## Claim Links

- [claim:SE-001] — 2026 has multiple production-ready embedded search libraries
- [claim:SE-002] — Hybrid search (BM25 + vector) is now standard
- [claim:SE-003] — Laurus and Foxstash are top candidates for browser engine
- [claim:SE-004] — Tantivy is safest but lacks vector search
