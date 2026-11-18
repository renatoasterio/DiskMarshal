//! Everything the analysis of the connection found, and what follows from it.

use std::time::Duration;

use crate::cleanup::RiskLevel;
use crate::network::{
    Advice, ConnectionProfile, DnsTiming, Finding, NetworkChange, Optimization, OptimizationKind,
    PingResult, PublicResolver, WifiBand,
};

/// From this answer time up the name server in use may be called slow.
const SLOW_DNS_FROM: Duration = Duration::from_millis(20);
/// How many times slower than the fastest public server the one in use has
/// to be before switching is worth offering.
const SLOW_DNS_FACTOR: u32 = 2;
/// From this answer time up the internet is slow to answer.
const SLOW_ANSWER_FROM: Duration = Duration::from_millis(100);
/// From this many other networks on the channel up, the channel is crowded.
const CROWDED_FROM_NEIGHBORS: usize = 2;

/// The state of the connection at the moment it was analyzed.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NetworkReport {
    /// How this Mac is set up to reach the internet.
    pub profile: ConnectionProfile,
    /// Pings to the router. `None` when there is no router to ping.
    pub gateway: Option<PingResult>,
    /// Pings to the internet. `None` when they were not sent.
    pub internet: Option<PingResult>,
    /// Answer times of the name servers in use and of the public ones.
    pub dns: Vec<DnsTiming>,
}

impl NetworkReport {
    /// Answer time of the fastest name server in use that answered.
    #[must_use]
    pub fn current_dns_time(&self) -> Option<Duration> {
        self.dns
            .iter()
            .filter(|timing| timing.is_in_use)
            .filter_map(|timing| timing.answer_time)
            .min()
    }

    /// The fastest public name service that is not in use, with its answer
    /// time.
    #[must_use]
    pub fn fastest_public_dns(&self) -> Option<(&'static PublicResolver, Duration)> {
        self.dns
            .iter()
            .filter(|timing| !timing.is_in_use)
            .filter_map(|timing| {
                Some((PublicResolver::serving(timing.server)?, timing.answer_time?))
            })
            .min_by_key(|(_, time)| *time)
    }

    /// The server in use and a public one that answers much faster, when
    /// the difference is worth a change.
    fn slower_dns(&self) -> Option<(Duration, &'static PublicResolver, Duration)> {
        let current = self.current_dns_time()?;
        let (resolver, fastest) = self.fastest_public_dns()?;
        (current >= SLOW_DNS_FROM && current >= fastest * SLOW_DNS_FACTOR)
            .then_some((current, resolver, fastest))
    }

    /// What the analysis concluded, what asks for attention first.
    #[must_use]
    pub fn findings(&self) -> Vec<Finding> {
        let mut findings = Vec::new();
        let proxy = self.profile.proxy;
        if proxy.is_auto_discovery_on {
            findings.push(if proxy.is_discovery_answered == Some(false) {
                Finding::ProxyDiscoveryUnanswered
            } else {
                Finding::ProxyDiscoveryOn
            });
        }
        if let Some((current, _, fastest)) = self.slower_dns() {
            findings.push(Finding::SlowDns { current, fastest });
        }
        if let Some(internet) = self.internet.filter(|ping| ping.sent > 0) {
            let lost = internet.sent.saturating_sub(internet.received);
            findings.push(if lost == 0 {
                Finding::NoPacketLoss {
                    sent: internet.sent,
                }
            } else {
                Finding::PacketLoss {
                    sent: internet.sent,
                    lost,
                }
            });
            if let Some(latency) = internet.average.filter(|time| *time >= SLOW_ANSWER_FROM) {
                findings.push(Finding::SlowAnswer { latency });
            }
        }
        if let Some(wifi) = &self.profile.wifi {
            findings.push(Finding::WifiSignal {
                quality: wifi.quality(),
                signal_dbm: wifi.signal_dbm,
                band: wifi.band,
                generation: wifi.generation(),
            });
        }
        if self.profile.vpn.connected > 0 {
            findings.push(Finding::VpnConnected);
        }
        findings.sort_by_key(Finding::severity);
        findings
    }

    /// What would help and only the user can do.
    #[must_use]
    pub fn advice(&self) -> Vec<Advice> {
        let mut advice = Vec::new();
        if let Some(wifi) = &self.profile.wifi {
            if wifi.neighbors_on_channel >= CROWDED_FROM_NEIGHBORS {
                advice.push(Advice::CrowdedChannel {
                    channel: wifi.channel,
                    neighbors: wifi.neighbors_on_channel,
                });
            }
            if wifi.band == Some(WifiBand::TwoPointFourGigahertz) {
                advice.push(Advice::SlowBand);
            }
        }
        if self.profile.vpn.installed > 0 {
            advice.push(Advice::VpnProfiles {
                installed: self.profile.vpn.installed,
            });
        }
        if self.profile.wifi.is_some() {
            advice.push(Advice::PreferCable);
        }
        advice
    }

