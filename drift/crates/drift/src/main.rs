use std::env;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};
use drift_core::actions::Action;
use drift_core::ipc::{IpcClient, IpcCommandType};
use drift_core::state::LockfileState;
use drift_core::DriftError;

#[derive(clap::ValueEnum, Clone)]
enum Shell {
    Bash,
    Zsh,
    Fish,
}

#[derive(Parser)]
#[command(name = "drift")]
#[command(about = "Manage horizontal scroll-style workspace layout on Sway WM", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,

    /// Override $SWAYSOCK
    #[arg(long)]
    socket: Option<String>,
}

#[derive(Subcommand)]
enum Commands {
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

fn dispatch_action(action: Action, socket: &str) -> Result<(), DriftError> {
    let mut client = IpcClient::connect(socket)?;
    client.send(action.ipc_command(), IpcCommandType::RunCommand)?;
    Ok(())
}

fn run() -> Result<(), DriftError> {
    let cli = Cli::parse();

    if let Commands::Completions { shell } = cli.command {
        let mut cmd = Cli::command();
        match shell {
            Shell::Bash => generate(Bash, &mut cmd, "drift", &mut std::io::stdout()),
            Shell::Zsh => generate(Zsh, &mut cmd, "drift", &mut std::io::stdout()),
            Shell::Fish => generate(Fish, &mut cmd, "drift", &mut std::io::stdout()),
        }
        return Ok(());
    }

    let socket = get_sway_socket(cli.socket)?;
    let state = LockfileState::new("/tmp/drift.lock");

    match cli.command {
        Commands::Status => {
            if state.is_active() {
                println!("active");
            } else {
                println!("inactive");
            }
        }
        Commands::Activate => state.set_active()?,
        Commands::Deactivate => state.set_inactive()?,
        Commands::Toggle => {
            if state.is_active() {
                state.set_inactive()?;
            } else {
                state.set_active()?;
            }
        }
        Commands::Next => {
            if state.is_active() {
                dispatch_action(Action::Next, &socket)?
            }
        }
        Commands::Prev => {
            if state.is_active() {
                dispatch_action(Action::Prev, &socket)?
            }
        }
        Commands::MoveNext => {
            if state.is_active() {
                dispatch_action(Action::MoveNext, &socket)?
            }
        }
        Commands::MovePrev => {
            if state.is_active() {
                dispatch_action(Action::MovePrev, &socket)?
            }
        }
        Commands::Back => {
            if state.is_active() {
                dispatch_action(Action::Back, &socket)?
            }
        }
        Commands::Completions { .. } => unreachable!(),
    }

    Ok(())
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
