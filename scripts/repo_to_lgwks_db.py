#!/usr/bin/env python3
"""Load a deterministic source graph into the lgwks research SQLite DB.

Usage:
  python3 repo_to_lgwks_db.py NAME GRAPH_PATH REPO_ROOT OUT_DIR [--source-url URL] [--tracked-entries N]

Output: OUT_DIR/research.sqlite
"""

import argparse
import hashlib
import json
import os
import re
import sqlite3
import subprocess
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path

CHUNK_SAMPLE_SIZE = 5000
CHUNK_SIZE_TOKENS = 512  # Azure / Firecrawl default starting point
CHUNK_OVERLAP_TOKENS = 128  # ~25% overlap per Azure guidance; Firecrawl treats as tunable
TOKENS_PER_CHAR = 0.25  # ~4 characters per token for code / technical English
CHUNK_SIZE_CHARS = int(CHUNK_SIZE_TOKENS / TOKENS_PER_CHAR)  # ~2048 chars
CHUNK_OVERLAP_CHARS = int(CHUNK_OVERLAP_TOKENS / TOKENS_PER_CHAR)  # ~512 chars

GRAPH_PATH = Path()
REPO_ROOT = Path()
OUT_DIR = Path()
DB_PATH = Path()
RUN_ID = ""
SOURCE_ID = ""
SOURCE_URL = ""
NAME = ""
TRACKED_ENTRIES = 0


def now_iso() -> str:
    return datetime.now(timezone.utc).isoformat()


def sha256_str(text: str) -> str:
    return hashlib.sha256(text.encode("utf-8")).hexdigest()


