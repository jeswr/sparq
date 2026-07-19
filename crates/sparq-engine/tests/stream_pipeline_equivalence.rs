//! [FABLE-5] sq-7d3dj.34.2.2 + sq-7d3dj.34.2.3 — differential streaming-equivalence
//! harness for the SELECT-JSON emit core (`exec::eval_select_json_emit`) and the
//! streaming pipeline's §5.1 per-shape suites.
//!
//! 🤖 SPARQ agent — the design record's §10 B2 deliverable
//! (`research/lazy-pull-incremental-emission.md`), EXTENDED by `.2.3` per the amended
//! §5.1 contract (decision `sq-7d3dj.34.2.7`). The harness now carries TWO layers:
//!
//! 1. **The original byte-diff suite** (below) — the CONCATENATION of the chunks
//!    handed to `eval_select_json_emit` is BYTE-IDENTICAL to the single-string
//!    `query_json` for the same query, across every front door. This holds for ALL
//!    shapes (the streaming pipeline classifies independently of `flush`, so the
//!    front doors always agree with each other), and for fallback/buffered shapes it
//!    also pins the historical buffered bytes.
//! 2. **The §5.1 suites** (`sect_5_1` module at the bottom) — for shapes the
//!    streaming pipeline ADMITS (seed + bind-join chains), the emitted order is the
//!    deterministic seed-lex order, permitted to differ from the retired buffered
//!    order; verification is per-shape SOLUTION equivalence vs the toggled-off
//!    buffered path (set under DISTINCT, multiset otherwise, sub-multiset + count
//!    under Slice), double-run byte-identity (determinism), a fallback-shape
//!    path-taken assertion, a perturbed-SOLUTION mutation witness, and the
//!    first-emit-before-eval-complete probe. MULTI-BLOCK NON-VACUITY: the fixtures
//!    force the tiny testing ramp so a join value's seed rows PROVABLY straddle a
//!    block boundary (the sq-7d3dj.34.2.7 counterexample class), and one fixture
//!    exceeds the default 4096-row first block.
//!
//! ## What the byte-diff layer asserts, and why byte-level
//!
//! The `#1745` API contract (`eval_select_json_emit` rustdoc) is: *the concatenation of
//! the chunks is byte-identical to `eval_select_json`/`eval_select_json_chunks` for the
//! same `flush`; only the chunk boundaries may differ.* That invariant is
//! **byte-level and order-sensitive, NOT multiset**: two front doors emitting the same
//! set of solutions in a different serialised order (or different whitespace / escaping /
//! value formatting) are a FAILURE, because a streaming HTTP consumer sees the raw bytes
//! and parses them positionally.
//!
//! ## How the private emit core is reached (real path, not a mock)
//!
//! `exec` is a private module, so an integration test cannot call
//! `exec::eval_select_json_emit` by name. It drives the SAME emit core through its two
//! public front doors, so the code under test is the real one:
//!
//! * **`flush = Some(64 KiB)`** — `query_json_chunks_with_budget` (collected `Vec`) and
//!   `query_json_stream_with_budget` (live `FnMut` sink) both call
//!   `eval_select_json_emit(.., Some(JSON_CHUNK_BYTES = 64 KiB), ..)`. Testing BOTH covers
//!   the buffered-`Vec` layout AND the streaming-sink layout the design is actually about.
//! * **`flush = None`** — `query_json` returns `join_chunks(eval_select_json_chunks(.., None))`,
//!   i.e. the `eval_select_json_emit(.., None, ..)` concatenation. This is ALSO the buffered
//!   reference (`eval_select_json`), so for `flush = None` the emit-concat and the reference
//!   coincide by construction; the harness still exercises it as a distinct observation and
//!   pins it to a valid, non-empty SPARQL-results document.
//!
//! ## The corpus (design record §10 B2)
//!
//! * a multi-variable `SELECT DISTINCT` over a 3+-pattern join (the **q04 class** — the
//!   shape #1747's distinct-pushdown REJECTS because the projection is multi-variable, so
//!   it falls through to the streaming pipeline);
//! * a `UNION` of BGPs;
//! * `LIMIT` / `OFFSET` slices;
//! * residual `FILTER`s;
//! * `ORDER BY` (the buffered pipeline-breaker floor, §5 ladder step 3);
//! * empty results;
//!
//! each under **both** `flush = Some(64 KiB)` and `flush = None`.
//!
//! ## Non-vacuity (empirically verified, not by inspection)
//!
//! `mutation_witness_a_perturbed_expected_byte_goes_red` deliberately perturbs one byte
//! of the expected reference and asserts the byte-diff catches it — proving the harness
//! is not vacuously green. It is `#[should_panic]`, so the test run itself demonstrates
//! (empirically) that a single wrong byte fails the equality the other tests rely on.

