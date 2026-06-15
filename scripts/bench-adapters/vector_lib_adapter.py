#!/usr/bin/env python3
# [OPUS-4.8] sq-eifd: PYTHON-LIB (vector-lib) adapter KIND. Authored by Opus 4.8
# (Fable unavailable; flag for re-review when Fable returns).
#
# In-process, cross-language ANN harness — the vector analogue of the embedded-Rust
# Oxigraph model, but Python. Covers FAISS / hnswlib / Lucene-via-Anserini
# (research/capability-benchmark-program.md §3.3 + G3 `python-lib`). The deliverable
# shape: build index -> query -> emit recall@k + latency TSV + an EXACT-kNN
# correctness oracle (G4 — ANN is approximate, so the gate is recall vs exact-kNN,
# not a row-count diff).
#
# TWO pieces, separable so the gather box can run the heavy half and CI can unit-
# test the light half:
#   1. recall_at_k(approx_ids, exact_ids, k)  — the SCORING/ORACLE. Pure function:
#      mean over queries of |approx_topk ∩ exact_topk| / k. This is fixture-unit-
#      tested (a captured neighbour TSV) WITHOUT FAISS/hnswlib installed.
#   2. exact_knn / run_hnswlib — the heavy harness: build the index, query, compute
#      the exact-kNN ground truth with numpy (the oracle), emit recall@k + latency.
#      Requires numpy (+ hnswlib for the ANN engine); gather-only.
#
# Output: `<engine>\t<recall_deficit_milli>\t<query_us>` TSV on stdout, where
# recall_deficit_milli = round((1 - recall@k) * 1000) — emitted as a DEFICIT so it
# slots into the smaller-is-better mode:"auto" ratchet with zero perf-gate change
# (the G4 trick). --json carries the raw recall + k for the dashboard.
#
# NEIGHBOUR-TSV format (what an external ANN tool dumps, and the fixture format):
#   one line per query: `<query_id>\t<id1>,<id2>,...,<idk>`  (ranked neighbour ids).
# recall_at_k consumes two such maps (approx + exact) and scores them.
import json
import sys
import time


def parse_neighbour_tsv(text):
    """`<qid>\\t<csv of ranked neighbour ids>` per line -> {qid: [ids...]}.

    Blank lines and `#`-comments are skipped. Ids are kept as strings (ids may be
    dict-encoded entity ids in sparq's case)."""
    out = {}
    for line in text.splitlines():
        line = line.strip()
        if not line or line.startswith("#"):
            continue
        parts = line.split("\t")
        if len(parts) != 2:
            raise ValueError("bad neighbour-tsv line: %r" % line)
        qid, ids = parts
        out[qid] = [t for t in ids.split(",") if t != ""]
    return out


def recall_at_k(approx, exact, k):
    """Mean recall@k of `approx` vs the `exact` ground truth (both {qid:[ids]}).

    recall@k for a query = |approx[:k] ∩ exact[:k]| / k, averaged over the queries
    present in BOTH maps. Returns (recall: float, n_queries: int). Raises if there
    is no overlapping query (so a mis-aligned fixture fails loudly, not silently
    as recall 0)."""
    qids = [q for q in exact if q in approx]
    if not qids:
        raise ValueError("no query ids common to approx and exact neighbour sets")
    total = 0.0
    for q in qids:
        a = set(approx[q][:k])
        e = set(exact[q][:k])
        denom = min(k, len(e)) or 1
        total += len(a & e) / denom
    return total / len(qids), len(qids)


def recall_deficit_milli(recall):
    """(1 - recall) scaled to integer milli-units — the smaller-is-better metric
    the mode:"auto" ratchet wants (G4)."""
    return int(round((1.0 - recall) * 1000))


# --- heavy harness (gather-only; needs numpy [+ hnswlib]) --------------------
def exact_knn(data, queries, k):
    """numpy exact L2 kNN ground truth -> {qid:[ids]} with qid/ids as str(index)."""
    import numpy as np  # local import: oracle is gather-only

    data = np.asarray(data, dtype="float32")
    queries = np.asarray(queries, dtype="float32")
    out = {}
    for qi, q in enumerate(queries):
        d = ((data - q) ** 2).sum(axis=1)
        idx = np.argsort(d)[:k]
        out[str(qi)] = [str(int(i)) for i in idx]
    return out


