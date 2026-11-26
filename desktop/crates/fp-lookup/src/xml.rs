//! Extraction from the registers' bulk open data.
//!
//! Element names live in the catalog, never here. No dump could be fetched from the
//! machine this was written on, so a guessed element name has to be editable as data
//! and reportable as unverified, exactly like the HTML labels.
//!
//! Dumps are read from disk rather than fetched. They are bulk files the user downloads
//! once, which removes the politeness problem for registers entirely, and streaming from
//! a file keeps a court-year of the register out of memory.

use std::collections::HashMap;
use std::io::BufRead;

use fp_core::{Attribute, name::surname_root, text};
use quick_xml::Reader;
use quick_xml::events::Event;

use crate::catalog::FieldSpec;

pub struct XmlPlan<'a> {
    /// The element enclosing one subject, so a company's own fields do not leak onto
    /// the next company's officers.
    pub subject_element: Option<&'a str>,
    /// The element that is one person.
    pub record_element: &'a str,
    pub record_fields: &'a [FieldSpec],
    pub subject_fields: &'a [FieldSpec],
    /// Rendered from the subject's values, so a request can cite the register entry
    /// rather than a file path on the user's disk.
    pub locator_template: Option<&'a str>,
}

pub struct Limits {
    pub max_bytes: u64,
    pub max_records: usize,
}

/// A cheap name block applied before anything is kept.
///
/// Blocking is a speed trick in record linkage. Here it is first of all the privacy
/// rule: one court-year holds hundreds of thousands of strangers, and none of them
/// belong in this user's report, let alone their store.
pub struct Block {
    pub surname_roots: Vec<String>,
}

impl Block {
    /// Empty means accept everything, which is only ever right for a fixture.
    pub fn accepts(&self, published_name: &str) -> bool {
        if self.surname_roots.is_empty() {
            return true;
        }
        let roots: Vec<String> = text::tokens(published_name)
            .iter()
            .map(|t| surname_root(t))
            .collect();
        self.surname_roots.iter().any(|r| roots.contains(r))
    }
}

#[derive(Debug)]
pub struct Hit {
    pub attrs: Vec<Attribute>,
    pub locator: Option<String>,
}

#[derive(Debug, Default)]
pub struct Extracted {
    pub hits: Vec<Hit>,
    /// Person elements seen, before blocking.
    pub seen: usize,
    /// Dropped by the name block, never kept.
    pub blocked: usize,
    /// A limit stopped the read, so absence of a finding proves nothing.
    pub truncated: bool,
}

pub fn extract<R: BufRead>(
    reader: R,
    plan: &XmlPlan<'_>,
    limits: &Limits,
    block: &Block,
) -> Result<Extracted, String> {
    // Text is deliberately not trimmed by the reader: it would eat the space in
    // "dozorčí rady", where the entity reference splits the run. Trimming happens once,
    // on the accumulated value.
    let mut reader = Reader::from_reader(reader);

    let mut out = Extracted::default();
    let mut buf = Vec::new();
    let mut subject: HashMap<String, String> = HashMap::new();
    let mut record: Option<HashMap<String, String>> = None;
    let mut current: Option<String> = None;
    // One element's text can arrive as several events, because the reader splits it at
    // every entity reference, and the dumps use one for each diacritic. Accumulate here
    // and flush on the closing tag, so a name is not cut at its first `&#345;`.
    let mut scratch = String::new();

    loop {
        buf.clear();
        match reader.read_event_into(&mut buf) {
            Err(e) => return Err(format!("XML at byte {}: {e}", reader.buffer_position())),
            Ok(Event::Eof) => break,
            Ok(Event::Start(e)) => {
                let name = local_name(e.local_name().as_ref());
                if plan.subject_element == Some(name.as_str()) {
                    subject.clear();
                }
                if name == plan.record_element {
                    record = Some(HashMap::new());
                }
                current = Some(name);
                scratch.clear();
            }
            Ok(Event::Text(e)) => {
                if current.is_some() {
                    scratch.push_str(&e.decode().map_err(|e| e.to_string())?);
                }
            }
            Ok(Event::GeneralRef(e)) => {
                if current.is_none() {
                    continue;
                }
                match e.resolve_char_ref() {
                    Ok(Some(ch)) => scratch.push(ch),
                    _ => {
                        let name = e.decode().map_err(|e| e.to_string())?;
                        if let Some(ch) = named_entity(&name) {
                            scratch.push(ch);
                        }
                    }
                }
            }
            Ok(Event::End(e)) => {
                let name = local_name(e.local_name().as_ref());

                if current.as_deref() == Some(name.as_str()) {
                    let value = scratch.trim().to_string();
                    if !value.is_empty() {
                        // First occurrence wins: a repeated element name in one buffer
                        // is the same fact restated, and guessing how to join two of
                        // them would invent data.
                        let target = record.as_mut().unwrap_or(&mut subject);
                        target.entry(name.clone()).or_insert(value);
                    }
                }
                current = None;
                scratch.clear();

                if name == plan.record_element {
                    if let Some(values) = record.take() {
                        out.seen += 1;
                        match finish(&values, &subject, plan, block) {
                            Some(hit) => out.hits.push(hit),
                            None => out.blocked += 1,
                        }
                    }
                }
                if plan.subject_element == Some(name.as_str()) {
                    subject.clear();
                }
            }
            Ok(_) => {}
        }

        if out.hits.len() >= limits.max_records || reader.buffer_position() > limits.max_bytes {
            out.truncated = true;
            break;
        }
    }
    Ok(out)
}