use sparq_core::Graph;
use sparq_engine::{query_json, query_json_chunks_with_budget, query_json_stream_with_budget, QueryBudget};

// ─── corpus construction ──────────────────────────────────────────────────────

/// A deterministic graph shaped like the SP2Bench q04 neighbourhood: `article`s each
/// have a `creator`, a `journal`, and a `year`; some articles share a creator (so a
/// multi-variable DISTINCT over the creator×journal join has genuine duplicates to
/// remove). `n_articles` scales the result so the JSON exceeds one 64 KiB chunk.
fn q04_graph(n_articles: usize) -> Graph {
    let mut nt = String::new();
    for i in 0..n_articles {
        let art = format!("<http://ex/article/{}>", i);
        // Deliberately few distinct creators / journals so DISTINCT removes rows and the
        // join fans out (multiple articles per creator, multiple per journal).
        let creator = i % 7;
        let journal = i % 5;
        let year = 1990 + (i % 30);
        nt.push_str(&format!("{} <http://ex/creator> <http://ex/person/{}> .\n", art, creator));
        nt.push_str(&format!("{} <http://ex/journal> <http://ex/journal/{}> .\n", art, journal));
        nt.push_str(&format!(
            "{} <http://ex/year> \"{}\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
            art, year
        ));
        // A padding literal so string escaping + value formatting are exercised in the bytes.
        nt.push_str(&format!(
            "{} <http://ex/title> \"Article {} — a \\\"quoted\\\" title, padding padding padding\" .\n",
            art, i
        ));
    }
    Graph::load_str(&nt, "ntriples").expect("load q04-class graph")
}

/// A tiny graph for the empty-result and small-shape cases (single chunk, `flush` never trips).
fn small_graph() -> Graph {
    let nt = "\
<http://ex/a> <http://ex/p> <http://ex/b> .
<http://ex/a> <http://ex/q> \"1\"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://ex/b> <http://ex/p> <http://ex/c> .
<http://ex/b> <http://ex/q> \"2\"^^<http://www.w3.org/2001/XMLSchema#integer> .
<http://ex/c> <http://ex/p> <http://ex/a> .
<http://ex/c> <http://ex/q> \"3\"^^<http://www.w3.org/2001/XMLSchema#integer> .
";
    Graph::load_str(nt, "ntriples").expect("load small graph")
}

const PFX: &str = "PREFIX ex: <http://ex/> PREFIX xsd: <http://www.w3.org/2001/XMLSchema#> ";

/// The corpus: `(label, &graph, query)` triples covering every shape §10 B2 names. Each
/// is run under both flush modes by the driver below.
fn corpus<'a>(q04: &'a Graph, small: &'a Graph) -> Vec<(&'static str, &'a Graph, String)> {
    let mut c: Vec<(&'static str, &Graph, String)> = Vec::new();

    // 1. q04 class — multi-variable SELECT DISTINCT over a 3-pattern join. The
    //    projection is (?creator, ?journal), which #1747's single-var distinct-pushdown
    //    REJECTS, so this falls through to the streaming pipeline the design targets.
    //    ORDER BY makes the row order deterministic so the byte-level reference is stable.
    c.push((
        "q04_distinct_multivar_join",
        q04,
        format!(
            "{PFX}SELECT DISTINCT ?creator ?journal WHERE {{ \
             ?a ex:creator ?creator . ?a ex:journal ?journal . ?a ex:year ?year \
             }} ORDER BY ?creator ?journal"
        ),
    ));
    // The same shape WITHOUT the stabilising ORDER BY — a self-consistency check confirms
    // the engine is deterministic on it (see `driver` self-check), so the cross-path
    // byte-diff is meaningful even for the unordered DISTINCT case.
    c.push((
        "q04_distinct_multivar_join_unordered",
        q04,
        format!(
            "{PFX}SELECT DISTINCT ?creator ?journal WHERE {{ \
             ?a ex:creator ?creator . ?a ex:journal ?journal . ?a ex:year ?year }}"
        ),
    ));

    // 2. UNION of BGPs.
    c.push((
        "union_of_bgps",
        q04,
        format!(
            "{PFX}SELECT ?a ?v WHERE {{ \
             {{ ?a ex:creator ?v }} UNION {{ ?a ex:journal ?v }} \
             }} ORDER BY ?a ?v"
        ),
    ));

    // 3. LIMIT / OFFSET.
    c.push((
        "limit_offset",
        q04,
        format!("{PFX}SELECT ?a ?year WHERE {{ ?a ex:year ?year }} ORDER BY ?year ?a LIMIT 137 OFFSET 41"),
    ));

    // 4. Residual FILTER (a join with a post-join scalar filter).
    c.push((
        "residual_filter",
        q04,
        format!(
            "{PFX}SELECT ?a ?year WHERE {{ \
             ?a ex:year ?year . ?a ex:creator ?creator \
             FILTER(?year > 2005 && ?year < 2015) \
             }} ORDER BY ?a"
        ),
    ));

    // 5. ORDER BY over the full join (the buffered pipeline-breaker floor, §5 ladder 3).
    c.push((
        "order_by_full",
        q04,
        format!(
            "{PFX}SELECT ?creator ?journal ?year WHERE {{ \
             ?a ex:creator ?creator . ?a ex:journal ?journal . ?a ex:year ?year \
             }} ORDER BY DESC(?year) ?creator ?journal"
        ),
    ));

    // 6. Empty results — a matched shape that yields zero rows, and a projected-empty.
    c.push((
        "empty_no_match",
        small,
        format!("{PFX}SELECT ?s ?o WHERE {{ ?s ex:nonexistent ?o }}"),
    ));
    c.push((
        "empty_join_no_solutions",
        small,
        format!("{PFX}SELECT ?s WHERE {{ ?s ex:p ?mid . ?mid ex:p ?s FILTER(?s = ex:nonexistent) }}"),
    ));

    // A couple of small deterministic shapes so `flush = None` (single-chunk) and the
    // fast single-pattern scan are both in the corpus.
    c.push((
        "small_star",
        small,
        format!("{PFX}SELECT ?s ?p ?o WHERE {{ ?s ?p ?o }} ORDER BY ?s ?p ?o"),
    ));

    c
}

