use std::fmt;

pub mod actions;
pub mod config;
pub mod ipc;
pub mod state;

#[derive(Debug)]
pub enum DriftError {
    SocketNotFound,
    IpcConnect(std::io::Error),
    IpcRead(std::io::Error),
    IpcWrite(std::io::Error),
    InvalidResponse(String),
    StateIo(std::io::Error),
    ConfigIo(std::io::Error),
    ConfigParse(String),
    DaemonNotRunning,
    DaemonAlreadyRunning,
}

impl fmt::Display for DriftError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::SocketNotFound => write!(f, "Sway IPC socket not found"),
            Self::IpcConnect(e) => write!(f, "Failed to connect to Sway IPC socket: {}", e),
            Self::IpcRead(e) => write!(f, "Failed to read from Sway IPC socket: {}", e),
            Self::IpcWrite(e) => write!(f, "Failed to write to Sway IPC socket: {}", e),
            Self::InvalidResponse(msg) => write!(f, "Invalid Sway IPC response: {}", msg),
            Self::StateIo(e) => write!(f, "State I/O error: {}", e),
            Self::ConfigIo(e) => write!(f, "Config I/O error: {}", e),
            Self::ConfigParse(msg) => write!(f, "Config parse error: {}", msg),
            Self::DaemonNotRunning => write!(f, "Daemon is not running"),
            Self::DaemonAlreadyRunning => write!(f, "Daemon is already running"),
        }
    }
}

impl std::error::Error for DriftError {}
