//! Port for fingerprinting the content of files.

use std::io;
use std::path::Path;

use maccleaner_domain::duplicates::ContentHash;

/// Computes digests of file content.
pub trait ContentHasher: Send + Sync {
    /// A cheap digest of the beginning and the end of the file. Files with
    /// different quick digests are certainly different.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read.
    fn quick_hash(&self, path: &Path) -> io::Result<ContentHash>;

    /// A digest of the whole file.
    ///
    /// # Errors
    ///
    /// Fails when the file cannot be read.
    fn full_hash(&self, path: &Path) -> io::Result<ContentHash>;
}
