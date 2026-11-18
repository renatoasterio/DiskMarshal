//! A change the app offers to make the connection faster.

use std::time::Duration;

use crate::cleanup::RiskLevel;
use crate::network::NetworkChange;

/// What an optimization does, with the figures that justify it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OptimizationKind {
    /// Stop looking for a proxy before each new connection.
    DisableProxyDiscovery,
    /// Make macOS look names up again instead of using what it remembers.
    FlushDnsCache,
    /// Switch to the fastest public name service measured.
    UseFasterDns {
        /// Who runs that service.
        provider: &'static str,
        /// Answer time of the server in use.
        current: Duration,
        /// Answer time of the faster one.
        fastest: Duration,
    },
}

/// One change offered to the user, never applied without a confirmation.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Optimization {
    /// What it does and why.
    pub kind: OptimizationKind,
    /// The change to the settings.
    pub change: NetworkChange,
    /// The change that takes it back. `None` when there is nothing to take
    /// back.
    pub undo: Option<NetworkChange>,
    /// How careful to be before applying it.
    pub risk: RiskLevel,
    /// Whether it starts selected. Only changes that cannot take anything
    /// away from the user do.
    pub is_recommended: bool,
}
