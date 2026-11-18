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

    /// A rule catalog shipped with the app is malformed.
    #[error("invalid rule catalog: {0}")]
    InvalidCatalog(String),

    /// A cleanup was asked to run a rule that does not exist, or that may not
    /// run with the privileges of the caller.
    #[error("rule not allowed here: {0}")]
    RuleNotAllowed(String),

    /// An uninstall named an app that is not installed, or that is installed
    /// more than once, so it is not known which one is meant.
    #[error("no single installed app has the identifier {0}")]
    AppNotFound(String),

    /// An uninstall named an app that is open. It would recreate its files.
    #[error("the app {0} is running")]
    AppRunning(String),

    /// An uninstall asked for an item that does not belong to the app, or
    /// that may not be removed with the privileges of the caller.
    #[error("item not allowed here: {}", .0.display())]
    ItemNotAllowed(PathBuf),

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
