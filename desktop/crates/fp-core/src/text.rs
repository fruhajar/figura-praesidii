//! Folding and tokenising.
//!
//! Every comparison runs on folded text; the string a source actually published is
//! kept beside it untouched, because an erasure request has to quote it verbatim.

/// Lowercase, then strip Czech and German diacritics. `ß` expands to `ss`.
pub fn fold(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for ch in s.to_lowercase().chars() {
        match ch {
            'á' | 'à' | 'â' | 'ä' | 'ã' | 'å' | 'ą' => out.push('a'),
            'č' | 'ć' | 'ç' => out.push('c'),
            'ď' => out.push('d'),
            'é' | 'ě' | 'è' | 'ê' | 'ë' | 'ę' => out.push('e'),
            'í' | 'ì' | 'î' | 'ï' => out.push('i'),
            'ľ' | 'ĺ' | 'ł' => out.push('l'),
            'ň' | 'ń' | 'ñ' => out.push('n'),
            'ó' | 'ò' | 'ô' | 'ö' | 'õ' | 'ø' => out.push('o'),
            'ř' => out.push('r'),
            'š' | 'ś' => out.push('s'),
            'ť' => out.push('t'),
            'ú' | 'ů' | 'ù' | 'û' | 'ü' => out.push('u'),
            'ý' | 'ÿ' => out.push('y'),
            'ž' | 'ź' | 'ż' => out.push('z'),
            'ß' => out.push_str("ss"),
            c => out.push(c),
        }
    }
    out
}

/// Academic and professional titles, folded. Czech registers print them inside the
/// name field, so they would otherwise count as name tokens.
const TITLES: &[&str] = &[
    "ing", "mgr", "bc", "bca", "mga", "judr", "mudr", "mvdr", "phdr", "rndr", "paeddr", "thdr",
    "dr", "doc", "prof", "csc", "drsc", "phd", "ph", "d", "dis", "mba", "llm",
];

pub fn is_title(token: &str) -> bool {
    TITLES.contains(&token)
}

/// Folded alphanumeric tokens with titles dropped. Order is not meaningful: Czech
/// registers print `Surname Given`, prose prints `Given Surname`, and both must
/// compare equal.
pub fn tokens(s: &str) -> Vec<String> {
    fold(s)
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty() && !is_title(t))
        .map(str::to_string)
        .collect()
}

pub fn digits(s: &str) -> String {
    s.chars().filter(char::is_ascii_digit).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn folds_czech_and_german() {
        assert_eq!(fold("Dvořáková"), "dvorakova");
        assert_eq!(fold("Müller"), "muller");
        assert_eq!(fold("Strauß"), "strauss");
        assert_eq!(fold("ŽLUŤOUČKÝ"), "zlutoucky");
    }

    #[test]
    fn drops_titles_and_punctuation() {
        assert_eq!(tokens("Ing. Jan Novák, CSc."), vec!["jan", "novak"]);
        assert_eq!(tokens("NOVÁKOVÁ Jana"), vec!["novakova", "jana"]);
    }
}
