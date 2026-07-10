//! Acceptance / witness tests for the correlated (theta) anti-join — the SPARQL
//! negation idiom `OPTIONAL { … FILTER(F) } FILTER(!bound(?nb))` with an OPTIONAL
//! condition `F` that references OUTER variables (SP2Bench q06). [FABLE-5]
//! (bead sq-7d3dj.30.9)
//!
//! The load-bearing invariant is BAG-SEMANTICS RESULT-EQUIVALENCE with the fast
//! path forced OFF (the cold `Filter{LeftJoin}` plan) vs ON. Every semantic trap
//! from the bead brief has a dedicated witness:
//!
//!   * `q06_shape_equivalence` / `q06_shape_anti_vacuity` — the SP2Bench q06 shape
//!     (documents with no earlier document by the same author): on == off, and the
//!     path FIRED with a collapsed correlated right side.
//!   * `theta_type_error_survives` — a residual `?yr2 < ?yr` that TYPE-ERRORs (a
//!     non-numeric year) must NOT eliminate the outer row (error ⇒ no match).
//!   * `literal_correlation_value_correct` — a correlation var bound to a LITERAL
//!     (the sq-lr2ii value-equality class) must go through the value-correct path,
//!     never the IRI id-seeding fast path; on == off.
//!   * `decline_nb_in_left` — `?nb` also bound on the LEFT side ⇒ not an anti-join
//!     predicate ⇒ decline; the plan is unchanged and correct.
//!   * `multiplicity_preserved` — a surviving outer row with duplicates is emitted
//!     with its original multiplicity, never multiplied by right non-matches.
//!   * `no_correlation_declines` — an OPTIONAL condition with no seedable equality
//!     falls back to the cold plan (still correct).
//!   * `nested_optional_equivalence` — an outer anti-join whose left side itself
//!     contains an OPTIONAL: on == off.
//!   * `randomised_differential` — randomised graphs (correlated + uncorrelated,
//!     error-producing literals) fuzzed against the cold path.
//!   * `public_toggle_and_stats` — the `theta_antijoin_testing` public surface.

use sparq_core::Graph;
use sparq_engine::{query, theta_antijoin_testing};

const PFX: &str = "PREFIX ex: <http://example.org/>\n\
                   PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
                   PREFIX dc: <http://purl.org/dc/elements/1.1/>\n\
                   PREFIX dcterms: <http://purl.org/dc/terms/>\n\
                   PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
                   PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
                   PREFIX xsd: <http://www.w3.org/2001/XMLSchema#>\n";

fn load(ttl: &str) -> Graph {
    Graph::load_str(&format!("{PFX}{ttl}"), "turtle").expect("test graph")
}

type Table = Vec<Vec<Option<String>>>;

/// Result rows as a SORTED multiset of stringified cells (order-insensitive but
/// MULTIPLICITY-preserving — the bag-semantics oracle).
fn multiset(g: &Graph, q: &str) -> Table {
    let r = query(g, &format!("{PFX}{q}")).expect("query failed");
    let mut rows: Vec<Vec<Option<String>>> = r
        .rows
        .iter()
        .map(|row| row.iter().map(|c| c.as_ref().map(|t| t.to_string())).collect())
        .collect();
    rows.sort();
    rows
}

/// Runs `q` with the theta anti-join forced OFF then ON, returning both multisets.
fn on_off(g: &Graph, q: &str) -> (Table, Table) {
    let prev = theta_antijoin_testing::set_enabled(false);
    let off = multiset(g, q);
    theta_antijoin_testing::set_enabled(true);
    let on = multiset(g, q);
    theta_antijoin_testing::set_enabled(prev);
    (off, on)
}

// ── q06 dataset ───────────────────────────────────────────────────────────────
//
// The SP2Bench q06 shape: articles with an issue year and a creator; the query
// finds documents that have NO EARLIER document by the SAME author. Authors are
// IRIs (so the correlation seeds sideways as an IRI); a share of documents are the
// author's earliest (they survive the anti-join).
const Q06_BODY: &str = "\
  ?class rdfs:subClassOf foaf:Document .
  ?document rdf:type ?class .
  ?document dcterms:issued ?yr .
  ?document dc:creator ?author .
  ?author foaf:name ?name
  OPTIONAL {
    ?class2 rdfs:subClassOf foaf:Document .
    ?document2 rdf:type ?class2 .
    ?document2 dcterms:issued ?yr2 .
    ?document2 dc:creator ?author2
    FILTER (?author = ?author2 && ?yr2 < ?yr)
  } FILTER (!bound(?author2))
