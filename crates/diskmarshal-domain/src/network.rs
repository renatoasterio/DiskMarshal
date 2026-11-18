//! What crosses the network of this Mac: which programs send and receive,
//! through which connections and how fast; how good the connection is; and
//! what can be changed to make it faster.

mod advice;
mod bit_rate;
mod connection;
mod connection_profile;
mod finding;
mod network_change;
mod network_report;
mod optimization;
mod program_name;
mod reachability;
mod speed_test;
mod traffic_meter;
mod traffic_reading;
mod traffic_snapshot;
mod wifi_link;

pub use advice::Advice;
pub use bit_rate::BitRate;
pub use connection::{Connection, Endpoint, Transport};
pub use connection_profile::{ConnectionProfile, ProxySettings, VpnProfiles};
pub use finding::{Direction, Finding, Severity};
pub use network_change::{NetworkChange, is_service_name};
pub use network_report::NetworkReport;
pub use optimization::{Optimization, OptimizationKind};
pub use program_name::program_name;
pub use reachability::{DnsTiming, INTERNET_ANCHOR, PUBLIC_RESOLVERS, PingResult, PublicResolver};
pub use speed_test::{Responsiveness, ResponsivenessLevel, SpeedTest};
pub use traffic_meter::TrafficMeter;
pub use traffic_reading::{ProcessTraffic, TrafficReading};
pub use traffic_snapshot::{ProgramTraffic, TrafficSnapshot};
pub use wifi_link::{SignalQuality, WifiBand, WifiLink};
