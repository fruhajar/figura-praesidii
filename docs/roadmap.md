# Roadmap

Possible expansions, drafted against what comparable tools actually do. Nothing here is
committed work; it is the shortlist and the reasoning, so the next decision starts from
evidence rather than from scratch.

The research behind it was done in November 2026. Sources are listed at the end, so
every claim can be re-checked when it goes stale.

## The finding that reshapes slice 1

**The Czech registers publish the data this tool scrapes as official open data.**

`dataor.justice.cz` serves the public registers as per-court, per-year XML and CSV
dumps (`ks-actual-praha-2021.xml` and siblings), and their `STATUTARNI_ORGAN` /
`STATUTARNI_ORGAN_CLEN` elements carry given name, surname, date of birth as
`YYYY-MM-DD`, and address. `ares.gov.cz` exposes a documented REST service over the
business register, RŽP, RES and ROS, with `GET /ekonomicke-subjekty/{ico}` and a
`POST /ekonomicke-subjekty/vyhledat` complex filter.

That makes the HTML scraper the weakest part of the pipeline for registers. Its guessed
labels, its permanently false `verified` flag, and the politeness budget all exist to
work around a problem the state already solved. The scraper still earns its place
against commercial mirrors, which publish no open data and are where erasure actually
bites.

## What comparable tools do

| Tool | Approach | Report | Data handling |
|---|---|---|---|
| **Datenanfragen.de / datarequests.org** (open source, non-profit) | Request generator plus a CC0 database of company contacts, supervisory authorities and templates. The user sends the request themselves. | Generated PDF or e-mail per request; per-request "proceeding" holding every message, with an overdue reminder widget. | Stays in the user's browser. The project never sees it. |
| **YourDigitalRights.org** (open source) | Generates the erasure request and opens it in the user's own mail client. | The letter itself, nothing more. | Nothing stored server-side. |
| **Consumer Reports Permission Slip** | Authorized agent: files requests on the user's behalf and chases them. Transferred to DeleteMe in 2024. | Status per company in an app. | Holds the user's identity and request state as an intermediary. |
| **DeleteMe** | Managed removal by staff analysts. | Periodic report written by a reviewer, listing found and removed. | Full custody of the user's profile. |
| **Incogni** | Automated opt-outs across 400+ brokers, re-checked every 60 to 90 days. | Dashboard status labels only. | Full custody. |
| **Optery** | Scans people-search sites, submits opt-outs, re-checks. Free tier is the exposure report alone. | **Before-and-after screenshots of each exposed profile.** The differentiator is proof. | Full custody, screenshots held in their dashboard. |
| **SpiderFoot** (open source, investigator side) | ~200 modules, YAML correlation rules deriving higher-level findings from raw events. | CSV, JSON, GEXF graph; STIX and SARIF via the API. | One SQLite database per install: scan metadata, config, events, correlation output, logs. A scan is the unit. |
| **OCCRP Aleph / FollowTheMoney** | Entity schemas (150+) over documents and records. | Graph and document views. | Facts as statement tuples with provenance, and every property multi-valued, because real data gives one person several names and sometimes several birth dates. |
| **Splink** | Fellegi-Sunter record linkage, the same maths as `fp-correlate`. | n/a, a library. | Estimates m and u by EM instead of hand-setting them. Documented pitfall: blocking hands EM a non-random sample, so the estimates describe blocked pairs, not the population. |
| **Have I Been Pwned** | Breach index, queried by address. | n/a, an API. | API v3 is paid, roughly $4.39 to $5.29 a month at the entry tier, rate limited per key. Pwned Passwords is free, unlimited, k-anonymous. |
| **Mozilla Monitor** | Breach alerts plus broker removal through a partner. | Dashboard. | The cautionary row: its removal partner Onerep was reported to have founder ties to people-search sites. |

**What to take.** Optery's insistence on proof. SpiderFoot's run-as-a-unit, exportable.
Aleph's statement shape, because provenance is this project's product. datarequests.org's
whole posture: templates plus a contact database plus local-only state is figprae's own
architecture, already validated by a non-profit running it at scale.

**What to avoid.** Every service above that works by taking custody of the user's
profile. The authorized-agent model needs identity verification and staff, makes the
operator a processor acting for the subject, and ends local-first in one step. Permission
Slip being handed to DeleteMe is worth noticing: running this as a free public service is
not a stable position if it depends on labour.

## Ranked expansions

### E1. Registers via official open data

**Partly done.** The dump half is built: `extract.kind: "xml"` streams a downloaded
dump, blocks on the surname before keeping anything, renders the locator from the
subject's IČO, and reports its caps. `dataor.justice.cz` is in the catalog and the
or.justice.cz scrape is disabled behind it. Still to do: the ARES REST service, which is
what covers sole traders where no dump exists, and verifying the element names against a
real file.

