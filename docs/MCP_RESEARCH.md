# MCP/Chrome DevTools Research Findings

> Date: 2026-06-18
> Method: Direct observation via `mcp__chrome-devtools` on live pages.
> Goal: Understand what Chrome/CDP/MCP actually expose, so our own MCP design is grounded in real behavior, not speculation.

## 1. What the agent actually sees

Chrome DevTools Protocol (via the MCP wrapper) exposes two useful structured surfaces:

### 1.1 Accessibility tree snapshot
Example from `https://example.com`:

```text
uid=1_0 RootWebArea "Example Domain" url="https://example.com/"
  uid=1_1 heading "Example Domain" level="1"
  uid=1_2 StaticText "This domain is for use in documentation examples ..."
  uid=1_3 link "Learn more" url="https://iana.org/domains/example"
    uid=1_4 StaticText "Learn more"
```

Example from `https://github.com/login`:

```text
uid=3_0 RootWebArea "Sign in to GitHub · GitHub" url="https://github.com/login"
  uid=3_32 form
    uid=3_34 LabelText "Username or email address"
    uid=3_36 textbox "Username or email address" focusable focused required
    uid=3_39 LabelText "Password"
    uid=3_41 textbox "Password" required
    uid=3_43 link "Forgot password?"
    uid=3_47 button "Sign in"
  uid=3_56 button "Continue with Google"
  uid=3_62 button "Continue with Apple"
  uid=3_77 button "Sign in with a passkey"
```

**Finding:** The accessibility tree is already a typed, compact, machine-readable representation of the page. It is much closer to our desired Braid observation graph than raw HTML or pixels.

### 1.2 Network surface
GitHub `/login` triggered **97 network requests** including:
- 1 HTML document
- ~60 JavaScript bundles
- ~20 CSS files
- Analytics/telemetry endpoints (`collector.github.com`, `api.github.com/_private/browser/stats`)
- Feature-flag JSON embedded in the HTML (`client-env`)

**Finding:** The network layer is noisy. A production agent must filter network observations by origin, resource type, and privacy tier, not record everything.

### 1.3 DOM extraction timing (measured via `performance.now()`)

| Page | Total DOM nodes | Visible nodes | Extraction time | Tree bytes |
|---|---|---|---|---|
| example.com | 12 | 6 | 0.10 ms | 889 |
| iana.org/help/example-domains | 99 | 46 | 1.0 ms | — |
| github.com/login | ~300+ | 34 interactables | 1.1 ms | — |

**Finding:** DOM traversal is not the latency bottleneck. WebKit/Chrome layout, network, and JavaScript execution dominate. Symbolic extraction is cheap.

### 1.4 Interactable element discovery
A naive JS heuristic (`tagName === 'A' || tagName === 'BUTTON' || cursor === 'pointer'`) over-reports on GitHub login:
- Reports 34 interactables.
- Includes nested SVG/span children of the Google/Apple buttons as separate interactables.
- Includes invisible `Skip to content` link (bounds 1x1).

**Finding:** A robust agent must de-duplicate interactables by semantic role and visibility, not just tag name. The a11y snapshot is better than raw DOM for this.

## 2. What the agent actually does

The MCP tool surface we observed:
- `navigate_page(url)` — load URL.
- `take_snapshot()` — return a11y tree.
- `click(uid)` — click by a11y UID.
- `evaluate_script(function)` — run arbitrary JS.
- `list_network_requests()` — network log.
- `list_console_messages()` — console log.

**Finding:** The action vocabulary is small and closed by design. Our 9 verbs (`web.navigate`, `web.observe`, `web.click`, `web.type`, `web.scroll`, `web.download`, `web.wait`, `web.execute_js`, `web.execute_wasm`) map cleanly onto these MCP primitives.

## 3. Implications for our own MCP

### 3.1 The agent reads a text manifest + API version of affordances
This is viable because:
- CDP already produces a compact a11y tree.
- We can render that tree into an OKF/Braid text manifest with stable IDs.
- We can expose affordances as typed MCP tools bound to those IDs.

### 3.2 mac-eye is one driver, not the only architecture
- **Headless Chrome/CDP** is the cross-platform default driver.
- **mac-eye (WKWebView)** is a mac-native, higher-trust alternate driver.
- **Mock driver** for deterministic tests.

This directly challenges the old ADR-001 "Mac-Native Supremacy." The engine should not be locked to macOS.

### 3.3 The canonical anchor lives behind the MCP server
The MCP server is not the canonical state. It is the action/render gateway:

```text
Agent
  ├─ reads: text manifest (derived OKF/Braid lens)
  └─ calls: MCP tools

MCP Server
  ├─ routes each call through PolicyBroker
  ├─ executes via BrowserDriver (CDP / mac-eye / mock)
  └─ writes observations to Braid canonical anchor

BrowserDriver
  └─ Chrome DevTools Protocol / WKWebView / test fixture
```

