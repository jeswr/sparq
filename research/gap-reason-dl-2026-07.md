<!-- [FABLE-5] sq-hmd7l.10 — OWL DL (ORE) competitor gap record.
Harness description + first-read methodology for the sparq-reason-dl vs HermiT/Openllet
same-box comparison on ORE corpus ontologies. No fabricated numbers; NO hard-coded
performance numbers presented as canonical. Every timing row this harness collects is
flagged NON-canonical (canonical:false) until a dedicated quiet EC2 run re-measures it. -->

# OWL DL (ORE corpora) competitor gap — 2026-07

**Status:** harness record / first-read methodology description (NON-canonical).
**Date:** 2026-07-18.
**Bead:** sq-hmd7l.10.
**Epic:** sq-hmd7l (comparative-benchmarking-everything).
**Competitors:** `hermit-openllet` — HermiT (the reference OWL 2 DL tableau reasoner,
LGPL-3.0) and Openllet (its actively-maintained fork, Apache-2.0);
`bench/competitors.json`, suite `reason-dl-ore`. Prefer Openllet for publishable numbers
(licensing note in `bench/competitors.json`).
**Canonical run:** deferred to a dedicated quiet-box wave (`quiet_box_sensitive = true`).

---

## 0. Prior state — and honest priority framing

`sparq-reason-dl` (`tableau.rs`, `profile.rs`) had **no benchmark at all** beyond
`benches/tableau_micro.rs` — a **self-relative** Criterion probe over pinned synthetic
concept expressions (`research/gap-reason-dl-micro-2026-07.md`) with no ontology-level
run and no competitor column. `bench/benchmarks.toml` carried a `reason-dl-ore` registry
stub (from sq-hmd7l.1) with every field `TBD`.

**Honest framing (per the sq-hmd7l survey): this surface is LOWER priority than EL/QL.**
`sparq-reason-dl` is a scoped **ALCH** profile-checker/tableau — the L1–L3 layers of the
Direct-Semantics workstream (`research/owl2-direct-semantics-scoping.md`) — **not** the
headline reasoner and **not** full OWL 2 DL (no inverses, cardinality, nominals,
datatypes, `sameAs`; transitive roles only behind the opt-in `dl_transitive` feature).
On real ORE corpora it is *expected* to abstain (`unknown(out-of-fragment)`) on every
ontology outside ALCH, and the harness records that abstention as an abstention — the
comparison's first deliverable is an honest **coverage picture** (what fraction of a real
DL corpus the scoped fragment can even engage), not a speed claim.

This bead delivers the sparq-side harness FIRST (Phase 1) and the comparison columns
second (Phase 2), and fills the registry stub in.

## 1. The verdict ORACLE — cross-check BEFORE timing (the sq-hmd7l.10 invariant)

The comparison metric is the **ontology consistency verdict**
(`consistent`/`inconsistent`) — the ORE competition's consistency task — plus wall time.

**NO timing row without verdict agreement.** For every ontology each engine's verdict is
recorded and cross-checked **before** any timing is trusted; a timing row is emitted only
when at least two engines return a *definitive* verdict and all definitive verdicts
agree. A disagreement is a **correctness finding**: the envelope records
`verdicts_agree=false` with **all timings nulled**, and a bug bead is filed before any
rerun — the harness never times past it. This is *stricter* than the reason-el count
flag (which recorded timing alongside `counts_agree=false`): a DL verdict is a single
bit, so a disagreement voids the row outright. sparq abstentions (`unknown(...)`) are
excluded from agreement — an abstention can neither agree nor disagree, and it never
gets a timing entry.

## 2. Phase 1 — the sparq-side harness (`examples/ore_bench.rs`)

`crates/sparq-reason-dl/examples/ore_bench.rs` runs the real fail-closed pipeline
(parse → L1 `extract` → L2 `profile` → L3 tableau `consistency`) in two modes:

