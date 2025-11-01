//! CLI argument parsing

use std::{
    net::{AddrParseError, Ipv4Addr, SocketAddr},
    str::FromStr,
};

use crate::ListenerSpec;

/// Arguments for UDP forwarding
pub struct Args {
    /// Specification of the listener
    ///
    /// Can be unicast or a multicast group,
    /// both IPv4 and IPv6.
    pub listener_spec: ListenerSpec,
    /// Addresses to forward UDP packets to
    ///
    /// Can be unicast or a multicast group,
    /// both IPv4 and IPv6.
    pub forward_addrs: Vec<SocketAddr>,
}

/// Error or parsing arguments
pub enum ParseArgsError {
    /// CLI help requested
    Help,
    /// Missing required arguments
    MissingArgs,
    /// Failed to parse listener specification
    ListenerSpec,
    /// Failed to parse forward address specification
    ForwardSpec(AddrParseError),
}

/// Parse arguments of UDP forwarding
pub fn parse_args(args: impl IntoIterator<Item = String>) -> Result<Args, ParseArgsError> {
    let mut args = args.into_iter();

    let listener_spec: ListenerSpec = match args.next() {
        None => return Err(ParseArgsError::MissingArgs),
        Some(arg) if arg == "--help" || arg == "-h" => return Err(ParseArgsError::Help),
        Some(spec) => match spec.parse() {
            Ok(spec) => spec,
            Err(_) => return Err(ParseArgsError::ListenerSpec),
        },
    };

    let forward_addrs = args
        .map(|arg| arg.parse())
        .collect::<Result<Vec<SocketAddr>, AddrParseError>>()
        .map_err(ParseArgsError::ForwardSpec)?;

    if forward_addrs.is_empty() {
        return Err(ParseArgsError::MissingArgs);
    }

    Ok(Args {
        listener_spec,
        forward_addrs,
    })
}

impl FromStr for ListenerSpec {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // Try to parse as socket address without further details
        if let Ok(addr) = s.parse() {
            return match addr {
                SocketAddr::V4(addr_v4) => {
                    if addr_v4.ip().is_multicast() {
                        Ok(ListenerSpec::MulticastV4 {
                            multicast_group: addr_v4,
                            // Use unspecified local address for any interface
                            local_addr: Ipv4Addr::UNSPECIFIED,
                        })
                    } else {
                        Ok(ListenerSpec::Unicast(addr))
                    }
                }
                SocketAddr::V6(addr_v6) => {
                    if addr_v6.ip().is_multicast() {
                        Ok(ListenerSpec::MulticastV6 {
                            multicast_group: addr_v6,
                            // Use ID zero for any interface
                            interface_id: 0,
                        })
                    } else {
                        Ok(ListenerSpec::Unicast(addr))
                    }
                }
            };
        }

        // Try to interpret as combination of multicast group and details
        let Some((multicast_group, local_intf)) = s.split_once('/') else {
            return Err(());
        };

        match multicast_group.parse() {
            // IPv4 multicast with details
            Ok(SocketAddr::V4(multicast_group)) if multicast_group.ip().is_multicast() => {
                match local_intf.parse() {
                    Ok(local_addr) => Ok(ListenerSpec::MulticastV4 {
                        multicast_group,
                        local_addr,
                    }),
                    Err(_) => Err(()),
                }
            }
            // IPv6 multicast with details
            Ok(SocketAddr::V6(multicast_group)) if multicast_group.ip().is_multicast() => {
                match local_intf.parse() {
                    Ok(interface_id) => Ok(ListenerSpec::MulticastV6 {
                        multicast_group,
                        interface_id,
                    }),
                    Err(_) => Err(()),
                }
            }
            // Unicast with multicast details or unparsable
            Ok(_) | Err(_) => Err(()),
        }
    }
}

#[cfg(test)]
mod test {
    use std::net::{Ipv6Addr, SocketAddrV4, SocketAddrV6};

    use super::*;

    #[test]
    fn listener_spec_ipv4_unicast_ok() {
        let spec = "10.1.1.10:4000";
        let expected = ListenerSpec::Unicast(SocketAddr::V4(SocketAddrV4::new(
            Ipv4Addr::new(10, 1, 1, 10),
            4000,
        )));

        assert_eq!(expected, spec.parse().unwrap());
    }

    #[test]
    fn listener_spec_ipv4_multicast_no_details_ok() {
        let spec = "224.10.10.10:4000";
        let expected = ListenerSpec::MulticastV4 {
            multicast_group: SocketAddrV4::new(Ipv4Addr::new(224, 10, 10, 10), 4000),
            local_addr: Ipv4Addr::UNSPECIFIED,
        };

        assert_eq!(expected, spec.parse().unwrap());
    }

    #[test]
    fn listener_spec_ipv4_multicast_local_addr_ok() {
        let spec = "224.10.10.10:4000/192.168.1.10";
        let expected = ListenerSpec::MulticastV4 {
            multicast_group: SocketAddrV4::new(Ipv4Addr::new(224, 10, 10, 10), 4000),
            local_addr: Ipv4Addr::new(192, 168, 1, 10),
        };

        assert_eq!(expected, spec.parse().unwrap());
    }

    #[test]
    fn listener_spec_ipv6_unicast_ok() {
        let spec = "[2001::1]:4000";
        let expected = ListenerSpec::Unicast(SocketAddr::V6(SocketAddrV6::new(
            Ipv6Addr::new(0x2001, 0, 0, 0, 0, 0, 0, 1),
            4000,
            0,
            0,
        )));

        assert_eq!(expected, spec.parse().unwrap());
    }

    #[test]
    fn listener_spec_ipv6_multicast_no_details_ok() {
        let spec = "[ff0e::1]:4000";
        let expected = ListenerSpec::MulticastV6 {
            multicast_group: SocketAddrV6::new(
                Ipv6Addr::new(0xff0e, 0, 0, 0, 0, 0, 0, 1),
                4000,
                0,
                0,
            ),
            interface_id: 0,
        };

        assert_eq!(expected, spec.parse().unwrap());
    }

    #[test]
    fn listener_spec_ipv6_multicast_interface_id_ok() {
        let spec = "[ff0e::1]:4000/2";
        let expected = ListenerSpec::MulticastV6 {
            multicast_group: SocketAddrV6::new(
                Ipv6Addr::new(0xff0e, 0, 0, 0, 0, 0, 0, 1),
                4000,
                0,
                0,
            ),
            interface_id: 2,
        };

        assert_eq!(expected, spec.parse().unwrap());
    }
}
