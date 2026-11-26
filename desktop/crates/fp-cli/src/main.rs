//! `fp`: run a self-lookup, show the evidence, name the route.

mod report;

use std::collections::{HashMap, HashSet};
use std::path::PathBuf;
use std::process::ExitCode;
use std::time::{SystemTime, UNIX_EPOCH};

use fp_core::Identity;
use fp_correlate::Verdict;
use fp_lookup::{Catalog, Fetcher, OfflineFetcher, Query, ScanOptions, scan};
use fp_request::Disclosure;

const USAGE: &str = "\
fp, the Figura Praesidii self-lookup

  fp scan --identity me.json [options]   look yourself up and show what is exposed
  fp requests --identity me.json [opts]  draft the GDPR request for each finding
  fp sources [--catalog c.json]          list the sources and whether they are checked
  fp verify-sources [options]            check each source against the live site

Options
  --identity <file>   JSON description of yourself (required for scan)
  --catalog <file>    source catalog; the built-in one is used when omitted
  --locality <town>   scope the search; defaults to the town in your identity file
  --cache <dir>       store and reuse fetched pages here
  --dump <file>       a register open-data file to read (see `fp sources`)
  --dumps <dir>       read every .xml in this directory as a register dump
  --offline           serve only from the cache, send nothing
  --budget <n>        maximum requests per run (default 20)
  --out <file>        also write the report to a file. Off by default: the report is a
                      dossier on you, so saving it is your decision, not ours.
  --out-dir <dir>     requests only: write one letter per file into this directory
  --include-possible  requests only: also draft letters for records that could still
                      be a namesake's. Read the evidence before sending any of them.

An identity file:
  {
    \"names\": [\"Jana Dvořáková\", \"Jana Marková\"],
    \"birth_date\": \"14.3.1981\",
    \"localities\": [\"Nuselská 123/45, Praha 4, 140 00\"],
    \"emails\": [\"jana@example.cz\"],
    \"name_rarity\": \"common\"
  }
";

fn main() -> ExitCode {
    let argv: Vec<String> = std::env::args().skip(1).collect();
    let Some(command) = argv.first().cloned() else {
        print!("{USAGE}");
        return ExitCode::SUCCESS;
    };
    let args = Args::parse(&argv[1..]);

    let result = match command.as_str() {
        "scan" => cmd_scan(&args),
        "requests" => cmd_requests(&args),
        "sources" => cmd_sources(&args),
        "verify-sources" => cmd_verify(&args),
        "help" | "--help" | "-h" => {
            print!("{USAGE}");
            return ExitCode::SUCCESS;
        }
        other => Err(format!("unknown command {other:?}. Try `fp help`.")),
    };

    match result {
        Ok(()) => ExitCode::SUCCESS,
        Err(e) => {
            eprintln!("fp: {e}");
            ExitCode::FAILURE
        }
    }
}

struct Args {
    values: HashMap<String, String>,
    switches: HashSet<String>,
}

impl Args {
    fn parse(argv: &[String]) -> Self {
        let mut values = HashMap::new();
        let mut switches = HashSet::new();
        let mut i = 0;
        while i < argv.len() {
            let arg = argv[i].trim_start_matches("--").to_string();
            match argv.get(i + 1) {
                Some(next) if !next.starts_with("--") => {
                    values.insert(arg, next.clone());
                    i += 2;
                }
                _ => {
                    switches.insert(arg);
                    i += 1;
                }
            }
        }
        Self { values, switches }
    }

    fn get(&self, key: &str) -> Option<&str> {
        self.values.get(key).map(String::as_str)
    }

    fn has(&self, key: &str) -> bool {
        self.switches.contains(key)
    }

    fn catalog(&self) -> Result<Catalog, String> {
        match self.get("catalog") {
            Some(path) => Catalog::load(&PathBuf::from(path)).map_err(|e| format!("{path}: {e}")),
            None => Ok(Catalog::builtin()),
        }
    }
}

