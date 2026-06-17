# AX-Browser Topography

> Source of truth map. Every feature, file, and CI gate must point to a node here.
> This document is locked as basement. Changes require ADR amendment.

## Legend

- `L#` — layer
- `N#.#` — node
- `→` — data flow
- `↔` — bidirectional mapping
- `DAL-A/B/C/D` — Design Assurance Level per DO-178C

---

## Layer 0 — Host Trust Membrane

```text
L0.1 Process Launcher
     Spawn renderer / network / symbolic / compute processes.
     DAL: B

L0.2 Sandbox Broker
     OS-level confinement; deny-by-default; no direct host FS/NET authority.
     DAL: A

L0.3 Capability Registry
     Signed, scoped, verifiable tokens; attenuation-only grant lattice.
     DAL: A

L0.4 Host Network Inheritance
     macOS: CFNetwork/Safari trust slice (anti-bot wedge).
     Elsewhere: minimal auditable network channel.
     DAL: B

L0.5 Audit Tape Substrate
     Append-only, content-addressed, hash-chained fact store.
     DAL: A
```

## Layer 1 — Canonical Web Semantics

```text
L1.1 URL / Origin Algebra
     Canonicalization, origin computation, same-origin policy, cross-origin checks.
     DAL: A

L1.2 Fetch Policy
     Cookies, cache, CORS, CORB, redirects, partitioned storage, service workers.
     DAL: A

L1.3 HTML Tokenizer + Tree Builder
     WHATWG-equivalent over supported subset; produces DOM as human lens.
     DAL: B

L1.4 DOM Core
     Node tree, mutation algorithms, events, shadow DOM subset, ranges.
     DAL: B

L1.5 CSS Parse + Cascade
     Selectors, specificity, layers, custom properties, media/container queries.
     DAL: B

L1.6 Layout Engine
     Block/inline/flex/grid/tables, writing modes, text shaping, hit testing.
     DAL: C

L1.7 Paint + Raster
     Display list, damage tracking, tiles, GPU raster.
     DAL: C

L1.8 Compositor
     Layers, surfaces, synchronization, viewport lens.
     DAL: C
```

## Layer 2 — Compute Lanes

```text
L2.1 JS Embedding
     Rented engine (JSC/V8/SpiderMonkey) behind WebIDL/capability bindings.
     DAL: C

L2.2 Wasm Embedding
     Validated module format, linear memory, capability-import table.
     DAL: B

L2.3 Symbolic Engine
     Owned deterministic reasoning over canonical state.
     DAL: B

L2.4 Math Spine
     Calculator-reconstructable scoring and gating.
     DAL: A

L2.5 Embedding Sensor
     Frozen ML model producing vectors; advisory only.
     DAL: D
```

## Layer 3 — Event + Action System

```text
L3.1 Event Dispatch
     Capture/bubble, target, default action; all events are capability requests.
     DAL: B

L3.2 Closed Action Vocabulary
     web.navigate, web.observe, web.click, web.type, web.scroll, web.download,
     web.wait, web.execute_js, web.execute_wasm.
     DAL: A

L3.3 Idempotency Fences
     Intent UUIDs, exactly-once execution, dedup by UUID.
     DAL: A

L3.4 Policy Broker
     Deterministic admission/rejection/escalation before any mutation.
     DAL: A
```

## Layer 4 — Observability + Tape

```text
L4.1 Observation Pipeline
     Convert host state into typed, CID-addressed Facts.
     DAL: B

L4.2 Interaction Tape
     (state, action, decision, trace, outcome) records.
     DAL: A

L4.3 Provenance Graph
     Every derived fact knows source, transformation, reversibility.
     DAL: B

L4.4 Replay Harness
     Reconstruct state from tape deterministically.
     DAL: A

L4.5 Telemetry Recorder
     AXObserver-style human trajectory capture → PII-scrubbed tape.
     DAL: C
```

## Layer 5 — Derived Lenses

```text
L5.1 Accessibility Tree
     ARIA-mapped projection for assistive tech.
     DAL: C

L5.2 Agent Semantic Graph
     Affordances, entities, relationships for AI reasoning.
     DAL: C

L5.3 Reader Mode
     Content extraction projection.
     DAL: D

L5.4 Human Visual Shell
     Tabs, navigation, rendering — thin product lens.
     DAL: D

L5.5 Developer Lens
     Point-to-CID, provenance trace, action preview, replay.
     DAL: B

L5.6 AIP Protocol Bridge
     Ingests `/.well-known/agent-policy.json`, `/agent/state`, `/agent/actions`,
     and ASHP HTML attributes; projects them into Braid canonical terms.
     DAL: B
```

## Layer 6 — Verification

```text
L6.1 WPT Integration
     Compatibility oracle over declared subset.
     DAL: B

L6.2 Parser Equivalence
     WHATWG test corpus + fuzz.
     DAL: A

L6.3 Layout / Pixel Diffing
     Screenshot and geometry regression tests.
     DAL: C

L6.4 Security Fuzzers
     Origin, sandbox, capability, JS/Wasm escape, prompt-injection.
     DAL: A

L6.5 Airworthiness Gate
     Structural traceability + forbidden-API detection + MC/DC + tape reachability.
     DAL: A
```