// ─── the differential driver ──────────────────────────────────────────────────

/// The two flush modes the harness exercises. `Some(64 KiB)` is the value both public
/// chunked entry points hand to `eval_select_json_emit`; `None` is the single-string
/// (`eval_select_json`) layout.
#[derive(Clone, Copy, Debug)]
enum Flush {
    /// `eval_select_json_emit(.., Some(JSON_CHUNK_BYTES = 64 KiB), ..)`, driven through
    /// both `query_json_chunks_with_budget` (Vec) and `query_json_stream_with_budget` (sink).
    SixtyFourKiB,
    /// `eval_select_json_emit(.., None, ..)` — the single-string `eval_select_json` layout.
    None,
}

/// Collect the emit-core chunks for `q` under `flush`, using the *streaming sink* front
/// door (`query_json_stream_with_budget`) for the `Some(64 KiB)` case.
fn stream_concat(g: &Graph, q: &str) -> String {
    let b = QueryBudget::unlimited();
    let mut out = String::new();
    query_json_stream_with_budget(g, q, &b, |c| {
        out.push_str(&c);
        std::ops::ControlFlow::Continue(())
    })
    .expect("stream query error");
    out
}

/// The emit-core concatenation under `flush`.
fn emit_concat(g: &Graph, q: &str, flush: Flush) -> String {
    let b = QueryBudget::unlimited();
    match flush {
        // Both `query_json_chunks_with_budget` (Vec) and the streaming sink go through
        // `eval_select_json_emit(.., Some(64 KiB), ..)`. Assert they AGREE, then return one.
        Flush::SixtyFourKiB => {
            let via_vec = query_json_chunks_with_budget(g, q, &b).expect("chunks query error").concat();
            let via_sink = stream_concat(g, q);
            assert_eq!(
                via_vec, via_sink,
                "the Vec and live-sink front doors of eval_select_json_emit(Some(64KiB)) must agree byte-for-byte"
            );
            via_vec
        }
        // `query_json` == join_chunks(eval_select_json_chunks(.., None)) == the
        // eval_select_json_emit(.., None, ..) concatenation (and the buffered reference).
        Flush::None => query_json(g, q).expect("buffered query error"),
    }
}

/// The buffered reference: `eval_select_json` (== `query_json`). Byte-identity is asserted
/// against THIS for every flush mode.
fn reference(g: &Graph, q: &str) -> String {
    query_json(g, q).expect("reference query error")
}

/// Run the full corpus under both flush modes, asserting byte-level order-sensitive
/// equality of the emit-concat against the buffered reference. Also self-checks that the
/// engine is DETERMINISTIC on each query (two reference evaluations agree) — otherwise a
/// cross-path byte-diff would be meaningless.
#[test]
fn stream_emit_concat_is_byte_identical_to_buffered_over_corpus() {
    let q04 = q04_graph(4000); // > 64 KiB of JSON on the wide shapes → multi-chunk
    let small = small_graph();
    let cases = corpus(&q04, &small);

    for (label, g, q) in &cases {
        // Self-check: the reference must be deterministic (order-preserving parallel
        // collection, design record §2) so byte-level cross-path equality is well-defined.
        let ref1 = reference(g, q);
        let ref2 = reference(g, q);
        assert_eq!(ref1, ref2, "reference is non-deterministic for [{label}] — byte-diff would be meaningless: {q}");

        // A valid, closed SPARQL-results document (the empty cases still close correctly).
        assert!(ref1.starts_with("{\"head\""), "[{label}] reference must open a results doc: {}", &ref1[..ref1.len().min(40)]);
        assert!(ref1.ends_with("]}}"), "[{label}] reference must close the results doc");

        for flush in [Flush::None, Flush::SixtyFourKiB] {
            let got = emit_concat(g, q, flush);
            assert_eq!(
                got, ref1,
                "[{label}] emit-concat under {flush:?} is NOT byte-identical to the buffered reference: {q}"
            );
        }
    }
}

