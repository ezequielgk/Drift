import json

with open('tree.json', 'r') as f:
    tree = json.load(f)

def find_ws_of_focused(node, current_ws=None):
    if node.get("type") == "workspace":
        current_ws = node
    if node.get("focused"):
        return current_ws
    for child in node.get("nodes", []) + node.get("floating_nodes", []):
        res = find_ws_of_focused(child, current_ws)
        if res: return res
    return None

def count_leaves(node):
    count = 0
    t = node.get("type")
    nodes = node.get("nodes", [])
    floating = node.get("floating_nodes", [])
    
    if not nodes and not floating:
        if t in ("con", "floating_con"):
            if node.get("app_id") or node.get("window") or node.get("name"):
                return 1
    
    for c in nodes: count += count_leaves(c)
    for c in floating: count += count_leaves(c)
    return count

ws = find_ws_of_focused(tree)
print(f"Workspace: {ws.get('name')}")
print(f"Leaves: {count_leaves(ws)}")
