import json

def transform_to_okf_text(raw_json: str) -> str:
    """[DO-178C] Tracing: HLR-02 (OKF Pipeline)
    Converts raw JSON tree to semantic, token-efficient text.
    """
    try:
        data = json.loads(raw_json)
        nodes = []
        _walk(data, nodes)
        return "\n".join(nodes)
    except:
        return ""

def _walk(node: dict, out: list, depth: int = 0, node_id_ref: list = [0]):
    if not node: return
    tag = node.get("tag", "div")
    text = node.get("text", "")
    bounds = node.get("bounds", [0, 0, 0, 0])
    interactable = node.get("interactable", False)
    
    ref_id = ""
    if interactable:
        node_id_ref[0] += 1
        ref_id = f"[@e{node_id_ref[0]}] "

    if text or interactable:
        attrs = f'bounds="{bounds}"'
        if interactable: attrs += ' interactable="true"'
        out.append(f"{'  ' * depth}{ref_id}<{tag} {attrs}>{text}")

    for child in node.get("children", []):
        _walk(child, out, depth + 1, node_id_ref)