/// A dedicated case pinning that the wide q04-class result actually splits into MULTIPLE
/// chunks under `Some(64 KiB)` — otherwise the `flush = Some(64 KiB)` path would be
/// exercising only the single-chunk layout and the multi-chunk boundary logic would be
/// untested (a vacuity trap distinct from the byte-value mutation below).
#[test]
fn q04_class_result_is_genuinely_multi_chunk_under_64kib() {
    let q04 = q04_graph(4000);
    let q = format!(
        "{PFX}SELECT DISTINCT ?a ?creator ?journal ?year WHERE {{ \
         ?a ex:creator ?creator . ?a ex:journal ?journal . ?a ex:year ?year \
         }} ORDER BY ?a"
    );
    let b = QueryBudget::unlimited();

    let single = query_json(&q04, &q).unwrap();
    assert!(single.len() > 64 * 1024, "corpus must exceed one 64 KiB chunk (got {} bytes)", single.len());

    let chunks = query_json_chunks_with_budget(&q04, &q, &b).unwrap();
    assert!(chunks.len() > 1, "expected a genuinely multi-chunk stream, got {}", chunks.len());
    // First chunk opens, only the last closes — the boundary logic the byte-diff protects.
    assert!(chunks[0].starts_with("{\"head\""), "first chunk must carry the header");
    assert!(chunks.last().unwrap().ends_with("]}}"), "only the last chunk closes the document");
    assert!(!chunks[0].ends_with("]}}"), "the header chunk must not be the whole document");
    // The load-bearing invariant, on the wide multi-chunk shape.
    assert_eq!(chunks.concat(), single, "multi-chunk concat must be byte-identical to the buffered reference");
}

// ─── non-vacuity: a perturbed expected byte MUST go red ────────────────────────

/// NON-VACUITY WITNESS (design record §10 B2: *a perturbed byte goes red*).
///
/// This test deliberately corrupts ONE byte of the expected reference and then asserts
/// the emit-concat equals the corrupted expectation. The assertion MUST fail — proving,
/// EMPIRICALLY (not by inspection), that the byte-level equality the real tests rely on
/// actually catches a single wrong byte. `#[should_panic]` inverts it: the test suite is
/// GREEN precisely because the perturbed comparison went RED.
///
/// The panic message is matched so this can only pass by failing the INTENDED assertion
/// (a compile/setup panic with a different message would not satisfy `expected`).
#[test]
#[should_panic(expected = "MUTATION WITNESS: perturbed byte must differ")]
fn mutation_witness_a_perturbed_expected_byte_goes_red() {
    let g = small_graph();
    let q = format!("{PFX}SELECT ?s ?p ?o WHERE {{ ?s ?p ?o }} ORDER BY ?s ?p ?o");

    // The true emit-concat (Some(64 KiB) front door) and the true reference.
    let got = emit_concat(&g, &q, Flush::SixtyFourKiB);
    let reference = query_json(&g, &q).expect("reference query error");
    // Sanity: without perturbation they ARE equal (so the failure below is caused ONLY by
    // the deliberate perturbation, not a pre-existing mismatch).
    assert_eq!(got, reference, "pre-condition: unperturbed emit-concat must match the reference");

    // Perturb exactly one byte of the EXPECTED value. Pick an ASCII byte in the body and
    // flip it to a different ASCII byte so the corrupted expectation is still valid UTF-8
    // but differs from `got` in exactly one position.
    let mut perturbed = reference.clone().into_bytes();
    // Choose a mid-document position that is a plain ASCII letter/digit to flip.
    let pos = perturbed
        .iter()
        .position(|&b| b.is_ascii_alphanumeric() && b != b'h') // skip the "head" region start
        .map(|p| p + 8) // a little into the body, still ASCII in this corpus
        .filter(|&p| p < perturbed.len())
        .expect("corpus must contain an ASCII body byte to perturb");
    let original = perturbed[pos];
    perturbed[pos] = if original == b'z' { b'a' } else { original + 1 };
    let perturbed = String::from_utf8(perturbed).expect("single-byte ASCII flip stays valid UTF-8");

    // Confirm the perturbation actually changed a byte (guards against a no-op flip).
    assert_ne!(perturbed, reference, "internal: perturbation must change the expected string");

    // THE ASSERTION UNDER TEST: comparing the true emit-concat to the perturbed
    // expectation MUST fail — the harness's byte-level equality is non-vacuous.
    assert_eq!(got, perturbed, "MUTATION WITNESS: perturbed byte must differ from the true emit-concat");
}

