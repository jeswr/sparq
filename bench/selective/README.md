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

## Two-sided selectivity (gen2.py + queries2/) — predicate-transfer characterization

`gen2.py [N] [fanout]` adds a SECOND rare predicate so the query
`?a premA ?x . ?a follows ?b . ?b follows ?c . ?c premC ?y` is selective at BOTH ends
through a dense middle. Bind join seeds from one end and must expand `a→b→c` before the far
end prunes, so the intermediate grows as |seed|·fanout². Measured (count mode, M1):

| fanout | edges | rows | count time | intermediate mem (peak−load) | vs Oxigraph |
|---:|---:|---:|---:|---:|:--:|
| 4   | 2.0M  | 8    | 0.89 ms | ~0 MB   | ✓ |
| 50  | 5.0M  | 274  | 7.46 ms | ~14 MB  | ✓ (274=274) |
| 200 | 10.0M | 1985 | 48.7 ms | ~208 MB | ✓ |

The cost is density-conditional (negligible sparse, real at fanout 200). This is the
validated workload for **predicate transfer / bitmap semi-join** — see
`research/predicate-transfer-measured.md` for the full analysis and the recommended
bidirectional-bind-join implementation.
