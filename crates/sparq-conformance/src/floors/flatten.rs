//! [FABLE-5] sq-oy1f.26 — the W3C JSON-LD 1.1 `flatten` lane ratchet floor
//! (lib-side single source; imported by the runner + scoreboard + ci grep).

/// [FABLE-5] sq-oy1f.26 — `flatten` pass floor over the `flatten` category of
/// `w3c/json-ld-api`, under the NATIVE DOCUMENT-LEVEL oracle
/// (`sparq_jsonld::flatten()` + `json_ld_equal`). RATCHET: may only RISE.
///
/// ## Oracle re-pin (side-by-side, per the sq-oy1f honesty rule)
///
/// The lane MOVED from the RDF-writer round-trip oracle to the native Flattening
/// Algorithm (JSON-LD 1.1 API §7.1, bead sq-oy1f.26): `flatten()` expands the input,
/// generates the node map (§7.2), folds named graphs under `@graph`, sorts by `@id`,
/// and drops empty nodes, and the result is deep-compared to the suite's NORMATIVE
/// expected document via `json_ld_equal` (object key order insignificant; array order
/// significant only inside `@list`; integers exact, non-integral as f64) — the SAME
/// comparator the `expand` lane uses (sq-kk1mq). This is a document-level oracle, not
/// an RDF round-trip, so it detects JSON-LD structure the RDF oracle could not.
///
/// * **Old oracle (RDF round-trip, `graph_to_jsonld(Flattened)`): 50 pass / 0 fail / 8 skip.**
/// * **New oracle (native `flatten()`): MEASURED 46 pass / 7 fail / 5 skip** at the
///   pinned suite revision.
///
/// The re-pin goes DOWN (50 → 46) because the native oracle is stricter AND the native
/// flatten pipeline inherits the current gaps of the native `expand()` it composes over
/// (flatten = expand ∘ node-map ∘ fold). The 7 honest fails are ALL upstream expand-lane
/// gaps, NOT flatten-algorithm bugs, and each is owned by a different (file-disjoint) bead:
///
/// * `#t0002/#t0013/#t0028/#t0036` — expand raises `invalid typed value` on inputs the
///   expand lane also fails today (owned by the expand-lane correctness bead sq-oy1f.37,
///   which owns `src/expand.rs`).
/// * `#t0004/#t0015/#t0016` — native `expand()` drops empty-array properties
///   (`"…set1": []`) a spec-conformant expander retains; the retained-but-empty property
///   is what the expected flattened document keeps (same `src/expand.rs` surface).
///
/// The 5 SKIP are the documented buckets: 1 NegativeEvaluationTest, the JSON-LD-1.0-only
/// positives, and 1 compaction case (`#t0044`, which supplies a `context` member —
/// post-flatten compaction is the separate document-level Compaction bead sq-oy1f.27).
///
/// RATCHET after this re-pin: rise-only. As sq-oy1f.37 lands the expand fixes, the
/// inherited fails flip to passes and this floor rises.
pub const FLOOR: usize = 46;
