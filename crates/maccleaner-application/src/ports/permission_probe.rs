//! Port for finding out what macOS lets this app read.

/// Whether macOS lets the app do something.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum AccessStatus {
    /// The app can.
    Granted,
    /// macOS refuses.
    Denied,
    /// There was nothing to test with, so nobody knows.
    #[default]
    Unknown,
}

/// Finds out which permissions the user gave the app.
pub trait PermissionProbe: Send + Sync {
    /// Whether the app has Full Disk Access. Without it, macOS hides parts
    /// of the home folder, such as Mail and Safari data, from every scan.
    fn full_disk_access(&self) -> AccessStatus;
}
