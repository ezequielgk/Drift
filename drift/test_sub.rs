use std::os::unix::net::UnixStream;
use std::io::{Read, Write};

fn main() {
    let mut stream = UnixStream::connect(std::env::var("SWAYSOCK").unwrap()).unwrap();
    let magic = b"i3-ipc";
    
    let payload = r#"["window"]"#;
    let payload_bytes = payload.as_bytes();
    let payload_len = payload_bytes.len() as u32;

    let mut header = Vec::with_capacity(14);
    header.extend_from_slice(magic);
    header.extend_from_slice(&payload_len.to_ne_bytes());
    header.extend_from_slice(&(2u32).to_ne_bytes()); // Type 2 = SUBSCRIBE

    stream.write_all(&header).unwrap();
    stream.write_all(payload_bytes).unwrap();
    
    let mut h2 = [0u8; 14];
    stream.read_exact(&mut h2).unwrap();
    let len = u32::from_ne_bytes([h2[6], h2[7], h2[8], h2[9]]);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).unwrap();
    println!("Response: {}", String::from_utf8_lossy(&buf));
    
    println!("Waiting for event...");
    stream.read_exact(&mut h2).unwrap();
    let len = u32::from_ne_bytes([h2[6], h2[7], h2[8], h2[9]]);
    let mut buf = vec![0u8; len as usize];
    stream.read_exact(&mut buf).unwrap();
    println!("Event: {}", String::from_utf8_lossy(&buf));
}