def sha256_bytes(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def word_count(text: str) -> int:
    return len(re.findall(r"\b\w+\b", text))


def _get_separators(path: str) -> list[str]:
    """Return language-aware separator hierarchy, coarsest first."""
    ext = Path(path).suffix.lower()

    if ext == ".py":
        return [
            "\n\n",
            "\nclass ",
            "\ndef ",
            "\nasync def ",
            "\n    def ",
            "\n    async def ",
            "\n",
            " ",
        ]

    if ext in {".c", ".cc", ".cpp", ".h", ".hpp", ".mm", ".m", ".java", ".swift", ".kt"}:
        return [
            "\n\n",
            "\n};\n",
            "\n}\n",
            "\nnamespace ",
            "\nclass ",
            "\nstruct ",
            "\ntemplate",
            "\nvoid ",
            "\nint ",
            "\nbool ",
            "\nstd::",
            "\n",
            " ",
        ]

    if ext in {".js", ".ts", ".jsx", ".tsx", ".go"}:
        return [
            "\n\n",
            "\nfunction ",
            "\nconst ",
            "\nlet ",
            "\nvar ",
            "\nclass ",
            "\nexport ",
            "\nimport ",
            "\n",
            " ",
        ]

    if ext in {".md", ".markdown"}:
        return ["\n\n# ", "\n\n## ", "\n\n### ", "\n\n", "\n", " "]

    if ext in {".html", ".htm", ".xml", ".svg"}:
        return ["\n\n", "<", "\n", " "]

    if ext in {".json", ".yaml", ".yml"}:
        return ["\n\n", "\n", " "]

    # Generic text / unknown
    return ["\n\n", "\n", " "]


def _recursive_split(text: str, separators: list[str], chunk_size: int) -> list[str]:
    """Recursively split text into chunks no larger than chunk_size.

    Splits on the coarsest separator first and falls back to finer separators
    for oversized pieces, preserving structural boundaries where possible.
    """
    if not text:
        return []
    if len(text) <= chunk_size:
        return [text]
    if not separators:
        # No separators left: hard split and recurse on the remainder.
        return [text[:chunk_size]] + _recursive_split(text[chunk_size:], [], chunk_size)

    sep = separators[0]
    parts = text.split(sep)
    if len(parts) == 1:
        # Separator not useful; try the next finer one.
        return _recursive_split(text, separators[1:], chunk_size)

    chunks: list[str] = []
    current: list[str] = []
    current_len = 0

    for part in parts:
        part_len = len(part) + (len(sep) if current else 0)
        if current and current_len + part_len > chunk_size:
            chunks.append(sep.join(current))
            current = [part]
            current_len = len(part)
        else:
            current_len += part_len if current else len(part)
            current.append(part)

    if current:
        chunks.append(sep.join(current))

    # Recurse on any chunk that is still too large.
    result: list[str] = []
    finer = separators[1:] if len(separators) > 1 else []
    for chunk in chunks:
        if len(chunk) > chunk_size:
            result.extend(_recursive_split(chunk, finer, chunk_size))
        else:
            result.append(chunk)
    return result


def chunk_text(
    text: str,
    path: str,
    chunk_size: int = CHUNK_SIZE_CHARS,
    overlap: int = CHUNK_OVERLAP_CHARS,
) -> list[str]:
    """Chunk text for code-aware RAG embedding.

    Strategy (hardened against Firecrawl + Azure chunking guidance):
      * 512-token target chunks (~2048 chars, Azure starting default).
      * 128-token overlap (~512 chars, ~25%, Azure recommendation).
      * Recursive, language-aware splitting that respects structural boundaries.
      * Each chunk prefixed with its source path so middle chunks retain context.
    """
    text = text.replace("\r\n", "\n").replace("\r", "\n")
    if not text.strip():
        return []

    separators = _get_separators(path)
    # Base chunks are sized so that prefix + overlap + content fits the budget.
    prefix = f"[{path}]\n"
    base_budget = max(64, chunk_size - len(prefix) - overlap)
    base_chunks = _recursive_split(text, separators, base_budget)

    final_chunks: list[str] = []
    prev_overlap = ""
    for i, base in enumerate(base_chunks):
        # Prepend the trailing overlap of the previous chunk to preserve continuity.
        content = prev_overlap + base
        final = prefix + content

        # Hard safety trim in case a single structural unit still exceeds budget.
        available = chunk_size - len(prefix)
        if len(content) > available:
            content = content[:available]
            final = prefix + content

        final_chunks.append(final)

        # Compute overlap for the next chunk from this chunk's *content* (no prefix).
        if overlap > 0 and i < len(base_chunks) - 1:
            prev_overlap = content[-overlap:] if len(content) >= overlap else content
        else:
            prev_overlap = ""

    return final_chunks


def load_graph() -> dict:
    print(f"Loading {GRAPH_PATH}", file=sys.stderr)
    with GRAPH_PATH.open("r", encoding="utf-8") as f:
        return json.load(f)


def init_schema(conn: sqlite3.Connection) -> None:
    schema_sql = """
    CREATE TABLE IF NOT EXISTS meta(
        key TEXT PRIMARY KEY,
        value TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS runs(
        run_id TEXT PRIMARY KEY,
        name TEXT NOT NULL,
        created_at TEXT NOT NULL,
        schema_version TEXT NOT NULL,
        manifest_path TEXT NOT NULL,
        prompt TEXT NOT NULL,
        keyword_json TEXT NOT NULL,
        config_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS sources(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        url TEXT NOT NULL,
        title TEXT NOT NULL,
        axis TEXT NOT NULL,
        tier TEXT NOT NULL,
        raw_path TEXT,
        status TEXT NOT NULL,
        error TEXT,
        elapsed_seconds REAL NOT NULL DEFAULT 0,
        discovered_by TEXT NOT NULL DEFAULT 'seed',
        score REAL NOT NULL DEFAULT 0
    );
    CREATE TABLE IF NOT EXISTS documents(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        source_id TEXT NOT NULL,
        title TEXT NOT NULL,
        path TEXT NOT NULL,
        content_sha256 TEXT NOT NULL,
        word_count INTEGER NOT NULL,
        chunk_count INTEGER NOT NULL
    );
    CREATE TABLE IF NOT EXISTS chunks(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        document_id TEXT NOT NULL,
        source_id TEXT NOT NULL,
        position INTEGER NOT NULL,
        text TEXT NOT NULL,
        content_sha256 TEXT NOT NULL,
        word_count INTEGER NOT NULL,
        semantic_type_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS embeddings(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        scope TEXT NOT NULL,
        target_id TEXT NOT NULL,
        provider TEXT NOT NULL,
        model TEXT NOT NULL,
        dimensions INTEGER NOT NULL,
        vector_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS nodes(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        kind TEXT NOT NULL,
        label TEXT NOT NULL,
        weight REAL NOT NULL,
        metadata_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS edges(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        from_id TEXT NOT NULL,
        to_id TEXT NOT NULL,
        kind TEXT NOT NULL,
        weight REAL NOT NULL,
        evidence TEXT,
        metadata_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS understandings(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        scope TEXT NOT NULL,
        before_snapshot_id TEXT,
        after_snapshot_id TEXT,
        summary TEXT NOT NULL,
        coverage_score REAL NOT NULL,
        uncertainty_score REAL NOT NULL,
        evidence_json TEXT NOT NULL,
        schema_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS question_events(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        drill_id TEXT NOT NULL,
        ask_index INTEGER NOT NULL,
        question TEXT NOT NULL,
        what_were_you_thinking TEXT NOT NULL,
        expected_information_gain REAL NOT NULL,
        answered INTEGER NOT NULL DEFAULT 0,
        answer TEXT
    );
    CREATE TABLE IF NOT EXISTS snapshots(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        phase TEXT NOT NULL,
        page_count INTEGER NOT NULL,
        chunk_count INTEGER NOT NULL,
        node_count INTEGER NOT NULL,
        edge_count INTEGER NOT NULL,
        frontier_json TEXT NOT NULL,
        top_terms_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS drills(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        keyword TEXT NOT NULL,
        state TEXT NOT NULL,
        target_pages INTEGER NOT NULL,
        crawled_pages INTEGER NOT NULL,
        ask_count INTEGER NOT NULL,
        compute_estimate_seconds REAL NOT NULL,
        metadata_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS crawl_events(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        created_at TEXT NOT NULL,
        url TEXT NOT NULL,
        status TEXT NOT NULL,
        elapsed_seconds REAL NOT NULL,
        detail_json TEXT NOT NULL
    );
    CREATE TABLE IF NOT EXISTS compressed_nodes(
        id TEXT PRIMARY KEY,
        run_id TEXT NOT NULL,
        reason TEXT NOT NULL,
        source_node_json TEXT NOT NULL,
        compressed_label TEXT NOT NULL,
        metadata_json TEXT NOT NULL
    );
    CREATE INDEX IF NOT EXISTS idx_edges_run ON edges(run_id);
    CREATE INDEX IF NOT EXISTS idx_edges_from ON edges(from_id);
    CREATE INDEX IF NOT EXISTS idx_edges_to ON edges(to_id);
    CREATE INDEX IF NOT EXISTS idx_edges_kind ON edges(kind);
    CREATE INDEX IF NOT EXISTS idx_nodes_run ON nodes(run_id);
    CREATE INDEX IF NOT EXISTS idx_nodes_kind ON nodes(kind);
    CREATE INDEX IF NOT EXISTS idx_documents_run ON documents(run_id);
    """
    conn.executescript(schema_sql)


def insert_meta(conn: sqlite3.Connection, commit: str) -> None:
    meta = [
        ("run_id", RUN_ID),
        ("source", SOURCE_URL),
        ("commit", commit),
        ("extraction_method", "git tree + git grep includes/gn + lgwks repo graph"),
        ("graph_path", str(GRAPH_PATH)),
        ("repo_root", str(REPO_ROOT)),
    ]
    conn.executemany("INSERT OR REPLACE INTO meta(key, value) VALUES (?, ?)", meta)


def insert_run(conn: sqlite3.Connection, counts: dict, commit: str) -> None:
    prompt = f"deterministic structural extraction of the {NAME} source tree"
    config = {
        "max_depth": None,
        "max_pages": None,
        "engine": "git + lgwks hybrid",
        "same_site": False,
        "search_expansion": False,
        "workers": 1,
    }
    conn.execute(
        """INSERT INTO runs(run_id, name, created_at, schema_version, manifest_path, prompt, keyword_json, config_json)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            RUN_ID,
            f"{NAME}-source-deterministic",
            now_iso(),
            "jarvis-crawl/2",
            str(OUT_DIR / "manifest.jsonl"),
            prompt,
            json.dumps([NAME, "browser engine", "source code", "dependency graph"]),
            json.dumps(config, sort_keys=True),
        ),
    )


def insert_source(conn: sqlite3.Connection) -> None:
    conn.execute(
        """INSERT INTO sources(id, run_id, url, title, axis, tier, raw_path, status, error,
                              elapsed_seconds, discovered_by, score)
           VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
        (
            SOURCE_ID,
            RUN_ID,
            SOURCE_URL,
            f"{NAME} source tree",
            "source",
            "primary",
            str(REPO_ROOT),
            "ok",
            None,
            0.0,
            "seed",
            1.0,
        ),
    )


def insert_documents(conn: sqlite3.Connection, graph: dict) -> dict:
    file_nodes = [n for n in graph["nodes"] if n.get("kind") == "file"]
    doc_map = {}
    rows = []
    missing = 0
    for i, node in enumerate(file_nodes):
        nid = node["id"]
        path = nid[5:]
        fpath = REPO_ROOT / path
        content = ""
        size = node.get("size", 0)
        sha = node.get("sha256", "")
        if fpath.exists():
            try:
                data = fpath.read_bytes()
                content = data.decode("utf-8", errors="ignore")
                sha = sha256_bytes(data)
                size = len(data)
            except Exception:
                missing += 1
        wcount = word_count(content)
        doc_id = f"doc-{sha256_str(nid)[:16]}"
        doc_map[nid] = doc_id
        rows.append(
            (doc_id, RUN_ID, SOURCE_ID, path, str(fpath), sha, wcount, 0)
        )
        if (i + 1) % 50000 == 0:
            print(f"  prepared {i + 1}/{len(file_nodes)} documents", file=sys.stderr)

    print(f"Inserting {len(rows)} documents ({missing} unreadable)", file=sys.stderr)
    with conn:
        conn.executemany(
            """INSERT INTO documents(id, run_id, source_id, title, path, content_sha256,
                                     word_count, chunk_count)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            rows,
        )
    return doc_map


def insert_chunks_for_sample(conn: sqlite3.Connection, graph: dict, doc_map: dict) -> None:
    degree = Counter()
    for e in graph["links"]:
        degree[e.get("source", "")] += 1
        degree[e.get("target", "")] += 1

    file_nodes = [n for n in graph["nodes"] if n.get("kind") == "file"]
    ranked = sorted(file_nodes, key=lambda n: degree.get(n["id"], 0), reverse=True)
    sample = ranked[:CHUNK_SAMPLE_SIZE]

    chunk_rows = []
    for node in sample:
        nid = node["id"]
        doc_id = doc_map.get(nid)
        if not doc_id:
            continue
        path = nid[5:]
        fpath = REPO_ROOT / path
        if not fpath.exists():
            continue
        try:
            text = fpath.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        if not text.strip():
            continue
        chunks = chunk_text(text, path)
        for pos, chunk in enumerate(chunks):
            chunk_id = f"{doc_id}-chunk-{pos}"
            chunk_rows.append(
                (
                    chunk_id,
                    RUN_ID,
                    doc_id,
                    SOURCE_ID,
                    pos,
                    chunk,
                    sha256_str(chunk),
                    word_count(chunk),
                    json.dumps(
                        {
                            "constraint": 0.0,
                            "evidence": 0.0,
                            "machine": 0.0,
                            "risk": 0.0,
                            "state": 0.0,
                            "topology": 0.0,
                        }
                    ),
                )
            )

    print(f"Inserting {len(chunk_rows)} chunks for {len(sample)} files", file=sys.stderr)
    with conn:
        conn.executemany(
            """INSERT INTO chunks(id, run_id, document_id, source_id, position, text,
                                  content_sha256, word_count, semantic_type_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            chunk_rows,
        )
    counts_by_doc = Counter(c[2] for c in chunk_rows)
    with conn:
        conn.executemany(
            "UPDATE documents SET chunk_count = ? WHERE id = ?",
            [(cnt, doc_id) for doc_id, cnt in counts_by_doc.items()],
        )


def insert_nodes(conn: sqlite3.Connection, graph: dict) -> None:
    rows = []
    for i, node in enumerate(graph["nodes"]):
        nid = node["id"]
        kind = node.get("kind", "unknown")
        label = node.get("label", nid)
        weight = float(node.get("weight", 1.0))
        meta = {k: v for k, v in node.items() if k not in ("id", "label", "kind", "weight")}
        rows.append(
            (nid, RUN_ID, kind, label, weight, json.dumps(meta, sort_keys=True, default=str))
        )
        if (i + 1) % 100000 == 0:
            print(f"  prepared {i + 1}/{len(graph['nodes'])} nodes", file=sys.stderr)

    print(f"Inserting {len(rows)} nodes", file=sys.stderr)
    with conn:
        conn.executemany(
            "INSERT INTO nodes(id, run_id, kind, label, weight, metadata_json) VALUES (?, ?, ?, ?, ?, ?)",
            rows,
        )


def insert_edges(conn: sqlite3.Connection, graph: dict) -> None:
    rows = []
    for i, edge in enumerate(graph["links"]):
        eid = f"edge-{i:08d}-{sha256_str(json.dumps(edge, sort_keys=True))[:12]}"
        src = edge.get("source", "")
        tgt = edge.get("target", "")
        kind = edge.get("kind", "unknown")
        weight = float(edge.get("weight", 1.0))
        evidence = edge.get("evidence")
        meta = {k: v for k, v in edge.items() if k not in ("source", "target", "kind", "weight", "evidence")}
        rows.append(
            (eid, RUN_ID, src, tgt, kind, weight, evidence, json.dumps(meta, sort_keys=True, default=str))
        )
        if (i + 1) % 200000 == 0:
            print(f"  prepared {i + 1}/{len(graph['links'])} edges", file=sys.stderr)

    print(f"Inserting {len(rows)} edges", file=sys.stderr)
    with conn:
        conn.executemany(
            """INSERT INTO edges(id, run_id, from_id, to_id, kind, weight, evidence, metadata_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?)""",
            rows,
        )


def insert_snapshot(conn: sqlite3.Connection, graph: dict) -> str:
    snapshot_id = f"snapshot-{RUN_ID}"
    node_kinds = Counter(n.get("kind") for n in graph["nodes"])
    edge_kinds = Counter(e.get("kind") for e in graph["links"])
    frontier = {
        "node_kinds": dict(node_kinds),
        "edge_kinds": dict(edge_kinds),
        "top_level_dirs": [n["label"] for n in graph["nodes"] if n.get("kind") == "dir" and n["label"].count("/") == 0 and n["label"] != "."],
    }
    top_terms = [
        {"term": NAME, "weight": 100.0},
        {"term": "browser", "weight": 90.0},
        {"term": "engine", "weight": 80.0},
        {"term": "layout", "weight": 70.0},
        {"term": "dom", "weight": 60.0},
    ]
    with conn:
        conn.execute(
            """INSERT INTO snapshots(id, run_id, created_at, phase, page_count, chunk_count,
                                      node_count, edge_count, frontier_json, top_terms_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                snapshot_id,
                RUN_ID,
                now_iso(),
                "complete",
                sum(1 for n in graph["nodes"] if n.get("kind") == "file"),
                0,
                len(graph["nodes"]),
                len(graph["links"]),
                json.dumps(frontier, sort_keys=True, default=str),
                json.dumps(top_terms, sort_keys=True),
            ),
        )
    return snapshot_id


def insert_understanding(conn: sqlite3.Connection, snapshot_id: str, graph: dict, commit: str) -> None:
    node_kinds = Counter(n.get("kind") for n in graph["nodes"])
    edge_kinds = Counter(e.get("kind") for e in graph["links"])
    summary = (
        f"Deterministic structural extraction of {SOURCE_URL} at {commit}. "
        f"{len(graph['nodes'])} nodes ({dict(node_kinds)}) and {len(graph['links'])} edges "
        f"({dict(edge_kinds)}). Includes full git tree manifest, C/C++ include graph, "
        f"GN import graph where present, and best-effort public-header symbols. "
        f"Coverage: {node_kinds.get('file', 0)} of {TRACKED_ENTRIES} tracked files."
    )
    evidence = {
        "commit": commit,
        "graph_path": str(GRAPH_PATH),
        "repo_root": str(REPO_ROOT),
        "extraction_steps": [
            "git ls-tree -r HEAD",
            "git grep '#include' -- '*.cc' '*.h'",
            "git grep '^import(\"' -- '*.gn' '*.gni'",
            "best-effort public-header symbol extraction",
        ],
    }
    with conn:
        conn.execute(
            """INSERT INTO understandings(id, run_id, created_at, scope, before_snapshot_id,
                                           after_snapshot_id, summary, coverage_score,
                                           uncertainty_score, evidence_json, schema_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                f"understanding-{RUN_ID}",
                RUN_ID,
                now_iso(),
                f"{NAME}-source-tree",
                None,
                snapshot_id,
                summary,
                0.85,
                0.25,
                json.dumps(evidence, sort_keys=True),
                json.dumps({"schema": f"{NAME}-extract.v1"}, sort_keys=True),
            ),
        )


def insert_drill_and_events(conn: sqlite3.Connection) -> None:
    drill_id = f"drill-{RUN_ID}"
    with conn:
        conn.execute(
            """INSERT INTO drills(id, run_id, keyword, state, target_pages, crawled_pages,
                                  ask_count, compute_estimate_seconds, metadata_json)
               VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?)""",
            (
                drill_id,
                RUN_ID,
                f"{NAME} source structure",
                "complete",
                TRACKED_ENTRIES,
                TRACKED_ENTRIES,
                0,
                0.0,
                json.dumps({"deterministic": True, "source": "git"}),
            ),
        )
        conn.execute(
            """INSERT INTO crawl_events(id, run_id, created_at, url, status, elapsed_seconds, detail_json)
               VALUES (?, ?, ?, ?, ?, ?, ?)""",
            (
                f"crawl-{RUN_ID}-tree",
                RUN_ID,
                now_iso(),
                SOURCE_URL,
                "ok",
                0.0,
                json.dumps({"operation": "git ls-tree", "entries": TRACKED_ENTRIES}),
            ),
        )


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("name", help="short repo name: chromium, webkit, gecko")
    parser.add_argument("graph_path", type=Path, help="path to graph.json")
    parser.add_argument("repo_root", type=Path, help="path to the cloned source tree")
    parser.add_argument("out_dir", type=Path, help="output directory for research.sqlite")
    parser.add_argument("--source-url", default="", help="canonical source URL")
    parser.add_argument("--tracked-entries", type=int, default=0, help="total tracked git entries")
    args = parser.parse_args()

    global GRAPH_PATH, REPO_ROOT, OUT_DIR, DB_PATH, RUN_ID, SOURCE_ID, SOURCE_URL, NAME, TRACKED_ENTRIES

    NAME = args.name.lower()
    ts = datetime.now(timezone.utc).strftime("%Y%m%d-%H%M")
    RUN_ID = f"{NAME}-source-{ts}"
    SOURCE_ID = f"source-{NAME}-github"
    GRAPH_PATH = args.graph_path.resolve()
    REPO_ROOT = args.repo_root.resolve()
    OUT_DIR = args.out_dir
    DB_PATH = OUT_DIR / "research.sqlite"
    SOURCE_URL = args.source_url or f"https://github.com/{NAME}/{NAME}"
    TRACKED_ENTRIES = args.tracked_entries

    if not GRAPH_PATH.exists():
        print(f"Missing graph: {GRAPH_PATH}", file=sys.stderr)
        sys.exit(1)

    commit = "unknown"
    try:
        commit = subprocess.run(
            ["git", "-C", str(REPO_ROOT), "rev-parse", "HEAD"],
            capture_output=True, text=True, check=True, timeout=30
        ).stdout.strip()
    except Exception:
        pass

    OUT_DIR.mkdir(parents=True, exist_ok=True)
    if DB_PATH.exists():
        DB_PATH.unlink()

    graph = load_graph()
    node_kinds = Counter(n.get("kind") for n in graph["nodes"])
    edge_kinds = Counter(e.get("kind") for e in graph["links"])
    print(f"Graph: {len(graph['nodes'])} nodes {dict(node_kinds)}, {len(graph['links'])} edges {dict(edge_kinds)}", file=sys.stderr)

    conn = sqlite3.connect(str(DB_PATH))
    conn.execute("PRAGMA journal_mode = OFF")
    conn.execute("PRAGMA synchronous = OFF")
    conn.execute("PRAGMA cache_size = 1000000")

    init_schema(conn)
    insert_meta(conn, commit)
    insert_run(conn, {}, commit)
    insert_source(conn)
    doc_map = insert_documents(conn, graph)
    insert_chunks_for_sample(conn, graph, doc_map)
    insert_nodes(conn, graph)
    insert_edges(conn, graph)
    snapshot_id = insert_snapshot(conn, graph)
    insert_understanding(conn, snapshot_id, graph, commit)
    insert_drill_and_events(conn)

    conn.execute("PRAGMA optimize")
    conn.close()

    size_mb = DB_PATH.stat().st_size / 1024 / 1024
    print(f"Wrote {DB_PATH} ({size_mb:.1f} MB)", file=sys.stderr)


if __name__ == "__main__":
    main()
