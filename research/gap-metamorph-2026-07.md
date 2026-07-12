<!-- [GPT-5.6] sq-hgqza — self-relative metamorphic-harness throughput record. -->

# Gap record — SPARQL metamorphic bug-harness throughput (2026-07)

## Scope and comparison verdict

The `sparq-metamorph` bench measures deterministic case generation followed by both TLP
and NoREC verdict checks over a fixed seed window. It reports cases per second through
Criterion and is self-relative only: there is no directly runnable open-source
RDF/SPARQL peer exposing the same SQLancer-family logic-bug harness. Its output is a
regression signal, not evidence of a competitive performance win.

No measured result belongs in this tracked record. Local Criterion output is
environment-specific and should be compared only with a baseline captured under the
same conditions.

## Correctness envelope

Before reporting throughput, the bench checks that regenerating each seed produces the
identical case and that every case produces exactly one counted TLP verdict and one
counted NoREC verdict. Classification is exhaustive across pass, wrong-result violation,
and engine failure, so no outcome can be silently discarded. A load failure aborts the
run rather than shrinking the measured corpus.

Run the smoke form with:

```sh
cargo bench -p sparq-metamorph --features protocol-drivers \
  --bench metamorph_throughput -- --test
```
