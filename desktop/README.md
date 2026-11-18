# Figura Praesidii: desktop

The tool itself. A Rust workspace:

```
crates/fp-core        domain types: identity, records, sources, legal routes
crates/fp-correlate   does this record describe the user, and on what evidence
crates/fp-lookup      locality-scoped queries, fetching, extraction, the pipeline
crates/fp-cli         the `fp` binary
example/              a runnable offline demo
```

```sh
cargo test
cargo run -p fp-cli -- scan --identity example/me.json \
  --catalog example/catalog.json --offline --cache example/cache
```

Scope and the legal reasoning are in [../docs/desktop.md](../docs/desktop.md); how the
code works is in [../docs/architecture.md](../docs/architecture.md). Agent instructions
are in [../docs/agents.md](../docs/agents.md).
