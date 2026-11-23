//! Turning a finding into the letter that actually works against it.
//!
//! The route is decided in `fp-core::protect`; this crate only writes it down. Getting
//! the pairing right is the whole point: an erasure request aimed at a statutory
//! register is refused, and the refusal costs the month the controller is allowed to
//! take before answering.

pub mod cs;

use fp_core::{Attribute, Controller, Protection, Record, Remedy, Source};
use fp_correlate::{Linkage, Verdict};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RequestKind {
    /// Art. 15 only, for a record that cannot be erased.
    Access,
    /// Art. 21 objection, then Art. 17(1)(c).
    ObjectAndErase,
    /// Removal from search results, source untouched.
    Delist,
    /// Art. 17 erasure with nothing competing against it.
    Erase,
}

impl RequestKind {
    pub fn for_remedy(remedy: Remedy) -> Self {
        match remedy {
            Remedy::Immovable => Self::Access,
            Remedy::ObjectThenErase => Self::ObjectAndErase,
            Remedy::Delist => Self::Delist,
            Remedy::Erase => Self::Erase,
        }
    }

    /// Short slug for a filename.
    pub fn slug(self) -> &'static str {
        match self {
            Self::Access => "pristup-cl15",
            Self::ObjectAndErase => "namitka-vymaz",
            Self::Delist => "odstraneni-z-vysledku",
            Self::Erase => "vymaz-cl17",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Request {
    pub kind: RequestKind,
    pub source_id: String,
    pub record_id: String,
    pub locator: Option<String>,
    pub recipient: Option<Controller>,
    pub subject: String,
    /// The letter, ready to copy. Placeholders are in square brackets.
    pub body: String,
    pub deadline_days: u32,
    pub extension_days: u32,
    /// Everything the user must check before sending. Never empty for a source whose
    /// template has not been verified, or for a record that could still be a namesake's.
    pub warnings: Vec<String>,
}

impl Request {
    pub fn filename(&self) -> String {
        let sanitize = |s: &str| -> String {
            s.chars()
                .map(|c| if c.is_ascii_alphanumeric() { c } else { '-' })
                .collect()
        };
        let source = sanitize(&self.source_id);
        let record = sanitize(&self.record_id);
        // Record ids from the lookup pipeline already carry the source, so do not say
        // it twice.
        let stem = if record.starts_with(&source) {
            record
        } else {
            format!("{source}-{record}")
        };
        format!("{stem}-{}.txt", self.kind.slug())
    }
}

/// What a source published about the user, as the letter has to describe it.
pub struct Disclosure<'a> {
    pub source: &'a Source,
    pub record: &'a Record,
    pub linkage: &'a Linkage,
    pub protection: &'a Protection,
    /// False when the URL template or the labels behind this record are unchecked.
    pub verified: bool,
    /// Readable name of the register a mirror copies, resolved from the catalog.
    /// `Source::copies_from` holds an id, and an id in a letter reads as a typo.
    pub copies_from_title: Option<String>,
}

impl Disclosure<'_> {
    pub(crate) fn copied_register(&self) -> Option<String> {
        self.copies_from_title
            .clone()
            .or_else(|| self.source.copies_from.clone())
    }
}

pub fn build(d: &Disclosure<'_>) -> Request {
    let kind = RequestKind::for_remedy(d.protection.remedy);
    let mut warnings = Vec::new();

    if d.linkage.verdict == Verdict::Possible {
        warnings.push(
            "The evidence does not separate you from a namesake. Sending this asserts \
             the record is yours: open the locator and confirm it by eye first."
                .into(),
        );
    }
    if !d.verified {
        warnings.push(
            "This record came from an unverified source template, so the fields may \
             have been read from the wrong labels. Check them against the page."
                .into(),
        );
    }
    if d.source.controller.is_none() {
        warnings.push(
            "No recipient on file. Take the controller and its contact address from \
             the site's imprint before sending."
                .into(),
        );
    }
    if matches!(kind, RequestKind::ObjectAndErase) && d.copied_register().is_none() {
        warnings.push(
            "The register this site copies from is not recorded, so the letter cannot \
             name it. Fill in `copies_from` for a materially stronger objection."
                .into(),
        );
    }

    Request {
        kind,
        source_id: d.source.id.clone(),
        record_id: d.record.id.clone(),
        locator: d.record.locator.clone(),
        recipient: d.source.controller.clone(),
        subject: cs::subject(kind).to_string(),
        body: cs::letter(kind, d),
        deadline_days: d.protection.response_days,
        extension_days: d.protection.extension_days,
        warnings,
    }
}

/// Czech names for the field kinds, for the "what you publish about me" list.
pub(crate) fn category_cs(attr: &Attribute) -> String {
    let label = match attr {
        Attribute::Name(_) => "jméno a příjmení",
        Attribute::BirthDate { .. } => "datum narození",
        Attribute::Address(_) => "adresa",
        Attribute::Email { .. } => "e-mailová adresa",
        Attribute::Phone { .. } => "telefonní číslo",
        Attribute::Ico { .. } => "IČO",
        Attribute::Other { label, .. } => label,
    };
    format!("{label} (\"{}\")", attr.raw())
}
