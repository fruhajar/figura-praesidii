//! The pipeline: query each source in the user's locality, extract records, decide
//! which ones are the user's, and attach the route for getting each removed.

use std::path::{Path, PathBuf};

use fp_core::{Identity, Protection, Record, Source, name::surname_root, protection_for};
use fp_correlate::{Linkage, Verdict, link};

use crate::catalog::{Catalog, SourceEntry};
use crate::fetch::{FetchError, Fetcher};
use crate::query::Query;
use crate::xml;

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
    /// Person records read out of a bulk dump, before the surname block.
    pub dump_seen: usize,
    /// Dropped by the surname block, so never held at all.
    pub dump_blocked: usize,
    /// Sources whose dump hit a limit, where an absent finding proves nothing.
    pub truncated: Vec<String>,
}

/// Everything a scan needs besides the identity and the catalog.
pub struct ScanOptions<'a> {
    pub locality: Option<&'a str>,
    /// ISO date the run happened on.
    pub observed: &'a str,
    /// Bulk open-data files the user has downloaded.
    pub dumps: &'a [PathBuf],
}

pub fn scan(
    id: &Identity,
    catalog: &Catalog,
    fetcher: &mut dyn Fetcher,
    opts: &ScanOptions<'_>,
) -> Report {
    let queries = Query::from_identity(id, opts.locality);
    let observed = opts.observed;
    let mut report = Report::default();

    for entry in catalog.sources.iter().filter(|e| e.enabled) {
        if !entry.verified {
            report.unverified.push(entry.id.clone());
        }
        if entry.dump.is_some() {
            read_dumps(entry, opts.dumps, observed, id, &mut report);
            continue;
        }
        for query in &queries {
            let Some(url) = entry.url_for(query) else {
                report
                    .skipped
                    .push(format!("{}: needs a locality, none given", entry.id));
                continue;
            };
            match fetcher.get(&url, entry.min_interval_ms()) {
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

/// A dump is read from disk, never fetched: the files are bulk open data the user
/// downloads once, which is also why there is no rate limit to respect here.
fn read_dumps(
    entry: &SourceEntry,
    dumps: &[PathBuf],
    observed: &str,
    id: &Identity,
    report: &mut Report,
) {
    if dumps.is_empty() {
        let hint = entry.dump_hint().unwrap_or_default();
        report.skipped.push(format!(
            "{}: no dump supplied. Download one from {hint}",
            entry.id
        ));
        return;
    }
    // The block is what keeps a court-year of strangers out of the report, so an
    // identity with no usable surname must not silently match everyone.
    let block = xml::Block {
        surname_roots: id
            .names
            .iter()
            .filter_map(|n| n.surname.as_ref().map(|s| surname_root(s)))
            .collect(),
    };
    if block.surname_roots.is_empty() {
        report
            .skipped
            .push(format!("{}: identity has no surname to match on", entry.id));
        return;
    }

    for path in dumps {
        match entry.dump_records(Path::new(path), observed, &block) {
            Ok(outcome) => {
                report.dump_seen += outcome.seen;
                report.dump_blocked += outcome.blocked;
                if outcome.truncated {
                    report
                        .truncated
                        .push(format!("{}: {}", entry.id, path.display()));
                }
                for record in outcome.records {
                    judge(entry, record, id, report);
                }
            }
            Err(e) => report.errors.push((entry.id.clone(), e)),
        }
    }
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
        judge(entry, record, id, report);
    }
}

fn judge(entry: &SourceEntry, record: Record, id: &Identity, report: &mut Report) {
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

fn rank(v: Verdict) -> u8 {
    match v {
        Verdict::Match => 0,
        Verdict::Possible => 1,
        Verdict::Unrelated => 2,
        Verdict::Excluded => 3,
    }
}
