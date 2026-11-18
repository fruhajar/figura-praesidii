use serde::de::Error as _;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Address, BirthDate, Ico, PersonName, contact};

/// One field a source published. Each carries its raw string, because a request has to
/// quote the data as it appears rather than as the tool normalised it.
#[derive(Debug, Clone)]
pub enum Attribute {
    Name(PersonName),
    BirthDate {
        raw: String,
        value: BirthDate,
    },
    Address(Address),
    Email {
        raw: String,
        local: String,
        domain: String,
    },
    Phone {
        raw: String,
        value: String,
    },
    Ico {
        raw: String,
        value: Ico,
    },
    /// Published but not comparable: a role, an employer, a photo caption. Carried so
    /// the report can show the full extent of what the source holds.
    Other {
        label: String,
        value: String,
    },
}

impl Attribute {
    pub fn parse(kind: &str, value: &str, label: Option<String>) -> Result<Self, String> {
        let raw = value.trim().to_string();
        match kind {
            "name" => Ok(Self::Name(PersonName::parse(&raw))),
            "birth_date" => BirthDate::parse(&raw)
                .map(|value| Self::BirthDate { raw, value })
                .ok_or_else(|| format!("unreadable birth_date: {value:?}")),
            "address" => Ok(Self::Address(Address::parse(&raw))),
            "email" => contact::canonical_email(&raw)
                .map(|(local, domain)| Self::Email { raw, local, domain })
                .ok_or_else(|| format!("not an email: {value:?}")),
            "phone" => contact::canonical_phone(&raw)
                .map(|value| Self::Phone { raw, value })
                .ok_or_else(|| format!("not a phone number: {value:?}")),
            "ico" => Ico::parse(&raw)
                .map(|value| Self::Ico { raw, value })
                .ok_or_else(|| format!("IČO fails its check digit: {value:?}")),
            other => Ok(Self::Other {
                label: label.unwrap_or_else(|| other.to_string()),
                value: raw,
            }),
        }
    }

    pub fn kind(&self) -> &'static str {
        match self {
            Self::Name(_) => "name",
            Self::BirthDate { .. } => "birth_date",
            Self::Address(_) => "address",
            Self::Email { .. } => "email",
            Self::Phone { .. } => "phone",
            Self::Ico { .. } => "ico",
            Self::Other { .. } => "other",
        }
    }

    pub fn raw(&self) -> &str {
        match self {
            Self::Name(n) => &n.raw,
            Self::BirthDate { raw, .. } => raw,
            Self::Address(a) => &a.raw,
            Self::Email { raw, .. } => raw,
            Self::Phone { raw, .. } => raw,
            Self::Ico { raw, .. } => raw,
            Self::Other { value, .. } => value,
        }
    }
}

#[derive(Serialize, Deserialize)]
struct AttrWire {
    kind: String,
    value: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    label: Option<String>,
}

impl<'de> Deserialize<'de> for Attribute {
    fn deserialize<D: Deserializer<'de>>(d: D) -> Result<Self, D::Error> {
        let w = AttrWire::deserialize(d)?;
        Attribute::parse(&w.kind, &w.value, w.label).map_err(D::Error::custom)
    }
}

impl Serialize for Attribute {
    fn serialize<S: Serializer>(&self, s: S) -> Result<S::Ok, S::Error> {
        let label = match self {
            Self::Other { label, .. } => Some(label.clone()),
            _ => None,
        };
        AttrWire {
            kind: self.kind().to_string(),
            value: self.raw().to_string(),
            label,
        }
        .serialize(s)
    }
}

/// One subject-looking entry at one source. A register page about one company officer
/// is a record; the same person at a mirror is a second record.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Record {
    pub id: String,
    /// Matches `Source::id`.
    pub source: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub locator: Option<String>,
    /// ISO-8601 date the record was seen. Text, so the store needs no clock crate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observed: Option<String>,
    pub attrs: Vec<Attribute>,
}

impl Record {
    pub fn names(&self) -> impl Iterator<Item = &PersonName> {
        self.attrs.iter().filter_map(|a| match a {
            Attribute::Name(n) => Some(n),
            _ => None,
        })
    }

    pub fn birth_dates(&self) -> impl Iterator<Item = (&str, BirthDate)> {
        self.attrs.iter().filter_map(|a| match a {
            Attribute::BirthDate { raw, value } => Some((raw.as_str(), *value)),
            _ => None,
        })
    }

    pub fn addresses(&self) -> impl Iterator<Item = &Address> {
        self.attrs.iter().filter_map(|a| match a {
            Attribute::Address(ad) => Some(ad),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_scraped_record() {
        let json = r#"{
            "id": "or-1",
            "source": "or.justice.cz",
            "locator": "https://or.justice.cz/ias/ui/rejstrik-firma.vysledky?subjektId=1",
            "observed": "2026-11-18",
            "attrs": [
                {"kind": "name", "value": "Jana Dvořáková"},
                {"kind": "birth_date", "value": "14.3.1981"},
                {"kind": "address", "value": "Nuselská 123/45, Praha 4, 140 00"},
                {"kind": "role", "value": "jednatel", "label": "function"}
            ]
        }"#;
        let r: Record = serde_json::from_str(json).unwrap();
        assert_eq!(r.names().count(), 1);
        assert_eq!(r.birth_dates().next().unwrap().1.year, 1981);
        assert_eq!(
            r.addresses().next().unwrap().number.as_deref(),
            Some("123/45")
        );
        // An uncomparable field survives round-tripping so the report can show it.
        assert!(matches!(&r.attrs[3], Attribute::Other { label, .. } if label == "function"));
    }
}
