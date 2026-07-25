//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `frame` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [FABLE-5] sq-oy1f.29 — `frame` pass floor over the SEPARATE `w3c/json-ld-framing`
/// suite (`scripts/fetch-jsonld-framing-tests.sh`), for the NATIVE document-level
/// Framing Algorithm (`sparq_jsonld::frame::frame()` — expand → frameExpansion →
/// node map + `@merged` → frame matching → prune → compact against the frame's
/// `@context`). RATCHET: may only RISE. This is the MEASURED pass count at the
/// pinned framing-suite revision.
///
/// ## Oracle (re-pinned side-by-side, sq-oy1f.29)
///
/// * **Old (RDF-first framer, `graph_to_jsonld_framed`): 61/92** under the
///   RDF-answer-equivalence oracle (`reparse(frame(D, F)) ≡ reparse(expected)`), with
///   3 SKIPs (the suite's NegativeEvaluationTests — that framer was TOTAL and never
///   raised the spec's frame-validation errors) and 28 honest divergences
///   (value-pattern matching over `@value` alternative arrays, `@explicit`/`@default`
///   fill, named-graph `@graph` framing shapes, `@list`/`@set` re-emit, blank-node
///   `@embed` table edges).
/// * **New (native pipeline, THIS floor): 92/92** under the STRONGER normative
///   document oracle — the framed output is deep-compared to the suite's expected
///   document with `json_ld_equal` (object key order insignificant; array order
///   significant only inside `@list`), and the 3 NegativeEvaluationTests are RUN
///   (pass iff `frame()` raises exactly the manifest's `expectErrorCode`:
///   `invalid frame`, `invalid @embed value`), not skipped.
///
/// All 28 old-framer divergences are resolved and the negatives are modelled, so the
/// lane holds a full score with ZERO skips at the pinned revision. Documented
/// behavioural fallbacks that do NOT currently cost a case: `@embed: @link` is
/// treated as `@once` (design record §11 — no output-tree object identity), and
/// `@embed: @last` uses an embed-then-demote post-pass equivalent to the reference
/// processors' remove-embed.
pub const FLOOR: usize = 92;
