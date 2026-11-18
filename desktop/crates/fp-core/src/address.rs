use serde::{Deserialize, Serialize};

use crate::text;

/// A Czech address as a register prints it: `Nuselská 123/45, Praha 4 - Nusle, 140 00`.
/// Everything is optional because mirrors routinely keep only the town.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Address {
    pub raw: String,
    pub street: Option<String>,
    /// `čp/čo` kept as printed but whitespace-free, e.g. `123/45`.
    pub number: Option<String>,
    pub city: Vec<String>,
    /// Postal code, five digits, no space.
    pub psc: Option<String>,
}

impl Address {
    /// Only the first comma-separated segment is read as street-and-number, which is
    /// the order every Czech register prints. A reversed `Praha 4, Nuselská 123/45`
    /// degrades to a town-only address rather than being misread.
    pub fn parse(raw: &str) -> Self {
        let mut street = None;
        let mut number = None;
        let mut city: Vec<String> = Vec::new();
        let mut psc = None;

        for (idx, segment) in raw.split(',').enumerate() {
            // `/` stays inside a word so `123/45` survives as one house number.
            let words: Vec<String> = text::fold(segment)
                .split(|c: char| !(c.is_alphanumeric() || c == '/'))
                .filter(|w| !w.is_empty())
                .map(str::to_string)
                .collect();

            if psc.is_none() {
                psc = find_psc(&words);
            }

            if idx == 0 && number.is_none() {
                if let Some(pos) = words.iter().position(|w| house_number(w).is_some()) {
                    number = Some(words[pos].clone());
                    if pos > 0 {
                        street = Some(words[..pos].join(" "));
                    }
                    city.extend(
                        words[pos + 1..]
                            .iter()
                            .filter(|w| is_place_word(w))
                            .cloned(),
                    );
                    continue;
                }
            }
            city.extend(words.iter().filter(|w| is_place_word(w)).cloned());
        }

        Self {
            raw: raw.trim().to_string(),
            street,
            number,
            city,
            psc,
        }
    }

    pub fn has_number(&self) -> bool {
        self.number.is_some() && self.street.is_some()
    }
}

/// `140 00` or `14000` standing on its own.
fn find_psc(words: &[String]) -> Option<String> {
    let numeric = |w: &String| w.chars().all(|c| c.is_ascii_digit());
    for (i, w) in words.iter().enumerate() {
        if numeric(w) && w.len() == 5 {
            return Some(w.clone());
        }
        if numeric(w) && w.len() == 3 {
            if let Some(next) = words.get(i + 1) {
                if numeric(next) && next.len() == 2 {
                    return Some(format!("{w}{next}"));
                }
            }
        }
    }
    None
}

fn is_place_word(word: &str) -> bool {
    word.chars().count() > 1 && word.chars().any(char::is_alphabetic)
}

/// `123`, `123/45` or `123/45a`. Anything five digits or longer without a slash is a
/// postal code or a year, not a house number.
fn house_number(word: &str) -> Option<String> {
    if !word.starts_with(|c: char| c.is_ascii_digit()) {
        return None;
    }
    if !word
        .chars()
        .all(|c| c.is_ascii_digit() || c == '/' || c.is_alphabetic())
    {
        return None;
    }
    let digits = text::digits(word);
    if digits.len() >= 5 && !word.contains('/') {
        return None;
    }
    Some(word.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn splits_a_register_address() {
        let a = Address::parse("Nuselská 123/45, Praha 4 - Nusle, 140 00");
        assert_eq!(a.street.as_deref(), Some("nuselska"));
        assert_eq!(a.number.as_deref(), Some("123/45"));
        assert_eq!(a.psc.as_deref(), Some("14000"));
        assert!(a.city.contains(&"praha".to_string()));
        assert!(a.city.contains(&"nusle".to_string()));
        assert!(a.has_number());
    }

    #[test]
    fn city_only_is_valid() {
        let a = Address::parse("Brno");
        assert_eq!(a.city, vec!["brno"]);
        assert!(!a.has_number());
    }
}
