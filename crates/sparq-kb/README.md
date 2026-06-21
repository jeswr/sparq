# sparq-kb

> 🤖 **SPARQ agent** [OPUS-4.8] — Phase-1 artifacts for dogfooding sparq as a
> **Project Knowledge Graph (PKG)**: store the project's own research findings,
> sources, techniques, and `bd` task model as RDF behind sparq's SPARQL + SHACL +
> reasoning surface. Research prototype; adoption is gated on a token-A/B verdict
> (epic `sq-2m6zm`, design record `research/dogfooding-sparq-knowledge-graph.md`,
> PR #1063). NOT published.

## 🚀 Quickstart

The crate ships four Turtle artifacts as `&str` constants (default build is a pure data +
Rust-vocab crate — no engine code):

```text
sparq_kb::PKG_ONTOLOGY   // ontology/pkg/pkg.ttl      — the reuse-first PKG vocabulary
sparq_kb::PKG_SHAPES     // shapes/pkg.shapes.ttl     — the SHACL write-time guardrails
sparq_kb::PKG_EXAMPLE    // examples/pkg-example.ttl  — a tiny instance file
sparq_kb::PKG_INSTANCES  // ingest/pkg-instances.ttl  — the Phase-1 ingested graph
sparq_kb::vocab::*       // the pkg: IRIs as constants, pinned against the Turtle
```

Dogfood the guardrails with sparq's own SHACL engine (opt-in `validate` feature):

```text
cargo test -p sparq-kb --features validate -- --nocapture
```

This loads the ontology + example instances, runs `pkg.shapes.ttl` via `sparq-shacl`, and
asserts the valid Findings/Tasks PASS while the deliberately invalid ones (missing
source/confidence, out-of-enum status, stale dependency edge) are REPORTED.

## 🧪 Phase-1 ingestion PoC (`sq-2m6zm.2`)

`ingest/ingest_pkg.py` is the **structured-parse** ingestion pipeline over the
head docs (the highest read-frequency × formalisability slice — **not** a full
corpus capture):

- `.beads/issues.jsonl` → `pkg:Task` triples — a **mechanical** projection of the `bd`
  model (status / type / priority / labels / typed edges; `bd blocks` → §2.2
  `pkg:dependsOn`, `parent-child` → `dcterms:isPartOf`, `research/<doc>.md` spec-ref →
  `dcterms:relation` a `pkg:Source`). `bd` stays source-of-record; the PKG mirrors it.
- the heaviest `skills/*/SKILL.md` **front-matter** → `pkg:Source` + `pkg:Technique`, plus
  `ingest/agents-findings.ttl` (`pkg:Finding`s from `AGENTS.md`) appended verbatim.

Regenerate the committed `ingest/pkg-instances.ttl` from the repo root:

```text
python3 crates/sparq-kb/ingest/ingest_pkg.py \
  --beads .beads/issues.jsonl --skills-dir skills \
  --findings crates/sparq-kb/ingest/agents-findings.ttl \
  --out crates/sparq-kb/ingest/pkg-instances.ttl
```

**SHACL conformance is the gate**: the ingest **conforms with 0 violations**
(`--features validate --test ingest_shacl`). The `bd` backlog's stale closed→open
dependency edges are **guardrail-excluded** by the script (to
`pkg-instances.ttl.stale-edges.tsv`, not silently dropped); `stale_edge_is_caught_by_the_guardrail`
proves the §4.4 constraint fires. Example lookups (`--test ingest_query`) answer real
questions returning the minimal engine-computed triples.

## 🔬 bd-bridge eval — can sparq REPLACE bd? (`sq-2m6zm.5`)

Can sparq replace `bd` for task-tracking + structural queries in RDF/SPARQL? The honest,
non-sycophantic answer (`tests/bd_bridge_eval.rs`, over the real backlog) is **BRIDGE, do
not replace** — the read-model meets **0/4** of the §4.5 gate. The **bd→RDF read-model**
(`ingest_pkg.py`) projects every issue to a faithful, SHACL-conformant `pkg:Task` and
exercises three structural queries `bd`'s flat CLI cannot do: transitive blocked-by chains
(`pkg:dependsOn+`), the §4.1 ready-frontier, and the **knowledge↔task JOIN** — which `bd`
**cannot express at all**.

| # | §4.5 replacement criterion | Met? | What `bd` does that the read-model does not |
|---|---|---|---|
| a | conflict-safe mutation ≥ Dolt 3-way merge | ❌ | read-model has no write-path / row merge; `bd`+Dolt give 3-way merge of issue rows (biggest reason to bridge) |
| b | frontier includes live git/gh/nproc state | ❌ | SPARQL is the dependency half only; `push-frontier.sh`'s in-flight / conflict-partition / CPU-cap are live state in no store |
| c | ready-query latency ≈ `bd ready` (offline) | ❌ | `bd ready` is offline, sub-second, no spin-up; sparq pays graph-load per process; nlq/frontier unexposed over CLI/HTTP |
| d | SessionStart / CLI ergonomics ≈ `bd` | ❌ | `bd` is a mature CLI + hooks + autoclose CI; sparq exposes raw SPARQL + VoID only |

**Verdict: BRIDGE.** Keep `bd` as source-of-record and mirror it into the KG — the bridge
captures all demo value; replacing the write/merge/latency/CLI estate buys none of it.
Asserted in `part2_four_criterion_replacement_gate` so no future change silently flips it.

## ✨ Features

- **Reuse-first ontology** — generalises the vendored `zkp-sparql`
  `sig-impl:Assertion` reified-claim pattern into `pkg:Finding`; reuses PROV-O,
  SKOS, DCAT, FaBiO/FRBR/DC, CiTO, schema.org, and nanopublications. Only ~4 terms
  are genuinely net-new (`pkg:exploredStatus`, `pkg:followUpPriority`,
  `pkg:confidence`, `pkg:couldBeMergedWith`) plus the single `pkg:dependsOn`
  `owl:inverseOf` `pkg:blockedBy` pair (there is **no** `pkg:blocks`). Full reuse +
  live-ontology-alignment record in [`ontology/pkg/PROVENANCE.md`](ontology/pkg/PROVENANCE.md).
- **SHACL guardrails** — `pkg.shapes.ttl` makes a source + a confidence value + an
  assurance basis + non-filler content **mandatory** on every `pkg:Finding`, and
  enforces a valid status / bounded priority / no-stale-edge on every `pkg:Task`.
- **Opt-in by construction** — default build is data + constants only; the
  `validate` feature is the only thing that pulls in `sparq-core` + `sparq-shacl`.
- **No-drift guard** — `vocab.rs` is byte-pinned against `pkg.ttl` by a sync test.
- **NL-tool envelope** (`query` feature, `sq-ve5dy`) — `query::nl_tool` runs a query
  and returns the **executed SPARQL + resolved IRIs + grounding confidence** so the
  caller can verify the answer was computed, not guessed; `pkg-query --json` emits it.
  The `model: haiku` agent-flavor sub-agent (`.claude/agents/sparq-pkg-nl.md`) drives
  it so the orchestrator pays cheap-model tokens for the verbose middle.

## 📚 Learn more

- Design record: `research/dogfooding-sparq-knowledge-graph.md` (PR #1063) — the
  reuse-first ontology (§2), SHACL guardrails (§2.4, §4.4), bd-bridge + §4.5 replacement
  gate (§4), and the falsifiable token-A/B protocol (§5).
- `ontology/pkg/PROVENANCE.md` — the per-term reuse + verified `skos:closeMatch`
  alignment record; precedent: `crates/sparq-trust/ontologies/zkp-sparql/` +
  `crates/sparq-trust/src/vocab.rs` (ship-an-ontology + Rust-constants pattern).
- Epic `sq-2m6zm`: the ontology/shapes are `sq-2m6zm.1`; the ingestion PoC above is
  `sq-2m6zm.2`; the **query-the-PKG** helper + skill (introspect→ground→ask canned
  queries via `pkg-query`, `.claude/skills/query-pkg/SKILL.md`) is `sq-2m6zm.3`; the
  bd-bridge eval + four-criterion replacement gate above is `sq-2m6zm.5`. Next: the
  token-A/B harness (`sq-2m6zm.4`).

## License

MIT — see the repository-root [`LICENSE`](../../LICENSE). © 2026 Jesse Wright.
