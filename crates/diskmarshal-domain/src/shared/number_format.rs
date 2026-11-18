//! How whole numbers are written for people.

/// Each unit of a scale is this many times the one before it.
const UNIT_STEP: u128 = 1000;

/// Writes `value` in the largest of `units` that keeps it at one or more,
/// with one decimal place and the pt-BR decimal comma: `12,4 GB`. Each unit
/// is a thousand times the one before. A value below one thousand is written
/// whole, in the first unit: `512 B`.
#[must_use]
pub fn scale_by_thousands(value: u64, units: &[&str]) -> String {
    let value = u128::from(value);
    let first = units.first().copied().unwrap_or_default();
    if value < UNIT_STEP || units.len() < 2 {
        return format!("{value} {first}");
    }

    let last_unit = units.len() - 1;
    let mut unit = 1;
    let mut divisor = UNIT_STEP;
    while unit < last_unit && value >= divisor * UNIT_STEP {
        unit += 1;
        divisor *= UNIT_STEP;
    }

    // Work in tenths so that rounding (999,95 MB -> 1,0 GB) stays exact.
    let mut tenths = (value * 10 + divisor / 2) / divisor;
    if tenths >= UNIT_STEP * 10 && unit < last_unit {
        unit += 1;
        divisor *= UNIT_STEP;
        tenths = (value * 10 + divisor / 2) / divisor;
    }

    let label = units.get(unit).copied().unwrap_or(first);
    format!("{},{} {label}", tenths / 10, tenths % 10)
}

/// Formats an integer with the pt-BR thousands separator: `4.712.330`.
#[must_use]
pub fn group_digits(value: u64) -> String {
    let digits = value.to_string();
    let mut grouped = String::with_capacity(digits.len() + digits.len() / 3);
    for (index, digit) in digits.chars().enumerate() {
        if index > 0 && (digits.len() - index).is_multiple_of(3) {
            grouped.push('.');
        }
        grouped.push(digit);
    }
    grouped
}

#[cfg(test)]
mod tests {
    use super::{group_digits, scale_by_thousands};

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(999), "999");
        assert_eq!(group_digits(1_000), "1.000");
        assert_eq!(group_digits(4_712_330), "4.712.330");
    }

    #[test]
    fn scales_a_value_to_the_unit_that_fits() {
        let units = ["b/s", "kb/s", "Mb/s"];
        assert_eq!(scale_by_thousands(999, &units), "999 b/s");
        assert_eq!(scale_by_thousands(1_500, &units), "1,5 kb/s");
        assert_eq!(scale_by_thousands(999_950, &units), "1,0 Mb/s");
        assert_eq!(scale_by_thousands(5_000_000_000, &units), "5000,0 Mb/s");
        assert_eq!(scale_by_thousands(7, &[]), "7 ");
    }
}
