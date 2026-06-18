# Handoff: Browser Engine — Foundation + Ingestion + Policy/State/URL Seams

**Date:** 2026-06-18  
**Repo:** `srinji-kaggss/next-gen-browser-engine`  
**Branch:** `foundation/aip-integration`  
**Merged PR:** [#8](https://github.com/srinji-kaggss/next-gen-browser-engine/pull/8) — *feat: browser-engine ingestion + deterministic policy/state/URL seams* (8ca1ab3 on main).  
**In-flight PR:** [#10](https://github.com/srinji-kaggss/next-gen-browser-engine/pull/10) — *feat: braid observations, OKF lens, and reconciled governance (#3/#4/#5)*; verification now passes after no_std test fix.  
**Cutoff note:** Session interrupted while OpenRouter embedding pass was running. State captured below.

## What just landed

### 1. Deterministic browser-engine ingestion
- `scripts/build_repo_graph.py` — git-tree + `git grep` + best-effort C/C++/GN graph builder.
- `scripts/repo_to_lgwks_db.py` — load any generated `graph.json` into canonical lgwks `research.sqlite`.
- `docs/BROWSER_ENGINE_INGESTION.md` — run instructions and SQLite quick-reference.
- `docs/BROWSER_ENGINE_UNDERSTANDINGS.json` — aggregate counts per engine.

Generated DBs (too large for git):

| Engine | Location | documents | chunks | nodes | edges |
|---|---|---:|---:|---:|---:|
| Chromium | `/Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite` | 519 053 | 46 212 | 572 717 | 1 661 885 |
| WebKit | `/Users/srinji/ingestion_results/webkit_lgwks_db/research.sqlite` | 456 079 | 27 059 | 564 715 | 602 314 |
| Gecko | `/Users/srinji/ingestion_results/gecko_lgwks_db/research.sqlite` | 387 841 | 35 326 | 429 722 | 466 799 |

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

- `CapabilityBroker::issue` and `CapabilityBroker::attenuate` are still `todo!()` in `src/capability/mod.rs`. Policy broker tests build capabilities directly. Implement these only when a concrete signature scheme is chosen.
- `compute/lane_manager.rs`, `audit/lens.rs`, `tape/fact_store.rs`, and `observation/pixel_anchor.rs` remain `todo!()` stubs. They should be filled once observations flow end-to-end through a real driver.
- The ingestion pipeline workaround for `lgwks repo graph` only parsing `.py`/`.rs` is logged at `srinji-kaggss/logicalworks-#234`. Do not remove the custom `git grep` parsers until that issue is closed.
- The research SQLite DBs are not in git; if regenerating them, use the same scripts and verify counts against `docs/BROWSER_ENGINE_UNDERSTANDINGS.json`.
- Background embedding pass was stopped after the temporary `Octen/Octen-Embedding-8B` runner proved too slow. It produced 352 Chromium embeddings, then a single batch of 32 real code chunks saturated the 10-minute benchmark window. The bottleneck is the 8B parameter model, not MPS compilation. Decision tracked in #12.
- **OpenRouter pass in progress** (started ~2026-06-18 07:58 UTC). A temporary 24-hour OpenRouter key was added to `/Users/srinji/.hermes/.env` and the temporary runner `/Users/srinji/ingestion_results/scripts/browser_embedding_runner_openrouter_temp.py` is calling `qwen/qwen3-embedding-8b` via OpenRouter's `/api/v1/embeddings` endpoint. Rate observed: ~64 Chromium chunks per minute (batch size 64). The runner processes Chromium, then WebKit, then Gecko sequentially. Key expires ~2026-06-19 07:58 UTC and must be removed from `.env` then. Temporary runner file should also be deleted once the pass completes or the key expires.
- **Current counts at cutoff**:
  - Chromium: 576 `qwen/qwen3-embedding-8b` + 142 `Qwen/Qwen3-VL-Embedding-8B` + 352 `Octen/Octen-Embedding-8B`.
  - WebKit: 0 embeddings.
  - Gecko: 0 embeddings.
- **Process**: PID 44183 running `browser_embedding_runner_openrouter_temp.py`. Log: `/Users/srinji/ingestion_results/browser_embeddings.log`. Lock: `/Users/srinji/ingestion_results/browser_embeddings_openrouter.lock`.

## Final seams for the next agent

1. **Chrome/CDP driver** — `platform/chrome_adapter.rs` is implemented in PR #11; merge it and then build the real CDP driver binary that emits the JSONL shape. This becomes the default driver; mac-eye is the mac-native alternate.
2. **Policy-to-observation wiring** — route `WebKitAdapter::execute_js` through `PolicyBroker` and emit an execution trace observation.
3. **Tape append** — implement `tape/fact_store.rs` so every observation and action appends a `TapeRecord`.
4. **Pixel anchor** — implement `observation/pixel_anchor.rs` to bind screenshot regions to element CIDs.
5. **Embedding model swap** — evaluate a text-only model for code chunks; keep Qwen3-VL for visual chunks only.