fn load_identity(args: &Args) -> Result<Identity, String> {
    let path = args.get("identity").ok_or("scan needs --identity <file>")?;
    let text = std::fs::read_to_string(path).map_err(|e| format!("{path}: {e}"))?;
    serde_json::from_str(&text).map_err(|e| format!("{path}: {e}"))
}

/// Bulk dumps the user has downloaded. Read from disk, never fetched.
fn dumps(args: &Args) -> Result<Vec<PathBuf>, String> {
    let mut out = Vec::new();
    if let Some(file) = args.get("dump") {
        out.push(PathBuf::from(file));
    }
    if let Some(dir) = args.get("dumps") {
        let entries = std::fs::read_dir(dir).map_err(|e| format!("{dir}: {e}"))?;
        for entry in entries {
            let path = entry.map_err(|e| format!("{dir}: {e}"))?.path();
            if path
                .extension()
                .is_some_and(|e| e.eq_ignore_ascii_case("xml"))
            {
                out.push(path);
            }
        }
        out.sort();
    }
    for path in &out {
        if !path.is_file() {
            return Err(format!("{}: not a file", path.display()));
        }
    }
    Ok(out)
}

fn fetcher(args: &Args) -> Result<Box<dyn Fetcher>, String> {
    let cache = args.get("cache").map(fp_lookup::Cache::new);
    let budget: u32 = args
        .get("budget")
        .unwrap_or("20")
        .parse()
        .map_err(|_| "--budget wants a number")?;

    if args.has("offline") {
        let dir = args
            .get("cache")
            .ok_or("--offline needs --cache <dir> to read from")?;
        return Ok(Box::new(OfflineFetcher::new(dir)));
    }
    #[cfg(feature = "http")]
    {
        Ok(Box::new(fp_lookup::HttpFetcher::new(budget, cache)))
    }
    #[cfg(not(feature = "http"))]
    {
        let _ = (cache, budget);
        Err("this build has no HTTP support; use --offline --cache <dir>".into())
    }
}

fn cmd_scan(args: &Args) -> Result<(), String> {
    let id = load_identity(args)?;
    let catalog = args.catalog()?;
    let mut fetcher = fetcher(args)?;
    let today = today();

    let opts = ScanOptions {
        locality: args.get("locality"),
        observed: &today,
        dumps: &dumps(args)?,
    };
    let report = scan(&id, &catalog, fetcher.as_mut(), &opts);
    let text = report::render(&report, &id, args.get("locality"), &today);
    print!("{text}");

    if let Some(out) = args.get("out") {
        std::fs::write(out, &text).map_err(|e| format!("{out}: {e}"))?;
        eprintln!("fp: written to {out}. Keep it out of any synced folder.");
    }
    Ok(())
}

/// Drafts a letter per finding. Deliberately a separate command rather than part of
/// `scan`: a request asserts to a controller that the record is yours, so it should
/// take a second deliberate step after the evidence has been read.
fn cmd_requests(args: &Args) -> Result<(), String> {
    let id = load_identity(args)?;
    let catalog = args.catalog()?;
    let mut fetcher = fetcher(args)?;
    let today = today();

    let opts = ScanOptions {
        locality: args.get("locality"),
        observed: &today,
        dumps: &dumps(args)?,
    };
    let report = scan(&id, &catalog, fetcher.as_mut(), &opts);
    let include_possible = args.has("include-possible");

    let mut requests = Vec::new();
    let mut held_back = 0;
    for f in &report.findings {
        if f.linkage.verdict != Verdict::Match && !include_possible {
            held_back += 1;
            continue;
        }
        requests.push(fp_request::build(&Disclosure {
            source: &f.source,
            record: &f.record,
            linkage: &f.linkage,
            protection: &f.protection,
            verified: f.verified,
            copies_from_title: f
                .source
                .copies_from
                .as_deref()
                .and_then(|id| catalog.get(id))
                .map(|e| e.title.clone()),
        }));
    }

    print!("{}", report::render_requests(&requests, held_back, &report));

    if let Some(dir) = args.get("out-dir") {
        std::fs::create_dir_all(dir).map_err(|e| format!("{dir}: {e}"))?;
        for r in &requests {
            let path = PathBuf::from(dir).join(r.filename());
            std::fs::write(&path, &r.body).map_err(|e| format!("{}: {e}", path.display()))?;
        }
        eprintln!(
            "fp: {} letter(s) written to {dir}. Read the warnings above before sending any.",
            requests.len()
        );
    }
    Ok(())
}

