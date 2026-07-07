<!-- [OPUS-4.8] sq-7iai — SHACL validation benchmark suite. Design: research/capability-benchmark-program.md §3.1. -->
# SHACL validation suite

The SHACL analogue of the LUBM template: an **overview** dashboard row, a **self-asserting
deterministic gate** (regression alerts), and a **competitor** comparison surface. It is the
cleanest competitor surface — Jena-SHACL / pySHACL / rdf-validate-shacl all run the *identical*
`(data graph, shape graph)` pair and emit a `sh:ValidationReport`, so `#violations`/`conforms`
cross-check before any timing is trusted.

## Data substrate

The suite reuses the **LUBM(1) ABox** (`bench/lubm/gen.sh -univ 1 -seed 0`, ~103k N-Triples, the
sha256-pinned UBA generator) as its SHACL **data graph** — exactly what Trav-SHACL / Re-SHACL do.
`bench/shacl/gen.sh` is a thin wrapper over the LUBM generator: it emits two paths on stdout, the
data N-Triples and the committed `shapes/` directory. The workloads validate the **raw ABox** (no
entailment), so their violation counts are deterministic constants at the pinned corpus.

Scale tiers: `univ=1` (per-commit, ~103k triples); `univ=5`/`univ=10` (EC2/heavy — the shapes are
scale-parametric, but `expected.tsv` is pinned to `univ=1`).

## Workloads — the 5 committed shape graphs (`shapes/*.ttl`)

| Workload | Constraints exercised | Targets |
|---|---|---|
| `cardinality` | `sh:minCount` / `sh:maxCount` (focus-node enumeration + path counting) | `ub:FullProfessor` |
| `datatype_range` | `sh:datatype` + `sh:pattern` (XSD lexical space) | `ub:GraduateStudent` |
| `class_nodekind` | `sh:class` (subclass-closure target) + `sh:nodeKind` (target selection at scale) | `ub:FullProfessor` |
| `node_paths` | `sh:node` + sequence/inverse paths (inter-shape recursion — Trav-SHACL's hard case) | `ub:GraduateStudent` |
| `sparql_constraint` | `sh:sparql` (SHACL §5.2) routed through `sparq-engine` — the path unique to sparq's architecture (kept separate, **not** the headline) | `ub:GraduateStudent` |

Each shape is authored so a **fixed fraction of focus nodes is invalid** at the pinned corpus, making
the violation count a deterministic constant. The constants in `expected.tsv` were **derived by
running `sparq-shacl`** on the corpus (not guessed) and each is **independently cross-checked against
the raw ABox** (see the `expected.tsv` header).

## Deterministic gate (HARD) vs timing (ADVISORY)

`run.sh` is the self-asserting entry point (the LUBM pattern). It validates with the
`crates/sparq-shacl/examples/bench_shacl.rs` runner and **asserts, per workload, vs `expected.tsv`,
exit 1 on any drift**:

- **`<workload>_violations`** — `report.results.len()` (the primary gate).
- **`<workload>_conforms`** — `report.conforms` as 0/1 (a correctness drift detector).
- **`<workload>_focus_nodes`** — the distinct focus nodes the targets select (a target-selection
  regression detector; engine-independent — pure target selection via `sparq_shacl::count_focus_nodes`).

Plus the **`shacl_w3c_pass` ratchet** — the W3C `data-shapes` core pass count (only tightens) — which
lives in `crates/sparq-shacl/tests/w3c_core.rs` (`BASELINE_PASS`, asserted by `cargo test`), not on the
benchmark dashboard.

**Timing is ADVISORY** (`mode:noise`, trend-only, **never hard-gated** — and this dev box is
non-canonical, so its timings are advisory only): the ci-bench hook harvests
`shacl_<workload>_validate_us` (the headline validate time) into the dashboard; it is **not** in
`scripts/perf-gate.py`. `bench_shacl` also reports a load time per run (advisory). The hard
correctness gate lives in `run.sh`'s `expected.tsv` diff, so the harvested timings stay advisory.

## Running it

```sh
cargo build --release -p sparq-shacl --example bench_shacl
bench/shacl/run.sh                       # self-asserting: exit 1 on any count drift
```

The G1 runner (`bench_shacl`) emits, per `shapes/*.ttl`, a 6-column TSV
`name<TAB>violations<TAB>validate_us<TAB>conforms<TAB>focus_nodes<TAB>load_us`. `run.sh` asserts the
deterministic columns and forwards the `name<TAB>violations<TAB>validate_us` 3-column contract the
ci-bench hook consumes. (`sparq-shacl` is the *isolated* SHACL crate — not a `sparq-cli` dependency —
so the runner is a crate `--example`, not a CLI subcommand.)

## Competitors

All run the identical `(data, shapes)` pair and emit a `sh:ValidationReport`:

| Engine | Lang | License | Adapter kind | Role |
|---|---|---|---|---|
| **Apache Jena SHACL** (`shacl validate`) | JVM | Apache-2.0 | `report-cli` | Tier-1 correctness oracle + mainstream baseline |
| **pySHACL** | Python | Apache-2.0 | `report-cli` | Tier-1 W3C reference impl; the correct-but-slow anchor |
| **Zazuko rdf-validate-shacl** | JS | MIT | `js-lib` | WASM-peer for sparq's browser SHACL story |

Registered in `bench/competitors.json` (with the `engines`/`values` dashboard seam **empty** in git
per AGENTS.md — no hard-coded perf). A real `scripts/gather-competitors.sh --run --only <id>` writes
git-ignored `bench/competitor-results/`. Docker-based competitors (Jena via `stain/jena`) are
inherently gather-only on a Docker EC2 box (no Docker on the dev box), so they add zero recurring CI
cost. The shared adapters live in `scripts/bench-adapters/` (`report_cli_adapter.py` +
`shacl_report_count.py`; `js_shacl_adapter.mjs`).

**Honest caveat:** `sparq-shacl` does **not** deduplicate results across traversal routes (matching
the W3C suite's per-occurrence expectations). Engines that *do* dedup will report a different
`#violations` for the same data, so a cross-engine comparison uses **per-engine expected counts**
(`scripts/bench-adapters/cross_check_shacl.sh`), not the single sparq constant in `expected.tsv`.

**Same-box comparison harness** (`sq-7d3dj.33`): `scripts/bench/shacl-same-box.sh` runs sparq vs
pySHACL vs Jena SHACL **in-process, validate-only, best-of-N** over these workloads *plus* the
SPARQL-constraint-heavy `shapes-sparql/sparql_heavy.ttl` set (kept out of `shapes/` so this
suite's `expected.tsv` gate is untouched), cross-checks counts per workload, and emits
`bench/canonical-competitor-results/`-shaped envelopes (`canonical:false` off the quiet box).
First read + the `sh:sparql` root-cause: `research/shacl-baseline-2026-07.md`.
`<focus_nodes>` is engine-independent. `sh:sparql` coverage differs across engines — scope perf to
constraints all Tier-1 implement.
