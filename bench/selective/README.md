# Selective-join benchmark

Exercises the **index-nested-loop (bind) join**: a dense `follows` graph + a rare
`premium` leaf. `queries/chain_premium.rq` returns ~7.6k rows but the `follows·follows`
side is ~millions. The harness compares the merge-join plan (full-relation scan) against
the bind join (index lookup per join value) in count + materialize modes; the bind join
is much faster, with no regression on the non-selective synthetic (the planner uses bind
join only when the running result is ≥8× smaller than the next pattern). Run it for the
numbers:

```sh
python3 gen.py > selective.nt
sparq-cli bench selective.nt ntriples queries 3 count
```

## All selective shapes handled (vs Oxigraph, 500k nodes)

The harness runs three selective shapes (chain, reverse chain, star) and cross-checks
each row count against Oxigraph — all match:

| query | rows | matches Oxigraph |
|---|--:|:--:|
| chain `a→b→c, c premium`     | 7623 | ✓ |
| reverse chain `c premium, b→c, a→b` | 7623 | ✓ |
| star `s premium, s→o, o→t`   | 8000 | ✓ |

The bind join propagates selectivity from the small `premium` seed through each join
(chain / reverse / star) — the planner picks the bind join independent of which end the
selectivity is on. All much faster than a full-relation scan, all correct.

## Two-sided selectivity (gen2.py + queries2/) — predicate-transfer characterization

`gen2.py [N] [fanout]` adds a SECOND rare predicate so the query
`?a premA ?x . ?a follows ?b . ?b follows ?c . ?c premC ?y` is selective at BOTH ends
through a dense middle. Bind join seeds from one end and must expand `a→b→c` before the far
end prunes, so the intermediate grows as |seed|·fanout². The harness sweeps fanout
(count mode), reporting count time + intermediate memory and cross-checking row counts
against Oxigraph (all match); run it for the numbers. The cost is density-conditional
(negligible when sparse, real at high fanout). This is the
validated workload for **predicate transfer / bitmap semi-join** — see
`research/predicate-transfer-measured.md` for the full analysis and the recommended
bidirectional-bind-join implementation.
