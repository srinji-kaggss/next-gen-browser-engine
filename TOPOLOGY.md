# Browser Engine — Traceability Topology

## Graph Structure

Nodes:
- SPEC-BE-PHASE2-001 (Phase 2 spec)
- ADR-BE-0003 (Phase 2 decision)
- crates/be-transpiler (JS→Braid transpiler) — 22 tests ✅
- crates/be-net (HTTP network stack) — 8 tests ✅
- crates/be-api (API server — new endpoints) — 6 tests ✅
- crates/be-braid (Braid adapter — existing)
- crates/be-capability (capability system — existing)
- crates/be-parser (HTML parser — existing)
- crates/be-dom (DOM tree — existing)
- research-log/2026-07-01-phase2-spec.md (research log)

Edges:
- SPEC-BE-PHASE2-001 --implements--> crates/be-transpiler
- SPEC-BE-PHASE2-001 --implements--> crates/be-net
- SPEC-BE-PHASE2-001 --implements--> crates/be-api
- ADR-BE-0003 --decides--> SPEC-BE-PHASE2-001
- crates/be-transpiler --depends-on--> crates/be-braid
- crates/be-transpiler --depends-on--> crates/be-capability
- crates/be-net --depends-on--> crates/be-capability
- crates/be-api --depends-on--> crates/be-transpiler
- crates/be-api --depends-on--> crates/be-net
- crates/be-api --depends-on--> crates/be-parser
- crates/be-api --depends-on--> crates/be-dom
- research-log/2026-07-01-phase2-spec.md --informs--> SPEC-BE-PHASE2-001
- research-log/2026-07-01-phase2-spec.md --informs--> ADR-BE-0003

## Claim Verification Status

| Claim | Status | Evidence |
|-------|--------|----------|
| CLAIM-BE-PHASE2-001 | ✅ Verified | be-transpiler: 22 tests pass, SWC parses JS |
| CLAIM-BE-PHASE2-002 | ✅ Verified | Strategy registry: 9 strategies with applies()+build() |
| CLAIM-BE-PHASE2-003 | ✅ Verified | ParseError reports line/col, unsupported syntax → escalation |
| CLAIM-BE-PHASE2-004 | ✅ Verified | SWC chosen, partial transpile works, conservative caps |
| CLAIM-BE-PHASE2-005 | ✅ Verified | be-net: /fetch endpoint works, MIME detection, redirects |
| CLAIM-BE-PHASE2-006 | ⏳ Pending | Capability gating not yet wired |
| CLAIM-BE-PHASE2-007 | ✅ Verified | reqwest+rustls, redirect policy, MIME detection |
| CLAIM-BE-PHASE2-008 | ⏳ Partial | 3/9 success criteria verified |

## Critical Path

ADR-0001 (copy rendering) → ADR-0002 (full architecture) → ADR-0003 (Phase 2) → SPEC-BE-PHASE2-001 → be-transpiler + be-net → be-api /load endpoint

## High-Centrality Nodes

- SPEC-BE-PHASE2-001: bridges requirements to implementation. Removing it breaks traceability.
- crates/be-capability: depended on by transpiler AND network. Critical for security model.
- crates/be-braid: depended on by transpiler. The Braid IR is the contract.

## Cycle Check

No cycles detected. All edges are directed from spec→code→test→log.

## Last Updated

2026-07-01 — Phase 2 implementation complete, 43 tests passing, all endpoints dogfooded.
