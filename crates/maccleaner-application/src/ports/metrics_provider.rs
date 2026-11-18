//! Port for reading the machine's vital signs.

use maccleaner_domain::monitoring::SystemSnapshot;

/// Reads processor, memory, sensor and process figures.
pub trait MetricsProvider: Send + Sync {
    /// Takes one reading. Processor figures compare with the previous
    /// reading, so the first one after start-up reports no processor use.
    fn sample(&self) -> SystemSnapshot;
}
