# UDP forwarder

This is a simple, single-threaded, dependency-free implementation of a UDP forwarder in Rust.

It supports both unicast and multicast for IPv4 and IPv6.
The application is intentionally kept small and single-threaded.
One goal was implementing it with only the Rust standard library,
therefore async was not an option.

## Building

Build with `cargo` in release mode:

```sh
cargo build --release
```

## Running

The help lists plenty of example usages, mentioned here again as an overview.

```sh
udpforwarder -h
```

```
UDP forwarder

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
```

## Testing

Until I find time to add integration tests,
you can test locally using `netcat` (`nc`) on Linux.

Run the following commands in separate terminals

```sh
# Create netcat invocation to push data for forwarding
nc -u 127.0.0.1 4000

# Allow multicast traffic ports
sudo ufw allow 4002/udp 
sudo ufw allow 4003/udp

# Forward from local port to local port and multicast groups
udpforwarder 127.0.0.1:4000 127.0.0.1:4001 224.10.10.10:4002 [ff02::1]:4003

# Forward from IPv4 multicast group to local port
udpforwarder 224.10.10.10:4002 127.0.0.1:4004

# Forward from IPv6 multicast group to local port
udpforwarder [ff02::1]:4003 [::1]:4005

# Listen on forwarded ports
nc -ul 127.0.0.1 4001
nc -ul 127.0.0.1 4004
nc -ul ::1 4005
```
