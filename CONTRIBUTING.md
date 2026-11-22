# Contributing

## Licence and contribution terms

This project is **MPL-2.0**, and contributions stay MPL-2.0. There is no copyright
assignment and no CLA to sign.

The terms match Figura Munda deliberately, and they work for the same structural
reason: any paid layer sits in a separate private repository that **links** this
engine rather than absorbing its source. MPL's copyleft is file-level, so linking is
permitted, the paid layer can stay proprietary, and no contributor's code is ever
relicensed. If a feature ever needs contributed code pulled into a proprietary file,
the answer is to keep that feature open.

Two consequences worth stating plainly:

- **Nobody can close a fork of the lookup and correlation engine.** Modified MPL files
  stay MPL. For a tool whose whole claim is that it runs locally and reports honestly,
  that is the point rather than a side effect.
- **Which expansion is paid is still open.** That is a product question, not a
  licensing one. Whatever lands in this repository is MPL-2.0, including the parts a
  paid tier might later want.

Sign off commits under the [Developer Certificate of
Origin](https://developercertificate.org/) with `git commit -s`. Nothing enforces it
yet, since this repository has no CI, so it is on the author to remember.
`git rebase --signoff <the commit before yours>` fixes a branch that already has
commits without it.

## Before sending a change

`main` is protected. Work on `dev` and open a pull request. From `desktop/`, run
`cargo test`, `cargo clippy --all-targets` and `cargo fmt --all`.

Read [docs/agents.md](docs/agents.md) for how the code is expected to read, and
[docs/scope.md](docs/scope.md) for what this tool refuses to become. Both carry
settled decisions, the largest being that it looks up only you.

## No real personal data

**Never put real personal data in a test, a fixture or an example.** Fictional
subjects only. A valid-looking IČO or phone number in a committed fixture is a leak
with a plausible excuse attached, and a corpus of real register extracts is exactly
the collection this project exists to help people shrink.

An unverified source template is marked unverified and says so in the report. Writing
a fixture that guesses what a page returns is fine; implying the guess was checked is
not.
