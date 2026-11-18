//! Use case: find out how good the connection is. Changes nothing.

use std::net::IpAddr;
use std::thread;

use diskmarshal_domain::network::{DnsTiming, INTERNET_ANCHOR, NetworkReport, PUBLIC_RESOLVERS};

use crate::ports::{ConnectionInspector, InspectionError, ReachabilityProbe};

/// Name servers in use that are timed. The ones after these are spares
/// macOS rarely asks.
const TIMED_SERVERS_IN_USE: usize = 2;

/// Reads the settings of the connection and times the router, the internet
/// and the name servers.
pub struct AnalyzeNetwork<'a> {
    inspector: &'a dyn ConnectionInspector,
    probe: &'a dyn ReachabilityProbe,
}

impl<'a> AnalyzeNetwork<'a> {
    /// Creates the use case over the reader of the settings and the probe
    /// that times answers.
    #[must_use]
    pub fn new(inspector: &'a dyn ConnectionInspector, probe: &'a dyn ReachabilityProbe) -> Self {
        Self { inspector, probe }
    }

    /// Analyzes the connection. The router and the internet are pinged and
    /// the name servers are timed, all at the same time: the servers in
    /// use, and each public service that is not already in use.
    ///
    /// # Errors
    ///
    /// Fails when this Mac is not connected to a network or its settings
    /// cannot be read.
    pub fn execute(&self) -> Result<NetworkReport, InspectionError> {
        let profile = self.inspector.profile()?;
        let probe = self.probe;
        let targets = dns_targets(&profile.dns_servers);

        let (gateway, internet, dns) = thread::scope(|scope| {
            let gateway = profile
                .gateway
                .map(|router| scope.spawn(move || probe.ping(router)));
            let internet = scope.spawn(move || probe.ping(INTERNET_ANCHOR));
            // Side by side: a server that does not answer takes seconds.
            let timings: Vec<_> = targets
                .into_iter()
                .map(|(server, is_in_use)| {
                    (
                        server,
                        is_in_use,
                        scope.spawn(move || probe.time_dns(server)),
                    )
                })
                .collect();
            // A probe that crashed is the same as a probe that got no answer.
            let dns: Vec<DnsTiming> = timings
                .into_iter()
                .map(|(server, is_in_use, timing)| DnsTiming {
                    server,
                    is_in_use,
                    answer_time: timing.join().ok().flatten(),
                })
                .collect();
            (
                gateway.and_then(|pings| pings.join().ok()),
                internet.join().ok(),
                dns,
            )
        });
        Ok(NetworkReport {
            profile,
            gateway,
            internet,
            dns,
        })
    }
}

