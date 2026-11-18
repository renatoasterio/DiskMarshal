//! Port for measuring how fast the connection is.

use diskmarshal_domain::network::SpeedTest;

/// Why a speed test produced no result.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum SpeedTestError {
    /// This Mac has no way to run the test.
    #[error("the speed test is not available here: {0}")]
    Unavailable(String),
    /// The test did not end within the time limit.
    #[error("the speed test took too long")]
    TimedOut,
    /// The test ran and could not finish, for example with no internet.
    #[error("the speed test failed: {0}")]
    Failed(String),
}

/// Measures the connection by moving real data to and from the internet.
///
/// A test uses the connection at full speed for several seconds and spends
/// real data, more than a gigabyte on a fast connection. Implementations run
/// only when asked: never on their own and never in the background.
pub trait SpeedTester: Send + Sync {
    /// Runs one test. Blocks until it ends.
    ///
    /// # Errors
    ///
    /// Fails when the test cannot run here, does not end in time, or cannot
    /// reach the internet.
    fn measure(&self) -> Result<SpeedTest, SpeedTestError>;
}
