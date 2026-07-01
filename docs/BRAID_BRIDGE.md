# Braid Bridge: Web Reality → Braid IR

> Braid IR is the eventual canonical IR. This document defines the bridge that
> converts raw web observations and actions into Braid terms.

## 1. The Bridge is Not Optional

OKF must not become a parallel IR. If it remains the canonical tree, we will
pay a full migration rewrite to Braid. OKF is permitted only as a human-readable
lens generated from the Braid anchor.

## 2. Braid Terms for the Browser

These term families are owned by Braid vocabulary packages. The closed action
verbs are consumed from `braid-vocab-web`; observation payloads project into
`braid_ir::Value` and do not recreate a local Braid enum.

### Observation terms
- `web.obs.node { tag, role, text, attrs, stable_id, parent_cid, source }`
- `web.obs.text { content, lang, parent_cid }`
- `web.obs.affordance { kind, target_cid, action_hint }`
- `web.obs.viewport { width, height, device_pixel_ratio }`
- `web.obs.network { url_cid, method, status, headers_cid, body_cid }`

### Action terms
- `web.act.navigate { url_cid, policy_cid }`
- `web.act.observe { scope_cid, budget }`
- `web.act.click { target_cid, intent_uuid }`
- `web.act.type { target_cid, value_cid, intent_uuid }`
- `web.act.scroll { target_cid, delta, intent_uuid }`
- `web.act.download { target_cid, intent_uuid }`
- `web.act.wait { condition_cid, timeout_ms }`
- `web.act.execute_js { script_cid, capability_set_cid, intent_uuid }`
- `web.act.execute_wasm { module_cid, capability_set_cid, intent_uuid }`

### Capability terms
- `web.cap.read { origin, scope }`
- `web.cap.write { origin, scope }`
- `web.cap.observe { scope }`
- `web.cap.compute { budget }`
- `web.cap.egress { budget, audited }`

### AIP protocol terms
- `web.obs.aip_state { surface, state, affordances, memory, risk, bindings }`
- `web.obs.aip_policy { version, site, observation_rules, action_rules, training_policy }`
- `web.act.aip_action { id, kind, risk, preconditions, postconditions, binding_cid }`
- `web.cap.aip_delegation { issuer_did, holder_did, audience, scope, privacy_tier, ttl }`

### Transition terms
- `web.tx { parent_cid, action_cid, policy_cid, observation_cid, outcome_cid }`

## 3. Canonical Serialization

All Braid terms use deterministic canonical CBOR/JSON subset. The CID is
BLAKE3 over the canonical bytes. The same fact always hashes to the same CID.

## 4. Logical vs Physical Separation

- **Logical observations** are Braid terms: tag, role, text, stable id,
  parent/child relationships. Their CID is stable across viewports.
- **Layout lenses** reference a logical observation CID and add bounds,
  viewport, visibility. They change when the window resizes.
- **Network lenses** reference a logical resource CID and add timing, cache
  state.

## 5. Adapter Components (implemented status)

`src/braid_bridge/` contains:

| File | Purpose | Status |
|---|---|---|
| `adapter.rs` | `WebAnchor` ↔ Braid IR | Implemented: `BraidAdapter::to_braid` maps `TermFamily::Observation` to canonical `braid_ir::Value`. Other families fail closed until they have canonical projections. |
| `observation.rs` | Convert DOM/AX to observation terms | Deferred; `src/observation/anchor.rs` owns the canonical observation fact seam. |
| `action.rs` | Convert action vocabulary to action terms | Deferred; `src/action/mod.rs` owns the closed `Action` seam. |
| `capability.rs` | Convert capability sets to capability terms | Deferred; `src/capability/mod.rs` owns capability facts. |
| `transition.rs` | Build tape transition terms | Deferred; `src/tape/fact_store.rs` is a stub. |
| `executor.rs` | Braid action term → WebKit/AX execution | Deferred. |
| `codec.rs` | Canonical serialization | Implemented for observations: `ObservationAnchor::canonical_bytes` uses Braid's canonical encoder over `braid_ir::Value`; protobuf remains a std-only wire seam in `proto/browser_state.proto`. |

Current canonical CID is `braid_ir::Cid`: BLAKE3 over domain-separated canonical
bytes. Hex is only the text-wire form.

## 6. Dependency Boundary

The browser depends on `braid-ir`, `braid-capability`, and `braid-vocab-web`
from the Braid repo. It does not recreate Braid's registry or maintain a second
closed action vocabulary.
