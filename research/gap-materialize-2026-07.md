# Materialization competitor comparison — gap analysis (D6)

> 🤖 SPARQ agent. [FABLE-5] sq-hmd7l.7. **FIRST-READ / NON-CANONICAL.** This record
> documents the *method and the fidelity caveats* of the same-box reasoning
> (deductive-closure materialization) comparison. It deliberately contains **no
> hard-coded performance numbers** (per repo hygiene: work-box timings are
> non-canonical and live only in the PR body / bead comments, never in committed
> markdown). Citable numbers come from the `CANONICAL=1` dedicated-quiet-box run
> (sq-hmd7l.26, `univ≥100`), never from this work box.

## Axis

Gap-table row **D6 = materialization throughput**, previously NOT-MEASURED in the
RDFox comparison matrix. The question: how fast does sparq compute the full
deductive closure of an RDF graph, and how does that compare to the mainstream
reasoners a reviewer expects — Apache Jena's rule reasoners, and the
high-performance Datalog engines VLog and Nemo?

## Workload + oracle

- **Corpus:** the deterministic LUBM(`univ`) ABox + the Univ-Bench OWL TBox
  (`bench/lubm/gen.sh`, READ-ONLY; `bench/lubm/run.sh` untouched), combined into
  one N-Triples input every engine closes. Scales `univ=1` (~100k) and `univ=10`
  (~1.3M) locally; `univ≥100` is the canonical EC2 wave (sq-hmd7l.26).
- **Oracle = closure size.** The load-bearing invariant is that no throughput row
  is trusted without a closure-count cross-check. sparq's `reason` self-reports its
  closure count; the harness (`scripts/bench/materialize-same-box.sh`) asserts it
  against a pinned per-profile expected at `univ=1` and records every engine's
  closure size in the envelope's `count_crosscheck`.

## The load-bearing caveat: rule-set / profile fidelity is NOT the same across engines

A "closure" is only comparable if the **rule set** is the same. It is not, and the
harness records the difference **per column** rather than silently absorbing it:

| engine | rule set actually run | comparability to sparq `reason owl` |
|---|---|---|
| **sparq** `reason owl` | FULL W3C **OWL 2 RL/RDF** rule table (`crates/sparq-reason/src/owl.rs`: cls-*/cax-*/scm-*/prp-*, incl. prp-trp / prp-inv / cls-svf / cls-int) | reference |
| **sparq** `reason rdfs` | RDFS subset (rdfs2/3/5/7/9/11) | reference (RDFS) |
| **Apache Jena** | Jena's OWN rule reasoners — `OWL_MICRO` / `OWL_MINI` / `OWL` are OWL-**subset** reasoners (no full OWL 2 RL), plus Jena's RDFS ruleset. Jena also adds axiomatic/reflexive triples and de-dups the ABox on load | **profile-different** — closure size differs *by construction*; a raw size delta is NOT a correctness gap |
| **VLog** / **Nemo** | general **Datalog** engines — need a separately-validated OWL-RL/RDFS Datalog encoding (`.dlog` / `.rls`) whose closure reproduces sparq's | not directly comparable until a validated encoding exists |

Why VLog/Nemo need a *validated* encoding and not a drop-in: the same reason EYE
was de-scoped from the LUBM entailed tier (`bench/competitors.json` #eye) — the
repo's only OWL-in-rules file is a ~12-rule demo that OMITS
transitivity/someValuesFrom/intersectionOf/inverseOf, so its closure would
**under-count** the exact OWL rules LUBM Q6/Q9/Q11/Q12/Q13 depend on. Authoring +
validating a faithful OWL 2 RL Datalog rule table is its own task; until it lands,
the VLog/Nemo columns emit an honest `NOT-RUN-LOCALLY` with the exact blocker (see
the follow-up beads), never a fabricated number.

## Harness

`scripts/bench/materialize-same-box.sh` (envelope mirrors
`scripts/bench/shacl-same-box.sh`):

- **sparq** via `sparq-cli reason <combined> ntriples <profile> <out.nt>`
  best-of-N; timed = sparq's self-reported materialize time (parse excluded), so
  it is the loaded-graph-materialize figure comparable to the others.
- **Jena** via `scripts/bench-adapters/jena_reason_adapter.java` — load once
  (advisory), time `InfModel` materialization best-of-N; one JVM per profile under
  `timeout` (JVM start-up + parse outside the timed section). A timeout degrades to
  an honest `ERROR` row (Jena's `OWL_MINI`/`OWL` reasoners are slow enough on
  LUBM(1) to time out — recorded, not hidden).
- **VLog / Nemo** via `scripts/bench-adapters/{vlog,nemo}_adapter.py` —
  `NOT-RUN-LOCALLY` unless the binary AND a validated rules file are supplied.

Emits one `bench/canonical-competitor-results/`-shaped envelope per scale;
`canonical:false` unless `CANONICAL=1`. Gather-only Jena tarball in `/tmp` (engines
stay out of git). Acceptance: `ONLY=sparq LUBM_UNIVS=1 …` exits 0, asserts the
pinned closure counts, emits a well-formed envelope.

## Performance-dominance disposition

On this work box, at `univ=1`/`univ=10`, sparq's OWL-RL/RDFS materialization is
faster than Jena's closest (profile-different, smaller) rule reasoner by a wide
margin — sparq is **ahead**, so no per-gap P1 fix bead is warranted for a
regression. The follow-up beads are the *encoding-fidelity* work needed to turn the
VLog/Nemo columns from `NOT-RUN-LOCALLY` into a real like-for-like comparison, plus
the canonical EC2 wave. See the PR body for the (non-canonical) directional table.
