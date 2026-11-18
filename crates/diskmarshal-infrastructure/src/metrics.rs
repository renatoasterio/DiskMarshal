//! Adapters that read the machine's vital signs.

mod sysinfo_metrics_provider;

pub use sysinfo_metrics_provider::{SysinfoMetricsProvider, UnavailableMetrics};
