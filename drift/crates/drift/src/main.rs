use std::env;
use std::process;

use clap::{CommandFactory, Parser, Subcommand};
use clap_complete::{
    generate,
    shells::{Bash, Fish, Zsh},
};
use drift_core::actions::Action;
use drift_core::config::DriftConfig;
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
enum ConfigCommand {
    /// Get a config value
    Get { key: String },
    /// Set a config value
    Set { key: String, value: String },
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

    /// Manage configuration
    Config {
        #[command(subcommand)]
        command: ConfigCommand,
    },

    /// Manually trigger window overflow check
    Overflow,
}

fn get_sway_socket(cli_socket: Option<String>) -> Result<String, DriftError> {
    if let Some(s) = cli_socket {
        return Ok(s);
    }
    env::var("SWAYSOCK").map_err(|_| DriftError::SocketNotFound)
}

fn dispatch_action(action: Action, socket: &str) -> Result<(), DriftError> {
    let mut client = IpcClient::connect(socket)?;
    let current = client.focused_workspace_number()?;
    client.send(&action.ipc_command_for(current), IpcCommandType::RunCommand)?;
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

    if let Commands::Config { command } = cli.command {
        let mut config = DriftConfig::load()?;
        match command {
            ConfigCommand::Get { key } => {
                if key == "max-windows" {
                    println!("{}", config.max_windows);
                } else if key == "overflow-delay" {
                    println!("{}", config.overflow_delay_ms);
                } else {
                    eprintln!("Error: Unknown config key '{}'", key);
                    process::exit(2);
                }
            }
            ConfigCommand::Set { key, value } => {
                if key == "max-windows" {
                    let val: u32 = value.parse().unwrap_or(0);
                    if val < 1 {
                        eprintln!("Error: max-windows must be a positive integer (>= 1)");
                        process::exit(2);
                    }
                    config.max_windows = val;
                    config.save()?;
                } else if key == "overflow-delay" {
                    let val: u64 = value.parse().unwrap_or_else(|_| {
                        eprintln!("Error: overflow-delay must be a positive integer");
                        process::exit(2);
                    });
                    config.overflow_delay_ms = val;
                    config.save()?;
                } else {
                    eprintln!("Error: Unknown config key '{}'", key);
                    process::exit(2);
                }
            }
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
        Commands::Overflow => {
            if state.is_active() {
                let config = DriftConfig::load()?;
                let mut client = IpcClient::connect(&socket)?;
                let count = client.focused_workspace_window_count()?;
                if count > config.max_windows {
                    let delay = config.overflow_delay_ms;
                    if delay > 0 {
                        std::thread::sleep(std::time::Duration::from_millis(delay));
                    }
                    let current = client.focused_workspace_number()?;
                    client.send(
                        &Action::MoveNext.ipc_command_for(current),
                        IpcCommandType::RunCommand,
                    )?;
                }
            }
        }
        Commands::Completions { .. } | Commands::Config { .. } => unreachable!(),
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
            DriftError::StateIo(_) | DriftError::ConfigIo(_) | DriftError::ConfigParse(_) => {
                process::exit(2);
            }
            DriftError::DaemonNotRunning | DriftError::DaemonAlreadyRunning => {
                process::exit(3);
            }
        }
    }
}
