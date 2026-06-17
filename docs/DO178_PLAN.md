# DO-178C / DO-178B Airworthiness Plan

> Applies to the entire codebase, not just the ML policy layer.
> The CI gate is the structural enforcer, not a Markdown checklist.

## 1. Scope and Objective

DO-178C/DO-178B provides a framework for software assurance in airborne
systems. We adopt its structure — DAL, HLR/LLR traceability, structural coverage,
independence, and tool qualification — to govern a web-cognition substrate
where:
- DAL-A failure could cause unauthorized host access or data exfiltration.
- DAL-B failure could corrupt the causal tape or policy history.
- DAL-C failure could degrade rendering or observability.
- DAL-D failure could produce bad advice from an LLM sensor, but never authority.

## 2. Design Assurance Level Assignment

| Component | DAL | Justification |
|---|---|---|
| Policy broker / verifier | A | Authority over every irreversible action. Failure → unauthorized effect. |
| Capability registry / broker | A | Security boundary. Failure → privilege escalation. |
| Fact store / tape | A | Audit and replay substrate. Failure → undetectable tampering. |
| URL / origin policy | A | First boundary against origin escape. |
| State machine transition table | A | Determinism of core lifecycle. |
| HTML parser / DOM algorithms | B | Canonical human lens; must be spec-equivalent. |
| JS/Wasm capability boundary | B | Confinement of untrusted compute. |
| Observation anchor / pixel mapping | B | Bridge between human and AI views. |
| Network policy / cache | B | Privacy and anti-tracking. |
| Layout / paint / compositor | C | Quality and performance, not safety-critical. |
| Accessibility / reader mode | C | Human lens correctness. |
| LLM / embedding sensor | D | Advisory; isolated from authority. |
| Human UI shell | D | Product convenience, no authority. |

## 3. HLR → LLR → Code → Test Traceability

```text
HLR-01: Untrusted web execution is confined.
  LLR-01.1: All JS/Wasm host calls route through the capability broker.
  LLR-01.2: Capability tokens are signed and scoped.
  Code: src/capability/*, src/policy/*, src/compute/*
  Test: test/capability_*, test/compute_escape_*, property tests for attenuation

HLR-02: Every action is admitted by the policy broker.
  LLR-02.1: Action enum is closed; no ad-hoc execution paths.
  LLR-02.2: Broker returns Verdict { allow/deny/escalate, rationale_cid }.
  Code: src/action/*, src/policy/broker.rs
  Test: test/policy_*, test/action_*, MC/DC on transition table

HLR-03: The tape is append-only and content-addressed.
  LLR-03.1: append(parent_cid, transition) -> cid is monotonic.
  LLR-03.2: prove(cid) returns the causal chain.
  Code: src/tape/*
  Test: test/tape_*, test/replay_*, fuzz for fork detection

HLR-04: The canonical anchor is separate from derived lenses.
  LLR-04.1: Observations are typed Braid terms.
  LLR-04.2: Lenses implement Lens trait and cannot mutate canonical state.
  Code: src/observation/*, src/audit/lens.rs
  Test: test/lens_*, test/cid_stability_*

HLR-05: Supported subset is enforced with fail-closed behavior.
  LLR-05.1: URL policy rejects unsupported schemes and origins.
  LLR-05.2: Parser declares supported subset and rejects the rest.
  Code: src/boundary/url_policy.rs, src/parser/*
  Test: test/url_policy_*, test/parser_subset_*

HLR-06: Performance is a verifiable budget.
  LLR-06.1: Frame budget, allocation budget, and network budget are declared.
  LLR-06.2: Violations are facts on the tape.
  Code: src/budget/*
  Test: test/budget_*, benchmark regression gate

HLR-07: Defense in depth prevents malware and prompt injection.
  LLR-07.1: Dangerous capability is unrepresentable in the vocabulary.
  LLR-07.2: Page content is data; it cannot be interpreted as instruction.
  LLR-07.3: Static analysis rejects forbidden API usage.
  Code: src/vocabulary/*, src/anti_inject/*, ci/structural_airworthiness_gate.mjs
  Test: test/prompt_inject_*, test/malware_unrepresentable_*
```

