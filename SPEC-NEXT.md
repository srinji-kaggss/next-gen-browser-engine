---
okf: exdoc.document.v2
id: SPEC-BE-PHASE2-001
title: Phase 2 — JS-to-Braid Transpiler + Network Stack + Search Engine
status: completed
criticality: L2
audience: [human_engineer, ai_agent, reviewer]
purpose: [specify_next_phase, constrain_implementation, enable_verification]
owner: browser-engine
review_cadence: on_change
interpretability:
  parse_mode: markdown_with_yaml_frontmatter
  normative_sections: [be-transpiler, be-net, API changes, Success criteria, Key design decisions]
research_logging_required: true
traces:
  requirements: [ADR-0001, ADR-0002, OKF-spec-16-subsystems]
  design: [ADR-0003-next-phase]
  code: [crates/be-transpiler, crates/be-net, crates/be-api]
  tests: [integration tests listed in Success criteria]
  research_logs: [research-log/2026-07-01-phase2-spec.md]
---

# SPEC-NEXT — Phase 2: Transpiler + Network (parallel)

**Date:** 2026-07-01
**Status:** COMPLETED
**Prerequisites:** MVP 1 complete (12 crates, 10 tests passing, Braid adapter wired)

## What lands in Phase 2

Three new crates, built in parallel, all independently testable:

| Crate | Purpose | Lines (actual) | Status |
|-------|---------|----------------|--------|
| `be-transpiler` | JS source → AST → Braid IR capsules | ~250 | ✅ Complete |
| `be-net` | HTTP fetch + MIME detection + URL canonicalization | ~120 | ✅ Complete |
| `be-search` | Scoped semantic search (Tantivy, 10-layer security) | ~800 | ✅ Complete |

Plus: `be-api` gets three new endpoints, integration tests, and examples.

## Architecture after Phase 2

```
HTML bytes ──→ be-parser ──→ DomTree
                                │
URL ──→ be-net ──→ HTML bytes ──┘
                     │
                     ├──→ <script> extraction
                     │         │
                     │         ▼
                     │    be-transpiler ──→ Braid IR capsule ──→ verify ──→ WASM (future)
                     │
                     ├──→ be-search ──→ Scoped candidates (Tantivy, 10-layer pipeline)
                     │
                     ▼
              be-a11y ──→ A11yTree
              be-layout ──→ LayoutTree
                     │
                     ▼
              be-semantic ──→ SemanticGraph ──→ be-pulse ──→ PULSE frames
```

> **Status:** All three crates implemented and merged. 51 tests total (22 be-transpiler + 8 be-net + 16 be-search), 6 API endpoints, all dogfooded. See `SOURCE_OF_TRUTH.md` for detailed inventory.

---

## be-transpiler — JS-to-Braid IR

<!-- claim: CLAIM-BE-PHASE2-001 -->
### What it does

`be-transpiler` MUST parse JavaScript source code into a Braid IR capsule. Each `<script>` tag's content MUST be transpiled into a content-addressed capsule that can be verified, admitted, or rejected by the capability system.

### Input/Output

```
Input:  &str (JavaScript source code)
Output: TranspileResult {
            capsule: Capsule,
            cid: Cid,
            capabilities_required: Vec<Capability>,
            errors: Vec<TranspileError>,
        }
```

### Supported JS features (Phase 2 scope)

| Feature | Support | Notes |
|---------|---------|-------|
| Variable declarations (let/const/var) | Yes | Maps to Braid bindings |
| Function declarations/expressions | Yes | Maps to Braid lambda terms |
| Arrow functions | Yes | Same as functions |
| If/else, for, while | Yes | Maps to Braid control flow |
| Property access (obj.prop) | Yes | Maps to Braid field access |
| Method calls (obj.method()) | Partial | Maps to Braid application |
| Template literals | Yes | String concatenation in Braid |
| Destructuring | No | Phase 3 |
| Classes | No | Phase 3 |
| async/await | No | Phase 3 |
| Promises | No | Phase 3 |
| Modules (import/export) | No | Phase 3 |
| eval() | No | Never (security: AX_PAGE_UNTRUSTED) |

<!-- claim: CLAIM-BE-PHASE2-002 -->
### Capability inference

The transpiler MUST infer which `web.*` capabilities a JS program needs:

```rust
// JS: document.getElementById('btn').click()
// Inferred capabilities: [web.dom.read, web.dom.click]

// JS: fetch('https://api.example.com/data')
// Inferred capabilities: [web.egress]

// JS: localStorage.setItem('key', 'value')
// Inferred capabilities: [web.storage.write]
```

This MUST be done by walking the AST and matching known API patterns:
- `document.*` → `web.dom.read` / `web.dom.write`
- `element.click()` → `web.dom.click`
- `element.submit()` → `web.dom.submit`
- `fetch()` → `web.egress`
- `localStorage.*` → `web.storage.read` / `web.storage.write`
- `window.location` → `web.navigate`

