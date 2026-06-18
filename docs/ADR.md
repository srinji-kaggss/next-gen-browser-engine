# Architecture Decision Records: AX-Browser

> Status: reconciled with `docs/AXIOMS.md` and `docs/DO178_PLAN.md`.
> Change requires ADR amendment + migration plan per Axiom A1.

## ADR-001: Driver-Agnostic Engine (replaces Mac-Native Supremacy)

**Decision:** The engine is a policy-gated, canonical-state browser substrate. The default driver is a headless Chromium/CDP bridge; `mac-eye` (WKWebView) is a mac-native, higher-trust alternate driver.

**Rationale:**
- macOS requires a native WebKit path for App Store / host-indistinguishable traffic, but the engine's value is not the renderer.
- The competitive surface is deterministic policy, closed action vocabulary, and canonical Braid state—capabilities Chrome does not provide.
- A driver-agnostic core lets the same policy/state engine run on macOS, Linux, and CI mock fixtures.

**Consequences:**
- `platform/webkit_adapter.rs` is one `BrowserDriver` implementation.
- The canonical anchor lives in Rust, not in Swift or Chrome.
- Rendering fidelity is delegated to the driver; safety is owned by the engine.

**Traceability:** AXIOM_A2 (canonical Braid state), AXIOM_A5 (closed actions), AXIOM_A11 (human UI is a lens).

## ADR-002: OKF as Derived Lens over Braid Canonical State

**Decision:** Optimized Knowledge Format (OKF) is a human-readable, token-efficient text manifest rendered from the Braid canonical anchor. It is not the source of truth.

**Rationale:**
- Stable, content-addressed CIDs survive re-renders and layout shifts.
- Human references (`@eN`) are assigned by sorting interactable elements by CID, not by DOM walk order.
- Page content remains data; it is never interpreted as instruction.

**Consequences:**
- `python/okf.py` consumes `WebAnchor`/`WebObservation` records from Rust.
- Layout bounds are facts, not identity.
- The same DOM tree rendered at different sizes produces the same OKF reference map.

**Traceability:** AXIOM_A2, AXIOM_A11, AXIOM_A14 (AIP wire lens), AXIOM_A15 (privacy tier).

## ADR-003: Interaction Tape via Closed Action Vocabulary

**Decision:** The interaction tape records only actions from the closed `web.*` vocabulary (`web.navigate`, `web.observe`, `web.click`, `web.type`, `web.scroll`, `web.download`, `web.wait`, `web.execute_js`, `web.execute_wasm`). Ambient human recordings are out of scope for the trusted tape.

**Rationale:**
- Ad-hoc scripting and free-form network calls are unrepresentable in production code (Axiom A5).
- Every tape entry carries `(parent_cid, state_snapshot_cid, action_cid, policy_verdict_cid, execution_trace_cid, outcome_cid)`.
- The tape is append-only and content-addressed (Axiom A8).

**Consequences:**
- mac-eye no longer injects arbitrary JS as a first-class operation. JS execution is requested as `web.execute_js` and admitted by the policy broker.
- The `AXObserver` ambient ingestion concept is deprecated; trusted trajectories are built through the engine's own action API.

**Traceability:** AXIOM_A5, AXIOM_A6 (model output never authority), AXIOM_A7 (provenance), AXIOM_A8 (append-only tape).

## ADR-004: DO-178C Deterministic State Machine with DAL Assignments

**Decision:** Core components carry a declared Design Assurance Level (DAL). Policy broker, capability registry, fact store, URL/origin policy, and state machine transition table are DAL-A. Rendering/layout/LLM sensors are DAL-C/D.

**Rationale:**
- DAL-A failure could cause unauthorized host access or data exfiltration.
- DAL-D components (LLM sensors) are advisory and may never hold authority over irreversible effects.
- The structural airworthiness gate enforces forbidden-API, coverage, and traceability requirements.

**Consequences:**
- Every code change outside `docs/**` triggers full CI.
- `ci/structural_airworthiness_gate.mjs` is the single source of truth for gate status.
- Tests for DAL-A components must exercise every transition and every policy decision outcome.

**Traceability:** AXIOM_A1 (design assurance), AXIOM_A8 (tape), AXIOM_A9 (supported subset, fail-closed), AXIOM_A13 (performance budget).
