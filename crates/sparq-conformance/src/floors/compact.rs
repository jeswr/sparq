//! [FABLE-5] sq-oy1f.40 — the W3C JSON-LD 1.1 `compact` lane ratchet floor
//! (relocated from `tests/jsonld_suite.rs` to this lib-side single source).

/// [OPUS-4.8] sq-3uos5 — compact (RDF → compacted JSON-LD via the native
/// hand-rolled Compaction Algorithm, round-trip) pass floor. RATCHET: may
/// only RISE. This is the MEASURED pass count at the pinned revision — the
/// number of `jld:CompactTest` cases for which compacting the input's RDF
/// against the case `@context` and re-parsing the compacted document
/// reconstructs the SAME RDF dataset (`reparse(compact(D, ctx)) ≡ D`). It is
/// NOT the suite total: many cases exercise input-document compaction
/// features sparq's fromRdf-then-compact writer does not target (scoped/typed
/// contexts, `@nest`, `@index`/`@id` maps the writer never emits, `@protected`
/// redefinition errors, processing-mode error-raising), and those are SKIPPED
/// (not failed) — see `run_compact` for the honest skip buckets.
///
/// [OPUS-4.8] sq-oy1f.16 — RAISED 163 → 186 after the compaction faithfulness
/// fixes (#978, sq-oy1f.12/.13/.14: `@reverse` double-invert, non-string
/// `@language`/`@none`, `@type:@id`-coerced-key-vs-plain-string IRI confusion,
/// container round-trips) landed on main. Re-MEASURED on current main at the
/// pinned revision: compact 186 pass / 35 fail / 25 skip (was 163/58/25). The
/// +23 FAIL→PASS are the cases those writer fixes made lossless. The 35
/// remaining failures are real writer gaps below the floor (scoped/typed
/// contexts, `@nest`, `@index`/`@id` map shapes the writer does not emit),
/// tracked for a future RISE; the 25 SKIP are unchanged (negatives sparq does
/// not raise, JSON-LD-1.0-only, non-inline/remote/multi `@context`, empty-RDF).
pub const FLOOR: usize = 186;
