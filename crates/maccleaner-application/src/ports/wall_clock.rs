//! Port for reading the time on the user's clock.

use maccleaner_domain::monitoring::LocalTime;

/// Reads the current time.
pub trait WallClock: Send + Sync {
    /// Returns the current local time.
    fn now(&self) -> LocalTime;

    /// Returns the current time in seconds since the Unix epoch.
    fn unix_seconds(&self) -> i64;
}
