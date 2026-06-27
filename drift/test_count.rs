use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

fn main() {
    let mut stream = UnixStream::connect(std::env::var("SWAYSOCK").unwrap()).unwrap();
    let magic = b"i3-ipc";
    
    // GET_WORKSPACES
    stream.write_all(magic).unwrap();
    stream.write_all(&0u32.to_ne_bytes()).unwrap();
    stream.write_all(&1u32.to_ne_bytes()).unwrap();
    
    let mut header = [0u8; 14];
    stream.read_exact(&mut header).unwrap();
    let len = u32::from_ne_bytes([header[6], header[7], header[8], header[9]]);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).unwrap();
    
    let ws_json: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    let focused = ws_json.as_array().unwrap().iter().find(|w| w["focused"].as_bool() == Some(true)).unwrap();
    let name = focused["name"].as_str().unwrap();
    println!("Focused workspace: {}", name);
    
    // GET_TREE
    stream.write_all(magic).unwrap();
    stream.write_all(&0u32.to_ne_bytes()).unwrap();
    stream.write_all(&4u32.to_ne_bytes()).unwrap();
    
    stream.read_exact(&mut header).unwrap();
    let len = u32::from_ne_bytes([header[6], header[7], header[8], header[9]]);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).unwrap();
    
    let tree: serde_json::Value = serde_json::from_slice(&buf).unwrap();
    
    fn find_ws<'a>(node: &'a serde_json::Value, name: &str) -> Option<&'a serde_json::Value> {
        if node["type"] == "workspace" && node["name"] == name { return Some(node); }
        if let Some(nodes) = node["nodes"].as_array() {
            for child in nodes {
                if let Some(f) = find_ws(child, name) { return Some(f); }
            }
        }
        None
    }
    
    let ws_node = find_ws(&tree, name).unwrap();
    
    fn count_leaves(node: &serde_json::Value) -> u32 {
        let mut count = 0;
        let t = node["type"].as_str().unwrap_or("");
        let nodes = node["nodes"].as_array().unwrap();
        let floating = node["floating_nodes"].as_array().unwrap();
        
        let has_children = !nodes.is_empty() || !floating.is_empty();
        if !has_children && (t == "con" || t == "floating_con") { return 1; }
        
        for c in nodes { count += count_leaves(c); }
        for c in floating { count += count_leaves(c); }
        count
    }
    
    println!("Count: {}", count_leaves(ws_node));
}
