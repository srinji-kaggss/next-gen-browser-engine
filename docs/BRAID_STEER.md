# Steer from the Braid project lead (2026-06-23)

> From: srinji-kaggss/Braid (the canonical IR). To: the browser-engine team.
> This is a steer, not a demand — the browser repo owns its own decisions.
> Cite `spec/braid/DECISIONS.md` D31 if you need the authority.

## Where Braid is now

Braid is no longer a browser-CMS framework. As of 2026-06-23 it is a
**standalone global-IR substrate** with two vocabulary packages:

- `braid-ir` — the substrate: `Value`/`canon`/`Cid`/`TermRegistry`/`Capsule`/
  `TypeTag` atoms. No domain vocabulary baked in.
- `braid-capability` — `Capability` is a string newtype
  (`Capability::new("web.dom.read")`), not a fixed enum. Each vocabulary owns
  its capability space; the verifier's attenuation check works on any token set.
- `braid-verify` — the ONE admission verifier. Registry-parametric: admits
  against any vocabulary's `TermRegistry`. Independent decoder (D9).
- `braid-vocab-cms` — the kernel/landing-port CMS vocabulary (the 10 kernel
  capability verbs as named consts + `registry_v0`).
- `braid-vocab-js` — the JS elaboration vocabulary (`js.*` capability space).
  Proves the global-IR claim: a JS capsule admits via the one `braid-verify`
  with a capability space the kernel never knew about.

103 tests green, clippy clean, fmt clean. `braid-capability` is crates.io-
publishable; the rest are `cargo add --git https://github.com/srinji-kaggss/Braid`
once a tag is cut.

## What this means for the browser engine

Your `docs/BRAID_BRIDGE.md §6` already says the right thing: "The browser
depends on `braid-ir` and `braid-capability`… It does not recreate Braid's
registry." Your `src/braid_bridge/term.rs` currently does the opposite — it
defines a parallel `BraidTerm` enum that drifts from the real IR. That is the
one collapse target.

### The collapse (when you're ready)

1. `cargo add braid-ir braid-capability braid-vocab-cms --git https://github.com/srinji-kaggss/Braid`
   (or path-dep for now). Cut a Braid tag first if you want a pinned rev.
2. Delete `src/braid_bridge/term.rs`'s `BraidTerm` enum. The browser's `web.*`
   terms become a **new vocabulary package** — either publish your own
   `braid-vocab-web` crate, or (if it's browser-engine-internal) keep it in-tree
   as a module that constructs a `braid_ir::TermRegistry` with `web.*` terms.
   The CMS vocab is the template to copy.
3. `WebAnchor` ↔ Braid goes through `braid_ir::Capsule`/`Cid`/`Value`, not a
   parallel enum. `BraidAdapter::to_braid` returns `braid_ir::Value` (the
   canonical form) or a `Capsule`, not `BraidTerm`.
4. Your `web.*` capability space (`web.dom.read`, `web.observe`, `web.navigate`,
   `web.egress` per `JS_WASM_POSITION.md §6`) is just more `Capability::new(...)`
   tokens — foreign to `js.*` and `signal.*`, same verifier.

### What you get for free

- The 8-stage fail-closed admission pipeline (canonical-form → version →
  structure → types → capability → effect → path-taint → bounds). You do not
  build a second verifier. Your `web.*` registry plugs into `braid-verify::verify`.
- Content-addressed CIDs (BLAKE3, `lw.braid.*` domains) — matches your ADR-005
  "immutable content-addressed acyclic term graph" exactly.
- The manifest + widening-diff gate (D12/T12) — your A2 "canonical state is the
  review object" is already built.
- Bijection guard, malleability regressions, the U9 adversarial pass — all
  closed on the substrate you'd inherit.

### What stays yours

- The `web.*` vocabulary definition (your term specs, your capability lattice
  order, your `EgressMediated` classification — exactly what your kernel
  `braid_vocab_binding.rs` asserts).
- The DOM/AX observation seam (`src/observation/anchor.rs`) — Braid carries no
  browser types.
- The render path (ADR-001: "the render is the visible fork") — Braid is the
  state/policy substrate, not the renderer.
- LLM sensors (ADR-007) — advisory, at the boundary, never in Braid's verifier.

## The one thing I'd push back on

`BRAID_BRIDGE.md §5` says the current CID is "SHA-256 (64 hex digits). Target
hash function is BLAKE3." Don't carry SHA-256 through the collapse — Braid's
CIDs are BLAKE3 under `lw.braid.*` domains (D8). The browser's `Cid = String`
interface should become `braid_ir::Cid` directly. A parallel hash discipline
is exactly the "second authority system" D11 forbids.

## Sequencing from my side

- I'll cut a `braid-v0.1` git tag when you're ready to depend (say the word).
- The kernel's `braid_vocab_binding.rs` snapshot is the same seam — when it
  live-wires, it should decode `braid_vocab_cms::registry_v0()` (the dotted
  names are preserved verbatim, so its snapshot assertions stay green).
- I am NOT building a `braid-vocab-web` package from the Braid side — the
  browser's `web.*` vocabulary is yours, not mine. Braid ships the substrate
  + the CMS/JS examples; every consumer owns its own domain vocabulary.

— Braid project lead