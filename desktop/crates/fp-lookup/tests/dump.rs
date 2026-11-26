use std::path::{Path, PathBuf};

use fp_core::{Identity, name::surname_root};
use fp_lookup::{Catalog, OfflineFetcher, ScanOptions, scan, xml};

fn fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/dump-sample.xml")
}

fn me() -> Identity {
    serde_json::from_str(
        r#"{"names":["Jana Dvořáková"],
            "birth_date":"14.3.1981",
            "localities":["Nuselská 123/45, Praha 4, 140 00"]}"#,
    )
    .unwrap()
}

fn block_for(id: &Identity) -> xml::Block {
    xml::Block {
        surname_roots: id
            .names
            .iter()
            .filter_map(|n| n.surname.as_ref().map(|s| surname_root(s)))
            .collect(),
    }
}

fn catalog() -> Catalog {
    Catalog::builtin()
}

#[test]
fn the_dump_yields_the_two_namesakes_and_keeps_nobody_else() {
    let catalog = catalog();
    let entry = catalog.get("dataor.justice.cz").unwrap();
    let outcome = entry
        .dump_records(&fixture(), "2026-11-26", &block_for(&me()))
        .unwrap();

    // Four officers in the file; only the two Dvořákovás are ever held.
    assert_eq!(outcome.seen, 4);
    assert_eq!(outcome.blocked, 2);
    assert_eq!(outcome.records.len(), 2);
    assert!(!outcome.truncated);

    let first = &outcome.records[0];
    assert_eq!(first.names().next().unwrap().raw, "Jana Dvořáková");
    assert_eq!(first.birth_dates().next().unwrap().1.year, 1981);
    let address = first.addresses().next().unwrap();
    assert_eq!(address.street.as_deref(), Some("nuselska"));
    assert_eq!(address.number.as_deref(), Some("123/45"));
    assert_eq!(address.psc.as_deref(), Some("14000"));
}

#[test]
fn subject_fields_follow_the_officer_they_belong_to() {
    let catalog = catalog();
    let entry = catalog.get("dataor.justice.cz").unwrap();
    let outcome = entry
        .dump_records(&fixture(), "2026-11-26", &block_for(&me()))
        .unwrap();

    let raws: Vec<Vec<&str>> = outcome
        .records
        .iter()
        .map(|r| r.attrs.iter().map(|a| a.raw()).collect())
        .collect();
    assert!(raws[0].contains(&"Pekárna Nusle s.r.o."), "{raws:?}");
    assert!(raws[1].contains(&"Jiná firma a.s."), "{raws:?}");
    // One company's name must never land on another company's officer.
    assert!(!raws[0].contains(&"Jiná firma a.s."), "{raws:?}");
}

/// The reader splits text at every entity reference, and the dumps use one per
/// diacritic. Getting this wrong truncates every Czech name at its first `ř` and eats
/// the spaces around it, which looks like data rather than a bug.
#[test]
fn entity_references_do_not_truncate_or_glue_text() {
    let catalog = catalog();
    let entry = catalog.get("dataor.justice.cz").unwrap();
    let outcome = entry
        .dump_records(&fixture(), "2026-11-26", &block_for(&me()))
        .unwrap();

    let second: Vec<&str> = outcome.records[1].attrs.iter().map(|a| a.raw()).collect();
    assert!(second.contains(&"Jana Dvořáková"), "{second:?}");
    // Two entities in a row, then a space: `dozor&#269;&#237; rady`.
    assert!(second.contains(&"členka dozorčí rady"), "{second:?}");
    assert!(second.contains(&"Jiná firma a.s."), "{second:?}");
    assert!(
        outcome.records[1]
            .addresses()
            .next()
            .unwrap()
            .raw
            .starts_with("Veveří 7")
    );
}

/// The letter has to cite the register entry, not a file on the user's laptop.
#[test]
fn the_locator_points_at_the_register_not_the_file() {
    let catalog = catalog();
    let entry = catalog.get("dataor.justice.cz").unwrap();
    let outcome = entry
        .dump_records(&fixture(), "2026-11-26", &block_for(&me()))
        .unwrap();
    assert_eq!(
        outcome.records[0].locator.as_deref(),
        Some("https://ares.gov.cz/ekonomicke-subjekty?ico=00000019")
    );
}

#[test]
fn a_record_limit_is_reported_rather_than_silently_truncating() {
    let mut catalog = catalog();
    let entry = catalog
        .sources
        .iter_mut()
        .find(|s| s.id == "dataor.justice.cz")
        .unwrap();
    entry.dump.as_mut().unwrap().max_records = 1;
    let entry = catalog.get("dataor.justice.cz").unwrap();

    let outcome = entry
        .dump_records(&fixture(), "2026-11-26", &block_for(&me()))
        .unwrap();
    assert_eq!(outcome.records.len(), 1);
    assert!(outcome.truncated);
}

#[test]
fn scanning_a_dump_finds_me_and_excludes_the_namesake() {
    let id = me();
    let catalog = catalog();
    let mut fetcher = OfflineFetcher::new(std::env::temp_dir().join("fp-dump-nocache"));
    let dumps = vec![fixture()];
    let opts = ScanOptions {
        locality: Some("praha"),
        observed: "2026-11-26",
        dumps: &dumps,
    };

    let report = scan(&id, &catalog, &mut fetcher, &opts);

    let mine: Vec<_> = report
        .findings
        .iter()
        .filter(|f| f.source.id == "dataor.justice.cz")
        .collect();
    assert_eq!(mine.len(), 1, "{:#?}", report.findings);
    assert_eq!(mine[0].linkage.verdict, fp_correlate::Verdict::Match);
    // The 1974 namesake was ruled out, and the two other-surname officers never counted.
    assert_eq!(report.excluded, 1);
    assert_eq!(report.dump_seen, 4);
    assert_eq!(report.dump_blocked, 2);
}

#[test]
fn without_a_dump_the_source_says_where_to_get_one() {
    let id = me();
    let catalog = catalog();
    let mut fetcher = OfflineFetcher::new(std::env::temp_dir().join("fp-dump-nocache"));
    let opts = ScanOptions {
        locality: Some("praha"),
        observed: "2026-11-26",
        dumps: &[],
    };

    let report = scan(&id, &catalog, &mut fetcher, &opts);
    assert!(
        report
            .skipped
            .iter()
            .any(|s| s.contains("dataor.justice.cz") && s.contains("dataor")),
        "{:?}",
        report.skipped
    );
}
