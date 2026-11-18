//! Reading what the programs macOS ships print about the network settings.
//!
//! Each function reads the answer of one program. A line that cannot be
//! understood is skipped: what the other lines say is still true.

use std::net::IpAddr;

use diskmarshal_domain::network::VpnProfiles;

/// What ends the name of a figure in a line like `gateway: 192.0.2.1`.
const NAME_END: char = ':';
const ROUTE_INTERFACE: &str = "interface";
const ROUTE_GATEWAY: &str = "gateway";
/// Longest name an interface can have on macOS.
const MAX_INTERFACE_NAME_LENGTH: usize = 15;
/// How the line that says which device a service uses starts.
const HARDWARE_START: &str = "(Hardware Port:";
/// What comes before the device in that line.
const DEVICE_START: &str = ", Device:";
/// What stands in place of the position of a service that is turned off.
const DISABLED_SERVICE: &str = "*";
/// How the first line of each resolver starts.
const RESOLVER_START: &str = "resolver #";
/// How the line of each name server of a resolver starts.
const NAME_SERVER_START: &str = "nameserver[";
const AUTO_DISCOVERY: &str = "Auto Proxy Discovery";
const ON: &str = "On";
const ENABLED: &str = "Enabled";
const YES: &str = "Yes";
/// The lines of a host entry that carry an address, for IPv4 and IPv6.
const HOST_ADDRESSES: [&str; 2] = ["ip_address", "ipv6_address"];
/// How the line of a VPN profile that is turned on starts.
const ENABLED_PROFILE: char = '*';
/// The state of a profile that is connected, which follows the mark.
const CONNECTED: &str = "(Connected)";

/// The way out of this Mac to other networks.
#[derive(Debug, PartialEq, Eq)]
pub(crate) struct DefaultRoute {
    /// Interface the traffic leaves through, such as `en0`.
    pub interface: String,
    /// Address of the router. A link that needs no router has none.
    pub gateway: Option<IpAddr>,
}

/// Reads `route -n get default`. `None` when there is no way out.
pub(crate) fn default_route(report: &str) -> Option<DefaultRoute> {
    let interface = value_of(report, ROUTE_INTERFACE).filter(|name| is_interface_name(name))?;
    Some(DefaultRoute {
        interface: interface.to_owned(),
        gateway: value_of(report, ROUTE_GATEWAY).and_then(|address| address.parse().ok()),
    })
}

/// Reads `networksetup -listnetworkserviceorder` and returns the name of the
/// service that uses `interface`.
///
/// Each service takes two lines: `(1) Wi-Fi`, then
/// `(Hardware Port: Wi-Fi, Device: en0)`.
pub(crate) fn service_of(report: &str, interface: &str) -> Option<String> {
    let mut listed = None;
    for line in report.lines() {
        if let Some(device) = device_of(line) {
            if !device.is_empty()
                && device == interface
                && let Some(name) = listed
            {
                return Some(name);
            }
            listed = None;
        } else if let Some(name) = service_name(line) {
            listed = Some(name.to_owned());
        }
    }
    None
}

/// Reads `scutil --dns` and returns the name servers of the first resolver:
/// the one asked for every name no other resolver claims.
pub(crate) fn name_servers_in_use(report: &str) -> Vec<IpAddr> {
    report
        .lines()
        .map(str::trim)
        .skip_while(|line| !line.starts_with(RESOLVER_START))
        .skip(1)
        .take_while(|line| !line.is_empty() && !line.starts_with(RESOLVER_START))
        .filter_map(name_server)
        .collect()
}

/// Reads `networksetup -getdnsservers`: one address per line, or a sentence
/// saying that none was set.
pub(crate) fn configured_name_servers(report: &str) -> Vec<IpAddr> {
    report
        .lines()
        .filter_map(|line| line.trim().parse().ok())
        .collect()
}

/// Reads `networksetup -getproxyautodiscovery`.
pub(crate) fn is_auto_discovery_on(report: &str) -> bool {
    value_of(report, AUTO_DISCOVERY) == Some(ON)
}

