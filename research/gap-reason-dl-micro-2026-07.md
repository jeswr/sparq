# Direct-semantics tableau microbenchmark notes

<!-- [GPT-5.6] sq-7ru8y -->

`sparq-reason-dl` now has a crate-local Criterion benchmark over pinned synthetic ALCH
concept expressions. It compares the checker only with itself across increasing quantifier
nesting depths; it has no competitor results and writes no measured values to tracked files.

The fixture families cover satisfiable and contradictory existential chains plus positive
and negative subsumption reductions. Before timing, the benchmark checks every result against
its pinned expected Boolean. This is an answer-stability oracle, not an independent soundness
argument; the reasoner's existing conformance tests remain responsible for that evidence.

Run the quick stability lane with:

```sh
cargo bench -p sparq-reason-dl --bench tableau_micro -- --test
```

For local depth-scaling exploration, omit `--test` and compare Criterion's per-depth output.
Criterion stores measurements below `target/criterion/`; do not copy machine-specific timing
values into this note.