";

fn q06_dataset() -> Graph {
    let mut ttl = String::from(
        "ex:Article rdfs:subClassOf foaf:Document .\n\
         ex:Book rdfs:subClassOf foaf:Document .\n",
    );
    // 5 authors, each with several documents across distinct years.
    for a in 0..5usize {
        ttl.push_str(&format!("ex:a{a} foaf:name \"Author {a}\" .\n"));
        // Years 2000 + offsets; author a has (a+2) documents.
        for (i, yr) in (0..(a + 2)).map(|i| (i, 2000 + a * 3 + i)).collect::<Vec<_>>() {
            let cls = if i % 2 == 0 { "ex:Article" } else { "ex:Book" };
            ttl.push_str(&format!(
                "ex:d{a}_{i} rdf:type {cls} .\n\
                 ex:d{a}_{i} dcterms:issued \"{yr}\"^^xsd:integer .\n\
                 ex:d{a}_{i} dc:creator ex:a{a} .\n"
            ));
        }
    }
    load(&ttl)
}

#[test]
fn q06_shape_equivalence() {
    let g = q06_dataset();
    // The real q06 projection.
    let (off, on) = on_off(&g, &format!("SELECT ?yr ?name ?document WHERE {{\n{Q06_BODY}\n}}"));
    assert_eq!(off, on, "q06 result differs with theta anti-join on vs off");
    assert!(!on.is_empty(), "q06 fixture must return the earliest document per author");
    // Exactly one earliest document per author (5 authors), each of whom has ≥2 docs.
    assert_eq!(on.len(), 5, "expected one surviving (earliest) document per author");
}

#[test]
fn q06_shape_anti_vacuity() {
    let g = q06_dataset();
    let sel = format!("SELECT ?yr ?name ?document WHERE {{\n{Q06_BODY}\n}}");
    theta_antijoin_testing::set_enabled(true);
    theta_antijoin_testing::reset_stats();
    let _ = query(&g, &format!("{PFX}{sel}")).expect("query");
    let (fired, correlated_rows, bindings) = theta_antijoin_testing::stats();
    assert!(fired, "theta anti-join did not fire on the q06 shape");
    assert!(bindings >= 1, "no distinct correlations were evaluated");
    // Anti-vacuity: seeding COLLAPSED the correlated right side. Each of the
    // `bindings` correlations seeds `?author2 := <that author>` so it returns only
    // that author's documents; a BLIND (unseeded) evaluation of the same right side
    // would return the whole 20-document `class ∩ issued ∩ creator` relation for
    // EVERY correlation, i.e. `bindings * 20`. The seeded total must be far smaller.
    let blind_total = bindings * 20;
    assert!(
        correlated_rows * 2 < blind_total,
        "correlated right side did not collapse: correlated={} blind={}",
        correlated_rows,
        blind_total
    );
}

#[test]
fn theta_type_error_survives() {
    // A residual `?yr2 < ?yr` where one document's year is a NON-NUMERIC literal:
    // the comparison TYPE-ERRORs, so that inner row does NOT match, so the outer row
    // must SURVIVE the anti-join — never be eliminated by an error. on == off pins it.
    let g = load(
        "ex:Article rdfs:subClassOf foaf:Document .
         ex:a0 foaf:name \"A\" .
         ex:d0 rdf:type ex:Article . ex:d0 dcterms:issued \"2001\"^^xsd:integer . ex:d0 dc:creator ex:a0 .
         ex:d1 rdf:type ex:Article . ex:d1 dcterms:issued \"not-a-year\" . ex:d1 dc:creator ex:a0 .",
    );
    let (off, on) = on_off(&g, &format!("SELECT ?document ?yr WHERE {{\n{Q06_BODY}\n}}"));
    assert_eq!(off, on, "type-error residual eliminated a row (must survive)");
    // d0 (year 2001) has no EARLIER same-author doc that compares numerically (d1's
    // year is non-numeric → error → no match) so d0 survives; d1 (non-numeric year)
    // also survives (its ?yr errors in every comparison). Both must be present.
    assert_eq!(on.len(), 2, "both documents must survive the error-producing residual");
}

