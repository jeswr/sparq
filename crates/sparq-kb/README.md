# sparq-kb

> 🤖 **SPARQ agent** [OPUS-4.8] — Phase-1 artifacts for dogfooding sparq as a
> **Project Knowledge Graph (PKG)**: store the project's own research findings,
> sources, techniques, and `bd` task model as RDF behind sparq's SPARQL + SHACL +
> reasoning surface. Research prototype; adoption is gated on a token-A/B verdict
> (epic `sq-2m6zm`, design record `research/dogfooding-sparq-knowledge-graph.md`,
> PR #1063). NOT published.

## 🚀 Quickstart

The crate ships three Turtle artifacts as `&str` constants (the default build is a
pure data + Rust-vocab crate — no engine code):

```text
sparq_kb::PKG_ONTOLOGY   // ontology/pkg/pkg.ttl   — the reuse-first PKG vocabulary
sparq_kb::PKG_SHAPES     // shapes/pkg.shapes.ttl  — the SHACL write-time guardrails
sparq_kb::PKG_EXAMPLE    // examples/pkg-example.ttl — a tiny instance file
sparq_kb::vocab::*       // the pkg: IRIs as constants, pinned against the Turtle
```

Dogfood the guardrails with sparq's own SHACL engine (opt-in `validate` feature):

```text
cargo test -p sparq-kb --features validate -- --nocapture
```

This loads the ontology + the example instances, runs `pkg.shapes.ttl` via
`sparq-shacl`, and asserts the valid Findings/Tasks PASS while the deliberately
invalid ones (missing source/confidence, out-of-enum status, stale dependency edge)
are REPORTED.

## ✨ Features

- **Reuse-first ontology** — generalises the vendored `zkp-sparql`
  `sig-impl:Assertion` reified-claim pattern into `pkg:Finding`; reuses PROV-O,
  SKOS, DCAT, FaBiO/FRBR/DC, CiTO, schema.org, and nanopublications. Only ~4 terms
  are genuinely net-new (`pkg:exploredStatus`, `pkg:followUpPriority`,
  `pkg:confidence`, `pkg:couldBeMergedWith`) plus the single
  `pkg:dependsOn` `owl:inverseOf` `pkg:blockedBy` pair (there is **no**
  `pkg:blocks`). Full reuse + live-ontology-alignment record in
  [`ontology/pkg/PROVENANCE.md`](ontology/pkg/PROVENANCE.md).
- **SHACL guardrails** — `pkg.shapes.ttl` makes a source + a confidence value + an
  assurance basis + non-filler content **mandatory** on every `pkg:Finding`, and
  enforces a valid status / bounded priority / no-stale-edge on every `pkg:Task`.
- **Opt-in by construction** — default build is data + constants only; the
  `validate` feature is the only thing that pulls in `sparq-core` + `sparq-shacl`.
- **No-drift guard** — `vocab.rs` is byte-pinned against `pkg.ttl` by a sync test.

## 📚 Learn more

- Design record: `research/dogfooding-sparq-knowledge-graph.md` (PR #1063) — the
  reuse-first ontology (§2), the SHACL guardrails (§2.4, §4.4), and the falsifiable
  token-A/B measurement protocol (§5).
- `ontology/pkg/PROVENANCE.md` — which external vocabulary each term reuses, and the
  verification of every `skos:closeMatch` alignment against the live ontology.
- The precedent it follows: `crates/sparq-trust/ontologies/zkp-sparql/` and
  `crates/sparq-trust/src/vocab.rs` (the ship-an-ontology + Rust-constants pattern).
- Epic `sq-2m6zm` (this crate is `sq-2m6zm.1`); blocks the ingestion PoC
  `sq-2m6zm.2`.

## License

MIT — see the repository-root [`LICENSE`](../../LICENSE). © 2026 Jesse Wright.