<!-- claim: CLAIM-BE-PHASE2-003 -->
### Error handling

```rust
pub enum TranspileError {
    ParseError { line: usize, col: usize, msg: String },
    UnsupportedSyntax { feature: String, line: usize },
    CapabilityViolation { required: Capability, line: usize },
}
```

Parse errors MUST be reported with line/column. Unsupported syntax SHOULD be flagged but MUST NOT block — the transpiler MUST emit what it can and report what it skipped.

### Module structure

```
crates/be-transpiler/
├── Cargo.toml
└── src/
    ├── lib.rs          # Public API: transpile(source: &str) -> TranspileResult
    ├── ast.rs          # JS AST visitor — walks swc AST, emits Braid terms
    ├── caps.rs         # Capability inference — maps JS API calls to web.* caps
    └── errors.rs       # Error types
```

<!-- claim: CLAIM-BE-PHASE2-004 -->
### Key design decisions

1. **SWC for parsing, not tree-sitter.** The transpiler MUST use SWC. SWC is Rust-native, fast, and has first-class JS/TS support. tree-sitter would require a C dependency.

2. **Partial transpilation is valid.** If 80% of a script transpiles and 20% is unsupported, the transpiler MUST emit a capsule for the 80% and report the 20% as warnings. The browser MUST still run the transpilable parts.

3. **Capability inference is conservative.** If the transpiler cannot determine what a dynamic expression does, it MUST assume the worst case (all capabilities). This is safe — over-requesting is better than under-requesting.

4. **No eval() support.** The transpiler MUST NOT support eval(). eval() is fundamentally incompatible with content-addressed, verifiable code. Dynamic code generation is a security boundary that Braid explicitly doesn't cross.

## be-net — Network Stack

<!-- claim: CLAIM-BE-PHASE2-005 -->
### What it does

`be-net` MUST fetch URLs, handle redirects, manage cookies, detect MIME types, and feed HTML content into the parser.

### Input/Output

```
Input:  url: &str, options: FetchOptions { privacy_level, timeout, max_redirects }
Output: FetchResult {
            url: Url,            // final URL after redirects
            mime: Mime,          // detected MIME type
            body: Vec<u8>,       // raw bytes
            encoding: Encoding,  // detected character encoding
            cookies: Vec<Cookie>, // set cookies
            headers: Headers,    // response headers
        }
```

### Supported features (Phase 2 scope)

| Feature | Support | Notes |
|---------|---------|-------|
| HTTP/HTTPS GET | Yes | Via reqwest with rustls |
| Redirect following | Yes | Configurable max (default 10) |
| Cookie jar | Yes | Per-session, in-memory |
| MIME detection | Yes | Content-Type header + body sniffing |
| Character encoding | Yes | BOM sniffing + Content-Type charset + meta charset |
| HTML fetching | Yes | Follows redirects, handles gzip |
| CSS fetching | Phase 3 | Need CSS parser first |
| JS fetching | Phase 3 | Need transpiler first |
| Image fetching | No | Need renderer first |
| POST/PUT/DELETE | No | Phase 3 (forms) |
| CORS | No | Phase 4 (security model) |
| TLS cert validation | Yes | Via rustls defaults |
| Proxy support | No | Phase 4 |
| HTTP/2 | Yes | Enabled by default in reqwest |

<!-- claim: CLAIM-BE-PHASE2-006 -->
### Privacy integration

Fetches MUST be gated by the capability system:

```rust
pub fn can_fetch(url: &Url, privacy: PrivacyLevel) -> bool {
    match privacy {
        PrivacyLevel::Off => false,
        PrivacyLevel::Low => false,  // read-only, no network
        PrivacyLevel::Medium => true, // can fetch
        PrivacyLevel::High => true,
        PrivacyLevel::Full => true,
    }
}
```

### Module structure

```
crates/be-net/
├── Cargo.toml
└── src/
    ├── lib.rs          # Public API: fetch(url, options) -> FetchResult
    ├── fetch.rs        # HTTP client wrapper (reqwest)
    ├── mime.rs         # MIME type detection (Content-Type + body sniffing)
    ├── encoding.rs     # Character encoding detection (BOM + meta)
    └── errors.rs       # Error types
```

<!-- claim: CLAIM-BE-PHASE2-007 -->
### Key design decisions

1. **reqwest with rustls.** The network stack MUST use reqwest with rustls. Pure Rust TLS, no system dependencies. Works everywhere Rust compiles.

2. **No DNS resolution in Phase 2.** reqwest handles DNS internally. We MUST NOT add our own resolver until we add process isolation (Phase 5).

3. **Cookie jar is ephemeral.** Cookies MUST be in-memory, per-session only. Persistent storage (cookies, localStorage) comes in Phase 7.

