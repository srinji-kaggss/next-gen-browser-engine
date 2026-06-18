"""OKF renderer: derive a human-readable text lens from a Braid anchor.

Traceability: AXIOM_DERIVED_LENS, HLR-02 (OKF Pipeline).

Input: a JSON array of WebObservation records produced by the Rust engine.
Each record carries a content-addressed `target_cid` and a list of typed facts.

Output: a compact text manifest with stable human references (`@eN`) assigned
by sorting interactable elements by CID, not by DOM tree walk order.
"""

import json
import sys
from typing import Any


def transform_to_okf_text(raw_json: str) -> str:
    """Convert a Braid anchor JSON blob into an OKF text manifest."""
    try:
        records = json.loads(raw_json)
        if not isinstance(records, list):
            return ""
        return render_okf(records)
    except json.JSONDecodeError:
        return ""


def _fact_dict(facts: list[list[str]]) -> dict[str, str]:
    return {pred: obj for pred, obj in facts}


def _is_interactable(fact_dict: dict[str, str]) -> bool:
    return fact_dict.get("interactable") == "true"


def _short_ref(cid: str) -> str:
    return cid[:8]


def render_okf(records: list[dict[str, Any]]) -> str:
    """Render a Braid observation list as an OKF text manifest.

    Reference IDs are assigned to interactable elements by sorting on CID,
    so the same page rendered twice produces the same reference map.
    """
    lines: list[str] = []

    # Collect interactable elements and assign deterministic human refs.
    interactables = [
        r for r in records
        if r.get("kind") == "element" and _is_interactable(_fact_dict(r.get("facts", [])))
    ]
    interactables.sort(key=lambda r: r.get("target_cid", ""))
    ref_by_cid = {
        r["target_cid"]: f"@e{idx + 1}"
        for idx, r in enumerate(interactables)
    }

    for record in records:
        kind = record.get("kind", "unknown")
        cid = record.get("target_cid", "")
        facts = _fact_dict(record.get("facts", []))

        if kind == "load":
            url = facts.get("url", "")
            title = facts.get("title", "")
            lines.append(f"[load] {url} | {title} | cid={_short_ref(cid)}")
            continue

        if kind != "element":
            continue

        tag = facts.get("tag", "div")
        text = facts.get("text", "").strip()
        bounds = facts.get("bounds", "")
        role = facts.get("role", "")
        element_id = facts.get("id", "")

        attrs = []
        if bounds:
            attrs.append(f'bounds="{bounds}"')
        if role:
            attrs.append(f'role="{role}"')
        if element_id:
            attrs.append(f'id="{element_id}"')
        if _is_interactable(facts):
            attrs.append('interactable="true"')

        ref = ""
        if cid in ref_by_cid:
            ref = f"{ref_by_cid[cid]} "

        attr_str = ""
        if attrs:
            attr_str = " " + " ".join(attrs)

        if text or ref:
            lines.append(f"{ref}<{tag}{attr_str}>{text}")

    return "\n".join(lines)


if __name__ == "__main__":
    data = sys.stdin.read()
    print(transform_to_okf_text(data))
