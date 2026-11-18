//! How this Mac is set up to reach the internet.

use std::net::IpAddr;

use crate::network::WifiLink;

/// How connections look for a proxy before leaving.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProxySettings {
    /// Whether macOS asks the network for a proxy before each new connection.
    pub is_auto_discovery_on: bool,
    /// Whether the network answers that question. `None` when it was not
    /// checked.
    pub is_discovery_answered: Option<bool>,
    /// Whether a proxy was set by hand or by a configuration file.
    pub has_configured_proxy: bool,
}

/// The VPN profiles set up on this Mac.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct VpnProfiles {
    /// How many profiles exist.
    pub installed: usize,
    /// How many of them are connected right now.
    pub connected: usize,
}

/// The settings that decide how this Mac reaches the internet.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ConnectionProfile {
    /// Interface the traffic leaves through, such as `en0`.
    pub interface: String,
    /// Name macOS gives the service of that interface, such as `Wi-Fi`.
    /// Empty when it could not be found; nothing can then be changed.
    pub service: String,
    /// Address of this Mac on the local network.
    pub local_address: Option<IpAddr>,
    /// Address of the router.
    pub gateway: Option<IpAddr>,
    /// The name servers in use, in the order they are asked.
    pub dns_servers: Vec<IpAddr>,
    /// The name servers set by hand on the service. Empty when the network
    /// hands them out.
    pub configured_dns: Vec<IpAddr>,
    /// How connections look for a proxy.
    pub proxy: ProxySettings,
    /// The VPN profiles on this Mac.
    pub vpn: VpnProfiles,
    /// The wireless link, when the interface is a wireless one.
    pub wifi: Option<WifiLink>,
}
