import json
with open('tree.json', 'r') as f:
    tree = json.load(f)

def count_all_leaves(node):
    count = 0
    t = node.get("type")
    nodes = node.get("nodes", [])
    floating = node.get("floating_nodes", [])
    
    has_children = bool(nodes or floating)
    if not has_children and t in ("con", "floating_con"):
        if node.get("app_id") or node.get("window") or node.get("name"):
            return 1
    
    for c in nodes: count += count_all_leaves(c)
    for c in floating: count += count_all_leaves(c)
    return count

for output in tree.get("nodes", []):
    for ws in output.get("nodes", []):
        if ws.get("type") == "workspace":
            print(f"WS {ws.get('name')}: {count_all_leaves(ws)} windows")
