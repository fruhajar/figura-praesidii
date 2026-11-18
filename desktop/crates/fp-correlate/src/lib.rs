//! Does this record describe the user, and on what evidence?
//!
//! The answer is never a bare score. Every linkage carries the per-field evidence that
//! produced it, because the user has to be able to check it against the source page,
//! and because a request that asserts "this entry is mine" has to be able to say why.

pub mod compare;
pub mod weights;

use fp_core::{Attribute, Identity, Record, SourceKind};

use weights::{
    DISCRIMINATING_GATE, Field, Level, MATCH_BITS, POSSIBLE_BITS, bits, is_discriminating,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Verdict {
    /// Confidently the user's own data.
    Match,
    /// Consistent with the user, but not distinguishable from a namesake.
    Possible,
    /// Too little in common to report.
    Unrelated,
    /// Actively someone else. Held only as a count, never as content.
    Excluded,
}

#[derive(Debug, Clone)]
pub struct FieldEvidence {
    pub field: Field,
    pub level: Level,
    pub bits: f64,
    /// What the user asserted.
    pub claimed: String,
    /// What the source published, verbatim.
    pub published: String,
    pub note: Option<String>,
}

#[derive(Debug, Clone)]
pub struct Linkage {
    pub verdict: Verdict,
    pub bits: f64,
    pub discriminating_bits: f64,
    pub evidence: Vec<FieldEvidence>,
    /// Why the record was ruled out, when it was.
    pub veto: Option<String>,
}

impl Verdict {
    pub fn label(self) -> &'static str {
        match self {
            Self::Match => "yours",
            Self::Possible => "possibly yours",
            Self::Unrelated => "unrelated",
            Self::Excluded => "someone else",
        }
    }
}

pub fn link(id: &Identity, record: &Record, kind: SourceKind) -> Linkage {
    let fidelity = kind.name_fidelity();
    let mut evidence = Vec::new();

    for attr in &record.attrs {
        let (field, cmp) = match attr {
            Attribute::Name(n) => (Field::Name, Some(compare::name(id, n, fidelity))),
            Attribute::BirthDate { value, .. } => {
                (Field::BirthDate, compare::birth_date(id, *value))
            }
            Attribute::Address(a) => (Field::Address, compare::address(id, a)),
            Attribute::Email { local, domain, .. } => {
                (Field::Email, compare::email(id, local, domain))
            }
            Attribute::Phone { value, .. } => (
                Field::Phone,
                Some(if id.phones.iter().any(|p| p == value) {
                    compare::Comparison {
                        level: Level::Exact,
                        note: None,
                        claimed: value.clone(),
                    }
                } else {
                    compare::Comparison {
                        level: Level::Weak,
                        note: None,
                        claimed: "not yours".into(),
                    }
                }),
            ),
            Attribute::Ico { value, .. } => (
                Field::Ico,
                Some(if id.icos.iter().any(|i| i == value) {
                    compare::Comparison {
                        level: Level::Exact,
                        note: None,
                        claimed: value.as_str().to_string(),
                    }
                } else {
                    compare::Comparison {
                        level: Level::Weak,
                        note: Some("a company you are not registered in".into()),
                        claimed: "none of yours".into(),
                    }
                }),
            ),
            // Published but not comparable. Reported, never scored.
            Attribute::Other { .. } => continue,
        };
        let Some(cmp) = cmp else { continue };
        evidence.push(FieldEvidence {
            field,
            level: cmp.level,
            bits: bits(field, cmp.level, id.name_rarity),
            claimed: cmp.claimed,
            published: attr.raw().to_string(),
            note: cmp.note,
        });
    }

    // Keep only the strongest comparison per field: two addresses on one page are two
    // chances at the same fact, not two independent facts.
    evidence.sort_by(|a, b| {
        a.field
            .label()
            .cmp(b.field.label())
            .then(b.bits.total_cmp(&a.bits))
    });
    evidence.dedup_by(|a, b| a.field == b.field);

    let veto = evidence
        .iter()
        .find(|e| e.level == Level::Conflict && e.bits < 0.0)
        .map(|e| {
            e.note
                .clone()
                .unwrap_or_else(|| format!("{} contradicts yours", e.field.label()))
        });

    let total: f64 = evidence.iter().map(|e| e.bits).sum();
    let discriminating: f64 = evidence
        .iter()
        .filter(|e| is_discriminating(e.field, e.level))
        .map(|e| e.bits)
        .sum();

    let verdict = if veto.is_some() {
        Verdict::Excluded
    } else if total >= MATCH_BITS && discriminating >= DISCRIMINATING_GATE {
        Verdict::Match
    } else if total >= POSSIBLE_BITS {
        Verdict::Possible
    } else {
        Verdict::Unrelated
    };

    Linkage {
        verdict,
        bits: total,
        discriminating_bits: discriminating,
        evidence,
        veto,
    }
}
