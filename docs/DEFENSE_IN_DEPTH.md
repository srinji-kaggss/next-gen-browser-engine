# Defense in Depth: Browser as World-Class Anti-Malware

> The browser is a hostile-code execution environment. Anti-malware is not a
> bolt-on. It is the architecture.

## 1. Core Doctrine

Following Braid D23 and the Logic OS capability model:

> **Confinement, not detection.** Detection is undecidable (Rice/Cohen). A
> closed typed vocabulary makes dangerous capability *unrepresentable*, turning
> classes of malice into confinement theorems.

The browser enforces a six-layer DiD stack:

1. **Capability confinement**
2. **Non-interference**
3. **Effect typing**
4. **Totality / bounds**
5. **Human confirm for irreversible effects**
6. **Manifest re-check and replay**

## 2. Threat Model

| Threat | Example | Layer that stops it |
|---|---|---|
| Malicious page JS | Keylogger, cryptominer, fingerprinting | L1 capability confinement |
| XSS via prompt injection | Page text tricks agent into clicking | L6 manifest re-check + human confirm |
| Drive-by download | Auto-download exploit | L3 effect typing + L5 confirm |
| Origin escape | A.com reads B.com storage | L1 origin policy + capability broker |
| Sandbox escape | Renderer compromise → host FS | L2 non-interference + OS sandbox |
| Supply-chain exploit | Compromised dependency | L6 audit + cargo-audit + recorded rationale |
| Bad AI agent | LLM proposes destructive action | L4 policy broker + L5 human escalate |
| Bad actor developer | Injects raw `evaluateJavaScript` | CI `SEC.forbidden.api` gate |
| Timing / side-channel | Covert channel via layout thrashing | L4 totality/bounds + budget enforcement |
| Data exfiltration | Page embeds PII in egress | L3 taint tracking + L5 confirm |

## 3. Layer 1 — Capability Confinement

- The only host effects available to page code are those in the capability
  vocabulary: `web.read`, `web.write`, `web.observe`, `web.compute.local`,
  `web.egress` (audited), etc.
- Capabilities are signed, scoped, and attenuation-only.
- A capability token is bound to `(tenant, principal, scope, nonce, signature)`.
- `CapabilitySet::check(action, caller)` is DAL-A and returns a `Verdict` with
  `rationale_cid`.

## 4. Layer 2 — Non-Interference

- Renderer process has no direct access to host FS, host network, or other
  origins' storage.
- All cross-origin data flows through the capability broker with taint tracking.
- `Taint` is a monotone exposure label: `Public`, `Tenant`, `Private`,
  `EgressMediated`. Data can only move to labels that are equal or wider.

## 5. Layer 3 — Effect Typing

Every action term has an effect signature:

```text
web.navigate(url)       : effect [network.read, history.write]
web.click(target_cid)     : effect [dom.mutate, event.dispatch]
web.execute_js(hash, caps)  : effect [compute.local, caps...]
web.download(target_cid)  : effect [fs.write, history.write]  irreversible
web.egress(bytes)         : effect [network.write]             irreversible, audited
```

Effect composition is checked by the verifier. A capsule cannot claim a narrower
effect set than the composition of its strands.

## 6. Layer 4 — Totality and Bounds

- Every compute lane has a declared budget: time, memory, allocations, network
  bytes, DOM mutations.
- `Totality`: functions must terminate or be bounded by budget. No unbounded
  loops in the policy path.
- Budget violations are facts on the tape and may trigger `PolicyVerdict::deny`.

## 7. Layer 5 — Human Confirm for Irreversible Effects

Irreversible actions (`download`, `egress`, `password_fill`, `payment_submit`)
require human confirm or a pre-authorized standing grant.

The confirm dialog is not a UX checkbox. It is a `PolicyVerdict::escalate` that
presents:
- The action term.
- The capability set being exercised.
- The provenance chain of the target.
- The outcome predicted by the policy broker.

## 8. Layer 6 — Manifest Re-Check and Replay

- Before execution, the verifier re-checks the admitted action capsule.
- After execution, the outcome is recorded on the tape.
- Any run can be replayed from a tape CID to reproduce state and detect drift.
- The CI gate statically rejects forbidden APIs and verifies tape reachability.

## 9. Bad-AI Specific Defenses

### Prompt injection
- Page text is parsed as data, not instructions, by the embedding sensor.
- The LLM receives observations with explicit provenance labels.
- The policy broker's input is typed `Action` terms, not raw LLM text.
- A `PageInstructionFirewall` strips instruction-like patterns from page text
  before it reaches any proposer.

### Sycophancy / reward hacking
- The LLM cannot observe the reward signal or the policy decision directly.
- Actions are evaluated by the deterministic math spine, not by the LLM's
  self-assessment.

### Capability escalation by AI
- AI can propose actions only within the caller's `CapabilitySet`.
- Proposals that require broader capability are `deny` with an escalation path.
- AI cannot mint tokens, grant permissions, or bypass the broker.

### Model drift
- Embedding models are version-pinned. Sensor observations include
  `(model_version, schema_version, vector_cid)`.
- The policy broker declares which sensor versions it accepts.

## 10. Bad-Actor Developer Defenses

- CI `SEC.forbidden.api` rejects `evaluateJavaScript` with non-literal strings,
  `URL(string:)` without policy check, raw file/network access, and global state
  mutation outside `FactStore::append`.
- CI `SEC.capability.reach` verifies every write path reaches the tape.
- CI `COH.dup.*` prevents drifted twins and single-SoT violations.
- All source code must carry `//why` rationale at non-obvious trust boundaries.
