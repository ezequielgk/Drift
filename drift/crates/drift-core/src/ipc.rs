use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::DriftError;

const IPC_MAGIC: &[u8; 6] = b"i3-ipc";

pub enum IpcCommandType {
    RunCommand = 0,
}

pub struct IpcClient {
    stream: UnixStream,
}

impl IpcClient {
    pub fn connect(socket_path: &str) -> Result<Self, DriftError> {
        let stream = UnixStream::connect(socket_path).map_err(DriftError::IpcConnect)?;
        Ok(Self { stream })
    }

    pub fn send(&mut self, payload: &str, cmd_type: IpcCommandType) -> Result<(), DriftError> {
        let payload_bytes = payload.as_bytes();
        let payload_len = payload_bytes.len() as u32;

        let mut header = Vec::with_capacity(14);
        header.extend_from_slice(IPC_MAGIC);
        header.extend_from_slice(&payload_len.to_ne_bytes());
        header.extend_from_slice(&(cmd_type as u32).to_ne_bytes());

        self.stream
            .write_all(&header)
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(payload_bytes)
            .map_err(DriftError::IpcWrite)?;

        self.read_response()?;
        Ok(())
    }

    fn read_response_raw(&mut self) -> Result<(u32, Vec<u8>), DriftError> {
        let mut magic_buf = [0u8; 6];
        self.stream
            .read_exact(&mut magic_buf)
            .map_err(DriftError::IpcRead)?;

        if &magic_buf != IPC_MAGIC {
            return Err(DriftError::InvalidResponse(
                "Invalid magic string".to_string(),
            ));
        }

        let mut len_buf = [0u8; 4];
        self.stream
            .read_exact(&mut len_buf)
            .map_err(DriftError::IpcRead)?;
        let len = u32::from_ne_bytes(len_buf);

        let mut type_buf = [0u8; 4];
        self.stream
            .read_exact(&mut type_buf)
            .map_err(DriftError::IpcRead)?;
        let msg_type = u32::from_ne_bytes(type_buf);

        let mut payload_buf = vec![0u8; len as usize];
        self.stream
            .read_exact(&mut payload_buf)
            .map_err(DriftError::IpcRead)?;

        Ok((msg_type, payload_buf))
    }

    fn read_response(&mut self) -> Result<String, DriftError> {
        let (_, payload_buf) = self.read_response_raw()?;
        String::from_utf8(payload_buf).map_err(|e| DriftError::InvalidResponse(e.to_string()))
    }

    pub fn subscribe_window(&mut self) -> Result<(), DriftError> {
        let payload = r#"["window"]"#;
        let payload_bytes = payload.as_bytes();
        let payload_len = payload_bytes.len() as u32;

        let mut header = Vec::with_capacity(14);
        header.extend_from_slice(IPC_MAGIC);
        header.extend_from_slice(&payload_len.to_ne_bytes());
        header.extend_from_slice(&(2u32).to_ne_bytes()); // Type 2 = SUBSCRIBE

        self.stream
            .write_all(&header)
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(payload_bytes)
            .map_err(DriftError::IpcWrite)?;

        self.read_response()?;
        Ok(())
    }

    pub fn read_event(&mut self) -> Result<SwayEvent, DriftError> {
        loop {
            let (msg_type, payload_buf) = self.read_response_raw()?;
            if msg_type == 0x80000002 {
                let payload = String::from_utf8(payload_buf)
                    .map_err(|e| DriftError::InvalidResponse(e.to_string()))?;
                let event: SwayEvent = serde_json::from_str(&payload)
                    .map_err(|e| DriftError::InvalidResponse(e.to_string()))?;
                return Ok(event);
            }
        }
    }

    pub fn focused_workspace_window_count(&mut self) -> Result<u32, DriftError> {
        // 1. Get focused workspace name
        self.stream
            .write_all(IPC_MAGIC)
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(&0u32.to_ne_bytes())
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(&(1u32).to_ne_bytes())
            .map_err(DriftError::IpcWrite)?; // GET_WORKSPACES

        let ws_resp = self.read_response()?;
        let workspaces: Vec<serde_json::Value> = serde_json::from_str(&ws_resp)
            .map_err(|e| DriftError::InvalidResponse(e.to_string()))?;

        let focused_ws = workspaces
            .iter()
            .find(|w| w.get("focused").and_then(|f| f.as_bool()).unwrap_or(false))
            .and_then(|w| w.get("name"))
            .and_then(|n| n.as_str());

        let ws_name = match focused_ws {
            Some(n) => n.to_string(),
            None => return Ok(0),
        };

        // 2. Get tree
        self.stream
            .write_all(IPC_MAGIC)
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(&0u32.to_ne_bytes())
            .map_err(DriftError::IpcWrite)?;
        self.stream
            .write_all(&(4u32).to_ne_bytes())
            .map_err(DriftError::IpcWrite)?; // GET_TREE

        let tree_resp = self.read_response()?;
        let tree: serde_json::Value = serde_json::from_str(&tree_resp)
            .map_err(|e| DriftError::InvalidResponse(e.to_string()))?;

        // 3. Find workspace node and count
        let mut count = 0;
        if let Some(ws_node) = find_workspace_node(&tree, &ws_name) {
            count = count_leaves(ws_node);
        }

        Ok(count)
    }
}

fn find_workspace_node<'a>(
    node: &'a serde_json::Value,
    ws_name: &str,
) -> Option<&'a serde_json::Value> {
    if node.get("type").and_then(|t| t.as_str()) == Some("workspace")
        && node.get("name").and_then(|n| n.as_str()) == Some(ws_name)
    {
        return Some(node);
    }

    if let Some(nodes) = node.get("nodes").and_then(|n| n.as_array()) {
        for child in nodes {
            if let Some(found) = find_workspace_node(child, ws_name) {
                return Some(found);
            }
        }
    }

    None
}

fn count_leaves(node: &serde_json::Value) -> u32 {
    let mut count = 0;

    let t = node.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let nodes = node.get("nodes").and_then(|n| n.as_array());
    let floating = node.get("floating_nodes").and_then(|n| n.as_array());

    let has_children = (nodes.is_some() && !nodes.unwrap().is_empty())
        || (floating.is_some() && !floating.unwrap().is_empty());

    if !has_children && (t == "con" || t == "floating_con") {
        return 1;
    }

    if let Some(n) = nodes {
        for child in n {
            count += count_leaves(child);
        }
    }

    if let Some(f) = floating {
        for child in f {
            count += count_leaves(child);
        }
    }

    count
}

#[derive(serde::Deserialize)]
pub struct SwayEvent {
    pub change: String,
}
