//! UDP listener
//!
//! This module contains the [ListenerSpec],
//! differentiating between unicast.
//! and multicast groups,
//! all available as IPv4 and IPv6.
//!
//! Note that firewall rules are a common source of issues with multicast setups.

use std::{
    io,
    net::{Ipv4Addr, Ipv6Addr, SocketAddr, SocketAddrV4, SocketAddrV6, UdpSocket},
};

/// Specification of the UDP listener
#[derive(Debug, PartialEq)]
pub enum ListenerSpec {
    /// Incoming unicast stream, IPv4 or IPv6
    Unicast(SocketAddr),
    /// IPv4 multicast group to join with local address of the interface to use
    ///
    /// If the user does not specify the local address, it is [Ipv4Addr::UNSPECIFIED].
    MulticastV4 {
        multicast_group: SocketAddrV4,
        local_addr: Ipv4Addr,
    },
    /// IPv6 multicast group to join with ID of the interface to use
    ///
    /// If the user does not specify the interface ID, it is `0` for any interface.
    MulticastV6 {
        multicast_group: SocketAddrV6,
        interface_id: u32,
    },
}

impl TryFrom<ListenerSpec> for UdpSocket {
    type Error = io::Error;

    fn try_from(listener_spec: ListenerSpec) -> Result<Self, Self::Error> {
        match listener_spec {
            ListenerSpec::Unicast(socket_addr) => UdpSocket::bind(socket_addr),
            ListenerSpec::MulticastV4 {
                multicast_group,
                local_addr,
            } => {
                let socket = UdpSocket::bind((Ipv4Addr::UNSPECIFIED, multicast_group.port()))?;
                socket.join_multicast_v4(multicast_group.ip(), &local_addr)?;

                Ok(socket)
            }
            ListenerSpec::MulticastV6 {
                multicast_group,
                interface_id,
            } => {
                let socket = UdpSocket::bind((Ipv6Addr::UNSPECIFIED, multicast_group.port()))?;
                socket.join_multicast_v6(multicast_group.ip(), interface_id)?;

                Ok(socket)
            }
        }
    }
}
