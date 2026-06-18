#!/usr/bin/env python3
"""Build a deterministic, lgwks-compatible graph of a git source tree.

Usage:
  python3 build_repo_graph.py NAME REPO_ROOT OUT_DIR [SOURCE_URL] [COMMIT]

Inputs (pre-generated in the same directory as this script):
  {NAME}_full_tree.txt    - git ls-tree -r HEAD output
  {NAME}_includes_raw.txt - git grep '#include' output for C/C++ files
  {NAME}_gn_imports_raw.txt - git grep 'import("' output for GN files
  {NAME}_lgwks_graph.json   - lgwks repo graph output (optional)

Output:
  OUT_DIR/graph.json
  OUT_DIR/manifest.jsonl
  OUT_DIR/meta.json
  OUT_DIR/summary.json
"""

import argparse
import json
import os
import re
import sys
from collections import Counter
from datetime import datetime, timezone
from pathlib import Path


def unquote_git_path(raw: str) -> str:
    if raw.startswith('"') and raw.endswith('"'):
        raw = raw[1:-1]
        octal_re = re.compile(r"\\([0-7]{1,3})")
        raw = octal_re.sub(lambda m: chr(int(m.group(1), 8)), raw)
        raw = raw.replace('\\"', '"').replace("\\\\", "\\")
    return raw


