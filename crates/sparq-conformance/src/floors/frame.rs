//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `frame` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [OPUS-4.8] sq-oy1f.19 — `frame` (RDF → framed+compacted JSON-LD via the
/// native hand-rolled Framing Algorithm) pass floor over the SEPARATE
/// `w3c/json-ld-framing` suite (`scripts/fetch-jsonld-framing-tests.sh`).
/// RATCHET: may only RISE. This is the MEASURED pass count at the pinned
/// framing-suite revision — the number of `jld:FrameTest` cases for which
/// framing the input's RDF against the case frame document and re-parsing the
/// framed output reconstructs the SAME RDF dataset as re-parsing the suite's
/// NORMATIVE expected output (`reparse(frame(D, F)) ≡ reparse(expected)`).
///
/// ## Why compare against `expect`, not the input
///
/// Framing is a SELECT + RESHAPE, not a lossless transform: `@explicit` prunes
/// properties, an unmatched frame yields an empty `@graph`, `@default` fills a
/// value not in the input. So `reparse(frame(D)) ≡ D` is the WRONG oracle (the
/// framed RDF legitimately differs from the input). The normative answer is the
/// suite's `*-out.jsonld`; sparq must produce the SAME RDF as that expected
/// document. Comparing the two as canonical RDF datasets is envelope-insensitive
/// (both the bare-node `omitGraph` collapse and the `{"@graph":[…]}` envelope
/// re-parse to the same triples) and value-faithful, while NOT requiring sparq's
/// JSON layout to match pyld byte-for-byte (it does not — the same posture as
/// the toRdf/fromRdf/compact lanes).
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * NegativeEvaluationTests (`expectErrorCode`) — sparq's framer is TOTAL; it
///   never raises the spec's frame-validation errors (`invalid frame`, out-of-
///   range `@embed`). A 1.1 framer that does not model those errors cannot
///   honestly "pass" by rejecting, so these are SKIPPED (the compact-lane posture).
/// * A positive case with no `expect`, a non-object `frame`, an `input` the real
///   oxjsonld path rejects, or a remote `input`/`frame` URL — out of the gated
///   surface (SKIP, never a counted pass).
///
/// Note: the `ordered` option (1 case) governs JSON-array element ORDER, a
/// concern the RDF-level oracle is blind to by design — such a case is still
/// gated on RDF equality and passes iff the framed RDF matches the expected RDF,
/// the honest verdict an RDF oracle can give.
///
/// MEASURED 61/92 at the pinned framing-suite revision: 61 normative
/// RDF-equivalent frames, 28 honest framer divergences (value-pattern matching
/// over `@value` alternative arrays, `@explicit`/`@default` fill differences,
/// named-graph `@graph` framing shapes, `@list`/`@set` re-emit, blank-node
/// `@embed` table edge cases), and 3 SKIP (the suite's 3 NegativeEvaluationTests
/// — sparq's TOTAL framer does not raise the spec's frame-validation errors).
/// The 28 divergences are real framer gaps below the floor (tracked for a future
/// RISE — a child of sq-oy1f); SKIP is reserved for genuinely-unsupported cases,
/// never used to hide a divergence.
pub const FLOOR: usize = 61;
