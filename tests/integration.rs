//! Integration tests

use std::{
    io::ErrorKind,
    net::{SocketAddr, UdpSocket},
    path::{Path, PathBuf},
    process::Command,
    time::Duration,
};

/// Launch the pre-built binary and kill it again
#[test]
fn launch_kill_process() {
    let binary_path = get_binary_path().expect("binary exists");
    println!("Using binary {}", binary_path.display());

    let mut handle = Command::new(binary_path)
        .args(["127.0.0.1:3000", "127.0.0.1:3001"])
        .spawn()
        .expect("spawn process");
    handle.kill().expect("kill child process");
}

/// Receive packets through a simple forward from one localhost port to another
#[test]
fn simple_ipv4_forward() {
    let binary_path = get_binary_path().expect("binary exists");
    println!("Using binary {}", binary_path.display());

    let incoming_address: SocketAddr = "127.0.0.1:4000".parse().unwrap();
    let forwarded_address: SocketAddr = "127.0.0.1:4001".parse().unwrap();

    let sender =
        UdpSocket::bind("127.0.0.1:0".parse::<SocketAddr>().unwrap()).expect("bind sender");
    let forwarded_listener = UdpSocket::bind(forwarded_address).expect("bind listener");
    forwarded_listener
        .set_read_timeout(Some(Duration::from_millis(100)))
        .expect("set read timeout");

    let mut handle = Command::new(binary_path)
        .args(["127.0.0.1:4000", "127.0.0.1:4001"])
        .spawn()
        .expect("spawn process");

    let mut recv_buffer = [0; 1500];

    // Fire packets until the forwarding is up
    let msg = b"establish connection";

    loop {
        sender.send_to(msg, incoming_address).expect("send");

        match forwarded_listener.recv(&mut recv_buffer) {
            Ok(num_received) => {
                if num_received == msg.len() {
                    break;
                }
            }
            Err(e) if e.kind() == ErrorKind::WouldBlock => {
                continue;
            }
            Err(e) => panic!("failed to receive on socket: {e}"),
        }
    }

    // Forwarding is established
    // Send a known sequence of packets and expect to receive it

    for i in 0..10 {
        let msg = format!("packet number {i}");

        sender
            .send_to(msg.as_bytes(), incoming_address)
            .expect("send");

        loop {
            match forwarded_listener.recv(&mut recv_buffer) {
                Ok(num_received) => {
                    assert_eq!(num_received, msg.bytes().len());
                    assert_eq!(&recv_buffer[..num_received], msg.as_bytes());
                    break;
                }
                Err(e) if e.kind() == ErrorKind::WouldBlock => {
                    continue;
                }
                Err(e) => panic!("failed to receive on socket: {e}"),
            }
        }
    }

    handle.kill().expect("kill child process");
}

fn get_binary_path() -> Option<PathBuf> {
    #[cfg(target_family = "unix")]
    const CANDIDATES: &[&str] = &[
        "./target/release/udpforwarder",
        "./target/debug/udpforwarder",
        "../target/release/udpforwarder",
        "../target/debug/udpforwarder",
    ];

    #[cfg(target_family = "windows")]
    const CANDIDATES: &[&str] = &[
        ".\\target\\release\\udpforwarder.exe",
        ".\\target\\debug\\udpforwarder.exe",
        "..\\target\\release\\udpforwarder.exe",
        "..\\target\\debug\\udpforwarder.exe",
    ];

    for candidate in CANDIDATES {
        let path = Path::new(candidate);
        if path.is_file() {
            return Some(path.to_owned());
        }
    }

    None
}