#[test]
fn literal_correlation_value_correct() {
    // The correlation variable binds a LITERAL, not an IRI. The value-equality class
    // (sq-lr2ii): `"1"^^xsd:integer` and `"01"^^xsd:integer` are `=` but NOT
    // sameTerm. The IRI id-seeding fast path would MISS the value-equal-but-not-
    // identical partner and spuriously KEEP a row; the value-correct path must drop
    // it. Correlate on a shared integer KEY that appears in two lexical forms.
    let g = load(
        "ex:d0 ex:key \"1\"^^xsd:integer .   ex:d0 ex:rank \"5\"^^xsd:integer .
         ex:d1 ex:key \"01\"^^xsd:integer .  ex:d1 ex:rank \"3\"^^xsd:integer .",
    );
    // Anti-join: a document with NO other document of the SAME key value and a lower
    // rank. d0 (key=1, rank=5) and d1 (key=01≡1, rank=3): d1's rank 3 < d0's rank 5,
    // and key 01 = key 1 by value, so d0 is eliminated (there IS an earlier one). d1
    // survives (no key-equal doc with rank < 3). A term-identity probe on "1" vs "01"
    // would wrongly keep d0.
    let q = "SELECT ?d ?k WHERE {
        ?d ex:key ?k . ?d ex:rank ?r
        OPTIONAL {
          ?d2 ex:key ?k2 . ?d2 ex:rank ?r2
          FILTER (?k = ?k2 && ?r2 < ?r)
        } FILTER(!bound(?k2))
    }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "literal value-equality correlation differs on vs off");
    // Only d1 survives (its rank is the minimum for key value 1).
    assert_eq!(on.len(), 1, "exactly one document (d1) survives the value-correct anti-join");
    assert_eq!(on[0][0].as_deref(), Some("<http://example.org/d1>"));
}

#[test]
fn decline_nb_in_left() {
    // `?author2` (the !bound witness) is ALSO bound on the LEFT side. Then
    // `!bound(?author2)` is NOT the anti-join predicate — the shape must DECLINE and
    // the plain `Filter{LeftJoin}` plan must run. on == off proves equivalence.
    let g = load(
        "ex:Article rdfs:subClassOf foaf:Document .
         ex:a0 foaf:name \"A\" .
         ex:d0 rdf:type ex:Article . ex:d0 dcterms:issued \"2001\"^^xsd:integer . ex:d0 dc:creator ex:a0 .
         ex:d0 ex:coauthor ex:a0 .",
    );
    // ?author2 appears on the left via `?document ex:coauthor ?author2` — so it is
    // certainly bound before the OPTIONAL, and `!bound(?author2)` is never true.
    let q = "SELECT ?document WHERE {
        ?class rdfs:subClassOf foaf:Document .
        ?document rdf:type ?class .
        ?document dcterms:issued ?yr .
        ?document dc:creator ?author .
        ?document ex:coauthor ?author2 .
        ?author foaf:name ?name
        OPTIONAL {
          ?document2 dcterms:issued ?yr2 .
          ?document2 dc:creator ?author2
          FILTER (?author = ?author2 && ?yr2 < ?yr)
        } FILTER(!bound(?author2))
    }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "decline path (?nb bound on left) differs on vs off");
    // ?author2 is always bound (coauthor), so !bound is never satisfied → no rows.
    assert!(on.is_empty(), "!bound(?author2) can never hold when ?author2 is left-bound");
}

