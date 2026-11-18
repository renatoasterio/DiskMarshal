//! Connection inspector that asks the programs macOS ships for this.

use std::process::Command;
use std::thread;
use std::time::Duration;

use diskmarshal_application::ports::{ConnectionInspector, InspectionError};
use diskmarshal_domain::network::{ConnectionProfile, ProxySettings, WifiLink, is_service_name};

use crate::network::airport_report;
use crate::network::settings_reports::{self, DefaultRoute};
use crate::process::bounded_command::{CommandFailure, CommandOutput, Stdout, run_bounded};

/// Absolute paths, so a different program earlier in `PATH` can never run.
const ROUTE: &str = "/sbin/route";
const NETWORKSETUP: &str = "/usr/sbin/networksetup";
const IPCONFIG: &str = "/usr/sbin/ipconfig";
const SCUTIL: &str = "/usr/sbin/scutil";
const DSCACHEUTIL: &str = "/usr/bin/dscacheutil";
const SYSTEM_PROFILER: &str = "/usr/sbin/system_profiler";

/// `-n` leaves addresses as numbers, so no name is looked up.
const DEFAULT_ROUTE: [&str; 3] = ["-n", "get", "default"];
const SERVICE_ORDER: [&str; 1] = ["-listnetworkserviceorder"];
const ADDRESS_OF: &str = "getifaddr";
const RESOLVERS: [&str; 1] = ["--dns"];
const NAME_SERVERS_OF: &str = "-getdnsservers";
const AUTO_DISCOVERY_OF: &str = "-getproxyautodiscovery";
/// One question for each kind of proxy a service can be given.
const PROXIES_OF: [&str; 4] = [
    "-getwebproxy",
    "-getsecurewebproxy",
    "-getsocksfirewallproxy",
    "-getautoproxyurl",
];
/// Looks up `wpad`, the name a network answers to when it has a proxy to
/// announce.
const DISCOVERY_LOOKUP: [&str; 5] = ["-q", "host", "-a", "name", "wpad"];
const VPN_PROFILES: [&str; 2] = ["--nc", "list"];
const WIRELESS_REPORT: [&str; 2] = ["SPAirPortDataType", "-json"];

/// Each of these programs answers in a fraction of a second.
const TIMEOUT: Duration = Duration::from_secs(5);
/// `system_profiler` listens for the networks in range before it answers,
/// which takes several seconds.
const WIRELESS_TIMEOUT: Duration = Duration::from_secs(30);
/// What `route` says when the routing table has no default route.
const NO_ROUTE: &str = "not in table";

/// Reads the network settings with `route`, `networksetup`, `ipconfig`,
/// `scutil`, `dscacheutil` and `system_profiler`.
///
/// Each program runs as the logged-in user, with no shell in between, a
/// cleared environment, no standard input and a time limit. Every question
/// only reads: no setting is changed. Nothing is sent to the network, with
/// one exception: when macOS is set to look for a proxy, the name it looks
/// for is looked up once, to see whether the network answers.
#[derive(Debug, Clone, Copy, Default)]
pub struct MacosConnectionInspector;

impl ConnectionInspector for MacosConnectionInspector {
    fn profile(&self) -> Result<ConnectionProfile, InspectionError> {
        let output = run(ROUTE, &DEFAULT_ROUTE, TIMEOUT).map_err(|failure| {
            InspectionError::Unavailable(match failure {
                CommandFailure::NotStarted(error) => error.to_string(),
                CommandFailure::TimedOut => "the routing table took too long".to_owned(),
            })
        })?;
        let route = route_of(&output)?;
        let interface = route.interface.as_str();

        Ok(thread::scope(|scope| {
            // By far the slowest question: it is asked while the others are.
            let wifi = scope.spawn(|| wifi_link(interface));
            let service = service_of(interface);
            let (configured_dns, proxy) = if service.is_empty() {
                Default::default()
            } else {
                let name_servers = answer(NETWORKSETUP, &[NAME_SERVERS_OF, &service], TIMEOUT)
                    .map(|report| settings_reports::configured_name_servers(&report));
                (name_servers.unwrap_or_default(), proxy_settings(&service))
            };
            ConnectionProfile {
                interface: interface.to_owned(),
                local_address: answer(IPCONFIG, &[ADDRESS_OF, interface], TIMEOUT)
                    .and_then(|address| address.trim().parse().ok()),
                gateway: route.gateway,
                dns_servers: answer(SCUTIL, &RESOLVERS, TIMEOUT)
                    .map(|report| settings_reports::name_servers_in_use(&report))
                    .unwrap_or_default(),
                configured_dns,
                proxy,
                vpn: answer(SCUTIL, &VPN_PROFILES, TIMEOUT)
                    .map(|report| settings_reports::vpn_profiles(&report))
                    .unwrap_or_default(),
                // A question that crashed is the same as one with no answer.
                wifi: wifi.join().ok().flatten(),
                service,
            }
        }))
    }
}

