//! Sources as data, not code.
//!
//! Every URL template and label here is a guess until someone runs `fp verify-sources`
//! from a machine that can reach the site. `verified: false` is carried through to the
//! report rather than hidden, because a silently wrong selector looks exactly like a
//! clean result.

use std::fs;
use std::io;
use std::path::Path;

use fp_core::{Attribute, Controller, Record, Source, SourceKind};
use serde::{Deserialize, Serialize};

use crate::html;
use crate::query::{Query, encode};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Catalog {
    pub sources: Vec<SourceEntry>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceEntry {
    pub id: String,
    pub title: String,
    pub kind: SourceKind,
    /// Has a human confirmed the URL template and the labels against the live site?
    #[serde(default)]
    pub verified: bool,
    /// Off for sources the fetcher cannot drive yet. Kept in the catalog so the gap
    /// stays visible.
    #[serde(default = "default_true")]
    pub enabled: bool,
    pub query: QuerySpec,
    pub extract: ExtractSpec,
    #[serde(default)]
    pub controller: Option<Controller>,
    #[serde(default)]
    pub copies_from: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuerySpec {
    /// Placeholders: `{name}`, `{surname}`, `{given}`, `{locality}`, `{year}`.
    pub url: String,
    /// Skip this source when the user gave no locality, rather than fetching the whole
    /// country.
    #[serde(default)]
    pub requires_locality: bool,
    /// Minimum gap between two requests to this host.
    #[serde(default = "default_interval")]
    pub min_interval_ms: u64,
}

fn default_interval() -> u64 {
    2_000
}

fn default_true() -> bool {
    true
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExtractSpec {
    /// A line that begins a new person on the page. Absent means the page is one record.
    #[serde(default)]
    pub record_split: Option<String>,
    /// Fields belonging to the person.
    pub fields: Vec<FieldSpec>,
    /// Fields belonging to the page rather than to any one person: the company a full
    /// extract is about, its IČO. Read once and attached to every record on the page,
    /// which is what keeps one officer's company off the next officer's record.
    #[serde(default)]
    pub page_fields: Vec<FieldSpec>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldSpec {
    /// `name`, `birth_date`, `address`, `email`, `phone`, `ico`, or any label for a
    /// field that is reported but never scored.
    pub kind: String,
    pub labels: Vec<String>,
}

impl Catalog {
    /// The shipped catalog. Deliberately small: a public register list plus the shape a
    /// mirror entry takes, so a user can add the mirrors that rank on their own name.
    pub fn builtin() -> Self {
        serde_json::from_str(include_str!("../sources.json")).expect("builtin catalog parses")
    }

    pub fn load(path: &Path) -> io::Result<Self> {
        let text = fs::read_to_string(path)?;
        serde_json::from_str(&text).map_err(io::Error::other)
    }

    pub fn get(&self, id: &str) -> Option<&SourceEntry> {
        self.sources.iter().find(|s| s.id == id)
    }
}

impl SourceEntry {
    pub fn source(&self) -> Source {
        Source {
            id: self.id.clone(),
            title: self.title.clone(),
            kind: self.kind,
            controller: self.controller.clone(),
            copies_from: self.copies_from.clone(),
        }
    }

    /// `None` when this source cannot answer the query, which is not an error.
    pub fn url_for(&self, q: &Query) -> Option<String> {
        if self.query.requires_locality && q.locality.is_none() {
            return None;
        }
        let mut url = self.query.url.clone();
        // A missing name leaves nothing to search for; a missing locality or year just
        // widens the query, so those placeholders collapse to nothing.
        for (key, value, essential) in [
            ("{name}", Some(q.full_name.clone()), true),
            ("{surname}", q.surname.clone(), true),
            ("{given}", q.given.clone(), true),
            ("{locality}", q.locality.clone(), false),
            ("{year}", q.birth_year.map(|y| y.to_string()), false),
        ] {
            if !url.contains(key) {
                continue;
            }
            match value {
                Some(value) => url = url.replace(key, &encode(&value)),
                None if essential => return None,
                None => url = url.replace(key, ""),
            }
        }
        Some(url)
    }

    /// Records from one fetched page. Chunks without a name are dropped: a page of
    /// navigation furniture should produce nothing, not an empty record.
    pub fn records(&self, body: &str, locator: &str, observed: &str) -> Vec<Record> {
        let lines = html::to_lines(body);
        let page_attrs: Vec<Attribute> = self
            .extract
            .page_fields
            .iter()
            .filter_map(|field| {
                let value = html::label_value(&lines, &field.labels)?;
                Attribute::parse(&field.kind, &value, Some(field.kind.clone())).ok()
            })
            .collect();
        let mut out = Vec::new();

        for (i, chunk) in self.chunks(&lines).into_iter().enumerate() {
            let mut attrs = Vec::new();
            for field in &self.extract.fields {
                let Some(value) = html::label_value(&chunk, &field.labels) else {
                    continue;
                };
                match Attribute::parse(&field.kind, &value, Some(field.kind.clone())) {
                    Ok(attr) => attrs.push(attr),
                    // A label that matched the wrong text is normal on a guessed
                    // selector; drop the field, keep the record.
                    Err(_) => continue,
                }
            }
            if !attrs.iter().any(|a| matches!(a, Attribute::Name(_))) {
                continue;
            }
            attrs.extend(page_attrs.iter().cloned());
            out.push(Record {
                id: format!("{}-{}", self.id, i + 1),
                source: self.id.clone(),
                locator: Some(locator.to_string()),
                observed: Some(observed.to_string()),
                attrs,
            });
        }
        out
    }

    fn chunks(&self, lines: &[String]) -> Vec<Vec<String>> {
        let Some(marker) = &self.extract.record_split else {
            return vec![lines.to_vec()];
        };
        let needle = fp_core::text::fold(marker);
        let starts: Vec<usize> = lines
            .iter()
            .enumerate()
            .filter(|(_, l)| fp_core::text::fold(l).contains(&needle))
            .map(|(i, _)| i)
            .collect();
        if starts.is_empty() {
            return vec![lines.to_vec()];
        }
        starts
            .iter()
            .enumerate()
            .map(|(n, &start)| {
                let end = starts.get(n + 1).copied().unwrap_or(lines.len());
                lines[start..end].to_vec()
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_builtin_catalog_parses() {
        let c = Catalog::builtin();
        assert!(c.get("or.justice.cz").is_some());
        // Nothing ships marked verified; the container that built it could not reach
        // any of these hosts.
        assert!(c.sources.iter().all(|s| !s.verified));
    }

    #[test]
    fn a_template_needing_a_missing_field_yields_no_url() {
        let c = Catalog::builtin();
        let entry = c.get("or.justice.cz").unwrap();
        let q = Query {
            full_name: "Jana Dvořáková".into(),
            surname: Some("dvorakova".into()),
            given: Some("jana".into()),
            locality: None,
            birth_year: None,
        };
        assert!(entry.url_for(&q).is_some());
        let locality_only = c.sources.iter().find(|s| s.query.requires_locality);
        if let Some(entry) = locality_only {
            assert!(entry.url_for(&q).is_none());
        }
    }
}