#[test]
fn multiplicity_preserved() {
    // A surviving outer row that has DUPLICATE left multiplicity (via a UNION of two
    // identical branches) must be emitted exactly that many times — the anti-join
    // never multiplies by (or collapses due to) right non-matches.
    let g = load(
        "ex:a0 foaf:name \"A\" .
         ex:d0 ex:tag ex:t . ex:d0 dcterms:issued \"2001\"^^xsd:integer . ex:d0 dc:creator ex:a0 .",
    );
    let q = "SELECT ?document WHERE {
        { { ?document ex:tag ex:t } UNION { ?document ex:tag ex:t } }
        ?document dcterms:issued ?yr .
        ?document dc:creator ?author
        OPTIONAL {
          ?document2 dcterms:issued ?yr2 .
          ?document2 dc:creator ?author2
          FILTER (?author = ?author2 && ?yr2 < ?yr)
        } FILTER(!bound(?author2))
    }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "multiplicity differs on vs off");
    // Two identical UNION branches → the single surviving document appears TWICE.
    assert_eq!(on.len(), 2, "duplicate left multiplicity not preserved");
}

#[test]
fn no_correlation_declines() {
    // An OPTIONAL condition with NO seedable `?outer = ?inner` equality — only an
    // inequality that does not correlate. The path declines (no SIP to do); the cold
    // plan must produce the correct anti-join answer regardless. on == off.
    let g = load(
        "ex:a0 foaf:name \"A\" .
         ex:d0 dcterms:issued \"2001\"^^xsd:integer . ex:d0 dc:creator ex:a0 .
         ex:d1 dcterms:issued \"1999\"^^xsd:integer . ex:d1 dc:creator ex:a0 .",
    );
    let q = "SELECT ?document WHERE {
        ?document dcterms:issued ?yr . ?document dc:creator ?author
        OPTIONAL {
          ?document2 dcterms:issued ?yr2
          FILTER (?yr2 < ?yr)
        } FILTER(!bound(?document2))
    }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "no-correlation decline path differs on vs off");
}

#[test]
fn nested_optional_equivalence() {
    // The outer anti-join's LEFT side itself contains an OPTIONAL — the recogniser
    // evaluates the left verbatim, so nesting must stay equivalent. on == off.
    let g = q06_dataset();
    let q = "SELECT ?document ?name ?extra WHERE {
           ?class rdfs:subClassOf foaf:Document .
           ?document rdf:type ?class .
           ?document dcterms:issued ?yr .
           ?document dc:creator ?author .
           ?author foaf:name ?name
           OPTIONAL { ?document ex:extra ?extra }
           OPTIONAL {
             ?class2 rdfs:subClassOf foaf:Document .
             ?document2 rdf:type ?class2 .
             ?document2 dcterms:issued ?yr2 .
             ?document2 dc:creator ?author2
             FILTER (?author = ?author2 && ?yr2 < ?yr)
           } FILTER(!bound(?author2))
         }";
    let (off, on) = on_off(&g, q);
    assert_eq!(off, on, "nested-OPTIONAL left side differs on vs off");
}

#[test]
fn partially_bound_shared_var_fill_through() {
    // WITNESS for the reviewer's counterexample (PR #1786): a shared join var `?s` is
    // bound only PARTIALLY on the left (an OPTIONAL inside A leaves it unbound for
    // `ex:d0`) but is CERTAIN in B, and the residual references it via `BOUND(?s)`.
    //
    // The cold `left_outer_join` builds the condition row via `merge_rows`, which FILLS
    // the left-unbound `?s` from the right, so `BOUND(?s)` is true, the inner matches,
    // and `ex:d0` is ELIMINATED (0 rows). The fast path previously sourced `?s` from the
    // left ONLY, saw UNBOUND → error → no match → `ex:d0` SPURIOUSLY SURVIVED (1 row).
    // The `MergeSrc::LeftThenRight` fill-through makes the fast path mirror `merge_rows`;
    // on == off pins it. This test is RED on the pre-fix code (fast=1 row, cold=0). It is
    // load-bearing that the fast path actually FIRES on this shape (asserted below).
    let g = load(
        "ex:d0 ex:p ex:val0 .
         ex:x0 ex:owner ex:s_any . ex:x0 ex:val ex:val0 . ex:x0 ex:mark ex:m .",
    );
    let q = "SELECT ?d WHERE {
        ?d ex:p ?v .
        OPTIONAL { ?d ex:q ?s }                # ?s partially bound in A (unbound for ex:d0)
        OPTIONAL {
          ?x ex:owner ?s .                     # ?s shared with A, certain in B
          ?x ex:val ?inner .
          ?x ex:mark ?nb .
          FILTER (?v = ?inner && BOUND(?s))    # residual references the shared var
        } FILTER(!bound(?nb))
      }";
    let (off, on) = on_off(&g, q);
    assert_eq!(
        off, on,
        "partially-bound shared var: fast path diverged from cold (fill-through bug)"
    );
    // The cold plan eliminates ex:d0 (fill-through makes BOUND(?s) true) ⇒ 0 rows.
    assert!(off.is_empty(), "cold plan must eliminate ex:d0 (0 rows), got {:?}", off);
    // And the fast path must have actually FIRED on this shape (otherwise the test is
    // vacuous — it would trivially match the cold path by falling back).
    theta_antijoin_testing::set_enabled(true);
    theta_antijoin_testing::reset_stats();
    let _ = query(&g, &format!("{PFX}{q}")).expect("query");
    let (fired, _rows, _bindings) = theta_antijoin_testing::stats();
    assert!(fired, "theta anti-join did not fire on the partially-bound-shared-var shape");
}