// ═══ §5.1 suites (sq-7d3dj.34.2.3) — the amended seed-lex order contract ═══════
//
// [FABLE-5] For shapes the streaming pipeline ADMITS, verification is per-shape
// SOLUTION equivalence vs the toggled-off buffered path, NOT a byte-diff (design
// record §5.1, decision sq-7d3dj.34.2.7). The fixtures are selective-seed fan-out
// graphs: the seed pattern is much smaller than 1/8 of the fan-out pattern's
// cardinality, so the GOO replay predicts a bind join and the classifier admits.
mod sect_5_1 {
    use super::PFX;
    use sparq_core::Graph;
    use sparq_engine::stream_pipeline_testing as spt;
    use sparq_engine::{query_json, query_json_stream_with_budget, QueryBudget};
    use std::collections::HashMap;

    /// Selective-seed fan-out fixture. `n_seed` seed triples `s_i ex:seed k_(i mod
    /// n_keys)`; each referenced key `k` has `fan` triples `k ex:val v`; `ballast`
    /// extra `ex:val` triples on UNREFERENCED subjects inflate the fan-out pattern's
    /// cardinality estimate (the bind-join admission inequality reads TOTAL predicate
    /// cardinality) without inflating the join result. Result rows = `n_seed × fan`.
    fn fanout_graph(n_seed: usize, n_keys: usize, fan: usize, ballast: usize) -> Graph {
        let mut nt = String::new();
        for i in 0..n_seed {
            nt.push_str(&format!("<http://ex/s/{}> <http://ex/seed> <http://ex/k/{}> .\n", i, i % n_keys));
        }
        for k in 0..n_keys {
            for j in 0..fan {
                nt.push_str(&format!("<http://ex/k/{}> <http://ex/val> <http://ex/v/{}_{}> .\n", k, k, j));
            }
        }
        for b in 0..ballast {
            nt.push_str(&format!("<http://ex/kb/{}> <http://ex/val> <http://ex/vb/{}> .\n", b, b));
        }
        Graph::load_str(&nt, "ntriples").expect("load fan-out graph")
    }

    /// The buffered reference: the SAME query with the streaming pipeline forced OFF
    /// on this thread — the historical buffered path.
    fn buffered(g: &Graph, q: &str) -> String {
        let prev = spt::set_enabled(false);
        let out = query_json(g, q).expect("buffered reference query error");
        spt::set_enabled(prev);
        out
    }

    /// The streamed document (live-sink front door) plus the per-query stats
    /// `(fired, blocks_run, seed_rows_processed, seed_rows_total)`.
    fn streamed(g: &Graph, q: &str) -> (String, (bool, usize, usize, usize)) {
        spt::reset_stats();
        let mut out = String::new();
        query_json_stream_with_budget(g, q, &QueryBudget::unlimited(), |c| {
            out.push_str(&c);
            std::ops::ControlFlow::Continue(())
        })
        .expect("streamed query error");
        (out, spt::stats())
    }

    /// Parses a SPARQL-results document into its solution list, each solution
    /// canonicalised to a string (serde_json object keys serialise in sorted order,
    /// so equal solutions canonicalise identically). Panics on invalid JSON — every
    /// caller therefore also asserts document validity.
    fn solutions(doc: &str) -> Vec<String> {
        let v: serde_json::Value = serde_json::from_str(doc).expect("response must be valid JSON");
        v["results"]["bindings"]
            .as_array()
            .expect("results.bindings must be an array")
            .iter()
            .map(|s| s.to_string())
            .collect()
    }

    /// The solution MULTISET: the canonicalised solutions, sorted.
    fn multiset(doc: &str) -> Vec<String> {
        let mut s = solutions(doc);
        s.sort();
        s
    }

    /// The head's projected-variable list (order-sensitive — head equality is part
    /// of every §5.1 contract).
    fn head_vars(doc: &str) -> Vec<String> {
        let v: serde_json::Value = serde_json::from_str(doc).expect("response must be valid JSON");
        v["head"]["vars"]
            .as_array()
            .expect("head.vars must be an array")
            .iter()
            .map(|x| x.as_str().expect("var name").to_string())
            .collect()
    }

