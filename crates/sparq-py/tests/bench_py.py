# [OPUS-4.8] sq-th1u: pytest-timed FFI-overhead micro-bench for the `sparq`
# Python bindings.
#
# WHAT this measures and WHY. The Python API is a thin pyo3 wrapper: the cost it
# adds over the Rust engine is (1) the GIL release/re-acquire around each call,
# (2) argument marshalling (str -> &str), and (3) per-ROW / per-CELL result
# materialisation (building Python dicts of `Term` objects in `select_result`).
# Item (3) is the regression we most want to catch: an accidental per-row copy or
# an extra clone in the binding layer would inflate the per-row cost without
# changing any Rust-engine benchmark. This is a TIMED, PRINTING test (min /
# median microseconds over N iterations); it is NOT a hard wall-clock threshold
# (this box is frequently busy — see bench/CATALOG.md "QUIET-BOX requirement"),
# except for one structural, load-ROBUST invariant noted below.
#
# Marked `@pytest.mark.bench` so CI can select (`pytest -m bench`) or deselect
# (`pytest -m "not bench"`) it; the marker is registered in pyproject.toml.
#
# Run:
#     pytest crates/sparq-py/tests/bench_py.py -m bench -s   # -s shows the prints

import statistics
import time

import pytest

import sparq

# A small graph: a handful of subjects, each with a few predicates. Small on
# purpose — we are measuring the BOUNDARY cost (load + query + materialise),
# which is dominated by fixed per-call overhead and per-row work, not by engine
# scan time over a big store.
SMALL_TTL = """
@prefix ex: <http://ex/> .
ex:a ex:p ex:b ; ex:age 1 ; ex:name "a" .
ex:c ex:p ex:d ; ex:age 2 ; ex:name "b" .
ex:e ex:p ex:f ; ex:age 3 ; ex:name "c" .
ex:g ex:p ex:h ; ex:age 4 ; ex:name "d" .
ex:i ex:p ex:j ; ex:age 5 ; ex:name "e" .
"""

SELECT_ALL = "SELECT ?s ?p ?o WHERE { ?s ?p ?o }"

# Iteration counts: enough to get a stable min/median without making the suite
# slow. A debug (`maturin develop`) build is ~10x slower than the wheel, so keep
# N modest; the MIN is the figure of merit (least perturbed by scheduler noise).
WARMUP = 50
ITERS = 500


def _time_us(fn, iters=ITERS, warmup=WARMUP):
    """Return (min_us, median_us) for `fn` over `iters` calls (after `warmup`)."""
    for _ in range(warmup):
        fn()
    samples = []
    for _ in range(iters):
        t0 = time.perf_counter()
        fn()
        samples.append((time.perf_counter() - t0) * 1e6)
    return min(samples), statistics.median(samples)


@pytest.mark.bench
def test_ffi_overhead_load(capsys):
    """Time `Graph.load` of a small Turtle document (parse + dictionary build +
    index build, all behind one FFI call)."""
    g_min, g_med = _time_us(lambda: sparq.Graph.load(SMALL_TTL))
    with capsys.disabled():
        print(f"\n[bench] Graph.load(small ttl, 15 triples): min={g_min:.1f}us median={g_med:.1f}us")
    # The graph really did load (sanity, not a perf assertion).
    assert len(sparq.Graph.load(SMALL_TTL)) == 15


@pytest.mark.bench
def test_ffi_overhead_query_materialise(capsys):
    """Time `Graph.query` (full per-cell `Term` materialisation) over the small
    graph — the path most exposed to a per-row binding-layer regression."""
    g = sparq.Graph.load(SMALL_TTL)
    assert len(g.query(SELECT_ALL)) == 15  # 15 triples -> 15 rows

    q_min, q_med = _time_us(lambda: g.query(SELECT_ALL))
    per_row_min = q_min / 15.0
    with capsys.disabled():
        print(
            f"[bench] Graph.query(15 rows, Term path): min={q_min:.1f}us median={q_med:.1f}us "
            f"({per_row_min:.2f}us/row)"
        )


@pytest.mark.bench
def test_ffi_overhead_query_json(capsys):
    """Time `Graph.query_json` (id-level fast path, no per-cell Term objects) for
    comparison against the Term path — the gap is the per-cell materialisation
    cost the binding adds."""
    g = sparq.Graph.load(SMALL_TTL)
    j_min, j_med = _time_us(lambda: g.query_json(SELECT_ALL))
    with capsys.disabled():
        print(f"[bench] Graph.query_json(15 rows, id path): min={j_min:.1f}us median={j_med:.1f}us")


@pytest.mark.bench
def test_ffi_overhead_load_plus_query(capsys):
    """End-to-end boundary cost: one load + one query, the smallest realistic
    'use the library' unit. This is the headline FFI-overhead number."""
    def load_and_query():
        g = sparq.Graph.load(SMALL_TTL)
        return g.query(SELECT_ALL)

    e_min, e_med = _time_us(load_and_query)
    with capsys.disabled():
        print(f"[bench] load + query (end-to-end): min={e_min:.1f}us median={e_med:.1f}us")


@pytest.mark.bench
def test_query_cost_scales_sublinearly_per_row(capsys):
    """Structural, LOAD-ROBUST regression guard (a ratio, not an absolute time).

    The per-row materialisation cost must not EXPLODE with row count — an
    accidental O(n^2) per-row copy (e.g. re-cloning the whole result on each row)
    would make the per-row time grow with N. We compare per-row cost on a small
    vs a 10x-larger result from the SAME graph and assert the per-row time does
    not blow up (generous 4x bound so scheduler noise on a busy box can't flake
    it; a real O(n^2) bug would be orders of magnitude over). Ratios survive
    contention where absolute wall-clock does not (bench/CATALOG.md)."""
    # Build a graph with many rows so a 10x slice is meaningful.
    lines = ["@prefix ex: <http://ex/> ."]
    for i in range(1000):
        lines.append(f"ex:s{i} ex:p ex:o{i} .")
    g = sparq.Graph.load("\n".join(lines))

    q_small = "SELECT ?s ?o WHERE { ?s ?p ?o } LIMIT 50"
    q_large = "SELECT ?s ?o WHERE { ?s ?p ?o } LIMIT 500"
    assert len(g.query(q_small)) == 50 and len(g.query(q_large)) == 500

    small_min, _ = _time_us(lambda: g.query(q_small), iters=200, warmup=20)
    large_min, _ = _time_us(lambda: g.query(q_large), iters=200, warmup=20)
    per_row_small = small_min / 50.0
    per_row_large = large_min / 500.0
    ratio = per_row_large / per_row_small
    with capsys.disabled():
        print(
            f"[bench] per-row materialise: 50-row={per_row_small:.3f}us/row, "
            f"500-row={per_row_large:.3f}us/row, ratio={ratio:.2f} (want ~1, fail if >4)"
        )
    # Linear per-row cost => ratio ~1. Allow slack for fixed per-call overhead
    # amortising DIFFERENTLY across the two sizes plus noise; a quadratic-copy
    # regression would push this far past the bound.
    assert ratio < 4.0, (
        f"per-row materialisation cost grew {ratio:.1f}x from 50 to 500 rows — "
        "possible O(n^2) per-row copy in the binding layer"
    )
