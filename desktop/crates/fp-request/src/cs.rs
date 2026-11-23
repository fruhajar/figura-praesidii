//! Czech letter text.
//!
//! Czech only for now: every recipient in the shipped catalog is a Czech controller,
//! and a request in the controller's own language is harder to ignore. Placeholders the
//! sender must fill in are in square brackets.

use std::fmt::Write as _;

use fp_correlate::weights::{Field, Level};

use crate::{Disclosure, RequestKind, category_cs};

pub fn subject(kind: RequestKind) -> &'static str {
    match kind {
        RequestKind::Access => "Žádost o přístup k osobním údajům podle čl. 15 GDPR",
        RequestKind::ObjectAndErase => {
            "Námitka proti zpracování podle čl. 21 GDPR a žádost o výmaz podle čl. 17 GDPR"
        }
        RequestKind::Delist => "Žádost o odstranění odkazů z výsledků vyhledávání na mé jméno",
        RequestKind::Erase => "Žádost o výmaz osobních údajů podle čl. 17 GDPR",
    }
}

pub fn letter(kind: RequestKind, d: &Disclosure<'_>) -> String {
    let mut out = String::new();

    let _ = writeln!(out, "Věc: {}\n", subject(kind));

    let _ = writeln!(out, "Adresát:");
    match &d.source.controller {
        Some(c) => {
            let _ = writeln!(out, "  {}", c.name);
            for line in [
                c.email.as_deref(),
                c.postal.as_deref(),
                c.dpo_email.as_deref(),
            ]
            .into_iter()
            .flatten()
            {
                let _ = writeln!(out, "  {line}");
            }
        }
        None => {
            let _ = writeln!(
                out,
                "  [doplňte správce a jeho kontaktní adresu z tiráže stránky]"
            );
        }
    }

    let _ = writeln!(out, "\nVážení,\n");
    let _ = writeln!(out, "{}\n", intro(kind));

    out.push_str(&identification(d));
    out.push_str(&categories(d));
    out.push_str(&sections(kind, d));

    let _ = writeln!(out, "Lhůta pro odpověď");
    let _ = writeln!(
        out,
        "Odpověď očekávám bez zbytečného odkladu, nejpozději do {} dnů od doručení této\n\
         žádosti (čl. 12 odst. 3 GDPR). Lhůtu lze prodloužit o další dva měsíce, pokud mě\n\
         o prodloužení a jeho důvodech informujete do jednoho měsíce od doručení.\n",
        d.protection.response_days
    );
    if let Some(authority) = d.protection.authority {
        let _ = writeln!(
            out,
            "Nebude-li žádosti vyhověno, podám stížnost u dozorového úřadu ({authority}),\n\
             čl. 77 GDPR.\n"
        );
    }

    let _ = writeln!(
        out,
        "Odpověď zasílejte na: [doplňte svůj kontaktní e-mail nebo poštovní adresu]\n\n\
         S pozdravem\n\n\
         [podpis]\n[jméno a příjmení]\n[datum]"
    );
    out
}

fn intro(kind: RequestKind) -> &'static str {
    match kind {
        RequestKind::Access => {
            "jako subjekt údajů podle čl. 4 odst. 1 nařízení (EU) 2016/679 (GDPR) žádám\n\
             o přístup k osobním údajům, které o mně zpracováváte."
        }
        RequestKind::ObjectAndErase => {
            "na níže uvedené adrese zpřístupňujete mé osobní údaje. Jako subjekt údajů\n\
             podle čl. 4 odst. 1 nařízení (EU) 2016/679 (GDPR) proti tomuto zpracování\n\
             vznáším námitku a žádám o výmaz."
        }
        RequestKind::Delist => {
            "při vyhledávání na mé jméno zobrazujete níže uvedené odkazy. Jako subjekt\n\
             údajů podle čl. 4 odst. 1 nařízení (EU) 2016/679 (GDPR) žádám o jejich\n\
             odstranění z výsledků vyhledávání."
        }
        RequestKind::Erase => {
            "zpracováváte mé osobní údaje uvedené níže. Jako subjekt údajů podle\n\
             čl. 4 odst. 1 nařízení (EU) 2016/679 (GDPR) žádám o jejich výmaz."
        }
    }
}

/// Only identifiers the recipient demonstrably already holds, taken from what they
/// published rather than from the user's identity file. A letter that volunteers an
/// e-mail address or a phone number the controller never had would hand them a new
/// identifier in the act of asking them to delete one.
fn identification(d: &Disclosure<'_>) -> String {
    let mut out = String::new();
    let _ = writeln!(
        out,
        "Identifikace subjektu údajů (pouze údaje, které již zpracováváte):"
    );

    for (field, label) in [
        (Field::Name, "jméno a příjmení"),
        (Field::BirthDate, "datum narození"),
        (Field::Address, "adresa"),
    ] {
        if let Some(e) = d
            .linkage
            .evidence
            .iter()
            .find(|e| e.field == field && e.level != Level::Conflict)
        {
            let _ = writeln!(out, "  {label}: {}", e.published);
        }
    }
    if let Some(locator) = &d.record.locator {
        let _ = writeln!(out, "  dotčený záznam: {locator}");
    }
    out.push('\n');
    out
}

fn categories(d: &Disclosure<'_>) -> String {
    let mut out = String::new();
    let _ = writeln!(out, "Rozsah údajů, které o mně zpřístupňujete:");
    for attr in &d.record.attrs {
        let _ = writeln!(out, "  - {}", category_cs(attr));
    }
    out.push('\n');
    out
}

