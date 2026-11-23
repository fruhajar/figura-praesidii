# Instructions for coding agents

Read [scope.md](scope.md) before anything else. It explains what this project refuses to
build and why, and most mistakes available here are reintroductions of something it
already rejected. [architecture.md](architecture.md) describes the code.

## Invariants

These are not preferences. Each one has a reason that outlives the person who wrote it,
and most are pinned by a test; if you find yourself removing or weakening one, stop and
ask instead.

**1. Self-lookup only.** No adapter whose input is somebody else's identifier. No search
by description. No "people this person knows". Enumeration adapters of the Holehe kind
report which sites *any* address is registered on, so they stay out unless bound behind
proof of control of the address.

**2. A namesake is never promoted.** `Verdict::Match` requires `MATCH_BITS` in total and
`DISCRIMINATING_GATE` bits from fields that separate two people. A name and a town can
never reach it, however high the total climbs. Pinned by
`name_and_town_alone_can_never_be_a_match` and
`month_and_year_need_an_address_to_clear_the_gate` in `fp-correlate/tests/link.rs`.
Raising a weight in `weights.rs` can breach this gate indirectly, so run those tests
after touching the table.

**3. A contradiction excludes, and the excluded are never kept.** `scan` counts an
excluded record and drops it. Do not add it to a findings list, a cache of parsed
records, or a debug dump: holding a stranger's record is the exact harm the tool exists
to undo. Address disagreement is deliberately *not* a veto, because people move.

**4. `rodné číslo` is never collected.** No Czech register publishes it, so it cannot
improve a match, and holding it would make the findings store far more damaging to lose.
`date.rs` has a detector so it can be recognised and refused, not stored.

**5. The published value is kept verbatim.** Comparisons run on folded text; the raw
string a source printed is carried alongside, because a request has to quote the data as
it appears. Never overwrite a raw value with its normalised form.

**6. Catalog entries ship `verified: false`.** Nothing in `sources.json` has been run
against a live site. Do not flip the flag because the code looks right, and do not hide
an unverified source from the report: a wrong selector produces an empty result that
looks exactly like a clean one. Only a human who has read a live page may mark a source
verified.

**7. Stay a polite client.** GET only, one locality per query, a per-host minimum
interval, a per-run request budget, an on-disk cache, and a user agent that says plainly
this is one person looking themselves up. Querying a register about yourself is
unremarkable; hammering it from a tool that ships to strangers is how the endpoint gets
closed for everyone.

**8. Nothing is written to disk unless asked.** The report prints to the terminal;
`--out` is the user's decision. There is no telemetry and no crash reporting.

**9. No real personal data in tests, fixtures or examples.** Fictional subjects only, and
`example.com`-style hosts for stand-in sources. Do not name a real company as a mirror in
the shipped catalog: a mirror entry asserts a legal claim about that company, so it
belongs to the user who adds it locally. A valid-looking IČO or phone number in a fixture
should be synthetic, not copied from a real entity.

**10. A letter discloses nothing the recipient did not already hold.** The
identification block in `fp-request` is assembled from the linkage evidence, which means
from values the source published, never from the identity file. Adding a field there
would hand a controller a new identifier in the act of asking them to erase one. Pinned
by `identification_discloses_nothing_the_source_did_not_publish` in
`fp-request/tests/letters.rs`. Warnings also stay out of the letter body, so a file
written with `--out-dir` is safe to send as-is.

## Verifying a change

From `desktop/`:

```sh
cargo test                  # 47 tests at the time of writing
cargo clippy --all-targets  # expected to be silent
cargo fmt --all
```

Then run the offline demo and read the output, because most regressions here are
plausible-looking text rather than a panic:

```sh
cargo run -p fp-cli -- scan --identity example/me.json \
  --catalog example/catalog.json --offline --cache example/cache
```

The demo must find two records (the register entry and the mirror), and rule out two
(one by name, one by date of birth).

### The fixture trap

`example/cache/` filenames are `<url-slug>-<fnv1a64 hex>.html`, derived from the request
URL by `Cache::path_for`. Change a URL template, a locality, or an identity name in the
demo and the fixtures silently stop matching: the scan then reports "offline and nothing
cached" rather than failing loudly. Regenerate them by mirroring `path_for` exactly, and
note that the FNV-1a prime is `0x100_0000_01b3`, which is 2^40 + 435. Writing it with one
zero too many produces a plausible wrong hash; reference vectors are pinned in
`fetch.rs`.

### No network in a sandbox

A sandboxed environment generally cannot reach `or.justice.cz`, `rzp.cz`,
`isir.justice.cz` or `haveibeenpwned.com`. That is why the catalog is data with a
`verified` flag and why `OfflineFetcher` exists. Never paper over a blocked host by
inventing what a page returns: write the fixture, mark the source unverified, and say so.

## Conventions

- Comments are sparse and on point. Do not restate the signature. Do comment units,
  statutory references, population estimates behind a weight, and anything knowingly
  approximate.
- Keep the match weights in one table with the `u` estimate behind each value written
  down. A weight with no stated reasoning is a bug waiting to be copied.
- Legal claims in code or docs name the article or the case. "Art. 17(3)(b)" and
  "C-131/12 Costeja", not "the GDPR says".
- Dependencies stay few and boring. The HTML path is tag stripping on purpose, because
  register label text outlives register markup.
- `main` is protected. Work on `dev` and open a pull request.
- The workspace is **MPL-2.0**, matching Munda. New crates carry
  `license.workspace = true`; no per-file licence headers, since the LICENSE file and
  the manifest field are the whole declaration. Any paid layer links the engine from a
  separate repository, so nothing here needs relicensing.
