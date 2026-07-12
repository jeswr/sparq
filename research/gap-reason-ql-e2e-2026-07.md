# OWL 2 QL end-to-end answering: first-read gap record

<!-- [GPT-5.6] sq-mg1wx -->

**Bead:** `sq-mg1wx`  
**Status:** non-canonical, gather-only comparison

## Fixed-vocabulary verdict

**PARTIAL — local correctness gate built; external Ontop timings not gathered.** sparq now has a
hermetic rewrite-then-execute harness over NPD/Requiem-class query shapes. It self-asserts per-query
answer-set sizes before exposing durations. Ontop remains an external gather column and no comparative
performance verdict is claimed until equivalent mappings, data, and a pinned Ontop build are run.

## Measurement boundary

The result table labels two different quantities:

- **rewriter phase**: only `rewrite_production`;
- **end to end**: query parse, rewrite, and UCQ execution over the materialized ABox.

Ontop's column is end-to-end only. It must not be compared with sparq's rewriter-phase column. The
deterministic evidence is answer-set-size agreement with `bench/reason-ql-e2e/expected.tsv`; elapsed
times are non-canonical observations from the gathering host.

## Fixture coverage and limits

The hermetic corpus covers shallow class and role hierarchies, inverse roles, and a conjunctive join.
These are representative NPD/Requiem-class shapes, not a claim of complete coverage of either corpus.
The gate compares answer-set sizes; the `DISTINCT` queries make that an answer-set cardinality oracle.
