# 2026-07-01 — Source of Truth Creation

## Date

2026-07-01

## Hypothesis

A definitive, evidence-based source of truth can be created by querying all databases in ingestion_results and mapping them against the browser-engine codebase.

## Design

1. Query all 5 databases in ingestion_results
2. Map what repos are indexed and what code is available
3. Map browser-engine crates and their status
4. Identify gaps between OKF spec and reality
5. Create SOURCE_OF_TRUTH.md as single reference document

## Method

1. Query repo_topology.db (45,903 nodes from openclaw)
2. Query repo_topology_v2.db (65,237 nodes from SWE-agent, langgraph, letta, openclaw, smolagents)
3. Query crawlers_source.db (5,485 files, 35,631 chunks from 20+ web scraping repos)
4. Query unified_agent_brain_multimodal.db (1,380 perceptions from logic-os-kernel, logicalworks, the-startup)
5. Query agent_brain.db (25 nodes: 10 failure modes, 12 lessons, 3 categories)
6. Inspect browser-engine workspace (14 crates, 129 tests)
7. Cross-reference OKF spec (16 subsystems) against implemented code

## Raw Data

### Database Summary
- repo_topology.db: 610MB, 45,903 nodes, 45,902 edges (openclaw)
- repo_topology_v2.db: 860MB, 65,237 nodes, 67,955 edges (SWE-agent, langgraph, letta, openclaw, smolagents)
- crawlers_source.db: 310MB, 5,485 files, 35,631 chunks (20+ repos)
- unified_agent_brain_multimodal.db: 94MB, 1,380 perceptions, 102 intelligence
- agent_brain.db: 25 nodes (10 failure modes, 12 lessons, 3 categories)

### Browser Engine State
- 14 crates, 129 tests, 5 API endpoints
- Pipeline: HTML → DOM → A11y → Semantic → PULSE → Wire (100% complete)
- JS → Transpile (100% complete)
- URL → Fetch (100% complete)
- State Management: 0%
- Search/Index: 0%

### Key Borrowable Repos (from crawlers_source)
- playwright-python: Browser automation API patterns
- puppeteer: Browser automation API patterns
- cheerio: DOM traversal patterns
- crawl4ai: AI-powered crawling integration
- scrapy: Spider patterns, middleware
- curl-cffi: TLS fingerprinting

## Observation

- **The corpus is rich but underutilized.** 20+ browser automation repos are indexed but not yet queried for patterns.
- **The browser engine is 75% complete** by OKF subsystem count (12/16).
- **The missing 25% is layout and execution** — the hardest parts.
- **State management and search are NOT in the OKF spec** but are needed for production use.
- **Borrowed code is declared and minimal** — only HTTP client pattern and SSRF prevention.

## Next Steps

1. Read SOURCE_OF_TRUTH.md for the definitive mapping
2. Decide: implement missing OKF subsystems (layout, execution) or productionize (state, search)?
3. Query crawlers_source.db for borrowable patterns if building browser automation features

## Tags

`analysis` `corpus` `source-of-truth`

## Claim Links

- [claim:ST-001] — Corpus contains 20+ browser automation repos
- [claim:ST-002] — Browser engine is 75% complete by OKF subsystem count
- [claim:ST-003] — State management and search are missing from OKF spec
- [claim:ST-004] — Layout engine is stub-only, needs implementation
- [claim:ST-005] — 129 tests passing, 100% pass rate
