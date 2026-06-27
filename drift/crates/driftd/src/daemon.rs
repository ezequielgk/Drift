use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};
use std::thread;

use drift_core::actions::Action;
use drift_core::config::DriftConfig;
use drift_core::ipc::{IpcClient, IpcCommandType};
use drift_core::state::LayoutState;
use drift_core::DriftError;

const DAEMON_SOCKET: &str = "/tmp/drift.sock";

fn dispatch_action(action: Action, client: &mut IpcClient) -> Result<(), DriftError> {
    let current = client.focused_workspace_number()?;
    client.send(&action.ipc_command_for(current), IpcCommandType::RunCommand)?;
    Ok(())
}

pub fn run_daemon(sway_socket: &str) -> Result<(), DriftError> {
    if fs::metadata(DAEMON_SOCKET).is_ok() {
        if std::os::unix::net::UnixStream::connect(DAEMON_SOCKET).is_ok() {
            return Err(DriftError::DaemonAlreadyRunning);
        } else {
            // Orphaned socket, remove it
            let _ = fs::remove_file(DAEMON_SOCKET);
        }
    }

    let listener = UnixListener::bind(DAEMON_SOCKET).map_err(DriftError::StateIo)?;
    let state = Arc::new(Mutex::new(LayoutState::default()));

    let sway_sock = sway_socket.to_string();
    let state_clone = Arc::clone(&state);

    thread::spawn(move || {
        println!("[daemon] Background thread started");
        if let Ok(mut event_conn) = IpcClient::connect(&sway_sock) {
            println!("[daemon] Connected to sway IPC for events");
            if event_conn.subscribe_window().is_ok() {
                println!("[daemon] Subscribed to window events successfully");
                loop {
                    match event_conn.read_event() {
                        Ok(event) => {
                            println!("[daemon] Received event: {}", event.change);
                            if event.change == "new" {
                                println!("[daemon] Detected new window!");
                                let is_active = { state_clone.lock().unwrap().active };
                                println!("[daemon] is_active: {}", is_active);
                                if is_active {
                                    match DriftConfig::load() {
                                        Ok(config) => {
                                            println!(
                                                "[daemon] Loaded config, max_windows = {}",
                                                config.max_windows
                                            );
                                            match IpcClient::connect(&sway_sock) {
                                                Ok(mut action_conn) => {
                                                    match action_conn
                                                        .focused_workspace_window_count()
                                                    {
                                                        Ok(count) => {
                                                            println!(
                                                                "[daemon] Window count = {}",
                                                                count
                                                            );
                                                            if count > config.max_windows {
                                                                println!("[daemon] Triggering overflow! ({} > {})", count, config.max_windows);

                                                                let delay =
                                                                    config.overflow_delay_ms;
                                                                if delay > 0 {
                                                                    std::thread::sleep(std::time::Duration::from_millis(delay));
                                                                }

                                                                match action_conn.focused_workspace_number() {
                                                                    Ok(current) => {
                                                                        let res = action_conn.send(
                                                                            &Action::MoveNext.ipc_command_for(current),
                                                                            IpcCommandType::RunCommand,
                                                                        );
                                                                        println!("[daemon] Overflow action result: {:?}", res);
                                                                    }
                                                                    Err(e) => println!("[daemon] Error getting current ws: {:?}", e),
                                                                }
                                                            }
                                                        }
                                                        Err(e) => println!(
                                                            "[daemon] Error getting count: {:?}",
                                                            e
                                                        ),
                                                    }
                                                }
                                                Err(e) => println!(
                                                    "[daemon] Error connecting for action: {:?}",
                                                    e
                                                ),
                                            }
                                        }
                                        Err(e) => {
                                            println!("[daemon] Error loading config: {:?}", e)
                                        }
                                    }
                                }
                            }
                        }
                        Err(DriftError::InvalidResponse(e)) => {
                            println!("[daemon] Ignored unparseable event: {}", e);
                            continue;
                        }
                        Err(e) => {
                            println!("[daemon] Fatal IO error reading event: {:?}", e);
                            break;
                        }
                    }
                }
            }
        }
    });

    // Test connection to Sway early
    let mut client = IpcClient::connect(sway_socket)?;

    for stream in listener.incoming() {
        match stream {
            Ok(mut stream) => {
                let mut buf = String::new();
                if stream.read_to_string(&mut buf).is_err() {
                    continue;
                }

                let command = buf.trim();
                let mut state = state.lock().unwrap();
                let mut response = "ok\n".to_string();

                let res: Result<(), DriftError> = match command {
                    "toggle" => {
                        state.active = !state.active;
                        Ok(())
                    }
                    "activate" => {
                        state.active = true;
                        Ok(())
                    }
                    "deactivate" => {
                        state.active = false;
                        Ok(())
                    }
                    "status" => {
                        response = if state.active {
                            "active\n".to_string()
                        } else {
                            "inactive\n".to_string()
                        };
                        Ok(())
                    }
                    "next" => {
                        if state.active {
                            dispatch_action(Action::Next, &mut client)
                        } else {
                            Ok(())
                        }
                    }
                    "prev" => {
                        if state.active {
                            dispatch_action(Action::Prev, &mut client)
                        } else {
                            Ok(())
                        }
                    }
                    "move-next" => {
                        if state.active {
                            dispatch_action(Action::MoveNext, &mut client)
                        } else {
                            Ok(())
                        }
                    }
                    "move-prev" => {
                        if state.active {
                            dispatch_action(Action::MovePrev, &mut client)
                        } else {
                            Ok(())
                        }
                    }
                    "back" => {
                        if state.active {
                            dispatch_action(Action::Back, &mut client)
                        } else {
                            Ok(())
                        }
                    }
                    "config get max-windows" => {
                        match DriftConfig::load() {
                            Ok(config) => response = format!("{}\n", config.max_windows),
                            Err(e) => response = format!("error: {}\n", e),
                        }
                        Ok(())
                    }
                    "config get overflow-delay" => {
                        match DriftConfig::load() {
                            Ok(config) => response = format!("{}\n", config.overflow_delay_ms),
                            Err(e) => response = format!("error: {}\n", e),
                        }
                        Ok(())
                    }
                    cmd_str if cmd_str.starts_with("config get ") => {
                        response = "error: unknown config key\n".to_string();
                        Ok(())
                    }
                    cmd_str if cmd_str.starts_with("config set max-windows ") => {
                        let val_str = cmd_str.trim_start_matches("config set max-windows ");
                        if let Ok(val) = val_str.parse::<u32>() {
                            if val >= 1 {
                                match DriftConfig::load() {
                                    Ok(mut config) => {
                                        config.max_windows = val;
                                        if let Err(e) = config.save() {
                                            response = format!("error: {}\n", e);
                                        }
                                    }
                                    Err(e) => response = format!("error: {}\n", e),
                                }
                            } else {
                                response = "error: max-windows must be a positive integer (>= 1)\n"
                                    .to_string();
                            }
                        } else {
                            response = "error: max-windows must be an integer\n".to_string();
                        }
                        Ok(())
                    }
                    cmd_str if cmd_str.starts_with("config set overflow-delay ") => {
                        let val_str = cmd_str.trim_start_matches("config set overflow-delay ");
                        if let Ok(val) = val_str.parse::<u64>() {
                            match DriftConfig::load() {
                                Ok(mut config) => {
                                    config.overflow_delay_ms = val;
                                    if let Err(e) = config.save() {
                                        response = format!("error: {}\n", e);
                                    }
                                }
                                Err(e) => response = format!("error: {}\n", e),
                            }
                        } else {
                            response = "error: overflow-delay must be an integer\n".to_string();
                        }
                        Ok(())
                    }
                    cmd_str if cmd_str.starts_with("config set ") => {
                        response = "error: unknown config key\n".to_string();
                        Ok(())
                    }
                    _ => {
                        response = "error: invalid command\n".to_string();
                        Ok(())
                    }
                };

                if let Err(e) = res {
                    response = format!("error: {}\n", e);
                }

                let _ = stream.write_all(response.as_bytes());
            }
            Err(_) => continue,
        }
    }

    Ok(())
}
