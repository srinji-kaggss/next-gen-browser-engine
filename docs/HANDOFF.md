# Handoff: Browser Engine — Foundation + Ingestion + Policy/State/URL Seams

**Date:** 2026-06-18  
**Repo:** `srinji-kaggss/next-gen-browser-engine`  
**Branch:** `foundation/aip-integration`  
**Merged PR:** [#8](https://github.com/srinji-kaggss/next-gen-browser-engine/pull/8) — *feat: browser-engine ingestion + deterministic policy/state/URL seams* (8ca1ab3 on main).  
**In-flight PR:** [#10](https://github.com/srinji-kaggss/next-gen-browser-engine/pull/10) — *feat: braid observations, OKF lens, and reconciled governance (#3/#4/#5)*; verification now passes after no_std test fix.  
**Cutoff note:** Session interrupted while OpenRouter embedding pass was running. State captured below.

---

## Direction correction + new findings (2026-06-21)

Source: design session + queries against `/Users/srinji/ingestion_results/ast_graphs/unified_browsers.sqlite` (13 GB unified Chromium/WebKit/Gecko source graph; query protocol in `ingestion_results/QUERYME.md`).

### Direction corrections (supersede prior framing)
- **Standalone browser, not an agent substrate.** The render is the visible fork (ADR-001 rewritten). Prior "driver-agnostic / CDP-driver" AND "agent layer as product" framings are both rejected. Canonical state + policy are internal plumbing/defense, not the deliverable.
- **Build by bring-in + defense.** Bring in the converged engine core; build defense-in-depth (A12) beneath it. Don't rebuild what 3-way convergence proves is solved; don't drive a black-box Chrome.
- **LLM is not core runtime** (ADR-007). Only admissible model use = tiny (~10M) frozen advisory sensors at the boundary (charset/language/reader-mode), exempt sensor layer, barred from the deterministic spine by the calculator test. Precedent: Chrome's CLD3. Finite/known mapping → verifiable table, not a learned net.
- **Code-size discipline** (ADR-006): target the current internet, drop legacy-web compat, product shell, multi-platform breadth, and donor test farms; keep the engine core.
- **Restricted substrate grammar** (A18 proposed, ADR-005): Power-of-Ten `no_std` grammar on the written substrate; confine (not restrict) the brought-in muscle.

### DB findings (evidence-grounded)
- **Convergence pile confirmed:** 20 vendored libs present in ALL THREE engines (harfbuzz, icu, skia, boringssl, freetype, cairo, zlib, brotli, libwebp, libvpx, libaom, dav1d, libpng, libjpeg, woff2, angle, webrtc, libxml, sqlite, expat) — literal same-source convergence = the bring-in-verbatim pile. All spec-organs (parse/dom/style/layout/paint/compositor/net/storage/service-worker/media/a11y/wasm/gc/allocator) present in all three.
- **Data model is where the three DIVERGE** (no W3C spec governs in-memory representation): ownership = Blink Oilpan GC (Chromium-only) vs WebKit WTF refcount vs Gecko refcount+cycle-collector; values = Gecko/Stylo immutable Arc-shared (509 files) vs others refcounted; wire = Mojo (665) vs IPDL protocol state machines (526) vs hand-rolled. → ADR-005 mines: refcount/arena ownership + immutable content-addressed acyclic DAG (no GC; dissolves the cycle problem) + IPDL-style protocol surface.
- **Blindspots/caveats:** (1) Chromium graph under-represents Chromium — V8/Skia/WebRTC/ANGLE are DEPS/gclient, not in-tree (V8 `js_engine`=0). (2) Per-engine file counts are test-contaminated; real engine core ≈ 20–30k files (WebKit `Source/`) vs ~368k test files. (3) Graphics/raster is the least-converged "settled" layer (Skia for Chromium/WebKit vs cairo+freetype for Gecko) — a choice, not a freebie. (4) Crypto is two camps: BoringSSL (Chromium/WebKit) vs NSS (Gecko) — pick BoringSSL. (5) Symbol/call-graph layer is thin and inconsistent (Chromium 8.6k vs WebKit 85.9k symbols) — DB is good for structural/topology decisions, NOT yet function-level call alignment; a real call-graph extraction is needed before a per-function rebuild manifest.

### Next steps implied
1. Re-scope/park PR #11 (Chrome/CDP adapter) under the rewritten ADR-001 — it is the wrong direction as a remote driver.
2. Optionally regenerate the Chromium graph WITH DEPS pulled to close the third-party blind spot.
3. Build the test-stripped real-engine sizing so the bring-in manifest has true magnitudes.
4. Draft the bring-in manifest at subsystem granularity (verdict per organ: bring-in-from-{engine} / modernize / build / delete).
5. Ratify A18 + ADR-005/006/007.

---

## What just landed

### 1. Deterministic browser-engine ingestion
- `scripts/build_repo_graph.py` — git-tree + `git grep` + best-effort C/C++/GN graph builder.
- `scripts/repo_to_lgwks_db.py` — load any generated `graph.json` into canonical lgwks `research.sqlite`.
- `docs/BROWSER_ENGINE_INGESTION.md` — run instructions and SQLite quick-reference.
- `docs/BROWSER_ENGINE_UNDERSTANDINGS.json` — aggregate counts per engine.
- Chunking hardened against [Firecrawl best-practice chunking](https://www.firecrawl.dev/blog/best-chunking-strategies-rag.md) and [Azure vector-search chunking](https://docs.azure.cn/en-us/search/vector-search-how-to-chunk-documents): 512-token target chunks (~2048 chars), 128-token overlap (~512 chars, ~25%), recursive language-aware splitting with code-specific separators (class/def/function/struct/namespace), and source-path context prefix on every chunk. Tests in `tests/test_repo_to_lgwks_db.py`.
- **UQA property-graph tie-back (Chromium only):** exported the full Chromium source graph to UQA JSONL vertices/edges (`/Users/srinji/ingestion_results/chromium_uqa/`) and built a sidecar mapping from every UQA `file:` vertex to its chunked embeddings in `research.sqlite` (`chromium_uqa_to_embeddings.jsonl.gz`). This lets an agent query the symbolic graph (includes, GN imports, file containment) and the semantic vector space together.

Generated DBs (too large for git):

| Engine | Location | documents | chunks | nodes | edges |
|---|---|---:|---:|---:|---:|
| Chromium | `/Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite` | 519 053 | 46 212 | 572 717 | 1 661 885 |
| WebKit | `/Users/srinji/ingestion_results/webkit_lgwks_db/research.sqlite` | 456 079 | 27 059 | 564 715 | 602 314 |
| Gecko | `/Users/srinji/ingestion_results/gecko_lgwks_db/research.sqlite` | 387 841 | 35 326 | 429 722 | 466 799 |

### UQA / embedding joint query example (Chromium)

UQA symbolic graph + vector embeddings are now linked. Each embedded chunk maps to a UQA `file:` vertex by relative path:

```python
import gzip, json, sqlite3

# 1. Find UQA file vertices for files that include "layout/block.h"
uqa_id = "file:third_party/blink/renderer/core/layout/layout_block.h"

# 2. Look up the chunk IDs + embedding IDs for that file
with gzip.open("/Users/srinji/ingestion_results/chromium_uqa/chromium_uqa_to_embeddings.jsonl.gz", "rt") as f:
    for line in f:
        rec = json.loads(line)
        if rec["uqa_vertex_id"] == uqa_id:
            chunk_ids = [c["chunk_id"] for c in rec["chunks"]]
            embedding_ids = [c["embedding_id"] for c in rec["chunks"]]
            break

# 3. Fetch the actual vectors from research.sqlite
con = sqlite3.connect("/Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite")
rows = con.execute(
    "SELECT target_id, vector_json FROM embeddings WHERE id IN ({})".format(",".join("?" * len(embedding_ids))),
    embedding_ids,
).fetchall()
```

Coverage note: the UQA graph contains all 519K Chromium file vertices, but the embedding pass only covered the top 5K files by graph centrality (`CHUNK_SAMPLE_SIZE=5000` in `repo_to_lgwks_db.py`). `chromium_uqa_to_embeddings.jsonl.gz` links 4 876 UQA file vertices to all 46 212 embedded chunks. Expand the sample size or chunk all files to close the gap.

### 2. Hardened Rust seams
- `src/policy/broker.rs` — `PolicyBroker::decide` with deny-first, capability-attenuated, closed-vocabulary decision ladder. 7 tests.
- `src/state_machine/transition_table.rs` — deterministic state transitions; `Verdict::Deny` preserves state. 6 tests.
- `src/boundary/url_policy.rs` — deny-first URL/origin gate: http/https only, rejects userinfo/private-IP/loopback/non-ASCII hosts unless explicitly allowed. 8 tests.

### 3. CI cleanup
- Removed obsolete `ci/airworthiness-gate.mjs`; the structural gate is now the single source of truth. Closes #6.

## In-flight implementation (issues #3 / #4 / #5)

Mode A is now locked: we are **not** rebuilding the renderer. We are building the policy-gated, canonical-state agent layer that Chrome/WebKit do not provide. See `docs/MCP_RESEARCH.md` §6 for the full multimodal/native-understanding assessment and the Mode A decision log.

### #3 — Refactor mac-eye bridge to emit Braid observations
**Status:** Implemented.  
**Seam:** `native/mac-eye/Sources/mac-eye/main.swift` emits typed JSONL observations; `src/platform/webkit_adapter.rs` parses them into `WebAnchor` facts.  
**Final interface preserved:**
- Swift emits one JSONL line per observation: `{"kind":"element","path":"body>div>a:0","facts":[["tag","a"],["text","Sign in"],["interactable","true"]]}`.
- `WebKitAdapter::observe_from_jsonl(jsonl)` returns `Vec<WebAnchor>` with content-addressed CIDs.
- No direct `evaluateJavaScript` injection authority in Swift; JS execution is requested as a closed action.

**Deferred behind seam:**
- Process spawning for `lgwks-mac-eye` is intentionally outside the core library (gate `SEC.forbidden.api` forbids `std::process`). The driver binary / test harness is responsible for invocation.
- Real timestamp in `observed_at` and load/navigation completion signaling.

### #4 — Refactor okf.py to derive from Braid anchor
**Status:** Implemented.  
**Seam:** `python/okf.py` consumes a JSON array of `WebObservation` records and renders OKF text.  
**Final interface preserved:**
- OKF is a derived human-readable lens; canonical IDs are 64-hex content-addressed CIDs.
- Human references (`@eN`) are assigned by sorting interactable elements by CID, not by DOM walk order.
- Layout bounds are facts, not identity.

**Deferred behind seam:**
- Python/Rust IPC format is JSON over stdin for now. A protobuf/CBOR wire format will be introduced once the FFI seam is hardened.

### #5 — Reconcile docs/ADR.md and docs/SPEC.md
**Status:** Reconciled.  
**Seam:** governance documents now trace to `docs/AXIOMS.md` and `docs/DO178_PLAN.md`.  
**Final interface preserved:**
- `docs/ADR.md` — ADR-001 replaced with driver-agnostic engine; ADR-002/003/004 rewritten to reflect Braid canonical state, closed action vocabulary, and DAL assignments.
- `docs/SPEC.md` — HLRs rewritten as canonical state, OKF lens, closed actions, policy membrane, deterministic state machine, supported subset, and driver-agnostic bridge. Each maps to a DAL and a Rust seam.

**Deferred behind seam:**
- Full LLR/verifier expansion for HLR-06 and HLR-07 awaits concrete parser and mock-driver tests.

## Verification commands

```bash
cargo test --all-features
node ci/structural_airworthiness_gate.mjs --phase=all
cargo check --no-default-features --features no_std
cargo clippy --all-features -- -D warnings
```

Results: **32 tests OK, gate OK, no_std OK, clippy clean**.

## Known limitations / debts

- `CapabilityBroker::issue` and `CapabilityBroker::attenuate` now fail closed after validating principals/scope/attenuation. They still do not mint or resign credentials; wire these to a concrete signer only when the DID/signature scheme is chosen.
- `compute/lane_manager.rs` now performs capability/effect admission for JS and Wasm and returns the closed action verb; it does not execute guest code. Native sandbox execution remains owned by the brought-in runtime boundary.
- `platform/webkit_adapter.rs` still leaves native `load` and raw `execute_js` behind explicit seams. Use `execute_js_trace` for policy/audit observations until the native bridge is wired.
- The ingestion pipeline workaround for `lgwks repo graph` only parsing `.py`/`.rs` is logged at `srinji-kaggss/logicalworks-#234`. Do not remove the custom `git grep` parsers until that issue is closed.
- The research SQLite DBs are not in git; if regenerating them, use the same scripts and verify counts against `docs/BROWSER_ENGINE_UNDERSTANDINGS.json`.
- Embedding model decision for #12: `qwen/qwen3-embedding-8b` via OpenRouter is the viable choice. It is SOTA on MTEB(Code, v1) (80.68 nDCG@10 as of June 2025) and the OpenRouter runner sustains ~16–33 s per 64-chunk batch (well under the #12 <60 s / 32-chunk acceptance bar). The temporary `Octen/Octen-Embedding-8B` runner was abandoned because it could not complete a 32-chunk batch in 10 min locally. The 352 Octen embeddings already in the Chromium DB should be considered a stale experiment and can be left or deleted once the pass is complete.
- **OpenRouter pass active** (restarted concurrent ~2026-06-18 23:01 UTC). A temporary 24-hour OpenRouter key is in `/Users/srinji/.hermes/.env` and the temporary runner `/Users/srinji/ingestion_results/scripts/browser_embedding_runner_openrouter_temp.py` is calling `qwen/qwen3-embedding-8b` via OpenRouter's `/api/v1/embeddings` endpoint. Concurrency is capped at one batch per browser (max 3 concurrent OpenRouter calls) to respect rate limits. Rate observed after concurrent restart: ~192 total chunks per minute across all browsers (batch size 64 × 3 browsers). Key expires ~2026-06-19 07:58 UTC and must be removed from `.env` then. Temporary runner file should also be deleted once the pass completes or the key expires.
- **Current counts** (DB query at 2026-06-18 ~23:20 UTC):
  - Chromium: 8 960 `qwen/qwen3-embedding-8b` + 170 `Qwen/Qwen3-VL-Embedding-8B` + 352 `Octen/Octen-Embedding-8B` (9482 total; target ~38 916 chunks).
  - WebKit: 2 112 `qwen/qwen3-embedding-8b` (target ~27 059 chunks).
  - Gecko: 1 792 `qwen/qwen3-embedding-8b` (target ~35 326 chunks).
- **Process**: PID 57445 running `browser_embedding_runner_openrouter_temp.py`. Log: `/Users/srinji/ingestion_results/browser_embeddings.log`. Lock: `/Users/srinji/ingestion_results/browser_embeddings_openrouter.lock`.
- **Chunking note**: The in-flight OpenRouter pass is embedding chunks produced by the previous word-count splitter. The hardened chunker above will take effect only when the DBs are regenerated with `repo_to_lgwks_db.py`.

## Final seams for the next agent

1. **Chrome/CDP driver** — `platform/chrome_adapter.rs` is implemented in PR #11; merge it and then build the real CDP driver binary that emits the JSONL shape. This becomes the default driver; mac-eye is the mac-native alternate.
2. **Policy-to-observation wiring** — route `WebKitAdapter::execute_js` through `PolicyBroker` and emit an execution trace observation.
3. **Tape append** — implement `tape/fact_store.rs` so every observation and action appends a `TapeRecord`.
4. **Pixel anchor** — implement `observation/pixel_anchor.rs` to bind screenshot regions to element CIDs.
5. **Embedding model swap** — evaluate a text-only model for code chunks; keep Qwen3-VL for visual chunks only.
