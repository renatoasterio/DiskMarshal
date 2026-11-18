//! A count followed by the word for what is counted.

use diskmarshal_domain::shared::group_digits;

/// The word for what is counted, with whatever follows it, for one and for
/// many: "1 regra", "2 regras".
///
/// Portuguese takes the singular for exactly one; zero is said in the plural.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct Counted {
    one: &'static str,
    many: &'static str,
}

impl Counted {
    /// The word for one and the word for many.
    pub(crate) const fn new(one: &'static str, many: &'static str) -> Self {
        Self { one, many }
    }

    /// The count with its digits grouped, then the word that agrees with it:
    /// "1 item", "1.204 itens".
    pub(crate) fn of(self, count: impl TryInto<u64>) -> String {
        // A count that does not fit is larger than anything a Mac holds, so
        // it is said as the largest one, which keeps the plural.
        let count = count.try_into().unwrap_or(u64::MAX);
        let word = if count == 1 { self.one } else { self.many };
        format!("{} {word}", group_digits(count))
    }
}

#[cfg(test)]
mod tests {
    use super::Counted;

    const RULES: Counted = Counted::new("regra", "regras");

    #[test]
    fn only_one_takes_the_singular() {
        assert_eq!(RULES.of(0_usize), "0 regras");
        assert_eq!(RULES.of(1_usize), "1 regra");
        assert_eq!(RULES.of(2_u64), "2 regras");
    }

    #[test]
    fn groups_the_digits_of_a_large_count() {
        assert_eq!(RULES.of(1_204_u64), "1.204 regras");
    }

    #[test]
    fn what_follows_the_word_agrees_too() {
        let unrecorded = Counted::new(
            "remoção não pôde ser gravada",
            "remoções não puderam ser gravadas",
        );
        assert_eq!(unrecorded.of(1_usize), "1 remoção não pôde ser gravada");
        assert_eq!(
            unrecorded.of(3_usize),
            "3 remoções não puderam ser gravadas"
        );
    }
}
