# sparq-reason vs EYE — N3 inference benchmark (2026-06)

**Machine:** Apple M1 (fanless), macOS 25.4.0, same machine for both engines.
**Engines:** sparq `target/release/sparq-cli reason <f> n3 n3 /dev/null`
(parse → forward closure → serialize full closure to /dev/null) vs
**EYE v11.24.4 (2026-05-12)** on SWI-Prolog 10.0.2 (`swipl -x eye.pvm`,
saved-state install per eyereasoner/eye `install.sh`), invoked
`eye --quiet --nope <f> --pass > /dev/null` (same pipeline: parse → closure →
serialize). Wall-clock seconds. sparq: **min of 3** everywhere. EYE: min of 3
on socrates/dt1k; **single run** on dt10k/anc500/grid30 (runs are minutes —
noted per cell); dt100k **not run** (extrapolated, see below). Runner:
`bench/inference/eye-comparison.sh` (full min-of-3 protocol when you have the
hours).

EYE could not come from Homebrew (no formula); installed from the official
repo at commit `e909c5b22d4edf7b86d366b08274eea31a6f82e5` via the install.sh
recipe (`swipl -g main -- --image eye.pvm`).

## Workloads

| workload | shape | input | closure |
|---|---|---:|---:|
| socrates | vendored EYE case (1 rule, 2 facts) — startup/latency floor | 3 stmts | +1 |
| dt1k/10k/100k | DeepTaxonomy: 1 instance, N-deep `:sc` chain, 1 transitivity meta-rule (`gen_deeptaxonomy.py`) | N+2 | +N |
| anc500 | transitive `:ancestor` chain, 500 links (quadratic closure) | 501 | +124,750 |
| grid30 | 30×30 grid reachability — `edge→reach`, `reach+edge→reach` | 1,742 | +215,325 |

## Results

| workload | sparq | EYE | ratio (EYE/sparq) |
|---|--:|--:|--:|
| socrates | 0.113 s | 0.300 s | 2.7× |
| dt1k | 0.068 s | 4.19 s | 62× |
| dt10k | 0.133 s | 377.9 s ¹ | 2,840× |
| dt100k | 0.729 s | not run ² (extrap. ≈9 h) | ≈46,000× |
| anc500 | 30.0 s | 108.4 s ¹ | 3.6× |
| grid30 | 1.02 s | 68.3 s ¹ | 67× |

¹ single run (minutes per run; sparq cell still min-of-3).
² EYE's observed DT scaling here is ≈N^1.95 (4.19 s → 377.9 s for 10×);
  at that rate dt100k is ≈9 hours and was not run.

Closure-size cross-checks (engines agree): dt1k +1,000 derived both sides;
dt100k closure 200,001; anc500 closure 125,250 (= 501·500/2 pairs + originals);
grid30 closure 217,065.

## Reading (honest)

- **DeepTaxonomy is the headline**: sparq's semi-naive, delta-indexed fixpoint
  is effectively linear in the closure (dt1k→dt100k: 0.068→0.73 s, ~10× for
  100× input, startup-dominated below 10k), while this EYE version with the
  same generic meta-rule (`{?x a ?c. ?c :sc ?d} => {?x a ?d}` + `--pass`)
  scales ≈quadratically. Caveat: the RR-2023 literature reports EYE-fw at
  ~0.1 s on DT-1000 — far from the 4.2 s measured here; the published runs
  use EYE's dedicated DT setup (goal-directed query form) rather than a full
  forward closure with `--pass`, so treat cross-paper numbers as a different
  task. On the *full-materialization* task, measured same-machine, the gap is
  as tabled.
- **Pure transitive chains are the weak spot of BOTH engines** (anc500 — the
  O(N³) derivation storm of a binary transitivity rule over one chain): sparq
  30 s vs EYE 108 s. sparq still wins 3.6×, but this is sparq's worst ratio,
  and the same combinatorics shows in owl-bench's `owl-transitive` (below).
  **Optimization target** (not pursued in this thread): chain-transitivity
  needs either derivation dedup before instantiation (the N³ attempts collapse
  onto N²/2 distinct conclusions) or a path-index/SCC-condensation special
  case in the semi-naive loop; the same fix would serve prp-trp in the OWL
  path.
- **Graph traversal (grid30)** behaves like DT: delta-driven linear-ish for
  sparq (1.0 s for a 217 k closure), 67× slower in EYE.
- EYE remains the more *featureful* N3 system (full proof output `--why`,
  RDF surfaces, decades of builtins); these numbers measure forward closure
  throughput only — the thing `sparq-cli reason … n3` does — not breadth.
- sparq wall floors (~0.07–0.11 s) are process startup + parse; the engine
  time inside (CLI's own timer) is 6–10 ms on the small inputs. EYE's floor
  is ~0.2–0.3 s (SWI-Prolog saved-state boot).

## RDFS/OWL closure throughput (owl-bench.sh, same machine, min-of-3)

The materialization paths behind the rdf-mt / OWL 2 RL / entailment-regime
conformance results (`sparq_reason::materialize`, id-level, semi-naive):

| workload | shape | time |
|---|---|--:|
| rdfs-instances | 100k individuals × depth-20 subClassOf chain → 2.1 M-triple closure (+2,000,190 entailed), profile=rdfs | **0.047 s** (~44 M derived/s) |
| owl-route-rdfs | same data through profile=owl (no OWL features → single-pass RDFS route) | 0.082 s |
| owl-transitive | 2,000-edge `owl:TransitiveProperty` chain (quadratic ~2 M closure, prp-trp fixpoint) | 54.0 s |
| owl-restrictions | 50k individuals × someValuesFrom/hasValue/intersectionOf + equivalence + inverseOf | 0.089 s |

`owl-transitive` is the known weak spot of the OWL path: a pure 2k-link chain
is the worst case for prp-trp (the closure itself is ~2 M pairs and every
round re-joins the full transitive frontier). The N3 engine on the equivalent
workload (anc500, also pure-chain quadratic) is dramatically faster — the gap
is the optimization target noted below, NOT pursued in this thread.

## Artifact size

wasm32 release artifact (`cargo build -p sparq-wasm --target
wasm32-unknown-unknown --release`): **1,573,895 B** — +8 B over the tracked
1,573,887 B baseline, attributable to the `ground_triple` formula-value fix in
sparq-reason (the only engine change of this thread).
