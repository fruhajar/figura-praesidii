//! Tag stripping, not parsing.
//!
//! The Czech registers lay their full extracts out as labelled rows whose label text
//! is stable, so flattening the page to labelled lines extracts more reliably than a
//! selector would survive a redesign. It is also why there is no HTML dependency.

/// Flatten markup to lines. Block ends become newlines so a label and its value stay
/// on one line where the page put them in one row.
pub fn to_lines(html: &str) -> Vec<String> {
    let mut out = String::with_capacity(html.len());
    let mut i = 0;

    while i < html.len() {
        if html.as_bytes()[i] != b'<' {
            let ch = html[i..].chars().next().unwrap();
            out.push(ch);
            i += ch.len_utf8();
            continue;
        }
        let Some(rel) = html[i..].find('>') else {
            break;
        };
        let tag = html[i + 1..i + rel].trim();
        let name: String = tag
            .trim_start_matches('/')
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric())
            .flat_map(char::to_lowercase)
            .collect();

        // Content that is never user-visible is dropped whole.
        if OPAQUE_TAGS.contains(&name.as_str()) && !tag.starts_with('/') {
            let close = format!("</{name}");
            i = match find_ci(&html[i..], &close) {
                Some(rel) => i + rel,
                None => html.len(),
            };
            continue;
        }
        if BLOCK_TAGS.contains(&name.as_str()) {
            out.push('\n');
        }
        i += rel + 1;
    }

    decode_entities(&out)
        .lines()
        .map(|l| l.split_whitespace().collect::<Vec<_>>().join(" "))
        .filter(|l| !l.is_empty())
        .collect()
}

const OPAQUE_TAGS: &[&str] = &["script", "style", "head", "noscript", "svg"];

const BLOCK_TAGS: &[&str] = &[
    "br", "p", "div", "tr", "td", "th", "li", "ul", "ol", "table", "h1", "h2", "h3", "h4", "h5",
    "h6", "section", "article", "dt", "dd", "option",
];

/// ASCII case-insensitive search. Only ever called with an ASCII needle, so it cannot
/// land on a non-boundary byte.
fn find_ci(haystack: &str, needle: &str) -> Option<usize> {
    let (h, n) = (haystack.as_bytes(), needle.as_bytes());
    if n.is_empty() || h.len() < n.len() {
        return None;
    }
    (0..=h.len() - n.len()).find(|&i| h[i..i + n.len()].eq_ignore_ascii_case(n))
}

fn decode_entities(s: &str) -> String {
    let named = s
        .replace("&nbsp;", " ")
        .replace("&amp;", "&")
        .replace("&lt;", "<")
        .replace("&gt;", ">")
        .replace("&quot;", "\"")
        .replace("&apos;", "'");
    decode_numeric(&named)
}

/// `&#345;` and `&#x159;`. Czech registers emit these for every diacritic.
fn decode_numeric(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    let mut rest = s;
    while let Some(start) = rest.find("&#") {
        out.push_str(&rest[..start]);
        let after = &rest[start + 2..];
        let Some(end) = after.find(';') else {
            out.push_str(&rest[start..]);
            return out;
        };
        let body = &after[..end];
        let parsed = match body.strip_prefix(['x', 'X']) {
            Some(hex) => u32::from_str_radix(hex, 16).ok(),
            None => body.parse::<u32>().ok(),
        };
        match parsed.and_then(char::from_u32) {
            Some(ch) => out.push(ch),
            None => out.push_str(&rest[start..start + 2 + end + 1]),
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

/// Value for a label, taking the rest of the line, or the following line when the page
/// puts the value in the next cell.
///
/// Matching happens on folded text but the value is cut out of the raw line, which is
/// sound because folding maps one character to one character for everything Czech
/// registers print.
pub fn label_value(lines: &[String], labels: &[String]) -> Option<String> {
    for (i, line) in lines.iter().enumerate() {
        let folded: Vec<char> = fp_core::text::fold(line).chars().collect();
        let raw: Vec<char> = line.chars().collect();
        for label in labels {
            let needle: Vec<char> = fp_core::text::fold(label).chars().collect();
            let Some(pos) = char_find(&folded, &needle) else {
                continue;
            };
            let start = (pos + needle.len()).min(raw.len());
            let rest: String = raw[start..].iter().collect();
            let rest = rest.trim_matches([':', ' ', '\u{a0}', '\t']).to_string();
            if !rest.is_empty() {
                return Some(rest);
            }
            if let Some(next) = lines.get(i + 1) {
                if !is_label_line(next, labels) {
                    return Some(next.clone());
                }
            }
        }
    }
    None
}

fn char_find(haystack: &[char], needle: &[char]) -> Option<usize> {
    if needle.is_empty() || haystack.len() < needle.len() {
        return None;
    }
    (0..=haystack.len() - needle.len()).find(|&i| haystack[i..i + needle.len()] == *needle)
}

fn is_label_line(line: &str, labels: &[String]) -> bool {
    let folded = fp_core::text::fold(line);
    labels
        .iter()
        .any(|l| folded.starts_with(&fp_core::text::fold(l)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn flattens_rows_to_lines() {
        let html = "<html><head><title>x</title></head><body>\
            <table><tr><td>Jméno:</td><td>Jana Dvo&#345;áková</td></tr>\
            <tr><td>dat.&nbsp;nar.</td><td>14. března 1981</td></tr></table>\
            <script>var a = '<td>junk</td>';</script></body></html>";
        let lines = to_lines(html);
        assert!(lines.iter().any(|l| l == "Jméno:"), "{lines:?}");
        assert!(lines.iter().any(|l| l == "Jana Dvořáková"), "{lines:?}");
        assert!(!lines.iter().any(|l| l.contains("junk")), "{lines:?}");
    }

    #[test]
    fn reads_a_value_from_the_next_cell() {
        let lines = to_lines("<tr><td>dat. nar.</td><td>14. března 1981</td></tr>");
        let v = label_value(&lines, &["dat. nar.".to_string()]);
        assert_eq!(v.as_deref(), Some("14. března 1981"));
    }

    #[test]
    fn reads_a_value_from_the_same_line() {
        let lines = to_lines("<p>Bydliště: Nuselská 123/45, Praha 4, 140 00</p>");
        let v = label_value(&lines, &["Bydliště:".to_string()]);
        assert_eq!(v.as_deref(), Some("Nuselská 123/45, Praha 4, 140 00"));
    }
}
