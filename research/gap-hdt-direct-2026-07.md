# HDT direct-decoder self-relative A/B

<!-- [GPT-5.6] sq-wzzxg -->

This runner compares sparq-hdt's direct decoder with its upstream-backed oracle
using identical in-memory HDT bytes. It is an internal structural comparison,
not an external competitor result and not a source of headline performance
claims.

Run `bash bench/hdt/direct-vs-upstream.sh --smoke` for the smallest correctness
check, or omit `--smoke` for the normal synthetic workload. Standard output is
restricted to `<workload>\t<triples>\t<us>` rows so automation can collect it.
Diagnostics go to standard error.

The direct and upstream paths must agree before either timing is emitted. Any
decoder failure or triple-count divergence exits nonzero, making correctness a
hard gate. Timing values are advisory and non-canonical on a work box; canonical
comparisons require a quiet-box rerun.
