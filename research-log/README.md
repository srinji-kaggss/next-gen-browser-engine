# Research Log

Append-only research log following the AI Semantic RAG Pack format.

## Format

Each entry is a markdown file named `YYYY-MM-DD-topic.md`.

### Entry Sections

| Section | Description |
|---------|-------------|
| **Date** | ISO date of entry |
| **Hypothesis** | What we believe and why |
| **Design** | Architecture/approach chosen |
| **Method** | How we tested/built/verified |
| **Raw Data** | Actual outputs, metrics, negative results |
| **Observation** | What we learned |
| **Next Steps** | What to do next |
| **Tags** | Classification tags |

### Tags

| Tag | Meaning |
|-----|---------|
| `spec` | Specification or requirements work |
| `design` | Architecture or design decisions |
| `experiment` | Running a test or trial |
| `failure` | Something didn't work |
| `success` | Something worked as expected |
| `anomaly` | Unexpected behavior worth investigating |

## Rules

1. **Append-only.** Never edit past entries. Only append new ones.
2. **Include negative results.** Failed experiments are data.
3. **Record time and date.** Every entry has a Date field.
4. **Link to claim IDs.** When an entry relates to a spec claim, reference it (e.g., `[claim:MW-01]`).

## Entries

| Date | Topic | Tags |
|------|-------|------|
| 2026-07-01 | [Phase 2 Spec](2026-07-01-phase2-spec.md) | `spec`, `design` |
