//! How whole numbers are written for people.

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
    use super::group_digits;

    #[test]
    fn groups_digits_in_threes() {
        assert_eq!(group_digits(0), "0");
        assert_eq!(group_digits(999), "999");
        assert_eq!(group_digits(1_000), "1.000");
        assert_eq!(group_digits(4_712_330), "4.712.330");
    }
}
