<!-- [OPUS-4.8] Authored by Opus 4.8 (Fable unavailable; flag for re-review when Fable returns). -->
# LUBM (Lehigh University Benchmark) — extensional + entailed tiers

The canonical *reasoning* SPARQL suite: a synthetic university dataset over the Univ-Bench OWL
ontology plus 14 queries that, unlike the operator-focused suites (SP2Bench, WatDiv), are
designed so that **several queries only return their complete answer once OWL/RDFS entailment
has been applied**. That makes LUBM the suite that exercises sparq's reasoner
(`sparq-cli reason rdfs|owl`) end-to-end, not just its query engine. Registry entry: `lubm` in
[`bench/benchmarks.toml`](../benchmarks.toml).

> **Attribution.** Data + queries are the Lehigh University Benchmark, Yuanbo Guo, Zhengxiang
> Pan, Jeff Heflin, *"LUBM: A Benchmark for OWL Knowledge Base Systems"* (Journal of Web
> Semantics, 2005). The UBA (Univ-Bench Artificial) data generator and the Univ-Bench OWL
> ontology are from the Lehigh SWAT Lab. `gen.sh` builds the generator from the maintained
> source mirror **[github.com/nandana/LUBM-UBA](https://github.com/nandana/LUBM-UBA)**, pinned
> at commit `05c3f3a30b8d1872faad8a40e29b8e81daa15894` (UBA 1.7). That mirror is **Apache-2.0**
> per its `pom.xml <licenses>` (NB: this corrects the bead's "GPL-2.0" note — the pinned source
> is Apache-2.0). The 14 `.rq` files here are faithful SPARQL 1.1 translations of the canonical
> LUBM queries Q1-Q14. We do not vendor the generator; `gen.sh` clones the pinned commit and
> compiles it.

## Layout

```
bench/lubm/
├── gen.sh                  build-once-cache the UBA jar; emit a FIXED LUBM(1) ABox + the TBox (both N-Triples)
├── run.sh                  CI entry point: extensional tier on raw data; entailed tier on the OWL-RL closure
├── expected-rows.tsv       deterministic per-commit solution counts for BOTH tiers (correctness diff)
├── queries-extensional/    Q1, Q2, Q3, Q14  — need NO reasoning; run against the raw ABox
└── queries-entailed/       Q4-Q13           — reasoning-dependent; run against the OWL-RL closure
```

## The reasoning split (the point of this suite)

Each entailed query returns **0 on the raw data** and its correct count **only after the OWL-RL
closure is materialized** — that delta is what the suite tests. Measured at the fixed LUBM(1)
`-univ 1 -seed 0` corpus (≈100.5k distinct triples; the OWL-RL closure adds ≈49.7k entailed
triples):

| query | tier | rows (raw → reasoned) | OWL/RDFS feature exercised |
|---|---|---|---|
| Q1  | extensional | 4 | leaf type + property (none) |
| Q2  | extensional | 0 | leaf types only; 0 is the canonical LUBM(1) answer |
| Q3  | extensional | 6 | leaf type + property (none) |
| Q14 | extensional | 5916 | leaf type (none) |
| Q4  | entailed | 0 → 34 | `rdfs:subClassOf` (Professor over the leaf ranks) |
| Q5  | entailed | 0 → 719 | `rdfs:subClassOf` (Person) |
| Q6  | entailed | 0 → 7790 | OWL restriction class (Student = undergrad **+** grad — see below) |
| Q7  | entailed | 0 → 67 | Student closure |
| Q8  | entailed | 0 → 7790 | Student closure |
| Q9  | entailed | 0 → 208 | Student **and** Faculty closure (the heaviest query) |
| Q10 | entailed | 0 → 4 | Student closure (grad takers) |
| Q11 | entailed | 0 → 224 | `owl:TransitiveProperty` (`subOrganizationOf`) |
| Q12 | entailed | 0 → 15 | OWL defined-class `Chair` (intersectionOf) + transitive `subOrganizationOf` |
| Q13 | entailed | 0 → 1 | `owl:inverseOf` (`hasAlumnus`) + `rdfs:subClassOf` (Person) |

**Why the entailed tier uses `owl`, not `rdfs`.** RDFS reasoning is *incomplete* for this suite.
The Univ-Bench ontology does **not** assert `GraduateStudent rdfs:subClassOf Student`; instead
`GraduateStudent ⊑ (takesCourse some GraduateCourse)` and `Student` is the **defined class**
`Person ⊓ (takesCourse some Course)`. Concluding that grad-student individuals are Students
requires OWL `someValuesFrom` / `intersectionOf` classification, which RDFS lacks. Measured:
RDFS-only under-counts Q6/Q8 (5916 instead of 7790 — it misses the 1874 grad students) and
returns **0** for Q10/Q11/Q12/Q13 (no transitivity, no inverse, no defined-class). OWL-RL
(`sparq-cli reason ... owl`) produces the complete, correct answers in the table above, so
`run.sh` materializes the **OWL-RL** closure for the entailed tier.

## Run it

```sh
cargo build --release -p sparq-cli
bench/lubm/run.sh                 # ensures data, runs both tiers, asserts expected-rows.tsv
```

`run.sh`'s exact reasoning invocation (confirmed against `crates/sparq-cli/src/main.rs`):

```sh
# materialize the OWL-RL closure of (ABox + Univ-Bench TBox) as N-Triples:
sparq-cli reason <data+ontology>.nt ntriples owl <closure>.nt
# then run each tier in count mode:
sparq-cli bench <raw-abox>.nt  ntriples bench/lubm/queries-extensional 3 count
sparq-cli bench <closure>.nt   ntriples bench/lubm/queries-entailed    3 count
```

`run.sh` exits 1 if any query's solution count diverges from `expected-rows.tsv`, so a reasoner
*or* engine regression fails the build. The counts are deterministic (fixed `-univ 1 -seed 0`;
the UBA RDF/XML is byte-identical across runs — sha256-verified during development).

## Generator decision (empirical)

The UBA generator is tiny pure-Java (5 source files importing only `java.io.*`/`java.util.*` —
no third-party jars), so `gen.sh` compiles it with the JDK's own `javac` + `jar` and **skips
Maven** (upstream ships a `pom.xml`, but nothing needs it). The committed `univ-bench.owl` in
the mirror is wrapped in an **Apple Safari Webarchive** (a `bplist` container) that `rapper`
cannot parse; `gen.sh` byte-extracts the embedded RDF/XML payload (first `<?xml` … `</rdf:RDF>`)
with a one-line `perl`, and the result is triple-for-triple identical to the canonical copy at
`http://swat.cse.lehigh.edu/onto/univ-bench.owl` (verified). RDF/XML → N-Triples uses
`rapper` (raptor2-utils). Hermetic after first run: one shallow pinned `git clone`, then the jar
+ corpus + ontology are cached under `$LUBM_CACHE` (default `/tmp/lubm`, ~58 MB), so steady-state
per-commit runs do no network and no rebuild. The corpus is gitignored & regenerable (see
`bench/CATALOG.md` disk discipline).

## Tiering — per-commit (LUBM(1)) vs EC2/nightly (LUBM(1000))

The per-commit path uses **LUBM(1)** (~100k triples; full pipeline incl. OWL-RL closure runs in
a few seconds, well within CI budget). The **full-scale tier is LUBM(1000)** (~133M triples)
with the full OWL closure, which belongs to `bench-ec2.yml` / nightly with per-query timeouts and
result-size assertions — run `bench/lubm/gen.sh 1000 0` there (the same generator, larger
`-univ`; expect a multi-GB N-Triples corpus + a much larger closure, so size the instance disk
accordingly and clean `$LUBM_CACHE` afterward).
