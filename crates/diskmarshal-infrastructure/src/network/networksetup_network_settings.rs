//! Network settings changed through the programs macOS ships for that.

use std::process::Command;
use std::time::Duration;

use diskmarshal_application::ports::{NetworkSettings, NetworkSettingsError};
use diskmarshal_domain::network::NetworkChange;

use crate::process::bounded_command::{CommandFailure, CommandOutput, Stdout, run_bounded};

/// Absolute paths, so a different program earlier in `PATH` can never run.
const NETWORKSETUP: &str = "/usr/sbin/networksetup";
const DSCACHEUTIL: &str = "/usr/bin/dscacheutil";
const KILLALL: &str = "/usr/bin/killall";
/// The service that answers name lookups. The hang-up signal makes it drop
/// what it remembers, without stopping it.
const NAME_SERVICE: &str = "mDNSResponder";
/// What `networksetup` takes in place of a list of servers to go back to
/// the ones the network hands out.
const NO_SERVERS: &str = "empty";
/// How `networksetup` marks a disabled service in its list.
const DISABLED_MARK: char = '*';
const TIMEOUT: Duration = Duration::from_secs(15);

/// Changes the settings with `/usr/sbin/networksetup`, and empties the name
/// cache with `/usr/bin/dscacheutil` and a signal to `mDNSResponder`.
///
/// Every program is named by an absolute path and receives an argument
/// list, a cleared environment and no standard input. Most changes need an
/// administrator, so this is what the privileged helper plugs in.
#[derive(Debug, Clone, Copy, Default)]
pub struct NetworksetupNetworkSettings;

impl NetworkSettings for NetworksetupNetworkSettings {
    fn services(&self) -> Result<Vec<String>, NetworkSettingsError> {
        let listing = run(NETWORKSETUP, &["-listallnetworkservices"])?;
        Ok(parse_services(&String::from_utf8_lossy(&listing.stdout)))
    }

    fn apply(&self, change: &NetworkChange) -> Result<(), NetworkSettingsError> {
        match change {
            NetworkChange::FlushDnsCache => {
                run(DSCACHEUTIL, &["-flushcache"])?;
                run(KILLALL, &["-HUP", NAME_SERVICE])?;
            }
            NetworkChange::SetProxyAutoDiscovery {
                service,
                is_enabled,
            } => {
                let state = if *is_enabled { "on" } else { "off" };
                run(NETWORKSETUP, &["-setproxyautodiscovery", service, state])?;
            }
            NetworkChange::SetDnsServers { service, servers } => {
                let servers: Vec<String> = servers.iter().map(ToString::to_string).collect();
                let mut arguments = vec!["-setdnsservers", service.as_str()];
                if servers.is_empty() {
                    arguments.push(NO_SERVERS);
                } else {
                    arguments.extend(servers.iter().map(String::as_str));
                }
                run(NETWORKSETUP, &arguments)?;
            }
        }
        Ok(())
    }
}

/// Runs one program to its end and fails unless it reports success.
fn run(program: &str, arguments: &[&str]) -> Result<CommandOutput, NetworkSettingsError> {
    let mut command = Command::new(program);
    command.args(arguments).env_clear();
    let output = run_bounded(&mut command, Stdout::Capture, TIMEOUT).map_err(|failure| {
        NetworkSettingsError(match failure {
            CommandFailure::NotStarted(error) => error.to_string(),
            CommandFailure::TimedOut => "macOS took too long to answer".to_owned(),
        })
    })?;
    if output.code == Some(0) {
        return Ok(output);
    }
    // `networksetup` explains a refusal on its standard output.
    let printed = String::from_utf8_lossy(&output.stdout);
    let reason = printed
        .lines()
        .map(str::trim)
        .find(|line| !line.is_empty())
        .map_or_else(|| output.first_error_line(), str::to_owned);
    Err(NetworkSettingsError(reason))
}

/// Reads the list of services: one per line after a first line of
/// explanation, a disabled one marked with an asterisk.
fn parse_services(listing: &str) -> Vec<String> {
    listing
        .lines()
        .skip(1)
        .map(|line| line.strip_prefix(DISABLED_MARK).unwrap_or(line).trim())
        .filter(|name| !name.is_empty())
        .map(str::to_owned)
        .collect()
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::NetworkSettings;

    use super::{NetworksetupNetworkSettings, parse_services};

    #[test]
    fn reads_the_services_and_the_disabled_ones_too() {
        let listing = "An asterisk (*) denotes that a network service is disabled.\n\
Wi-Fi\n\
Thunderbolt Bridge\n\
*USB 10/100/1000 LAN\n\
\n\
Example VPN\n";

        assert_eq!(
            parse_services(listing),
            [
                "Wi-Fi",
                "Thunderbolt Bridge",
                "USB 10/100/1000 LAN",
                "Example VPN"
            ]
        );
        assert!(parse_services("").is_empty());
    }

    #[test]
    fn lists_the_services_of_this_machine() {
        let services = NetworksetupNetworkSettings.services().unwrap();
        assert!(services.iter().all(|name| !name.trim().is_empty()));
    }
}
