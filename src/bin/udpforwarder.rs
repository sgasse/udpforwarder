//! UDP forwarder

use udpforwarder::{ParseArgsError, forward, parse_args};

fn main() {
    // Parse and handle arguments
    let args = match parse_args(std::env::args().skip(1)) {
        Ok(args) => args,
        Err(e) => {
            match e {
                ParseArgsError::Help => {
                    println!("{HELP}");
                }
                ParseArgsError::MissingArgs => {
                    eprintln!("Missing arguments\n");
                    println!("{HELP}");
                }
                ParseArgsError::ListenerSpec => {
                    eprintln!("Failed to parse the listener specification");
                }
                ParseArgsError::ForwardSpec(e) => {
                    eprintln!("Failed to parse the listener specification: {e}");
                }
            }
            return;
        }
    };

    // Forward from listening socket to forward addresses
    if let Err(e) = forward(args.listener_spec, &args.forward_addrs) {
        eprintln!("Failed to forward: {e}");
    }
}

const HELP: &str = r#"UDP forwarder

usage: udpforwarder [listener_spec] [target_addr] [...target_addr]

examples:

  Forward incoming IPV4 unicast stream to IPv4 localhost

    udpforwarder 10.1.1.10:4000 127.0.0.1:4001

  Forward incoming IPv4 unicast stream to IPv4 and IPv6 localhost

    udpforwarder 10.1.1.10:4000 127.0.0.1:4001 [::1]:4002

  Subscribe to IPv4 multicast group on any interface and forward to remote address

    udpforwarder 224.10.10.10:4000 10.1.1.11:4000

  Subscribe to IPv4 multicast group specifying the local address of the interface to use
  and forward to local port

    udpforwarder 224.10.10.10:4000 127.0.0.1:4001

  Subscribe to IPv6 multicast group specifying the ID of the interface to use
  and forward to local port

    udpforwarder [ff05::1]:4000 [::1]:4001

"#;