/// Reads what `networksetup` prints about one kind of proxy and returns
/// whether that proxy is in use.
pub(crate) fn is_proxy_enabled(report: &str) -> bool {
    value_of(report, ENABLED) == Some(YES)
}

/// Reads `dscacheutil -q host` and returns whether the name asked for has
/// an address. A name nobody knows prints nothing.
pub(crate) fn names_an_address(report: &str) -> bool {
    HOST_ADDRESSES
        .iter()
        .any(|name| value_of(report, name).is_some_and(|address| !address.is_empty()))
}

/// Reads `scutil --nc list`: one line per VPN profile, such as
/// `* (Connected)  <identifier> VPN (com.example.vpn) "Example VPN"  [VPN:…]`.
pub(crate) fn vpn_profiles(report: &str) -> VpnProfiles {
    let states: Vec<&str> = report
        .lines()
        .filter_map(|line| line.strip_prefix(ENABLED_PROFILE))
        .map(str::trim_start)
        .collect();
    VpnProfiles {
        installed: states.len(),
        connected: states
            .iter()
            .filter(|state| state.starts_with(CONNECTED))
            .count(),
    }
}

/// The value of the first line that reads `name: value`.
fn value_of<'a>(report: &'a str, name: &str) -> Option<&'a str> {
    report.lines().find_map(|line| {
        let (found, value) = line.split_once(NAME_END)?;
        (found.trim() == name).then_some(value.trim())
    })
}

