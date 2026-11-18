//! How careful the user should be before removing something.

/// How much thought a removal deserves.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RiskLevel {
    /// The data is recreated automatically. Selected by default.
    Safe,
    /// The data can be useful to keep. Never selected by default.
    Review,
}
