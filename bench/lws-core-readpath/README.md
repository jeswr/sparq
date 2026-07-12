<!-- [GPT-5.6] -->
# lws-core read-path allocation harness

This standalone harness runs the crate's `read_response_alloc_microbench` example and captures
its deterministic allocation-operation counts in a machine-readable JSON envelope. The example
first asserts that the before and after header sets are byte-identical; the harness refuses to
emit a successful envelope unless both measurements and their delta are present and consistent.

Run the acceptance tier from anywhere in the repository:

```sh
bash bench/lws-core-readpath/run.sh --smoke
```

Generated envelopes land under `results/`, which is ignored. They identify the source revision,
toolchain, command, measurement kind, and raw example output. Counts remain observations produced
by the current build and are never baked into this documentation.
