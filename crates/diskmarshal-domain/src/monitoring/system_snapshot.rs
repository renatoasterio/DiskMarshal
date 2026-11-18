//! One reading of the machine's vital signs.

use crate::shared::ByteSize;

/// A temperature reported by a sensor, or the average of a group of sensors.
#[derive(Debug, Clone, PartialEq)]
pub struct SensorReading {
    /// What is being measured.
    pub label: String,
    /// Temperature in degrees Celsius.
    pub celsius: f32,
}

/// How much one running program is using.
#[derive(Debug, Clone, PartialEq)]
pub struct ProcessUsage {
    /// Name of the program.
    pub name: String,
    /// Processor use in cores: 1.0 is one core fully busy.
    pub cpu_cores: f32,
    /// Memory in use.
    pub memory: ByteSize,
}

/// The state of the processor, memory, sensors and busiest programs at one moment.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct SystemSnapshot {
    /// Processor use across all cores, from 0.0 to 1.0.
    pub cpu_usage: f32,
    /// Use of each core, from 0.0 to 1.0.
    pub core_usage: Vec<f32>,
    /// Memory in use.
    pub memory_used: ByteSize,
    /// Memory installed.
    pub memory_total: ByteSize,
    /// Swap space in use.
    pub swap_used: ByteSize,
    /// Temperatures, already grouped by component.
    pub sensors: Vec<SensorReading>,
    /// The programs using the most processor, busiest first.
    pub processes: Vec<ProcessUsage>,
}

impl SystemSnapshot {
    /// Share of the memory in use, from 0.0 to 1.0. No memory reports 0.0.
    #[must_use]
    pub fn memory_ratio(&self) -> f32 {
        if self.memory_total.is_zero() {
            return 0.0;
        }
        (self.memory_used.as_bytes() as f64 / self.memory_total.as_bytes() as f64) as f32
    }

    /// The hottest reading, when there is any sensor.
    #[must_use]
    pub fn hottest(&self) -> Option<&SensorReading> {
        self.sensors
            .iter()
            .max_by(|a, b| a.celsius.total_cmp(&b.celsius))
    }
}