/// Runs one of the programs. All of them are asked things that only read.
fn run(
    program: &str,
    arguments: &[&str],
    timeout: Duration,
) -> Result<CommandOutput, CommandFailure> {
    let mut command = Command::new(program);
    command.args(arguments).env_clear();
    run_bounded(&mut command, Stdout::Capture, timeout)
}

/// What a program answered. `None` when it could not answer: the part of
/// the reading that depends on it is then left empty.
fn answer(program: &str, arguments: &[&str], timeout: Duration) -> Option<String> {
    let output = run(program, arguments, timeout).ok()?;
    (output.code == Some(0)).then(|| String::from_utf8_lossy(&output.stdout).into_owned())
}

/// Reads what `route` left behind. It ends normally when there is no default
/// route and says so as an error; anything else it could not do is unknown.
fn route_of(output: &CommandOutput) -> Result<DefaultRoute, InspectionError> {
    if let Some(route) = settings_reports::default_route(&String::from_utf8_lossy(&output.stdout)) {
        return Ok(route);
    }
    let has_no_route =
        output.code == Some(0) || String::from_utf8_lossy(&output.stderr).contains(NO_ROUTE);
    Err(if has_no_route {
        InspectionError::Offline
    } else {
        InspectionError::Unavailable(output.first_error_line())
    })
}

/// The name of the service that uses `interface`, or nothing when it cannot
/// be found. The name becomes an argument of the questions that follow, so
/// one that could be read as something else is left out as well.
fn service_of(interface: &str) -> String {
    answer(NETWORKSETUP, &SERVICE_ORDER, TIMEOUT)
        .and_then(|report| settings_reports::service_of(&report, interface))
        .filter(|name| is_service_name(name))
        .unwrap_or_default()
}

fn proxy_settings(service: &str) -> ProxySettings {
    let ask = |question: &str| answer(NETWORKSETUP, &[question, service], TIMEOUT);
    let is_auto_discovery_on = ask(AUTO_DISCOVERY_OF)
        .is_some_and(|report| settings_reports::is_auto_discovery_on(&report));
    ProxySettings {
        is_auto_discovery_on,
        is_discovery_answered: is_auto_discovery_on
            .then(|| answer(DSCACHEUTIL, &DISCOVERY_LOOKUP, TIMEOUT))
            .flatten()
            .map(|report| settings_reports::names_an_address(&report)),
        has_configured_proxy: PROXIES_OF.iter().any(|question| {
            ask(question).is_some_and(|report| settings_reports::is_proxy_enabled(&report))
        }),
    }
}

fn wifi_link(interface: &str) -> Option<WifiLink> {
    answer(SYSTEM_PROFILER, &WIRELESS_REPORT, WIRELESS_TIMEOUT)
        .and_then(|report| airport_report::wifi_link(&report, interface))
}

#[cfg(test)]
mod tests {
    use diskmarshal_application::ports::{ConnectionInspector, InspectionError};

    use super::{MacosConnectionInspector, route_of};
    use crate::process::bounded_command::CommandOutput;

    fn output(code: Option<i32>, stdout: &str, stderr: &str) -> CommandOutput {
        CommandOutput {
            code,
            stdout: stdout.as_bytes().to_vec(),
            stderr: stderr.as_bytes().to_vec(),
        }
    }

    #[test]
    fn reads_the_settings_of_this_machine_or_finds_it_offline() {
        let result = MacosConnectionInspector.profile();

        // A machine that builds this may have no network and no wireless
        // interface, so nothing is said about what was read.
        assert!(
            matches!(result, Ok(_) | Err(InspectionError::Offline)),
            "{:?}",
            result.err()
        );
    }

    #[test]
    fn a_default_route_is_read_whatever_else_the_program_said() {
        let printed = "    gateway: 192.168.0.1\n  interface: en0\n";
        let route = route_of(&output(Some(0), printed, "")).unwrap();
        assert_eq!(route.interface, "en0");
        assert_eq!(route.gateway, "192.168.0.1".parse().ok());
        assert!(route_of(&output(Some(1), printed, "route: something else\n")).is_ok());
    }

    #[test]
    fn no_default_route_means_this_mac_is_offline() {
        let not_found = "route: writing to routing socket: not in table\n";
        for code in [Some(0), Some(1)] {
            assert_eq!(
                route_of(&output(code, "", not_found)).unwrap_err(),
                InspectionError::Offline
            );
        }
        assert_eq!(
            route_of(&output(Some(0), "   route to: default\n", "")).unwrap_err(),
            InspectionError::Offline
        );
    }

    #[test]
    fn a_routing_table_that_cannot_be_asked_says_why() {
        let refused = "route: socket: Operation not permitted\n";
        assert_eq!(
            route_of(&output(Some(1), "", refused)).unwrap_err(),
            InspectionError::Unavailable("route: socket: Operation not permitted".to_owned())
        );
        assert_eq!(
            route_of(&output(None, "", "")).unwrap_err(),
            InspectionError::Unavailable("no message".to_owned())
        );
    }
}
