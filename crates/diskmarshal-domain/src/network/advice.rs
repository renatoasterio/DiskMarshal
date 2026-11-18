//! What would help the connection and only the user can do.

/// A suggestion the app cannot carry out by itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Advice {
    /// Other networks in range share the channel: another channel, chosen
    /// on the router, would be quieter.
    CrowdedChannel {
        /// The channel in use.
        channel: u32,
        /// How many other networks use it.
        neighbors: usize,
    },
    /// The link is on the 2,4 GHz band: the 5 GHz network of the same
    /// router carries much more.
    SlowBand,
    /// VPN profiles are installed: a connected VPN usually costs speed.
    VpnProfiles {
        /// How many profiles exist.
        installed: usize,
    },
    /// The Mac is on a wireless link: a cable is steadier for calls and games.
    PreferCable,
}
