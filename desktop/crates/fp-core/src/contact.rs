use crate::text;

/// Canonical address for comparison only. Gmail ignores dots in the local part and
/// everything after a `+`, so `j.novak+shop@gmail.com` and `jnovak@gmail.com` are one
/// mailbox; treating them as two would miss a real correlation.
pub fn canonical_email(s: &str) -> Option<(String, String)> {
    let folded = text::fold(s.trim());
    let (local, domain) = folded.rsplit_once('@')?;
    if local.is_empty() || !domain.contains('.') {
        return None;
    }
    let local = local.split('+').next().unwrap_or(local);
    let (local, domain) = match domain {
        // One mailbox under two names.
        "gmail.com" | "googlemail.com" => (local.replace('.', ""), "gmail.com"),
        _ => (local.to_string(), domain),
    };
    Some((local, domain.to_string()))
}

/// Czech numbers are stored without a prefix often enough that a bare nine-digit
/// number has to be read as domestic.
pub fn canonical_phone(s: &str) -> Option<String> {
    let mut d = text::digits(s);
    if let Some(rest) = d.strip_prefix("00") {
        d = rest.to_string();
    }
    match d.len() {
        9 => Some(format!("+420{d}")),
        12 if d.starts_with("420") => Some(format!("+{d}")),
        11..=15 => Some(format!("+{d}")),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn gmail_aliases_collapse() {
        assert_eq!(
            canonical_email("J.Novak+shop@Gmail.com"),
            canonical_email("jnovak@gmail.com")
        );
        assert_eq!(
            canonical_email("j.novak@googlemail.com"),
            canonical_email("jnovak@gmail.com")
        );
        // Dots are significant everywhere else.
        assert_ne!(
            canonical_email("j.novak@seznam.cz"),
            canonical_email("jnovak@seznam.cz")
        );
    }

    #[test]
    fn phones_normalise_to_e164() {
        assert_eq!(
            canonical_phone("606 123 456").as_deref(),
            Some("+420606123456")
        );
        assert_eq!(
            canonical_phone("+420 606 123 456").as_deref(),
            Some("+420606123456")
        );
        assert_eq!(
            canonical_phone("00420606123456").as_deref(),
            Some("+420606123456")
        );
        assert_eq!(canonical_phone("12345").as_deref(), None);
    }
}
