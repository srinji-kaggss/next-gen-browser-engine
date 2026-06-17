# AX-Browser: High-Level Requirements (HLR)

## 1. System Overview
The AX-Browser is a Mac-native, AI-first semantic browser. It discards visual rendering in favor of semantic state extraction (OKF).

## 2. Requirements

### HLR-01: Mac-Native Engine
The browser MUST execute using the host OS's native `WKWebView`. 
* **Tracing Marker:** `Tracing: HLR-01`

### HLR-02: OKF Pipeline
The browser MUST serialize page state into OKF with stable reference IDs (`@e1`).
* **Tracing Marker:** `Tracing: HLR-02`

### HLR-03: Ambient Ingestion
The browser MUST support vectorizing human interaction trajectories.
* **Tracing Marker:** `Tracing: HLR-03`

### HLR-04: Axiomatic Fault Tolerance
The browser MUST transition between deterministic states.
* **Tracing Marker:** `Tracing: HLR-04`

### HLR-05: Policy Membrane
All data MUST pass through a PII scrubber before hitting storage.
* **Tracing Marker:** `Tracing: HLR-05`
