#!/usr/bin/env python3
# [OPUS-4.8] sq-6te5: capture harness for the hnswlib reference fixture used by the
# sparq-vectors end-to-end gather verification (tests/ref_lib_verify.rs). Authored by
# Opus 4.8 (Fable unavailable; flag for re-review when Fable returns).
#
# WHY this exists ------------------------------------------------------------
# tests/diskann.rs / tests/recall.rs already prove sparq-vectors' approximate index
# (DiskANN/Vamana, and HNSW under `approx-ann`) recovers its OWN exact brute-force
# ground truth at a recall floor. What they do NOT do is anchor that floor against an
# ESTABLISHED ANN library. sq-6te5 closes that gap: prove sparq-vectors' search is
# consistent with hnswlib (the reference FAISS/hnswlib adapter from sq-eifd) on a fixed
# fixture — recall/correctness equivalence, so the sparq adapter is verified against the
# established library, not only against itself.
#
# HEAVY vs COMMITTED ----------------------------------------------------------
# numpy + hnswlib are heavy, native, and NOT available in CI (the same reason the
# sq-eifd vector-lib adapter's --hnswlib path is gather-only). So this harness is
# GATHER-ONLY: run it once on a box with numpy + hnswlib installed, commit its output as
# tests/fixtures/hnswlib_ref.tsv, and from then on the Rust test verifies sparq-vectors
# against that COMMITTED real-hnswlib capture deterministically in CI — no native deps.
#
# DETERMINISM is the load-bearing trick --------------------------------------
# The fixture stores only the captured NEIGHBOUR IDS, never the vectors. Both this
# harness and the Rust test regenerate the SAME corpus + queries from the same splitmix64
# seed (the raw u64 stream is bit-identical across Python and Rust — verified). So the
# Rust test mmaps an identical VectorStore and can compute its OWN exact-kNN ground truth
# and compare it to numpy's (the metric-agreement anchor) before scoring recall.
#
# RUN (gather box):
#   python3 -m venv /tmp/v && /tmp/v/bin/pip install numpy hnswlib
#   /tmp/v/bin/python crates/sparq-vectors/scripts/capture_hnswlib_ref.py \
#       > crates/sparq-vectors/tests/fixtures/hnswlib_ref.tsv
#
# The default parameters below MUST stay in lockstep with the consts in
# tests/ref_lib_verify.rs — the Rust test re-asserts them off the fixture header and
# fails closed on any drift, so a regenerated fixture with different params cannot
# silently pass.
import sys

# --- fixture parameters (keep in lockstep with tests/ref_lib_verify.rs) -------
SEED = 0xC0FFEE          # corpus splitmix64 seed (matches tests/recall.rs rand_vec)
QUERY_SEED = 0xDECAF     # query splitmix64 seed
N = 5000                 # corpus size
DIM = 32                 # vector dimension
K = 10                   # neighbours per query
QUERIES = 50             # number of queries
SPACE = "cosine"         # hnswlib metric — MUST match sparq-vectors' cosine searcher
# hnswlib build/search knobs deliberately tuned so hnswlib's OWN recall is ~0.95, NOT
# 1.0: a perfect-recall fixture would make "sparq is as good as hnswlib" vacuous. With
# these knobs the cross-library equivalence claim is substantive — sparq-vectors' index
# has to clear a real, established-library recall floor (work-box NON-CANONICAL: ~0.946
# at capture time; the Rust test recomputes it off the ids and never trusts a baked
# number).
M = 12
EF_CONSTRUCTION = 100
EF_SEARCH = 48
# hnswlib's graph build uses a random level generator + multi-threaded inserts, so a
# default capture is NON-deterministic run-to-run. We pin a build seed AND force a single
# insert/query thread so the fixture is byte-for-byte reproducible (the live-comparison
# test re-captures and asserts byte-equality). This does NOT make hnswlib's recall
# canonical — it makes the COMMITTED reference stable.
HNSW_SEED = 100

MASK = (1 << 64) - 1


def splitmix64(state):
    """One step of splitmix64. Returns (new_state, output). Bit-identical to the Rust
    `splitmix64` in tests/recall.rs / tests/ref_lib_verify.rs."""
    state = (state + 0x9E3779B97F4A7C15) & MASK
    z = state
    z = ((z ^ (z >> 30)) * 0xBF58476D1CE4E5B9) & MASK
    z = ((z ^ (z >> 27)) * 0x94D049BB133111EB) & MASK
    z = z ^ (z >> 31)
    return state, z


def rand_vec(state, dim):
    """`dim` f32-valued samples from the splitmix64 stream, mirroring the Rust
    `rand_vec`: ((z >> 40) as f32 / (1<<23) as f32) * 2 - 1, cast to float32 last so the
    stored bytes match Rust's f32 store exactly."""
    import numpy as np

    out = []
    for _ in range(dim):
        state, z = splitmix64(state)
        # float32 cast mirrors `(z >> 40) as f32` in Rust; the arithmetic then matches.
        v = (np.float32(z >> 40) / np.float32(1 << 23)) * np.float32(2.0) - np.float32(1.0)
        out.append(v)
    return state, np.asarray(out, dtype="float32")


