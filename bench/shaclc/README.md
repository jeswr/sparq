<!-- [GPT-5.6] PR #2136: sparq-shaclc parse-throughput benchmark. -->

# SHACL Compact Syntax parse throughput

This benchmark parses the crate's committed positive fixture corpus through the public
`sparq_shaclc::parse` API. The strict row covers the standard and RDF 1.2 fixtures; the extended
row adds the shaclc-js extension fixtures. Every repetition must produce the same triple count
before the runner reports timing, so parser or fixture failures suppress measurement output.

Run the release benchmark from the repository root:

```sh
bench/shaclc/run.sh
```

Use `--smoke` for a single-pass harness check. `SHACLC_PASSES` controls how many complete corpus
passes make one sample, and `SHACLC_SAMPLES` controls the number of samples; the best elapsed time
is reported. Pin both values when comparing runs.

The TSV columns are profile, parsed documents, input bytes, emitted triples, best elapsed
microseconds, and MiB/s. Elapsed time and throughput are host-sensitive, non-canonical runtime
measurements; do not commit observed values as performance claims.
