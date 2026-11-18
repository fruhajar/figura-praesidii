use serde::{Deserialize, Serialize};

use crate::text;

/// Partial dates are the normal case, not an edge case: the register prints a full
/// date of birth, while a mirror site often keeps only the month and year.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct BirthDate {
    pub year: i32,
    pub month: Option<u8>,
    pub day: Option<u8>,
}

impl BirthDate {
    pub fn parse(s: &str) -> Option<Self> {
        let named_month = czech_month(s);
        let groups: Vec<&str> = s
            .split(|c: char| !c.is_ascii_digit())
            .filter(|g| !g.is_empty())
            .collect();
        if let Some(month) = named_month {
            // Full extracts print `14. března 1981`.
            let year = groups.iter().find(|g| g.len() == 4)?.parse().ok()?;
            let day = groups
                .iter()
                .find(|g| g.len() < 3)
                .and_then(|g| g.parse().ok());
            return Self {
                year,
                month: Some(month),
                day,
            }
            .validate();
        }
        let date = match groups.as_slice() {
            [a, b, c] => {
                if a.len() == 4 {
                    Self {
                        year: a.parse().ok()?,
                        month: b.parse().ok(),
                        day: c.parse().ok(),
                    }
                } else {
                    Self {
                        year: c.parse().ok()?,
                        month: b.parse().ok(),
                        day: a.parse().ok(),
                    }
                }
            }
            [a, b] => {
                if a.len() == 4 {
                    Self {
                        year: a.parse().ok()?,
                        month: b.parse().ok(),
                        day: None,
                    }
                } else {
                    Self {
                        year: b.parse().ok()?,
                        month: a.parse().ok(),
                        day: None,
                    }
                }
            }
            [a] if a.len() == 4 => Self {
                year: a.parse().ok()?,
                month: None,
                day: None,
            },
            _ => return None,
        };
        date.validate()
    }

    fn validate(self) -> Option<Self> {
        if !(1850..=2100).contains(&self.year) {
            return None;
        }
        if self.month.is_some_and(|m| !(1..=12).contains(&m)) {
            return None;
        }
        if self.day.is_some_and(|d| !(1..=31).contains(&d)) {
            return None;
        }
        // A day without a month carries no comparable information.
        Some(if self.month.is_none() {
            Self { day: None, ..self }
        } else {
            self
        })
    }

    pub fn is_full(&self) -> bool {
        self.month.is_some() && self.day.is_some()
    }

    /// Czech written form.
    pub fn to_cz(&self) -> String {
        match (self.day, self.month) {
            (Some(d), Some(m)) => format!("{d}. {m}. {}", self.year),
            (None, Some(m)) => format!("{m}/{}", self.year),
            _ => self.year.to_string(),
        }
    }
}

/// Czech month names, nominative and genitive. July is tested before June because
/// `červen` is a prefix of `červenec`.
fn czech_month(s: &str) -> Option<u8> {
    const MONTHS: &[(&str, u8)] = &[
        ("ledna", 1),
        ("leden", 1),
        ("unora", 2),
        ("unor", 2),
        ("brezna", 3),
        ("brezen", 3),
        ("dubna", 4),
        ("duben", 4),
        ("kvetna", 5),
        ("kveten", 5),
        ("cervence", 7),
        ("cervenec", 7),
        ("cervna", 6),
        ("cerven", 6),
        ("srpna", 8),
        ("srpen", 8),
        ("zari", 9),
        ("rijna", 10),
        ("rijen", 10),
        ("listopadu", 11),
        ("listopad", 11),
        ("prosince", 12),
        ("prosinec", 12),
    ];
    let folded = text::fold(s);
    MONTHS
        .iter()
        .find(|(name, _)| folded.contains(name))
        .map(|(_, m)| *m)
}

/// `rodné číslo` is deliberately absent. No Czech public register publishes it, so it
/// would add nothing to a comparison while making the findings store far more
/// damaging to lose.
pub fn looks_like_rodne_cislo(s: &str) -> bool {
    let d = text::digits(s);
    (d.len() == 9 || d.len() == 10) && s.contains('/')
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_the_forms_registers_use() {
        assert_eq!(
            BirthDate::parse("14.3.1981"),
            Some(BirthDate {
                year: 1981,
                month: Some(3),
                day: Some(14)
            })
        );
        assert_eq!(
            BirthDate::parse("14. 3. 1981"),
            Some(BirthDate {
                year: 1981,
                month: Some(3),
                day: Some(14)
            })
        );
        assert_eq!(
            BirthDate::parse("1981-03-14"),
            Some(BirthDate {
                year: 1981,
                month: Some(3),
                day: Some(14)
            })
        );
        assert_eq!(
            BirthDate::parse("3/1981"),
            Some(BirthDate {
                year: 1981,
                month: Some(3),
                day: None
            })
        );
        assert_eq!(
            BirthDate::parse("1981"),
            Some(BirthDate {
                year: 1981,
                month: None,
                day: None
            })
        );
    }

    #[test]
    fn parses_a_czech_month_name() {
        assert_eq!(
            BirthDate::parse("14. března 1981"),
            Some(BirthDate {
                year: 1981,
                month: Some(3),
                day: Some(14)
            })
        );
        assert_eq!(
            BirthDate::parse("dat. nar. 2. července 1974"),
            Some(BirthDate {
                year: 1974,
                month: Some(7),
                day: Some(2)
            })
        );
        assert_eq!(
            BirthDate::parse("červen 1990"),
            Some(BirthDate {
                year: 1990,
                month: Some(6),
                day: None
            })
        );
    }

    #[test]
    fn rejects_impossible_dates() {
        assert_eq!(BirthDate::parse("32.1.1981"), None);
        assert_eq!(BirthDate::parse("1.13.1981"), None);
        assert_eq!(BirthDate::parse("1.1.1700"), None);
        assert_eq!(BirthDate::parse("not a date"), None);
    }
}
