//! The pipeline: query each source in the user's locality, extract records, decide
//! which ones are the user's, and attach the route for getting each removed.

use fp_core::{Identity, Protection, Record, Source, protection_for};
use fp_correlate::{Linkage, Verdict, link};

use crate::catalog::{Catalog, SourceEntry};
use crate::fetch::{FetchError, Fetcher};
use crate::query::Query;

#[derive(Debug)]
pub struct Finding {
    pub source: Source,
    /// False means the URL template or the labels behind this finding are unchecked.
    pub verified: bool,
    pub record: Record,
    pub linkage: Linkage,
    pub protection: Protection,
}

#[derive(Debug, Default)]
pub struct Report {
    /// Only `Match` and `Possible`, strongest first.
    pub findings: Vec<Finding>,
    /// Records ruled out as another person. Counted, never kept: holding a stranger's
    /// record would be the exact harm this tool exists to undo.
    pub excluded: usize,
    /// Seen, too little in common to report.
    pub dismissed: usize,
    pub skipped: Vec<String>,
    pub errors: Vec<(String, String)>,
    pub unverified: Vec<String>,
    pub requests: u32,
}

pub fn scan(
    id: &Identity,
    catalog: &Catalog,
    fetcher: &mut dyn Fetcher,
    locality: Option<&str>,
    observed: &str,
) -> Report {
    let queries = Query::from_identity(id, locality);
    let mut report = Report::default();

    for entry in catalog.sources.iter().filter(|e| e.enabled) {
        if !entry.verified {
            report.unverified.push(entry.id.clone());
        }
        for query in &queries {
            let Some(url) = entry.url_for(query) else {
                report
                    .skipped
                    .push(format!("{}: needs a locality, none given", entry.id));
                continue;
            };
            match fetcher.get(&url, entry.query.min_interval_ms) {
                Ok(body) => collect(entry, &body, &url, observed, id, &mut report),
                Err(FetchError::Budget) => {
                    report
                        .errors
                        .push((entry.id.clone(), FetchError::Budget.to_string()));
                    break;
                }
                Err(e) => report.errors.push((entry.id.clone(), e.to_string())),
            }
        }
    }

    report.findings.sort_by(|a, b| {
        rank(a.linkage.verdict)
            .cmp(&rank(b.linkage.verdict))
            .then(b.linkage.bits.total_cmp(&a.linkage.bits))
    });
    report.requests = fetcher.requests();
    report
}

fn collect(
    entry: &SourceEntry,
    body: &str,
    url: &str,
    observed: &str,
    id: &Identity,
    report: &mut Report,
) {
    for record in entry.records(body, url, observed) {
        let linkage = link(id, &record, entry.kind);
        match linkage.verdict {
            Verdict::Excluded => report.excluded += 1,
            Verdict::Unrelated => report.dismissed += 1,
            Verdict::Match | Verdict::Possible => report.findings.push(Finding {
                source: entry.source(),
                verified: entry.verified,
                record,
                linkage,
                protection: protection_for(entry.kind),
            }),
        }
    }
}

fn rank(v: Verdict) -> u8 {
    match v {
        Verdict::Match => 0,
        Verdict::Possible => 1,
        Verdict::Unrelated => 2,
        Verdict::Excluded => 3,
    }
}
