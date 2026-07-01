# Mobile Handoff — Browser ↔ logic-os ↔ Braid wiring

> For a Claude session continuing this on a phone. Everything below is on GitHub
> (no local state needed). Read this first, then pick the next increment.
> Written 2026-06-24.

## The one mental model
A browser is **the OS subsystem for running untrusted networked code safely** —
logic-os's hardest problem. So `next-gen-browser-engine` is **logic-os's execution +
presentation plane** (run JS/Wasm, render, isolate per-origin); **logic-os** is the
control plane / **brain** (capability · tape · rights · identity · sandbox · model-port).

- **Braid** (`srinji-kaggss/Braid`) is the **one canonical core** (content addressing,
  IR, canonical encoding, term registry/vocabulary). **Both** the browser and logic-os
  *consume* Braid; **neither depends on the other.** A user can download just the browser
  (it pulls Braid as a git dep) without running logic-os.
- Rule: **brain written + restricted-grammar (A18); muscle brought-in + caged (A12).
  Heavy in capability, minimal in what we author.** Every increment **deletes**
  re-declared/hand-rolled code (consume Braid) rather than adding.

Full strategy: `logic-os-kernel/docs/browser-as-os-strategic-gaps.md`.

## What is DONE and on `main` (all merged, all verified)
| Repo | `main` @ | What landed |
|---|---|---|
| `Braid` | `a9c9a0f` | `braid-ir`+`braid-capability` made **`no_std`** (#12); **`braid-vocab-web`** crate — canonical home for the closed `web.*` verb vocabulary (#13) |
| `next-gen-browser-engine` | `f55c93f` | `Cid` now IS `braid_ir::Cid` (deleted the hand-rolled SHA-256 CID; #13); observation canonical encoding now uses Braid's `canon`+`Value` (deleted the hand-rolled serializer); broker origin is a typed `Action.origin` field |
| `logic-os-kernel` | `a9e3e81` | strategy doc only (#703) — the semantic line + under-planning finding |

Root cause that was fixed (don't re-derive): Braid was **std-only**, so no `no_std`
substrate could depend on it — that's why both repos had re-declared/snapshotted the IR.

## NEXT increments (bottom-up, in order — each is one PR that removes repeat)

1. **Wire the browser to `braid-vocab-web`** (the verb list is in THREE drifting copies):
   - `src/browser_types.rs` `ActionVerb` enum (the 9 `web.*` strings),
   - `src/policy/broker.rs` `is_closed_verb` (same 9 strings),
   - `ci/structural_airworthiness_gate.mjs` `allowedVerbs` Set (same 9).
   Collapse onto `braid_vocab_web::registry_v0()` (add `braid-vocab-web` as a git dep,
   same `rev` as the other braid crates). The broker check becomes
   `registry_v0().get(verb).is_some()`. The JS gate can't import Rust — emit the verb
   list from the registry to a small generated JSON the gate reads (one source).
   *Design note:* decide whether `ActionVerb` stays (ergonomic enum) as a typed view
   over the registry, or is replaced by registry lookups.

2. **`src/braid_bridge/`** — `term.rs`'s `BraidTerm`/`WebObservation`/… are a misnamed
   *shadow* of Braid's model and have **no consumer** (grep confirms: only an `okf.py`
   comment). Either map `to_braid` to `(term_id, braid_ir::Value)` against the registry,
   or delete the dead seam. Don't keep a shadow that calls itself "Braid".

3. **`python/okf.py`** (the human-readable OKF lens) re-implements what
   `braid-render` already does (`manifest` / `render_text` / `manifest_diff`). Consolidate
   — note the shape gap: `braid-render` renders a `Capsule`, `okf` renders observations,
   so this needs a small design pass, not a 1:1 swap.

4. **Finish native/kernel-backed seams** against kernel primitives (the brain):
   `capability/mod.rs` now validates and fails closed but still needs the real
   DID signer; `compute/lane_manager.rs` now admits JS/Wasm effects but still
   needs the native sandbox runtime; any remaining tape/native bridge work
   should consume `logic-os-kernel` primitives, not re-author them locally.

5. **logic-os-kernel side:** retire the pinned-snapshot seam
   (`kernel/crates/canvas-syscall/tests/braid_vocab_binding.rs`, issues #565/#602) for a
   live `braid_ir::registry_v0()` decode — now possible because Braid is `no_std`.

## Working on a phone — practical constraints (be honest about verification)
- **All three repos are PRIVATE** under `srinji-kaggss`. The browser depends on Braid via
  a **git dependency**, so any `cargo build/fetch` needs
  **`CARGO_NET_GIT_FETCH_WITH_CLI=true`** (cargo's libgit2 can't auth to a private repo;
  the `git` CLI can).
- If no Rust toolchain is available on the device: do **reading / planning / code edits /
  PRs via GitHub** (the GitHub tools), and **defer `cargo` verification to CI or a desktop
  session**. **Do not claim "tests pass" without running them** — say what's verified vs.
  pending (CLAUDE.md: claim = demonstration).
- The browser dep pin lives in `Cargo.toml`: `braid-ir`/`braid-capability` at
  `rev = "fdd8515"`. **Bump all braid crate revs together** to a single Braid `main` commit
  when adding `braid-vocab-web` — a cargo git dep resolves **one rev** per repo, so mixed
  revs won't compile.

### Verification commands (when a toolchain exists)
```
# browser (from repo root)
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo test --all-features
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo clippy --all-features -- -D warnings
CARGO_NET_GIT_FETCH_WITH_CLI=true cargo check --no-default-features --features no_std
node ci/structural_airworthiness_gate.mjs --phase=all      # must print "OK: all"
# Braid (from repo root) — no_std proof is the bare-metal target
cargo test -p braid-vocab-web
cargo build -p braid-ir --target x86_64-unknown-none
```

### Gotchas already hit (save yourself the loop)
- `#![no_std]` + `extern crate alloc;` must come **after** the `//!` module-doc block
  (inner doc comments must precede all items — else `E0753`).
- The airworthiness gate **string-scans** for `web.<word>` and rejects any not in its
  allow-list. Don't put `web.*` literals in non-vocab code; CID domains are named
  `lw.browser.*` for exactly this reason.
- New no_std code needs explicit `use alloc::{string::String, vec::Vec, vec, ...}` — the
  std prelude isn't there.

## Do NOT touch
- **Browser PR #11** (`feat/chrome-cdp-adapter`) — driving Chrome over CDP. **ADR-001
  explicitly rejected this direction** ("standalone browser, the render is the fork; not a
  remote driver"). Leave it parked / close it; do not merge.

## Pointers
- Semantic line + canonical ownership table: `logic-os-kernel/docs/browser-as-os-strategic-gaps.md`
- Browser end-state: `docs/{TOPOGRAPHY,AXIOMS,ANTIVIRUS_BROWSER,JS_WASM_POSITION}.md`, `docs/ADR.md`
- Braid vocab pattern to mirror: `Braid/crates/braid-vocab-cms/src/lib.rs`
- Code DB for browser-engine evidence: `~/ingestion_results/QUERYME.md` (desktop only; multi-GB)
