use drift_core::ipc::IpcClient;
fn main() {
    let sock = std::env::var("SWAYSOCK").unwrap();
    let mut client = IpcClient::connect(&sock).unwrap();
    println!("Count: {}", client.focused_workspace_window_count().unwrap());
}
