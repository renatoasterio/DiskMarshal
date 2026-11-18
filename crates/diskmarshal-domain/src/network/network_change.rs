//! A change to the network settings of this Mac.

use std::net::IpAddr;

/// Longest name of a network service accepted.
const MAX_SERVICE_NAME_LENGTH: usize = 64;

/// One change to how this Mac uses the network. Every change can be taken
/// back by another one, and none of them touches a file of the user.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NetworkChange {
    /// Empty the addresses macOS remembers for names it already looked up.
    FlushDnsCache,
    /// Turn on or off the search for a proxy before each new connection.
    SetProxyAutoDiscovery {
        /// The network service to change, such as `Wi-Fi`.
        service: String,
        /// Whether the search stays on.
        is_enabled: bool,
    },
    /// Set the name servers of a service.
    SetDnsServers {
        /// The network service to change, such as `Wi-Fi`.
        service: String,
        /// The servers, in the order they are asked. Empty goes back to the
        /// ones the network hands out.
        servers: Vec<IpAddr>,
    },
}

impl NetworkChange {
    /// Most name servers one service may be given.
    pub const MAX_DNS_SERVERS: usize = 4;

    /// The network service the change applies to, when it applies to one.
    #[must_use]
    pub fn service(&self) -> Option<&str> {
        match self {
            Self::FlushDnsCache => None,
            Self::SetProxyAutoDiscovery { service, .. } | Self::SetDnsServers { service, .. } => {
                Some(service)
            }
        }
    }

    /// Whether the change is well formed: a plausible service name and no
    /// more name servers than a service takes.
    #[must_use]
    pub fn is_well_formed(&self) -> bool {
        let is_service_plausible = self.service().is_none_or(is_service_name);
        let fits = match self {
            Self::SetDnsServers { servers, .. } => servers.len() <= Self::MAX_DNS_SERVERS,
            Self::FlushDnsCache | Self::SetProxyAutoDiscovery { .. } => true,
        };
        is_service_plausible && fits
    }
}

/// Whether `text` can be the name of a network service: printable, not too
/// long, with no space around it, and nothing that reads as an option.
#[must_use]
pub fn is_service_name(text: &str) -> bool {
    !text.is_empty()
        && text.chars().count() <= MAX_SERVICE_NAME_LENGTH
        && text.trim() == text
        && !text.starts_with('-')
        && !text.chars().any(char::is_control)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use super::{NetworkChange, is_service_name};

    #[test]
    fn a_service_name_is_printable_and_cannot_be_read_as_an_option() {
        for name in ["Wi-Fi", "Thunderbolt Bridge", "USB 10/100/1000 LAN"] {
            assert!(is_service_name(name), "{name}");
        }
        for name in [
            "",
            " Wi-Fi",
            "Wi-Fi ",
            "-setdnsservers",
            "Wi\nFi",
            &"x".repeat(65),
        ] {
            assert!(!is_service_name(name), "{name:?}");
        }
    }

    #[test]
    fn a_change_is_well_formed_only_with_a_plausible_service() {
        let server = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 53));
        let dns = |service: &str, count: usize| NetworkChange::SetDnsServers {
            service: service.to_owned(),
            servers: vec![server; count],
        };
        assert!(NetworkChange::FlushDnsCache.is_well_formed());
        assert!(dns("Wi-Fi", 0).is_well_formed());
        assert!(dns("Wi-Fi", 4).is_well_formed());
        assert!(!dns("Wi-Fi", 5).is_well_formed());
        assert!(!dns("--help", 1).is_well_formed());
        let proxy = NetworkChange::SetProxyAutoDiscovery {
            service: String::new(),
            is_enabled: false,
        };
        assert!(!proxy.is_well_formed());
        assert_eq!(dns("Wi-Fi", 1).service(), Some("Wi-Fi"));
        assert_eq!(NetworkChange::FlushDnsCache.service(), None);
    }
}