    /// The changes worth offering for this connection. Without the name of
    /// the network service nothing that names one can be offered.
    #[must_use]
    pub fn optimizations(&self) -> Vec<Optimization> {
        let profile = &self.profile;
        let service = (!profile.service.is_empty()).then(|| profile.service.clone());
        let mut optimizations = Vec::new();

        let proxy = profile.proxy;
        if let Some(service) = service
            .clone()
            .filter(|_| proxy.is_auto_discovery_on && !proxy.has_configured_proxy)
        {
            // Safe only when the network was seen not to answer: where a
            // proxy does exist, turning the search off cuts the connection.
            let is_unanswered = proxy.is_discovery_answered == Some(false);
            optimizations.push(Optimization {
                kind: OptimizationKind::DisableProxyDiscovery,
                change: NetworkChange::SetProxyAutoDiscovery {
                    service: service.clone(),
                    is_enabled: false,
                },
                undo: Some(NetworkChange::SetProxyAutoDiscovery {
                    service,
                    is_enabled: true,
                }),
                risk: if is_unanswered {
                    RiskLevel::Safe
                } else {
                    RiskLevel::Review
                },
                is_recommended: is_unanswered,
            });
        }

        optimizations.push(Optimization {
            kind: OptimizationKind::FlushDnsCache,
            change: NetworkChange::FlushDnsCache,
            undo: None,
            risk: RiskLevel::Safe,
            is_recommended: true,
        });

        if let (Some(service), Some((current, resolver, fastest))) = (service, self.slower_dns()) {
            optimizations.push(Optimization {
                kind: OptimizationKind::UseFasterDns {
                    provider: resolver.provider,
                    current,
                    fastest,
                },
                change: NetworkChange::SetDnsServers {
                    service: service.clone(),
                    servers: resolver.servers.iter().copied().map(Into::into).collect(),
                },
                undo: Some(NetworkChange::SetDnsServers {
                    service,
                    servers: profile.configured_dns.clone(),
                }),
                // The server in use may filter content or know names that
                // exist only inside the network. A public one does neither.
                risk: RiskLevel::Review,
                is_recommended: false,
            });
        }
        optimizations
    }
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};
    use std::time::Duration;

    use super::NetworkReport;
    use crate::cleanup::RiskLevel;
    use crate::network::{
        Advice, BitRate, ConnectionProfile, DnsTiming, Finding, NetworkChange, OptimizationKind,
        PingResult, ProxySettings, Severity, SignalQuality, VpnProfiles, WifiBand, WifiLink,
    };

    const ROUTER: IpAddr = IpAddr::V4(Ipv4Addr::new(192, 168, 0, 1));
    const CLOUDFLARE: IpAddr = IpAddr::V4(Ipv4Addr::new(1, 1, 1, 1));
    const GOOGLE: IpAddr = IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8));

    fn timing(server: IpAddr, is_in_use: bool, millis: Option<u64>) -> DnsTiming {
        DnsTiming {
            server,
            is_in_use,
            answer_time: millis.map(Duration::from_millis),
        }
    }

    fn wifi(signal_dbm: i32, band: WifiBand, neighbors_on_channel: usize) -> WifiLink {
        WifiLink {
            signal_dbm,
            noise_dbm: -93,
            channel: 44,
            band: Some(band),
            width_mhz: Some(80),
            standard: "802.11ax".to_owned(),
            transmit_rate: BitRate::from_bits_per_second(1_201_000_000),
            neighbors_on_channel,
        }
    }

    /// A connection with everything the analysis can complain about.
    fn troubled() -> NetworkReport {
        NetworkReport {
            profile: ConnectionProfile {
                interface: "en0".to_owned(),
                service: "Wi-Fi".to_owned(),
                gateway: Some(ROUTER),
                dns_servers: vec![ROUTER],
                proxy: ProxySettings {
                    is_auto_discovery_on: true,
                    is_discovery_answered: Some(false),
                    has_configured_proxy: false,
                },
                vpn: VpnProfiles {
                    installed: 3,
                    connected: 0,
                },
                wifi: Some(wifi(-52, WifiBand::FiveGigahertz, 3)),
                ..ConnectionProfile::default()
            },
            gateway: Some(PingResult {
                sent: 10,
                received: 0,
                average: None,
            }),
            internet: Some(PingResult {
                sent: 10,
                received: 10,
                average: Some(Duration::from_millis(14)),
            }),
            dns: vec![
                timing(ROUTER, true, Some(38)),
                timing(GOOGLE, false, Some(12)),
                timing(CLOUDFLARE, false, Some(9)),
            ],
        }
    }

    #[test]
    fn lists_what_asks_for_attention_first() {
        let findings = troubled().findings();

        assert_eq!(
            findings,
            [
                Finding::ProxyDiscoveryUnanswered,
                Finding::SlowDns {
                    current: Duration::from_millis(38),
                    fastest: Duration::from_millis(9),
                },
                Finding::NoPacketLoss { sent: 10 },
                Finding::WifiSignal {
                    quality: SignalQuality::Strong,
                    signal_dbm: -52,
                    band: Some(WifiBand::FiveGigahertz),
                    generation: Some("Wi-Fi 6"),
                },
            ]
        );
        assert_eq!(findings[0].severity(), Severity::Attention);
    }

    #[test]
    fn reports_loss_a_slow_answer_and_a_connected_vpn() {
        let mut report = troubled();
        report.internet = Some(PingResult {
            sent: 10,
            received: 7,
            average: Some(Duration::from_millis(180)),
        });
        report.profile.vpn.connected = 1;
        report.profile.proxy.is_discovery_answered = None;
        let findings = report.findings();

        assert!(findings.contains(&Finding::PacketLoss { sent: 10, lost: 3 }));
        assert!(findings.contains(&Finding::SlowAnswer {
            latency: Duration::from_millis(180)
        }));
        assert!(findings.contains(&Finding::VpnConnected));
        assert!(findings.contains(&Finding::ProxyDiscoveryOn));
    }

    #[test]
    fn a_name_server_is_slow_only_against_a_much_faster_public_one() {
        let mut report = troubled();
        assert_eq!(report.current_dns_time(), Some(Duration::from_millis(38)));
        assert_eq!(
            report.fastest_public_dns().unwrap().0.provider,
            "Cloudflare"
        );

        report.dns = vec![
            timing(ROUTER, true, Some(15)),
            timing(CLOUDFLARE, false, Some(2)),
        ];
        assert!(report.slower_dns().is_none(), "fast enough already");
        report.dns = vec![
            timing(ROUTER, true, Some(30)),
            timing(CLOUDFLARE, false, Some(20)),
        ];
        assert!(report.slower_dns().is_none(), "not twice as slow");
        report.dns = vec![
            timing(ROUTER, true, None),
            timing(CLOUDFLARE, false, Some(9)),
        ];
        assert!(
            report.slower_dns().is_none(),
            "the one in use did not answer"
        );
        report.dns = vec![
            timing(ROUTER, true, Some(38)),
            timing(CLOUDFLARE, false, None),
        ];
        assert!(report.slower_dns().is_none(), "nothing to compare with");
    }

    #[test]
    fn offers_the_three_changes_with_the_way_back_of_each() {
        let optimizations = troubled().optimizations();

        let kinds: Vec<_> = optimizations.iter().map(|offer| &offer.kind).collect();
        assert_eq!(
            kinds,
            [
                &OptimizationKind::DisableProxyDiscovery,
                &OptimizationKind::FlushDnsCache,
                &OptimizationKind::UseFasterDns {
                    provider: "Cloudflare",
                    current: Duration::from_millis(38),
                    fastest: Duration::from_millis(9),
                },
            ]
        );
        let selected: Vec<bool> = optimizations
            .iter()
            .map(|offer| offer.is_recommended)
            .collect();
        assert_eq!(selected, [true, true, false]);
        assert_eq!(optimizations[2].risk, RiskLevel::Review);
        assert_eq!(
            optimizations[2].change,
            NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: vec![CLOUDFLARE, IpAddr::V4(Ipv4Addr::new(1, 0, 0, 1))],
            }
        );
        assert_eq!(
            optimizations[2].undo,
            Some(NetworkChange::SetDnsServers {
                service: "Wi-Fi".to_owned(),
                servers: Vec::new(),
            }),
            "back to the servers the network hands out"
        );
        assert!(optimizations[1].undo.is_none());
        assert!(
            optimizations
                .iter()
                .all(|offer| offer.change.is_well_formed())
        );
    }

    #[test]
    fn turning_the_proxy_search_off_is_safe_only_when_nothing_answers() {
        let mut report = troubled();
        report.profile.proxy.is_discovery_answered = Some(true);
        let offer = &report.optimizations()[0];
        assert_eq!(offer.kind, OptimizationKind::DisableProxyDiscovery);
        assert_eq!(offer.risk, RiskLevel::Review);
        assert!(!offer.is_recommended);

        report.profile.proxy.has_configured_proxy = true;
        assert_eq!(
            report.optimizations()[0].kind,
            OptimizationKind::FlushDnsCache,
            "a proxy set on purpose is left alone"
        );
    }

    #[test]
    fn without_the_service_name_only_the_cache_can_be_flushed() {
        let mut report = troubled();
        report.profile.service = String::new();
        let optimizations = report.optimizations();
        assert_eq!(optimizations.len(), 1);
        assert_eq!(optimizations[0].kind, OptimizationKind::FlushDnsCache);
    }

    #[test]
    fn advises_on_what_only_the_user_can_change() {
        assert_eq!(
            troubled().advice(),
            [
                Advice::CrowdedChannel {
                    channel: 44,
                    neighbors: 3
                },
                Advice::VpnProfiles { installed: 3 },
                Advice::PreferCable,
            ]
        );

        let mut slow = troubled();
        slow.profile.wifi = Some(wifi(-75, WifiBand::TwoPointFourGigahertz, 0));
        slow.profile.vpn.installed = 0;
        assert_eq!(slow.advice(), [Advice::SlowBand, Advice::PreferCable]);

        let wired = NetworkReport::default();
        assert!(wired.advice().is_empty());
        assert!(wired.findings().is_empty());
    }
}
