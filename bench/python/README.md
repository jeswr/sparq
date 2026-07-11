<!-- [FABLE-5] sq-hmd7l.18 -->
# python-bindings-bench — sparq-py vs pyoxigraph vs rdflib

Measures the cost of driving an RDF engine **from Python**. Registry entry:
`python-bindings-bench` in [`bench/benchmarks.toml`](../benchmarks.toml); competitor
entries `pyoxigraph` / `rdflib` in [`bench/competitors.json`](../competitors.json).
First-read results record: `research/gap-python-binding-2026-07.md`.

Three columns, one honesty split:

| column | boundary | what its whole-call time means |
|---|---|---|
| **sparq-py** (`pip` name `sparq-rdf`, import `sparq`) | pyo3 FFI | Rust engine + binding overhead |
| **pyoxigraph** | pyo3 FFI | Rust (Oxigraph) engine + binding overhead |
| **rdflib** | none (pure Python) | engine-bound, NOT binding-bound — the ecosystem reference |

**Primary metric — binding overhead.** For sparq the split is measured directly:
the same queries on the same corpus run through `sparq-cli bench … materialize`
(engine-internal timing, matched `--profile python-release` codegen), and
`binding overhead = python whole-call − engine-internal`. pyoxigraph has no
same-process engine-only reference here, so cross-binding comparison uses two
isolation instruments instead (both in the adapter):

- **floor** — calls where engine work is ~nil on a fixed 8-triple graph
  (`len(g)`, a hit `ASK`, a 0-row SELECT, `LIMIT 1`, an 8-row SELECT): the
  per-call boundary cost dominates.
- **slope** — `SELECT ?s ?p ?o` over 64- vs 8192-triple synthetic graphs; the
  per-row slope ≈ result-materialisation cost per row (includes the engine's
  per-row scan — stated, not hidden).

**Row-count agreement gate.** No timing row is reported unless every engine
(and the CLI reference) returns the same solution count per query — the
adapter's `--compare` stage exits 1 on any disagreement.

## Run

```bash
pip install pyoxigraph rdflib                                  # gather-time only
(cd crates/sparq-py && maturin develop --profile python-release)
cargo build --profile python-release -p sparq-cli              # native reference

bash bench/python/run.sh --smoke    # SP2B tiny tier, 3 queries, agreement gate
bash bench/python/run.sh            # + floor & slope micro-benchmarks
```

Knobs (`PYBENCH_PYTHON`, `SP2B_T`, `ITERS`, `QUERIES`, `CLI`) are documented in
[`run.sh`](./run.sh). Engines absent from the interpreter are skipped gracefully.
Results land in `bench/competitor-results/` (git-ignored, regenerable). Corpus:
the deterministic SP2Bench generator ([`bench/sp2b/gen.sh`](../sp2b/gen.sh)),
tiny tier by default so rdflib stays tractable.

Wall-clock caveat: `quiet_box_sensitive` — absolute numbers from a busy box are
indicative only; ratios and the overhead delta are the robust read (see
`bench/CATALOG.md` conventions).