- **SMOKE** (the acceptance gate; hermetic, no network/JVM):
  `cargo run -p sparq-reason-dl --release --example ore_bench -- --smoke` exits 0 iff
  the pinned verdicts on the **vendored ORE-style subset** still hold (§3).
- **GATHER** (`<path> [format]`): one ontology per invocation, printing
  `verdict= profile_el= profile_ql= profile_rl= axioms=` **before** `extract_s=`/`check_s=`
  so the wrapper can enforce §1 structurally. No assertions — real-corpus verdicts are
  cross-checked by the wrapper, never baked in. Verdicts run under the tableau's
  deterministic count budget (`ORE_BENCH_MAX_NODES` / `ORE_BENCH_MAX_RULE_APPS`);
  exhaustion prints `unknown(budget:…)`, an abstention, never a verdict.

## 3. The vendored smoke subset — pinned, hand-verified

The real ORE 2014/2015 corpora are **gather-only downloads** (multi-GB; big corpora stay
out of git per AGENTS.md), so the hermetic acceptance fixtures are four small
**hand-authored** ontologies in the shape of ORE consistency/satisfiability tasks
(`examples/data/ore_smoke_*.ttl`), each with a hand-verified pinned verdict:

| fixture | task shape | pinned verdict |
|---|---|---|
| `ore_smoke_univ` | consistency | **consistent** (⊔-branching + ∃ + role hierarchy) |
| `ore_smoke_role_disjoint` | consistency | **inconsistent** — the clash is *derived* (∀ on a super-role constrains a sub-role edge + disjointness), so the seed-clash shortcut cannot fire and the tableau must actually run (non-vacuous) |
| `ore_smoke_madcow` | class satisfiability | ontology **consistent**, class `:MadCow` **unsatisfiable** |
| `ore_smoke_cardinality` | out-of-fragment | **unknown(out-of-fragment)** — pins the fail-closed abstention §1 depends on |

Each in-fragment fixture also pins its **EL/QL/RL profile memberships** (hand-derived
from the W3C OWL 2 Profiles grammar; rationale in the example), so `profile.rs` — the
other unbenchmarked half of the surface — sits under the same gate.

## 4. Phase 2 — HermiT / Openllet columns (`scripts/bench/reason-dl-same-box.sh`)

The wrapper mirrors `scripts/bench/reason-el-same-box.sh` (same envelope shape, same
gather-only-deps discipline; JRE CLI adapters in the JenaShaclBench spirit — the
reasoners' own CLIs, no committed Java):

- `--smoke` — builds + runs the sparq example on the vendored subset. The exit-0
  acceptance path; no downloads, no JVM.
- Full mode — `ORE_CORPUS_DIR` (a local directory of ORE ontology files; **not**
  auto-downloaded, hosting has moved over the years — see `bench/competitors.json`)
  is walked up to `ORE_MAX_ONTOLOGIES` files (capped LOUDLY, never silently). Per
  ontology: riot converts OWL→NT for sparq; HermiT runs `-k` (consistency); Openllet
  runs `consistency`. Verdicts are normalised, cross-checked per §1, and one envelope
  per ontology lands in `OUT_DIR` with `verdict_before_timing` recorded ahead of the
  engine rows. Jar paths come from `HERMIT_JAR` / `OPENLLET_JAR` (pinned at gather
  time; recorded in the envelope; missing jars record an honest ERROR row).

## 5. What is pinned vs deferred

**Pinned now:** the four smoke verdicts + profile memberships (assertions in the
example); the envelope schema; the §1 invariant (structurally enforced — timings are
nulled unless `verdicts_agree=true`).

**Deferred:** ORE corpus sha256 pins and reasoner jar versions are recorded **at gather
time** (`unverified_pin` honesty caveat in `bench/competitors.json` stands until the
first real gather); canonical timings await the dedicated quiet-box wave. **No timing
number in this record or anywhere in tracked markdown** — work-box wall-clock is
trend-only, `canonical:false` in every envelope.
