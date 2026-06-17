# Browser as World-Class Anti-Malware

> A browser is a hostile-code execution environment. Ours is designed like an
> antivirus: confinement-first, detection-second, replay-always.

## 1. The Antivirus Analogy

Traditional antivirus:
- Allows all code.
- Tries to detect bad patterns.
- Fails against novel malware.

Our browser:
- Removes dangerous capability from the vocabulary.
- Confines all code to least-privilege sandboxes.
- Records every effect on an append-only tape.
- Replays any session to detect drift.

This is defense in depth, not detection theater.

## 2. Malware Classes and Countermeasures

### Keyloggers / input stealers
- Page JS cannot read keyboard input unless the input field is in its origin
  and the capability `web.read` is granted for that field.
- AXObserver/telemetry is capability-gated and PII-scrubbed.
- `UserInput` facts record which principal consumed the input.

### Cryptominers / CPU abusers
- Untrusted JS runs interpreted by default.
- JIT requires `web.compute.local` with a CPU budget.
- Budget violations are facts and may trigger termination.

### Fingerprinting / tracking
- Network timing and canvas fingerprinting produce facts, not hidden side
  channels.
- Partitioned storage and cookie policy enforced by capability broker.
- `web.egress` is audited and budget-limited.

### Drive-by downloads
- `web.download` is an irreversible action requiring policy admission + human
  confirm or pre-authorized grant.
- Auto-download via hidden iframe is unrepresentable in the closed action
  vocabulary.

### XSS / prompt injection
- Page text is data. It never reaches the policy broker as instruction.
- `PageInstructionFirewall` strips instruction-like patterns before LLM
  consumption.
- LLM output is not authority; it is filtered through typed action proposal.
- Every fact carries a **trust class**:
  - `SYSTEM_POLICY` — browser/site/user policy; never overridden by page content.
  - `DEVELOPER_POLICY` — site-signed agent schema and policy.
  - `USER_INTENT` — current user goal.
  - `TRUSTED_STATE` — signed/typed state from same-origin app runtime.
  - `UNTRUSTED_CONTENT` — user-generated content, ads, comments, email bodies.
- Untrusted content cannot become instructions.

### Extension malware
- No extension model in the substrate. All code is either page code
  (sandboxed) or host code (policy-brokered).

### Supply-chain attacks
- `cargo audit` is a CI gate.
- Vendored dependencies are pinned and hashed.
- Tool-qualification scripts self-test.

### Ransomware / FS encryptors
- Renderer has no host FS authority.
- `web.fs.write` requires human confirm, a standing grant, and a narrow scope.

## 3. The Six DiD Layers (Reprise)

1. **Capability confinement** — dangerous effects unrepresentable.
2. **Non-interference** — origins and processes isolated.
3. **Effect typing** — every action's effects are statically known.
4. **Totality/bounds** — budgets prevent runaway compute.
5. **Human confirm** — irreversible effects require human gate.
6. **Manifest re-check + replay** — every admitted action is re-verified and
   every outcome is recorded.

## 4. Detection as Secondary Layer

Confinement handles the known cases. Detection handles the residual:
- Behavioral anomaly detection on the tape (advisory, DAL-D).
- Differential replay: same tape CID should reproduce the same state.
- Static analysis in CI rejects forbidden APIs and unsafe patterns.
- Fuzzers probe for capability escalation and sandbox escape.

Detection never overrides confinement. If detection says "maybe safe" but the
capability broker says "deny," the answer is deny.

## 5. Replay as the Ultimate Audit

Any malware that slips through can be reconstructed:
- The tape records every input, action, and outcome.
- `replay(tape_cid)` reproduces the infected state.
- `diff(golden_cid, infected_cid)` isolates the divergence.
- The divergence becomes a new test case and a new confinement rule.
