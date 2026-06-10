# sparq-vectors — recorded follow-ups

v1 scope and rationale live in `research/genai-design.md` (phase 4) and the
crate README. These are the deliberate cuts, in rough priority order:

- **Streaming store builds.** `VectorStore::create`/`put` accumulate the dense
  data in RAM and `finalize` writes the file once. Fine to ~10M×384 f32
  (≈15 GB — already marginal); a streaming writer that appends vectors to the
  data section and spills the id→slot index would remove the RAM ceiling.
- **Persistent / out-of-core ANN.** The HNSW index is rebuilt from the mmap'd
  store on open (a one-off per-process cost; see the README throughput table —
  ~33 s release for 50k×32 on an M1). For 10M+ vector stores the index itself
  should be a versioned on-disk artifact — DiskANN/Vamana-style graph with
  mmap'd adjacency, or `instant-distance` serde persistence as a stopgap
  (adds serde+bincode and a second format to version, which is why v1
  skipped it).
- **Quantization.** f32-only today. Scalar i8 (4×) or product quantization for
  100M-scale stores; the `.spqv` header has 12 reserved bytes for an encoding
  tag.
- ~~**Hybrid RRF fusion** (lexical + structural + vector)~~ **DONE** —
  `fuse_rrf` / `fuse_scores` in `src/fuse.rs` (rank-based RRF `k = 60` +
  min-max alpha-blend over plain ranked lists, so `sparq-vectors` never
  depends on `sparq-sim`); recipe in the README, research in
  `research/genai-text-embedding-practices.md`. Remaining cut: per-list
  *weights* for RRF (Elasticsearch-style weighted RRF) — add a weighted
  variant if a third signal ever needs down-weighting.
- **Planner cardinality hook** (GNCE-style) — explicitly out of v1.
- **WASM.** Nothing here is wasm-hostile except memmap2 (the build-phase store
  and the in-RAM index are pure computation); a wasm feature would need a
  bytes-backed `VectorStore::open_from_bytes`. Not wired up because the wasm
  graph crate must not grow vector deps by default.
- **Big-endian targets.** `.spqv` is little-endian; `create`/`open` reject
  big-endian hosts rather than byte-swap. Swap-on-read is easy if anyone ever
  needs it.