#[test]
fn hash_path_large_cardinality_equivalence() {
    // > SIP_MAX_SMALL_ROWS (64) distinct IRI correlations forces the HASH anti-join
    // strategy (evaluate B once, partition by ?inner). Must equal the cold path AND
    // the SIP-seed path. 200 authors, each with 3 documents at distinct years.
    let mut ttl = String::from("ex:Article rdfs:subClassOf foaf:Document .\n");
    for a in 0..200usize {
        ttl.push_str(&format!("ex:a{} foaf:name \"A{}\" .\n", a, a));
        for i in 0..3usize {
            let yr = 2000 + i;
            ttl.push_str(&format!(
                "ex:d{}_{} rdf:type ex:Article . ex:d{}_{} dcterms:issued \"{}\"^^xsd:integer . ex:d{}_{} dc:creator ex:a{} .\n",
                a, i, a, i, yr, a, i, a
            ));
        }
    }
    let g = load(&ttl);
    let (off, on) = on_off(&g, &format!("SELECT ?document ?yr ?name WHERE {{\n{Q06_BODY}\n}}"));
    assert_eq!(off, on, "hash-path large-cardinality differs from cold path");
    // One earliest document per author.
    assert_eq!(on.len(), 200, "one surviving earliest document per author");
    // Confirm the hash path actually fired (>64 distinct correlations).
    theta_antijoin_testing::set_enabled(true);
    theta_antijoin_testing::reset_stats();
    let _ = query(&g, &format!("{PFX}SELECT ?document WHERE {{\n{Q06_BODY}\n}}")).unwrap();
    let (fired, _rows, bindings) = theta_antijoin_testing::stats();
    assert!(fired && bindings > 64, "expected the hash strategy (>64 correlations): {}", bindings);
}

#[test]
fn blank_node_correlation_equivalence() {
    // The REAL SP2Bench q06 shape correlates on `?author`, which is a BLANK NODE (the
    // dataset's `dc:creator` targets are mostly blank-node persons). Per SPARQL 1.1
    // §17.4.1.7 (RDFterm-equal), `=` on two DISTINCT non-literal terms (here blank nodes)
    // returns FALSE — a type error arises only when BOTH arguments are literals — and
    // FALSE and error both mean NO MATCH under the anti-join, so an anti-join match
    // requires the SAME blank node. The id-bucket probe reproduces this exactly (same id =
    // match; different id = no match = survive). Uses >64 distinct authors → the hash path.
    let mut ttl = String::from("ex:Article rdfs:subClassOf foaf:Document .\n");
    for a in 0..80usize {
        // Each author is a BLANK NODE with a name; give each 2 documents at years a, a+1.
        for i in 0..2usize {
            let yr = 2000 + i;
            ttl.push_str(&format!(
                "ex:d{}_{} rdf:type ex:Article . ex:d{}_{} dcterms:issued \"{}\"^^xsd:integer . ex:d{}_{} dc:creator _:auth{} .\n",
                a, i, a, i, yr, a, i, a
            ));
        }
        ttl.push_str(&format!("_:auth{} foaf:name \"Author {}\" .\n", a, a));
    }
    let g = load(&ttl);
    let (off, on) = on_off(&g, &format!("SELECT ?document ?yr ?name WHERE {{\n{Q06_BODY}\n}}"));
    assert_eq!(off, on, "blank-node correlation differs on vs off");
    // One earliest document per author (80 authors).
    assert_eq!(on.len(), 80, "one surviving earliest document per blank-node author");
    // The hash path with blank-node keys fired.
    theta_antijoin_testing::set_enabled(true);
    theta_antijoin_testing::reset_stats();
    let _ = query(&g, &format!("{PFX}SELECT ?document WHERE {{\n{Q06_BODY}\n}}")).unwrap();
    let (fired, _rows, bindings) = theta_antijoin_testing::stats();
    assert!(fired && bindings > 64, "expected hash path on blank-node keys: {}", bindings);
}

