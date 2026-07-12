<!-- [GPT-5.6] PR #2136: sparq-shaclc parse-throughput benchmark. -->

# SHACL Compact Syntax parse throughput

This benchmark generates a deterministic shapes corpus, parses it through the public
`sparq_shaclc::parse` API in strict and extended modes, and serialises the strict triples through
the residual-consumption `write` API. A parse or write failure stops the run before that operation
can report timing.

Run the release benchmark from the repository root:

```sh
bench/shaclc/run.sh
```

Use `--smoke` for a small harness check. `SHACLC_SHAPES` controls the generated corpus size; pin it
when comparing runs. The in-crate driver reports the minimum elapsed time across its repetitions.

Output includes the corpus shape/byte counts plus strict parse, extended parse, and strict write
`metric_us` rows. Elapsed times are host-sensitive, non-canonical runtime measurements; do not
commit observed values as performance claims.
