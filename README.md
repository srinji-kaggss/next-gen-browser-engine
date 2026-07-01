# Next-Gen Browser Engine (AX-Browser)

A self-contained, Mac-native, AI-first semantic browser engine. Built to be token-efficient, bot-resilient, and compliant with DO-178C aviation software standards.

## Core Pillars
1. **Local macOS Supremacy**: Uses `WKWebView` to inherit the host's actual network stack and Safari trust, bypassing bot-detection fingerprints.
2. **OKF (Optimized Knowledge Format)**: Serializes page state into a semantic XML subset with spatial bounding boxes, reducing token counts by 90%+.
3. **Axiomatic State Machine**: Deterministic transitions between navigation, challenges, and ready states (DO-178C).
4. **Ambient Ingestion**: Background telemetry for building agent training datasets (Pokemon Go paradigm).

## Project Structure
- `native/mac-eye/`: Swift binary that commands the macOS WebKit stack.
- `python/`: Python orchestrator and OKF transformation lens.
- `ci/`: Airworthiness and traceability verification scripts.
- `docs/`: Formal HLR specs and Architecture Decision Records (ADRs).

## Getting Started (macOS)
```bash
# 1. Build the native bridge
cd native/mac-eye && swift build -c release

# 2. Run a test render
cd ../../python
python3 engine.py https://example.com
```
