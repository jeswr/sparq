<!-- [FABLE-5] Authored by Claude Fable 5. 🤖 SPARQ agent — sq-hmd7l.30 / sq-hmd7l.31. -->
# reason-encodings — validated VLog / Nemo Datalog encodings of sparq's LUBM closure

Datalog rule programs that make the **VLog** and **Nemo** columns of the materialization
same-box comparison ([`scripts/bench/materialize-same-box.sh`](../../scripts/bench/materialize-same-box.sh))
run a *like-for-like* closure vs `sparq-cli reason`. VLog and Nemo are **general Datalog
engines**, not native OWL 2 RL reasoners — a fair comparison needs a rule program whose
forward materialization reproduces sparq's closure. These encodings do exactly that, and
are **VALIDATED set-for-set** against sparq on LUBM(1) (`-univ 1 -seed 0`):

| profile | sparq closure | VLog `T` | Nemo `closed` | set-identical? |
|---------|--------------:|---------:|--------------:|:--------------:|
| `owl`  (OWL 2 RL) | **150589** | 150589 | 150589 | yes (0 diff both directions) |
| `rdfs` (RDFS subset) | **126732** | 126732 | 126732 | yes (0 diff both directions) |

"Set-identical" = every non-blank-node triple matches, and the blank-node-containing
triples match in count and predicate histogram (blank-node *labels* are opaque, so the
19 fixed TBox bnodes are compared structurally). The counts are the harness's pinned
acceptance oracle (`KNOWN_CLOSURE`).

## Layout

```
bench/reason-encodings/
├── vlog/rdfs.dlog     RDFS 6-rule program (VLog Datalog)
├── vlog/owl-rl.dlog   OWL-RL program: RDFS-6 + the OWL rules the LUBM TBox exercises
├── nemo/rdfs.rls      RDFS 6-rule program (Nemo .rls)
└── nemo/owl-rl.rls    OWL-RL program (Nemo .rls; @@DATA@@/@@OUT@@ placeholders)
```

Each program closes the input into a **single ternary predicate** (`T` for VLog,
`closed` for Nemo) holding the *whole* materialized graph (base + entailed), so its
cardinality is directly comparable to sparq's self-reported closure count. The adapters
[`scripts/bench-adapters/vlog_adapter.py`](../../scripts/bench-adapters/vlog_adapter.py)
and [`nemo_adapter.py`](../../scripts/bench-adapters/nemo_adapter.py) wire these as the
default rules file per profile.

## Rule set (why these rules, and not others)

The LUBM Univ-Bench TBox uses only `subClassOf/domain/range/subPropertyOf`,
`owl:inverseOf` (3 pairs), `owl:TransitiveProperty` (`subOrganizationOf`),
`owl:someValuesFrom` restrictions (8), and `owl:intersectionOf` (6). It has **no**
`owl:sameAs`, functional/cardinality, `unionOf`, `oneOf`, `hasValue`, `propertyChain`,
or `hasKey`. So each program is `rdfs2/3/5/7/9/11` **plus** exactly `prp-inv`, `prp-trp`,
`cls-svf1`, `cls-int1`, `scm-int`, `scm-dom1/rng1` (+ the subPropertyOf and inverseOf
domain/range propagations), and `scm-svf2` — the rules sparq derives on this corpus, and
**no reflexive `scm-*`/`eq-*` or axiomatic `owl:Thing` triples** (sparq emits none on
LUBM, so any would over-count). The attribution is empirical: an independent semi-naive
fixpoint of exactly these rules is set-identical to sparq's closure.

## Running / reproducing the validation

The engines are gather-only deps (**not** committed, per AGENTS.md). Build from source:

```sh
# VLog (github.com/karmaresearch/vlog, Apache-2.0)
git clone https://github.com/karmaresearch/vlog && cd vlog && mkdir build && cd build
cmake -DCMAKE_CXX_FLAGS="-include cstdint" .. && make -j   # cstdint flag: GCC-13 workaround
# Nemo (github.com/knowsys/nemo, Apache-2.0)
git clone https://github.com/knowsys/nemo && cd nemo && cargo build -r -p nemo-cli

# then, from the repo root:
VLOG=/path/to/vlog/build/vlog NEMO=/path/to/nemo/target/release/nmo \
  LUBM_UNIVS=1 MAT_ITERS=1 scripts/bench/materialize-same-box.sh
```

The harness asserts each engine's closure against the pinned count before recording any
timing. Work-box timings are **non-canonical** (directional only); the canonical
`univ>=100` run belongs to `sq-hmd7l.32` on a dedicated quiet EC2 box.

## Nemo performance note

Nemo's semi-naive evaluation does not converge in reasonable time if `cls-svf1` /
`cls-int1` inline the restriction/intersection schema atoms into the recursive `closed`
rule (a wide self-join over the growing type relation re-fires each round). The `.rls`
encoding therefore folds that schema into small `svfR` / `int{1,2}def` relations first;
the result is logically identical and converges in a couple of seconds. VLog evaluates
the inlined form directly. Both agree with sparq set-for-set.

## License

Apache-2.0 (the workspace license). The referenced engines (VLog, Nemo) are Apache-2.0.
