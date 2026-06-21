"""Tests for repo_to_lgwks_db chunking strategy."""

import sys
from pathlib import Path

sys.path.insert(0, str(Path(__file__).parent.parent / "scripts"))

from repo_to_lgwks_db import CHUNK_OVERLAP_CHARS, CHUNK_SIZE_CHARS, chunk_text


def test_chunk_text_preserves_path_context():
    chunks = chunk_text("def foo():\n    pass\n", path="src/foo.py")
    assert len(chunks) == 1
    assert chunks[0].startswith("[src/foo.py]\n")


def test_chunk_text_respects_size_budget():
    big = "\n".join([f"line {i:05d}" for i in range(500)])
    chunks = chunk_text(big, path="src/big.py")
    assert all(len(c) <= CHUNK_SIZE_CHARS for c in chunks)


def test_chunk_text_has_overlap_continuity():
    big = "\n".join([f"line {i:05d}" for i in range(500)])
    chunks = chunk_text(big, path="src/big.py")
    assert len(chunks) > 1
    shared = 0
    for i in range(len(chunks) - 1):
        prev = chunks[i].replace("[src/big.py]\n", "")[-CHUNK_OVERLAP_CHARS:]
        nxt = chunks[i + 1].replace("[src/big.py]\n", "")[:CHUNK_OVERLAP_CHARS]
        if prev and nxt and (prev[:20] in nxt or nxt[:20] in prev):
            shared += 1
    assert shared == len(chunks) - 1


def test_chunk_text_does_not_split_single_small_file():
    text = "def foo():\n    return 1\n"
    chunks = chunk_text(text, path="src/tiny.py")
    assert len(chunks) == 1


def test_chunk_text_returns_empty_for_whitespace_only():
    assert chunk_text("   \n\n  ", path="src/empty.py") == []


def test_chunk_text_preserves_class_boundary():
    text = "\n\n".join([
        "def foo():\n    return 1",
        "class Bar:\n    def a(self): pass\n    def b(self): pass",
        "def baz():\n    return 2",
    ])
    chunks = chunk_text(text, path="src/boundaries.py")
    assert all(c.startswith("[src/boundaries.py]\n") for c in chunks)
    # The class boundary should be preserved as a content line start.
    content_lines = [line for c in chunks for line in c.replace("[src/boundaries.py]\n", "").split("\n")]
    assert any(line.startswith("class Bar:") for line in content_lines)