fn cmd_sources(args: &Args) -> Result<(), String> {
    let catalog = args.catalog()?;
    println!(
        "{:<20} {:<18} {:<9} {:<8} title",
        "id", "kind", "checked", "enabled"
    );
    for s in &catalog.sources {
        println!(
            "{:<20} {:<18} {:<9} {:<8} {}",
            s.id,
            s.kind.label(),
            if s.verified { "yes" } else { "NO" },
            if s.enabled { "yes" } else { "no" },
            s.title
        );
        if let Some(hint) = s.dump_hint() {
            println!("  bulk open data, download from {hint}");
        }
        if let Some(notes) = &s.notes {
            println!("  {notes}");
        }
    }
    Ok(())
}

/// Reachability and label presence, which is all an automated check can honestly claim.
/// Whether the right field was picked up is still a human's call.
fn cmd_verify(args: &Args) -> Result<(), String> {
    let catalog = args.catalog()?;
    let id = load_identity(args)?;
    let queries = Query::from_identity(&id, args.get("locality"));
    let query = queries.first().ok_or("identity has no usable name")?;
    let mut fetcher = fetcher(args)?;

    for entry in catalog.sources.iter().filter(|e| e.enabled) {
        let Some(url) = entry.url_for(query) else {
            println!("{:<20} skipped (needs a locality)", entry.id);
            continue;
        };
        match fetcher.get(&url, entry.min_interval_ms()) {
            Ok(body) => {
                let records = entry.records(&body, &url, &today());
                let labels: Vec<&str> = entry
                    .extract
                    .fields
                    .iter()
                    .filter(|f| {
                        fp_lookup::html::label_value(&fp_lookup::html::to_lines(&body), &f.labels)
                            .is_some()
                    })
                    .map(|f| f.kind.as_str())
                    .collect();
                println!(
                    "{:<20} reachable, {} record(s), labels found: {}",
                    entry.id,
                    records.len(),
                    if labels.is_empty() {
                        "none".to_string()
                    } else {
                        labels.join(", ")
                    }
                );
                if records.is_empty() {
                    println!(
                        "  no records parsed. Either the name is absent, or the labels are wrong."
                    );
                }
            }
            Err(e) => println!("{:<20} {e}", entry.id),
        }
    }
    println!("\nMark a source verified in the catalog only after reading a page by hand.");
    Ok(())
}

/// Days since the epoch to a civil date, so the crate needs no clock dependency.
fn today() -> String {
    let days = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs() / 86_400)
        .unwrap_or(0) as i64;
    let (y, m, d) = civil_from_days(days);
    format!("{y:04}-{m:02}-{d:02}")
}

/// Howard Hinnant's algorithm, shifted to a March-based year.
fn civil_from_days(days: i64) -> (i64, u32, u32) {
    let z = days + 719_468;
    let era = z.div_euclid(146_097);
    let doe = z.rem_euclid(146_097);
    let yoe = (doe - doe / 1460 + doe / 36_524 - doe / 146_096) / 365;
    let y = yoe + era * 400;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let d = (doy - (153 * mp + 2) / 5 + 1) as u32;
    let m = if mp < 10 { mp + 3 } else { mp - 9 } as u32;
    (if m <= 2 { y + 1 } else { y }, m, d)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dates_round_trip() {
        assert_eq!(civil_from_days(0), (1970, 1, 1));
        assert_eq!(civil_from_days(20_775), (2026, 11, 18));
    }

    #[test]
    fn args_split_values_from_switches() {
        let argv: Vec<String> = ["--identity", "me.json", "--offline", "--budget", "5"]
            .iter()
            .map(|s| s.to_string())
            .collect();
        let args = Args::parse(&argv);
        assert_eq!(args.get("identity"), Some("me.json"));
        assert_eq!(args.get("budget"), Some("5"));
        assert!(args.has("offline"));
    }
}