#[test]
fn randomised_differential() {
    // Fuzz randomised graphs (deterministic LCG) against the cold path: mix IRI
    // correlations, a literal-keyed variant, and error-producing (non-numeric) years.
    let mut state: u64 = 0x9E3779B97F4A7C15;
    let mut next = |m: u64| {
        state = state.wrapping_mul(6364136223846793005).wrapping_add(1442695040888963407);
        (state >> 33) % m
    };
    for trial in 0..48u64 {
        // Alternate between the SIP-seed regime (few authors) and the hash regime
        // (> 64 authors), and between IRI-typed and BLANK-NODE authors, so both
        // strategies AND both id-hashable term kinds are fuzzed against the cold path.
        let big = trial % 2 == 0;
        let bnode_authors = trial % 3 == 0;
        // Every 4th trial ALSO adds a PARTIALLY-BOUND shared var `?tag` (bound on only
        // some documents via an OPTIONAL in A, certain in B) referenced by the residual
        // via `BOUND(?tag)` — the failing class the pre-fix generator could not produce
        // (left-unbound shared var read by the residual). [FABLE-5]
        let shared_var = trial % 4 == 0;
        let n_auth = if big { 70 + next(30) as usize } else { 1 + next(4) as usize };
        let mut ttl = String::from("ex:Article rdfs:subClassOf foaf:Document .\n");
        let author = |a: usize| {
            if bnode_authors {
                format!("_:a{}", a)
            } else {
                format!("ex:a{}", a)
            }
        };
        for a in 0..n_auth {
            ttl.push_str(&format!("{} foaf:name \"A{}\" .\n", author(a), a));
        }
        let n_doc = 1 + next(if big { 200 } else { 8 }) as usize;
        for d in 0..n_doc {
            let a = next(n_auth as u64) as usize;
            ttl.push_str(&format!(
                "ex:d{} rdf:type ex:Article . ex:d{} dc:creator {} .\n",
                d, d, author(a)
            ));
            // 1 in 6 documents gets a non-numeric year (error-producing residual).
            if next(6) == 0 {
                ttl.push_str(&format!("ex:d{} dcterms:issued \"bad{}\" .\n", d, d));
            } else {
                let yr = 1990 + next(20);
                ttl.push_str(&format!("ex:d{} dcterms:issued \"{}\"^^xsd:integer .\n", d, yr));
            }
            // For the shared-var variant, bind `?tag` on ~2/3 of documents (an IRI). The
            // OPTIONAL in A leaves it UNBOUND on the rest, so the merged condition row
            // must fill it from the right to reproduce the cold semantics. [FABLE-5]
            if shared_var && next(3) != 0 {
                ttl.push_str(&format!("ex:d{} ex:tag ex:t{} .\n", d, next(3)));
            }
        }
        let g = load(&ttl);
        // The shared-var body adds `OPTIONAL { ?document ex:tag ?tag }` in A and a
        // certain `?document2 ex:tag ?tag` + a `BOUND(?tag)` conjunct in the residual,
        // so a left-unbound `?tag` is READ by the residual (the counterexample class).
        let body = if shared_var {
            "\
  ?class rdfs:subClassOf foaf:Document .
  ?document rdf:type ?class .
  ?document dcterms:issued ?yr .
  ?document dc:creator ?author .
  ?author foaf:name ?name
  OPTIONAL { ?document ex:tag ?tag }
  OPTIONAL {
    ?class2 rdfs:subClassOf foaf:Document .
    ?document2 rdf:type ?class2 .
    ?document2 dcterms:issued ?yr2 .
    ?document2 dc:creator ?author2 .
    ?document2 ex:tag ?tag
    FILTER (?author = ?author2 && ?yr2 < ?yr && BOUND(?tag))
  } FILTER (!bound(?author2))
"
        } else {
            Q06_BODY
        };
        let q = format!("SELECT ?document ?yr ?name ?tag WHERE {{\n{body}\n}}");
        let (off, on) = on_off(&g, &q);
        assert_eq!(off, on, "randomised differential mismatch on trial {}", trial);
    }
}

