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
sparq_kb::PKG_ONTOLOGY   // ontology/pkg/pkg.ttl     — the reuse-first PKG vocabulary
sparq_kb::PKG_SHAPES     // shapes/pkg.shapes.ttl    — the SHACL write-time guardrails
sparq_kb::PKG_EXAMPLE    // examples/pkg-example.ttl  — a tiny instance file
sparq_kb::PKG_INSTANCES  // ingest/pkg-instances.ttl — the Phase-1 ingested graph
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

## 🧪 Phase-1 ingestion PoC (`sq-2m6zm.2`)

`ingest/ingest_pkg.py` is the **structured-parse** ingestion pipeline over the
head docs (the highest read-frequency × formalisability slice — **not** a full
corpus capture):

- `.beads/issues.jsonl` → `pkg:Task` triples — a **mechanical** projection of the
  `bd` model (status / type / priority / labels / the typed dependency edges, with
  `bd blocks` → the §2.2 `pkg:dependsOn`, `parent-child` → `dcterms:isPartOf`). `bd`
  stays the source-of-record; the PKG mirrors it as a read-model.
- the heaviest `skills/*/SKILL.md` **front-matter** → `pkg:Source` + `pkg:Technique`.
- the hand-authored `ingest/agents-findings.ttl` (`pkg:Finding`s extracted from
  `AGENTS.md` — merge discipline + the sub-agent standing rules) appended verbatim.

Regenerate the committed `ingest/pkg-instances.ttl` from the repo root:

```text
python3 crates/sparq-kb/ingest/ingest_pkg.py \
  --beads .beads/issues.jsonl --skills-dir skills \
  --findings crates/sparq-kb/ingest/agents-findings.ttl \
  --out crates/sparq-kb/ingest/pkg-instances.ttl
```

**SHACL conformance is the gate**: the ingest **conforms with 0 violations**
(`cargo test -p sparq-kb --features validate --test ingest_shacl`). The `bd`
backlog's stale closed→open dependency edges are **guardrail-excluded** by the
script (written to `pkg-instances.ttl.stale-edges.tsv`, not silently dropped) — the
`stale_edge_is_caught_by_the_guardrail` test proves the §4.4 constraint fires on
one. The example lookups (`--features query --test ingest_query`) answer real
questions — *"merge discipline for a ZK PR?"*, *"standing rules for sub-agents?"*,
the §4.1 ready-frontier — returning the minimal triples computed by the engine.

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
- Epic `sq-2m6zm`: the ontology/shapes are `sq-2m6zm.1`; the ingestion PoC above is
  `sq-2m6zm.2`. Next: the query-the-PKG skill (`sq-2m6zm.3`) + the token-A/B harness
  (`sq-2m6zm.4`).

## License

MIT — see the repository-root [`LICENSE`](../../LICENSE). © 2026 Jesse Wright.
