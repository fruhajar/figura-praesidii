use fp_core::{Identity, Record, SourceKind};
use fp_correlate::{Verdict, link, weights::Field};

fn me() -> Identity {
    serde_json::from_str(
        r#"{
            "names": ["Jana Dvořáková"],
            "birth_date": "14.3.1981",
            "localities": ["Nuselská 123/45, Praha 4, 140 00"],
            "emails": ["j.dvorakova@gmail.com"]
        }"#,
    )
    .unwrap()
}

fn record(attrs: &str) -> Record {
    serde_json::from_str(&format!(r#"{{"id":"r","source":"s","attrs":{attrs}}}"#)).unwrap()
}

#[test]
fn register_entry_with_name_date_and_address_is_mine() {
    let r = record(
        r#"[{"kind":"name","value":"DVOŘÁKOVÁ Jana"},
            {"kind":"birth_date","value":"14.3.1981"},
            {"kind":"address","value":"Nuselská 123/45, Praha 4, 140 00"}]"#,
    );
    let l = link(&me(), &r, SourceKind::PublicRegister);
    assert_eq!(l.verdict, Verdict::Match, "{l:#?}");
    assert!(l.bits > 30.0, "{} bits", l.bits);
}

#[test]
fn a_namesake_with_another_birth_date_is_excluded() {
    let r = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"2.9.1974"},
            {"kind":"address","value":"Nuselská 123/45, Praha 4"}]"#,
    );
    let l = link(&me(), &r, SourceKind::PublicRegister);
    assert_eq!(l.verdict, Verdict::Excluded);
    assert!(l.veto.as_deref().unwrap().contains("year"));
}

#[test]
fn another_person_at_my_address_is_excluded() {
    let r = record(
        r#"[{"kind":"name","value":"Petr Dvořák"},
            {"kind":"address","value":"Nuselská 123/45, Praha 4"}]"#,
    );
    let l = link(&me(), &r, SourceKind::PublicRegister);
    assert_eq!(l.verdict, Verdict::Excluded, "{l:#?}");
}

#[test]
fn name_and_town_alone_can_never_be_a_match() {
    let r = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"address","value":"Praha"}]"#,
    );
    let l = link(&me(), &r, SourceKind::CommercialMirror);
    assert_eq!(l.verdict, Verdict::Possible, "{l:#?}");
    assert_eq!(l.discriminating_bits, 0.0);
}

#[test]
fn month_and_year_need_an_address_to_clear_the_gate() {
    let bare = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"3/1981"}]"#,
    );
    assert_eq!(
        link(&me(), &bare, SourceKind::CommercialMirror).verdict,
        Verdict::Possible
    );

    let with_street = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"3/1981"},
            {"kind":"address","value":"Nuselská, Praha 4"}]"#,
    );
    assert_eq!(
        link(&me(), &with_street, SourceKind::CommercialMirror).verdict,
        Verdict::Match
    );
}

#[test]
fn a_breach_hit_on_my_address_is_mine() {
    let r = record(
        r#"[{"kind":"email","value":"J.Dvorakova+news@googlemail.com"},
            {"kind":"name","value":"jana dvorakova"}]"#,
    );
    let l = link(&me(), &r, SourceKind::BreachCorpus);
    assert_eq!(l.verdict, Verdict::Match, "{l:#?}");
}

#[test]
fn an_informal_source_is_not_vetoed_on_a_nickname() {
    let r = record(r#"[{"kind":"name","value":"Janicka Dvorakova"}]"#);
    let l = link(&me(), &r, SourceKind::SocialProfile);
    assert_ne!(l.verdict, Verdict::Excluded, "{l:#?}");
    // The same name in a register would be a different person.
    assert_eq!(
        link(&me(), &r, SourceKind::PublicRegister).verdict,
        Verdict::Excluded
    );
}

#[test]
fn moving_house_does_not_rule_me_out() {
    let r = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"14.3.1981"},
            {"kind":"address","value":"Veveří 7, Brno, 602 00"}]"#,
    );
    let l = link(&me(), &r, SourceKind::CommercialMirror);
    assert_eq!(l.verdict, Verdict::Match, "{l:#?}");
    let addr = l
        .evidence
        .iter()
        .find(|e| e.field == Field::Address)
        .unwrap();
    assert_eq!(addr.bits, 0.0);
}

#[test]
fn only_the_strongest_value_per_field_counts() {
    let r = record(
        r#"[{"kind":"name","value":"Jana Dvořáková"},
            {"kind":"birth_date","value":"14.3.1981"},
            {"kind":"address","value":"Praha"},
            {"kind":"address","value":"Nuselská 123/45, Praha 4"}]"#,
    );
    let l = link(&me(), &r, SourceKind::PublicRegister);
    assert_eq!(
        l.evidence
            .iter()
            .filter(|e| e.field == Field::Address)
            .count(),
        1
    );
}