    /// `true` when `sub` is a sub-multiset of `sup` (every solution occurs at most
    /// as often as in `sup`).
    fn is_sub_multiset(sub: &[String], sup: &[String]) -> bool {
        let mut counts: HashMap<&str, i64> = HashMap::new();
        for s in sup {
            *counts.entry(s.as_str()).or_insert(0) += 1;
        }
        sub.iter().all(|s| {
            let c = counts.entry(s.as_str()).or_insert(0);
            *c -= 1;
            *c >= 0
        })
    }

    /// RAII ramp override so a panicking assertion cannot leak the tiny testing ramp
    /// into a later query on this thread.
    struct RampGuard((usize, usize));
    impl RampGuard {
        fn set(first: usize, max: usize) -> Self {
            RampGuard(spt::set_block_ramp(first, max))
        }
    }
    impl Drop for RampGuard {
        fn drop(&mut self) {
            spt::set_block_ramp(self.0 .0, self.0 .1);
        }
    }

    /// The M1 bind-chain query over the fan-out fixture.
    fn chain_query() -> String {
        format!("{PFX}SELECT ?s ?v WHERE {{ ?s ex:seed ?k . ?k ex:val ?v }}")
    }

    /// (b) non-slice non-distinct → solution-MULTISET equality, on a TESTING-RAMP
    /// (first = 8) fixture where one join value's seed rows PROVABLY straddle a block
    /// boundary: 45 seed rows over 5 keys = 9 rows per key > the 8-row first block,
    /// so whichever key occupies the first block's rows continues into block 2
    /// (contiguously under the seed's ?k-sorted scan; interleaved every 5 rows under
    /// any other permutation — it straddles either way). Stats assert ≥ 2 blocks ran
    /// AND the streamed path was taken (multi-block non-vacuity — the 4000-row B2
    /// corpus fits one default block and masked the .2.7 bug).
    #[test]
    fn multiset_equality_with_block_boundary_straddle_testing_ramp() {
        let _ramp = RampGuard::set(8, 16);
        // est(seed) = 45; est(?k ex:val ?v) = 5×30 + 1000 = 1150 > 8×45 → bind join.
        let g = fanout_graph(45, 5, 30, 1000);
        let q = chain_query();

        let (got, (fired, blocks, _, seed_total)) = streamed(&g, &q);
        assert!(fired, "the streamed path must be taken for the admitted bind chain");
        assert!(blocks >= 2, "multi-block non-vacuity: expected >= 2 blocks, ran {blocks}");
        assert_eq!(seed_total, 45, "seed row count");

        let reference = buffered(&g, &q);
        assert_eq!(head_vars(&got), head_vars(&reference), "head must be identical");
        let (ms_got, ms_ref) = (multiset(&got), multiset(&reference));
        assert_eq!(ms_got.len(), 45 * 30, "expected n_seed x fan solutions");
        assert_eq!(ms_got, ms_ref, "streamed solutions must be MULTISET-equal to buffered");
    }