fn sections(kind: RequestKind, d: &Disclosure<'_>) -> String {
    match kind {
        RequestKind::Access => ACCESS.to_string(),
        RequestKind::ObjectAndErase => {
            let register = d
                .copied_register()
                .unwrap_or_else(|| "[doplňte rejstřík, z něhož údaje přebíráte]".into());
            OBJECT_AND_ERASE.replace("{register}", &register)
        }
        RequestKind::Delist => DELIST.to_string(),
        RequestKind::Erase => ERASE.to_string(),
    }
}

/// No erasure demand: Art. 17(3)(b) defeats it, and asking anyway invites a refusal
/// that burns the statutory month.
const ACCESS: &str = "\
1. Potvrzení a informace podle čl. 15 odst. 1 GDPR

Žádám o potvrzení, zda zpracováváte mé osobní údaje, a o tyto informace:

  a) účely zpracování,
  b) kategorie dotčených osobních údajů,
  c) příjemce nebo kategorie příjemců, kterým byly údaje zpřístupněny, včetně
     příjemců ve třetích zemích,
  d) plánovanou dobu uchovávání údajů, nebo kritéria pro její určení,
  e) zdroj, z něhož údaje pocházejí,
  f) existenci automatizovaného rozhodování včetně profilování.

2. Kopie údajů podle čl. 15 odst. 3 GDPR

Žádám o kopii osobních údajů, které o mně zpracováváte.

3. K výmazu

Jsem si vědom(a), že zveřejnění údajů ve veřejném rejstříku plní právní povinnost
podle čl. 6 odst. 1 písm. c) GDPR a že výmaz je v tomto rozsahu vyloučen podle
čl. 17 odst. 3 písm. b) GDPR. Tato žádost proto výmaz nepožaduje.

Zvláštní význam pro mě má informace o příjemcích podle bodu 1 písm. c). Subjekty,
které obsah rejstříku přebírají a dále zveřejňují pro vlastní účely, se na právní
povinnost zveřejnění odvolávat nemohou.

";

const OBJECT_AND_ERASE: &str = "\
1. Námitka proti zpracování (čl. 21 odst. 1 GDPR)

Zpracování provádíte pro své vlastní účely na základě oprávněného zájmu podle
čl. 6 odst. 1 písm. f) GDPR. Tímto proti němu vznáším námitku. Podle čl. 21
odst. 1 GDPR nesmíte mé osobní údaje dále zpracovávat, pokud neprokážete závažné
oprávněné důvody pro zpracování, které převažují nad mými zájmy, právy a
svobodami.

Nejde o plnění právní povinnosti. Povinnost zveřejnění má podle zákona {register},
nikoli provozovatel, který jeho obsah přebírá a dále šíří pro vlastní účely.

2. Žádost o výmaz (čl. 17 odst. 1 písm. c) GDPR)

Neprokážete-li závažné oprávněné důvody podle bodu 1, zaniká právní základ
zpracování. V takovém případě žádám o výmaz všech mých osobních údajů bez
zbytečného odkladu.

3. Informování příjemců (čl. 19 GDPR)

Žádám o informaci, kterým příjemcům jste mé údaje zpřístupnili, a o jejich
informování o výmazu.

4. Informace podle čl. 15 odst. 1 GDPR

Žádám rovněž o informace o účelech zpracování, kategoriích údajů, příjemcích,
době uchovávání a o zdroji, z něhož údaje pocházejí.

";

const DELIST: &str = "\
1. Žádost o odstranění odkazů (čl. 17 GDPR)

Žádám o odstranění odkazu uvedeného výše z výsledků vyhledávání na mé jméno.
Žádost se opírá o čl. 17 GDPR ve světle rozsudků Soudního dvora EU ve věci
C-131/12 (Google Spain a Google, tzv. Costeja) a C-507/17 (Google v. CNIL).

Odstranění z výsledků vyhledávání nevyžaduje žádný zásah do zdrojové stránky.

2. Odůvodnění

Jsem fyzická osoba, která nevykonává veřejnou funkci, a zobrazení těchto údajů u
vyhledávání na mé jméno neodpovídá účelu, pro který byly původně zveřejněny.

[doplňte, proč je zobrazení zatěžující: adresa bydliště dostupná komukoli,
obtěžování, zastaralý nebo nepřesný údaj]

3. Informace podle čl. 15 odst. 1 GDPR

Žádám o informaci o účelech zpracování, kategoriích údajů, příjemcích a o zdroji,
z něhož údaje pocházejí.

";

const ERASE: &str = "\
1. Žádost o výmaz (čl. 17 odst. 1 GDPR)

Žádám o výmaz všech mých osobních údajů. Údaje již nejsou potřebné pro účel, pro
který byly shromážděny (čl. 17 odst. 1 písm. a) GDPR). Nejsou-li zpracovávány na
žádném z právních základů podle čl. 6 odst. 1 GDPR, jde o zpracování protiprávní
(čl. 17 odst. 1 písm. d) GDPR).

2. Informace podle čl. 15 odst. 1 GDPR

Žádám o informace o účelech zpracování, kategoriích údajů, příjemcích, době
uchovávání a o zdroji, z něhož údaje pocházejí. U údajů pocházejících z narušení
bezpečnosti žádám výslovně o informaci o příjemcích podle čl. 15 odst. 1 písm. c)
GDPR.

3. Informování příjemců (čl. 19 GDPR)

Žádám o informaci, kterým příjemcům jste mé údaje zpřístupnili, a o jejich
informování o výmazu.

";
