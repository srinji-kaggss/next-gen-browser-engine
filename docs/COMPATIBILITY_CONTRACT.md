# Developer Compatibility Contract

> Status: initial contract. This is the migration promise the engine must make
> true before "bring your app, we handle the browser/runtime work" is credible.

## 1. Product Promise

Developers should not tune for browser quirks, Java-era deployment constraints,
or engine internals. They describe the app surface; the browser classifies it,
admits safe capabilities, runs the appropriate brought-in engine muscle, and
records every effect on the canonical tape.

This is not "ship a JVM in the browser." Java/JVM compatibility is a
quarantined legacy lane. Java bytecode may be accepted through an adapter only
when it is bounded as guest compute and denied ambient classloader, filesystem,
network, reflection, and native-library authority unless represented by explicit
closed capabilities.

## 2. Intake Contract

`src/compat/mod.rs` classifies developer workloads:

| Surface | Mode | Current decision |
|---|---|---|
| HTML/CSS/DOM, codecs | Native engine | Bring in proven engine pieces. |
| JavaScript, Wasm | Guest compute | Requires `web.compute.local`. |
| Java bytecode | Quarantined legacy | Adapter/sandbox only; no core JVM model. |
| Host filesystem | Quarantined legacy | Requires explicit `web.fs.write`. |
| Network egress | Reject for now | Missing closed vocabulary capability. |
| NPAPI/plugins/extensions/unknown legacy | Reject | Ambient authority or product shell debt. |

## 3. What Is Missing

1. **Closed egress capability.** Docs mention `web.egress`, but
   `braid-vocab-web` currently has no network-write capability. Enterprise and
   Java-era apps often hide egress behind libraries; that must be representable
   before it can be safely admitted.
2. **Compatibility adapter interface.** The repo can classify Java bytecode as
   quarantined, but there is no adapter ABI yet for classpath, classloader,
   reflection, JNI/native library blocking, or syscall/capability mediation.
3. **Developer migration manifest.** There is no file format that lets a team
   declare "these are my legacy surfaces; here are the desired budgets; here is
   what may be optimized away."
4. **Optimization oracle.** Performance docs list budgets, but there is no
   analyzer that turns observed hot paths into stable, tape-backed optimization
   suggestions.
5. **Native bridge consumption.** WebKit load/raw JS still sit behind seams; the
   native bridge must consume policy-admitted actions and emit existing JSONL
   observations.

## 4. Reference Policy

Copy engine muscle only where existing browsers already converged. Do not copy
their ambient authority model. The compatibility boundary must be stricter than
Chromium/WebKit/Gecko plugin and extension history:

- no raw plugin ABI in the substrate;
- no ambient JVM classloader authority;
- no hidden egress;
- no native library loading from legacy code;
- no browser-specific optimization requirements imposed on app developers.

The developer-facing story is therefore: keep your app semantics; remove your
runtime/quirk obligations. The browser takes responsibility for classification,
capability admission, sandboxing, optimization, and audit.
