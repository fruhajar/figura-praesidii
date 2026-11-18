//! Field comparisons. Each returns a level and, when it is not obvious, a note that
//! ends up in the report so the user can check the reasoning by hand.

use fp_core::{Address, BirthDate, Identity, NameFidelity, PersonName, name::surname_root};

use crate::weights::Level;

pub struct Comparison {
    pub level: Level,
    pub note: Option<String>,
    /// The asserted value this matched against, for the evidence table.
    pub claimed: String,
}

fn plain(level: Level, claimed: String) -> Comparison {
    Comparison {
        level,
        note: None,
        claimed,
    }
}

pub fn name(id: &Identity, published: &PersonName, fidelity: NameFidelity) -> Comparison {
    let my_surnames: Vec<String> = id.names.iter().filter_map(|n| n.surname.clone()).collect();
    let my_roots = id.surname_roots();
    let my_given = id.given_names();
    let claimed = id
        .names
        .iter()
        .map(|n| n.raw.clone())
        .collect::<Vec<_>>()
        .join(" / ");

    let their_tokens = &published.tokens;
    let their_roots = published.roots();

    let surname_exact = my_surnames.iter().any(|s| their_tokens.contains(s));
    let surname_root_hit = my_roots.iter().any(|r| their_roots.contains(r));
    let given_hit = my_given.iter().any(|g| their_tokens.contains(g));

    // A token that is neither our surname nor one of our given names looks like someone
    // else's first name.
    let foreign_token = their_tokens.iter().any(|t| {
        !my_surnames.contains(t) && !my_roots.contains(&surname_root(t)) && !my_given.contains(t)
    });

    match (surname_exact || surname_root_hit, given_hit) {
        (true, true) if surname_exact => plain(Level::Exact, claimed),
        (true, true) => Comparison {
            level: Level::Strong,
            note: Some("same family name in its other gendered form".into()),
            claimed,
        },
        (true, false) if published.is_bare() => Comparison {
            level: Level::Partial,
            note: Some("surname only; no given name published".into()),
            claimed,
        },
        (true, false) if foreign_token && fidelity == NameFidelity::Legal => Comparison {
            level: Level::Conflict,
            note: Some("surname agrees but the given name is someone else's".into()),
            claimed,
        },
        (true, false) => Comparison {
            level: Level::Weak,
            note: Some("surname agrees, given name does not".into()),
            claimed,
        },
        (false, _) if !their_tokens.is_empty() && fidelity == NameFidelity::Legal => Comparison {
            level: Level::Conflict,
            note: Some("different surname in a source that carries legal names".into()),
            claimed,
        },
        (false, true) => Comparison {
            level: Level::Weak,
            note: Some("given name only".into()),
            claimed,
        },
        (false, _) => plain(Level::Weak, claimed),
    }
}

pub fn birth_date(id: &Identity, published: BirthDate) -> Option<Comparison> {
    let mine = id.birth_date?;
    let claimed = mine.to_cz();
    if mine.year != published.year {
        return Some(Comparison {
            level: Level::Conflict,
            note: Some("born in a different year".into()),
            claimed,
        });
    }
    match (mine.month, published.month) {
        (Some(a), Some(b)) if a != b => {
            return Some(Comparison {
                level: Level::Conflict,
                note: Some("born in a different month".into()),
                claimed,
            });
        }
        (None, _) | (_, None) => {
            return Some(Comparison {
                level: Level::Partial,
                note: Some("only the year is comparable".into()),
                claimed,
            });
        }
        _ => {}
    }
    match (mine.day, published.day) {
        (Some(a), Some(b)) if a == b => Some(plain(Level::Exact, claimed)),
        (Some(a), Some(b)) if a != b => Some(Comparison {
            level: Level::Conflict,
            note: Some(format!("different day of birth ({a} vs {b})")),
            claimed,
        }),
        _ => Some(Comparison {
            level: Level::Strong,
            note: Some("month and year agree; no day published".into()),
            claimed,
        }),
    }
}

pub fn address(id: &Identity, published: &Address) -> Option<Comparison> {
    id.localities
        .iter()
        .map(|mine| one_address(mine, published))
        .max_by(|a, b| {
            a.level
                .partial_cmp(&b.level)
                .unwrap_or(std::cmp::Ordering::Equal)
        })
}

fn one_address(mine: &Address, published: &Address) -> Comparison {
    let claimed = mine.raw.clone();
    // An address with no house number does not parse into a street, so fall back to
    // asking whether the user's own street name turns up among its place words. Safe
    // only because the comparison is against a known identity, not a free-text search.
    let street_hit = match (&mine.street, &published.street) {
        (Some(a), Some(b)) => a == b,
        (Some(a), None) => published.city.contains(a),
        (None, Some(b)) => mine.city.contains(b),
        (None, None) => false,
    };
    let city_hit = mine.city.iter().any(|c| published.city.contains(c));
    let psc_hit = match (&mine.psc, &published.psc) {
        (Some(a), Some(b)) => a == b,
        _ => false,
    };

    if street_hit {
        match (&mine.number, &published.number) {
            (Some(a), Some(b)) if a == b => return plain(Level::Exact, claimed),
            (Some(a), Some(b)) if a != b => {
                return Comparison {
                    level: Level::Weak,
                    note: Some(format!("same street, different house ({a} vs {b})")),
                    claimed,
                };
            }
            _ => {
                return Comparison {
                    level: Level::Strong,
                    note: Some("street agrees; house number missing on one side".into()),
                    claimed,
                };
            }
        }
    }
    if psc_hit {
        return Comparison {
            level: Level::Partial,
            note: Some("same postal code, different or unpublished street".into()),
            claimed,
        };
    }
    if city_hit {
        return Comparison {
            level: Level::Weak,
            note: Some("same town only".into()),
            claimed,
        };
    }
    Comparison {
        level: Level::Conflict,
        note: Some("different place".into()),
        claimed,
    }
}

pub fn email(id: &Identity, local: &str, domain: &str) -> Option<Comparison> {
    if id.emails.is_empty() {
        return None;
    }
    if let Some((l, d)) = id
        .emails
        .iter()
        .find(|(l, d)| l.as_str() == local && d.as_str() == domain)
    {
        return Some(plain(Level::Exact, format!("{l}@{d}")));
    }
    if let Some((l, d)) = id.emails.iter().find(|(l, _)| l.as_str() == local) {
        return Some(Comparison {
            level: Level::Partial,
            note: Some("same handle at a different provider".into()),
            claimed: format!("{l}@{d}"),
        });
    }
    Some(Comparison {
        level: Level::Weak,
        note: None,
        claimed: "none of yours".into(),
    })
}
