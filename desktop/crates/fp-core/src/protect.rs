//! What can be done about a finding.
//!
//! The split per source kind is the product: a letter sent to the wrong target on the
//! wrong article wastes the one month the controller is allowed to take.

use crate::SourceKind;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Remedy {
    /// Publication is a statutory duty. Say so instead of generating a letter that
    /// will be refused.
    Immovable,
    /// Object first, then demand erasure once the legitimate-interest ground falls.
    ObjectThenErase,
    /// Remove from results, source untouched.
    Delist,
    /// Straight erasure; no competing obligation to publish.
    Erase,
}

#[derive(Debug, Clone)]
pub struct Protection {
    pub remedy: Remedy,
    pub articles: &'static [&'static str],
    pub authority: Option<&'static str>,
    pub rationale: &'static str,
    /// Art. 12(3): one month, extensible by two further months.
    pub response_days: u32,
    pub extension_days: u32,
}

pub fn protection_for(kind: SourceKind) -> Protection {
    match kind {
        SourceKind::PublicRegister => Protection {
            remedy: Remedy::Immovable,
            articles: &["Art. 15", "Art. 17(3)(b)"],
            authority: None,
            rationale: "Publication discharges a legal obligation, so erasure is excluded. \
                        An access request still works, and its answer is the evidence base \
                        for every mirror request.",
            response_days: 30,
            extension_days: 60,
        },
        SourceKind::CommercialMirror | SourceKind::DataBroker => Protection {
            remedy: Remedy::ObjectThenErase,
            articles: &["Art. 21(1)", "Art. 17(1)(c)", "Art. 15"],
            authority: Some("ÚOOÚ"),
            rationale: "Republishing for its own commercial ends is legitimate interest, not \
                        legal obligation. Once the objection stands, the ground is gone and \
                        17(1)(c) follows.",
            response_days: 30,
            extension_days: 60,
        },
        SourceKind::SearchEngine => Protection {
            remedy: Remedy::Delist,
            articles: &["Art. 17(1)", "C-131/12 Costeja", "C-507/17 GC v CNIL"],
            authority: Some("ÚOOÚ"),
            rationale: "Delisting detaches the name from the result without touching the \
                        source, which is usually what the person actually wanted.",
            response_days: 30,
            extension_days: 60,
        },
        SourceKind::BreachCorpus | SourceKind::SocialProfile => Protection {
            remedy: Remedy::Erase,
            articles: &["Art. 17(1)(a)", "Art. 17(1)(d)", "Art. 15"],
            authority: Some("ÚOOÚ"),
            rationale: "No publication duty and no surviving purpose, so erasure is \
                        unopposed. For a breach set, also demand Art. 15 disclosure of \
                        recipients.",
            response_days: 30,
            extension_days: 60,
        },
    }
}

impl Remedy {
    pub fn label(self) -> &'static str {
        match self {
            Self::Immovable => "immovable",
            Self::ObjectThenErase => "object, then erase",
            Self::Delist => "delist",
            Self::Erase => "erase",
        }
    }
}
