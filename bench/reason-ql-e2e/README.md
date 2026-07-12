# OWL 2 QL end-to-end comparison

<!-- [GPT-5.6] sq-mg1wx -->

This gather-only harness compares complete query answering: sparq rewrites each CQ and then executes
the emitted UCQ over a materialized ABox; Ontop rewrites and executes against an equivalent mapped
store. The embedded fixtures use NPD/Requiem-class shapes, not redistributed upstream corpora.

Run the hermetic smoke gate:

```sh
bash bench/reason-ql-e2e/run.sh --smoke
```

For a quiet-box sparq measurement, omit `--smoke`. To add Ontop, provide a gathered TSV whose columns
are `case`, `answers`, and `end_to_end_ms`:

```sh
ONTOP_RESULTS=/path/to/ontop.tsv bash bench/reason-ql-e2e/run.sh
```

Both implementations must agree with `expected.tsv` before the script prints any timing row. The
sparq rewriter-phase and end-to-end columns are deliberately distinct. Durations are non-canonical
and must be regenerated on the intended measurement host.
