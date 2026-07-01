# Locked Basement Files

These files define the foundational seam of the web-cognition substrate. They are locked: any change must be accompanied by an updated traceability entry in `docs/DO178_PLAN.md` and a passing `ci/structural_airworthiness_gate.mjs --phase traceability`.

## Axioms and Architecture

- `docs/AXIOMS.md` — 13 hard axioms.
- `docs/TOPOGRAPHY.md` — layer-by-layer source-of-truth map (L0–L6).
- `docs/DO178_PLAN.md` — DO-178B/C application and CI gate architecture.
- `docs/DEFENSE_IN_DEPTH.md` — six-layer defense-in-depth stack.
- `docs/PERFORMANCE.md` — performance targets and regression gates.
- `docs/OBSERVABILITY.md` — fact-based observability for machine realm.
- `docs/MACHINE_HUMAN_MEETING.md` — PixelAnchor + audit-first devtools.
- `docs/JS_WASM_POSITION.md` — unified compute lane boundary.
- `docs/HUMAN_LENS_DEFERRAL.md` — human UI as Phase-2+ lens.
- `docs/BRAID_BRIDGE.md` — browser-to-Braid IR requirements.
- `docs/ANTIVIRUS_BROWSER.md` — browser-as-antivirus doctrine.
- `docs/AIP_INTEGRATION.md` — AIP findings and AX-Browser mapping.

## Schemas

- `schemas/web_anchor.json`
- `schemas/aip_policy.json`
- `schemas/aip_state.json`
- `schemas/aip_delegation.json`
- `schemas/aip_privacy.json`
- `schemas/web_action.json`
- `schemas/web_observation.json`
- `schemas/web_capability.json`
- `schemas/web_tape.json`

## Rust Source Seams

- `src/action/mod.rs`
- `src/audit/lens.rs`
- `src/audit/mod.rs`
- `src/boundary/mod.rs`
- `src/boundary/url_policy.rs`
- `src/braid_bridge/adapter.rs`
- `src/braid_bridge/mod.rs`
- `src/browser_axioms.rs`
- `src/browser_types.rs`
- `src/capability/mod.rs`
- `src/compat/mod.rs`
- `src/compute/lane_manager.rs`
- `src/compute/mod.rs`
- `src/lib.rs`
- `src/observation/anchor.rs`
- `src/observation/mod.rs`
- `src/observation/pixel_anchor.rs`
- `src/platform/mod.rs`
- `src/platform/webkit_adapter.rs`
- `src/policy/broker.rs`
- `src/policy/mod.rs`
- `src/state_machine/mod.rs`
- `src/state_machine/transition_table.rs`
- `src/tape/fact_store.rs`
- `src/tape/mod.rs`

## Build / CI

- `Cargo.toml`
- `ci/registry.json`
- `ci/run.mjs`
- `ci/structural_airworthiness_gate.mjs`
- `ci/generated/web_action_verbs.json`
- `tools/export_web_vocab.rs`

## Diagrams

- `diagrams/topography.mmd`
- `diagrams/ci_gates.mmd`
- `diagrams/defense_in_depth.mmd`
- `diagrams/observability.mmd`
- `diagrams/machine_human_meeting.mmd`
