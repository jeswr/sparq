<!-- [FABLE-5] sq-on7r4. 🤖 SPARQ agent — written by Claude Fable 5. -->
# bench/trust-graph — certification-edge closure overhead + additivity envelope

Runs the **certification-edge trust-graph closure** benchmark: it times the shipped
`derive_effective_rules` closure (`crates/sparq-trust/src/graph.rs`, the default-OFF
`cert-graph` feature, bead `sq-pfae.15`) and — the load-bearing lane — **measures the
strict-additivity envelope**: zero (surviving) certifications must yield a derived rule
set **byte-EQUAL** to the input `direct_rules`. The 2026-07 trust-expression record's
cost/decidability concern (`sq-pfae.9`) previously had no runnable harness; this is it.

Standalone cargo project (own `[workspace]` table, same isolation pattern as `bench/ac`
+ `bench/parse` + `bench/dict`): the `trust-graph-bench` driver path-depends on
`sparq-trust` with `cert-graph` ON — in this detached project only, so the root
workspace's default build stays byte-identical (feature-OFF safety is structural).
The driver's only direct dependencies are `sparq-trust` + `sparq-zk` (fixture
signing); `sparq-engine` does enter the *link* transitively via `sparq-shacl`'s
SHACL-SPARQL path (the same linkage every production `cert-graph` consumer carries),
but the driver never invokes engine code, and the closure under test is a pure
function over in-process fixtures — so the measurement cannot perturb the system
under test. `Cargo.lock` is committed for reproducible `--locked` builds; `target/`
is gitignored.

## Run

```sh
# per-commit smoke tier (fixed sizes; the CI lane) — builds the driver, runs the lanes:
bench/trust-graph/run.sh --smoke

# a larger deterministic tier (nightly / EC2):
bench/trust-graph/run.sh --sf 10
```

The driver streams a TSV table
(`lane<TAB>kind<TAB>anchors<TAB>certs<TAB>derived<TAB>expected<TAB>min_us<TAB>per_edge_ns<TAB>status`)
and is **fail-closed**: any envelope or expected-count mismatch makes it exit non-zero,
and `run.sh` propagates that exit. Registered as `trust-graph-closure` in
`bench/benchmarks.toml`.

## Lanes

| lane | what it checks / measures |
| --- | --- |
| **additivity-zero-certs** | *(load-bearing)* zero certifications ⇒ output byte-equal to `direct_rules` (canonical render of every `TrustRule` field compared as bytes) |
| **additivity-depth0** | `depth_bound = 0` with edges present ⇒ output byte-equal to `direct_rules` |
| **additivity-all-rejected** | edges present but ALL rejected (forged signatures) ⇒ zero survivors ⇒ byte-equal output |
| **closure-overhead** | `derive_effective_rules` min-of-K wall time over (anchors × certifications × scope kind: `any` / `narrow` / `broaden` / `forged`), with a per-edge amortised column; every cell is correctness-gated (expected admitted/rejected count + verbatim anchors prefix) before it is timed |

## Honesty

Measurement-only, **clear-path**: fixtures are plaintext in-process structs; no ZK proof
is produced or verified and **no privacy or cryptographic-soundness claim is made** (the
ZK estate the closure shares a signature primitive with is externally unaudited — open
gate `sq-qhy4`). Every wall-clock line is **advisory + NON-CANONICAL** on a shared work
box (the QUIET-BOX convention in `bench/CATALOG.md`); no number produced here is
committed to markdown — results go to the console (or a git-ignored scratch file if you
redirect them). The load-robust contract is the deterministic fail-closed envelope exit
code.
