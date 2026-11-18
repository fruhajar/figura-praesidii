use serde::{Deserialize, Serialize};

use crate::text;

/// Company registration number. Eight digits with a mod-11 check digit, which makes it
/// the one identifier here that can be validated without asking anyone.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ico(String);

impl Ico {
    pub fn parse(s: &str) -> Option<Self> {
        let d = text::digits(s);
        if d.len() != 8 {
            return None;
        }
        let digits: Vec<u32> = d.chars().map(|c| c.to_digit(10).unwrap()).collect();
        let sum: u32 = digits[..7]
            .iter()
            .zip((2..=8).rev())
            .map(|(d, w)| d * w)
            .sum();
        let expected = match sum % 11 {
            0 => 1,
            1 => 0,
            r => 11 - r,
        };
        (expected == digits[7]).then_some(Self(d))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_the_check_digit() {
        // Czech National Bank, a published IČO, used here only as a known-good value.
        assert!(Ico::parse("48136450").is_some());
        assert!(Ico::parse("48 136 450").is_some());
        assert!(Ico::parse("48136451").is_none());
        assert!(Ico::parse("1234567").is_none());
    }
}
