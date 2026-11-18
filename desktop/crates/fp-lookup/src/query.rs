use fp_core::Identity;

/// One lookup, scoped to a locality. Locality is the whole point of the free tier: a
/// name alone returns every namesake in the country, while a name plus a town returns
/// a list the user can actually check by eye.
#[derive(Debug, Clone)]
pub struct Query {
    pub full_name: String,
    pub surname: Option<String>,
    pub given: Option<String>,
    pub locality: Option<String>,
    pub birth_year: Option<i32>,
}

impl Query {
    /// One query per asserted name, so a maiden name is searched too.
    pub fn from_identity(id: &Identity, locality: Option<&str>) -> Vec<Self> {
        let locality = locality
            .map(str::to_string)
            .or_else(|| id.localities.first().map(primary_place));

        id.names
            .iter()
            .map(|n| Self {
                full_name: n.raw.clone(),
                surname: n.surname.clone(),
                given: n.given().first().cloned(),
                locality: locality.clone(),
                birth_year: id.birth_date.map(|d| d.year),
            })
            .collect()
    }
}

/// The town, not the district: `Praha 4 - Nusle` searches better as `Praha`.
fn primary_place(a: &fp_core::Address) -> String {
    a.city.first().cloned().unwrap_or_else(|| a.raw.clone())
}

/// Percent-encode for a query string. Hand-rolled to keep the crate dependency-free
/// apart from the HTTP client itself.
pub fn encode(s: &str) -> String {
    let mut out = String::with_capacity(s.len());
    for b in s.as_bytes() {
        match b {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'-' | b'_' | b'.' | b'~' => {
                out.push(*b as char)
            }
            b' ' => out.push_str("%20"),
            other => out.push_str(&format!("%{other:02X}")),
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn one_query_per_name_sharing_the_locality() {
        let id: Identity = serde_json::from_str(
            r#"{"names":["Jana Dvořáková","Jana Marková"],
                "birth_date":"14.3.1981",
                "localities":["Nuselská 123/45, Praha 4, 140 00"]}"#,
        )
        .unwrap();
        let qs = Query::from_identity(&id, None);
        assert_eq!(qs.len(), 2);
        assert_eq!(qs[0].locality.as_deref(), Some("praha"));
        assert_eq!(qs[1].surname.as_deref(), Some("markova"));
        assert_eq!(qs[0].birth_year, Some(1981));
    }

    #[test]
    fn encodes_diacritics_and_spaces() {
        assert_eq!(encode("Jana Dvořáková"), "Jana%20Dvo%C5%99%C3%A1kov%C3%A1");
    }
}
