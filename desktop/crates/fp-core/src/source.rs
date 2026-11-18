use serde::{Deserialize, Serialize};

/// Where a finding came from. The kind decides both how much the name in it can be
/// trusted and which legal route exists against it, so it is not a cosmetic label.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SourceKind {
    /// Statutory register: or.justice.cz, RŽP, ISIR, katastr.
    PublicRegister,
    /// Commercial site republishing a register.
    CommercialMirror,
    SearchEngine,
    BreachCorpus,
    DataBroker,
    SocialProfile,
}

/// Whether the names a source carries are legally kept or self-chosen. A register
/// prints the name on the ID card; a forum prints whatever someone typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NameFidelity {
    Legal,
    Informal,
}

impl SourceKind {
    pub fn name_fidelity(self) -> NameFidelity {
        match self {
            Self::PublicRegister
            | Self::CommercialMirror
            | Self::BreachCorpus
            | Self::DataBroker => NameFidelity::Legal,
            Self::SearchEngine | Self::SocialProfile => NameFidelity::Informal,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::PublicRegister => "public register",
            Self::CommercialMirror => "commercial mirror",
            Self::SearchEngine => "search engine",
            Self::BreachCorpus => "breach corpus",
            Self::DataBroker => "data broker",
            Self::SocialProfile => "social profile",
        }
    }
}

/// The controller to address a request to. Absent means the recipient has to be found
/// by hand, which is itself a finding worth reporting.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Controller {
    pub name: String,
    #[serde(default)]
    pub email: Option<String>,
    #[serde(default)]
    pub postal: Option<String>,
    #[serde(default)]
    pub dpo_email: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Source {
    pub id: String,
    pub title: String,
    pub kind: SourceKind,
    #[serde(default)]
    pub controller: Option<Controller>,
    /// For a mirror: the register it copies from. Naming it is what makes the weaker
    /// legitimate-interest argument concrete.
    #[serde(default)]
    pub copies_from: Option<String>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Catalog {
    pub sources: Vec<Source>,
}

impl Catalog {
    pub fn get(&self, id: &str) -> Option<&Source> {
        self.sources.iter().find(|s| s.id == id)
    }
}
