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
