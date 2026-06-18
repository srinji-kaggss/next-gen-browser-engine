# Browser Engine Source Ingestion

Deterministic, exhaustive extraction of Chromium, WebKit, and Gecko into the lgwks
research SQLite schema. The ingestion lives in `scripts/` and produces a single
`research.sqlite` per engine that can be queried with standard SQL.

## Scripts

- `scripts/build_repo_graph.py` — build a deterministic graph from a git source tree.
- `scripts/repo_to_lgwks_db.py` — load that graph into canonical lgwks `research.sqlite`.

The pipeline is language-agnostic: it uses `git ls-tree`, `git grep`, and best-effort
C/C++/GN parsing. It optionally merges edges from `lgwks repo graph` when that tool
covers the repo's languages (currently `.py` and `.rs` only).

## Existing DBs

Built and verified locally:

| Engine | Path | Documents | Chunks | Nodes | Edges |
|---|---|---:|---:|---:|---:|---:|
| Chromium | `/Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite` | 519 053 | 46 212 | 572 717 | 1 661 885 |
| WebKit | `/Users/srinji/ingestion_results/webkit_lgwks_db/research.sqlite` | 456 079 | 27 059 | 564 715 | 602 314 |
| Gecko | `/Users/srinji/ingestion_results/gecko_lgwks_db/research.sqlite` | 387 841 | 35 326 | 429 722 | 466 799 |

The DBs are too large for git; only the scripts and this README are committed.

## Quick reference

| What | SQLite query |
|---|---|
| Tables and counts | `SELECT 'documents', COUNT(*) FROM documents UNION ALL SELECT 'chunks', COUNT(*) FROM chunks UNION ALL SELECT 'nodes', COUNT(*) FROM nodes UNION ALL SELECT 'edges', COUNT(*) FROM edges;` |
| Node / edge kinds | `SELECT kind, COUNT(*) FROM nodes GROUP BY kind;` / `SELECT kind, COUNT(*) FROM edges GROUP BY kind;` |
| Files under a top dir | `SELECT title FROM documents WHERE title LIKE 'content/%' ORDER BY title LIMIT 20;` |
| Most included files | `SELECT substr(e.to_id, 6), COUNT(*) FROM edges e WHERE e.kind = 'includes' GROUP BY e.to_id ORDER BY 2 DESC LIMIT 20;` |
| Includes from a file | `SELECT substr(e.to_id, 6), json_extract(e.metadata_json, '$.line') FROM edges e WHERE e.kind = 'includes' AND e.from_id = 'file:chrome/browser/ui/browser.cc' ORDER BY 2;` |
| Symbols in a header | `SELECT n.label, json_extract(n.metadata_json, '$.source_location') FROM edges e JOIN nodes n ON n.id = e.to_id WHERE e.from_id = 'file:base/logging.h' AND e.kind = 'defines' ORDER BY 2;` |
| Neighbors of a file | `SELECT e.kind, substr(CASE WHEN e.from_id = 'file:PATH' THEN e.to_id ELSE e.from_id END, 6) FROM edges e WHERE e.from_id = 'file:PATH' OR e.to_id = 'file:PATH' ORDER BY 1, 2 LIMIT 50;` |
| Full text of a file | `SELECT GROUP_CONCAT(c.text, CHAR(10)) FROM documents d JOIN chunks c ON c.document_id = d.id WHERE d.title = 'PATH';` |

Replace `content/` with `Source/WebCore/`, `dom/`, `widget/`, etc., and replace `PATH` with the repo-relative file path.

## Aggregate understandings

Structured counts per engine are in `docs/BROWSER_ENGINE_UNDERSTANDINGS.json`.
Highlights:

- **Chromium** is the largest by edges (~1.66M), driven by a dense C++ include graph. Dominant roots: `third_party/`, `chrome/`, `components/`. Top include targets are core `base/` primitives (`raw_ptr.h`, `bind.h`, `time.h`).
- **WebKit** has the richest symbol surface (~86K `class/struct/namespace` symbols) but a sparser include graph than Chromium. Roots are dominated by `LayoutTests/`, `Source/`, `JSTests/`.
- **Gecko** is the smallest of the three, with roots `testing/`, `mobile/`, `third_party/`, `js/`, `browser/`, `dom/`. It carries a small GN import footprint (25 edges) from shared build tooling.

## Run from scratch

### 1. Clone a shallow tree

```bash
mkdir -p /tmp/ingest-work && cd /tmp/ingest-work
git clone --filter=blob:none --depth 1 https://github.com/chromium/chromium.git repo
```

### 2. Extract raw git artifacts

```bash
cd repo
git ls-tree -r HEAD > /tmp/ingest-work/full_tree.txt
git grep -n '^#include' -- '*.cc' '*.cpp' '*.h' '*.hpp' '*.c' '*.mm' > /tmp/ingest-work/includes_raw.txt
git grep -n '^import("' -- '*.gn' '*.gni' > /tmp/ingest-work/gn_imports_raw.txt
```

### 3. Build the graph

```bash
cd /Users/srinji/next-gen-browser-engine
python3 scripts/build_repo_graph.py chromium /tmp/ingest-work/repo \
  /tmp/ingest-work/graph \
  --source-url https://github.com/chromium/chromium \
  --base-dir /tmp/ingest-work
```

Output:
- `/tmp/ingest-work/graph/graph.json`
- `/tmp/ingest-work/graph/manifest.jsonl`
- `/tmp/ingest-work/graph/meta.json`
- `/tmp/ingest-work/graph/summary.json`

### 4. Load into lgwks SQLite

```bash
python3 scripts/repo_to_lgwks_db.py chromium \
  /tmp/ingest-work/graph/graph.json \
  /tmp/ingest-work/repo \
  /tmp/ingest-work/lgwks_db \
  --source-url https://github.com/chromium/chromium \
  --tracked-entries $(wc -l < /tmp/ingest-work/full_tree.txt)
```

Output: `/tmp/ingest-work/lgwks_db/research.sqlite`

## Schema notes

- `documents.id` maps to `nodes.id` for `kind = 'file'` nodes via `file:{path}`.
- `edges.from_id` / `edges.to_id` are foreign keys into `nodes.id`.
- `edges.metadata_json` stores per-edge details such as `#include` line numbers.
- `nodes.metadata_json` stores `file_type`, `sha256`, `size` for files and `source_file` / `source_location` for symbols.
- `chunks` only covers the top 5 000 files by degree (most connected files) to keep the DB size bounded.

## Limitations

- `lgwks repo graph` itself only parses `.py` and `.rs`. The scripts here work around that by
  parsing `#include` and GN `import("...")` directly from `git grep` output.
- C/C++ include resolution is best-effort: relative to the including file, then repo root, then
  any trailing path suffix. Unresolved includes are counted in `meta.json` as `includes_dangling`.
- Symbol extraction is limited to `class/struct/enum/namespace` declarations in public-looking
  `.h` files.
