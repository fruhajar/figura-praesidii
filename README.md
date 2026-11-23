# Figura Praesidii

Find out what the public record holds about **you**, then get it taken down.

You run it on your own machine, against your own name. It searches Czech public
registers within a locality, works out which of the results are actually you rather
than a namesake, shows you the evidence for each decision, and names the legal route
that works against each holder: an Article 15 access request where the entry cannot be
erased, an Article 21 objection plus Article 17 erasure against the commercial sites
that republish it, delisting against a search engine.

Nothing is uploaded. There is no account, no server, and no shared compute.

> **Status: early.** The correlation engine and the lookup pipeline work end to end and
> are covered by tests. No source in the shipped catalog has been checked against a
> live site yet, so treat an empty result as untested rather than clean. Request
> drafting, the encrypted findings store and the calendar export are not built.

*Praesidii* is the genitive: "figure of protection".

## Why self-lookup only

Looking up **other people** is permanently out of scope, and that is an architectural
boundary rather than a policy note. Searching by description instead of by a known
identifier makes a tool a reverse-identification engine; inferring "people this person
knows" makes it a stalking aid. Pointing the same plumbing at yourself fixes both at
once: the data subject is the consenting user, and the hard parts of the GDPR become
the feature set. The full reasoning, including why a device-ID lookup has no lawful
corpus to query, is in [docs/scope.md](docs/scope.md).

## Install

Needs Rust 1.85 or newer.

```sh
cargo build --release --manifest-path desktop/Cargo.toml
```

The binary lands at `desktop/target/release/fp`.

## Usage

Describe yourself in a JSON file. Only what you are willing to match on, and nothing
you would mind losing:

```json
{
  "names": ["Jana Dvořáková", "Jana Marková"],
  "birth_date": "14.3.1981",
  "localities": ["Nuselská 123/45, Praha 4, 140 00"],
  "emails": ["jana@example.cz"],
  "name_rarity": "common"
}
```

Then run a scan:

```sh
fp scan --identity me.json --locality Praha --cache ~/.cache/figprae
```

Then draft the letters for what it found:

```sh
fp requests --identity me.json --cache ~/.cache/figprae --out-dir letters/
```

One Czech letter per finding, grounded on the route that target admits: an Article 15
access request where the entry is immovable, an Article 21 objection plus Article 17
erasure against a mirror, delisting against a search engine. Drafting is a separate
command on purpose, since sending one asserts to a controller that the record is yours.
Records that are only *possibly* yours are held back unless you pass
`--include-possible`.

Other commands:

```sh
fp sources           # every source, and whether it has been checked against the live site
fp verify-sources --identity me.json   # check the catalog's URL templates and labels
fp help
```

To see the output shape without touching the network, run the bundled demo. The subject
is fictional and the two sources are stand-ins served from a cache:

```sh
cargo run --manifest-path desktop/Cargo.toml -p fp-cli -- scan \
  --identity desktop/example/me.json \
  --catalog desktop/example/catalog.json \
  --offline --cache desktop/example/cache
```

It finds the register entry and the mirror copying it, names the route for each, and
discards two records belonging to other people.

### The report is a dossier on you

A finished report is the highest-value file on your disk: everything a stranger would
have to assemble by hand, assembled. It prints to the terminal and is written to disk
only when you pass `--out`. Keep it out of synced folders.

## How it decides a record is yours

Field agreements are scored in bits, Fellegi-Sunter style, and every verdict carries
the per-field evidence that produced it next to the value the source published, so you
can check the reasoning against the page. Two rules matter more than the arithmetic:

- **A namesake is never promoted.** Calling a record yours takes 20 bits in total *and*
  12 bits from fields that can tell two people apart: a date of birth, a street address
  with a house number, an exact e-mail, phone or IČO. A name and a town can agree
  perfectly and the verdict stays "possibly yours".
- **A contradiction excludes.** A different birth year or month, or a different given
  name in a source that carries legal names, rules a record out whatever else agrees.
  Excluded records are counted and discarded, never stored. Addresses are exempt,
  because people move.

Czech specifics carry real weight here: diacritic folding, `Nováková` and `Novák`
sharing a surname root, either name order, titles stripped out of the name field,
`14. března 1981` parsed, partial dates as a first-class case, IČO check digits.
`rodné číslo` is deliberately never collected.

## Limitations

- Every URL template in the catalog is unverified. Run `fp verify-sources` from a
  machine that can reach the hosts, and read a page by hand before trusting it.
- No commercial mirror ships in the catalog, so the route where erasure actually bites
  has nothing to run against until you add the sites that rank on your own name.
- The insolvency register needs a form POST the fetcher cannot drive; the land registry
  gates person search behind a captcha.
- Letters are Czech only, and they are templates rather than legal advice: each one has
  bracketed placeholders to fill in and should be read before it is sent.
- Breach and broker exposure, deadline tracking and the mobile emission measurement are
  described in the docs and not implemented.

## Documentation

| Document | What is in it |
|---|---|
| [docs/scope.md](docs/scope.md) | What this is and is not, and why. Read first. |
| [docs/architecture.md](docs/architecture.md) | How the code works, and what is not built. |
| [docs/desktop.md](docs/desktop.md) | The desktop tool: pipeline, slices, storage. |
| [docs/online.md](docs/online.md) | The hosted surface, and the invariant it must keep. |
| [docs/mobile.md](docs/mobile.md) | Device emission measurement on Android. |
| [docs/agents.md](docs/agents.md) | Instructions for coding agents working in this repo. |

## Contributing

`main` is protected; work on `dev` and open a pull request. Before pushing, run
`cargo test`, `cargo clippy --all-targets` and `cargo fmt --all` from `desktop/`.

Sign off commits under the Developer Certificate of Origin with `git commit -s`.
There is no CLA. See [CONTRIBUTING.md](CONTRIBUTING.md). Never put real personal
data in a test, a fixture or an example.

## Licence

**MPL-2.0**, the same terms as Figura Munda and for the same reason. Any paid
expansion lives in a separate private repository that *links* this engine rather than
absorbing its source, which file-level copyleft permits, so no contribution is ever
relicensed and no CLA is needed. Nobody can close a fork of the lookup and correlation
engine, which for a tool about your own exposure is the point rather than a side effect.

Which expansion is paid is still open. That is a product question, and it no longer
blocks anything: whatever lands in this repository is MPL-2.0. See
[CONTRIBUTING.md](CONTRIBUTING.md).
