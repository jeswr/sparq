# sparq-vectors — outstanding work

Tracked in beads (not here). Run `bd ready -l area:sparq-vectors` or
`bd list -l area:sparq-vectors`. See AGENTS.md for the no-markdown-TODOs policy.

## Notes

Design rationale and DONE-status records retained from the previous follow-ups
list (not task tracking). Forward-looking items are tracked in beads. v1 scope
and rationale live in `research/genai-design.md` (phase 4) and the crate README.
Status of the deliberate cuts:

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
- ~~**Persistent / out-of-core ANN.**~~ **DONE** (sq-7zc) [OPUS-4.8] —
  [`DiskAnnIndex`] in `src/diskann.rs`: a self-contained **Vamana on-disk
  graph** (`.spqg`) we build and encode end to end (the `instant-distance`
  HNSW is a closed graph — its adjacency can't be laid out on disk, so this is
  a second, owned index rather than a serde dump of the first). Build once
  (RobustPrune, α-passes, medoid entry); `open` is mmap + header validation —
  **no rebuild**, the whole point. Each node record co-locates `[id, degree,
  dim·f32 vector, R·u32 neighbour slots]` so a greedy-search hop is one
  contiguous page read (the DiskANN locality property). Cosine throughout,
  identical to `ann` (unit vectors, `cos = 1 − d²/2`), so scores/rankings match
  the exact/HNSW searchers. **Recall@10 = 0.966 vs exact brute force on the
  50k×32 synthetic set** (`tests/diskann.rs`), and a reopened handle returns
  byte-identical neighbours to the freshly built one (restart-survival gate).
  HONEST GAP to *full* DiskANN: full-precision vectors are searched directly
  from the mmap; there is **no PQ-compressed in-RAM candidate cache** to skip
  disk reads (full DiskANN ranks on PQ codes, re-ranks the beam on disk). That
  quantization layer is the sibling task below (sq-nq5) — at scales where the
  graph fits page cache the two are equivalent; PQ matters only once the
  vectors themselves exceed RAM. Build is single-threaded O(n·L·R) and slower
  than HNSW's rayon build (~50 s vs ~33 s for 50k×32 at opt-level 2); a
  rayon-parallel Vamana build is tracked in beads.
- ~~**Quantization.**~~ **DONE** (sq-nq5) [OPUS-4.8] — [`ScalarQuantizer`] +
  [`ProductQuantizer`] in `src/quant.rs`, both over the L2-normalized /
  squared-Euclidean cosine convention (`cos = 1 − d²/2`) so estimates are
  directly comparable to the exact/HNSW/DiskANN searchers.
  - **SQ**: per-dimension `[min,max]` linear `f32 → u8` (256 levels), 4×
    smaller; max per-component error is half a quantization step (measured
    0.00096 ≤ the 0.0039 bound), reconstruction cosine > 0.99.
  - **PQ**: `M` subspaces (default 16), k-means codebook of `K` (default 256)
    per subspace, `M`-byte codes, **asymmetric distance computation** (ADC) via
    per-subspace [`DistanceTable`]. Measured recall@10 on a 10k clustered
    synthetic set at 8× (M=16): **PQ-alone ≈ 0.60**, **PQ candidate filter +
    full-precision re-rank of top-50 ≈ 0.98** — the latter is the DiskANN search
    loop and the load-bearing number; PQ alone is honestly a coarse filter, not
    a final ranking. Higher M trades compression for PQ-alone recall (M=32 / 4×
    → ~0.94 PQ-alone; dim=128 / 32× → ~0.23 PQ-alone, ~0.61 after re-rank).
  - **Standalone, not yet wired into `.spqg`** (the honest call): the encoders /
    decoders / distance-estimators + the in-RAM candidate cache
    ([`EncodedStore`]) are complete and tested, and `quant.rs` documents exactly
    how `diskann.rs` would consume them (fit PQ on the store → encode every
    vector → rank candidates on codes in RAM → re-rank the beam off the mmap).
    The `.spqg` format is **unchanged** — baking PQ codes into the on-disk record
    now would still be speculation against a recall budget we don't have, and the
    `.spqv`/`.spqg` reserved bytes keep that a backwards-compatible bump when a
    driving workload arrives. (The `search_slots` integration in `diskann.rs` —
    the localized change to rank on PQ codes + a re-rank pass — and persisting the
    codebook beside the graph are tracked in beads.)
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
  follow the same pattern. Deferred here; tracked in beads.)
- ~~**WASM** (bytes-backed open)~~ **DONE** (the cheap half) —
  `VectorStore::open_from_bytes(Vec<u8>)` opens a `.spqv` document held in
  memory with validation identical to `open`; all read paths are shared
  between the mmap and owned-bytes backings. NO wasm feature is wired up (by
  design): the crate stays outside the wasm dependency graph, and memmap2 is
  still an unconditional dep — a `wasm` cargo feature gating memmap2 out is
  the remaining (deliberate) cut, to be added only with an actual wasm
  consumer so the wasm bundle never grows by default (tracked in beads).
- **Big-endian targets.** Out-of-scope-by-design (tracked in beads): `.spqv` is
  little-endian; `create`/`open`/`open_from_bytes`/`StreamingWriter::create`
  reject big-endian hosts rather than byte-swap. Swap-on-read is easy if
  anyone ever needs it; no big-endian deployment target exists today.
