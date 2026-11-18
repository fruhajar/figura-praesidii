//! Domain types shared by every stage: what a source published, what the user asserts
//! about themselves, and what can legally be done about the overlap.

pub mod address;
pub mod contact;
pub mod date;
pub mod ico;
pub mod ident;
pub mod name;
pub mod protect;
pub mod record;
pub mod source;
pub mod text;

pub use address::Address;
pub use date::BirthDate;
pub use ico::Ico;
pub use ident::{Identity, Rarity};
pub use name::PersonName;
pub use protect::{Protection, Remedy, protection_for};
pub use record::{Attribute, Record};
pub use source::{Catalog, Controller, NameFidelity, Source, SourceKind};
