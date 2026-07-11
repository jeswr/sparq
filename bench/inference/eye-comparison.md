# sparq-reason vs EYE — N3 inference benchmark (2026-06, updated fixpoint-opt)

> **Update (sq-hmd7l.11, 2026-07)** [FABLE-5]: the runner grew two optional
> competitor columns — **cwm** (W3C SWAP, Python; `cwm <f> --think --data`) and
> **jen3** (`java -jar jen3.jar -n3 <f> -conclusion`; a Java/Apache-Jena fork
> with N3 support, GitHub release v0.0.1 — *not* an npm library). Absent tool ⇒
> the column prints `absent` and the run stays green; EYE remains the pinned,
> required reference. Before any competitor cell is timed, its closure count is
> now cross-checked in-run against the deterministic expected sizes below
> (sparq's own count is asserted too, mirroring `bench/deep-taxonomy`); jen3 is
> gated via its rule-free `-inferences` output (ground + derived = closure),
> cwm via its rule-stripped `--data` closure. cwm defaults to the small cells
> only (`CWM_HEAVY=1` opts into the rest) — it is the honest *slow* column.
> No numbers in this file were re-measured; the tables below remain the 2026-06
> EYE-only record. Method + caveats: `research/gap-n3-2026-07.md`.

**Machine:** Apple M1 (fanless), macOS 25.4.0, same machine for both engines.
**Engines:** sparq `target/release/sparq-cli reason <f> n3 n3 /dev/null`
(parse → forward closure → serialize full closure to /dev/null) vs
**EYE v11.24.4 (2026-05-12)** on SWI-Prolog 10.0.2 (`swipl -x eye.pvm`,
saved-state install per eyereasoner/eye `install.sh`), invoked
`eye --quiet --nope <f> --pass > /dev/null` (same pipeline: parse → closure →
serialize). Wall-clock seconds. Runner: `bench/inference/eye-comparison.sh`
(NOTE: as written it runs EYE on dt100k once — that single run is hours; skip
that cell unless you mean it).

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

## Results (2026-06-12, post fixpoint-opt — same-session head-to-head)

Both engines measured in the SAME session on a heavily contended machine
(load avg 30–125 from concurrent workloads): the absolute numbers are
inflated ~2–3× vs the idle-machine table below (EYE dt1k idle 4.19 s vs 10.1 s
here; sparq floors likewise), but the comparison is internally fair. sparq:
min of 3. EYE: min of 3 on socrates/dt1k, single run on dt10k/anc500/grid30
(minutes per run), dt100k not run (extrapolated ≈9 h at EYE's observed ≈N^1.95
DT scaling even on an idle machine).

The same-session head-to-head (sparq vs EYE wall-clock and the EYE/sparq ratio per
workload) is produced by the runner `bench/inference/eye-comparison.sh`; run it for the
numbers (the EYE column is a CITED third-party measurement of EYE v11.24.4). The
load-bearing finding: sparq is orders of magnitude faster than EYE on the DeepTaxonomy
chains (the gap widens with N — EYE scales ≈N^1.95, so dt100k was extrapolated, not run)
and faster on the quadratic-closure anc500 / grid30 cases, while computing identical
closures (cross-checked below).

Closure-size cross-checks (engines agree, and identical to the
pre-optimization sparq closures): dt1k 2,001; dt10k 20,001; dt100k 200,001;
anc500 125,250 (= 501·500/2 pairs + originals); grid30 217,065.

## What the fixpoint-opt thread changed (before/after, same session)

Engine-internal closure time (the CLI's own `in Xs` timer — excludes process
startup/serialization, so it is robust to machine load), min of 5 interleaved
runs, baseline `eac94d7` vs this branch. Closures byte-identical.

| workload | before (eac94d7) | after | what did it |
|---|--:|--:|---|
| anc500 | 52.0 s ² | **0.180 s** | linearized transitivity fast path (N3) |
| grid30 | 1.94 s | **1.02 s** | StepMode: no proof bookkeeping on the closure path |
| dt100k | 0.757 s | **0.617 s** | StepMode |
| dt10k | 0.058 s | **0.045 s** | StepMode |
| dt1k | 0.006 s | 0.005 s | — (startup floor) |
| socrates | ~0 s | ~0 s | — |

² single baseline run (the pre-fix derivation storm is ~a minute per run).

The two optimizations (commits `1eab571`, `b8f001a` — the OWL analogue is
`579a0d4`):

1. **Transitivity linearization.** `{?x P ?y. ?y P ?z} => {?x P ?z}` under
   semi-naive evaluation is NONLINEAR — each new fact joins the FULL
   P-relation, so an N-chain re-derives every closure pair once per
   intermediate node, O(N³) bindings. Rules of that exact shape are detected
   and evaluated as the LINEAR rule `R(x,y), GEN(y,z) ⊢ R(x,z)` where GEN is
   the set of P-edges not derived by the rule itself (TC(GEN) = TC(R)) —
   O(N²), two adjacency lookups per delta fact, no binding machinery. The
   same construction fixed OWL-RL prp-trp (below).
2. **StepMode.** The closure loop recorded a full proof step (premises
   re-instantiated + fact lookups, then a second interning pass) per derived
   fact even when the caller discards them; the closure-only entry points now
   skip all of it.

## Reading (honest)

- **DeepTaxonomy**: sparq's semi-naive, delta-indexed fixpoint is effectively
  linear in the closure, while this EYE version with the same generic
  meta-rule (`{?x a ?c. ?c :sc ?d} => {?x a ?d}` + `--pass`) scales
  ≈quadratically. Caveat: the RR-2023 literature reports EYE-fw at ~0.1 s on
  DT-1000 — far from what we measure; the published runs use EYE's dedicated
  DT setup (goal-directed query form) rather than a full forward closure with
  `--pass`, so treat cross-paper numbers as a different task. On the
  *full-materialization* task, measured same-machine, the gap is as tabled.
- **Pure transitive chains were the weak spot of BOTH engines** — the O(N³)
  derivation storm of a binary transitivity rule over one chain. It WAS
  sparq's worst ratio (3.6× at 30 s); the linearization makes anc500 a
  ~300× win (0.18 s engine-internal for the 125 k closure). EYE retains the
  nonlinear evaluation (267 s here, 108 s idle).
- **Graph traversal (grid30)** behaves like DT: delta-driven linear-ish for
  sparq, ~77× slower in EYE.
- EYE remains the more *featureful* N3 system (full proof output `--why`,
  RDF surfaces, decades of builtins); these numbers measure forward closure
  throughput only — the thing `sparq-cli reason … n3` does — not breadth.
- sparq wall floors are process startup + parse; the engine time inside
  (CLI's own timer) is 5–10 ms on the small inputs. EYE's floor is
  ~0.2–0.3 s idle (SWI-Prolog saved-state boot), ~1 s under this contention.

## Idle-machine table (2026-06, pre-linearization — kept for absolute numbers)

Measured before the fixpoint-opt thread on the same machine, idle. sparq
cells are the OLD engine (pre-linearization, with proof bookkeeping); the
EYE cells are the best (least-contended) EYE measurements available.

| workload | sparq (old) | EYE | ratio |
|---|--:|--:|--:|
| socrates | 0.113 s | 0.300 s | 2.7× |
| dt1k | 0.068 s | 4.19 s | 62× |
| dt10k | 0.133 s | 377.9 s ¹ | 2,840× |
| dt100k | 0.729 s | not run (extrap. ≈9 h) | ≈46,000× |
| anc500 | 30.0 s | 108.4 s ¹ | 3.6× |
| grid30 | 1.02 s | 68.3 s ¹ | 67× |

¹ single run.

## RDFS/OWL closure throughput (owl-bench.sh, same machine, min-of-3)

The materialization paths behind the rdf-mt / OWL 2 RL / entailment-regime
conformance results (`sparq_reason::materialize`, id-level, semi-naive).
Idle-machine numbers except owl-transitive's "after", whose before/after pair
was measured in the contended session (engine-internal timer):

| workload | shape | time |
|---|---|--:|
| rdfs-instances | 100k individuals × depth-20 subClassOf chain → 2.1 M-triple closure (+2,000,190 entailed), profile=rdfs | **0.047 s** (~44 M derived/s) |
| owl-route-rdfs | same data through profile=owl (no OWL features → single-pass RDFS route) | 0.082 s |
| owl-transitive | 2,000-edge `owl:TransitiveProperty` chain (quadratic ~2 M closure, prp-trp fixpoint) | 54.0 s → **0.25 s** (fixpoint-opt: prp-trp linearized via generator edges, commit `579a0d4`) |
| owl-restrictions | 50k individuals × someValuesFrom/hasValue/intersectionOf + equivalence + inverseOf | 0.089 s |

`owl-transitive` WAS the known weak spot of the OWL path — every round
re-joined the full transitive frontier. The generator-edge linearization
(same idea as the N3 fast path) removes it; the closure (2,001,001) is
identical. rdfs-instances / owl-route-rdfs / owl-restrictions are unchanged
within measurement noise (paired interleaved runs under contention agree to
~10%, well inside the run-to-run spread).

## Artifact size

wasm32 release artifact (`cargo build -p sparq-wasm --target
wasm32-unknown-unknown --release`): **1,573,895 B** — unchanged through the
fixpoint-opt thread (linearization + StepMode + the PropExpand domain/range
fix); +8 B over the tracked 1,573,887 B baseline, attributable to the earlier
`ground_triple` formula-value fix in sparq-reason.
