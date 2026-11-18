//! Match weights in bits.
//!
//! Fellegi-Sunter: a field agreement is worth `log2(m/u)` bits, where `m` is the chance
//! the field agrees given the record really is the user's, and `u` the chance it agrees
//! by coincidence. The `u` values behind this table are rough population estimates for
//! the Czech Republic and are deliberately pessimistic, because overstating a weight
//! promotes a stranger's record into the user's report.
//!
//! One-to-many against a single asserted identity, never n x n clustering: the user is
//! the ground truth, which is the one luxury self-lookup has over record linkage.

use fp_core::Rarity;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    Name,
    BirthDate,
    Address,
    Email,
    Phone,
    Ico,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd)]
pub enum Level {
    /// Field present on both sides and contradicting.
    Conflict,
    /// Present but barely informative.
    Weak,
    Partial,
    Strong,
    Exact,
}

impl Field {
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::BirthDate => "date of birth",
            Self::Address => "address",
            Self::Email => "e-mail",
            Self::Phone => "phone",
            Self::Ico => "IČO",
        }
    }
}

impl Level {
    pub fn label(self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::Strong => "strong",
            Self::Partial => "partial",
            Self::Weak => "weak",
            Self::Conflict => "CONFLICT",
        }
    }
}

/// `u` for a full-name agreement swings by two orders of magnitude between `Novák` and
/// a rare surname, which is the single biggest lever on a false match.
fn name_rarity_bonus(rarity: Rarity) -> f64 {
    match rarity {
        Rarity::Common => 0.0,
        Rarity::Uncommon => 3.0,
        Rarity::Rare => 6.0,
    }
}

pub fn bits(field: Field, level: Level, rarity: Rarity) -> f64 {
    use Field as F;
    use Level as L;
    match (field, level) {
        // u ~ 1e-3 for a common given+surname pair in a 10.9M population.
        (F::Name, L::Exact) => 9.0 + name_rarity_bonus(rarity),
        (F::Name, L::Strong) => 7.0 + name_rarity_bonus(rarity),
        (F::Name, L::Partial) => 4.0 + name_rarity_bonus(rarity) / 2.0,
        (F::Name, L::Weak) => 0.5,
        (F::Name, L::Conflict) => -12.0,

        // u ~ 1/36500 for a full date; 1/1000 for month+year; 1/80 for a year.
        (F::BirthDate, L::Exact) => 15.0,
        (F::BirthDate, L::Strong) => 10.0,
        (F::BirthDate, L::Partial) => 6.0,
        (F::BirthDate, L::Weak) => 0.5,
        (F::BirthDate, L::Conflict) => -20.0,

        (F::Address, L::Exact) => 12.0,
        (F::Address, L::Strong) => 8.0,
        // A shared PSČ is ~1/2600 of the country but correlates hard with the city.
        (F::Address, L::Partial) => 5.0,
        (F::Address, L::Weak) => 2.5,
        // People move. A different address is not evidence of a different person.
        (F::Address, L::Conflict) => 0.0,

        (F::Email, L::Exact) => 20.0,
        // Handle reuse across providers: real signal, weak on its own.
        (F::Email, L::Partial) => 3.0,
        (F::Email, _) => 0.0,

        (F::Phone, L::Exact) => 18.0,
        (F::Phone, _) => 0.0,

        (F::Ico, L::Exact) => 21.0,
        // A company the user has no part in says nothing about the person.
        (F::Ico, _) => 0.0,
    }
}

/// Fields that can tell two namesakes apart. A name and a town cannot, however many of
/// them agree, so they are excluded here however high the total climbs.
pub fn is_discriminating(field: Field, level: Level) -> bool {
    use Field as F;
    use Level as L;
    matches!(
        (field, level),
        (F::BirthDate, L::Exact | L::Strong | L::Partial)
            | (F::Address, L::Exact | L::Strong)
            | (F::Email | F::Phone | F::Ico, L::Exact)
    )
}

/// Bits of discriminating evidence required before a record may be called the user's.
/// Set so that a full date of birth clears it alone, while month-and-year needs an
/// address to go with it.
pub const DISCRIMINATING_GATE: f64 = 12.0;

/// Total weight for a match. Roughly one in a million by coincidence.
pub const MATCH_BITS: f64 = 20.0;

/// Below this there is nothing worth showing the user.
pub const POSSIBLE_BITS: f64 = 10.0;
