# Architecture

How the built thing works. The reasoning behind *what* it is, and the boundaries it
must not cross, are in [scope.md](scope.md) and [desktop.md](desktop.md).

A Rust workspace under `desktop/`, first built 2026-11-18. `cargo test` covers all of
it; nothing here has ever reached a live register, because the machine it was written on
has no route to one. Paths below are relative to `desktop/`.

```
crates/fp-core        domain types: identity, records, sources, legal routes
crates/fp-correlate   does this record describe the user, and on what evidence
crates/fp-lookup      locality-scoped queries, fetching, extraction, the pipeline
crates/fp-cli         the `fp` binary
example/              a runnable offline demo
```

## Run the demo

From `desktop/`:

```sh
cargo run -p fp-cli -- scan \
  --identity example/me.json \
  --catalog example/catalog.json \
  --offline --cache example/cache
```

Fictional subject, two stand-in sources served from a cache. The run finds her register
entry and the mirror copying it, names the legal route for each, and drops two records
that belong to other people.

## Correlation

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

## Lookup

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

## Not built yet

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
