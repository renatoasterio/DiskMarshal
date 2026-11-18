//! How warm a temperature reading is.

use crate::monitoring::SensorReading;

/// From this temperature up a component counts as warm.
const WARM_CELSIUS: f32 = 60.0;
/// From this temperature up a component counts as hot.
const HOT_CELSIUS: f32 = 85.0;
/// The top of the scale temperatures are drawn on.
const SCALE_CELSIUS: f32 = 100.0;

/// How warm a component is, in the three steps the interface tells apart.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ThermalLevel {
    /// Nothing to notice.
    Normal,
    /// Working hard, still within what the machine is built for.
    Warm,
    /// Close to the point where the machine slows itself down.
    Hot,
}

impl ThermalLevel {
    /// Classifies a temperature in degrees Celsius.
    #[must_use]
    pub fn of(celsius: f32) -> Self {
        if celsius >= HOT_CELSIUS {
            Self::Hot
        } else if celsius >= WARM_CELSIUS {
            Self::Warm
        } else {
            Self::Normal
        }
    }
}

impl SensorReading {
    /// How warm this reading is.
    #[must_use]
    pub fn level(&self) -> ThermalLevel {
        ThermalLevel::of(self.celsius)
    }

    /// Where this reading sits on the drawing scale, from 0.0 to 1.0.
    #[must_use]
    pub fn scale_ratio(&self) -> f32 {
        (self.celsius / SCALE_CELSIUS).clamp(0.0, 1.0)
    }
}

#[cfg(test)]
mod tests {
    use super::ThermalLevel;
    use crate::monitoring::SensorReading;

    fn reading(celsius: f32) -> SensorReading {
        SensorReading {
            label: "CPU".to_owned(),
            celsius,
        }
    }

    #[test]
    fn classifies_temperatures_at_the_thresholds() {
        assert_eq!(ThermalLevel::of(59.9), ThermalLevel::Normal);
        assert_eq!(ThermalLevel::of(60.0), ThermalLevel::Warm);
        assert_eq!(ThermalLevel::of(84.9), ThermalLevel::Warm);
        assert_eq!(ThermalLevel::of(85.0), ThermalLevel::Hot);
        assert_eq!(reading(47.0).level(), ThermalLevel::Normal);
    }

    #[test]
    fn the_scale_never_leaves_its_ends() {
        assert!((reading(50.0).scale_ratio() - 0.5).abs() < f32::EPSILON);
        assert!((reading(140.0).scale_ratio() - 1.0).abs() < f32::EPSILON);
        assert!(reading(-10.0).scale_ratio().abs() < f32::EPSILON);
    }
}
