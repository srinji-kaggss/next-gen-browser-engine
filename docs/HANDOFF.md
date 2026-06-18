# Handoff: Browser Engine — Foundation + Ingestion + Policy/State/URL Seams

**Date:** 2026-06-18  
**Repo:** `srinji-kaggss/next-gen-browser-engine`  
**Branch status:** `foundation/aip-integration` is one commit ahead of `origin/main` (cabffdd).  
**Merged PR:** [#8](https://github.com/srinji-kaggss/next-gen-browser-engine/pull/8) — *feat: browser-engine ingestion + deterministic policy/state/URL seams* (8ca1ab3 on main).  
**Next open issues:** #3, #4, #5.

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

## Verification commands

```bash
cargo test
node ci/structural_airworthiness_gate.mjs --phase=all
```

Both pass: **21 tests OK, gate OK**.

## Final seams for the next agent

The next work items are tracked and should be attacked in this order:

### #6 — CLOSED
Obsolete gate removed.

### #3 — Refactor mac-eye bridge to emit Braid observations
**Seam:** `native/mac-eye/Sources/mac-eye/main.swift` ↔ `src/observation/anchor.rs` / `src/braid_bridge/adapter.rs`.
**Final interface the next agent should preserve:**
- Swift bridge emits typed `WebObservation` facts (not raw JSON).
- All JS execution is requested as an `Action` that receives a `Verdict` from the Rust policy broker before execution.
- No direct `evaluateJavaScript` injection from Swift; the native side is a controlled I/O device, not a policy authority.
**Deferred behind seam:** actual `WKWebView` navigation and script-evaluation implementation stays in `platform/webkit_adapter.rs`.

### #4 — Refactor okf.py to derive from Braid anchor
**Seam:** `python/okf.py` consumes `WebAnchor`/`BraidTerm` serialized from Rust, not raw DOM JSON.
**Final interface the next agent should preserve:**
- OKF is a human-readable derived lens; canonical IDs are content-addressed CIDs.
- Remove layout-dependent `@eN` reference IDs from the canonical path; keep them only in the OKF rendering if needed for human interaction.
**Deferred behind seam:** the actual Swift/Rust serialization format (JSONL over stdout or IPC) is a stub until #3 lands.

### #5 — Reconcile docs/ADR.md and docs/SPEC.md
**Seam:** governance documents must trace to `docs/AXIOMS.md` and `docs/DO178_PLAN.md`.
**Final interface the next agent should preserve:**
- Every ADR maps to one or more `AXIOM_*` constants.
- Every HLR maps to a DAL assignment and a Rust seam / verifier.
- No duplicate source-of-truth: archive or rewrite `ADR.md`/`SPEC.md` rather than leave them competing with `AXIOMS.md`/`DO178_PLAN.md`.

## Known limitations / debts

- `CapabilityBroker::issue` and `CapabilityBroker::attenuate` are still `todo!()` in `src/capability/mod.rs`. Policy broker tests build capabilities directly. The next agent should implement these only when a concrete signature scheme is chosen.
- `platform/webkit_adapter.rs`, `compute/lane_manager.rs`, `audit/lens.rs`, `braid_bridge/adapter.rs`, `tape/fact_store.rs`, and `observation/pixel_anchor.rs` remain `todo!()` stubs. They should be filled in order: #3 (platform/observation/braid bridge), then #4 (okf lens), then audit/lane/tape can follow once observations flow.
- The ingestion pipeline workaround for `lgwks repo graph` only parsing `.py`/`.rs` is logged at `srinji-kaggss/logicalworks-#234`. Do not remove the custom `git grep` parsers until that issue is closed.
- The research SQLite DBs are not in git; if the next agent regenerates them, use the same scripts and verify counts against `docs/BROWSER_ENGINE_UNDERSTANDINGS.json`.
