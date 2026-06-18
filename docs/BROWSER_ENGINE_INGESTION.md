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
|---|---|---|---:|---:|---:|
| Chromium | `/Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite` | 519 053 | 46 212 | 572 717 | 1 661 885 |
| WebKit | `/Users/srinji/ingestion_results/webkit_lgwks_db/research.sqlite` | 456 079 | 27 059 | 564 715 | 602 314 |
| Gecko | `/Users/srinji/ingestion_results/gecko_lgwks_db/research.sqlite` | 387 841 | 35 326 | 429 722 | 466 799 |

The DBs are too large for git; only the scripts and this README are committed.

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

## Querying the DB

Open any generated DB:

```bash
sqlite3 /Users/srinji/ingestion_results/chromium_lgwks_db/research.sqlite
```

### List tables and counts

```sql
SELECT 'documents' AS table_name, COUNT(*) FROM documents
UNION ALL SELECT 'chunks', COUNT(*) FROM chunks
UNION ALL SELECT 'nodes', COUNT(*) FROM nodes
UNION ALL SELECT 'edges', COUNT(*) FROM edges;
```

### Node kinds and edge kinds

```sql
SELECT kind, COUNT(*) FROM nodes GROUP BY kind ORDER BY 2 DESC;
SELECT kind, COUNT(*) FROM edges GROUP BY kind ORDER BY 2 DESC;
```

### Files under a top-level directory

```sql
SELECT title FROM documents
WHERE title LIKE 'content/%'
ORDER BY title
LIMIT 20;
```

Replace `content/` with `Source/WebCore/`, `dom/`, `widget/`, etc.

### Most included files

```sql
SELECT substr(e.to_id, 6) AS path, COUNT(*) AS in_degree
FROM edges e
WHERE e.kind = 'includes'
GROUP BY e.to_id
ORDER BY in_degree DESC
LIMIT 20;
```

### Files a given file depends on

```sql
SELECT substr(e.to_id, 6) AS included_path, json_extract(e.metadata_json, '$.line') AS line
FROM edges e
WHERE e.kind = 'includes'
  AND e.from_id = 'file:chrome/browser/ui/browser.cc'
ORDER BY line;
```

### Symbols defined in a header

```sql
SELECT n.label, json_extract(n.metadata_json, '$.source_location') AS loc
FROM edges e
JOIN nodes n ON n.id = e.to_id
WHERE e.from_id = 'file:base/logging.h'
  AND e.kind = 'defines'
ORDER BY loc;
```

### Neighbors of a file (any edge kind)

```sql
SELECT e.kind, substr(CASE WHEN e.from_id = 'file:chrome/browser/ui/browser.cc' THEN e.to_id ELSE e.from_id END, 6) AS neighbor
FROM edges e
WHERE e.from_id = 'file:chrome/browser/ui/browser.cc'
   OR e.to_id   = 'file:chrome/browser/ui/browser.cc'
ORDER BY e.kind, neighbor
LIMIT 50;
```

### Full text of a file

```sql
SELECT d.title, GROUP_CONCAT(c.text, CHAR(10)) AS content
FROM documents d
JOIN chunks c ON c.document_id = d.id
WHERE d.title = 'chrome/browser/ui/browser.cc'
GROUP BY d.id;
```

### Directory containment tree

```sql
SELECT substr(parent.label, 5) AS parent_dir,
       substr(child.label, 5) AS child_dir
FROM edges e
JOIN nodes parent ON parent.id = e.source
JOIN nodes child  ON child.id  = e.target
WHERE e.kind = 'contains'
  AND parent.label = 'dir:'
  AND child.kind = 'dir'
ORDER BY parent_dir, child_dir
LIMIT 30;
```

### Search file names

```sql
SELECT title FROM documents
WHERE title LIKE '%render_widget%'
ORDER BY title
LIMIT 20;
```

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
