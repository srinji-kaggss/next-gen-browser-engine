# AIP Integration: Website Protocol Layer

> Status: locked basement. This document extracts findings from
> `/Users/srinji/Downloads/llm_native_browser_okf-2` and maps the Agent Interface
> Protocol (AIP) to the AX-Browser engine architecture.

## 1. Findings Summary

The OKF bundle defines a **website-side protocol** for LLM-native browsing:

- **AIP** — Agent Interface Protocol: how a site exposes symbolic state,
  actions, policy, risk, and memory to a user-delegated agent.
- **ASHP** — Agent Semantic HTML Profile: a cleaned HTML profile with stable
  `data-agent-*` attributes.
- **AST** — Agent State Tree: typed compact state exposed to models.
- **AAG** — Agent Action Graph: affordances with pre/postconditions, risk, and
  confirmation policy.
- **APG** — Agent Policy Graph: site policy, user policy, memory rules, and
  training/export posture.

The bundle adds new cross-cutting concerns:

- **DID-first identity** — Principal/session/agent DIDs with signed delegation
  envelopes.
- **Privacy-accuracy tiers** — `local_full`, `cloud_redacted`,
  `cloud_selective_reveal`, `cloud_full_context_explicit`.
- **Small-model operability** — Interface determinism should let a 2B model
  operate basic sites.
- **Trust classes** — `SYSTEM_POLICY`, `DEVELOPER_POLICY`, `USER_INTENT`,
  `TRUSTED_STATE`, `UNTRUSTED_CONTENT`.
- **Formal axioms** — Lean skeleton for authority, policy, origin containment,
  sensitivity non-disclosure, redaction, confirmation, deterministic execution,
  audit, minimality, and small-model operability.
- **Governance** — SQLite-style stewardship: open use, strict contribution gate,
  signed releases.

## 2. Mapping AIP to AX-Browser

| AIP concept | AX-Browser seam | Notes |
|---|---|---|
| `/.well-known/agent-policy.json` | `src/boundary/url_policy.rs` + `src/policy/broker.rs` | Site policy becomes a capability input, not the authority. |
| `/agent/state`, `/agent/actions` | `src/observation/anchor.rs` + `src/braid_bridge/` | AIP state/action graphs are observations, then canonical Braid terms. |
| Agent State Tree (AST) | `BraidTerm::Observation(WebObservation)` | Term family `web.observation.aip_state`. |
| Agent Action Graph (AAG) | `BraidTerm::Action(WebActionTerm)` + `WebAction.verb` | AAG actions map to the closed `web.*` vocabulary or are unrepresentable. |
| Agent Policy Graph (APG) | `BraidTerm::Verdict` + capability scope | Site policy is evidence; broker is authority. |
| DID delegation envelope | `WebCapability` issuer/subject + signature | DIDs become capability principals. |
| Privacy tier | `PrivacyTier` + `SensitivityClass` | Model exposure is a policy decision with redaction projection. |
| Risk level | `Risk` enum | `low`, `medium`, `high`, `human_only`, `denied`. |
| Trust class | Content provenance tag in `Provenance` | Untrusted content is data, never instruction. |
| Small-model operability | `src/state_machine/`, deterministic action set | `|ValidActions| <= K` (first K = 12). |
| LLM observability panels | `docs/OBSERVABILITY.md` + `src/audit/` | Agent state, action graph, model context, policy trace panels. |

## 3. Tensions and Resolutions

### Tension 1: HTML as bridge vs. Braid as canonical
AIP says HTML is the inspectable human+agent bridge. AX-Browser says DOM is a
human lens and Braid is canonical.

**Resolution**: HTML is the *authoring* bridge. The browser ingests HTML plus
sidecar JSON and immediately canonicalizes into Braid terms. The DOM remains a
derived lens. AIP does not compete with Braid; it is a wire format the bridge
consumes.

### Tension 2: Site policy as authority vs. capability broker as authority
AIP exposes `agent-policy.json` from the site. AX-Browser gives the policy
broker sole authority.

**Resolution**: Site policy is a *constraint* provided to the broker. The broker
also applies user policy, risk policy, privacy tier, and capability grants.
Site policy can deny; it cannot unilaterally permit dangerous actions.

### Tension 3: Open protocol vs. strict contribution gate
AIP wants broad adoption. AX-Browser is a hardline substrate.

**Resolution**: The *protocol* is open (schemas, conformance tests). The *reference
implementation* is stewarded like SQLite: open use, strict contribution gate,
signed releases.

## 4. New Axioms (added to AXIOMS.md)

- **A14 — AIP is a first-class wire lens.** The engine ingests AIP-compliant
  state and action graphs natively and degrades gracefully for legacy sites.
- **A15 — Privacy tier precedes model exposure.** No field reaches a cloud model
  without passing through the privacy tier, redaction, and user consent gates.
- **A16 — DID-backed delegation binds capabilities.** Every capability token has
  a DID principal chain; unbounded or unsigned delegation is unrepresentable.
- **A17 — Small-model operability is a design constraint.** Basic tasks must
  expose a finite, typed action set small enough for a local 2B model.

## 5. New Schemas

- `schemas/aip_policy.json`
- `schemas/aip_state.json`
- `schemas/aip_delegation.json`
- `schemas/aip_privacy.json`

## 6. New Rust Types

- `PrivacyTier`, `SensitivityClass`, `Risk` in `src/browser_types.rs`.
- `AipState`, `AipAffordance`, `AipPolicy` terms in `src/braid_bridge/term.rs`.
- `WebCapability` gains `privacy_tier` and `delegation` fields.
- `Provenance` gains `trust_class` and `did_principal` fields.

## 7. Implementation Order

1. Land this integration document and schema/types skeleton.
2. Refactor `mac-eye` to ingest `agent-policy.json` and emit AIP observations.
3. Refactor `okf.py` to derive from Braid canonical state (AIP + legacy).
4. Implement AIP devtools panels in `src/audit/`.
5. Add Lean formal axioms under `formal/`.

## 8. Relation to Existing Issues

- #3 (mac-eye refactor) — must consume AIP state/action endpoints.
- #4 (okf.py refactor) — must generate from Braid, not own canonical tree.
- #5 (ADR/SPEC reconciliation) — AIP and these axioms supersede old HLRs.
- #6 (remove old gate) — unchanged.
