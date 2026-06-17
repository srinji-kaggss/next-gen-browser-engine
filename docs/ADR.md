# Architecture Decision Records: AX-Browser

## ADR-001: Mac-Native Supremacy
Discard Chromium/Playwright for native `WKWebView`. This inherits the host's `CFNetwork` cryptographic footprint and Safari session state, rendering the engine indistinguishable from a human user.

## ADR-002: OKF (Optimized Knowledge Format)
Serialize page state into OKF—a semantic XML subset. Preserves bounding boxes and hierarchical relationships while stripping visual noise, achieving 90%+ token reduction.

## ADR-003: Ambient Ingestion (The Interaction Tape)
Record human web interactions (clicks/scrolls) paired with OKF state via macOS `AXObserver` to build an Imitation Learning dataset for autonomous agents.

## ADR-004: DO-178C Deterministic State Machine
Implement core logic as a formal state machine. Transitions between `NAVIGATING`, `CHALLENGE_DETECTED`, and `READY` must be deterministic and verifiable.
