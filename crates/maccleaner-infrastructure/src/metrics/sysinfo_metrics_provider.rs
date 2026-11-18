//! Metrics read through the `sysinfo` crate.

use std::sync::{Mutex, PoisonError};

use maccleaner_application::ports::MetricsProvider;
use maccleaner_domain::monitoring::{
    ProcessUsage, SensorReading, SystemSnapshot, summarize_sensors,
};
use maccleaner_domain::shared::ByteSize;
use sysinfo::{Components, ProcessRefreshKind, ProcessesToUpdate, System};

/// How many of the busiest programs a reading lists.
const TOP_PROCESSES: usize = 6;
/// `sysinfo` reports percentages; the domain works with fractions.
const PERCENT: f32 = 100.0;

/// Stands in when the system cannot be queried: every reading is empty, and
/// the interface says the figures are unavailable.
#[derive(Debug, Clone, Copy, Default)]
pub struct UnavailableMetrics;

impl MetricsProvider for UnavailableMetrics {
    fn sample(&self) -> SystemSnapshot {
        SystemSnapshot::default()
    }
}

/// Programs are listed again every this many readings. Listing them is
/// much slower than reading the processor and the memory.
const PROCESS_INTERVAL: u64 = 2;
/// Temperatures are read again every this many readings. They change slowly
/// and reading them is the slowest part of all.
const SENSOR_INTERVAL: u64 = 5;

/// What is kept between readings: what `sysinfo` needs to compute rates,
/// and the slow figures that are not read every time.
struct State {
    system: System,
    components: Option<Components>,
    readings: u64,
    processes: Vec<ProcessUsage>,
    sensors: Vec<SensorReading>,
}

/// Reads processor, memory, process and sensor figures with `sysinfo`.
pub struct SysinfoMetricsProvider {
    state: Mutex<State>,
}

impl SysinfoMetricsProvider {
    /// Creates the provider. Returns `None` when the system cannot be queried
    /// at all. Missing temperature sensors are not an error: readings then
    /// simply carry no sensors.
    #[must_use]
    pub fn new() -> Option<Self> {
        let state = State {
            system: System::new().ok()?,
            components: Components::new_with_refreshed_list().ok(),
            readings: 0,
            processes: Vec::new(),
            sensors: Vec::new(),
        };
        Some(Self {
            state: Mutex::new(state),
        })
    }
}

impl MetricsProvider for SysinfoMetricsProvider {
    fn sample(&self) -> SystemSnapshot {
        let mut state = self.state.lock().unwrap_or_else(PoisonError::into_inner);
        let state = &mut *state;
        state.system.refresh_cpu_usage();
        state.system.refresh_memory();
        if state.readings.is_multiple_of(PROCESS_INTERVAL) {
            state.processes = busiest_processes(&mut state.system);
        }
        if state.readings.is_multiple_of(SENSOR_INTERVAL) {
            state.sensors = summarize_sensors(&temperatures(state.components.as_mut()));
        }
        state.readings += 1;

        let system = &state.system;
        SystemSnapshot {
            cpu_usage: (system.global_cpu_usage() / PERCENT).clamp(0.0, 1.0),
            core_usage: system
                .cpus()
                .iter()
                .map(|cpu| (cpu.usage() / PERCENT).clamp(0.0, 1.0))
                .collect(),
            memory_used: ByteSize::from_bytes(system.used_memory()),
            memory_total: ByteSize::from_bytes(system.total_memory()),
            swap_used: ByteSize::from_bytes(system.used_swap()),
            sensors: state.sensors.clone(),
            processes: state.processes.clone(),
        }
    }
}

/// Lists the programs again and returns the ones using the most processor.
fn busiest_processes(system: &mut System) -> Vec<ProcessUsage> {
    system.refresh_processes_specifics(
        ProcessesToUpdate::All,
        true,
        ProcessRefreshKind::nothing().with_cpu().with_memory(),
    );
    let mut processes: Vec<ProcessUsage> = system
        .processes()
        .values()
        .map(|process| ProcessUsage {
            name: process.name().to_string_lossy().into_owned(),
            cpu_cores: process.cpu_usage() / PERCENT,
            memory: ByteSize::from_bytes(process.memory()),
        })
        .collect();
    processes.sort_by(|a, b| b.cpu_cores.total_cmp(&a.cpu_cores));
    processes.truncate(TOP_PROCESSES);
    processes
}

/// Reads every temperature sensor that answers.
fn temperatures(components: Option<&mut Components>) -> Vec<SensorReading> {
    let Some(components) = components else {
        return Vec::new();
    };
    components.refresh(false);
    components
        .iter()
        .filter_map(|component| {
            let celsius = component.temperature().filter(|value| value.is_finite())?;
            Some(SensorReading {
                label: component.label().to_owned(),
                celsius,
            })
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::thread;
    use std::time::Duration;

    use maccleaner_application::ports::MetricsProvider;

    use super::SysinfoMetricsProvider;

    #[test]
    fn reads_plausible_figures_from_this_machine() {
        let provider = SysinfoMetricsProvider::new().unwrap();
        provider.sample();
        thread::sleep(Duration::from_millis(250));
        let snapshot = provider.sample();

        assert!(!snapshot.memory_total.is_zero());
        assert!(snapshot.memory_used <= snapshot.memory_total);
        assert!(!snapshot.core_usage.is_empty());
        assert!((0.0..=1.0).contains(&snapshot.cpu_usage));
        assert!(
            snapshot
                .core_usage
                .iter()
                .all(|usage| (0.0..=1.0).contains(usage))
        );
        assert!(!snapshot.processes.is_empty());
        assert!(
            snapshot
                .sensors
                .iter()
                .all(|sensor| sensor.celsius > -50.0 && sensor.celsius < 150.0)
        );
    }
}
