//! How network changes travel on the command line of the helper.
//!
//! After [`NETWORK_FLAG`] comes one group of words per change, the groups
//! separated by `--then`:
//!
//! - `flush-dns`
//! - `proxy-discovery on|off <service>`
//! - `dns <service> [<server>...]`, where no server means the servers the
//!   network hands out.
//!
//! No group carries a command or a path, and the helper checks every
//! service against the ones macOS lists before it changes anything.

use std::net::IpAddr;

use diskmarshal_domain::network::NetworkChange;

/// Argument that starts a request to change network settings.
pub const NETWORK_FLAG: &str = "--network";
/// Argument that separates one change from the next. A service can never
/// be named like this: a service name does not start with a hyphen.
const NEXT_CHANGE: &str = "--then";
const FLUSH_DNS: &str = "flush-dns";
const PROXY_DISCOVERY: &str = "proxy-discovery";
const DNS: &str = "dns";
const ON: &str = "on";
const OFF: &str = "off";
/// Most changes one request may carry.
const MAX_CHANGES: usize = 8;

/// Writes `changes` as the words that follow [`NETWORK_FLAG`].
#[must_use]
pub(crate) fn encode_network_changes(changes: &[NetworkChange]) -> Vec<String> {
    let mut words = Vec::new();
    for (index, change) in changes.iter().enumerate() {
        if index > 0 {
            words.push(NEXT_CHANGE.to_owned());
        }
        match change {
            NetworkChange::FlushDnsCache => words.push(FLUSH_DNS.to_owned()),
            NetworkChange::SetProxyAutoDiscovery {
                service,
                is_enabled,
            } => {
                let state = if *is_enabled { ON } else { OFF };
                words.extend([
                    PROXY_DISCOVERY.to_owned(),
                    state.to_owned(),
                    service.clone(),
                ]);
            }
            NetworkChange::SetDnsServers { service, servers } => {
                words.extend([DNS.to_owned(), service.clone()]);
                words.extend(servers.iter().map(ToString::to_string));
            }
        }
    }
    words
}

/// Reads the words that follow [`NETWORK_FLAG`].
///
/// # Errors
///
/// Fails, naming nothing the caller sent, when a group is not one of the
/// three forms, when a change is malformed, or when there are no changes
/// or too many.
pub(crate) fn parse_network_changes(words: &[String]) -> Result<Vec<NetworkChange>, &'static str> {
    const NOT_A_CHANGE: &str = "argument is not a network change";
    let mut changes = Vec::new();
    for group in words.split(|word| word == NEXT_CHANGE) {
        let (kind, details) = group.split_first().ok_or(NOT_A_CHANGE)?;
        let change = match (kind.as_str(), details) {
            (FLUSH_DNS, []) => NetworkChange::FlushDnsCache,
            (PROXY_DISCOVERY, [state, service]) => NetworkChange::SetProxyAutoDiscovery {
                service: service.clone(),
                is_enabled: match state.as_str() {
                    ON => true,
                    OFF => false,
                    _ => return Err(NOT_A_CHANGE),
                },
            },
            (DNS, [service, servers @ ..]) => NetworkChange::SetDnsServers {
                service: service.clone(),
                servers: servers
                    .iter()
                    .map(|server| server.parse::<IpAddr>())
                    .collect::<Result<_, _>>()
                    .map_err(|_| "argument is not an address")?,
            },
            _ => return Err(NOT_A_CHANGE),
        };
        if !change.is_well_formed() {
            return Err(NOT_A_CHANGE);
        }
        changes.push(change);
    }
    if changes.len() > MAX_CHANGES {
        return Err("too many network changes");
    }
    Ok(changes)
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};

    use diskmarshal_domain::network::NetworkChange;

    use super::{encode_network_changes, parse_network_changes};

    fn words(text: &[&str]) -> Vec<String> {
        text.iter().map(|word| (*word).to_owned()).collect()
    }

    fn every_kind() -> Vec<NetworkChange> {
        vec![
            NetworkChange::SetProxyAutoDiscovery {
                service: "Thunderbolt Bridge".to_owned(),
                is_enabled: false,
            },
            NetworkChange::FlushDnsCache,
            NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: vec![
                    IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1)),
                    IpAddr::V6(Ipv6Addr::new(0x2001, 0xdb8, 0, 0, 0, 0, 0, 0x53)),
                ],
            },
            NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: Vec::new(),
            },
        ]
    }

    #[test]
    fn changes_survive_the_trip_from_the_app_to_the_helper() {
        let changes = every_kind();

        let sent = encode_network_changes(&changes);

        assert_eq!(
            sent,
            words(&[
                "proxy-discovery",
                "off",
                "Thunderbolt Bridge",
                "--then",
                "flush-dns",
                "--then",
                "dns",
                "Wi-Fi",
                "1.1.1.1",
                "2001:db8::53",
                "--then",
                "dns",
                "Wi-Fi",
            ])
        );
        assert_eq!(parse_network_changes(&sent), Ok(changes));
    }

    #[test]
    fn refuses_anything_that_is_not_one_of_the_three_forms() {
        for request in [
            &[][..],
            &["flush-dns", "now"],
            &["flush-dns", "--then"],
            &["--then", "flush-dns"],
            &["proxy-discovery", "maybe", "Wi-Fi"],
            &["proxy-discovery", "off"],
            &["proxy-discovery", "off", "--force"],
            &["dns"],
            &["dns", "Wi-Fi", "not-an-address"],
            &["dns", "Wi-Fi", "1.1.1.1; rm -rf /"],
            &["dns", "-setwebproxy", "1.1.1.1"],
            &[
                "dns", "Wi-Fi", "1.1.1.1", "1.0.0.1", "8.8.8.8", "8.8.4.4", "9.9.9.9",
            ],
            &["rm", "-rf", "/"],
            &["/usr/sbin/networksetup"],
        ] {
            assert!(
                parse_network_changes(&words(request)).is_err(),
                "{request:?}"
            );
        }
    }

    #[test]
    fn refuses_more_changes_than_one_request_may_carry() {
        let many = vec![NetworkChange::FlushDnsCache; 9];
        assert!(parse_network_changes(&encode_network_changes(&many)).is_err());
        let most = vec![NetworkChange::FlushDnsCache; 8];
        assert_eq!(
            parse_network_changes(&encode_network_changes(&most)).map(|changes| changes.len()),
            Ok(8)
        );
    }
}
