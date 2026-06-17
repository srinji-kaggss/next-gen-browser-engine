# Machine-Human Meeting at the Screen

> The human points. The machine knows. They meet at a content-addressed anchor.

## 1. The Problem

Current browsers make humans and AI look at different things:
- Humans see pixels and DOM.
- AI sees chunked text or screenshots.
- Neither shares a stable identity for "the thing I clicked."

This creates:
- Brittle selectors (`@e1` reorders when the page changes).
- Prompt-injection via DOM text.
- Agents that click the wrong element because they reasoned over text, not
  semantics.
- Humans who cannot verify why an agent did what it did.

## 2. The Solution: PixelAnchor + Canonical Anchor

```text
Screen (pixels)
    ↑ rendered by
LayoutLens (bounds, viewport)
    ↑ derived from
Canonical Anchor (typed Braid terms)
    ←→ LLM symbolic view
```

A human click at `(x, y)` resolves to a set of `element_cid` candidates. The LLM
proposes `web.click(element_cid)`. The policy broker admits it. The action
executes. The outcome is recorded on the tape.

## 3. Human Devtools Rebuilt

Current devtools are DOM inspectors with a console. They breed tech debt
because they let developers mutate state arbitrarily and then inspect the
wreckage.

Our devtools are **audit-first**:

| Tool | Function | Prevents debt by |
|---|---|---|
| **Pointer Inspector** | Point at screen → see CID + provenance | Forcing identity before action |
| **Provenance Trace** | Show fact chain for any element | Making causality visible |
| **Action Preview** | Propose action → see verdict + predicted diff | Preventing bad actions |
| **Replay from CID** | Reconstruct any past state | Making bugs reproducible |
| **Diff Two Sessions** | Compare tape CIDs structurally | Catching regressions |
| **Policy Rationale** | Explain why an action was admitted/denied | Teaching the model |
| **Capability Ledger** | Show every grant exercised | Surfacing over-privilege |
| **Privacy Diff** | Show what each model tier sees | Preventing accidental cloud leaks |
| **Trust Class Inspector** | Color facts by trust boundary | Stopping prompt-injection drift |

The default debugging surface is the canonical anchor, not the DOM.

## 4. For New Coders

New developers learn the browser by reasoning about:
- Facts, not DOM nodes.
- Actions, not event handlers.
- Capabilities, not permissions.
- Tapes, not logs.

This reduces tech debt because:
- Arbitrary JS injection is unrepresentable in production.
- Every action has a policy rationale.
- Bugs are replayable from a CID.
- Refactors are diffed against golden tapes.

The devtool explains *why* an action was admitted or denied in natural
language, surfacing the policy rationale as a teaching moment.

## 5. Example Workflow

1. Human points at a "Submit" button.
2. Browser resolves `(x, y)` to `element_cid = cid_submit_42`.
3. Devtool shows:
   - `cid_submit_42` is a `Button` affordance.
   - It came from `fetch(cid_page_7)` and `observation_pipeline_v2`.
   - It has `effect [dom.mutate, event.dispatch]`.
   - Clicking it is reversible until `form.submit` fires.
4. Human asks agent to "click Submit."
5. Agent proposes `web.click(cid_submit_42)`.
6. Policy broker checks capability, effect, and confirm policy.
7. Action executes; outcome appended to tape.
8. Human can replay the entire sequence from the tape CID.
