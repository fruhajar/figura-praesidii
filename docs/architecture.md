# Architecture

How the built thing works. The reasoning behind *what* it is, and the boundaries it
must not cross, are in [scope.md](scope.md) and [desktop.md](desktop.md).

A Rust workspace under `desktop/`, first built 2026-11-18. `cargo test` covers all of
it; nothing here has ever reached a live register, because the machine it was written on
has no route to one. Paths below are relative to `desktop/`.

```
crates/fp-core        domain types: identity, records, sources, legal routes
crates/fp-correlate   does this record describe the user, and on what evidence
crates/fp-lookup      locality-scoped queries, open data, fetching, extraction, the pipeline
crates/fp-request     the letter per finding, in Czech
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

There are two extraction paths, chosen per source by `extract.kind`.

**Open data, for registers.** The Czech registers publish officer records as bulk
XML, so a dump is read from a file the user downloads rather than fetched: no rate limit
to respect, a stable schema, and the registration court in the filename doing the
locality scoping. The reader streams the file, so a court-year never lands in memory,
and both a record cap and a byte cap are reported rather than silently truncating.
Element names live in the catalog like everything else, because no dump could be read
from the machine that wrote them.

Two details carry weight here. A **surname block** runs before anything is kept: one
court-year holds hundreds of thousands of strangers, and blocking is a speed trick in
record linkage but the privacy rule here, since those people must never reach the user's
report. And the **locator is rendered from the subject's own fields**
(`https://ares.gov.cz/ekonomicke-subjekty?ico={ICO}`), so a request cites the register
entry rather than a file path on the user's laptop.

**Labels, for everything else.** Extraction flattens HTML to labelled lines rather than
parsing it, because the Czech registers' label text outlives their markup. Page-level fields (the company a full
extract is about) are read separately from per-person fields, so one officer's company
cannot leak onto the next officer's record.

Politeness is built in on the fetching path: GET only, one locality at a time, a
per-host minimum interval, a per-run request budget, an on-disk cache, and a user agent
that says plainly it is one person looking themselves up. The open-data path sidesteps
all of it by not making requests.

## Requests

`fp requests` drafts one letter per finding. The route comes from `fp-core::protect`, so
the pairing is decided once: Article 15 only against a register, because Article 17(3)(b)
defeats erasure and a refusal costs the statutory month; Article 21 objection then
Article 17(1)(c) against a mirror, naming the register it copies, since that is what
breaks its legitimate-interest claim; Article 17 delisting on *Costeja* and *GC v CNIL*
against a search engine; plain Article 17 where nothing competes.

Two rules shape the text. The identification block is built from what the recipient
already published, taken from the linkage evidence rather than from the identity file, so
a letter never hands a controller an e-mail address or phone number they did not have.
And warnings stay outside the letter: a file written with `--out-dir` is the letter and
nothing else, while "this record could be a namesake's" and "this source is unverified"
print to the terminal.

Drafting is a separate command rather than part of `scan` because a request asserts to a
controller that the record is the sender's, which deserves a second deliberate step.

## Not built yet

The findings store and its encryption, the ICS deadline export, HIBP, and the mirror
sweep. [roadmap.md](roadmap.md) ranks those against what comparable tools do, and
records the one finding that would reshape the register adapters: the Czech registers
publish officer records as official open data, which makes the HTML scraper unnecessary
for registers and leaves it useful only against commercial mirrors. The report ends by naming them rather than implying the free tier
covers them.

Known gaps, in the order they bite:

- Every URL template is a guess. ISIR needs a form POST the fetcher cannot drive, and the
  land registry gates person search behind a captcha.
- No mirror is shipped in the catalog, so the route where erasure actually bites has no
  source to run against. Mirrors have to be added by hand, per name.
- The report has no encryption and `--out` writes plain text. It is off by default for
  that reason.
