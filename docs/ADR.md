# Architecture Decision Records: AX-Browser

> Status: reconciled with `docs/AXIOMS.md` and `docs/DO178_PLAN.md`.
> Change requires ADR amendment + migration plan per Axiom A1.

## ADR-001: Standalone Browser, Built by Bring-In + Defense (replaces Driver-Agnostic Engine and Mac-Native Supremacy)

> Supersedes the prior ADR-001 ("Driver-Agnostic Engine" / CDP-driver) and the original
> "Mac-Native Supremacy". Both were rejected: see Rationale.

**Decision:** We build a **standalone, independently-running, efficient browser** — a real browser that renders pages, not an automation shim or an agent substrate. We do **not** drive an external Chrome over CDP, and we do **not** treat the canonical-state/policy layer as the product. We construct the browser by **bringing in the converged, irreducible engine pieces** (parse/style/layout/raster/JS/crypto/codecs — see ADR-006) and **building defense-in-depth** around and beneath them. The canonical state machinery and policy membrane are *internal plumbing and confinement*, not the visible deliverable.

**What is the visible fork:** **the render.** For any outside observer, a browser is differentiated by what it renders and how fast. Rendering fidelity and efficiency are therefore first-class product properties — not delegated to a black box, but owned. Safety/state/policy are how we keep that render trustworthy, not the thing we sell.

**Rationale:**
- *Why not CDP-drive Chrome:* a driven black box cannot have dangerous capability removed at the source (A12 confinement, not detection); you inherit its full attack surface and update treadmill; canonical state would be a downstream scrape, not owned.
- *Why not "agent substrate as product":* the browser is standalone and runs independently of any AI. Any LLM integration is a guest at the boundary, never the core runtime (ADR-007). Framing the engine as an agent layer mis-sites the product — the render is the product.
- *Why bring-in + defense:* the engine pieces are commodity (proven by 3-way convergence across Chromium/WebKit/Gecko — same vendored libs in all three). Rebuilding them is negative ROI. The differentiated, un-converged work is the data model (ADR-005), the supported-subset/size discipline (ADR-006), and the defense membrane.

**Consequences:**
- `platform/*_adapter.rs` are integration seams to brought-in engine modules, not remote drivers. PR #11 (Chrome/CDP adapter) is parked pending re-scope under this ADR.
- The render path is owned and budgeted (A13), not delegated.
- Defense-in-depth (A12) is built beneath the engine: untrusted page content is data, never instruction (A7); dangerous capability is removed from the vocabulary, not detected at runtime.

**Traceability:** AXIOM_A9 (supported subset, fail-closed), AXIOM_A12 (confinement), AXIOM_A13 (performance budget), AXIOM_A7 (page content is data).

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

## ADR-005: Canonical Data Model — Immutable, Content-Addressed, Acyclic Term Graph

**Decision:** The first and ceiling-setting decision is the in-memory data model. It is an **immutable, content-addressed (CID), acyclic term graph** owned by **refcount/arena** (no tracing GC), with a **typed protocol-state-machine** action/wire surface.

**Rationale (the data model is where the three engines do NOT converge):** The DOM *API* is spec-mandated and converged; the DOM *representation* is a free engineering choice and each browser chose differently — confirmed from the unified source graph:
- **Object ownership:** Blink = Oilpan **tracing GC** (Chromium-only, 70 signature files; non-deterministic pauses, unbounded → hostile to a bounded/fail-closed substrate). WebKit = **WTF refcount** (deterministic, but leaks cycles). Gecko = refcount + **cycle collector** (re-adds non-determinism).
- **Value representation:** Gecko/Stylo = **immutable, Arc-shared, content-identity** values (Rust, 509 signature files). Blink/WebKit = refcounted mutable style.
- **Cross-boundary wire:** Chromium = **Mojo** (665). Gecko = **IPDL protocol state machines** (526; illegal message sequences unrepresentable). WebKit = hand-rolled IPC.

We mine the safe pieces from each: WebKit's deterministic ownership, Stylo's immutable content-identity values, IPDL's protocol-state-machine wire. The decisive move only an intentionally-designed model makes: **choose an acyclic graph and the cycle problem that forced Gecko's cycle collector ceases to exist** — refcount/arena then suffices with zero GC. We dissolve the deepest non-determinism (the GC) at the only layer where it can be dissolved.

**Why it sets the ceiling:** the data model defines the vocabulary of representable states. Harm that is not a representable term is unreachable by construction (A12). The data model caps achievable defense; that is why it is the first decision.

