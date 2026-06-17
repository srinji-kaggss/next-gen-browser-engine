# Machine Observability Model

> Observability is not logs. It is a typed, content-addressed, replayable fact
> graph that both LLM and human can consume through separate lenses.

## 1. The Fact is the Unit

Everything that happens in the browser is a `Fact`:

```text
Fact {
  cid: BLAKE3(canonical_encoding),
  parent_cid: Option<CID>,
  kind: FactKind,
  payload: BraidTerm,
  provenance: Provenance,
  timestamp: MonotonicTick,
}
```

`FactKind` variants:
- `Observation` — page state observation
- `Action` — proposed or executed action
- `CapabilityCheck` — capability grant lookup
- `PolicyVerdict` — broker admission decision
- `ComputeEvent` — JS/Wasm execution event
- `NetworkEvent` — fetch/cookie/cache event
- `LayoutLens` — viewport-dependent geometry
- `UserInput` — pointer/keyboard event resolved to CID
- `Outcome` — result of an action

## 2. Two Angles of Observability

### Angle A — For the LLM
The LLM sees a symbolic graph. It predicts the next state by reasoning over:
- Current canonical anchor CID.
- Admitted actions and their effects.
- Policy verdicts and capability bounds.
- Historical transitions from the tape.

The LLM does not see:
- Pixels.
- Full DOM text.
- Layout coordinates unless explicitly requested.
- Raw JS heap state.

### Angle B — For the Human
The human sees rendered lenses. Human input is resolved back to the canonical
anchor through `PixelAnchor`:

```text
resolve(viewport_cid, x, y) -> Vec<(Candidate {
  element_cid,
  coverage,      // 0.0..1.0
  z_order,
  visibility,
  provenance,
})>
```

The human points; the browser returns candidate canonical identities. The human
selects; the agent uses the same `element_cid`.

## 3. The Meeting Point: PixelAnchor

`PixelAnchor` is the bidirectional bridge:

```text
PixelAnchor
├── resolve(viewport_cid, x, y) -> [Candidate]
├── project(element_cid, viewport_cid) -> LayoutLens
├── trace(element_cid) -> (observation_cid, [Fact])
├── stale_check(element_cid) -> Current | Stale { new_cid }
└── observe(viewport_cid) -> [LayoutLens]
```

This is how a human pointing at a button and an LLM proposing `web.click` end
up targeting the same canonical fact.

## 4. Provenance Chain

Every derived fact knows its chain:

```text
Provenance {
  source_cid: CID,           // where the raw bytes came from
  transformation: Term,      // what produced this fact
  policy_cid: CID,           // which policy admitted it
  intent_uuid: UUID,         // which user/agent intent this served
  reversibility: Reversibility,
}
```

A human devtool can render this as:
```text
This button [@e3]
  came from: fetch(cid=abc123)
  observed by: observation_pipeline_v2
  admitted by: policy_verdict(cid=def456)
  on behalf of: intent_uuid=uuid789
  action: reversible
```

## 5. Replay and Diff

From any tape CID, `replay(cid)` reconstructs the canonical state by replaying
the fact chain. `diff(cid_a, cid_b)` returns a structured delta of facts, not
text.

This makes debugging deterministic:
- A bug state is a CID.
- Reproducing it is `replay(cid)`.
- Comparing two runs is `diff`.
- Regressions are detected by CI diff against golden tapes.

## 6. No Hidden Side Effects

A side effect is hidden if it does not produce a fact. The CI gate
`SEC.capability.reach` verifies that every write path appends a fact. If a
component cannot produce a fact for an effect, the effect is unrepresentable
and therefore forbidden.

## 7. AIP / LLM Devtools Panels

The developer lens renders panels over the canonical fact fabric:

| Panel | Purpose |
|---|---|
| **Agent State** | Compact symbolic state exposed to the model. |
| **Action Graph** | Valid affordances, pre/postconditions, risk, confirmation. |
| **Model Context** | `raw state → redacted state → prompt projection → token count`. |
| **Privacy Diff** | Compare `local_full`, `cloud_redacted`, `cloud_selective_reveal`, `cloud_full_context_explicit`. |
| **Policy Trace** | Why an action was allowed/confirmed/denied. |
| **Prompt-Injection Inspector** | Trust-class labeling and blocked instruction-like content. |
| **Small-Model Test** | Validate that a 2B model can choose from `|valid actions| <= 12`. |

## 8. Telemetry vs. Tape

Telemetry is not a separate stream. Human interaction trajectories are recorded
as `UserInput` facts on the tape, paired with the current canonical snapshot.
Before persistence:
- PII is scrubbed deterministically.
- Vectors are computed by the frozen embedding sensor.
- The tape append is content-addressed and tamper-evident.
