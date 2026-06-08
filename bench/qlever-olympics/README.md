# sparq vs QLever — Olympics benchmark

A reproducible comparison of sparq against [QLever](https://github.com/ad-freiburg/qlever)
on the standard **Olympics** dataset (120 years of Olympics, ~1.78M triples).

Only the harness is tracked; the data (~323 MB) and QLever index are generated.

## Run

```sh
# one-time setup (in this directory)
python3 -m venv ../../.qlever-venv && ../../.qlever-venv/bin/pip install qlever
../../.qlever-venv/bin/qlever setup-config olympics   # writes Qleverfile (tracked)
../../.qlever-venv/bin/qlever get-data                # downloads olympics.nt
../../.qlever-venv/bin/qlever index                   # builds QLever index (Docker)
../../.qlever-venv/bin/qlever start                   # serves on :7019

cargo build -p sparq-cli --release                    # (from repo root)

# compare:  compare.py <iters> <queries-dir> <sparq-mode>
../../.qlever-venv/bin/python compare.py 5 queries        json         # fair end-to-end
../../.qlever-venv/bin/python compare.py 5 queries-count  materialize  # fair compute-only
```

## What it measures

- `queries/` — 10 SELECT queries (scan, star joins, chains, filter, aggregation,
  OPTIONAL). The **end-to-end** pass: both engines compute *and serialise the full
  result to SPARQL JSON*. Result sizes are the correctness cross-check.
- `queries-count/` — the same patterns wrapped in `SELECT (COUNT(*) ...)`. The
  **compute-only** pass: both compute the join and return one row, isolating
  engine compute from serialisation.

Timing is min-of-N **cold** runs (QLever cache cleared each run; sparq has no
query cache). QLever's time is its own `query-time-ms`; sparq's is in-process.

See `research/BENCHMARKS.md` for results and the (important) caveats: QLever runs
in Docker-on-macOS here and the dataset fits in RAM — both favour sparq — yet
**QLever's engine is ~3× faster on compute**, which is what M3/M4 target.