    /// (b) over a UNION of streamable branches (branch-sequential drivers).
    #[test]
    fn union_of_bind_chains_multiset_equality() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        let q = format!(
            "{PFX}SELECT ?x ?v WHERE {{ \
             {{ ?x ex:seed ?k . ?k ex:val ?v }} UNION {{ ?x ex:val ?v }} }}"
        );
        let (got, (fired, ..)) = streamed(&g, &q);
        assert!(fired, "the streamed path must be taken for the union of admitted branches");
        let reference = buffered(&g, &q);
        assert_eq!(head_vars(&got), head_vars(&reference));
        assert_eq!(multiset(&got), multiset(&reference), "union solutions must be MULTISET-equal");
    }

    /// (a) DISTINCT shapes → solution-SET equality + head + valid JSON. The
    /// projection is multi-variable (a single-variable DISTINCT belongs to the #1747
    /// skip-scan and is declined). First-seen dedup must also make the streamed
    /// output itself duplicate-free.
    #[test]
    fn distinct_set_equality() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        let q = format!("{PFX}SELECT DISTINCT ?k ?v WHERE {{ ?s ex:seed ?k . ?k ex:val ?v }}");

        let (got, (fired, blocks, ..)) = streamed(&g, &q);
        assert!(fired, "the streamed path must be taken");
        assert!(blocks >= 2, "the DISTINCT suite must also cross a block boundary");
        let reference = buffered(&g, &q);
        assert_eq!(head_vars(&got), head_vars(&reference));

        let sols = solutions(&got);
        let set_got: std::collections::BTreeSet<String> = sols.iter().cloned().collect();
        assert_eq!(sols.len(), set_got.len(), "streamed DISTINCT output must be duplicate-free");
        let set_ref: std::collections::BTreeSet<String> = solutions(&reference).into_iter().collect();
        assert_eq!(set_got, set_ref, "streamed DISTINCT solutions must be SET-equal to buffered");
        assert_eq!(set_got.len(), 5 * 30, "expected n_keys x fan distinct solutions");
    }

    /// (c) Slice → the streamed solutions are a SUB-MULTISET of the UN-sliced
    /// buffered result with the buffered sliced count. Under a different legal order
    /// LIMIT legitimately selects different rows, so bag-equality against the sliced
    /// buffered output would be a false invariant.
    #[test]
    fn slice_is_sub_multiset_of_unsliced_with_buffered_count() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        let unsliced = chain_query();
        let sliced = format!("{} LIMIT 100 OFFSET 7", unsliced);

        let (got, (fired, ..)) = streamed(&g, &sliced);
        assert!(fired, "the streamed path must be taken for the sliced chain");
        let sub = multiset(&got);
        let sup = multiset(&buffered(&g, &unsliced));
        let buffered_sliced_count = solutions(&buffered(&g, &sliced)).len();
        assert_eq!(sub.len(), buffered_sliced_count, "streamed slice must have the buffered sliced count");
        assert_eq!(sub.len(), 100, "sanity: OFFSET 7 LIMIT 100 of 1350 rows");
        assert!(is_sub_multiset(&sub, &sup), "streamed slice must be a sub-multiset of the un-sliced buffered result");
    }

    /// (d) determinism — the streamed path run twice is byte-identical to itself
    /// (§5.1: the seed-lex order is deterministic, unlike a hash-bucket order).
    #[test]
    fn streamed_double_run_is_byte_identical_to_itself() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        for q in [chain_query(), format!("{PFX}SELECT DISTINCT ?k ?v WHERE {{ ?s ex:seed ?k . ?k ex:val ?v }}")] {
            let (run1, (fired, ..)) = streamed(&g, &q);
            let (run2, _) = streamed(&g, &q);
            assert!(fired, "determinism suite must exercise the streamed path");
            assert_eq!(run1, run2, "streamed emission must be deterministic (double-run byte-identical): {q}");
        }
    }

    /// Default-ramp fixture with seed > 4096 rows (STREAM_BLOCK_FIRST), so the
    /// PRODUCTION ramp constants also demonstrably multi-block. est(seed) = 4500;
    /// est(?k ex:val ?v) = 100×2 + 40000 = 40200 > 8×4500 → bind join admitted.
    #[test]
    fn default_ramp_seed_over_first_block_multiset_equality() {
        // No ramp override: production constants (4096 doubling to 65536).
        let g = fanout_graph(4500, 100, 2, 40_000);
        let q = chain_query();
        let (got, (fired, blocks, _, seed_total)) = streamed(&g, &q);
        assert!(fired, "the streamed path must be taken under the default ramp");
        assert_eq!(seed_total, 4500);
        assert!(blocks >= 2, "a 4500-row seed must span >= 2 default-ramp blocks, ran {blocks}");
        assert_eq!(multiset(&got), multiset(&buffered(&g, &q)), "default-ramp multiset equality");
    }

    /// The TTFS probe: an emitted chunk is observed WHILE processed seed rows <
    /// total seed rows — i.e. first solutions leave the engine before the join (or
    /// even the seed scan) completes. The 8-row first block fans out to 1600 rows
    /// (> 64 KiB of JSON, the public entry points' flush threshold), so the first
    /// chunk is handed to the sink during block 1 of 4+.
    #[test]
    fn first_emit_observed_before_seed_scan_complete() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 200, 2000);
        let q = chain_query();
        spt::reset_stats();
        let mut first_chunk_stats: Option<(bool, usize, usize, usize)> = None;
        let mut chunks = 0usize;
        query_json_stream_with_budget(&g, &q, &QueryBudget::unlimited(), |_c| {
            if first_chunk_stats.is_none() {
                first_chunk_stats = Some(spt::stats());
            }
            chunks += 1;
            std::ops::ControlFlow::Continue(())
        })
        .expect("probe query error");
        let (fired, _, processed, total) = first_chunk_stats.expect("at least one chunk must be emitted");
        assert!(fired, "the probe must exercise the streamed path");
        assert!(chunks >= 2, "the probe corpus must be genuinely multi-chunk (got {chunks})");
        assert!(
            processed < total,
            "first emit must be observed BEFORE the seed scan completes (processed {processed} of {total})"
        );
    }

    /// (e) fallback shapes → the buffered path is taken (testing-stats assertion,
    /// not assumption) and stays byte-identical to the pipeline-disabled reference.
    #[test]
    fn fallback_shapes_take_buffered_path_byte_identical() {
        let g = fanout_graph(45, 5, 30, 1000);
        let cases = [
            // ORDER BY: a pipeline breaker at the top — never admitted (v1).
            format!("{PFX}SELECT ?s ?v WHERE {{ ?s ex:seed ?k . ?k ex:val ?v }} ORDER BY ?v ?s"),
            // OPTIONAL: not a flattenable conjunction.
            format!("{PFX}SELECT ?s ?v WHERE {{ ?s ex:seed ?k OPTIONAL {{ ?k ex:val ?v }} }}"),
            // Two connecting variables: the GOO replay predicts a hash step (M1
            // admits single-variable bind joins only).
            format!("{PFX}SELECT ?a ?k WHERE {{ ?a ex:seed ?k . ?a ex:val ?k }}"),
            // Cyclic (triangle) BGP: routed to the WCOJ plan, never the binary chain.
            format!("{PFX}SELECT ?a ?b WHERE {{ ?a ex:seed ?k . ?a ex:seed ?b . ?b ex:val ?k }}"),
            // Equal-cardinality patterns: the bind-join inequality (8x) fails, so a
            // hash/merge step is predicted — the q04 class stays buffered until M2.
            format!("{PFX}SELECT DISTINCT ?k ?v WHERE {{ ?s ex:val ?k . ?s ex:val ?v }}"),
        ];
        for q in &cases {
            let (got, (fired, blocks, ..)) = streamed(&g, q);
            assert!(!fired, "fallback shape must NOT take the streamed path: {q}");
            assert_eq!(blocks, 0, "fallback shape must run zero stream blocks: {q}");
            let reference = buffered(&g, q);
            assert_eq!(got, reference, "fallback shape must stay byte-identical to the buffered path: {q}");
        }
    }

    /// Admitted-shape edges: a provably-absent constant (unsatisfiable branch) and
    /// `LIMIT 0` both emit the empty document byte-identically to the buffered path.
    #[test]
    fn unsat_branch_and_limit_zero_emit_empty_document() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        for q in [
            // ex:nope is absent from the dictionary → the branch is unsatisfiable.
            format!("{PFX}SELECT ?s ?v WHERE {{ ?s ex:seed ?k . ?k ex:nope ?v }}"),
            format!("{} LIMIT 0", chain_query()),
        ] {
            let (got, _) = streamed(&g, &q);
            assert!(solutions(&got).is_empty(), "expected zero solutions: {q}");
            assert_eq!(got, buffered(&g, &q), "empty document must be byte-identical to buffered: {q}");
        }
    }

    /// `ControlFlow::Break` from the sink (the HTTP client disconnected) aborts the
    /// driver cleanly: no error, and the remaining chunks are abandoned (the
    /// document is left unterminated — the server's truncation floor).
    #[test]
    fn sink_break_aborts_streamed_driver() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 200, 2000);
        spt::reset_stats();
        let mut got = String::new();
        let mut chunks = 0usize;
        query_json_stream_with_budget(&g, &chain_query(), &QueryBudget::unlimited(), |c| {
            got.push_str(&c);
            chunks += 1;
            std::ops::ControlFlow::Break(())
        })
        .expect("a sink Break is a clean early stop, not an error");
        let (fired, ..) = spt::stats();
        assert!(fired, "the Break probe must exercise the streamed path");
        assert_eq!(chunks, 1, "no further chunk may follow the Break");
        assert!(!got.ends_with("]}}"), "the aborted stream must be left unterminated");
    }

    /// SOLUTION MUTATION WITNESS (the .2.3 acceptance's non-vacuity check): perturb
    /// one SOLUTION of the expected multiset and assert the multiset check goes RED —
    /// proving empirically that the §5.1 solution-equivalence suites can catch a
    /// wrong/missing solution (not just a wrong byte). `#[should_panic]` inverts it.
    #[test]
    #[should_panic(expected = "SOLUTION MUTATION WITNESS")]
    fn perturbed_solution_goes_red_under_multiset_check() {
        let _ramp = RampGuard::set(8, 16);
        let g = fanout_graph(45, 5, 30, 1000);
        let q = chain_query();
        let (got, (fired, ..)) = streamed(&g, &q);
        assert!(fired, "pre-condition: the streamed path must be taken");
        let ms_got = multiset(&got);
        let mut ms_expected = multiset(&buffered(&g, &q));
        // Sanity: unperturbed they ARE equal, so the failure below is caused ONLY by
        // the deliberate solution perturbation.
        assert_eq!(ms_got, ms_expected, "pre-condition: unperturbed multisets must match");
        // Perturb exactly one SOLUTION: swap a real binding for one that cannot occur.
        ms_expected[0] = "{\"s\":{\"type\":\"uri\",\"value\":\"http://ex/PERTURBED\"}}".to_string();
        ms_expected.sort();
        assert_ne!(ms_got, ms_expected, "internal: the perturbation must change the multiset");
        // THE ASSERTION UNDER TEST: the multiset equality MUST fail.
        assert_eq!(ms_got, ms_expected, "SOLUTION MUTATION WITNESS: perturbed solution must differ");
    }
}
