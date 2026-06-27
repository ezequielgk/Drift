use std::io::{Read, Write};
use std::os::unix::net::UnixStream;

use crate::Commands;
use drift_core::DriftError;

const DAEMON_SOCKET: &str = "/tmp/drift.sock";

pub fn run_ctl(cmd: Commands) -> Result<(), DriftError> {
    let mut stream =
        UnixStream::connect(DAEMON_SOCKET).map_err(|_| DriftError::DaemonNotRunning)?;

    let command_str = match cmd {
        Commands::Toggle => "toggle".to_string(),
        Commands::Activate => "activate".to_string(),
        Commands::Deactivate => "deactivate".to_string(),
        Commands::Status => "status".to_string(),
        Commands::Next => "next".to_string(),
        Commands::Prev => "prev".to_string(),
        Commands::MoveNext => "move-next".to_string(),
        Commands::MovePrev => "move-prev".to_string(),
        Commands::Back => "back".to_string(),
        Commands::Config { command } => match command {
            crate::ConfigCommand::Get { key } => format!("config get {}", key),
            crate::ConfigCommand::Set { key, value } => format!("config set {} {}", key, value),
        },
        Commands::Completions { .. } => unreachable!(),
    };

    stream
        .write_all(command_str.as_bytes())
        .map_err(DriftError::StateIo)?;
    stream
        .shutdown(std::net::Shutdown::Write)
        .map_err(DriftError::StateIo)?;

    let mut response = String::new();
    stream
        .read_to_string(&mut response)
        .map_err(DriftError::StateIo)?;

    let response = response.trim();
    if response.starts_with("error: ") {
        eprintln!("{}", response);
        std::process::exit(1);
    } else if !response.is_empty() && response != "ok" {
        println!("{}", response);
    }

    Ok(())
}
