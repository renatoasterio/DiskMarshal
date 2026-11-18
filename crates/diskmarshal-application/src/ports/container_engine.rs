//! Port for asking Docker what it holds and for removing what nothing uses.

use diskmarshal_domain::docker::{DockerInventory, DockerResource, EngineStatus};
use diskmarshal_domain::shared::ByteSize;

/// Why Docker could not do what was asked.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum EngineError {
    /// No Docker program was found on this Mac.
    #[error("docker is not installed")]
    NotInstalled,
    /// Docker is installed but its engine does not answer.
    #[error("docker is not running")]
    NotRunning,
    /// Docker did not answer within the time limit.
    #[error("docker took too long to answer")]
    TimedOut,
    /// Docker answered with a failure; the text is what it said.
    #[error("docker failed: {0}")]
    Failed(String),
}

/// Talks to the Docker engine installed on this Mac.
///
/// What Docker holds lives inside Docker, not in files this app can see, so
/// it is measured and removed by Docker itself. Implementations must never
/// force a removal: a resource in use is refused by the engine and stays.
pub trait ContainerEngine: Send + Sync {
    /// Whether Docker is installed and its engine answers.
    fn status(&self) -> EngineStatus;

    /// Lists what Docker holds and what of it nothing uses. Changes nothing.
    ///
    /// # Errors
    ///
    /// Fails when Docker is not installed, is not running, or does not
    /// answer in a form this app understands.
    fn inventory(&self) -> Result<DockerInventory, EngineError>;

    /// Space everything Docker holds occupies inside it right now.
    ///
    /// # Errors
    ///
    /// Fails for the same reasons as [`inventory`](Self::inventory).
    fn disk_usage(&self) -> Result<ByteSize, EngineError>;

    /// Removes one unused resource.
    ///
    /// # Errors
    ///
    /// Fails when Docker refuses, for example because the resource came
    /// into use since it was listed.
    fn remove(&self, resource: &DockerResource) -> Result<(), EngineError>;
}