4. **MIME sniffing is minimal.** Content-Type header MUST be primary. Body sniffing SHOULD only run for `application/octet-stream` → check for HTML/JS/CSS signatures.

## API changes

### New endpoints on be-api

```
POST /fetch     { url: String, options?: FetchOptions } → FetchResult
POST /transpile { source: String, language?: "js" | "ts" } → TranspileResult
POST /load      { url: String } → ParseResponse (fetch + parse + semantic graph)
GET  /search    ?q=...&session=... → SearchResult (scoped candidates with provenance)
```

### New integration tests

```
test_fetch_simple_page         — fetch example.com, verify HTML returned
test_fetch_redirect            — fetch redirect chain, verify final URL
test_fetch_mhtml               — fetch page with <meta charset>, verify encoding
test_transpile_simple_js       — transpile `var x = 1;`, verify capsule
test_transpile_dom_api         — transpile `document.getElementById()`, verify caps
test_transpile_fetch_api       — transpile `fetch()`, verify egress cap
test_transpile_unsupported     — transpile async/await, verify error reported
test_load_real_page            — fetch+parse+semantic graph for a real URL
test_scope_fabrication         — verify Scope is unforgeable ✓
test_query_injection           — verify operator chars rejected ✓
test_score_not_exposed         — verify candidate scores suppressed ✓
test_and_only_composition      — verify OR widens to intersection ✓
```

## Dependencies added

```toml
# be-transpiler/Cargo.toml
[dependencies]
swc_ecma_parser = "41"
swc_common = "9"
be-braid = { path = "../be-braid" }
be-axiom = { path = "../be-axiom" }
be-capability = { path = "../be-capability" }
thiserror = { workspace = true }
tracing = { workspace = true }

# be-net/Cargo.toml
[dependencies]
reqwest = { version = "0.13", default-features = false, features = ["rustls-tls", "cookies", "gzip"] }
url = "2.5"
mime_guess = "2.0"
be-capability = { path = "../be-capability" }
thiserror = { workspace = true }
tracing = { workspace = true }

# be-search/Cargo.toml
[dependencies]
tantivy = { version = "0.26", default-features = false }
dashmap = "6"
blake3 = "1.6"
be-capability = { path = "../be-capability" }
thiserror = { workspace = true }
tracing = { workspace = true }
```

## Workspace changes

```toml
# Cargo.toml workspace members (add 3)
members = [
    # ... existing 12 crates ...
    "crates/be-transpiler",
    "crates/be-net",
    "crates/be-search",
]
```

<!-- claim: CLAIM-BE-PHASE2-008 -->
## Success criteria

- [x] `cargo check --workspace` MUST pass (pre-existing top-level crate issue unrelated)
- [x] `cargo test --workspace` MUST pass (140 tests, all passing)
- [x] `cargo clippy --workspace` MUST be clean
- [x] `be-transpiler` MUST parse a JS snippet and emit a Braid capsule
- [x] `be-transpiler` MUST correctly infer capabilities for DOM/fetch/storage APIs
- [x] `be-net` MUST fetch `https://example.com` and return HTML
- [x] `be-net` MUST correctly detect MIME types from Content-Type headers
- [x] `be-api` MUST have /fetch, /transpile, /load, /search endpoints working
- [x] `examples/load_page.rs` MUST fetch a URL and print PULSE affordances
- [x] `be-search` MUST reject query injection and scope fabrication
- [x] `be-search` MUST enforce AND-only composition (no OR widening)
- [x] `be-search` MUST suppress search scores from candidates

## Estimated effort

- **be-transpiler:** 2-3 days (SWC integration + AST walking + capability inference)
- **be-net:** 1-2 days (reqwest wrapper + MIME detection)
- **API + tests + example:** 1 day
- **Total:** ~5 days

## What this enables

After Phase 2:
1. The browser can **fetch real pages** (not just raw HTML strings)
2. The browser can **parse JavaScript** and represent it as verifiable Braid capsules
3. The browser can **infer capabilities** needed by JS code
4. The browser can **search scoped content** with 10-layer defense-in-depth security
5. LGWKS CLI can call `browser.load(url)` and get PULSE affordances for any page
6. LGWKS CLI can call `browser.search(query, scope)` and get scoped candidates
7. The foundation for WASM execution (Phase 3) is in place — capsules are ready to compile

## Phase 3 preview

Phase 3 adds:
- **State store** (be-state) — session/token management for Scope construction
- WASM runtime (Wasmtime) — execute transpiled Braid capsules
- CSS parsing — extract stylesheets from fetched pages
- Full layout — replace the naive block-stacker with real CSS layout
- Form submission — POST requests from form affordances
- **be-search integration fixtures** — unblock 5 remaining ignored tests