/// Work-box (NON-CANONICAL) before/after measurement of SP2Bench q06 on the 250k
/// dataset. Ignored by default (needs the dataset); run with the path in `SP2B_250K`:
///   SP2B_250K=/tmp/sp2b/sp2b-250000.ttl cargo test -p sparq-engine --release \
///     --test theta_antijoin -- --ignored --nocapture q06_measure_250k
#[test]
#[ignore]
fn q06_measure_250k() {
    let path = std::env::var("SP2B_250K").expect("set SP2B_250K to the 250k .ttl path");
    let ttl = std::fs::read_to_string(&path).expect("read dataset");
    let t_load = std::time::Instant::now();
    let g = Graph::load_str(&ttl, "turtle").expect("load graph");
    eprintln!("loaded {} in {:?}", path, t_load.elapsed());
    let q06 = "PREFIX rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#>\n\
               PREFIX rdfs: <http://www.w3.org/2000/01/rdf-schema#>\n\
               PREFIX foaf: <http://xmlns.com/foaf/0.1/>\n\
               PREFIX dc: <http://purl.org/dc/elements/1.1/>\n\
               PREFIX dcterms: <http://purl.org/dc/terms/>\n\
               SELECT ?yr ?name ?document WHERE {\n\
                 ?class rdfs:subClassOf foaf:Document .\n\
                 ?document rdf:type ?class .\n\
                 ?document dcterms:issued ?yr .\n\
                 ?document dc:creator ?author .\n\
                 ?author foaf:name ?name\n\
                 OPTIONAL {\n\
                   ?class2 rdfs:subClassOf foaf:Document .\n\
                   ?document2 rdf:type ?class2 .\n\
                   ?document2 dcterms:issued ?yr2 .\n\
                   ?document2 dc:creator ?author2\n\
                   FILTER (?author=?author2 && ?yr2<?yr)\n\
                 } FILTER (!bound(?author2))\n\
               }";

    theta_antijoin_testing::set_enabled(false);
    let t = std::time::Instant::now();
    let off = sparq_engine::query(&g, q06).expect("q06 off");
    let d_off = t.elapsed();

    theta_antijoin_testing::set_enabled(true);
    theta_antijoin_testing::reset_stats();
    let t = std::time::Instant::now();
    let on = sparq_engine::query(&g, q06).expect("q06 on");
    let d_on = t.elapsed();
    let (fired, child_rows, bindings) = theta_antijoin_testing::stats();
    eprintln!("theta stats: fired={} child_rows={} correlations={}", fired, child_rows, bindings);
    eprintln!("q06 rows: off={} on={}", off.rows.len(), on.rows.len());
    eprintln!("q06 OFF (cold left_outer_join): {:?}", d_off);
    eprintln!("q06 ON  (correlated theta anti-join): {:?}", d_on);
    assert_eq!(off.rows.len(), on.rows.len(), "q06 row count must be identical on vs off");
}

#[test]
fn public_toggle_and_stats() {
    // Direct unit coverage of the public `theta_antijoin_testing` surface (one direct
    // test per new public fn, for the coverage ratchet).
    let prev = theta_antijoin_testing::set_enabled(false);
    assert!(!theta_antijoin_testing::set_enabled(true), "set_enabled returns the prior value");
    assert!(theta_antijoin_testing::set_enabled(prev));

    theta_antijoin_testing::reset_stats();
    let (fired, rows, bindings) = theta_antijoin_testing::stats();
    assert!(!fired && rows == 0 && bindings == 0, "stats not cleared by reset");
}
