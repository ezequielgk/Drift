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

    fn read_response(&mut self) -> Result<String, DriftError> {
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
        let _type = u32::from_ne_bytes(type_buf);

        let mut payload_buf = vec![0u8; len as usize];
        self.stream
            .read_exact(&mut payload_buf)
            .map_err(DriftError::IpcRead)?;

        String::from_utf8(payload_buf).map_err(|e| DriftError::InvalidResponse(e.to_string()))
    }
}
