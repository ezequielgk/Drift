use std::fs;
use std::io::{Read, Write};
use std::os::unix::net::UnixListener;
use std::sync::{Arc, Mutex};

use drift_core::actions::Action;
use drift_core::ipc::{IpcClient, IpcCommandType};
use drift_core::state::LayoutState;
use drift_core::DriftError;

const DAEMON_SOCKET: &str = "/tmp/drift.sock";

fn dispatch_action(action: Action, client: &mut IpcClient) -> Result<(), DriftError> {
    client.send(action.ipc_command(), IpcCommandType::RunCommand)?;
    Ok(())
}

pub fn run_daemon(sway_socket: &str) -> Result<(), DriftError> {
    if fs::metadata(DAEMON_SOCKET).is_ok() {
        return Err(DriftError::DaemonAlreadyRunning);
    }

    let listener = UnixListener::bind(DAEMON_SOCKET).map_err(DriftError::StateIo)?;
    let state = Arc::new(Mutex::new(LayoutState::default()));

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