Replace the or.justice.cz scrape with the `dataor.justice.cz` dumps, and the
session-based RŽP form with the ARES REST service. The registration court in the dump
filename does locality scoping for free, which is exactly the free tier's selling point.

Lands as a second extract kind in `fp-lookup` (`catalog.rs`) or a sibling `fp-registry`
crate, keeping `Fetcher` and `Cache` as they are, adding a streaming XML reader and a
disk budget. Never load a whole dump into memory.

The real prize is verifiability: a committed sample dump becomes a genuine fixture, and
`verified` starts meaning "schema checked against a real file" instead of "nobody has
looked". Effort: medium to large. No invariant is at risk; the politeness rules get
easier rather than harder.

### E2. The mirror registry, published as CC0 open data

Already proposed in [online.md](online.md). datenanfragen/data proves the shape works,
and its CC0 licence means its controller contacts can be imported rather than retyped.

Lands as a `mirrors/` directory of JSON entries (host, fields carried, controller from
the site's imprint, `copies_from`, opt-out URL where one exists) plus an importer into
`fp_core::Controller`.

Candidates to check on a networked machine, none to be shipped unverified:
`rejstrik-firem.kurzy.cz`, `ares.cz` (a private operator, **not** `ares.gov.cz`, a trap
worth documenting in the catalog), `rejstrik-obchodni.eu`,
`justice-obchodni-rejstrik.cz`, `vyhledavace.cz`, `merk.cz`, `detail.cz`, `firmo.cz`,
`cribis.cz`, `edb.cz`, `zivefirmy.cz`.

Effort: small in code, ongoing in curation. Risk: invariant 9. A mirror entry asserts a
legal claim about a named company, so each one needs a human to have read the page.

### E3. Evidence preservation

Optery sells screenshots; figprae currently keeps nothing. A controller can take the
page down and deny it ever said that, and the ÚOOÚ complaint expects copies of the
correspondence.

Store the fetched bytes, a SHA-256, the URL and the fetch timestamp beside each finding.
The letter can then cite the state as of a date, and a complaint can attach the
preserved copy.

One hard rule to write down with it: **no third-party archiving by default.** Submitting
the URL to the Wayback Machine or archive.today tells a third party which page about the
user is being examined, which is the online-tier invariant wearing a different coat.
Local snapshot only, with third-party archiving an explicit opt-in.

Lands on `fp-lookup`'s fetch path plus E4's store. Effort: small once E4 exists.

### E4. The encrypted findings store

The largest hole, and the comparators disagree usefully. SpiderFoot's per-scan SQLite
gives the run-as-diffable-unit. Aleph's statement tuples give provenance per value and
multi-valued properties. Take both: one local store, encrypted at rest under a
passphrase, holding runs, records, evidence blobs, linkages, requests and deadlines, each
value carrying its source and the date it was observed.

Invariant 3 constrains the schema: excluded records stay a counter and must never become
a quarantine table. Add retention, so findings expire by default, and `fp forget`.

Lands as a new `fp-store` crate. Effort: large. It is the spine for E3, E5 and E8.

### E5. Deadline tracking and the ÚOOÚ complaint

datarequests.org's per-request proceeding and overdue widget, done locally: `fp track`
over the store, the ICS export [online.md](online.md) already promises, and a complaint
generator for čl. 77 GDPR plus § 31 zákona č. 110/2019 Sb.

The ÚOOÚ form asks for the subject's identification, the controller's identification, a
description of the processing and the breach, copies of the correspondence, and the
remedy sought. All of it is already in the store by then, which makes the complaint the
cheapest letter in the set.

Lands as a second letter kind in `fp-request`, reusing `cs.rs`. Effort: small to medium,
and it closes the loop the README currently stops short of.

### E6. Calibrate the weights instead of hand-setting them

`fp-correlate/src/weights.rs` carries deliberately pessimistic constants and asks the
user to self-report `name_rarity`. The dumps from E1 make the real distributions
computable: surname and given-name frequency, birth-date spread, city population. Ship a
small frequency table and turn `name_rarity` into a lookup.

Keep Splink's EM out of scope. One-to-many self-lookup has no labelled pairs, and
Splink's own warning about blocking biasing the estimates applies directly.

Non-negotiable: the discriminating gate survives any recalibration (invariant 2), so
`fp-correlate/tests/link.rs` is the acceptance criterion, not a suggestion. Effort:
medium, and the best accuracy-per-line available, but it depends on E1.

### E7. Slice 2, breach and broker exposure, priced honestly

HIBP with the user's own key, with the monthly cost stated in the docs rather than
implied to be free, plus the free k-anonymous Pwned Passwords check as a separate
no-key feature.

Enumeration adapters of the Holehe kind stay out unless bound to proof of control of the
address (invariant 1). Effort: small per source, and mostly an exercise in not
overclaiming.

### E8. Report formats beyond the terminal

SpiderFoot's five export formats are over-built for one person looking themselves up,
but `--format json` for scripting and a single self-contained HTML file are both cheap.
No external assets, nothing fetched when the file is opened.

Explicitly not a hosted dashboard, and not synced, per the dossier warning in the
README. Effort: small.

## Non-goals, reaffirmed

The commercial landscape does not change the answers already in [scope.md](scope.md).

- **Authorized-agent or managed removal.** Filing on someone's behalf needs identity
  verification and staff, makes the operator a processor for the subject, and ends
  local-first.
- **Server-side lookup or correlation of any kind**, per the invariant in
  [online.md](online.md).
- **Buying or reselling a broker database**, which would make this project the holder of
  the dataset it exists to shrink.

## Open questions, to check from a networked machine

1. **Does the business register still publish officers' home addresses in its public
   part, and when is that suppressed?** The README and [scope.md](scope.md) assert it
   flatly, and it is the load-bearing claim of slice 1. One source found during this
   research says permanent-residence addresses may be withheld. Check against zákon
   č. 304/2013 Sb. and a real extract before the claim stands unqualified.
2. Can ARES `POST /ekonomicke-subjekty/vyhledat` filter by a natural person's name and
   city, or is person search only possible through the dumps?
3. Do the `dataor.justice.cz` dumps carry the same address fields as the web extract, and
   how large is one court-year?
4. Do datenanfragen's CC0 records include Czech controllers worth importing?

## Sources

- Datenanfragen.de: [website repo](https://github.com/datenanfragen/website),
  [data repo](https://github.com/datenanfragen/data),
  [datarequests.org](https://www.datarequests.org/),
  [v2 and the complaint generator](https://www.datarequests.org/blog/v2-release/)
- [YourDigitalRights.org](https://yourdigitalrights.org/) and its
  [repo](https://github.com/your-digital-rights/yourdigitalrights.org)
- Consumer Reports: [how Permission Slip
  works](https://innovation.consumerreports.org/how-does-permission-slip-work/),
  [transfer to
  DeleteMe](https://innovation.consumerreports.org/permission-slips-next-chapter-with-deleteme/)
- Removal services compared:
  [Optery vs Incogni](https://cyberinsider.com/data-removal/optery-vs-incogni/),
  [Optery review 2026](https://techsifted.com/reviews/optery-review-2026/),
  [Optery vs DeleteMe vs
  Incogni](https://comparesubscriptions.com/security/optery-vs-deleteme-vs-incogni-2026)
- [SpiderFoot](https://github.com/smicallef/spiderfoot) and its
  [architecture overview](https://deepwiki.com/smicallef/spiderfoot/1-overview)
- FollowTheMoney:
  [explanation](https://docs.aleph.occrp.org/developers/explanation/followthemoney/),
  [Person schema](https://followthemoney.tech/explorer/schemata/Person/)
- Record linkage: [Splink feasibility
  study](https://fcds.med.miami.edu/downloads/RecordLinkageMonograph/2023/splink_monograph_2023_supplement.pdf),
  [Fellegi-Sunter
  limitations](https://www.zingg.ai/post/fellegi-sunter-model-limitations-modern-entity-resolution)
- [HIBP API v3](https://haveibeenpwned.com/api/v3) and a
  [pricing summary](https://agentdeals.dev/vendor/have-i-been-pwned)
- ARES: [technical
  documentation](https://mf.gov.cz/assets/attachments/2024-02-16_ARES-Technical-documentation-Catalog-of-public-services_v03.pdf),
  [API changelog](https://ares.gov.cz/stranky/changelog-api),
  [EkonomickeSubjekty
  endpoints](https://github.com/JanBukva/ares-gov-cz-swagger/blob/master/docs/Api/EkonomickeSubjektyApi.md),
  [MF open data](https://data.mf.gov.cz/topics/ares)
- Justice open data: [ISVR open-data
  manual](https://dataor.justice.cz/files/ISVR_OpenData_Uzivatelska_prirucka.pdf),
  [full-scope
  announcement](https://msp.gov.cz/en/web/msp/-/otevrena-data-verejnych-rejstriku-jiz-brzy-v-plnem-rozsahu)
- ÚOOÚ: [online complaint
  form](https://uoou.gov.cz/online-formular-pro-podani-stiznosti-na-poruseni-predpisu-na-ochranu-osobnich-udaju),
  [complaint form
  PDF](https://uoou.gov.cz/media/formulare/form-stiznost-zpracovani-ou1.pdf)
- Czech register landscape: [Veřejné rejstříky v
  Česku](https://dostupnyadvokat.cz/blog/verejne-rejstriky),
  [rejstrik-firem.kurzy.cz](https://rejstrik-firem.kurzy.cz/)