### 3.4 Network and compute policy must be first-class
Observed GitHub login loads scripts from `github.githubassets.com`, analytics from `collector.github.com`, and feature flags from inline JSON. Our policy broker needs:
- Origin allowlist/denylist.
- Resource-type budget (JS, CSS, image, fetch, beacon).
- Telemetry blocking by default.

## 4. Open questions for Director

1. Should our own MCP server wrap an existing Chrome MCP (faster) or implement CDP directly (more control)?
2. Is the primary target still eventual competition with Chromium, meaning we must own the renderer long-term, or is the near-term goal a policy-gated agent layer over existing browsers?
3. Should mac-eye remain in the repo as a mac-native driver, or be extracted to a separate repo now that the core seam is driver-agnostic?

## 5. Traceability

- AXIOM_BRAID_CANONICAL: text manifest is derived from canonical anchor.
- AXIOM_OBSERVABILITY_TYPED: a11y tree maps to `ObservationKind::A11y`/`Element`.
- AXIOM_POLICY_AUTHORITY: every MCP tool call routes through broker.
- AXIOM_CLOSED_ACTIONS: MCP exposes only the 9 closed verbs.
- AXIOM_HUMAN_DEFERRAL: `web.click` on password field can return `Confirm`.
- AXIOM_DERIVED_LENS: OKF text manifest is a lens over the a11y/CBOR anchor.

## 6. Multimodal and native understanding assessment

This section answers the specific question: *what does Chrome/CDP/MCP give us for multimodal and native understanding, and where does our own work begin?*

### 6.1 Multimodal inputs Chrome provides well

| Modality | Chrome capability | Our access |
|---|---|---|
| Text / DOM / a11y | Excellent | CDP `Accessibility.*` and `DOM.*` domains |
| Screenshots / pixels | Excellent | `Page.captureScreenshot` |
| PDF / printed output | Good | `Page.printToPDF` |
| Video / WebRTC elements | Good (presence) | `DOM.querySelector` / media element metadata |
| Audio | Limited | Not a clean semantic transcript channel via CDP |
| Accessibility metadata | Very good | Role, name, state, bounds, labeled-by |

### 6.2 What Chrome does not provide

| Need | Chrome gap |
|---|---|
| Semantic element summary | A11y tree is noisy (`InlineTextBox`, ignored nodes, SVG tangles) |
| Action affordance graph | Must derive "clickable things with intent" ourselves |
| Cross-modal binding | Linking a screenshot region to an a11y node CID requires our own pixel→anchor mapping |
| Model-context compression | Chrome hands raw DOM/pixels; we must compress to Braid/OKF |
| Privacy-aware redaction | Chrome sees cookies/localStorage; we must gate what reaches the model |
| Closed action vocabulary for models | CDP exposes hundreds of commands; models cannot choose safely |

### 6.3 Native understanding distinction

- **Machine-native understanding** = deterministic symbolic extraction. Chrome does this well via the a11y tree; we make it canonical, typed, and policy-gated.
- **AI-native understanding** = the LLM comprehending the page as an agent. Chrome does not do this; it only supplies raw inputs.

### 6.4 Decision: Mode A — policy/canonicalization layer over existing browsers

We are **not** rebuilding the renderer. We are building the agent-facing interface that Chrome does not provide.

| Chrome worldview | Our worldview |
|---|---|
| Browser is for humans | Browser is for agents |
| CDP is a debugging surface | CDP is a sensory input to a policy-gated agent |
| JavaScript is a first-class environment | JavaScript is a capability-bounded compute lane |
| Cookies/storage are ambient authority | Every origin access requires a capability token |
| Page content is trusted within origin | Page content is data, never instruction |

### 6.5 Architecture consequence

```text
Chrome/CDP raw senses
  ├─ a11y tree     → Braid Element/Observation facts
  ├─ screenshot    → layout lens bound to element CIDs
  ├─ network log   → filtered Network observations
  └─ JS console    → filtered Console observations

Braid canonical anchor (CBOR + CID)
  ↓

OKF text manifest + affordance API
  ↓

Agent (Claude / small local model)
```

The agent receives:
- A text file (OKF) with stable CIDs and interactables.
- An MCP API with only the 9 closed verbs.
- A policy membrane that denies anything outside the declared scope.

### 6.6 Traceability

- AXIOM_BRAID_CANONICAL: a11y/DOM facts become CBOR facts under CID anchors.
- AXIOM_OBSERVABILITY_TYPED: each modality maps to a typed `Observation`.
- AXIOM_POLICY_AUTHORITY: every CDP/MCP call routes through `PolicyBroker`.
- AXIOM_CLOSED_ACTIONS: model may only use the 9 `web.*` verbs.
- AXIOM_PRIVACY_TIER: screenshot and storage data pass through redaction before reaching the model.
- AXIOM_SMALL_MODEL_OPERABLE: Braid/OKF compression is bounded for small-model context windows.

### 6.7 Logged action

Decision recorded here: **Mode A selected**. Implementation proceeds with a Chrome/CDP-first driver, a Braid canonical anchor, an OKF renderer, and a closed 9-verb MCP action surface. mac-eye becomes one alternate driver, not the sole architecture.

