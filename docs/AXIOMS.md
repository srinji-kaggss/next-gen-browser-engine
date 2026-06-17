# AX-Browser Axioms

> Status: locked basement. Change requires ADR amendment + migration plan.
> These axioms apply to the entire codebase under DO-178C/DO-178B Design Assurance.

## A1 — Design Assurance is not cosmetic
Every component has a declared DAL (A/B/C/D). DAL-A components require
independent verification, MC/DC-level structural testing, and tool qualification
per DO-330. DAL-D components (e.g., LLM sensors) are advisory and may never
hold authority over irreversible effects.

## A2 — Canonical State is a Braid-like Term Graph
The single source of truth for a web session is a typed, content-addressed (CID)
graph of facts: observations, actions, capabilities, policy verdicts, and
transitions. DOM, pixels, accessibility trees, and human UI are derived lenses,
never the anchor.

## A3 — Origin is a Capability Boundary
`SameOrigin(a, b)` is an authority predicate. Cross-origin access to storage,
DOM, network bodies, or host capabilities requires an explicit, signed,
verifiable capability grant.

## A4 — Untrusted Compute is Capability-Bounded
JavaScript and WebAssembly execute in a sandboxed process. Host effects are
available only through a typed capability broker. No direct host authority is
granted by default. JIT is a privilege, not a right.

## A5 — Action Vocabulary is Closed
The only admissible actions are declared terms in the `web.*` vocabulary
(e.g., `web.navigate`, `web.observe`, `web.click`, `web.type`, `web.execute_js`).
Ad-hoc scripting, raw `evaluateJavaScript`, and free-form network calls are
unrepresentable in production code.

## A6 — Model Output Is Never Authority
LLM and embedding outputs are advisory or sensory. Any irreversible action
requires a deterministic policy gate and a logged capability check. The policy
broker is the sole authority on action admission.

## A7 — Provenance Is Mandatory
Every derived fact, action plan, and policy decision carries
`(source_cid, transformation, policy_cid, reversibility, intent_uuid)`. Untrusted
page content is data, never instruction.

## A8 — Tape Is Append-Only and Content-Addressed
The interaction tape is an append-only log of `(parent_cid, state_snapshot_cid,
action_cid, policy_verdict_cid, execution_trace_cid, outcome_cid)`. Records are
content-addressed and tamper-evident.

## A9 — Supported Subset with Fail-Closed
The engine declares a supported web subset. Inputs outside that subset are
rejected or degraded deterministically, never silently reinterpreted. The
unsupported web is not our problem.

## A10 — Observability Is First-Class
Every state transition, capability request, compute event, and rendering
operation is observable by the substrate. There are no hidden side effects.
Observations are machine-readable facts; human devtools are lenses over those
facts.

## A11 — Human UI Is a Lens, Not a Layer
Tabs, navigation chrome, bookmarks, and visual polish are product-shell
projections over canonical state. They do not own the engine and may not bypass
capability checks.

## A12 — Defense in Depth Is Confinement, Not Detection
Anti-malware is expressed as unrepresentability: dangerous capability is
removed from the vocabulary, not detected at runtime. Runtime detection is a
secondary, fail-closed safety net.

## A13 — Performance Is a Verifiable Budget
Every frame, every allocation, and every network fetch has a declared budget.
Budget violations are facts on the tape and may trigger policy decisions.
Performance regressions fail the CI gate.