## 4. Structural Coverage Requirements

- **DAL-A components**: MC/DC (Modified Condition/Decision Coverage) on the state
  machine and policy broker. Property-based tests must exercise every transition
  and every decision outcome.
- **DAL-B components**: Decision coverage + statement coverage.
- **DAL-C components**: Statement coverage.
- **DAL-D components**: No coverage requirement; monitoring and adversarial
  testing only.

## 5. Independence and Anti-Trusting-Trust

- The verifier (`ci/structural_airworthiness_gate.mjs`) is maintained separately
  from the executor (`src/action/*`).
- The verifier does not reuse serialization code from the engine.
- Tool qualification (DO-330): internal audit scripts must self-test with planted
  failures and pass before gating others.

## 6. CI Gate Architecture

Modeled after `logic-os-kernel/.github/workflows/ci.yml` and
`scripts/ci/run.mjs`.

### Gates (foundation-first)

1. **foundation** — governance, repo-map, doc schema, closed vocabulary,
   anti-duplication, registry-ontology.
2. **security** — forbidden-API scan, capability reachability, origin policy,
   prompt-injection static checks.
3. **build** — fmt, clippy(-D warnings), cargo check --locked, per-crate build.
4. **portability** — no_std / dependency firewall for core crates.
5. **test** — cargo test, property tests, differential tests, MC/DC harness.
6. **audit** — cargo audit, recorded-rationale ignore list, supply chain.
7. **contract** — schema parse, WPT subset, layout/pixel diffing.
8. **performance** — budget regression benchmarks, allocation-per-frame gate.

### Verifier registry

`ci/registry.json` mirrors the kernel's `scripts/ci/registry.json`:
- `id`, `division`, `gate`, `tier`, `dal`, `severity`
- `mandate` — one sentence
- `standard` — DO-178C clause or exceeds
- `run` — command
- `fix` — targeted remediation
- `atom` — mapping to Excellent Code Framework atom

### New verifiers not in logic-os-kernel

| id | gate | mandate | standard |
|---|---|---|---|
| SEC.forbidden.api | security | No raw evaluateJavaScript or freeform network calls. | DO-178C §6.3.4 |
| SEC.capability.reach | security | Every mutation path reaches FactStore::append. | DO-178C §6.4 |
| SEC.origin.policy | security | URL policy enforced at first boundary. | DO-178C §6.3.4 |
| SEC.prompt.inject | security | Page content cannot be interpreted as instruction. | Exceeds DO-178C |
| COV.transition.mcdc | test | MC/DC coverage on state machine transition table. | DO-178C §6.4.4 |
| PERF.frame.budget | performance | Frame budget not regressed. | DO-178C §6.4 |
| PERF.alloc.budget | performance | Allocations per frame within declared limit. | DO-178C §6.4 |

## 7. Path Scoping

Docs-only PRs skip Rust jobs. Any code change outside `docs/**`, `laws/**`, or
`*.md` triggers full CI. Upstream human UI is deferred behind a variable until
the substrate is stable.

## 8. Failure Model

- `GO` — all block-severity verifiers green.
- `NO-GO` — any block-severity verifier failed/blocked.
- `RUNNER FAULT` — exit 2, tool qualification issue, treated as NO-GO.
- Advisory findings surface but never flip the verdict.

## 9. Required Artifacts

- Software Requirements Data (`docs/SPEC.md`, `docs/HLR.md`)
- Design Description (`docs/TOPOGRAPHY.md`, `docs/ARCH.md`)
- Source Code with traceability markers
- Test Procedures and Results (`tests/`, `.ci-runs/*/manifest.json`)
- Verification Report per release
- Problem Report and Change Records (GitHub issues + ADRs)
