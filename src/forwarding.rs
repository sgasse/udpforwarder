//! Forwarding

use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
};

use crate::ListenerSpec;

/// Forward from a listener to a set of forward addresses
pub fn forward(listener_spec: ListenerSpec, forward_addrs: &[SocketAddr]) -> Result<(), io::Error> {
    let listener: UdpSocket = listener_spec.try_into()?;
    let senders = Senders::for_addresses(forward_addrs)?;

    const MTU: usize = 1500;
    let mut buffer = [0; MTU];

    loop {
        let num_bytes = listener.recv(&mut buffer)?;

        for forward_addr in forward_addrs {
            senders.send_to(&buffer[..num_bytes], forward_addr)?;
        }
    }
}

/// Set of IPv4/IPv6-bound [UdpSocket]s to use for sending
struct Senders {
    /// IPv4-bound socket, only used if we have any IPv4 forwarding targets
    sender_v4: Option<UdpSocket>,
    /// IPv6-bound socket, only used if we have any IPv6 forwarding targets
    sender_v6: Option<UdpSocket>,
}

impl Senders {
    /// Create a set of senders for the given forward specifications
    fn for_addresses(forward_specs: &[SocketAddr]) -> Result<Self, io::Error> {
        let sender_v4 = if forward_specs.iter().any(|addr| addr.is_ipv4()) {
            match UdpSocket::bind(SocketAddr::V4(SocketAddrV4::new(Ipv4Addr::UNSPECIFIED, 0))) {
                Ok(sender) => Some(sender),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        let sender_v6 = if forward_specs.iter().any(|addr| addr.is_ipv6()) {
            match UdpSocket::bind(SocketAddr::V6(SocketAddrV6::new(
                Ipv6Addr::UNSPECIFIED,
                0,
                0,
                0,
            ))) {
                Ok(sender) => Some(sender),
                Err(e) => return Err(e),
            }
        } else {
            None
        };

        Ok(Self {
            sender_v4,
            sender_v6,
        })
    }

    /// Send data to the given address, using the correct sender for the IP family of the address
    fn send_to(&self, data: &[u8], addr: &SocketAddr) -> Result<usize, io::Error> {
        let sender = match addr {
            SocketAddr::V4(_) => self
                .sender_v4
                .as_ref()
                .expect("initialized sender for IPv4"),
            SocketAddr::V6(_) => self
                .sender_v6
                .as_ref()
                .expect("initialized sender for IPv6"),
        };

        sender.send_to(data, addr)
    }
}