def run_hnswlib(data, queries, k, ef=64, m=16, ef_construction=200):
    """Build an hnswlib index, query top-k, time it -> ({qid:[ids]}, query_us)."""
    import hnswlib  # local import: gather-only
    import numpy as np

    data = np.asarray(data, dtype="float32")
    queries = np.asarray(queries, dtype="float32")
    dim = data.shape[1]
    index = hnswlib.Index(space="l2", dim=dim)
    index.init_index(max_elements=len(data), ef_construction=ef_construction, M=m)
    index.add_items(data, list(range(len(data))))
    index.set_ef(ef)
    t0 = time.perf_counter()
    labels, _ = index.knn_query(queries, k=k)
    query_us = (time.perf_counter() - t0) * 1e6 / max(1, len(queries))
    approx = {str(qi): [str(int(i)) for i in row] for qi, row in enumerate(labels)}
    return approx, int(round(query_us))


def main(argv):
    """CLI:
      vector_lib_adapter.py --score --approx <tsv> --exact <tsv> --k K [--engine N]
            offline: score two neighbour TSVs (fixture-testable, no numpy/hnswlib).
      vector_lib_adapter.py --hnswlib --npz <file.npz> --k K [--engine N]
            heavy: build hnswlib index over npz {data,queries}, score vs exact-kNN.
    Emits `<engine>\\t<recall_deficit_milli>\\t<query_us>`; --json adds raw recall."""
    mode = None
    approx_f = exact_f = npz_f = None
    k = 10
    engine = "engine"
    want_json = False
    query_us = 0
    i = 0
    while i < len(argv):
        a = argv[i]
        if a == "--score":
            mode = "score"
        elif a == "--hnswlib":
            mode = "hnswlib"
        elif a == "--approx":
            i += 1
            approx_f = argv[i]
        elif a == "--exact":
            i += 1
            exact_f = argv[i]
        elif a == "--npz":
            i += 1
            npz_f = argv[i]
        elif a == "--k":
            i += 1
            k = int(argv[i])
        elif a == "--engine":
            i += 1
            engine = argv[i]
        elif a == "--json":
            want_json = True
        else:
            sys.stderr.write("vector_lib_adapter: unknown arg %s\n" % a)
            return 2
        i += 1

    try:
        if mode == "score":
            if not (approx_f and exact_f):
                raise ValueError("--score needs --approx <tsv> --exact <tsv>")
            with open(approx_f, encoding="utf-8") as fh:
                approx = parse_neighbour_tsv(fh.read())
            with open(exact_f, encoding="utf-8") as fh:
                exact = parse_neighbour_tsv(fh.read())
            recall, _n = recall_at_k(approx, exact, k)
        elif mode == "hnswlib":
            import numpy as np

            if not npz_f:
                raise ValueError("--hnswlib needs --npz <file.npz {data,queries}>")
            npz = np.load(npz_f)
            approx, query_us = run_hnswlib(npz["data"], npz["queries"], k)
            exact = exact_knn(npz["data"], npz["queries"], k)
            recall, _n = recall_at_k(approx, exact, k)
        else:
            sys.stderr.write("vector_lib_adapter: pick --score or --hnswlib\n")
            return 2
    except Exception as e:  # noqa: BLE001 — adapter boundary
        sys.stderr.write("vector_lib_adapter: %s\n" % e)
        return 1

    sys.stdout.write("%s\t%d\t%d\n" % (engine, recall_deficit_milli(recall), query_us))
    if want_json:
        sys.stderr.write(
            json.dumps({"engine": engine, "recall_at_k": recall, "k": k, "query_us": query_us})
            + "\n"
        )
    return 0


if __name__ == "__main__":
    sys.exit(main(sys.argv[1:]))
