//! How long the other side takes to answer.

use std::net::{IpAddr, Ipv4Addr};
use std::time::Duration;

/// A name service anyone may use, and the pair of addresses it answers on.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PublicResolver {
    /// Who runs it.
    pub provider: &'static str,
    /// Its main and its spare address.
    pub servers: [Ipv4Addr; 2],
}

/// The public name services the current one is compared with.
pub const PUBLIC_RESOLVERS: [PublicResolver; 3] = [
    PublicResolver {
        provider: "Cloudflare",
        servers: [Ipv4Addr::new(1, 1, 1, 1), Ipv4Addr::new(1, 0, 0, 1)],
    },
    PublicResolver {
        provider: "Google",
        servers: [Ipv4Addr::new(8, 8, 8, 8), Ipv4Addr::new(8, 8, 4, 4)],
    },
    PublicResolver {
        provider: "Quad9",
        servers: [Ipv4Addr::new(9, 9, 9, 9), Ipv4Addr::new(149, 112, 112, 112)],
    },
];

/// The address pinged to tell whether the internet answers: one that is
/// reachable from everywhere and close to most places.
pub const INTERNET_ANCHOR: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));

impl PublicResolver {
    /// The public resolver that answers on `server`, if any.
    #[must_use]
    pub fn serving(server: IpAddr) -> Option<&'static Self> {
        PUBLIC_RESOLVERS.iter().find(|resolver| {
            resolver
                .servers
                .iter()
                .any(|known| IpAddr::V4(*known) == server)
        })
    }
}

/// What came back from a series of pings to one address.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PingResult {
    /// Packets sent.
    pub sent: u32,
    /// Packets answered.
    pub received: u32,
    /// Average answer time, when anything was answered.
    pub average: Option<Duration>,
}

impl PingResult {
    /// Share of the packets that got no answer, from 0.0 to 1.0.
    #[must_use]
    pub fn loss_ratio(&self) -> f32 {
        if self.sent == 0 {
            return 0.0;
        }
        (self.sent.saturating_sub(self.received) as f32 / self.sent as f32).clamp(0.0, 1.0)
    }

    /// Whether nothing was answered. Many routers ignore pings on purpose,
    /// so silence alone does not mean the address is unreachable.
    #[must_use]
    pub const fn is_silent(&self) -> bool {
        self.received == 0
    }
}

/// How long one name server took to answer a question.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DnsTiming {
    /// The server asked.
    pub server: IpAddr,
    /// Whether this Mac is using it now.
    pub is_in_use: bool,
    /// Its answer time. `None` when it did not answer.
    pub answer_time: Option<Duration>,
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::{PingResult, PublicResolver};

    #[test]
    fn loss_is_the_share_of_pings_without_an_answer() {
        let ping = |sent, received| PingResult {
            sent,
            received,
            average: None,
        };
        assert!(ping(10, 10).loss_ratio().abs() < f32::EPSILON);
        assert!((ping(10, 7).loss_ratio() - 0.3).abs() < 1e-6);
        assert!((ping(5, 0).loss_ratio() - 1.0).abs() < f32::EPSILON);
        assert!(ping(0, 0).loss_ratio().abs() < f32::EPSILON);
        assert!(ping(5, 0).is_silent() && !ping(5, 1).is_silent());
    }

    #[test]
    fn finds_the_public_resolver_by_either_of_its_addresses() {
        let spare = IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4));
        assert_eq!(PublicResolver::serving(spare).unwrap().provider, "Google");
        assert!(PublicResolver::serving(IpAddr::V4(Ipv4Addr::new(192, 0, 2, 1))).is_none());
    }
}
