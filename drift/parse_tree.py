import json

with open('tree.json', 'r') as f:
    tree = json.load(f)
    
def find_focused_ws(node):
    if node.get("type") == "workspace" and node.get("focused"):
        return node
    for child in node.get("nodes", []):
        res = find_focused_ws(child)
        if res: return res
    for child in node.get("floating_nodes", []):
        res = find_focused_ws(child)
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

focused_ws = None

def find_ws(node, name):
    if node.get("type") == "workspace" and node.get("name") == name: return node
    for child in node.get("nodes", []):
        res = find_ws(child, name)
        if res: return res
    return None

# Find focused workspace name (simulating GET_WORKSPACES)
def get_focused_ws_name(node):
    if node.get("type") == "workspace":
        # check if it or its children are focused
        if node.get("focused"): return node.get("name")
    if node.get("focused"):
        return get_focused_ws_name_from_tree(tree, node.get("id"))
    for child in node.get("nodes", []) + node.get("floating_nodes", []):
        res = get_focused_ws_name(child)
        if res: return res
    return None

def find_focused_node(node):
    if node.get("focused"): return node
    for child in node.get("nodes", []) + node.get("floating_nodes", []):
        res = find_focused_node(child)
        if res: return res
    return None

focused_node = find_focused_node(tree)
print(f"Focused node name: {focused_node.get('name')} type: {focused_node.get('type')}")

