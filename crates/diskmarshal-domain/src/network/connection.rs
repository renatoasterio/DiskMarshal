//! One open connection of a program.

use std::fmt;
use std::time::Duration;

use crate::shared::ByteSize;

/// Ports whose use is fixed well enough to name what a connection is for.
const WELL_KNOWN_SERVICES: [(u16, &str); 14] = [
    (22, "SSH"),
    (53, "DNS"),
    (80, "HTTP"),
    (123, "NTP"),
    (143, "IMAP"),
    (443, "HTTPS"),
    (465, "SMTP"),
    (587, "SMTP"),
    (853, "DNS"),
    (993, "IMAP"),
    (3478, "STUN"),
    (5223, "APNs"),
    (5353, "mDNS"),
    (5900, "VNC"),
];

/// How a connection carries its data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Transport {
    /// Delivery is confirmed and ordered.
    Tcp,
    /// Each packet travels on its own.
    Udp,
}

impl Transport {
    /// The name people know the transport by.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Tcp => "TCP",
            Self::Udp => "UDP",
        }
    }
}

/// One side of a connection.
#[derive(Debug, Clone, Default, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Endpoint {
    /// The address, as numbers. Empty when the connection is not tied to one.
    pub address: String,
    /// The port, when the connection is tied to one.
    pub port: Option<u16>,
}

impl Endpoint {
    /// Whether nothing is known about this side: a socket that is only
    /// waiting, not talking to anyone.
    #[must_use]
    pub fn is_unspecified(&self) -> bool {
        self.address.is_empty() && self.port.is_none()
    }
}

/// Writes the endpoint as `address:port`, with `*` for a part that is open.
impl fmt::Display for Endpoint {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let address = if self.address.is_empty() {
            "*"
        } else {
            &self.address
        };
        match self.port {
            // An IPv6 address has colons of its own.
            Some(port) if address.contains(':') => write!(f, "[{address}]:{port}"),
            Some(port) => write!(f, "{address}:{port}"),
            None => write!(f, "{address}:*"),
        }
    }
}

/// One open connection of a program, with what macOS counted on it since it
/// was opened.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Connection {
    /// How the data is carried.
    pub transport: Transport,
    /// The side on this Mac.
    pub local: Endpoint,
    /// The other side.
    pub remote: Endpoint,
    /// The network interface it goes through, such as `en0`. Empty when
    /// macOS does not say.
    pub interface: String,
    /// Bytes that arrived.
    pub received: ByteSize,
    /// Bytes that left.
    pub sent: ByteSize,
    /// Bytes that had to be sent again because they were lost on the way.
    pub retransmitted: ByteSize,
    /// Average time the other side takes to answer, when macOS measured it.
    pub round_trip: Option<Duration>,
}

impl Connection {
    /// What the connection is for, when the port of the other side says so.
    #[must_use]
    pub fn service(&self) -> Option<&'static str> {
        let port = self.remote.port.or(self.local.port)?;
        WELL_KNOWN_SERVICES
            .iter()
            .find(|(known, _)| *known == port)
            .map(|(_, name)| *name)
    }

    /// Whether anything was ever sent or received on it.
    #[must_use]
    pub fn has_traffic(&self) -> bool {
        !(self.received.is_zero() && self.sent.is_zero())
    }
}

#[cfg(test)]
mod tests {
    use super::{Connection, Endpoint, Transport};
    use crate::shared::ByteSize;

    fn endpoint(address: &str, port: Option<u16>) -> Endpoint {
        Endpoint {
            address: address.to_owned(),
            port,
        }
    }

    fn connection(remote: Endpoint, local: Endpoint) -> Connection {
        Connection {
            transport: Transport::Tcp,
            local,
            remote,
            interface: "en0".to_owned(),
            received: ByteSize::ZERO,
            sent: ByteSize::ZERO,
            retransmitted: ByteSize::ZERO,
            round_trip: None,
        }
    }

    #[test]
    fn writes_an_endpoint_the_way_people_read_it() {
        assert_eq!(
            endpoint("203.0.113.24", Some(443)).to_string(),
            "203.0.113.24:443"
        );
        assert_eq!(
            endpoint("2001:db8::1", Some(443)).to_string(),
            "[2001:db8::1]:443"
        );
        assert_eq!(endpoint("", Some(5353)).to_string(), "*:5353");
        assert_eq!(endpoint("", None).to_string(), "*:*");
        assert!(endpoint("", None).is_unspecified());
        assert!(!endpoint("", Some(53)).is_unspecified());
    }

    #[test]
    fn names_the_service_by_the_port_of_the_other_side() {
        let web = connection(
            endpoint("203.0.113.24", Some(443)),
            endpoint("", Some(51_000)),
        );
        assert_eq!(web.service(), Some("HTTPS"));
        let unknown = connection(endpoint("203.0.113.24", Some(4070)), endpoint("", None));
        assert_eq!(unknown.service(), None);
        let listening = connection(endpoint("", None), endpoint("", Some(5353)));
        assert_eq!(listening.service(), Some("mDNS"));
    }

    #[test]
    fn a_connection_that_never_moved_a_byte_has_no_traffic() {
        let mut idle = connection(endpoint("", None), endpoint("", None));
        assert!(!idle.has_traffic());
        idle.sent = ByteSize::from_bytes(1);
        assert!(idle.has_traffic());
        assert_eq!(Transport::Udp.label(), "UDP");
    }
}
