//! Failures a use case can report.

use std::io;
use std::path::PathBuf;

/// Everything that can go wrong while running a use case.
#[derive(Debug, thiserror::Error)]
pub enum ApplicationError {
    /// The folder to scan does not exist.
    #[error("path not found: {}", .0.display())]
    RootNotFound(PathBuf),

    /// The path to scan exists but is not a folder.
    #[error("not a directory: {}", .0.display())]
    RootNotDirectory(PathBuf),

    /// The user cancelled the operation before it finished.
    #[error("operation cancelled")]
    Cancelled,

    /// The scan found more items than a tree can hold.
    #[error("too many items to hold in one scan")]
    TreeCapacityExceeded,

    /// The operating system refused an operation on `path`.
    #[error("i/o failure at {}: {source}", .path.display())]
    Io {
        /// Where the failure happened.
        path: PathBuf,
        /// What the operating system reported.
        #[source]
        source: io::Error,
    },
}
