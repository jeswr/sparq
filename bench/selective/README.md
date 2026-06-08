# Selective-join benchmark

Exercises the **index-nested-loop (bind) join**: a dense `follows` graph + a rare
`premium` leaf. `queries/chain_premium.rq` returns ~7.6k rows but the `follows·follows`
side is ~millions. Measured (M1, 500k nodes / 2M triples):

| plan | count | materialize |
|---|--:|--:|
| merge join (scans the full relation) | 30.6 ms | 40.5 ms |
| **bind join** (index lookup per join value) | **0.86 ms** | **3.2 ms** |

~35× faster, with no regression on the non-selective synthetic (the planner uses bind
join only when the running result is >=8x smaller than the next pattern).

Run: `python3 gen.py > selective.nt` then
`sparq-cli bench selective.nt ntriples queries 3 count`.

## All selective shapes handled (vs Oxigraph, 500k nodes)

| query | rows | sparq count | matches Oxigraph |
|---|--:|--:|:--:|
| chain `a→b→c, c premium`     | 7623 | 0.87 ms | ✓ |
| reverse chain `c premium, b→c, a→b` | 7623 | 0.81 ms | ✓ |
| star `s premium, s→o, o→t`   | 8000 | 0.89 ms | ✓ |

The bind join propagates selectivity from the small `premium` seed through each join
(chain / reverse / star) — the planner picks the bind join independent of which end the
selectivity is on. All ~35× faster than a full-relation scan, all correct.
