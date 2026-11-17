//! Port for reading the time on the user's clock.

use maccleaner_domain::monitoring::LocalTime;

/// Reads the current time of day in the user's time zone.
pub trait WallClock: Send + Sync {
    /// Returns the current local time.
    fn now(&self) -> LocalTime;
}
