//! Turning dozens of raw sensors into a few readings people understand.

use crate::monitoring::SensorReading;

/// A component and the words that identify its sensors, in lowercase.
const GROUPS: [(&str, &[&str]); 5] = [
    ("CPU", &["tdie", "cpu"]),
    ("GPU", &["gpu"]),
    ("SSD", &["nand", "ssd"]),
    ("Bateria", &["battery"]),
    ("Placa", &["tdev"]),
];
/// How many raw sensors are shown when none of them is recognized.
const UNRECOGNIZED_LIMIT: usize = 5;

/// Groups raw sensors by component and averages each group.
///
/// A Mac reports many sensors per component, with names such as
/// `PMU tdie4`. The result has one reading per recognized component, in a
/// fixed order. When nothing is recognized, the first raw sensors are
/// returned as they are, so the user still sees real temperatures.
#[must_use]
pub fn summarize_sensors(raw: &[SensorReading]) -> Vec<SensorReading> {
    let summary: Vec<SensorReading> = GROUPS
        .iter()
        .filter_map(|(label, _)| {
            let temperatures: Vec<f32> = raw
                .iter()
                .filter(|sensor| group_of(&sensor.label) == Some(*label))
                .map(|sensor| sensor.celsius)
                .collect();
            (!temperatures.is_empty()).then(|| SensorReading {
                label: (*label).to_owned(),
                celsius: temperatures.iter().sum::<f32>() / temperatures.len() as f32,
            })
        })
        .collect();
    if summary.is_empty() {
        raw.iter().take(UNRECOGNIZED_LIMIT).cloned().collect()
    } else {
        summary
    }
}

/// The first group whose words appear in the sensor's name.
fn group_of(sensor_label: &str) -> Option<&'static str> {
    let lowercase = sensor_label.to_lowercase();
    GROUPS
        .iter()
        .find(|(_, words)| words.iter().any(|word| lowercase.contains(word)))
        .map(|(label, _)| *label)
}

#[cfg(test)]
mod tests {
    use super::summarize_sensors;
    use crate::monitoring::SensorReading;

    fn sensor(label: &str, celsius: f32) -> SensorReading {
        SensorReading {
            label: label.to_owned(),
            celsius,
        }
    }

    #[test]
    fn averages_the_sensors_of_each_component() {
        let raw = [
            sensor("PMU tdie1", 50.0),
            sensor("PMU tdie2", 60.0),
            sensor("PMU tdev3", 40.0),
            sensor("NAND CH0 temp", 41.0),
            sensor("gas gauge battery", 33.0),
        ];

        let summary = summarize_sensors(&raw);

        let labels: Vec<&str> = summary
            .iter()
            .map(|reading| reading.label.as_str())
            .collect();
        assert_eq!(labels, ["CPU", "SSD", "Bateria", "Placa"]);
        assert!((summary[0].celsius - 55.0).abs() < f32::EPSILON);
    }

    #[test]
    fn recognizes_sensor_names_of_intel_macs() {
        let summary = summarize_sensors(&[sensor("CPU Proximity", 61.0), sensor("GPU Die", 58.0)]);
        let labels: Vec<&str> = summary
            .iter()
            .map(|reading| reading.label.as_str())
            .collect();
        assert_eq!(labels, ["CPU", "GPU"]);
    }

    #[test]
    fn shows_raw_sensors_when_none_is_recognized() {
        let raw: Vec<_> = (0..8)
            .map(|index| sensor(&format!("zone {index}"), 30.0))
            .collect();
        let summary = summarize_sensors(&raw);
        assert_eq!(summary.len(), 5);
        assert_eq!(summary[0].label, "zone 0");
    }

    #[test]
    fn no_sensors_gives_no_readings() {
        assert!(summarize_sensors(&[]).is_empty());
    }
}