/// The five entities XML predefines. Anything else is a document-defined entity the
/// registers do not use, and silently dropping it beats inventing a character.
fn named_entity(name: &str) -> Option<char> {
    match name {
        "amp" => Some('&'),
        "lt" => Some('<'),
        "gt" => Some('>'),
        "quot" => Some('"'),
        "apos" => Some('\''),
        _ => None,
    }
}

/// `None` means the person was dropped: either no name could be read, or the name is
/// not the user's. Both are counted, neither is kept.
fn finish(
    values: &HashMap<String, String>,
    subject: &HashMap<String, String>,
    plan: &XmlPlan<'_>,
    block: &Block,
) -> Option<Hit> {
    let mut attrs = Vec::new();
    let mut published_name = None;

    for field in plan.record_fields {
        let Some(template) = &field.template else {
            continue;
        };
        let rendered = render_field(template, values);
        if rendered.is_empty() {
            continue;
        }
        if field.kind == "name" {
            published_name = Some(rendered.clone());
        }
        if let Ok(attr) = Attribute::parse(&field.kind, &rendered, Some(field.kind.clone())) {
            attrs.push(attr);
        }
    }

    let name = published_name?;
    if !block.accepts(&name) {
        return None;
    }

    for field in plan.subject_fields {
        let Some(template) = &field.template else {
            continue;
        };
        let rendered = render_field(template, subject);
        if rendered.is_empty() {
            continue;
        }
        if let Ok(attr) = Attribute::parse(&field.kind, &rendered, Some(field.kind.clone())) {
            attrs.push(attr);
        }
    }

    let locator = plan
        .locator_template
        .map(|t| render(t, subject))
        .filter(|l| !l.is_empty() && !l.contains('{'));

    Some(Hit { attrs, locator })
}

fn local_name(raw: &[u8]) -> String {
    String::from_utf8_lossy(raw).to_string()
}

/// A field value: placeholders substituted, then tidied, because a missing part should
/// collapse rather than leave a hole. An address without its house number reads
/// `Nuselská 123`, not `Nuselská 123/,  , 14000`.
pub fn render_field(template: &str, values: &HashMap<String, String>) -> String {
    tidy(&render(template, values))
}

/// Plain substitution, no tidying. Used for the locator, where `https://` must survive.
pub fn render(template: &str, values: &HashMap<String, String>) -> String {
    let mut out = String::with_capacity(template.len());
    let mut rest = template;

    while let Some(start) = rest.find('{') {
        out.push_str(&rest[..start]);
        let after = &rest[start + 1..];
        let Some(end) = after.find('}') else {
            out.push_str(&rest[start..]);
            return out;
        };
        if let Some(value) = values.get(&after[..end]) {
            out.push_str(value);
        }
        rest = &after[end + 1..];
    }
    out.push_str(rest);
    out
}

fn tidy(s: &str) -> String {
    s.split(',')
        .map(|segment| {
            segment
                .split_whitespace()
                .collect::<Vec<_>>()
                .join(" ")
                .trim_matches('/')
                .replace("//", "/")
        })
        .filter(|segment| !segment.is_empty())
        .collect::<Vec<_>>()
        .join(", ")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn values(pairs: &[(&str, &str)]) -> HashMap<String, String> {
        pairs
            .iter()
            .map(|(k, v)| (k.to_string(), v.to_string()))
            .collect()
    }

    #[test]
    fn a_missing_part_collapses_instead_of_leaving_holes() {
        let full = values(&[
            ("ULICE", "Nuselská"),
            ("CISLO_PO", "123"),
            ("CISLO_OR", "45"),
            ("MESTO", "Praha 4"),
            ("PSC", "140 00"),
        ]);
        let template = "{ULICE} {CISLO_PO}/{CISLO_OR}, {MESTO}, {PSC}";
        assert_eq!(
            render_field(template, &full),
            "Nuselská 123/45, Praha 4, 140 00"
        );

        let no_orientation = values(&[
            ("ULICE", "Nuselská"),
            ("CISLO_PO", "123"),
            ("MESTO", "Brno"),
        ]);
        assert_eq!(
            render_field(template, &no_orientation),
            "Nuselská 123, Brno"
        );

        let town_only = values(&[("MESTO", "Brno")]);
        assert_eq!(render_field(template, &town_only), "Brno");
    }

    #[test]
    fn a_locator_keeps_its_double_slash() {
        let subject = values(&[("ICO", "00000019")]);
        assert_eq!(
            render(
                "https://ares.gov.cz/ekonomicke-subjekty?ico={ICO}",
                &subject
            ),
            "https://ares.gov.cz/ekonomicke-subjekty?ico=00000019"
        );
    }

    #[test]
    fn the_block_matches_across_gendered_forms_and_name_order() {
        let block = Block {
            surname_roots: vec!["dvorak".into()],
        };
        assert!(block.accepts("Jana Dvořáková"));
        assert!(block.accepts("DVOŘÁK Petr"));
        assert!(!block.accepts("Marie Svobodová"));
    }
}