/// Whether `text` can be the name of an interface: a few letters and digits
/// and nothing else, so it can never be read as an option.
fn is_interface_name(text: &str) -> bool {
    (1..=MAX_INTERFACE_NAME_LENGTH).contains(&text.len())
        && text.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

/// The device in a line like `(Hardware Port: Wi-Fi, Device: en0)`. A service
/// with no device of its own, such as a VPN, has an empty one.
fn device_of(line: &str) -> Option<&str> {
    let (_, device) = line
        .strip_prefix(HARDWARE_START)?
        .rsplit_once(DEVICE_START)?;
    let device = device.trim();
    Some(device.strip_suffix(')').unwrap_or(device).trim())
}

/// The name in a line like `(1) Wi-Fi`. The name is kept as printed, spaces
/// included.
fn service_name(line: &str) -> Option<&str> {
    let (position, name) = line.strip_prefix('(')?.split_once(") ")?;
    let is_position = position == DISABLED_SERVICE
        || (!position.is_empty() && position.bytes().all(|byte| byte.is_ascii_digit()));
    is_position.then_some(name)
}

/// The address in a line like `nameserver[0] : 192.0.2.53`.
fn name_server(line: &str) -> Option<IpAddr> {
    let (name, address) = line.split_once(NAME_END)?;
    name.starts_with(NAME_SERVER_START)
        .then(|| address.trim().parse().ok())
        .flatten()
}

#[cfg(test)]
mod tests {
    use std::net::IpAddr;

    use diskmarshal_domain::network::VpnProfiles;

    use super::{
        DefaultRoute, configured_name_servers, default_route, is_auto_discovery_on,
        is_proxy_enabled, name_servers_in_use, names_an_address, service_of, vpn_profiles,
    };

    /// The answers below have the form macOS 27 prints, with made-up names
    /// and addresses from the ranges reserved for documentation.
    const ROUTE: &str = "\
   route to: default
destination: default
       mask: default
    gateway: 192.168.0.1
  interface: en0
      flags: <UP,GATEWAY,DONE,STATIC,PRCLONING,GLOBAL>
 recvpipe  sendpipe  ssthresh  rtt,msec    rttvar  hopcount      mtu     expire
       0         0         0         0         0         0      1500         0
";

    const SERVICE_ORDER: &str = "\
An asterisk (*) denotes that a network service is disabled.
(1) USB 10/100/1000 LAN
(Hardware Port: USB 10/100/1000 LAN, Device: en7)

(2) Wi-Fi
(Hardware Port: Wi-Fi, Device: en0)

(3) Thunderbolt Bridge
(Hardware Port: Thunderbolt Bridge, Device: bridge0)

(*) Old Dock (2)
(Hardware Port: Old Dock, Device: en9)

(4) Example VPN
(Hardware Port: com.example.vpn, Device: )

";

    const RESOLVERS: &str = "\
DNS configuration

resolver #1
  search domain[0] : example.test
  nameserver[0] : 192.0.2.53
  nameserver[1] : 2001:db8::53
  nameserver[2] : fe80::1%en0
  flags    : Request A records
  reach    : 0x00000002 (Reachable)

resolver #2
  domain   : local
  options  : mdns
  nameserver[0] : 198.51.100.9
  timeout  : 5
  order    : 300000

DNS configuration (for scoped queries)

resolver #1
  nameserver[0] : 203.0.113.53
  if_index : 6 (en0)
  flags    : Scoped, Request A records
";

    const PROXY_OFF: &str = "\
Enabled: No
Server:
Port: 0
Authenticated Proxy Enabled: 0
";

    const VPN_LIST: &str = "\
Available network connection services in the current set (*=enabled):
* (Disconnected)   00000000-0000-4000-8000-000000000001 VPN (com.example.vpn) \"Example VPN\"                  [VPN:com.example.vpn]
* (Connected)      00000000-0000-4000-8000-000000000002 VPN (com.example.tunnel) \"Example Tunnel\"            [VPN:com.example.tunnel]
  (Disconnected)   00000000-0000-4000-8000-000000000003 VPN (com.example.old) \"Old (Connected)\"              [VPN:com.example.old]
* (Connecting)     00000000-0000-4000-8000-000000000004 IPSec \"Office (Connected)\"                           [IPSec]
";

    fn address(text: &str) -> IpAddr {
        text.parse().unwrap()
    }

    #[test]
    fn reads_the_interface_and_the_router_of_the_default_route() {
        assert_eq!(
            default_route(ROUTE),
            Some(DefaultRoute {
                interface: "en0".to_owned(),
                gateway: Some(address("192.168.0.1")),
            })
        );
    }

    #[test]
    fn a_route_through_a_link_without_a_router_has_no_gateway() {
        let report = "   route to: default\n    gateway: link#22\n  interface: utun4\n";
        assert_eq!(
            default_route(report),
            Some(DefaultRoute {
                interface: "utun4".to_owned(),
                gateway: None,
            })
        );
    }

    #[test]
    fn without_a_default_route_there_is_no_way_out() {
        assert_eq!(default_route(""), None);
        assert_eq!(default_route("   route to: default\n"), None);
        assert_eq!(
            default_route("route: writing to routing socket: not in table\n"),
            None
        );
    }

    #[test]
    fn refuses_an_interface_that_could_be_read_as_an_option() {
        for name in ["-v", "en0 -x", "", "averyveryverylongname0"] {
            let report = format!("    gateway: 192.168.0.1\n  interface: {name}\n");
            assert_eq!(default_route(&report), None, "{name:?}");
        }
    }

    #[test]
    fn finds_the_service_that_uses_an_interface() {
        let service = |interface| service_of(SERVICE_ORDER, interface);
        assert_eq!(service("en0").as_deref(), Some("Wi-Fi"));
        assert_eq!(service("en7").as_deref(), Some("USB 10/100/1000 LAN"));
        assert_eq!(service("bridge0").as_deref(), Some("Thunderbolt Bridge"));
        assert_eq!(
            service("en9").as_deref(),
            Some("Old Dock (2)"),
            "a service that is turned off still has its name"
        );
    }

    #[test]
    fn an_interface_no_service_uses_has_no_service() {
        assert_eq!(service_of(SERVICE_ORDER, "en5"), None);
        assert_eq!(service_of(SERVICE_ORDER, "en"), None);
        assert_eq!(
            service_of(SERVICE_ORDER, ""),
            None,
            "a service without a device is not the one of an unnamed interface"
        );
        assert_eq!(service_of("", "en0"), None);
    }

    #[test]
    fn a_device_line_without_a_readable_name_before_it_is_skipped() {
        let report = "\
(Hardware Port: Wi-Fi, Device: en0)
(one) Wi-Fi
(Hardware Port: Wi-Fi, Device: en0)
(2) Ethernet
no hardware line here
(Hardware Port: Ethernet, Device: en0)
";
        assert_eq!(service_of(report, "en0").as_deref(), Some("Ethernet"));
    }

    #[test]
    fn reads_the_name_servers_of_the_first_resolver_only() {
        assert_eq!(
            name_servers_in_use(RESOLVERS),
            [address("192.0.2.53"), address("2001:db8::53")],
            "in the order they are asked, without the one tied to an interface"
        );
    }

    #[test]
    fn a_first_resolver_without_name_servers_gives_none() {
        let report =
            "resolver #1\n  domain   : local\n\nresolver #2\n  nameserver[0] : 192.0.2.53\n";
        assert!(name_servers_in_use(report).is_empty());
        assert!(name_servers_in_use("").is_empty());
        assert!(name_servers_in_use("No DNS configuration available\n").is_empty());
    }

    #[test]
    fn reads_the_name_servers_set_by_hand() {
        assert_eq!(
            configured_name_servers("192.0.2.53\n2001:db8::53\n"),
            [address("192.0.2.53"), address("2001:db8::53")]
        );
    }

    #[test]
    fn a_service_with_no_name_server_set_by_hand_gives_none() {
        assert!(configured_name_servers("There aren't any DNS Servers set on Wi-Fi.\n").is_empty());
        assert!(configured_name_servers("").is_empty());
    }

    #[test]
    fn reads_whether_the_search_for_a_proxy_is_on() {
        assert!(is_auto_discovery_on("Auto Proxy Discovery: On\n"));
        assert!(!is_auto_discovery_on("Auto Proxy Discovery: Off\n"));
        assert!(!is_auto_discovery_on(""));
        assert!(!is_auto_discovery_on(
            "** Error: Unable to find item in network database.\n"
        ));
    }

    #[test]
    fn reads_whether_a_proxy_is_in_use() {
        assert!(!is_proxy_enabled(PROXY_OFF));
        assert!(is_proxy_enabled(
            "Enabled: Yes\nServer: proxy.example.test\nPort: 8080\nAuthenticated Proxy Enabled: 0\n"
        ));
        assert!(is_proxy_enabled(
            "URL: http://proxy.example.test/proxy.pac\nEnabled: Yes\n"
        ));
        assert!(!is_proxy_enabled("URL: (null)\nEnabled: No\n"));
        assert!(!is_proxy_enabled(""));
    }

    #[test]
    fn a_proxy_that_asks_for_a_password_is_not_thereby_in_use() {
        assert!(!is_proxy_enabled(
            "Enabled: No\nServer: \nPort: 0\nAuthenticated Proxy Enabled: Yes\n"
        ));
    }

    #[test]
    fn reads_whether_a_host_name_has_an_address() {
        assert!(names_an_address(
            "name: wpad.example.test\nip_address: 192.0.2.80\n\n"
        ));
        assert!(names_an_address(
            "name: wpad.example.test\nipv6_address: 2001:db8::80\n\n"
        ));
        assert!(!names_an_address(""));
        assert!(!names_an_address("name: wpad.example.test\n\n"));
        assert!(!names_an_address("name: wpad.example.test\nip_address: \n"));
    }

    #[test]
    fn counts_the_vpn_profiles_turned_on_and_the_connected_ones() {
        assert_eq!(
            vpn_profiles(VPN_LIST),
            VpnProfiles {
                installed: 3,
                connected: 1,
            },
            "a name that says connected does not make a profile connected"
        );
    }

    #[test]
    fn a_mac_without_vpn_profiles_has_none() {
        let header = "Available network connection services in the current set (*=enabled):\n";
        assert_eq!(vpn_profiles(header), VpnProfiles::default());
        assert_eq!(vpn_profiles(""), VpnProfiles::default());
    }
}
