use fp_core::{Identity, Protection, Record, Source, SourceKind, protection_for};
use fp_correlate::{Verdict, link};
use fp_request::{Disclosure, RequestKind, build};

fn me() -> Identity {
    serde_json::from_str(
        r#"{"names":["Jana Dvořáková"],
            "birth_date":"14.3.1981",
            "localities":["Nuselská 123/45, Praha 4, 140 00"],
            "emails":["j.dvorakova@gmail.com"],
            "phones":["606 123 456"]}"#,
    )
    .unwrap()
}

fn record(attrs: &str) -> Record {
    serde_json::from_str(&format!(
        r#"{{"id":"r1","source":"s","locator":"https://example.test/osoba/1","attrs":{attrs}}}"#
    ))
    .unwrap()
}

fn full_record() -> Record {
    record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"14.3.1981"},
            {"kind":"address","value":"Nuselská 123/45, Praha 4, 140 00"}]"#,
    )
}

fn source(kind: SourceKind, controller: bool, copies_from: Option<&str>) -> Source {
    serde_json::from_str(&format!(
        r#"{{"id":"src.test","title":"Zdroj","kind":"{}"{}{}}}"#,
        match kind {
            SourceKind::PublicRegister => "public_register",
            SourceKind::CommercialMirror => "commercial_mirror",
            SourceKind::SearchEngine => "search_engine",
            SourceKind::BreachCorpus => "breach_corpus",
            SourceKind::DataBroker => "data_broker",
            SourceKind::SocialProfile => "social_profile",
        },
        if controller {
            r#","controller":{"name":"Správce s.r.o.","email":"gdpr@src.test"}"#
        } else {
            ""
        },
        copies_from
            .map(|c| format!(r#","copies_from":"{c}""#))
            .unwrap_or_default()
    ))
    .unwrap()
}

struct Case {
    source: Source,
    record: Record,
    linkage: fp_correlate::Linkage,
    protection: Protection,
}

fn case(kind: SourceKind, rec: Record, controller: bool, copies_from: Option<&str>) -> Case {
    let source = source(kind, controller, copies_from);
    let linkage = link(&me(), &rec, kind);
    Case {
        source,
        record: rec,
        linkage,
        protection: protection_for(kind),
    }
}

impl Case {
    fn disclosure(&self, verified: bool) -> Disclosure<'_> {
        Disclosure {
            source: &self.source,
            record: &self.record,
            linkage: &self.linkage,
            protection: &self.protection,
            verified,
            copies_from_title: None,
        }
    }
}

#[test]
fn a_register_gets_an_access_request_and_no_erasure_demand() {
    let c = case(SourceKind::PublicRegister, full_record(), true, None);
    let r = build(&c.disclosure(true));

    assert_eq!(r.kind, RequestKind::Access);
    assert!(r.body.contains("čl. 15 odst. 1 GDPR"));
    assert!(r.body.contains("čl. 17 odst. 3 písm. b) GDPR"));
    assert!(r.body.contains("Tato žádost proto výmaz nepožaduje"));
    // The thing that makes the access request worth sending: the recipients list is
    // how the mirrors get found.
    assert!(r.body.contains("příjemcích"));
    assert!(!r.body.contains("Žádám o výmaz"));
}

#[test]
fn a_mirror_gets_the_objection_and_names_the_register_it_copies() {
    let c = case(
        SourceKind::CommercialMirror,
        full_record(),
        true,
        Some("obchodní rejstřík"),
    );
    let r = build(&c.disclosure(true));

    assert_eq!(r.kind, RequestKind::ObjectAndErase);
    assert!(r.body.contains("čl. 21 odst. 1 GDPR"));
    assert!(r.body.contains("čl. 17 odst. 1 písm. c) GDPR"));
    assert!(
        r.body
            .contains("Povinnost zveřejnění má podle zákona obchodní rejstřík")
    );
    assert!(r.body.contains("Správce s.r.o."));
    assert!(r.warnings.is_empty(), "{:?}", r.warnings);
}

#[test]
fn a_mirror_without_a_named_register_warns_and_leaves_a_placeholder() {
    let c = case(SourceKind::CommercialMirror, full_record(), true, None);
    let r = build(&c.disclosure(true));
    assert!(r.body.contains("[doplňte rejstřík"));
    assert!(r.warnings.iter().any(|w| w.contains("copies_from")));
}

#[test]
fn a_search_engine_gets_delisting_on_costeja() {
    let c = case(SourceKind::SearchEngine, full_record(), true, None);
    let r = build(&c.disclosure(true));
    assert_eq!(r.kind, RequestKind::Delist);
    assert!(r.body.contains("C-131/12"));
    assert!(r.body.contains("C-507/17"));
    assert!(
        r.body
            .contains("nevyžaduje žádný zásah do zdrojové stránky")
    );
}

#[test]
fn a_breach_corpus_gets_plain_erasure_plus_recipients() {
    let c = case(SourceKind::BreachCorpus, full_record(), true, None);
    let r = build(&c.disclosure(true));
    assert_eq!(r.kind, RequestKind::Erase);
    assert!(r.body.contains("čl. 17 odst. 1 písm. a) GDPR"));
    assert!(r.body.contains("čl. 19 GDPR"));
}

/// The letter must not hand the recipient an identifier they never had.
#[test]
fn identification_discloses_nothing_the_source_did_not_publish() {
    let c = case(SourceKind::CommercialMirror, full_record(), true, None);
    let r = build(&c.disclosure(true));

    assert!(r.body.contains("Jana Dvořáková"));
    assert!(r.body.contains("14.3.1981"));
    assert!(!r.body.contains("gmail"), "e-mail leaked into the letter");
    assert!(
        !r.body.contains("606"),
        "phone number leaked into the letter"
    );
}

#[test]
fn a_possible_match_is_flagged_before_anything_is_sent() {
    let name_and_town = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"address","value":"Praha"}]"#,
    );
    let c = case(SourceKind::CommercialMirror, name_and_town, true, None);
    assert_eq!(c.linkage.verdict, Verdict::Possible);

    let r = build(&c.disclosure(true));
    assert!(
        r.warnings.iter().any(|w| w.contains("namesake")),
        "{:?}",
        r.warnings
    );
}

#[test]
fn an_unverified_source_and_a_missing_recipient_both_warn() {
    let c = case(
        SourceKind::DataBroker,
        full_record(),
        false,
        Some("rejstřík"),
    );
    let r = build(&c.disclosure(false));
    assert!(r.warnings.iter().any(|w| w.contains("unverified")));
    assert!(r.warnings.iter().any(|w| w.contains("No recipient")));
    assert!(r.body.contains("[doplňte správce"));
}

#[test]
fn filenames_are_safe_and_say_what_the_letter_is() {
    let c = case(SourceKind::PublicRegister, full_record(), true, None);
    let r = build(&c.disclosure(true));
    assert_eq!(r.filename(), "src-test-r1-pristup-cl15.txt");
}
