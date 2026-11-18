# Figura Praesidii — desktop

The tool itself. Everything the user actually runs lives here.

## Pipeline

```
adapters ──► findings store ──► request generator ──► ICS deadlines
(sources)    (encrypted,        (per-target legal
              local)             grounds)
```

## Slice 1: public register exposure

Czech registers publish more than people expect. `or.justice.cz` carries company
officers' full name, date of birth and **home address**; RŽP exposes sole traders'
places of business, usually also home; `isir.justice.cz` covers insolvency; the land
registry covers property.

**Three target classes, three different legal routes.** Getting this right per target
is the product.

| Target | Route | Outcome |
|---|---|---|
| The register itself | None available. Publication is a statutory obligation, so Art. 17(3)(b) defeats erasure. | Immovable. Say so plainly rather than generating a letter that will fail. |
| Commercial mirror sites | Art. 21 objection to processing based on legitimate interest, then Art. 17(1)(c) erasure. Their balancing claim is far weaker than the register's. | Removable. This is where the value is. |
| Search engines | Art. 17 delisting, on *Costeja* and *GC v CNIL*. | Delistable. Often what the user actually wants, even when the source stays up. |

Enumerate the footprint, identify which mirrors carry it, generate the correctly
grounded request for each. Hand-verifiable against the source, which is the whole
appeal: no corpus, no API keys, no trust required in our data.

**Be a polite client.** Querying a public register about yourself is fine; hammering
it is not. Rate-limit, identify the client honestly, cache aggressively.

## Slice 2: breach and broker exposure

HIBP for breach hits by email and phone, with the user's own API key. Then the part
with the real value: knowing *which* holders to ask, generating the requests, and
tracking the one-month deadline (extensible to three) with escalation to ÚOOÚ when a
controller ignores it.

## Storage

Encrypted at rest under a user passphrase. Not synced by default. API keys in the OS
keyring, never in a config file.

This matters more than usual: **the findings store is a ready-made dossier on the
user**, which makes it the highest-value file on their machine. Saving a report is
explicit, never automatic, and the export path should run through Figura Munda before
anything is shared.

## Out of scope

Any adapter whose input is *someone else's* identifier. Holehe-style enumeration
reports which sites an arbitrary address is registered on, which quietly turns this
into a lookup tool for third parties. Either bind such adapters behind proof of
control of the address, or leave them out. Do not ship them unguarded and describe
the result as self-OSINT.

---

## What is built (2026-11-18)

A Rust workspace. `cargo test` covers all of it; nothing here has ever reached a live
register, because the machine it was written on has no route to one.

```
crates/fp-core        domain types: identity, records, sources, legal routes
crates/fp-correlate   does this record describe the user, and on what evidence
crates/fp-lookup      locality-scoped queries, fetching, extraction, the pipeline
crates/fp-cli         the `fp` binary
example/              a runnable offline demo
```

### Run the demo

```sh
cargo run -p fp-cli -- scan \
  --identity example/me.json \
  --catalog example/catalog.json \
  --offline --cache example/cache
```

Fictional subject, two stand-in sources served from a cache. The run finds her register
entry and the mirror copying it, names the legal route for each, and drops two records
that belong to other people.

### Correlation

Fellegi-Sunter match weights in bits: a field agreement is worth `log2(m/u)`, with `u`
estimated from Czech population figures and deliberately pessimistic. The weights sit in
one table in `fp-correlate/src/weights.rs` and every verdict carries the per-field
evidence that produced it, printed beside the raw published value.

Two rules matter more than the arithmetic:

**A namesake can never be promoted.** `Match` needs 20 bits total *and* 12 bits from
fields that can tell two people apart: date of birth, a street address with a house
number, an exact e-mail, phone or IČO. A name and a town can agree perfectly and the
verdict stays `Possible`, because asserting otherwise would put a stranger's data in the
user's report and a stranger's record in a letter signed by the user.

**A contradiction excludes.** A different year or month of birth, or a different given
name in a source that carries legal names, rules the record out whatever else agrees.
Excluded records are counted and discarded, never stored. Addresses are exempt: people
move, so a different address is worth zero bits rather than a veto.

Czech specifics that carry real weight: diacritic folding, `Nováková` and `Novák` sharing
a surname root, either name order (`DVOŘÁKOVÁ Jana`), titles stripped out of the name
field, `14. března 1981` parsed, partial dates as a first-class case, IČO check digits,
and gmail dot-and-plus aliasing. `rodné číslo` is deliberately not collected: no register
publishes it, so it would add nothing to a comparison while making the findings store far
more damaging to lose.

### Lookup

Sources are data, not code: `crates/fp-lookup/sources.json` holds a URL template and the
field labels per source, with a `verified` flag. **Nothing ships verified.** An unchecked
source is named in every report, because a silently wrong selector produces an empty
result that looks exactly like a clean one. `fp verify-sources` checks reachability and
label presence from a machine that can reach the hosts; a human still has to read a page
before marking anything verified.

Extraction flattens HTML to labelled lines rather than parsing it, because the Czech
registers' label text outlives their markup. Page-level fields (the company a full
extract is about) are read separately from per-person fields, so one officer's company
cannot leak onto the next officer's record.

Politeness is built in: GET only, one locality at a time, a per-host minimum interval, a
per-run request budget, an on-disk cache, and a user agent that says plainly it is one
person looking themselves up.

### Not built yet

Request generation, the findings store and its encryption, the ICS deadline export, HIBP,
and the mirror sweep. The report ends by naming them rather than implying the free tier
covers them.

Known gaps, in the order they bite:

- Every URL template is a guess. ISIR needs a form POST the fetcher cannot drive, and the
  land registry gates person search behind a captcha.
- No mirror is shipped in the catalog, so the route where erasure actually bites has no
  source to run against. Mirrors have to be added by hand, per name.
- The report has no encryption and `--out` writes plain text. It is off by default for
  that reason.
- Licensing is unsettled, so the manifests carry no licence field. Contribution terms
  need deciding before the first commit, per the open-core split.
