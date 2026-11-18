//! Whether Docker can be asked anything right now.

/// The state of the Docker installation on this Mac.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum EngineStatus {
    /// No Docker program was found.
    #[default]
    NotInstalled,
    /// Docker is installed but its engine does not answer: it is closed or
    /// still starting.
    NotRunning,
    /// The engine answered.
    Running {
        /// Version of the engine.
        version: String,
    },
}
