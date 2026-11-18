//! Adapters that read, measure and change how this Mac uses the network.
//!
//! Every one of them asks a program that ships with macOS. Reading the
//! traffic and the settings uses counters and settings the system already
//! keeps: no packet is captured. Timing answers and testing the speed use
//! the network, and run only when the user asks.

mod airport_report;
mod macos_connection_inspector;
mod nettop_report;
mod nettop_traffic_probe;
mod network_quality_speed_tester;
mod networksetup_network_settings;
mod ping_reachability_probe;
mod program_names;
mod settings_reports;

pub use macos_connection_inspector::MacosConnectionInspector;
pub use nettop_traffic_probe::NettopTrafficProbe;
pub use network_quality_speed_tester::NetworkQualitySpeedTester;
pub use networksetup_network_settings::NetworksetupNetworkSettings;
pub use ping_reachability_probe::PingReachabilityProbe;
