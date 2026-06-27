use std::env;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};
use drift_core::DriftError;

pub mod ctl;
pub mod daemon;

#[derive(clap::ValueEnum, Clone)]
pub enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Parser)]
#[command(name = "driftd")]
#[command(about = "Daemon for drift workspace layout manager", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Override $SWAYSOCK
    #[arg(long)]
    socket: Option<String>,
}

#[derive(Subcommand, Clone)]
pub enum Commands {
    /// Toggle active/inactive state
    Toggle,
    /// Print "active" or "inactive" to stdout
    Status,
    /// Force activate
    Activate,
    /// Force deactivate
    Deactivate,

    /// Focus next workspace on output
    Next,
    /// Focus previous workspace on output
    Prev,
    /// Move container to next workspace and follow
    MoveNext,
    /// Move container to prev workspace and follow
    MovePrev,
    /// Toggle between last two workspaces
    Back,

    /// Generate shell completions
    Completions {
        #[arg(value_enum)]
        shell: Shell,
    },
}

fn get_sway_socket(cli_socket: Option<String>) -> Result<String, DriftError> {
    if let Some(s) = cli_socket {
        return Ok(s);
    }
    env::var("SWAYSOCK").map_err(|_| DriftError::SocketNotFound)
}

fn run() -> Result<(), DriftError> {
    let args: Vec<String> = env::args().collect();
    let program_name = std::path::Path::new(&args[0])
        .file_name()
        .unwrap_or_default()
        .to_str()
        .unwrap_or_default();

    let cli = Cli::parse();

    if let Some(Commands::Completions { shell }) = cli.command {
        let mut cmd = Cli::command();
        match shell {
            Shell::Bash => generate(Bash, &mut cmd, "driftd", &mut std::io::stdout()),
            Shell::Zsh => generate(Zsh, &mut cmd, "driftd", &mut std::io::stdout()),
            Shell::Fish => generate(Fish, &mut cmd, "driftd", &mut std::io::stdout()),
        }
        return Ok(());
    }

    // If invoked as drift-ctl, or if a subcommand was provided, act as ctl
    if program_name == "drift-ctl" || cli.command.is_some() {
        let cmd = cli.command.unwrap_or(Commands::Toggle); // Default to toggle if drift-ctl was run with no args
        return ctl::run_ctl(cmd);
    }

    // Otherwise, start the daemon
    let socket = get_sway_socket(cli.socket)?;
    daemon::run_daemon(&socket)
}

fn main() {
    if let Err(e) = run() {
        eprintln!("Error: {}", e);
        match e {
            DriftError::SocketNotFound
            | DriftError::IpcConnect(_)
            | DriftError::IpcRead(_)
            | DriftError::IpcWrite(_)
            | DriftError::InvalidResponse(_) => {
                process::exit(1);
            }
            DriftError::StateIo(_) => {
                process::exit(2);
            }
            DriftError::DaemonNotRunning | DriftError::DaemonAlreadyRunning => {
                process::exit(3);
            }
        }
    }
}
