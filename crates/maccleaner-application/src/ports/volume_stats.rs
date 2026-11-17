//! Port for reading how full a volume is.

use std::path::Path;

use maccleaner_domain::monitoring::VolumeUsage;

use crate::ApplicationError;

/// Reads the capacity and free space of the volume that holds a path.
pub trait VolumeStats: Send + Sync {
    /// Returns the usage of the volume `path` is on.
    ///
    /// # Errors
    ///
    /// Fails when the operating system cannot describe that volume.
    fn usage(&self, path: &Path) -> Result<VolumeUsage, ApplicationError>;
}
