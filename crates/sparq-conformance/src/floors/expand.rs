//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `expand` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [SONNET-4.6] sq-kk1mq (oracle-correction re-baseline; supersedes the old
/// sq-oy1f RDF-equivalence oracle) — `expand` pass floor over the
/// `expand` category of `w3c/json-ld-api`, now measured with the NATIVE
/// DOCUMENT-LEVEL oracle: `sparq_jsonld::expand(input, opts, &NoopLoader)`
/// produces a `Json` value; that value is deep-compared against the suite's
/// expected expanded document using the runner's `json_ld_equal` comparator
/// (object key order insignificant; array order SIGNIFICANT only inside `@list`
/// values, insignificant elsewhere; numeric equality as f64).
///
/// ## Oracle-correction rationale
///
/// The OLD oracle (sq-oy1f, merged before this bead) compared the RDF dataset
/// produced by the engine's JSON-LD WRITER (`graph_to_jsonld(Expanded)`) against
/// the RDF produced by re-parsing the suite's expected document. That oracle
/// measured the writer's RDF fidelity, not the expansion algorithm. A case could
/// "pass" by coincidence (writer happened to produce equivalent RDF) even when the
/// expansion JSON shape differed from the spec, or fail spuriously when the input
/// had no RDF projection but the expander output was correct. The new oracle
/// measures JSON-LD data-model (semantic) equivalence — order-insensitive outside
/// `@list` per bead sq-kk1mq — NOT structural identity with the reference output.
///
/// NOTE: ~18 of the 240 passes are semantically-equal-but-reordered vs. the W3C
/// reference (@type order #tpr30, @id-map #tm001, @index #tpi06, multi-value
/// #tn004/#t0030 families); a strict order-sensitive harness would fail these 18
/// (strict-ordered count = 222).  The comparator is intentionally order-insensitive
/// per the JSON-LD data model (elements outside `@list` are a set).
///
/// This is an HONEST REBASE, not a ratchet weakening: the new floor may be lower
/// than 247 (old oracle measured 247/385) because some old passes were oracle
/// artefacts (the writer round-tripped to equivalent RDF while the JSON was wrong).
/// The gain is precision: passes now mean the expander produced a semantically
/// correct expanded document (data-model equivalence, not byte-identical structure).
/// See `run_expand_native` for the skip buckets and the PR body for the full
/// old-vs-new breakdown (bead sq-kk1mq).
///
/// ## What changed from the old oracle
///
/// * OLD (sq-oy1f, EXPAND floor = 247): RDF-equivalence —
///   `reparse_rdf(write_expanded(ingest_rdf(input))) ≡ reparse_rdf(expected)`.
///   Forwarded NO options; skipped empty-RDF inputs, 1.0-mode cases.
/// * NEW (sq-kk1mq, EXPAND floor = measured): document-level JSON comparison —
///   `json_ld_equal(expand(input, opts), expected_json)`. Forwards base,
///   expandContext, processingMode from the manifest; attempts all positive tests
///   regardless of RDF projection emptiness.
///
/// ## Honest SKIP buckets (recorded, not passed, not failed)
///
/// * `requires` optional-feature cases — out of the gated surface (same as before).
/// * NegativeEvaluationTests — sparq's expander raises some spec errors but
///   error-code completeness is unverified; deferred to a child bead of sq-oy1f.
///   SKIP (honest), never a counted pass.
/// * Remote `input` URLs — no network (SKIP).
/// * No `expect` file — nothing to compare (SKIP).
///
/// ## RATCHET: may only RISE
///
/// This floor is the MEASURED pass count with the new oracle at the pinned
/// suite revision. It is NOT aspirational. Future rises land when the native
/// expander fixes known divergences (tracked as children of sq-oy1f).
/// MEASURED 240/385 at the pinned revision under the new document-level oracle
/// (sq-kk1mq): 240 pass / 36 fail / 109 skip.  Breakdown vs. old oracle (247/10/128):
///   - 7 fewer passes (247→240): NET of 20 old-pass→new-fail flips (old oracle over-passed;
///     writer happened to produce equivalent RDF while JSON structure differed from spec),
///     offset by 13 recoveries: 8 old-fail→new-pass via oracle precision (new oracle
///     correctly passes cases the old oracle spuriously failed), plus 5 old-skip→new-pass
///     via options forwarding (processingMode/expandContext now forwarded; previously skipped)
///   - 26 more fails (10→36): honest divergences the new oracle reveals (JSON-level
///     mismatches invisible to the old RDF oracle, options-driven, or @direction shapes)
///   - 19 fewer skips (128→109): the old 1.0-mode and empty-RDF skip buckets are gone
///     (the new oracle forwards processingMode and doesn't require a non-empty RDF
///     projection); the 109 that remain are the NegativeEvaluationTests (deferred)
pub const FLOOR: usize = 240;
