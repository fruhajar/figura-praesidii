use serde::{Deserialize, Serialize};

use crate::{Address, BirthDate, Ico, PersonName, contact};

/// How much a full-name agreement is worth as evidence. Defaulting to `Common` is the
/// conservative choice: it lowers every name weight, so borderline records land in
/// "possible" rather than "match".
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Rarity {
    #[default]
    Common,
    Uncommon,
    Rare,
}

/// What the user asserts about themselves. This is the only identity the tool will
/// ever match against, and it is supplied by the person it describes.
#[derive(Debug, Clone, Deserialize)]
#[serde(try_from = "IdentityWire")]
pub struct Identity {
    pub names: Vec<PersonName>,
    pub birth_date: Option<BirthDate>,
    /// Places lived or registered. Doubles as the lookup scope and as a weak
    /// comparison field.
    pub localities: Vec<Address>,
    /// Canonicalised `(local, domain)` pairs.
    pub emails: Vec<(String, String)>,
    pub phones: Vec<String>,
    pub icos: Vec<Ico>,
    pub name_rarity: Rarity,
}

impl Identity {
    /// Surname roots across every asserted name, including maiden names.
    pub fn surname_roots(&self) -> Vec<String> {
        let mut roots: Vec<String> = self
            .names
            .iter()
            .filter_map(|n| n.surname.as_ref().map(|s| crate::name::surname_root(s)))
            .collect();
        roots.sort();
        roots.dedup();
        roots
    }

    pub fn given_names(&self) -> Vec<String> {
        let mut given: Vec<String> = self.names.iter().flat_map(|n| n.given()).collect();
        given.sort();
        given.dedup();
        given
    }

    /// Localities as folded tokens, for scoping a register query.
    pub fn locality_tokens(&self) -> Vec<String> {
        let mut t: Vec<String> = self
            .localities
            .iter()
            .flat_map(|a| a.city.clone())
            .collect();
        t.sort();
        t.dedup();
        t
    }
}

#[derive(Deserialize)]
struct IdentityWire {
    names: Vec<String>,
    #[serde(default)]
    birth_date: Option<String>,
    #[serde(default)]
    localities: Vec<String>,
    #[serde(default)]
    emails: Vec<String>,
    #[serde(default)]
    phones: Vec<String>,
    #[serde(default)]
    icos: Vec<String>,
    #[serde(default)]
    name_rarity: Rarity,
}

impl TryFrom<IdentityWire> for Identity {
    type Error = String;

    fn try_from(w: IdentityWire) -> Result<Self, Self::Error> {
        if w.names.is_empty() {
            return Err("an identity needs at least one name".into());
        }
        let birth_date = match w.birth_date.as_deref() {
            Some(s) if !s.trim().is_empty() => {
                Some(BirthDate::parse(s).ok_or_else(|| format!("unreadable birth_date: {s:?}"))?)
            }
            _ => None,
        };
        let mut emails = Vec::new();
        for e in &w.emails {
            emails.push(contact::canonical_email(e).ok_or_else(|| format!("not an email: {e:?}"))?);
        }
        let mut phones = Vec::new();
        for p in &w.phones {
            phones.push(
                contact::canonical_phone(p).ok_or_else(|| format!("not a phone number: {p:?}"))?,
            );
        }
        let mut icos = Vec::new();
        for i in &w.icos {
            icos.push(Ico::parse(i).ok_or_else(|| format!("IČO fails its check digit: {i:?}"))?);
        }
        Ok(Self {
            names: w.names.iter().map(|n| PersonName::parse(n)).collect(),
            birth_date,
            localities: w.localities.iter().map(|l| Address::parse(l)).collect(),
            emails,
            phones,
            icos,
            name_rarity: w.name_rarity,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_a_self_description() {
        let json = r#"{
            "names": ["Jana Dvořáková", "Jana Marková"],
            "birth_date": "14.3.1981",
            "localities": ["Nuselská 123/45, Praha 4, 140 00"],
            "emails": ["J.Dvorakova+shop@gmail.com"],
            "phones": ["606 123 456"],
            "icos": ["48136450"]
        }"#;
        let id: Identity = serde_json::from_str(json).unwrap();
        assert_eq!(id.surname_roots(), vec!["dvorak", "mark"]);
        assert_eq!(id.given_names(), vec!["jana"]);
        assert!(id.locality_tokens().contains(&"praha".to_string()));
        assert_eq!(id.emails[0].0, "jdvorakova");
        assert_eq!(id.name_rarity, Rarity::Common);
    }

    #[test]
    fn a_bad_ico_is_rejected_at_the_door() {
        let json = r#"{"names":["Jan Novák"],"icos":["48136451"]}"#;
        let err = serde_json::from_str::<Identity>(json)
            .unwrap_err()
            .to_string();
        assert!(err.contains("check digit"), "{err}");
    }
}
