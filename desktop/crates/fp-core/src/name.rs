use serde::{Deserialize, Serialize};

use crate::text;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PersonName {
    /// As published. Never compared against; quoted in requests.
    pub raw: String,
    pub tokens: Vec<String>,
    /// Best guess. Comparison uses the token set and the surname roots as well, so a
    /// wrong guess here cannot decide a match on its own.
    pub surname: Option<String>,
}

impl PersonName {
    pub fn parse(raw: &str) -> Self {
        let tokens = text::tokens(raw);
        let surname = guess_surname(raw, &tokens);
        Self {
            raw: raw.trim().to_string(),
            tokens,
            surname,
        }
    }

    pub fn given(&self) -> Vec<String> {
        self.tokens
            .iter()
            .filter(|t| self.surname.as_deref() != Some(t.as_str()))
            .cloned()
            .collect()
    }

    /// Surname roots for every token, so a feminine form matches a masculine one
    /// whichever way round the name was printed.
    pub fn roots(&self) -> Vec<String> {
        self.tokens.iter().map(|t| surname_root(t)).collect()
    }

    pub fn is_bare(&self) -> bool {
        self.tokens.len() < 2
    }
}

/// Strip the Czech feminine suffix: `nováková` and `novák`, `černá` and `černý` share
/// a root. Over-strips in rare cases, which is why a root hit only ever yields a
/// `Strong` level, never `Exact`.
pub fn surname_root(token: &str) -> String {
    if token.len() > 4 {
        if let Some(base) = token.strip_suffix("ova") {
            return base.to_string();
        }
    }
    if token.len() > 3 && (token.ends_with('a') || token.ends_with('y')) {
        return token[..token.len() - 1].to_string();
    }
    token.to_string()
}

fn guess_surname(raw: &str, tokens: &[String]) -> Option<String> {
    if tokens.is_empty() {
        return None;
    }
    if let Some((head, _)) = raw.split_once(',') {
        // "Novák, Jan" is the register's own ordering.
        if let Some(last) = text::tokens(head).pop() {
            return Some(last);
        }
    }
    // A word shouted in capitals is the surname in Czech administrative prose.
    for word in raw.split(|c: char| !c.is_alphanumeric()) {
        let folded = text::fold(word);
        if word.chars().count() > 1
            && word.chars().all(|c| !c.is_alphabetic() || c.is_uppercase())
            && !text::is_title(&folded)
            && tokens.contains(&folded)
        {
            return Some(folded);
        }
    }
    tokens.last().cloned()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn feminine_and_masculine_share_a_root() {
        assert_eq!(surname_root("dvorakova"), "dvorak");
        assert_eq!(surname_root("cerna"), "cern");
        assert_eq!(surname_root("cerny"), "cern");
        assert_eq!(surname_root("svobodova"), "svobod");
        assert_eq!(surname_root("svoboda"), "svobod");
    }

    #[test]
    fn reads_either_name_order() {
        let a = PersonName::parse("Jana Dvořáková");
        let b = PersonName::parse("DVOŘÁKOVÁ Jana");
        let c = PersonName::parse("Dvořáková, Jana");
        assert_eq!(a.surname.as_deref(), Some("dvorakova"));
        assert_eq!(b.surname.as_deref(), Some("dvorakova"));
        assert_eq!(c.surname.as_deref(), Some("dvorakova"));
        assert_eq!(a.given(), vec!["jana"]);
        assert_eq!(b.given(), vec!["jana"]);
    }
}