/// The name servers to time, each with whether it is in use: the first ones
/// in use, then the main address of every public service none of whose
/// addresses is in use.
fn dns_targets(in_use: &[IpAddr]) -> Vec<(IpAddr, bool)> {
    let public = PUBLIC_RESOLVERS.iter().filter_map(|resolver| {
        let addresses = resolver.servers.map(IpAddr::V4);
        let is_in_use = addresses.iter().any(|address| in_use.contains(address));
        (!is_in_use).then_some((addresses[0], false))
    });
    in_use
        .iter()
        .take(TIMED_SERVERS_IN_USE)
        .map(|server| (*server, true))
        .chain(public)
        .collect()
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::sync::Mutex;
    use std::time::Duration;

    use diskmarshal_domain::network::{ConnectionProfile, INTERNET_ANCHOR, PingResult};

    use super::AnalyzeNetwork;
    use crate::ports::{ConnectionInspector, InspectionError, ReachabilityProbe};

    const ROUTER: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1));
    const GOOGLE: IpAddr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));

    struct FixedInspector(Result<ConnectionProfile, InspectionError>);

    impl ConnectionInspector for FixedInspector {
        fn profile(&self) -> Result<ConnectionProfile, InspectionError> {
            self.0.clone()
        }
    }

    /// A probe that answers at once and remembers what it was asked.
    #[derive(Default)]
    struct RecordingProbe {
        pinged: Mutex<Vec<IpAddr>>,
        timed: Mutex<Vec<IpAddr>>,
    }

    impl ReachabilityProbe for RecordingProbe {
        fn ping(&self, target: IpAddr) -> PingResult {
            self.pinged.lock().unwrap().push(target);
            PingResult {
                sent: 10,
                received: if target == ROUTER { 0 } else { 10 },
                average: (target != ROUTER).then_some(Duration::from_millis(14)),
            }
        }

        fn time_dns(&self, server: IpAddr) -> Option<Duration> {
            self.timed.lock().unwrap().push(server);
            (server != GOOGLE).then_some(Duration::from_millis(9))
        }
    }

    fn profile(dns_servers: Vec<IpAddr>) -> ConnectionProfile {
        ConnectionProfile {
            interface: "en0".to_owned(),
            gateway: Some(ROUTER),
            dns_servers,
            ..ConnectionProfile::default()
        }
    }

    #[test]
    fn pings_the_router_and_the_internet_and_times_the_name_servers() {
        let inspector = FixedInspector(Ok(profile(vec![ROUTER])));
        let probe = RecordingProbe::default();

        let report = AnalyzeNetwork::new(&inspector, &probe).execute().unwrap();

        assert!(report.gateway.unwrap().is_silent());
        assert_eq!(report.internet.unwrap().received, 10);
        let mut pinged = probe.pinged.lock().unwrap().clone();
        pinged.sort();
        assert_eq!(pinged, [INTERNET_ANCHOR, ROUTER]);

        let asked: Vec<(IpAddr, bool)> = report
            .dns
            .iter()
            .map(|timing| (timing.server, timing.is_in_use))
            .collect();
        assert_eq!(asked[0], (ROUTER, true));
        assert_eq!(asked.len(), 4, "the one in use and three public services");
        assert!(asked[1..].iter().all(|(_, is_in_use)| !is_in_use));
        let google = report.dns.iter().find(|timing| timing.server == GOOGLE);
        assert_eq!(google.unwrap().answer_time, None);
    }

    #[test]
    fn a_public_service_already_in_use_is_not_timed_against_itself() {
        let spare_of_google = IpAddr::V4(Ipv4Addr::new(8, 8, 4, 4));
        let third = IpAddr::V4(Ipv4Addr::new(192, 0, 2, 53));
        let inspector = FixedInspector(Ok(profile(vec![spare_of_google, ROUTER, third])));
        let probe = RecordingProbe::default();

        let report = AnalyzeNetwork::new(&inspector, &probe).execute().unwrap();

        let timed = probe.timed.lock().unwrap().clone();
        assert!(!timed.contains(&GOOGLE));
        assert!(
            !timed.contains(&third),
            "only the first two in use are timed"
        );
        assert_eq!(
            report.dns.iter().filter(|timing| timing.is_in_use).count(),
            2
        );
        assert_eq!(report.dns.len(), 4);
    }

    #[test]
    fn without_a_router_only_the_internet_is_pinged() {
        let inspector = FixedInspector(Ok(ConnectionProfile::default()));
        let probe = RecordingProbe::default();

        let report = AnalyzeNetwork::new(&inspector, &probe).execute().unwrap();

        assert_eq!(report.gateway, None);
        assert_eq!(*probe.pinged.lock().unwrap(), [INTERNET_ANCHOR]);
    }

    #[test]
    fn says_why_the_connection_could_not_be_analyzed() {
        let inspector = FixedInspector(Err(InspectionError::Offline));
        let probe = RecordingProbe::default();

        let result = AnalyzeNetwork::new(&inspector, &probe).execute();

        assert_eq!(result.unwrap_err(), InspectionError::Offline);
        assert!(probe.pinged.lock().unwrap().is_empty(), "nothing is sent");
    }
}