**Consequences:**
- `src/browser_types.rs` / the Braid bridge target an immutable acyclic CID DAG; no GC dependency enters the core.
- The action surface is a typed protocol (closed vocabulary, A5) with state-machine-validated sequences (A4), modeled on IPDL's discipline.

**Traceability:** AXIOM_A2 (content-addressed canonical state), AXIOM_A4 (deterministic state machine), AXIOM_A5 (closed actions), AXIOM_A12 (confinement), AXIOM_A18 (restricted substrate grammar — proposed).

## ADR-006: Code-Size Discipline — Bring In the Minority, Delete the Legacy, Drop the Shell

**Decision:** Target a **standalone, efficient browser for the current internet**. Browsers are tens of millions of lines, but the *engine* is a minority of that. We bring in the converged engine core and deliberately exclude the rest.

**Why browsers are so large (measured from the source graph, not assumed):**
1. **Test suites dominate the line count** — web-platform-tests / LayoutTests / JSTests were the single largest directories in all three engines (e.g. WebKit `Source/` engine ≈ 26k files vs LayoutTests+JSTests ≈ 368k). Tests are the inherited correctness oracle, not runtime.
2. **30 years of backward-compat** for a web that no longer exists — quirks mode, legacy encodings, vendor prefixes, dead plugin/codec paths.
3. **The full accreted web-platform API surface** — hundreds of specs, most untouched by a modern site.
4. **Multi-platform backends** — Win/Mac/Linux/Android/iOS, each with its own graphics/net/IPC layer.
5. **Product shell that is not "browser"** — sync, autofill, translate, telemetry, extensions, devtools.

**What we keep vs drop:**
- **Keep (bring in verbatim, remap edges only):** the convergent engine core — HTML/CSS/URL parse, DOM, style cascade, layout, raster (Skia), shaping (HarfBuzz), one JS engine, crypto (BoringSSL), codecs, Unicode (ICU), accessibility (a11y tree).
- **Drop:** legacy-web compat outside the declared supported subset (A9), product shell, multi-platform breadth beyond our target, and the donor test farms (we inherit conformance via web-platform-tests, we don't ship them).

**Measurement caveats logged:** (a) the Chromium graph under-represents Chromium because V8/Skia/WebRTC/ANGLE are pulled via **DEPS/gclient**, not in-tree — any "bring in from Chromium" decision is partly blind in the current snapshot; (b) per-engine file counts are **test-contaminated** and must be test-stripped before sizing.

**Traceability:** AXIOM_A9 (supported subset, fail-closed), AXIOM_A13 (performance budget).

## ADR-007: LLM Is Not Core Runtime — Tiny Frozen Sensors at the Boundary Only

**Decision:** The browser runs **independently of any LLM**. No model is in the core runtime, the render path, the policy gate, or the deterministic spine. The *only* admissible model use is a **tiny (~10M-class), frozen, deterministic sensor at the boundary**, emitting **advisory** data the deterministic engine may consume but never obey (A6).

**Rationale:**
- Determinism and the calculator-test provenance bar (every runtime value reconstructable by a human with data + calculator + 0 internet) **bar learned weights from the deterministic core** — they are magic constants there.
- A model's only legitimate home is the **sensor layer** (like a frozen embedding sensor), which is exempt from the purity rule precisely because it is advisory and outside the math spine.
- **"Regurgitative AI" assessment:** a memorized/overfit network reproducing a fixed mapping is an *amortized lookup*. If the domain is finite and known, a real lookup table / perfect hash beats it — exact, tiny, and it passes the calculator test (no learned floats). A memorized net only earns its place when generalization over fuzzy/large input is required (bytes→charset, text→language) — i.e., as a **sensor**, not a function. So: finite/known mapping → deterministic table; fuzzy/large input needing generalization → tiny frozen sensor, advisory only.
- **Precedent (this is existing browser practice, not speculation):** Chrome already ships CLD3 (a tiny NN for language detection) plus heuristic charset detectors. Bring in that pattern, not a runtime agent.

**Admissible tiny-sensor candidates:** charset/encoding detection, language detection, reader-mode boilerplate extraction, resource-priority hints. All advisory, all boundary, none in the core runtime.

**Consequences:**
- Any model artifact is DAL-C/D, advisory, behind the policy membrane; it can never hold authority over an irreversible effect (A6).
- A finite known mapping must be implemented as a verifiable table, not a learned net.

**Traceability:** AXIOM_A6 (model output never authority), AXIOM_A1 (DAL), AXIOM_A15 (privacy tier precedes model exposure).