def gen_matrix(seed, count, dim):
    import numpy as np

    state = seed
    rows = []
    for _ in range(count):
        state, v = rand_vec(state, dim)
        rows.append(v)
    return np.asarray(rows, dtype="float32")


def corpus_id(i):
    """Sparse, non-contiguous dict-style ids (matches tests/recall.rs: i*7 + 3) so the
    fixture exercises the id->slot mapping, not just 0..N."""
    return i * 7 + 3


def cosine_exact_knn(data, queries, k):
    """numpy exact top-k by COSINE similarity (not L2) -> {qid: [corpus_id...]}.

    Cosine because that is the metric sparq-vectors' `nearest_exact` uses; ties break on
    ascending corpus id to match the Rust searcher's deterministic tie-break."""
    import numpy as np

    dn = data / np.linalg.norm(data, axis=1, keepdims=True)
    qn = queries / np.linalg.norm(queries, axis=1, keepdims=True)
    out = {}
    for qi in range(len(queries)):
        sims = dn @ qn[qi]
        # argsort descending by sim, tie-break ascending corpus id: sort on
        # (-sim, id). lexsort sorts by the LAST key first.
        ids = np.array([corpus_id(i) for i in range(len(data))])
        order = np.lexsort((ids, -sims))[:k]
        out[str(qi)] = [int(ids[o]) for o in order]
    return out


def run_hnswlib(data, queries, k):
    """Build an hnswlib `cosine` index over `data`, query top-k -> {qid: [corpus_id...]}.

    hnswlib labels are the corpus ids we add_items with, so the returned ids are already
    in sparq dict-id space — directly comparable to the exact-kNN map."""
    import hnswlib
    import numpy as np

    dim = data.shape[1]
    index = hnswlib.Index(space=SPACE, dim=dim)
    # Seeded + single-threaded build/query: makes the capture byte-for-byte reproducible.
    index.init_index(
        max_elements=len(data), ef_construction=EF_CONSTRUCTION, M=M, random_seed=HNSW_SEED
    )
    index.set_num_threads(1)
    labels_in = [corpus_id(i) for i in range(len(data))]
    index.add_items(data, labels_in, num_threads=1)
    index.set_ef(EF_SEARCH)
    labels, _ = index.knn_query(queries, k=k, num_threads=1)
    return {str(qi): [int(x) for x in row] for qi, row in enumerate(labels)}


def recall_at_k(approx, exact, k):
    """Mean recall@k of approx vs exact ({qid:[ids]}); identical semantics to the
    sq-eifd vector_lib_adapter.recall_at_k (intersection-over-k, averaged)."""
    qids = [q for q in exact if q in approx]
    total = 0.0
    for q in qids:
        a = set(approx[q][:k])
        e = set(exact[q][:k])
        denom = min(k, len(e)) or 1
        total += len(a & e) / denom
    return total / len(qids)


def main():
    try:
        import numpy as np  # noqa: F401
        import hnswlib  # noqa: F401
    except ImportError as e:
        sys.stderr.write(
            "capture_hnswlib_ref: needs numpy + hnswlib (gather-only): %s\n" % e
        )
        return 1

    data = gen_matrix(SEED, N, DIM)
    queries = gen_matrix(QUERY_SEED, QUERIES, DIM)

    exact = cosine_exact_knn(data, queries, K)
    approx = run_hnswlib(data, queries, K)
    ref_recall = recall_at_k(approx, exact, K)

    # Emit the committed fixture on stdout. A self-describing header carries the
    # generation params so the Rust test fails closed if the fixture was regenerated
    # with different settings. recall is recorded for provenance only (the Rust test
    # recomputes hnswlib's recall off the ids; it does not trust this line).
    out = sys.stdout
    out.write("# sparq-vectors hnswlib reference fixture (sq-6te5) [OPUS-4.8]\n")
    out.write("# Captured from a REAL hnswlib run; regenerate with scripts/capture_hnswlib_ref.py.\n")
    out.write(
        "# params seed=%d query_seed=%d n=%d dim=%d k=%d queries=%d space=%s "
        "m=%d ef_construction=%d ef_search=%d\n"
        % (SEED, QUERY_SEED, N, DIM, K, QUERIES, SPACE, M, EF_CONSTRUCTION, EF_SEARCH)
    )
    out.write("# hnswlib_recall_at_k=%.6f (provenance only)\n" % ref_recall)
    out.write("# columns: <qid>\\t<hnswlib_ids_csv>\\t<exact_knn_ids_csv>\n")
    for qi in range(QUERIES):
        q = str(qi)
        hnsw_ids = ",".join(str(x) for x in approx[q])
        exact_ids = ",".join(str(x) for x in exact[q])
        out.write("%s\t%s\t%s\n" % (q, hnsw_ids, exact_ids))
    sys.stderr.write("captured %d queries; hnswlib recall@%d = %.4f\n" % (QUERIES, K, ref_recall))
    return 0


if __name__ == "__main__":
    sys.exit(main())
