# JS + Wasm Position

> Unified runtime boundary, separate language semantics, shared capability and
> memory accounting. JS and Wasm are guests; the symbolic engine owns the
> substrate.

## 1. The Mistake to Avoid

Do not merge JS and Wasm into one language or one VM. That creates a slop
runtime that inherits all of JS's dynamic mess and none of Wasm's safety
benefits.

## 2. The Correct Shape

```text
Compute Lane Manager
├── JS Realm
│     ├── ECMAScript parser + interpreter
│     ├── WebIDL bindings (generated from IDL + capability schema)
│     ├── dynamic capability broker calls
│     └── GC integration with host
├── Wasm Realm
│     ├── Binary decoder + validator
│     ├── Module instantiation
│     ├── Import table = capability table
│     ├── Linear memory + table sandbox
│     └── No GC (unless Wasm-GC enabled later)
└── Shared
      ├── One sandboxed process
      ├── One capability broker
      ├── One audit tape
      ├── One string/handle interning boundary
      └── One memory accounting policy
```

## 3. JS is Rented Instinct

JS execution is untrusted compute. We embed an existing engine on day one:
- **macOS**: JavaScriptCore (inherited from WebKit).
- **Linux/elsewhere**: SpiderMonkey or V8, capability-bounded.

Default execution mode: interpreted. JIT requires an explicit capability grant
and a budget. This is acceptable because most agent sessions do not need
adversarial JS to run fast.

## 4. Wasm is a Capability-Bounded Compute Lane

Wasm modules are validated, deterministic, and sandboxed:
- Binary format validated against the spec.
- Imports are resolved only from the capability table.
- Memory is isolated; bounds checks are enforced.
- Every host call is a `ComputeEvent` fact.

Wasm is the preferred lane for:
- Deterministic page compute.
- Engine plugins (filters, analyzers, codecs) behind capability imports.
- AI-proposed transformations running in a WASI-like constrained sandbox.

## 5. Shared Interning Boundary

To avoid GC stutter across the JS/Wasm ↔ host boundary:
- Strings are interned to stable `u32` IDs once.
- DOM handles are opaque `HandleId`s.
- Hot-loop data is written to pre-allocated shared memory planes (e.g.,
  Float32Array of positions).
- The host reads directly from the plane; no object allocation per frame.

## 6. Capability Table

Both JS and Wasm use the same capability table:

```text
CapabilityTable {
  web.read: read from current origin,
  web.observe: emit observation facts,
  web.navigate: load a new URL,
  web.compute.local: CPU-bounded local compute,
  web.egress: audited network write,
  web.fs.read: read from host FS (rare, confirmed),
  web.fs.write: write to host FS (rare, confirmed, irreversible),
}
```

A JS realm or Wasm module is instantiated with a `CapabilitySet`. It cannot
exercise capabilities outside that set. Runtime dynamic imports require broker
approval.

## 7. ComputeEvent Fact

Every meaningful compute action produces a fact:

```text
ComputeEvent {
  realm: RealmId,
  capability: Capability,
  entry_point: ScriptCid | ModuleCid,
  duration_us: u64,
  allocations: u64,
  outcome: Ok(Cid) | Err(ComputeError),
}
```

This makes JS/Wasm execution fully observable without parsing their internal
state.

## 8. Execution Admission

Before a script or module runs:
1. Static capability analysis where possible.
2. Capability set is computed.
3. Policy broker admits `web.execute_js` or `web.execute_wasm` action.
4. Execution is bounded by budget.
5. Outcome is appended to tape.

## 9. Legacy JVM Compatibility Is Quarantined

Java/JVM compatibility is a legacy lane, not a design center. If a target page
or enterprise surface truly requires Java-era behavior, it enters through a
compatibility adapter with the same capability table, policy admission, byte
budget, and tape obligations as every other guest runtime.

Strategic rule: no Java runtime, JVM object model, classloader authority, or
ambient permission pattern may shape the core substrate. Compatibility exists
to contain old dependencies while the browser moves the platform toward
smaller, typed, capability-bounded compute.
