//! Discovery: what the public sources hold, found by name within a locality.
//!
//! Three deliberate limits. Only GET requests against sources a user could open in a
//! browser themselves; one locality at a time, because a nationwide name sweep is a
//! namesake list and not a self-lookup; and every URL template kept as data with a
//! `verified` flag, so an unchecked guess cannot pass itself off as a clean result.

pub mod catalog;
pub mod fetch;
pub mod html;
pub mod query;
pub mod scan;
pub mod xml;

pub use catalog::{Catalog, SourceEntry};
pub use fetch::{Cache, FetchError, Fetcher, OfflineFetcher};
pub use query::Query;
pub use scan::{Finding, Report, ScanOptions, scan};

#[cfg(feature = "http")]
pub use fetch::HttpFetcher;
