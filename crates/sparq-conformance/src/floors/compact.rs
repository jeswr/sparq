//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `compact` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [FABLE-5] sq-oy1f.27 — `compact` pass floor, RE-PINNED for the ORACLE
/// CORRECTION to the native document-level Compaction Algorithm
/// (`sparq_jsonld::compact::compact()`), compared against the suite's
/// NORMATIVE EXPECTED document via `json_ld_equal` — the same oracle shape as
/// the `expand` (sq-kk1mq) and `flatten` (sq-oy1f.26) lanes. RATCHET: may only
/// RISE from here.
///
/// ## Side-by-side re-pin (old oracle vs new oracle, same pinned suite rev)
///
/// * OLD (sq-3uos5/sq-oy1f.16, floor **186**): the engine's RDF-first writer
///   (`graph_to_jsonld_compact`) + an **oxjsonld self-reparse round-trip**
///   (`reparse(compact(D, ctx)) ≡ D`) — measured 186 pass / 35 fail / 25 skip.
///   That oracle measured RDF losslessness of the writer, NOT the Compaction
///   Algorithm: it never compared against the suite's expected document, could
///   pass shapes a strict third-party processor reads differently (the
///   sq-oy1f.10 data-loss class), and had to SKIP every case whose features the
///   writer never emits (scoped/typed contexts, `@nest`, `@index`/`@id` maps,
///   non-inline contexts, empty-RDF inputs).
/// * NEW (sq-oy1f.27, floor **228**): the spec Compaction Algorithm over the
///   natively expanded document, deep-compared to the W3C EXPECTED document —
///   measured **228 pass / 0 fail / 18 skip** of 246 (sq-uzdw7 re-pin; the
///   sq-oy1f.27 measurement was 228/1/17 with `#t0038` a documented below-floor
///   FAIL, since reclassified — see below; the pass count and floor are
///   unchanged). The 42-case rise over the old floor is real algorithm coverage
///   (scoped/typed contexts, `@nest`, container maps, `@graph` containers,
///   options forwarding), not an oracle relaxation: the new oracle is strictly
///   harder (document equivalence vs self-reparse). Passes also hold under
///   STRICT order-sensitive JSON equality (strict-ordered count = 228 — no pass
///   relies on the comparator's outside-`@list` array order-insensitivity).
///
/// ## [OPUS-5] sq-gzsky — the NEGATIVE lane lands: 228 → 244
///
/// The 17 skipped NegativeEvaluationTests are now RUN, exactly like the `expand`
/// (sq-gzsky) and `frame` (sq-oy1f.29) lanes: a case passes **iff** `compact()`
/// errors with EXACTLY the manifest's `expectErrorCode`, so a raised-but-WRONG code
/// FAILS. MEASURED **244 pass / 0 fail / 2 skip** of 246 (99.1% strict) at the pin —
/// up from 92.6%, closing the peer gap (JSON-LD.ex 99.6, Titanium 98.0, jsonld.js
/// 97.5) that bead sq-hmd7l.22 recorded.
///
/// 15 of the 16 runnable negatives passed unchanged (including all ten
/// `processingMode: json-ld-1.0` cases, which a 1.1 processor MUST honour and which
/// RUN). One needed a real fix, plus one it exposed:
///
/// * **`IRI confused with prefix`** — IRI Compaction §7.1 step 5 tail ("if the IRI
///   scheme of var matches any term in active context with prefix flag set to true,
///   and var has no IRI authority") was entirely absent, so `compact_iri` emitted a
///   `tag:`-scheme IRI verbatim into a context that also defines `tag` as a prefix —
///   a document a re-expanding consumer resolves to a DIFFERENT IRI. Implementing it
///   made `compact_iri` fallible. Fixes `#te002`. 1.1-ONLY, like the sibling
///   `create_term_definition` round-trip check: framing `#t0010` pins the 1.0
///   behaviour with `specVersion: json-ld-1.0` (so `floors::frame` is unaffected).
/// * **Property-valued `@index` container key** (exposed by the above, §8 step
///   12.8.9.6.1) — the index mapping is stored VERBATIM as authored (§4.2.2 step 22),
///   i.e. normally a TERM or COMPACT IRI, and was being fed straight into IRI
///   Compaction without the spec's "after first IRI expanding it" leg. It is now
///   resolved by expanding the index key and taking the entry of the compacted item
///   that expands to the same IRI. `#t0112` and `#t0114` keep passing, now for the
///   right reason.
///
/// ## The 2 honest SKIPs (0 fails at the pin)
///
/// * `#te001` — the ONE 1.0-only NEGATIVE (`option.specVersion: json-ld-1.0`),
///   expecting `compaction to list of lists`: a code JSON-LD 1.1 RETIRED (absent from
///   the 1.1 error registry, so the closed `JsonLdErrorCode` enum cannot express it)
///   because 1.1 ALLOWS lists of lists. Raising would be WRONG, not unimplemented.
///   Scope pinned by the runner's `compact_1_0_negative_skips_are_pinned`, which also
///   asserts the `processingMode: json-ld-1.0` negatives still RUN.
/// * `#t0038` — the ONE 1.0-only POSITIVE, unchanged (see below).
///
/// ## The `#t0038` decision (unchanged)
///
/// * `#t0038` (`specVersion: json-ld-1.0`, the ONLY 1.0-only positive at the
///   pin) expects 1.0-era compact-IRI creation through an EXPANDED (map-valued)
///   term definition (`body:/format`). A REC-conformant 1.1 processor never
///   sets the prefix flag on an expanded term — in either processing mode
///   (`compact/p001` pins exactly that for 1.0 mode, so "fixing" `#t0038`
///   would break `#tp001`) — so this expectation is unreachable; jsonld.js and
///   pyld make the same trade, and the suite's own `vocab.jsonld` defines
///   `specVersion` as "the JSON-LD version to which the test applies".
///   DECISION (bead sq-uzdw7): reclassified from a documented below-floor FAIL
///   to an honest SKIP, NARROWLY PINNED to this exact manifest id (never a
///   blanket `specVersion: json-ld-1.0` match): the runner's
///   `is_pinned_1_0_only_skip` matches `#t0038` alone, its
///   `t0038_skip_is_narrowly_scoped` test enforces that scope, the
///   `processingMode: json-ld-1.0` cases of the 1.1 suite still RUN, and any
///   FUTURE 1.0-only positive added at a suite-pin bump RUNS (and fails the
///   scope test) rather than being silently skipped.
pub const FLOOR: usize = 244;
