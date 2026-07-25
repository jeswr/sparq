<!-- [GPT-5.6] sq-xngad — standing builtin-cost micro-benchmark. -->

# Builtin-cost micro-benchmark

This suite measures scalar builtins whose setup cost can accidentally become a per-row cost:
constant-pattern `REGEX` and `REPLACE`, `RAND`, and query-constant `NOW`. It generates a fixed
N-Triples corpus outside the checkout, runs the release `sparq-cli` query-suite driver in
materialize mode, checks the result cardinality of every probe, and prints a Markdown throughput
table. Materialization is intentional: it forces every RAND projection to execute.

```sh
cargo build --release -p sparq-cli
bench/builtins/run.sh
```

Useful knobs are `ROWS` (generated input rows), `ITERS` (minimum-of repetitions), `CLI` (binary
path), and `BUILTINS_CACHE` (generated-data directory). For a cheap harness check:

```sh
ROWS=100 ITERS=1 bench/builtins/run.sh
```

The lightweight harness self-test mutation-witnesses the cardinality and timing guards with a
controlled CLI double:

```sh
bench/builtins/test.sh
```

`now_query_constant.rq` materializes a NOW projection for every input row. It is a cost workload,
not a temporal-conformance assertion: the cardinality guard proves the projection was consumed,
but deliberately avoids inferring query constancy from clock-resolution-sensitive values.

Every elapsed-time and throughput value is a work-box measurement and **NON-CANONICAL**. The
canonical rerun belongs to sq-98w7z.9; in particular, it reruns the REGEX-heavy FILTER probe here.
The optimized `after` implementation is current main. There is no feature or environment toggle
for the old behavior, so a true `before` run must build the parent of commit `85a54650f` and invoke
this same script with `CLI` pointing at that binary. The memoization is unconditional.

The table's `rows/s` is derived from the CLI's minimum query time. It is a trend aid, not a
committed performance claim. Generated corpora and raw results remain under `BUILTINS_CACHE`
(default `/tmp/sparq-builtin-cost`) and are safe to delete after a run.
