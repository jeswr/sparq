# sparq-vectors — recorded follow-ups

v1 scope and rationale live in `research/genai-design.md` (phase 4) and the
crate README. Status of the deliberate cuts:

- ~~**Streaming store builds.**~~ **DONE** — [`StreamingWriter`]: every `put`
  appends the vector straight to the file's data section and spills the id to
  a sidecar file, so build-phase memory is O(1) regardless of store size
  (`finalize` transiently holds the 8-byte-per-vector id→slot index to sort
  it — a `dim·4 / 8` reduction over the in-RAM builder, 192× at 384-d f32;
  truly out-of-core index sorting would need an external sort and is not
  worth it below ~10⁹ vectors). The output is **byte-identical** to the
  in-RAM `create`/`put`/`finalize` path (round-trip-tested) — same version-1
  format, no format change. One documented contract difference: duplicate ids
  are reported at `finalize` (the sort reveals them), not at `put` (eager
  detection would need the in-RAM id set this writer exists to avoid).
- **Persistent / out-of-core ANN.** DEFERRED (out-of-scope-by-design for v2 as
  for v1): the HNSW index is rebuilt from the mmap'd store on open (a one-off
  per-process cost; see the README throughput table — ~33 s release for
  50k×32 on an M1). For 10M+ vector stores the index itself should be a
  versioned on-disk artifact — DiskANN/Vamana-style graph with mmap'd
  adjacency, or `instant-distance` serde persistence as a stopgap (adds
  serde+bincode and a SECOND format to version, which is why it stays
  deferred until a workload actually hits the rebuild wall).
- **Quantization.** DEFERRED (out-of-scope-by-design): f32-only today. Scalar
  i8 (4×) or product quantization for 100M-scale stores; the `.spqv` header
  has 12 reserved bytes for an encoding tag, so this lands as a backwards-
  compatible header bump when needed — designing the encoding now, without a
  driving workload or recall budget, would be speculation.
- ~~**Hybrid RRF fusion** (lexical + structural + vector)~~ **DONE** —
  `fuse_rrf` / `fuse_scores` in `src/fuse.rs` (rank-based RRF `k = 60` +
  min-max alpha-blend over plain ranked lists, so `sparq-vectors` never
  depends on `sparq-sim`); recipe in the README, research in
  `research/genai-text-embedding-practices.md`. The remaining cut —
  per-list *weights* — is now **DONE** too: `fuse_rrf_weighted`
  (Elasticsearch-style `Σ wₗ/(k + rankₗ)`; weight 0 mutes a list, unit
  weights reduce exactly to `fuse_rrf`, negative/non-finite weights rejected).
- **Planner cardinality hook** (GNCE-style) — explicitly out of v1/v2. (Engine-seams
  wave note: sparq-engine's opt-in `cs-planner` feature now demonstrates the
  injection shape — an external estimator installed via `with_cs_table` and
  consulted by the greedy planner for star joins; a learned/GNCE estimator could
  follow the same pattern. Still deferred here.)
- ~~**WASM** (bytes-backed open)~~ **DONE** (the cheap half) —
  `VectorStore::open_from_bytes(Vec<u8>)` opens a `.spqv` document held in
  memory with validation identical to `open`; all read paths are shared
  between the mmap and owned-bytes backings. NO wasm feature is wired up (by
  design): the crate stays outside the wasm dependency graph, and memmap2 is
  still an unconditional dep — a `wasm` cargo feature gating memmap2 out is
  the remaining (deliberate) cut, to be added only with an actual wasm
  consumer so the wasm bundle never grows by default.
- **Big-endian targets.** DEFERRED (out-of-scope-by-design): `.spqv` is
  little-endian; `create`/`open`/`open_from_bytes`/`StreamingWriter::create`
  reject big-endian hosts rather than byte-swap. Swap-on-read is easy if
  anyone ever needs it; no big-endian deployment target exists today.
