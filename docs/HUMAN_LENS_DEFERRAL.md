# Human Lens Deferral

> Human UI is a product-shell lens. Build the substrate first; polish last.

## 1. Human Needs Are Real, But Not Day-0

Humans need:
- Pixels to look at.
- Tabs to organize sessions.
- Navigation chrome.
- Bookmarks/history.
- Smooth animations.
- Media playback.

These are all valid. They are also all **derived lenses** over canonical state.
They do not need to be in the substrate.

## 2. Deferral Strategy

| Human feature | Deferral | Reason |
|---|---|---|
| Tabs/windows | Phase 2+ | Sessions are named tapes; tabs are a rendering of active tapes. |
| Omnibox/search | Phase 2+ | Navigation is `web.navigate`; search is a capability-gated action. |
| Bookmarks | Phase 2+ | Bookmarks are saved `Action` facts on the tape. |
| History UI | Phase 2+ | History is the tape itself, rendered. |
| Smooth visual transitions | Phase 2+ | Animations are a layout lens; substrate needs only state transitions. |
| Media playback | Phase 2+ | Delegate to host OS media pipeline behind capability. |
| Themes/extensions | Phase 3+ | Product chrome; must not bypass capability broker. |

## 3. What Is Built Day-0

- Minimal Swift wrapper around `WKWebView` (existing `mac-eye`).
- It emits observations to the canonical anchor, not just raw JSON.
- It renders when a human lens is active, but rendering is a byproduct.
- Pointer/keyboard events resolve to CIDs via `PixelAnchor`.

## 4. The Human Shell Is Thin

```text
Human Shell
├── TabController — renders active tape sessions
├── AddressBar — proposes web.navigate actions
├── PointerMapper — calls PixelAnchor::resolve
├── DevToolsLens — renders canonical anchor + provenance
└── ConfirmDialog — renders PolicyVerdict::escalate
```

None of these own state. All state lives in the substrate.

## 5. Why This Prevents Tech Debt

By deferring human UI, we avoid:
- Entangling rendering with policy.
- Building extension APIs that bypass the action vocabulary.
- Optimizing for visual polish before correctness.
- Creating multiple sources of truth (DOM, UI state, agent state).

When human UI is added later, it is forced to consume the same canonical anchor
and capability broker as the AI.
