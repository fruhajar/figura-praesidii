use std::fs;

use fp_core::Identity;
use fp_lookup::{Cache, Catalog, OfflineFetcher, ScanOptions, scan};

/// Two officers on one results page: the user, and a namesake born in another year.
const PAGE: &str = r#"
<html><body>
<table>
 <tr><td>Obchodní firma:</td><td>Pekárna Nusle s.r.o.</td></tr>
 <tr><td>Jméno:</td><td>Jana Dvo&#345;áková</td></tr>
 <tr><td>dat. nar.</td><td>14. b&#345;ezna 1981</td></tr>
 <tr><td>Bydliště:</td><td>Nuselská 123/45, Praha 4 - Nusle, 140 00</td></tr>
 <tr><td>Funkce:</td><td>jednatel</td></tr>
</table>
<table>
 <tr><td>Obchodní firma:</td><td>Jiná firma a.s.</td></tr>
 <tr><td>Jméno:</td><td>Jana Dvo&#345;áková</td></tr>
 <tr><td>dat. nar.</td><td>2. zá&#345;í 1974</td></tr>
 <tr><td>Bydliště:</td><td>Veveří 7, Brno, 602 00</td></tr>
</table>
</body></html>
"#;

fn catalog() -> Catalog {
    serde_json::from_str(
        r#"{"sources":[{
            "id":"register.test",
            "title":"Test register",
            "kind":"public_register",
            "verified":false,
            "query":{"url":"https://register.test/search?nazev={name}&obec={locality}"},
            "extract":{
                "record_split":"Jméno:",
                "fields":[
                    {"kind":"name","labels":["Jméno:"]},
                    {"kind":"birth_date","labels":["dat. nar."]},
                    {"kind":"address","labels":["Bydliště:"]},
                    {"kind":"function","labels":["Funkce:"]}
                ]}
        }]}"#,
    )
    .unwrap()
}

fn me() -> Identity {
    serde_json::from_str(
        r#"{"names":["Jana Dvořáková"],
            "birth_date":"14.3.1981",
            "localities":["Nuselská 123/45, Praha 4, 140 00"]}"#,
    )
    .unwrap()
}

#[test]
fn finds_the_user_and_drops_the_namesake() {
    let dir = std::env::temp_dir().join("fp-scan-test");
    let _ = fs::remove_dir_all(&dir);
    let cache = Cache::new(&dir);
    let catalog = catalog();
    let id = me();

    let url = catalog.sources[0]
        .url_for(&fp_lookup::Query::from_identity(&id, None)[0])
        .unwrap();
    cache.write(&url, PAGE).unwrap();

    let mut fetcher = OfflineFetcher::new(&dir);
    let opts = ScanOptions {
        locality: None,
        observed: "2026-11-18",
        dumps: &[],
    };
    let report = scan(&id, &catalog, &mut fetcher, &opts);

    assert_eq!(report.findings.len(), 1, "{report:#?}");
    assert_eq!(report.excluded, 1);
    let found = &report.findings[0];
    assert_eq!(found.linkage.verdict, fp_correlate::Verdict::Match);
    assert_eq!(found.protection.remedy, fp_core::Remedy::Immovable);
    // The locality came from the identity, so the query was scoped without being asked.
    assert!(url.contains("obec=praha"), "{url}");
    // An unverified template is reported as such rather than presented as a clean scan.
    assert_eq!(report.unverified, vec!["register.test"]);
    // The uncomparable field is kept for the report.
    assert!(found.record.attrs.iter().any(|a| a.raw() == "jednatel"));

    let _ = fs::remove_dir_all(&dir);
}

#[test]
fn an_offline_miss_is_an_error_not_an_empty_result() {
    let mut fetcher = OfflineFetcher::new(std::env::temp_dir().join("fp-scan-empty"));
    let opts = ScanOptions {
        locality: Some("praha"),
        observed: "2026-11-18",
        dumps: &[],
    };
    let report = scan(&me(), &catalog(), &mut fetcher, &opts);
    assert!(report.findings.is_empty());
    assert_eq!(report.errors.len(), 1);
    assert!(report.errors[0].1.contains("offline"));
}
