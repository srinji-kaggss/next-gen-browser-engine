# Performance Strategy: Faster Than WebKit, Cleaner Than Gecko, Chromium-Capable

> We aim to outperform WebKit on our supported subset while inheriting the
> security/process features of Chromium and the clarity/memory-safety ambition
> of Gecko. We do this by doing less, doing it deterministically, and doing it
> with machine-first observability.

## 1. Why Existing Engines Are Slow for Our Use Case

| Engine | Slowness source | Our advantage |
|---|---|---|
| WebKit | Full web compat, JIT tiering, media pipeline, human UI chrome | Declared subset; no JIT-by-default; no media chrome |
| Chromium | Multi-process overhead, telemetry, extensions, 2M-line surface | Lean process model; no extensions/telemetry product |
| Gecko | Quantum/Stylo/WebRender ambition + legacy XPCOM glue | Clean Rust core; no legacy debt on day one |

We are not building a general browser. We are building a web-cognition
substrate. The performance budget is smaller by design.

## 2. Performance Principles

### P1 — Do less
- No full WebGL/WebGPU/media pipeline unless explicitly requested.
- No JIT for untrusted JS unless profiled and capability-authorized.
- No extension content scripts, no telemetry, no sync.
- No human UI chrome in the substrate.

### P2 — Determinism is cheaper than adaptivity
Adaptive JITs and heuristics spend cycles guessing. A deterministic pipeline
with declared budgets does not:
- Quiescence contract before snapshotting (no arbitrary waits).
- Fixed viewport and policy-controlled network timing.
- Logical observations are CID-stable across viewports.

### P3 — Layout is constraint-first
- Adopt Stylo/WebRender lessons: parallel style, GPU-first display lists.
- Layout IR is a constraint problem, not a C++ inheritance tree.
- Target block/inline/flex/grid first; tables and print layout deferred.

### P4 — Rendering is lens-optional
- Most agent sessions do not need pixels. They need the symbolic anchor.
- Pixels are produced only when a human lens is active or for verification
  diffing.
- Display lists are the transport; raster is the last mile.

### P5 — Observability is not overhead
Every transition is a fact, but facts are compact, typed, and content-addressed:
- No free-form logging.
- No DevTools-style ad-hoc instrumentation.
- The tape replaces logs, metrics, and crash dumps.

## 3. Concrete Performance Targets

| Metric | Target | Measurement |
|---|---|---|
| Snapshot-to-OKF latency | <50 ms for 1000-node page | `web.observe` end-to-end |
| Action execution latency | <16 ms for click/type | Policy broker + dispatch |
| Allocations per frame | 0 in hot path | CI `PERF.alloc.budget` |
| Frame budget | 60 fps when rendering | CI `PERF.frame.budget` |
| Memory growth | <5% per hour continuous use | Tape + heap profiler |
| Network fetch latency | Ride host stack; no extra hop | CFNetwork / nsurl |
| JS execution | Interpreted by default; JIT requires grant | Capability check |
| Wasm instantiation | <1 ms for small modules | Benchmark regression gate |

## 4. The Hot Path

```text
web.navigate(url)
  → URL policy check (O(1))
  → host fetch (CFNetwork, inherited)
  → HTML tokenizer + tree builder (streaming)
  → DOM as human lens
  → observation pipeline (logical facts only)
  → Braid term graph / canonical anchor
  → policy broker admits observe action
  → tape append
  → agent semantic graph derived
  → LLM/proposer consumes observations
```

The LLM consumes symbolic CIDs, not pixels or full DOM. That is the win.

## 5. GPU and Compositor Strategy

- Use **Metal** on macOS, **Vulkan/WebGPU compute** elsewhere.
- Display lists are the IR; raster is batched and damage-tracked.
- Borrow WebRender's GPU-first philosophy: the CPU builds a display list; the
  GPU renders it.
- Avoid a full Skia port. Use platform vector APIs or GPU compute for the
  supported subset.

## 6. Concurrency Model

| Lane | Concurrency |
|---|---|
| Renderer process | One per origin (Chromium model) |
| Network process | Async, bounded connection pool |
| Symbolic process | Deterministic, single-threaded core; parallel for derived lenses |
| Compute lane | JS/Wasm sandbox; preemptible by budget |
| Tape writer | Sequential append with content addressing |

Lightweight concurrency comes from:
- Rust async for I/O-bound lanes.
- Process isolation for untrusted code.
- Parallel derived-lens computation over immutable canonical state.

## 7. Memory Strategy

- Generational arena for cross-language handles (WASM ↔ host).
- Liveness leases: host periodically sends alive IDs; engine reaps unleased
  generations.
- Bounded tape tail with snapshots (Causal Tape model from Logic OS).
- Zero-allocation hot path: the core observe→anchor→tape path must not allocate.
- Zero-copy where possible: shared memory planes for vectors, string interning,
  display lists.

## 8. Regression Gating

Performance is a CI gate, not a dashboard:

- `PERF.frame.budget`: no regression vs. baseline on reference pages.
- `PERF.alloc.budget`: allocations per frame bounded.
- `PERF.observe.latency`: snapshot-to-anchor latency bounded.
- `PERF.action.latency`: action dispatch latency bounded.
- `PERF.memory.growth`: memory growth over fixed-duration test bounded.

Regressions fail the gate. No exceptions without recorded rationale.
