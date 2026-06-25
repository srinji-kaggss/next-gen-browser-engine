# AX-Browser: High-Level Requirements (HLR)

> Status: reconciled with `docs/AXIOMS.md` and `docs/DO178_PLAN.md`.
> Every HLR maps to a DAL assignment and a Rust seam / verifier.

## 1. System Overview

AX-Browser is a deterministic, policy-gated web-cognition substrate. It does not compete with Chromium/WebKit/Gecko on rendering; it owns the layer those engines lack: canonical state, capability attenuation, closed action vocabulary, and privacy-aware model exposure.

The engine supports multiple browser drivers:
- **Default:** headless Chromium via Chrome DevTools Protocol (CDP).
- **macOS alternate:** `mac-eye` native WKWebView bridge.
- **Test fixture:** mock driver with deterministic observations.

Product-axis note: ADR-001 describes the standalone browser direction built by
bring-in + defense. This HLR keeps the substrate/bridge contract alive: the same
canonical state and policy membrane must work under copied/brought-in engine
muscle, CDP bootstrap drivers, WKWebView, or deterministic fixtures.

## 2. Requirements

### HLR-01: Canonical Braid State Engine

The engine MUST maintain a typed, content-addressed (CID) graph of facts—observations, actions, capabilities, policy verdicts, and transitions—as the single source of truth for a web session.

- **DAL:** A (fact store / tape / policy broker)
- **Tracing Marker:** `Tracing: HLR-01`
- **Rust Seams:** `src/observation/*`, `src/tape/*`, `src/browser_types.rs`
- **Verifier:** `test/cid_stability_*`, structural gate `SEC.capability.reach`
- **Axiom Mapping:** A2, A7, A8

### HLR-02: OKF as Derived Lens

The engine MUST render a human-readable, token-efficient text manifest (OKF) from the canonical Braid anchor. OKF references MUST be stable across re-renders and MUST be derived from content-addressed CIDs, not from layout-dependent tree walk order.

- **DAL:** B (observation anchor / lens)
- **Tracing Marker:** `Tracing: HLR-02`
- **Rust/Python Seams:** `src/audit/lens.rs`, `src/braid_bridge/adapter.rs`, `python/okf.py`
- **Verifier:** `test/lens_*`, `test/okf_reference_stability_*`
- **Axiom Mapping:** A2, A11, A14, A15

### HLR-03: Closed Action Vocabulary

The engine MUST admit only actions from the declared `web.*` vocabulary. Ad-hoc scripting, raw `evaluateJavaScript`, and free-form network calls MUST be unrepresentable in production code.

- **DAL:** A (policy broker / action enum)
- **Tracing Marker:** `Tracing: HLR-03`
- **Rust Seams:** `src/action/*`, `src/policy/broker.rs`, `src/state_machine/*`
- **Verifier:** `test/action_*`, `test/policy_*`, structural gate `SEC.forbidden.api`
- **Axiom Mapping:** A5, A6, A12

### HLR-04: Policy Membrane

Every origin access, compute event, network request, and model-exposed observation MUST pass through a deny-first policy broker. Authenticator, payment, and secret-class data MUST never reach a cloud model.

- **DAL:** A (policy broker / URL policy / capability registry)
- **Tracing Marker:** `Tracing: HLR-04`
- **Rust Seams:** `src/policy/*`, `src/boundary/url_policy.rs`, `src/capability/*`
- **Verifier:** `test/url_policy_*`, `test/capability_*`, `test/privacy_tier_*`
- **Axiom Mapping:** A3, A4, A15, A16

### HLR-05: Deterministic State Machine

The engine MUST transition between deterministic states (`Idle`, `Navigating`, `Observing`, `Quiescent`, `Executing`, `Terminated`). Denied actions MUST leave state unchanged.

- **DAL:** A (state machine transition table)
- **Tracing Marker:** `Tracing: HLR-05`
- **Rust Seams:** `src/state_machine/transition_table.rs`
- **Verifier:** `test/transition_*`, MC/DC harness `COV.transition.mcdc`
- **Axiom Mapping:** A1, A8, A9, A13

### HLR-06: Supported Subset with Fail-Closed Behavior

The engine MUST declare a supported web subset. Inputs outside that subset MUST be rejected or degraded deterministically, never silently reinterpreted.

- **DAL:** A (URL/origin policy) / C (parser)
- **Tracing Marker:** `Tracing: HLR-06`
- **Rust Seams:** `src/boundary/url_policy.rs`, `src/parser/*`
- **Verifier:** `test/url_policy_*`, `test/parser_subset_*`
- **Axiom Mapping:** A9, A12

### HLR-07: Driver-Agnostic Browser Bridge

The engine MUST consume observations from any compliant browser driver. The canonical anchor MUST be independent of whether the driver is Chromium/CDP, WKWebView, or a test fixture.

- **DAL:** B (platform adapter)
- **Tracing Marker:** `Tracing: HLR-07`
- **Rust Seams:** `src/platform/*`
- **Verifier:** `test/platform_*`, mock-driver differential tests
- **Axiom Mapping:** A2, A10, A14

### HLR-08: Developer Compatibility Contract

The engine MUST classify developer workloads into native engine, guest compute,
quarantined legacy, or reject modes before execution. Java/JVM compatibility
MUST be represented only as quarantined guest compute; JVM classloader,
filesystem, network, reflection, native library, plugin, or extension authority
MUST NOT enter the core substrate unless represented by closed capabilities.

- **DAL:** A (compatibility classifier / capability mapping)
- **Tracing Marker:** `Tracing: HLR-08`
- **Rust Seams:** `src/compat/*`, `src/capability/*`, `src/compute/*`
- **Verifier:** `test/compat_*`, structural gate `SEC.forbidden.api`
- **Axiom Mapping:** A4, A5, A9, A12, A13