def main() -> None:
    parser = argparse.ArgumentParser()
    parser.add_argument("name", help="run/project name, also prefix for raw input files")
    parser.add_argument("repo_root", type=Path, help="path to the cloned source tree")
    parser.add_argument("out_dir", type=Path, help="output directory for graph artifacts")
    parser.add_argument("--source-url", default="", help="canonical source URL")
    parser.add_argument("--commit", default="", help="git commit SHA")
    parser.add_argument("--base-dir", type=Path, default=Path(__file__).parent, help="directory holding *_full_tree.txt inputs")
    parser.add_argument("--lgwks-graph", type=Path, default=None, help="path to lgwks repo graph JSON (optional)")
    parser.add_argument("--symbol-limit", type=int, default=8600, help="max public-header symbols to extract")
    parser.add_argument("--max-symbol-headers", type=int, default=20000, help="max headers to scan for symbols")
    args = parser.parse_args()

    name = args.name
    repo_root = args.repo_root.resolve()
    out_dir = args.out_dir
    base_dir = args.base_dir
    source_url = args.source_url or f"file://{repo_root}"
    commit = args.commit or "unknown"

    if not commit or commit == "unknown":
        try:
            import subprocess
            commit = subprocess.run(
                ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
                capture_output=True, text=True, check=True, timeout=30
            ).stdout.strip()
        except Exception:
            pass

    tree_file = base_dir / f"{name}_full_tree.txt"
    includes_file = base_dir / f"{name}_includes_raw.txt"
    gn_file = base_dir / f"{name}_gn_imports_raw.txt"
    lg_graph_file = args.lgwks_graph or base_dir / f"{name}_lgwks_graph.json"

    if not tree_file.exists():
        print(f"Missing {tree_file}", file=sys.stderr)
        sys.exit(1)

    out_dir.mkdir(parents=True, exist_ok=True)

    # -----------------------------------------------------------------------
    # 1. Read full tree manifest
    # -----------------------------------------------------------------------
    entries = []
    paths = set()
    with tree_file.open("r", encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            if not line:
                continue
            parts = line.split(" ", 4)
            if len(parts) != 5:
                continue
            mode, objtype, sha, size, raw_path = parts
            path = unquote_git_path(raw_path)
            entries.append({
                "path": path,
                "type": objtype,
                "mode": mode,
                "sha": sha,
                "size": int(size) if size.isdigit() else 0,
            })
            paths.add(path)

    # -----------------------------------------------------------------------
    # 2. Build directory list
    # -----------------------------------------------------------------------
    dirs = set()
    for e in entries:
        p = Path(e["path"]).parent
        while str(p) != ".":
            dirs.add(str(p))
            p = p.parent
        dirs.add(str(p))

    # -----------------------------------------------------------------------
    # 3. Emit manifest JSONL
    # -----------------------------------------------------------------------
    manifest_path = out_dir / "manifest.jsonl"
    with manifest_path.open("w", encoding="utf-8") as out:
        for e in entries:
            out.write(json.dumps(e, separators=(",", ":")) + "\n")

    # -----------------------------------------------------------------------
    # 4. Build nodes
    # -----------------------------------------------------------------------
    nodes = []

    def add_node(nid, label, kind, meta=None):
        node = {"id": nid, "label": label, "kind": kind}
        if meta:
            node.update(meta)
        nodes.append(node)

    for d in sorted(dirs):
        add_node(f"dir:{d}", d, "dir")

    for e in entries:
        path = e["path"]
        ext = Path(path).suffix.lower() or "(none)"
        add_node(f"file:{path}", path, "file", {
            "file_type": ext,
            "sha256": e["sha"],
            "size": e["size"],
            "mode": e["mode"],
        })

    # -----------------------------------------------------------------------
    # 5. Build containment edges
    # -----------------------------------------------------------------------
    edges = []
    seen_edges = set()

    def add_edge(src, tgt, kind, meta=None):
        key = (src, tgt, kind)
        if key in seen_edges:
            return
        seen_edges.add(key)
        edge = {"source": src, "target": tgt, "kind": kind}
        if meta:
            edge.update(meta)
        edges.append(edge)

    for e in entries:
        path = e["path"]
        parent = str(Path(path).parent)
        add_edge(f"dir:{parent}", f"file:{path}", "contains")

    for d in dirs:
        parent = str(Path(d).parent)
        if parent != d:
            add_edge(f"dir:{parent}", f"dir:{d}", "contains")

    # -----------------------------------------------------------------------
    # 6. Resolve include paths
    # -----------------------------------------------------------------------
    include_re = re.compile(r'^\s*#include\s+["<]([^">]+)[">]')

    def resolve_include(from_path, include_str):
        candidate = (repo_root / Path(from_path).parent / include_str).resolve()
        try:
            rel = str(candidate.relative_to(repo_root))
            if rel in paths:
                return rel
        except ValueError:
            pass
        if include_str in paths:
            return include_str
        parts = Path(include_str).parts
        for i in range(len(parts)):
            tail = "/".join(parts[i:])
            if tail in paths:
                return tail
        return None

    include_resolved = 0
    include_dangling = 0
    if includes_file.exists():
        with includes_file.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.rstrip("\n")
                if ":" not in line:
                    continue
                file_part, rest = line.split(":", 1)
                m = re.match(r"^(\d+):(.*)$", rest)
                if not m:
                    continue
                lineno, content = m.groups()
                inc_match = include_re.search(content)
                if not inc_match:
                    continue
                include_str = inc_match.group(1)
                resolved = resolve_include(file_part, include_str)
                if resolved:
                    add_edge(f"file:{file_part}", f"file:{resolved}", "includes",
                             {"include_path": include_str, "line": int(lineno)})
                    include_resolved += 1
                else:
                    include_dangling += 1

    # -----------------------------------------------------------------------
    # 7. Resolve GN imports
    # -----------------------------------------------------------------------
    gn_import_re = re.compile(r'^import\("//([^"]+)"\)')
    gn_resolved = 0
    gn_dangling = 0
    if gn_file.exists():
        with gn_file.open("r", encoding="utf-8") as f:
            for line in f:
                line = line.rstrip("\n")
                if ":" not in line:
                    continue
                file_part, rest = line.split(":", 1)
                m = re.match(r"^(\d+):(.*)$", rest)
                if not m:
                    continue
                lineno, content = m.groups()
                gmatch = gn_import_re.search(content)
                if not gmatch:
                    continue
                target = gmatch.group(1)
                if target in paths:
                    add_edge(f"file:{file_part}", f"file:{target}", "gn_import", {"line": int(lineno)})
                    gn_resolved += 1
                else:
                    gn_dangling += 1

    # -----------------------------------------------------------------------
    # 8. Add lgwks repo graph edges (optional)
    # -----------------------------------------------------------------------
    lg_imports = 0
    lg_calls = 0
    if lg_graph_file.exists():
        with lg_graph_file.open("r", encoding="utf-8") as f:
            lg = json.load(f)
        for e in lg.get("edges", []):
            src = e.get("from")
            tgt = e.get("to")
            kind = e.get("type")
            if not src or not tgt or not kind:
                continue
            add_node(f"file:{src}", src, "file")
            add_node(f"file:{tgt}", tgt, "file")
            add_edge(f"file:{src}", f"file:{tgt}", kind)
            if kind == "import":
                lg_imports += 1
            elif kind == "call":
                lg_calls += 1

    # -----------------------------------------------------------------------
    # 9. Best-effort symbol extraction
    # -----------------------------------------------------------------------
    symbol_re = re.compile(
        r"(?:^|\n)\s*(?:class|struct|enum\s+class|enum|namespace)\s+([A-Za-z_][A-Za-z0-9_:]*)")

    public_header_patterns = [
        re.compile(rf"^{re.escape(name)}/[^/]+\.h$") if name else None,
        re.compile(r"^Source/"),
        re.compile(r"^WebCore/"),
        re.compile(r"^JavaScriptCore/"),
        re.compile(r"^WebKit/"),
        re.compile(r"^WebKitLegacy/"),
        re.compile(r"^WTF/"),
        re.compile(r"^platform/"),
        re.compile(r"^content/public/"),
        re.compile(r"^chrome/[^/]+\.h$"),
        re.compile(r"^net/[^/]+\.h$"),
        re.compile(r"^ui/[^/]+\.h$"),
        re.compile(r"^device/[^/]+\.h$"),
        re.compile(r"^mojo/public/"),
        re.compile(r"^sandbox/[^/]+\.h$"),
        re.compile(r"^extensions/[^/]+\.h$"),
        re.compile(r"^components/[^/]+/public/"),
        re.compile(r"^third_party/blink/public/"),
        re.compile(r"^base/[^/]+\.h$"),
        re.compile(r"^dom/[^/]+\.h$"),
        re.compile(r"^layout/[^/]+\.h$"),
        re.compile(r"^gfx/[^/]+\.h$"),
        re.compile(r"^ipc/[^/]+\.h$"),
        re.compile(r"^netwerk/"),
        re.compile(r"^docshell/"),
        re.compile(r"^widget/"),
        re.compile(r"^modules/[^/]+\.h$"),
    ]
    public_header_patterns = [p for p in public_header_patterns if p is not None]

    symbol_count = 0
    scanned = 0
    for e in entries:
        path = e["path"]
        if not path.endswith(".h"):
            continue
        if not any(p.search(path) for p in public_header_patterns):
            continue
        if scanned >= args.max_symbol_headers:
            break
        scanned += 1
        fpath = repo_root / path
        try:
            text = fpath.read_text(encoding="utf-8", errors="ignore")
        except Exception:
            continue
        for m in symbol_re.finditer(text):
            name = m.group(1).split("::")[0].split("<")[0].strip()
            if not name:
                continue
            sid = f"symbol:{path}:{name}:L{m.start(1)}"
            add_node(sid, name, "symbol", {"source_file": path, "source_location": f"L{m.start(1)}"})
            add_edge(f"file:{path}", sid, "defines")
            symbol_count += 1

    # -----------------------------------------------------------------------
    # 10. Write graph.json
    # -----------------------------------------------------------------------
    graph = {
        "directed": True,
        "multigraph": False,
        "graph": {
            "schema": "lgwks.graph.networkx.v0",
            "repo": str(repo_root),
            "source_url": source_url,
            "commit": commit,
            "generated_at": datetime.now(timezone.utc).isoformat(),
            "extraction_method": "git tree + git grep includes/gn + lgwks repo graph",
        },
        "nodes": nodes,
        "links": edges,
    }

    graph_path = out_dir / "graph.json"
    with graph_path.open("w", encoding="utf-8") as f:
        json.dump(graph, f, separators=(",", ":"))

    # -----------------------------------------------------------------------
    # 11. Write meta.json
    # -----------------------------------------------------------------------
    meta = {
        "repo": str(repo_root),
        "commit": commit,
        "source_url": source_url,
        "generated_at": datetime.now(timezone.utc).isoformat(),
        "input_files": {
            "tree_manifest": str(tree_file),
            "includes_raw": str(includes_file),
            "gn_imports_raw": str(gn_file),
            "lgwks_repo_graph": str(lg_graph_file),
        },
        "counts": {
            "tracked_entries": len(entries),
            "directories": len(dirs),
            "file_nodes": sum(1 for n in nodes if n["kind"] == "file"),
            "dir_nodes": sum(1 for n in nodes if n["kind"] == "dir"),
            "symbol_nodes": sum(1 for n in nodes if n["kind"] == "symbol"),
            "contains_edges": sum(1 for e in edges if e["kind"] == "contains"),
            "includes_edges": include_resolved,
            "includes_dangling": include_dangling,
            "gn_import_edges": gn_resolved,
            "gn_import_dangling": gn_dangling,
            "lgwks_import_edges": lg_imports,
            "lgwks_call_edges": lg_calls,
            "total_edges": len(edges),
        },
        "notes": [
            "C/C++ include resolution is best-effort relative to including file or repo root; unresolved includes are counted as dangling.",
            "GN imports resolved only when the target .gn/.gni file exists in the checkout.",
            "Symbol extraction is limited to public-looking .h files (class/struct/enum/namespace declarations).",
        ],
    }
    if lg_imports or lg_calls:
        meta["notes"].append("lgwks repo graph contributed .py/.rs/.json/.yaml relationships.")

    meta_path = out_dir / "meta.json"
    with meta_path.open("w", encoding="utf-8") as f:
        json.dump(meta, f, indent=2)

    # -----------------------------------------------------------------------
    # 12. Write summary.json
    # -----------------------------------------------------------------------
    child_counts = Counter()
    for e in edges:
        if e["kind"] == "contains" and e["source"].startswith("dir:"):
            child_counts[e["source"]] += 1
    top_dirs = []
    for n in nodes:
        if n["kind"] == "dir" and n["label"].count("/") == 0 and n["label"] != ".":
            top_dirs.append((n["label"], child_counts.get(n["id"], 0)))
    top_dirs.sort(key=lambda x: -x[1])

    incoming_files = Counter()
    for e in edges:
        if e["kind"] == "includes":
            incoming_files[e["target"]] += 1
    top_include_targets = []
    for tgt, cnt in incoming_files.most_common(20):
        top_include_targets.append((tgt[5:] if tgt.startswith("file:") else tgt, cnt))

    summary = {
        "top_level_dirs_by_direct_children": top_dirs[:30],
        "top_include_targets": top_include_targets,
    }
    (out_dir / "summary.json").write_text(json.dumps(summary, indent=2), encoding="utf-8")

    print(json.dumps(meta["counts"], indent=2))
    print(f"\nWrote {graph_path} ({graph_path.stat().st_size / 1024 / 1024:.1f} MB)")
    print(f"Wrote {manifest_path} ({manifest_path.stat().st_size / 1024 / 1024:.1f} MB)")
    print(f"Wrote {meta_path}")


if __name__ == "__main__":
    main()
