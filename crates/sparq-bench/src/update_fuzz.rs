//! SPARQL **UPDATE** differential fuzzer vs Oxigraph (beads sq-3dyje.4, sq-hodke). [SONNET-4.6]
//!
//! The query-side differential fuzzer (`fuzz.rs`) has zero UPDATE coverage, and
//! `sparq-engine/src/update.rs` is otherwise guarded by a single mechanism family
//! (unit tests + the fixed W3C conformance corpus). This module closes that gap with
//! a seeded generator of update sequences applied step-by-step to THREE independent
//! implementations:
//!
//!   1. sparq's rebuild path        (`sparq_engine::update` — decode / apply / rebuild),
//!   2. sparq's delta-overlay path  (`sparq_engine::update_in_place`),
//!   3. in-process Oxigraph 0.5     (`Store::update`).
//!
//! After EVERY step it asserts, per the AGENTS.md oracle-strength rule (term structure
//! and full answer sets, never row counts):
//!
//!   * **canonical dataset equality** — see *Adjudicating blank nodes* below;
//!   * **probe SELECT equality** — the full sorted binding set of
//!     `SELECT ?s ?p ?o { ?s ?p ?o }` and `SELECT ?g ?s ?p ?o { GRAPH ?g { ?s ?p ?o } }`
//!     through each engine's own query path;
//!   * **lexical-form preservation** (sparq-side, strict) — see *Numeric lexical forms*.
//!
//! A per-step compare means any divergence is localized to the exact offending
//! operation, and the final-store comparison is the last step's compare.
//!
//! # Generated subset
//!
//! Deterministic by construction — no `NOW()`/`RAND()`/`UUID()`/`BNODE()`:
//! `INSERT DATA` / `DELETE DATA` (with `GRAPH` blocks), `DELETE/INSERT … WHERE` (ground
//! and variable templates, `WITH`, `USING`, variable graph names, `DELETE WHERE`),
//! `CLEAR` / `DROP` / `CREATE` / `COPY` / `MOVE` / `ADD` (always `SILENT`, so absent-graph
//! error behaviour — which the spec leaves implementation-defined — cannot masquerade as
//! a semantic divergence; the state compare is the whole oracle), `LOAD` of a generated
//! local document, plus occasional two-operation compound requests (`op ; op`) for the
//! within-one-update sequencing path.
//!
//! v1 was restricted to the ground-term subset. The **v2 generator extensions** (sq-hodke)
//! are the four below; each keeps the seed-deterministic repro contract (the generator is
//! still a pure function of the seed) and the strict-allowlist posture.
//!
//! ## (1) Non-canonical numeric lexical forms
//!
//! `"0105"^^xsd:integer` / `"+105"^^xsd:integer` are DISTINCT RDF terms from
//! `"105"^^xsd:integer`: RDF term identity is over the lexical form, not the value space.
//! sparq preserves them (its dictionary inlines the *canonical* integer lexicals on a
//! fast path, so a non-canonical lexical must take — and survive — the general path).
//!
//! **Oxigraph 0.5 does not**: its storage encoder round-trips a typed numeric literal
//! through its value space, so `"0105"^^xsd:integer` comes back out as
//! `"105"^^xsd:integer`. That is a real, systematic, *representational* divergence, and
//! it is registered as the adjudicated class `update-numeric-lexical-value-normalization`
//! in `bench/differential-divergences.json`. It is NOT a blind skip:
//!
//!   * the cross-engine compare is retried under a **value-normalizing projection** that
//!     canonicalizes `xsd:integer` lexicals on BOTH sides (recursively, including inside
//!     triple terms). Only if that makes the two datasets equal is the divergence
//!     absorbed (and counted); any residual difference still FAILS with the raw diff. So
//!     the class costs the differential exactly one property — the lexical form of
//!     `xsd:integer` literals — and nothing else;
//!   * that one property is then pinned STRICTLY on the sparq side by
//!     [`unwritten_integer_lexicals`]: every `xsd:integer` lexical present in either
//!     sparq dataset must be one the generator actually WROTE. The non-canonical
//!     generator arm draws from a value range (100..=119) that no *canonical* arm ever
//!     emits, so if sparq ever canonicalized `"0105"` the resulting `"105"` would be a
//!     lexical no operation wrote, and this oracle fires. Oxigraph's store violates it
//!     today, which is what makes it non-vacuous (`tests::unwritten_integer_lexical_
//!     oracle_is_non_vacuous`).
//!
//! ## (2) Blank nodes — isomorphism adjudication
//!
//! With blank nodes in `INSERT DATA` and in `INSERT … WHERE` templates, sorted N-Quads is
//! no longer a canonical form: every engine mints its own fresh labels (sparq's `_:fbN`
//! counter is even process-global, so its own two paths disagree on labels). The
//! comparator therefore splits the dataset compare into three parts:
//!
//!   a. the **bnode-free** quads, compared as sorted N-Quads — exact, byte-level, and
//!      duplicate-preserving (a store yielding one quad twice is an invariant break worth
//!      failing on). On a bnode-free dataset this IS the whole v1 compare, so nothing
//!      weakened for the terms v1 already covered;
//!   b. the **count** of bnode-bearing quads per side — cardinality is invisible to a
//!      set-based canonicalization, so it is checked separately;
//!   c. RDFC-1.0 **canonicalization** of the full dataset (`sparq-canon`, the
//!      `rdf12-triple-terms` profile so triple terms may coexist with blank nodes),
//!      compared byte-for-byte — two datasets are RDF-isomorphic iff these agree.
//!
//! The probe SELECTs are relabelled through the same RDFC-1.0 issued-identifier map
//! before comparison, so a bound blank node compares by its canonical identity.
//!
//! ## (3) LOAD
//!
//! sparq implements `LOAD <file://…>` behind an explicitly-installed allowlisted base
//! directory (`sparq_engine::with_load_base`; the default policy is REJECT). **Oxigraph
//! cannot dereference the source at all** — built without its `http-client` feature its
//! LOAD returns `"HTTP client is not available"`, and `file://` is not a scheme its
//! loader supports in any configuration. So a symmetric LOAD differential is not
//! available, and this harness does not pretend otherwise: the generator emits the
//! document itself, so for each `LOAD` step Oxigraph is given the **semantically
//! equivalent `INSERT DATA`** of exactly those triples in exactly that destination graph.
//!
//! HONEST SCOPE of that substitution: it oracles sparq's LOAD against "merge these
//! triples into this graph" as evaluated by an independent engine — catching destination
//! targeting (`INTO GRAPH` vs the default graph), merge-vs-replace semantics, content
//! fidelity, and interaction with the surrounding sequence. It does NOT differentially
//! test the *fetch/parse* step, since only one engine performs it. `LOAD SILENT` of an
//! absent document is generated too; there the substitution is "apply nothing to
//! Oxigraph", which oracles that sparq's silent failure really was a no-op.
//!
//! ## (4) RDF 1.2 triple terms
//!
//! RDF 1.2 triple terms `<<( s p o )>>` are generated as objects in data blocks and in
//! `DELETE/INSERT` templates (object position only — spargebra rejects them elsewhere),
//! including one level of nesting. Both engines carry them through their own parsers,
//! stores and serializers, so this arm is a genuine three-way differential.
//!
//! # Known shared-oracle blind spot (honest boundary)
//!
//! Both engines parse updates with `spargebra`, so a parser-level desugaring bug (e.g. in
//! COPY/MOVE/ADD expansion, or in the RDF-1.2 annotation sugar) would affect both sides
//! identically and cannot be caught here — only evaluation-layer divergence is
//! observable. Graph EXISTENCE (an empty named graph created by `CREATE`) is invisible to
//! a quad-level compare and remains out of scope.
//!
//! # Reproducing a failure
//!
//! Usage: `sparq-bench update-fuzz --seed-start N --seed-count M`.
//! Every case is reproducible from its seed; on failure the log carries
//! `MISMATCH seed=N` lines plus a `FIRST FAILING CASE:` block (the same machine-parseable
//! contract as `fuzz.rs`, consumed by `scripts/ci-file-differential-failure.py`). The
//! LOAD documents are part of the seed: the harness re-materializes them into a
//! per-seed temporary directory before applying the sequence, so re-running the seed
//! reproduces the run — the `file://` names in the printed ops are relative to that
//! directory, not to the checkout.

use oxigraph::store::Store;
use oxrdf::vocab::xsd;
use oxrdf::{GraphName, Literal, NamedNode, NamedOrBlankNode, Quad, Term, Triple};
use sparq_core::Graph;
use std::collections::{BTreeSet, HashMap};

// ── deterministic RNG ────────────────────────────────────────────────────────────

/// Deterministic SplitMix64 — no clock/entropy, so every case is reproducible from
/// its seed. (Same generator as `fuzz.rs`; duplicated because the bead keeps this
/// file's edits disjoint from the query fuzzer's.)
struct Rng(u64);
impl Rng {
    fn new(seed: u64) -> Self {
        Rng(seed.wrapping_mul(0x9e37_79b9_7f4a_7c15).wrapping_add(1))
    }
    fn next(&mut self) -> u64 {
        self.0 = self.0.wrapping_add(0x9e37_79b9_7f4a_7c15);
        let mut z = self.0;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58_476d_1ce4_e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d0_49bb_1331_11eb);
        z ^ (z >> 31)
    }
    fn below(&mut self, n: u64) -> u64 {
        self.next() % n.max(1)
    }
    fn chance(&mut self, num: u64, den: u64) -> bool {
        self.below(den) < num
    }
}

// ── generator ────────────────────────────────────────────────────────────────────
//
// Small term pools so random DELETE DATA / WHERE conditions collide with previously
// inserted data at high probability (a delete that never matches exercises nothing).

/// The `xsd:integer` IRI spelled out, for the explicit typed-literal forms the
/// non-canonical arm needs (`"0105"^^<…#integer>`) and for N-Triples LOAD documents,
/// which have no bare-integer shorthand.
const XSD_INTEGER_IRI: &str = "http://www.w3.org/2001/XMLSchema#integer";

/// Values used by the CANONICAL integer arms. Disjoint from [`NONCANON_BASE`]..
const CANON_INT_RANGE: u64 = 20;
/// Values used by the NON-CANONICAL integer arms (`"0105"`, `"+105"`). Deliberately
/// disjoint from the canonical range so that the canonical rendering of a
/// non-canonical lexical (`"105"`) is a lexical NO generator arm ever writes — that
/// disjointness is what makes [`unwritten_integer_lexicals`] a sharp oracle.
const NONCANON_BASE: u64 = 100;
const NONCANON_RANGE: u64 = 20;

fn gen_subject(rng: &mut Rng) -> String {
    format!("<http://ex/s{}>", rng.below(6))
}

fn gen_predicate(rng: &mut Rng) -> String {
    format!("<http://ex/p{}>", rng.below(4))
}

/// One of the 3 named graphs. A small pool keeps CLEAR/DROP/COPY/MOVE/ADD landing on
/// graphs that actually hold data.
fn gen_graph_iri(rng: &mut Rng) -> String {
    format!("<http://ex/g{}>", rng.below(3))
}

/// Generator state threaded through one sequence.
///
/// `inserted` holds the GROUND quads emitted by earlier `INSERT DATA` operations, as
/// `(graph IRI or None-for-default, "s p o .")`. `DELETE DATA` (and the ground `WHERE`
/// condition) sample mostly from this pool so deletions/conditions actually HIT live
/// data — a purely random quad almost never matches the store, which was measured to
/// let a neutered DELETE DATA path slip through a 200-seed window undetected.
/// Blank-node-bearing quads are NOT recorded: `DELETE DATA` takes ground quads only
/// (spargebra rejects a blank node there), so re-emitting one would be a parse error.
#[derive(Default)]
struct GenState {
    inserted: Vec<(Option<String>, String)>,
    /// Fresh `_:bN` label counter (per sequence, so labels are seed-deterministic).
    bnodes: u64,
    /// LOAD document counter (`d0.nt`, `d1.nt`, …).
    docs: u64,
    /// Every `xsd:integer` LEXICAL form this sequence has written, in any position
    /// (data block, template, LOAD document). The strict lexical-preservation oracle
    /// checks sparq's stores against exactly this set.
    integer_lexicals: BTreeSet<String>,
}

impl GenState {
    fn fresh_bnode(&mut self) -> String {
        self.bnodes += 1;
        format!("_:b{}", self.bnodes)
    }

    /// Records `lex` as written and returns it, so every emission site funnels the
    /// lexical into the preservation oracle's expected set.
    fn write_integer(&mut self, lex: String) -> String {
        self.integer_lexicals.insert(lex.clone());
        lex
    }
}

/// A canonical integer object in SPARQL/Turtle shorthand (`7`), which both parsers
/// expand to `"7"^^xsd:integer`.
fn gen_canonical_integer(rng: &mut Rng, st: &mut GenState) -> String {
    let n = rng.below(CANON_INT_RANGE);
    st.write_integer(n.to_string());
    n.to_string()
}

/// A NON-canonical `xsd:integer` lexical: a leading zero (`"0105"`) or an explicit
/// plus (`"+105"`). Written as an explicit typed literal — the shorthand form cannot
/// express a non-canonical lexical.
fn gen_noncanonical_integer(rng: &mut Rng, st: &mut GenState) -> String {
    let n = NONCANON_BASE + rng.below(NONCANON_RANGE);
    let lex = if rng.chance(1, 2) {
        format!("0{}", n)
    } else {
        format!("+{}", n)
    };
    st.write_integer(lex.clone());
    format!("\"{}\"^^<{}>", lex, XSD_INTEGER_IRI)
}

/// A ground object with no triple term: the term kinds the dictionary treats
/// differently — IRIs, plain strings, canonical `xsd:integer` (the inline-id path),
/// NON-canonical `xsd:integer` (the general path, extension 1), language-tagged
/// strings. Used both directly and as the object slot inside a triple term.
fn gen_flat_object(rng: &mut Rng, st: &mut GenState) -> String {
    match rng.below(10) {
        0..=2 => format!("<http://ex/o{}>", rng.below(6)),
        3..=4 => format!("\"lit{}\"", rng.below(5)),
        5..=6 => gen_canonical_integer(rng, st),
        7 => gen_noncanonical_integer(rng, st),
        _ => format!("\"tag{}\"@en", rng.below(3)),
    }
}

/// An RDF 1.2 **triple term** `<<( s p o )>>` (extension 4). Object position only —
/// spargebra rejects a triple term as a subject. `depth` bounds the nesting: at depth
/// 1 the inner object is flat, above that it may itself be a triple term.
fn gen_triple_term(rng: &mut Rng, st: &mut GenState, depth: u64) -> String {
    let inner = if depth > 1 && rng.chance(1, 3) {
        gen_triple_term(rng, st, depth - 1)
    } else {
        gen_flat_object(rng, st)
    };
    format!(
        "<<( {} {} {} )>>",
        gen_subject(rng),
        gen_predicate(rng),
        inner
    )
}

/// Any ground object: flat, or (1-in-8) a triple term nested up to 2 levels.
fn gen_object(rng: &mut Rng, st: &mut GenState) -> String {
    if rng.chance(1, 8) {
        gen_triple_term(rng, st, 2)
    } else {
        gen_flat_object(rng, st)
    }
}

/// Renders a quad into a data block: bare triple or `GRAPH`-wrapped.
fn push_quad(block: &mut String, slot: &Option<String>, t: &str) {
    match slot {
        Some(g) => block.push_str(&format!(" GRAPH {} {{ {} }}", g, t)),
        None => block.push_str(&format!(" {}", t)),
    }
}

/// A random fresh ground quad: `(graph slot, triple text)`.
fn gen_quad(rng: &mut Rng, st: &mut GenState) -> (Option<String>, String) {
    let t = format!(
        "{} {} {} .",
        gen_subject(rng),
        gen_predicate(rng),
        gen_object(rng, st)
    );
    let slot = if rng.chance(2, 5) {
        Some(gen_graph_iri(rng))
    } else {
        None
    };
    (slot, t)
}

/// The `INSERT DATA` block: 1..=5 quads, only the GROUND ones recorded in the pool.
///
/// Blank nodes (extension 2) are drawn from a per-block pool of two labels, so the
/// same label recurs across quads within the operation — the parser must keep those
/// one node, and the store must mint it fresh w.r.t. everything already there. Reusing
/// labels (rather than one fresh label per quad) is what makes the resulting graph's
/// blank-node structure non-trivial, so the isomorphism compare has something to say.
fn gen_insert_block(rng: &mut Rng, st: &mut GenState) -> String {
    let n = 1 + rng.below(5);
    let labels: Vec<String> = if rng.chance(1, 3) {
        vec![st.fresh_bnode(), st.fresh_bnode()]
    } else {
        Vec::new()
    };
    let mut s = String::new();
    for _ in 0..n {
        let (slot, t) = gen_quad(rng, st);
        if labels.is_empty() {
            push_quad(&mut s, &slot, &t);
            st.inserted.push((slot, t));
            continue;
        }
        // A blank node in the subject and/or the object slot.
        let subject = if rng.chance(1, 2) {
            labels[rng.below(labels.len() as u64) as usize].clone()
        } else {
            gen_subject(rng)
        };
        let object = if rng.chance(2, 5) {
            labels[rng.below(labels.len() as u64) as usize].clone()
        } else {
            gen_object(rng, st)
        };
        let bnode_free = !subject.starts_with("_:") && !object.starts_with("_:");
        let t2 = format!("{} {} {} .", subject, gen_predicate(rng), object);
        push_quad(&mut s, &slot, &t2);
        if bnode_free {
            st.inserted.push((slot, t2));
        }
    }
    s
}

/// The `DELETE DATA` block: 1..=5 quads, each drawn from the (ground) inserted pool
/// with probability 7/10 (a hit unless an intervening op already removed it) and fresh
/// random otherwise (the delete-of-absent-quad no-op path).
fn gen_delete_block(rng: &mut Rng, st: &mut GenState) -> String {
    let n = 1 + rng.below(5);
    let mut s = String::new();
    for _ in 0..n {
        let (slot, t) = if !st.inserted.is_empty() && rng.chance(7, 10) {
            st.inserted[rng.below(st.inserted.len() as u64) as usize].clone()
        } else {
            gen_quad(rng, st)
        };
        push_quad(&mut s, &slot, &t);
    }
    s
}

/// A `DELETE/INSERT … WHERE` operation. Templates are ground or variable; the WHERE
/// is a single BGP (optionally GRAPH-wrapped / re-scoped) — instantiation against
/// ground data yields only ground triples, so every shape is deterministic. The
/// query-side differential owns complex WHERE evaluation; the shapes here exist to
/// drive the update-side template instantiation, `WITH`/`USING` re-scoping, and
/// variable-graph-name paths.
fn gen_where_op(rng: &mut Rng, st: &mut GenState) -> String {
    match rng.below(10) {
        // Conditional ground insert: fires iff the (ground) condition triple exists.
        // The condition samples the inserted pool half the time so both the
        // condition-holds and condition-fails branches are exercised.
        0 => {
            let cond = if !st.inserted.is_empty() && rng.chance(1, 2) {
                let (slot, t) = &st.inserted[rng.below(st.inserted.len() as u64) as usize];
                match slot {
                    Some(g) => format!("GRAPH {} {{ {} }}", g, t),
                    None => t.clone(),
                }
            } else {
                let (s, p) = (gen_subject(rng), gen_predicate(rng));
                format!("{} {} {} .", s, p, gen_object(rng, st))
            };
            let (s, p) = (gen_subject(rng), gen_predicate(rng));
            format!(
                "INSERT {{ {} {} {} }} WHERE {{ {} }}",
                s,
                p,
                gen_object(rng, st),
                cond
            )
        }
        // Remove every `p` edge from the default graph.
        1 => {
            let p = gen_predicate(rng);
            format!("DELETE {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}", p, p)
        }
        // Predicate rename in the default graph (delete + insert see the same pre-state).
        2 => {
            let p = gen_predicate(rng);
            let p2 = gen_predicate(rng);
            format!(
                "DELETE {{ ?s {} ?o }} INSERT {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}",
                p, p2, p
            )
        }
        // Copy the default graph into a named graph (template GRAPH block).
        3 => format!(
            "INSERT {{ GRAPH {} {{ ?s ?p ?o }} }} WHERE {{ ?s ?p ?o }}",
            gen_graph_iri(rng)
        ),
        // Cross-graph delete with a VARIABLE graph name in the template.
        4 => {
            let p = gen_predicate(rng);
            format!(
                "DELETE {{ GRAPH ?g {{ ?s {} ?o }} }} WHERE {{ GRAPH ?g {{ ?s {} ?o }} }}",
                p, p
            )
        }
        // WITH-scoped predicate rename inside one named graph.
        5 => {
            let g = gen_graph_iri(rng);
            let p = gen_predicate(rng);
            let p2 = gen_predicate(rng);
            format!(
                "WITH {} DELETE {{ ?s {} ?o }} INSERT {{ ?s {} ?o }} WHERE {{ ?s {} ?o }}",
                g, p, p2, p
            )
        }
        // USING re-scoping: WHERE evaluates against the named graph, the deletion
        // template applies to the default graph.
        6 => format!(
            "DELETE {{ ?s ?p ?o }} USING {} WHERE {{ ?s ?p ?o }}",
            gen_graph_iri(rng)
        ),
        // Pull a named graph's triples into the default graph.
        7 => format!(
            "INSERT {{ ?s ?p ?o }} WHERE {{ GRAPH {} {{ ?s ?p ?o }} }}",
            gen_graph_iri(rng)
        ),
        // BLANK NODE in an INSERT template (extension 2): SPARQL 1.1 Update §3.1.3
        // mints ONE FRESH blank node PER SOLUTION, so the cardinality of the WHERE
        // becomes observable in the graph's blank-node structure. Both engines must
        // agree up to isomorphism.
        8 => {
            let p = gen_predicate(rng);
            let link = gen_predicate(rng);
            format!(
                "INSERT {{ ?s {} {} }} WHERE {{ ?s {} ?o }}",
                link,
                st.fresh_bnode(),
                p
            )
        }
        // The DELETE WHERE shorthand (its own grammar production).
        _ => {
            let p = gen_predicate(rng);
            format!("DELETE WHERE {{ ?s {} ?o }}", p)
        }
    }
}

/// A graph-management operation. Always `SILENT`: the spec leaves the error-vs-no-op
/// choice for absent/existing targets implementation-defined (a store MAY succeed),
/// so a non-SILENT error-status difference between engines would not be a semantic
/// divergence — SILENT removes that ambiguity and keeps "every op must succeed on
/// every engine, and the states must match" as the whole oracle.
fn gen_structural(rng: &mut Rng) -> String {
    // DEFAULT or a named graph, for COPY/MOVE/ADD endpoints.
    fn graph_or_default(rng: &mut Rng) -> String {
        if rng.chance(1, 4) {
            "DEFAULT".to_string()
        } else {
            format!("GRAPH {}", gen_graph_iri(rng))
        }
    }
    match rng.below(8) {
        0 | 1 => {
            let target = match rng.below(5) {
                0 => "DEFAULT".to_string(),
                1 => "NAMED".to_string(),
                2 => "ALL".to_string(),
                _ => format!("GRAPH {}", gen_graph_iri(rng)),
            };
            format!("CLEAR SILENT {}", target)
        }
        2 | 3 => {
            let target = match rng.below(5) {
                0 => "DEFAULT".to_string(),
                1 => "NAMED".to_string(),
                2 => "ALL".to_string(),
                _ => format!("GRAPH {}", gen_graph_iri(rng)),
            };
            format!("DROP SILENT {}", target)
        }
        4 => format!("CREATE SILENT GRAPH {}", gen_graph_iri(rng)),
        // COPY/MOVE/ADD, including the same-source-and-destination corner (a spec
        // no-op) — both engines desugar via spargebra, so only evaluation-layer
        // handling can diverge here.
        _ => {
            let verb = ["COPY", "MOVE", "ADD"][rng.below(3) as usize];
            format!(
                "{} SILENT {} TO {}",
                verb,
                graph_or_default(rng),
                graph_or_default(rng)
            )
        }
    }
}

/// One generated update request, plus everything the differential needs to apply it.
///
/// `oxi` is normally `Some(text)` — the identical request. It differs only for LOAD
/// (extension 3), which Oxigraph cannot perform at all: there it carries the
/// equivalent `INSERT DATA`, or `None` when the sparq side is a proven no-op (a
/// `LOAD SILENT` of an absent document). See the module docs for the honest scope of
/// that substitution.
#[derive(Clone, Debug, PartialEq, Eq)]
struct Op {
    text: String,
    oxi: Option<String>,
    /// `(file name, N-Triples content)` documents this op needs on disk first.
    docs: Vec<(String, String)>,
}

impl Op {
    /// The common case: the same request goes to every engine.
    fn same(text: String) -> Op {
        Op {
            oxi: Some(text.clone()),
            text,
            docs: Vec::new(),
        }
    }

    /// `a ; b` as one request — including on the Oxigraph side, where a `None` half
    /// simply drops out (it is a no-op there by construction).
    fn compound(a: Op, b: Op) -> Op {
        Op {
            text: format!("{} ;\n{}", a.text, b.text),
            oxi: match (a.oxi, b.oxi) {
                (Some(x), Some(y)) => Some(format!("{} ;\n{}", x, y)),
                (x, None) => x,
                (None, y) => y,
            },
            docs: [a.docs, b.docs].concat(),
        }
    }
}

/// A `LOAD` operation over a generated local document (extension 3).
///
/// The document is N-Triples (`.nt`) and GROUND: no named graphs (so `INTO GRAPH` is
/// unambiguously the whole destination) and no blank nodes (sparq's loader preserves
/// document labels while Oxigraph's renames them, which would make the substituted
/// `INSERT DATA` inequivalent under a re-LOAD — see the follow-up note in the bead).
/// The reference is the RELATIVE `file://` form so the op text stays a pure function
/// of the seed; it resolves against the per-seed allowlisted base directory.
fn gen_load(rng: &mut Rng, st: &mut GenState) -> Op {
    let silent = if rng.chance(1, 2) { "SILENT " } else { "" };
    let dest = if rng.chance(1, 2) {
        Some(gen_graph_iri(rng))
    } else {
        None
    };
    let into = match &dest {
        Some(g) => format!(" INTO GRAPH {}", g),
        None => String::new(),
    };
    st.docs += 1;
    // 1-in-5: LOAD a document that was never written. sparq must SILENT-no-op; the
    // Oxigraph side applies nothing, so the compare oracles exactly that.
    if rng.chance(1, 5) {
        return Op {
            text: format!("LOAD SILENT <file://absent{}.nt>{}", st.docs, into),
            oxi: None,
            docs: Vec::new(),
        };
    }
    let name = format!("d{}.nt", st.docs);
    let mut lines = Vec::new();
    for _ in 0..1 + rng.below(3) {
        // N-Triples has no bare-integer shorthand, so integer objects are explicit.
        let object = match rng.below(8) {
            0..=2 => format!("<http://ex/o{}>", rng.below(6)),
            3..=4 => format!("\"lit{}\"", rng.below(5)),
            5 => {
                let n = rng.below(CANON_INT_RANGE);
                st.write_integer(n.to_string());
                format!("\"{}\"^^<{}>", n, XSD_INTEGER_IRI)
            }
            6 => gen_noncanonical_integer(rng, st),
            _ => format!("\"tag{}\"@en", rng.below(3)),
        };
        lines.push(format!(
            "{} {} {} .",
            gen_subject(rng),
            gen_predicate(rng),
            object
        ));
    }
    let body = lines.join("\n");
    let insert = match &dest {
        Some(g) => format!("INSERT DATA {{ GRAPH {} {{ {} }} }}", g, body),
        None => format!("INSERT DATA {{ {} }}", body),
    };
    Op {
        text: format!("LOAD {}<file://{}>{}", silent, name, into),
        oxi: Some(insert),
        docs: vec![(name, format!("{}\n", body))],
    }
}

/// One update request. Mostly a single operation; 1-in-10 a two-operation compound
/// (`op ; op`) to exercise within-one-request sequencing.
fn gen_op(rng: &mut Rng, st: &mut GenState) -> Op {
    fn single(rng: &mut Rng, st: &mut GenState) -> Op {
        match rng.below(100) {
            0..=32 => Op::same(format!("INSERT DATA {{{} }}", gen_insert_block(rng, st))),
            33..=46 => Op::same(format!("DELETE DATA {{{} }}", gen_delete_block(rng, st))),
            47..=71 => Op::same(gen_where_op(rng, st)),
            72..=78 => gen_load(rng, st),
            _ => Op::same(gen_structural(rng)),
        }
    }
    let first = single(rng, st);
    if rng.chance(1, 10) {
        Op::compound(first, single(rng, st))
    } else {
        first
    }
}

/// The whole update sequence for one seed: 4..=10 update requests, applied (and
/// differentially checked) one at a time. Returns the generator state alongside, whose
/// `integer_lexicals` set the lexical-preservation oracle checks the stores against.
fn gen_sequence(rng: &mut Rng) -> (Vec<Op>, GenState) {
    let n = 4 + rng.below(7) as usize;
    let mut st = GenState::default();
    let ops = (0..n).map(|_| gen_op(rng, &mut st)).collect();
    (ops, st)
}

// ── term utilities (numeric lexicals, blank nodes, triple terms) ─────────────────

/// The value-normalizing projection behind the `update-numeric-lexical-value-
/// normalization` adjudicated class: canonicalizes an `xsd:integer` LEXICAL form
/// ("0105" / "+105" -> "105"). Returns `None` when the term is not an `xsd:integer`
/// literal, when its lexical is not a parseable integer (an ill-typed literal is a
/// term in its own right and must NOT be rewritten), or when it is already canonical.
fn canonical_integer(l: &Literal) -> Option<Literal> {
    if l.datatype() != xsd::INTEGER {
        return None;
    }
    let canon = l.value().parse::<i128>().ok()?.to_string();
    if canon == l.value() {
        None
    } else {
        Some(Literal::new_typed_literal(canon, xsd::INTEGER))
    }
}

/// [`canonical_integer`] applied recursively, so a non-canonical lexical nested
/// inside an RDF-1.2 triple term is normalized too.
fn normalize_term(t: &Term) -> Term {
    match t {
        Term::Literal(l) => match canonical_integer(l) {
            Some(c) => Term::Literal(c),
            None => t.clone(),
        },
        Term::Triple(tr) => Term::Triple(Box::new(Triple::new(
            tr.subject.clone(),
            tr.predicate.clone(),
            normalize_term(&tr.object),
        ))),
        other => other.clone(),
    }
}

fn normalize_quad(q: &Quad) -> Quad {
    Quad::new(
        q.subject.clone(),
        q.predicate.clone(),
        normalize_term(&q.object),
        q.graph_name.clone(),
    )
}

/// Collects every `xsd:integer` LEXICAL form occurring in a term (recursing into
/// triple terms), for the strict lexical-preservation oracle.
fn collect_integer_lexicals(t: &Term, out: &mut BTreeSet<String>) {
    match t {
        Term::Literal(l) if l.datatype() == xsd::INTEGER => {
            out.insert(l.value().to_string());
        }
        Term::Triple(tr) => collect_integer_lexicals(&tr.object, out),
        _ => {}
    }
}

/// The `xsd:integer` lexical forms present in `quads` that the generator never WROTE
/// (`written` is [`GenState::integer_lexicals`]). MUST be empty for both sparq
/// datasets: RDF term identity is over the lexical form, so a store that turned the
/// generated `"0105"^^xsd:integer` into `"105"^^xsd:integer` silently replaced one RDF
/// term with a different one. The non-canonical generator arm draws from a value range
/// no canonical arm uses, so any such canonicalization necessarily produces a lexical
/// outside `written` and is caught here rather than absorbed by the adjudicated class.
fn unwritten_integer_lexicals(quads: &[Quad], written: &BTreeSet<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    for q in quads {
        collect_integer_lexicals(&q.object, &mut seen);
    }
    seen.into_iter().filter(|l| !written.contains(l)).collect()
}

/// Whether a term mentions a blank node (recursing into triple terms).
fn term_has_bnode(t: &Term) -> bool {
    match t {
        Term::BlankNode(_) => true,
        Term::Triple(tr) => {
            matches!(tr.subject, NamedOrBlankNode::BlankNode(_)) || term_has_bnode(&tr.object)
        }
        _ => false,
    }
}

/// Whether a quad mentions a blank node in ANY position — the split between the exact
/// sorted-N-Quads compare and the isomorphism compare.
fn quad_has_bnode(q: &Quad) -> bool {
    matches!(q.subject, NamedOrBlankNode::BlankNode(_))
        || term_has_bnode(&q.object)
        || matches!(q.graph_name, GraphName::BlankNode(_))
}

// ── dataset snapshots ────────────────────────────────────────────────────────────

/// Converts a decoded object term into the subject/graph position, which admit only
/// IRIs and blank nodes. A literal there is a store invariant break, so it is an error
/// rather than a silent drop.
fn to_named_or_blank(t: &Term, position: &str) -> Result<NamedOrBlankNode, String> {
    match t {
        Term::NamedNode(n) => Ok(NamedOrBlankNode::NamedNode(n.clone())),
        Term::BlankNode(b) => Ok(NamedOrBlankNode::BlankNode(b.clone())),
        other => Err(format!("non-node term in {} position: {}", position, other)),
    }
}

/// A sparq `Graph` (default graph + named graphs) as oxrdf quads — the shared input to
/// every compare (sorted N-Quads, RDFC-1.0 canonicalization, the lexical oracle).
fn sparq_quads(g: &Graph) -> Result<Vec<Quad>, String> {
    fn triples_of(g: &Graph, name: &GraphName, out: &mut Vec<Quad>) -> Result<(), String> {
        let scan = g.store.scan(&[None, None, None]);
        for r in scan.rows.iter() {
            let t = scan.to_spo(r);
            let subject = to_named_or_blank(&g.dict.term(t[0]), "subject")?;
            let predicate = match g.dict.term(t[1]) {
                Term::NamedNode(n) => n,
                other => return Err(format!("non-IRI term in predicate position: {}", other)),
            };
            out.push(Quad::new(
                subject,
                predicate,
                g.dict.term(t[2]),
                name.clone(),
            ));
        }
        Ok(())
    }
    let mut out = Vec::new();
    triples_of(g, &GraphName::DefaultGraph, &mut out)?;
    for (name, sub) in &g.named {
        let gname = match to_named_or_blank(name, "graph name")? {
            NamedOrBlankNode::NamedNode(n) => GraphName::NamedNode(n),
            NamedOrBlankNode::BlankNode(b) => GraphName::BlankNode(b),
        };
        triples_of(sub, &gname, &mut out)?;
    }
    Ok(out)
}

/// The Oxigraph store as oxrdf quads. Same `oxrdf` 0.3 term model as sparq's (one
/// unified crate version in the lock file), so the two sides are directly comparable.
fn oxi_quads(store: &Store) -> Result<Vec<Quad>, String> {
    store
        .iter()
        .map(|q| q.map_err(|e| format!("oxigraph iter error: {}", e)))
        .collect()
}

/// The sorted N-Quads lines of the BNODE-FREE quads. On a blank-node-free dataset this
/// is the whole v1 canonical form: exact, byte-level, and duplicate-PRESERVING (a store
/// yielding the same quad twice is an invariant break worth failing on).
fn ground_lines(quads: &[Quad]) -> Vec<String> {
    let mut out: Vec<String> = quads
        .iter()
        .filter(|q| !quad_has_bnode(q))
        .map(|q| format!("{} .", q))
        .collect();
    out.sort();
    out
}

/// The RDFC-1.0 canonical N-Quads document (the `rdf12-triple-terms` profile, so RDF
/// 1.2 triple terms may coexist with blank nodes). Two datasets are RDF-isomorphic iff
/// these agree byte-for-byte, which is what adjudicates the engines' differing fresh
/// blank-node labels.
fn canonical_document(quads: &[Quad]) -> Result<String, String> {
    sparq_canon::canonicalize_rdf12(quads).map_err(|e| format!("canonicalization failed: {}", e))
}

/// The RDFC-1.0 issued-identifier map (store blank-node label -> `c14nN`) used to
/// relabel probe bindings. Empty when the dataset has no blank nodes.
fn issued_labels(quads: &[Quad]) -> Result<HashMap<String, String>, String> {
    if !quads.iter().any(quad_has_bnode) {
        return Ok(HashMap::new());
    }
    sparq_canon::issue_dataset_rdf12(quads)
        .map_err(|e| format!("blank-node identifier issuance failed: {}", e))
}

/// The lines on exactly one side — the human-readable core of a divergence report.
fn one_sided(label_a: &str, a: &[String], label_b: &str, b: &[String]) -> String {
    let bset: BTreeSet<&String> = b.iter().collect();
    let aset: BTreeSet<&String> = a.iter().collect();
    let mut s = String::new();
    s.push_str(&format!("only in {}:\n", label_a));
    for l in a.iter().filter(|l| !bset.contains(l)) {
        s.push_str(&format!("  {}\n", l));
    }
    s.push_str(&format!("only in {}:\n", label_b));
    for l in b.iter().filter(|l| !aset.contains(l)) {
        s.push_str(&format!("  {}\n", l));
    }
    s
}

/// The full dataset compare between two engines' snapshots, in three parts (see the
/// module docs, *Blank nodes*): exact sorted N-Quads over the bnode-free quads, the
/// COUNT of bnode-bearing quads (cardinality is invisible to a set-based
/// canonicalization), and RDFC-1.0 canonical equality of the whole dataset.
///
/// `normalize` applies the [`normalize_term`] projection to BOTH sides — the retry the
/// adjudicated numeric-lexical class permits, never the first attempt.
fn compare_datasets(
    label_a: &str,
    a: &[Quad],
    label_b: &str,
    b: &[Quad],
    normalize: bool,
) -> Result<(), String> {
    let (pa, pb);
    let (a, b) = if normalize {
        pa = a.iter().map(normalize_quad).collect::<Vec<_>>();
        pb = b.iter().map(normalize_quad).collect::<Vec<_>>();
        (&pa[..], &pb[..])
    } else {
        (a, b)
    };

    let (ga, gb) = (ground_lines(a), ground_lines(b));
    if ga != gb {
        return Err(format!(
            "canonical dataset differs on the blank-node-free quads ({} vs {})\n{}",
            label_a,
            label_b,
            one_sided(label_a, &ga, label_b, &gb)
        ));
    }

    let (na, nb) = (
        a.iter().filter(|q| quad_has_bnode(q)).count(),
        b.iter().filter(|q| quad_has_bnode(q)).count(),
    );
    if na != nb {
        return Err(format!(
            "blank-node-bearing quad COUNT differs: {}={} {}={}",
            label_a, na, label_b, nb
        ));
    }
    if na == 0 {
        return Ok(());
    }

    let (ca, cb) = (canonical_document(a)?, canonical_document(b)?);
    if ca != cb {
        let (la, lb): (Vec<String>, Vec<String>) = (
            ca.lines().map(str::to_string).collect(),
            cb.lines().map(str::to_string).collect(),
        );
        return Err(format!(
            "datasets are NOT RDF-isomorphic ({} vs {}; RDFC-1.0 canonical forms differ)\n{}",
            label_a,
            label_b,
            one_sided(label_a, &la, label_b, &lb)
        ));
    }
    Ok(())
}

// ── probe SELECTs (full binding-set compare, per the oracle-strength rule) ────────

const PROBES: &[(&str, &[&str])] = &[
    ("SELECT ?s ?p ?o WHERE { ?s ?p ?o }", &["s", "p", "o"]),
    (
        "SELECT ?g ?s ?p ?o WHERE { GRAPH ?g { ?s ?p ?o } }",
        &["g", "s", "p", "o"],
    ),
];

/// Renders a bound term for the probe compare, replacing each blank node with its
/// RDFC-1.0 canonical identity so the two engines' differing fresh labels compare
/// equal. A label missing from `labels` renders as `_:UNMAPPED(<label>)`, which cannot
/// match the other side — fail-toward-flagging rather than silently agreeing.
fn render_binding(t: &Term, labels: &HashMap<String, String>, normalize: bool) -> String {
    let t = if normalize { normalize_term(t) } else { t.clone() };
    match &t {
        Term::BlankNode(b) => match labels.get(b.as_str()) {
            Some(c) => format!("_:{}", c),
            None => format!("_:UNMAPPED({})", b.as_str()),
        },
        Term::Triple(tr) => {
            let subject = match &tr.subject {
                NamedOrBlankNode::BlankNode(b) => {
                    render_binding(&Term::BlankNode(b.clone()), labels, normalize)
                }
                NamedOrBlankNode::NamedNode(n) => n.to_string(),
            };
            format!(
                "<<( {} {} {} )>>",
                subject,
                tr.predicate,
                render_binding(&tr.object, labels, normalize)
            )
        }
        other => other.to_string(),
    }
}

fn render_row<'a>(
    cells: impl Iterator<Item = Option<&'a Term>>,
    labels: &HashMap<String, String>,
    normalize: bool,
) -> String {
    cells
        .map(|c| match c {
            Some(t) => render_binding(t, labels, normalize),
            None => "UNDEF".to_string(),
        })
        .collect::<Vec<_>>()
        .join(" | ")
}

/// The full sorted binding set of a probe through sparq's query path.
fn sparq_probe(
    g: &Graph,
    q: &str,
    labels: &HashMap<String, String>,
    normalize: bool,
) -> Result<Vec<String>, String> {
    let r = sparq_engine::query(g, q).map_err(|e| format!("sparq probe error: {}", e))?;
    let mut rows: Vec<String> = r
        .rows
        .iter()
        .map(|row| render_row(row.iter().map(Option::as_ref), labels, normalize))
        .collect();
    rows.sort();
    Ok(rows)
}

/// The full sorted binding set of a probe through Oxigraph's query path.
// clippy: the differential oracle pins oxigraph's legacy Store::query semantics
#[allow(deprecated)]
fn oxi_probe(
    store: &Store,
    q: &str,
    vars: &[&str],
    labels: &HashMap<String, String>,
    normalize: bool,
) -> Result<Vec<String>, String> {
    match store.query(q).map_err(|e| e.to_string())? {
        oxigraph::sparql::QueryResults::Solutions(s) => {
            let mut rows = Vec::new();
            for sol in s {
                let sol = sol.map_err(|e| e.to_string())?;
                rows.push(render_row(
                    vars.iter().map(|v| sol.get(*v)),
                    labels,
                    normalize,
                ));
            }
            rows.sort();
            Ok(rows)
        }
        _ => Err("probe did not return solutions".to_string()),
    }
}

// ── adjudicated-divergence allowlist (update classes) ─────────────────────────────

/// The adjudicated `update-*` divergence class id owned by this harness.
const NUMERIC_LEXICAL_CLASS: &str = "update-numeric-lexical-value-normalization";

/// The adjudicated classes enabled for this run, loaded from the same
/// `bench/differential-divergences.json` the query fuzzer consumes; UPDATE classes use
/// an `update-` id prefix. A class the FILE does not list is STRICT, and a listed
/// `update-*` class this comparator has no detector for is ALSO strict (loud warning) —
/// mirroring the fail-toward-flagging posture of `fuzz.rs`. Never a silent skip.
#[derive(Default)]
struct UpdateAllowlist {
    /// `update-numeric-lexical-value-normalization` (sq-hodke): permits the
    /// value-normalizing RETRY of a cross-engine compare. See the module docs.
    numeric_lexical: bool,
    source: String,
}

impl UpdateAllowlist {
    fn load() -> Self {
        let path = std::env::var("SPARQ_FUZZ_DIVERGENCES").unwrap_or_else(|_| {
            concat!(
                env!("CARGO_MANIFEST_DIR"),
                "/../../bench/differential-divergences.json"
            )
            .to_string()
        });
        match std::fs::read_to_string(&path) {
            Ok(s) => Self::from_json(&s, &path),
            Err(e) => {
                eprintln!(
                    "warning: divergence allowlist {} unreadable ({}) — running STRICT",
                    path, e
                );
                UpdateAllowlist {
                    source: path,
                    ..Default::default()
                }
            }
        }
    }

    fn from_json(s: &str, source: &str) -> Self {
        let mut out = UpdateAllowlist {
            source: source.to_string(),
            ..Default::default()
        };
        let v: serde_json::Value = match serde_json::from_str(s) {
            Ok(v) => v,
            Err(e) => {
                eprintln!(
                    "warning: divergence allowlist {} is not valid JSON ({}) — running STRICT",
                    source, e
                );
                return out;
            }
        };
        for class in v["classes"].as_array().map(Vec::as_slice).unwrap_or(&[]) {
            match class["id"].as_str() {
                // The query-side classes; `fuzz.rs` owns their detectors.
                Some(id) if !id.starts_with("update-") => {}
                Some(id) if id == NUMERIC_LEXICAL_CLASS => out.numeric_lexical = true,
                Some(other) => eprintln!(
                    "warning: divergence allowlist {} lists unknown update class id {:?} — \
                     no detector for it; that class stays STRICT",
                    source, other
                ),
                None => eprintln!(
                    "warning: divergence allowlist {} has a class without a string `id` — \
                     ignored (strict)",
                    source
                ),
            }
        }
        out
    }

    /// The run-summary state line.
    fn describe(&self) -> String {
        if self.numeric_lexical {
            format!(
                "({}): {}=on (value-normalizing RETRY only; residual differences still fail)",
                self.source, NUMERIC_LEXICAL_CLASS
            )
        } else {
            format!(
                "({}): no adjudicated `update-*` classes — STRICT (every divergence fails)",
                self.source
            )
        }
    }
}

// ── the per-seed differential ────────────────────────────────────────────────────

/// What one seed contributed to the run summary.
#[derive(Debug, Default)]
struct SeedStats {
    /// Update requests applied.
    ops: u64,
    /// Steps absorbed by an adjudicated divergence class (never a silent skip — the
    /// step's compare still had to pass under the class's normalizing projection).
    adjudicated: u64,
}

/// The per-seed allowlisted base directory for `LOAD <file://…>`, removed on the way
/// out (including on unwind, so a failing seed leaves no temp files behind).
struct LoadDir(std::path::PathBuf);
impl Drop for LoadDir {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

/// Runs `f` with `base` installed as sparq's allowlisted LOAD root, when the sequence
/// has any documents at all. Without a base every `file://` LOAD is REJECTED, which is
/// sparq's secure default and exactly what the absent-document arm relies on.
fn with_optional_base<R>(base: Option<&std::path::Path>, f: impl FnOnce() -> R) -> R {
    match base {
        Some(b) => sparq_engine::with_load_base(b, f),
        None => f(),
    }
}

/// `quads`, optionally under the numeric-lexical normalizing projection.
fn projected(quads: &[Quad], normalize: bool) -> Vec<Quad> {
    if normalize {
        quads.iter().map(normalize_quad).collect()
    } else {
        quads.to_vec()
    }
}

/// One cross-engine dataset compare, consulting the adjudicated classes.
///
/// STRICT first: the raw compare must pass. Only if it fails, and only if the FILE
/// enables the numeric-lexical class, is the compare retried under the value-normalizing
/// projection; that retry passing means the entire difference was `xsd:integer` lexical
/// forms (adjudicated — `Ok(true)`). Anything else returns the RAW diff, so a residual
/// divergence is never absorbed.
fn cross_compare(
    label_a: &str,
    a: &[Quad],
    label_b: &str,
    b: &[Quad],
    allow: &UpdateAllowlist,
) -> Result<bool, String> {
    match compare_datasets(label_a, a, label_b, b, false) {
        Ok(()) => Ok(false),
        Err(raw) => {
            if allow.numeric_lexical && compare_datasets(label_a, a, label_b, b, true).is_ok() {
                Ok(true)
            } else {
                Err(raw)
            }
        }
    }
}

/// Applies one seed's update sequence to all three implementations, checking the
/// canonical dataset, the lexical-preservation oracle and both probes after every step.
///
/// `inject_divergence_at`: test-only comparator non-vacuity knob — after applying step
/// `i` to all three engines, a marker quad is inserted into the OXIGRAPH store only, so
/// the comparator MUST report a divergence at that step (see
/// `tests::injected_divergence_is_caught`). `None` in production.
fn check_seed(
    seed: u64,
    allow: &UpdateAllowlist,
    inject_divergence_at: Option<usize>,
) -> Result<SeedStats, String> {
    let mut rng = Rng::new(seed);
    let (ops, gen) = gen_sequence(&mut rng);
    let written = &gen.integer_lexicals;

    // The LOAD documents are part of the seed: materialize them into a per-seed
    // directory, which is also the allowlisted base the relative `file://` names in the
    // generated ops resolve against.
    let load_dir = if ops.iter().any(|o| !o.docs.is_empty()) {
        let dir = std::env::temp_dir().join(format!("sparq-update-fuzz-seed-{}", seed));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir)
            .map_err(|e| format!("could not create LOAD base {}: {}", dir.display(), e))?;
        Some(LoadDir(dir))
    } else {
        None
    };
    let base = load_dir.as_ref().map(|d| d.0.as_path());

    let mut g_rebuild = Graph::new();
    let mut g_inplace = Graph::new();
    let store = Store::new().map_err(|e| format!("oxigraph store init: {}", e))?;
    let mut stats = SeedStats::default();

    let fail = |step: usize, op: &str, detail: String| -> String {
        format!(
            "step={} of {}\nop: {}\n{}\nrepro: cargo run -p sparq-bench --release -- \
             update-fuzz --seed-start {} --seed-count 1\n--- full sequence ---\n{}",
            step,
            ops.len(),
            op,
            detail,
            seed,
            ops.iter()
                .enumerate()
                .map(|(i, o)| format!("[{}] {}", i, o.text))
                .collect::<Vec<_>>()
                .join("\n")
        )
    };

    for (i, op) in ops.iter().enumerate() {
        for (name, body) in &op.docs {
            let dir = base.ok_or_else(|| fail(i, &op.text, "LOAD base not created".into()))?;
            std::fs::write(dir.join(name), body)
                .map_err(|e| fail(i, &op.text, format!("could not write {}: {}", name, e)))?;
        }

        // Apply to all three implementations. Every generated op is inside the
        // supported deterministic subset, so an error from ANY engine is itself a
        // divergence (strict — there is no unsupported-skip in this harness).
        g_rebuild = with_optional_base(base, || sparq_engine::update(&g_rebuild, &op.text))
            .map_err(|e| fail(i, &op.text, format!("sparq update (rebuild path) error: {}", e)))?;
        with_optional_base(base, || {
            sparq_engine::update_in_place(&mut g_inplace, &op.text)
        })
        .map_err(|e| fail(i, &op.text, format!("sparq update_in_place error: {}", e)))?;
        if let Some(oxi) = &op.oxi {
            store
                .update(oxi.as_str())
                .map_err(|e| fail(i, &op.text, format!("oxigraph update error: {}", e)))?;
        }

        if inject_divergence_at == Some(i) {
            let n = |s: &str| NamedNode::new(s).expect("valid IRI");
            let marker = Quad::new(
                n("http://ex/injected"),
                n("http://ex/injected"),
                n("http://ex/injected"),
                GraphName::DefaultGraph,
            );
            store
                .insert(&marker)
                .map_err(|e| format!("marker insert failed: {}", e))?;
        }

        let q_rebuild = sparq_quads(&g_rebuild).map_err(|e| fail(i, &op.text, e))?;
        let q_inplace = sparq_quads(&g_inplace).map_err(|e| fail(i, &op.text, e))?;
        let q_oxi = oxi_quads(&store).map_err(|e| fail(i, &op.text, e))?;

        // (a) LEXICAL-FORM PRESERVATION (strict, sparq-side). Checked BEFORE the
        // cross-engine compare, because the adjudicated numeric-lexical class is
        // allowed to normalize exactly this property away over there.
        for (label, quads) in [("rebuild", &q_rebuild), ("in-place", &q_inplace)] {
            let bad = unwritten_integer_lexicals(quads, written);
            if !bad.is_empty() {
                return Err(fail(
                    i,
                    &op.text,
                    format!(
                        "sparq ({}) holds xsd:integer lexical form(s) no operation wrote: {:?}\n\
                         a non-canonical lexical must survive round-trip as a DISTINCT RDF term\n\
                         lexicals written by this sequence: {:?}",
                        label, bad, written
                    ),
                ));
            }
        }

        // (b) Dataset equality. sparq-vs-sparq is STRICT (one engine, no adjudication);
        // the two cross-engine compares may fall to an adjudicated class.
        compare_datasets(
            "sparq(rebuild)",
            &q_rebuild,
            "sparq(in-place)",
            &q_inplace,
            false,
        )
        .map_err(|e| fail(i, &op.text, e))?;
        let adj_r = cross_compare("sparq(rebuild)", &q_rebuild, "oxigraph", &q_oxi, allow)
            .map_err(|e| fail(i, &op.text, e))?;
        let adj_i = cross_compare("sparq(in-place)", &q_inplace, "oxigraph", &q_oxi, allow)
            .map_err(|e| fail(i, &op.text, e))?;
        let normalize = adj_r || adj_i;
        if normalize {
            stats.adjudicated += 1;
        }

        // (c) Probe SELECTs — the query-path view of the updated store, full sorted
        // binding sets (never counts). Checked for BOTH sparq graphs: the in-place one
        // reads through the live delta overlay. Blank-node bindings are relabelled
        // through each engine's own RDFC-1.0 issued-identifier map, so they compare by
        // canonical identity rather than by the fresh label the engine happened to
        // mint. The projection follows the step's dataset compare: probing raw when the
        // datasets matched raw, normalized when the adjudicated class was engaged.
        let p_rebuild = projected(&q_rebuild, normalize);
        let p_inplace = projected(&q_inplace, normalize);
        let p_oxi = projected(&q_oxi, normalize);
        let l_rebuild = issued_labels(&p_rebuild).map_err(|e| fail(i, &op.text, e))?;
        let l_inplace = issued_labels(&p_inplace).map_err(|e| fail(i, &op.text, e))?;
        let l_oxi = issued_labels(&p_oxi).map_err(|e| fail(i, &op.text, e))?;

        for (probe, vars) in PROBES {
            let oxi = oxi_probe(&store, probe, vars, &l_oxi, normalize)
                .map_err(|e| fail(i, &op.text, format!("oxigraph probe error: {}", e)))?;
            for (label, g, labels) in [
                ("rebuild", &g_rebuild, &l_rebuild),
                ("in-place", &g_inplace, &l_inplace),
            ] {
                let sparq = sparq_probe(g, probe, labels, normalize)
                    .map_err(|e| fail(i, &op.text, e))?;
                if sparq != oxi {
                    return Err(fail(
                        i,
                        &op.text,
                        format!(
                            "probe {:?} binding set differs (sparq {} vs oxigraph)\n{}",
                            probe,
                            label,
                            one_sided("sparq", &sparq, "oxigraph", &oxi)
                        ),
                    ));
                }
            }
        }
        stats.ops += 1;
    }
    Ok(stats)
}

// ── entry point ──────────────────────────────────────────────────────────────────

pub fn run(seed_start: u64, count: u64) {
    let allow = UpdateAllowlist::load();
    println!("update-fuzz divergence allowlist {}", allow.describe());
    let mut checked = 0u64;
    let mut ops_applied = 0u64;
    let mut adjudicated = 0u64;
    let mut mismatch = 0u64;
    let mut first_repro: Option<String> = None;

    for seed in seed_start..seed_start + count {
        match check_seed(seed, &allow, None) {
            Ok(s) => {
                checked += 1;
                ops_applied += s.ops;
                adjudicated += s.adjudicated;
            }
            Err(detail) => {
                mismatch += 1;
                // One machine-greppable line per failing seed (the contract
                // scripts/ci-file-differential-failure.py parses).
                eprintln!("MISMATCH seed={}", seed);
                if first_repro.is_none() {
                    first_repro = Some(format!("seed={}\n{}", seed, detail));
                }
            }
        }
    }

    println!(
        "update-fuzz seeds {}..{} : checked={} update_requests={} adjudicated_steps={} mismatch={}",
        seed_start,
        seed_start + count,
        checked,
        ops_applied,
        adjudicated,
        mismatch
    );
    if let Some(r) = first_repro {
        println!("\nFIRST FAILING CASE:\n{}", r);
        std::process::exit(1);
    }
}

// ── tests (the per-PR fixed-window smoke + comparator non-vacuity) ────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::BlankNode;

    /// Applies one update request to all three implementations and returns their quad
    /// snapshots — the shared setup for the hand-written extension tests.
    fn apply(op: &str) -> (Vec<Quad>, Vec<Quad>, Vec<Quad>) {
        let g = sparq_engine::update(&Graph::new(), op).expect("sparq update");
        let mut gi = Graph::new();
        sparq_engine::update_in_place(&mut gi, op).expect("sparq update_in_place");
        let store = Store::new().expect("oxigraph store");
        store.update(op).expect("oxigraph update");
        (
            sparq_quads(&g).expect("sparq quads"),
            sparq_quads(&gi).expect("sparq in-place quads"),
            oxi_quads(&store).expect("oxigraph quads"),
        )
    }

    fn lines(quads: &[Quad]) -> Vec<String> {
        let mut v: Vec<String> = quads.iter().map(|q| format!("{} .", q)).collect();
        v.sort();
        v
    }

    fn permissive() -> UpdateAllowlist {
        UpdateAllowlist {
            numeric_lexical: true,
            source: "test".into(),
        }
    }

    /// The per-PR BLOCKING smoke: a fixed seed window through the full three-way
    /// differential (both sparq update paths vs Oxigraph, canonical dataset + lexical
    /// preservation + probe binding sets per step) under the COMMITTED allowlist. The
    /// randomized soak (advancing window) is the nightly `differential-update.yml` lane,
    /// not this test — keep this window small enough to stay fast in debug builds.
    #[test]
    fn fixed_window_smoke() {
        let allow = UpdateAllowlist::load();
        for seed in 0..40 {
            if let Err(detail) = check_seed(seed, &allow, None) {
                panic!("seed={} diverged:\n{}", seed, detail);
            }
        }
    }

    /// Comparator NON-VACUITY: a quad present in only one store MUST be reported as
    /// a divergence at exactly the injected step. If this test ever passes with the
    /// comparison logic hollowed out, the whole harness is asserting nothing.
    #[test]
    fn injected_divergence_is_caught() {
        let err = check_seed(0, &permissive(), Some(0))
            .expect_err("an injected extra quad must diverge");
        assert!(
            err.contains("canonical dataset differs"),
            "divergence must be reported by the canonical-dataset compare, got:\n{}",
            err
        );
        assert!(
            err.contains("step=0"),
            "divergence must be localized to the injected step, got:\n{}",
            err
        );
        assert!(
            err.contains("http://ex/injected"),
            "the report must show the offending quad, got:\n{}",
            err
        );
    }

    /// The generator is a pure function of the seed — including the LOAD document
    /// bodies, which are part of the repro contract (the seed IS the corpus).
    #[test]
    fn generator_is_deterministic() {
        let (a, sa) = gen_sequence(&mut Rng::new(12345));
        let (b, sb) = gen_sequence(&mut Rng::new(12345));
        assert_eq!(a, b);
        assert_eq!(sa.integer_lexicals, sb.integer_lexicals);
        assert!(a.len() >= 4 && a.len() <= 10, "sequence length in 4..=10");
    }

    /// The SMOKE WINDOW must actually EMIT each extension. Deliberately the SAME
    /// 0..40 range as `fixed_window_smoke`, so the per-PR blocking test is known to
    /// exercise all four v2 extensions — a generator refactor that silently stopped
    /// producing (say) triple terms could otherwise leave the extension untested while
    /// the smoke stayed green.
    #[test]
    fn the_smoke_window_emits_every_v2_extension() {
        let (mut bnodes, mut noncanon, mut load, mut triple_terms) = (false, false, false, false);
        for seed in 0..40u64 {
            let (ops, st) = gen_sequence(&mut Rng::new(seed));
            noncanon |= st
                .integer_lexicals
                .iter()
                .any(|l| (l.starts_with('0') && l.len() > 1) || l.starts_with('+'));
            for op in &ops {
                bnodes |= op.text.contains("_:b");
                load |= op.text.contains("LOAD ");
                triple_terms |= op.text.contains("<<(");
            }
        }
        assert!(bnodes, "generator must emit blank nodes (extension 2)");
        assert!(
            noncanon,
            "generator must emit non-canonical integer lexicals (extension 1)"
        );
        assert!(load, "generator must emit LOAD (extension 3)");
        assert!(
            triple_terms,
            "generator must emit RDF 1.2 triple terms (extension 4)"
        );
    }

    /// The snapshot renderers agree on a concrete mixed default/named-graph dataset
    /// built through BOTH engines' own update paths — the byte-level foundation the
    /// canonical compare rests on (IRIs, plain + typed + tagged literals, named-graph
    /// rendering).
    #[test]
    fn snapshot_renderers_agree_on_known_dataset() {
        let op = "INSERT DATA { <http://ex/s0> <http://ex/p0> \"lit0\" . \
                  <http://ex/s1> <http://ex/p1> 7 . \
                  GRAPH <http://ex/g0> { <http://ex/s2> <http://ex/p2> \"tag0\"@en . } }";
        let (r, ip, oxi) = apply(op);
        assert_eq!(lines(&r), lines(&oxi));
        assert_eq!(lines(&r), lines(&ip));
        assert_eq!(r.len(), 3);
        let a = lines(&r);
        assert!(
            a.iter().any(|l| l.ends_with("<http://ex/g0> .")),
            "named-graph quad must carry its graph IRI: {:?}",
            a
        );
        assert!(
            a.iter()
                .any(|l| l.contains("\"7\"^^<http://www.w3.org/2001/XMLSchema#integer>")),
            "bare integer must render as a typed literal on both sides: {:?}",
            a
        );
    }

    // ── extension 1: non-canonical numeric lexical forms ─────────────────────────

    /// The property the extension exists for: `"0105"`, `"+105"` and `"105"` typed
    /// `xsd:integer` are THREE DISTINCT RDF terms, and both sparq paths must keep all
    /// three with their exact lexical forms. This also PINS the adjudicated class's
    /// premise — Oxigraph collapses them to one — so if Oxigraph ever stops normalizing,
    /// this test goes red and the class must be removed from the registry rather than
    /// lingering as an unearned exemption.
    #[test]
    fn noncanonical_integer_lexicals_survive_as_distinct_sparq_terms() {
        let int = XSD_INTEGER_IRI;
        let op = format!(
            "INSERT DATA {{ <http://ex/s> <http://ex/p> \"0105\"^^<{0}> . \
             <http://ex/s> <http://ex/p> \"+105\"^^<{0}> . \
             <http://ex/s> <http://ex/p> \"105\"^^<{0}> }}",
            int
        );
        let (r, ip, oxi) = apply(&op);
        assert_eq!(lines(&r), lines(&ip), "both sparq paths must agree");
        assert_eq!(r.len(), 3, "three distinct RDF terms, three quads: {:?}", lines(&r));
        for lex in ["0105", "+105", "105"] {
            assert!(
                lines(&r).iter().any(|l| l.contains(&format!("\"{}\"^^<{}>", lex, int))),
                "sparq must preserve the lexical form {:?}: {:?}",
                lex,
                lines(&r)
            );
        }
        assert_eq!(
            oxi.len(),
            1,
            "premise of the adjudicated class: oxigraph normalizes all three to one \
             term — if this now differs, delete the class from the registry: {:?}",
            lines(&oxi)
        );
    }

    /// The strict sparq-side oracle is NON-VACUOUS: it is silent on a store that
    /// preserved the written lexicals and fires on one that canonicalized them. Oxigraph
    /// supplies the canonicalizing store, so this is a live counterexample rather than a
    /// hand-built mock.
    #[test]
    fn unwritten_integer_lexical_oracle_is_non_vacuous() {
        let op = format!(
            "INSERT DATA {{ <http://ex/s> <http://ex/p> \"0105\"^^<{}> }}",
            XSD_INTEGER_IRI
        );
        let (r, _, oxi) = apply(&op);
        let written: BTreeSet<String> = ["0105".to_string()].into_iter().collect();
        assert!(
            unwritten_integer_lexicals(&r, &written).is_empty(),
            "sparq wrote only the lexical it was given"
        );
        assert_eq!(
            unwritten_integer_lexicals(&oxi, &written),
            vec!["105".to_string()],
            "a canonicalizing store must be caught by the preservation oracle"
        );
    }

    /// The adjudication is gated by the FILE, and it is a RETRY, not a skip: with the
    /// class enabled a pure numeric-lexical difference is absorbed; with it disabled the
    /// very same compare FAILS and reports the RAW diff.
    #[test]
    fn numeric_lexical_class_is_a_gated_retry_not_a_skip() {
        let op = format!(
            "INSERT DATA {{ <http://ex/s> <http://ex/p> \"0105\"^^<{}> }}",
            XSD_INTEGER_IRI
        );
        let (r, _, oxi) = apply(&op);
        assert!(
            cross_compare("sparq", &r, "oxigraph", &oxi, &permissive()).expect("absorbed"),
            "with the class enabled the numeric-lexical difference is adjudicated"
        );
        let strict = UpdateAllowlist::from_json(r#"{"version":1,"classes":[]}"#, "test");
        let err = cross_compare("sparq", &r, "oxigraph", &oxi, &strict)
            .expect_err("with the class disabled the compare must FAIL");
        assert!(
            err.contains("\"0105\"") && err.contains("\"105\""),
            "the failure must show the RAW lexical diff, got:\n{}",
            err
        );
    }

    /// The retry absorbs ONLY the lexical forms: a difference that survives value
    /// normalization still fails, even with the class enabled.
    #[test]
    fn numeric_lexical_retry_does_not_absorb_a_real_difference() {
        let int = XSD_INTEGER_IRI;
        let a = sparq_quads(
            &sparq_engine::update(
                &Graph::new(),
                &format!(
                    "INSERT DATA {{ <http://ex/s> <http://ex/p> \"0105\"^^<{}> }}",
                    int
                ),
            )
            .unwrap(),
        )
        .unwrap();
        let b = sparq_quads(
            &sparq_engine::update(
                &Graph::new(),
                &format!(
                    "INSERT DATA {{ <http://ex/s> <http://ex/p> \"0106\"^^<{}> }}",
                    int
                ),
            )
            .unwrap(),
        )
        .unwrap();
        let err = cross_compare("a", &a, "b", &b, &permissive())
            .expect_err("105 vs 106 is a real difference, not a lexical one");
        assert!(err.contains("canonical dataset differs"), "got:\n{}", err);
    }

    /// The projection must not rewrite an ILL-TYPED `xsd:integer` literal: `"abc"^^
    /// xsd:integer` is a perfectly good (if ill-typed) RDF term with no value, so
    /// normalizing it would let a genuine divergence hide behind the class.
    #[test]
    fn normalization_leaves_ill_typed_integers_alone() {
        let bad = Literal::new_typed_literal("abc", xsd::INTEGER);
        assert!(canonical_integer(&bad).is_none());
        assert!(canonical_integer(&Literal::new_typed_literal("105", xsd::INTEGER)).is_none());
        assert_eq!(
            canonical_integer(&Literal::new_typed_literal("0105", xsd::INTEGER))
                .map(|l| l.value().to_string()),
            Some("105".to_string())
        );
    }

    /// The COMMITTED allowlist must enable exactly the update classes this comparator
    /// has a detector for — an entry without one would be a claimed-but-unenforced
    /// allowlisting, and a detector without an entry would be an unregistered exemption.
    #[test]
    fn committed_allowlist_enables_exactly_the_update_classes() {
        let path = concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/../../bench/differential-divergences.json"
        );
        let s = std::fs::read_to_string(path).expect("committed allowlist readable");
        assert!(
            UpdateAllowlist::from_json(&s, path).numeric_lexical,
            "the sq-hodke numeric-lexical class must be listed"
        );
        let v: serde_json::Value = serde_json::from_str(&s).unwrap();
        for c in v["classes"].as_array().expect("classes array") {
            let id = c["id"].as_str().expect("string id");
            if !id.starts_with("update-") {
                continue;
            }
            assert_eq!(id, NUMERIC_LEXICAL_CLASS, "no detector in update_fuzz.rs");
            assert!(
                c["bead"].as_str().is_some_and(|b| b.starts_with("sq-")),
                "class {:?} must cite its adjudication bead",
                id
            );
        }
    }

    // ── extension 2: blank nodes / isomorphism adjudication ──────────────────────

    /// Blank-node datasets are adjudicated UP TO ISOMORPHISM, not by label equality:
    /// the two engines mint completely different fresh labels for the same INSERT DATA
    /// (sparq's `_:fbN` counter is even process-global), and the compare must still pass.
    #[test]
    fn blank_node_datasets_compare_up_to_isomorphism() {
        let op = "INSERT DATA { _:b0 <http://ex/p> _:b1 . _:b1 <http://ex/q> \"v\" . \
                  GRAPH <http://ex/g0> { _:b0 <http://ex/p> <http://ex/o> } }";
        let (r, ip, oxi) = apply(op);
        assert_ne!(
            lines(&r),
            lines(&oxi),
            "the premise: the engines' raw blank-node labels differ"
        );
        compare_datasets("sparq(rebuild)", &r, "oxigraph", &oxi, false).expect("isomorphic");
        compare_datasets("sparq(rebuild)", &r, "sparq(in-place)", &ip, false).expect("isomorphic");
    }

    /// …and isomorphism is not a rubber stamp: same quad COUNT, same ground terms, but a
    /// different blank-node structure (one shared node vs two distinct ones) still FAILS.
    #[test]
    fn non_isomorphic_blank_node_datasets_fail() {
        let shared = sparq_quads(
            &sparq_engine::update(
                &Graph::new(),
                "INSERT DATA { _:a <http://ex/p> <http://ex/o1> . _:a <http://ex/p> <http://ex/o2> }",
            )
            .unwrap(),
        )
        .unwrap();
        let mut g = sparq_engine::update(
            &Graph::new(),
            "INSERT DATA { _:x <http://ex/p> <http://ex/o1> }",
        )
        .unwrap();
        // A SECOND operation, so the engine must mint a DIFFERENT fresh blank node.
        g = sparq_engine::update(&g, "INSERT DATA { _:y <http://ex/p> <http://ex/o2> }").unwrap();
        let split = sparq_quads(&g).unwrap();
        assert_eq!(shared.len(), split.len(), "same cardinality by construction");
        let err = compare_datasets("shared", &shared, "split", &split, false)
            .expect_err("one shared bnode is not isomorphic to two distinct ones");
        assert!(err.contains("NOT RDF-isomorphic"), "got:\n{}", err);
    }

    /// A blank-node CARDINALITY difference is caught even though a set-based
    /// canonicalization would collapse it — the separate count check earns its keep.
    #[test]
    fn blank_node_cardinality_difference_is_caught() {
        let one = sparq_quads(
            &sparq_engine::update(&Graph::new(), "INSERT DATA { _:a <http://ex/p> \"v\" }").unwrap(),
        )
        .unwrap();
        let mut two = one.clone();
        two.push(Quad::new(
            BlankNode::new("extra").unwrap(),
            NamedNode::new("http://ex/p").unwrap(),
            Literal::new_simple_literal("v"),
            GraphName::DefaultGraph,
        ));
        let err = compare_datasets("one", &one, "two", &two, false)
            .expect_err("differing blank-node quad counts must fail");
        assert!(err.contains("COUNT differs"), "got:\n{}", err);
    }

    // ── extension 3: LOAD ────────────────────────────────────────────────────────

    /// sparq's `LOAD <file://…> INTO GRAPH …` must land exactly what an independent
    /// engine's equivalent `INSERT DATA` lands — for both the relative form (which the
    /// generator emits, resolved against the allowlisted base) and the absolute form.
    #[test]
    fn load_matches_its_equivalent_insert_data() {
        let dir = std::env::temp_dir().join("sparq-update-fuzz-test-load");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let guard = LoadDir(dir.clone());
        let body = format!(
            "<http://ex/s0> <http://ex/p0> <http://ex/o0> .\n\
             <http://ex/s1> <http://ex/p1> \"0105\"^^<{}> .\n",
            XSD_INTEGER_IRI
        );
        std::fs::write(dir.join("d.nt"), &body).unwrap();

        let store = Store::new().unwrap();
        store
            .update(&format!(
                "INSERT DATA {{ GRAPH <http://ex/g0> {{ {} }} }}",
                body
            ))
            .expect("oxigraph insert");
        let oxi = oxi_quads(&store).unwrap();

        for source in [
            "file://d.nt".to_string(),
            format!("file://{}", dir.join("d.nt").display()),
        ] {
            let op = format!("LOAD <{}> INTO GRAPH <http://ex/g0>", source);
            let g = sparq_engine::with_load_base(&dir, || sparq_engine::update(&Graph::new(), &op))
                .unwrap_or_else(|e| panic!("sparq LOAD {} failed: {}", source, e));
            let q = sparq_quads(&g).unwrap();
            assert_eq!(q.len(), 2, "both document triples must land");
            // The lexical form survives the LOAD path too; oxigraph normalized its copy,
            // which is precisely the adjudicated class.
            assert!(cross_compare("sparq", &q, "oxigraph", &oxi, &permissive()).unwrap());
            compare_datasets("sparq", &q, "oxigraph", &oxi, true).expect("equal once normalized");
        }
        drop(guard);
        assert!(!dir.exists(), "the LOAD base must be cleaned up on drop");
    }

    /// The absent-document arm: `LOAD SILENT` of a file that is not there is a NO-OP on
    /// sparq (which is why the harness applies nothing to Oxigraph for that step), and
    /// the same LOAD without SILENT is an ERROR — so the no-op is the SILENT contract,
    /// not a swallowed failure.
    #[test]
    fn silent_load_of_an_absent_document_is_a_no_op() {
        let dir = std::env::temp_dir().join("sparq-update-fuzz-test-absent");
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let _guard = LoadDir(dir.clone());
        let g = sparq_engine::with_load_base(&dir, || {
            sparq_engine::update(&Graph::new(), "LOAD SILENT <file://absent1.nt>")
        })
        .expect("SILENT LOAD of an absent document must succeed");
        assert!(sparq_quads(&g).unwrap().is_empty(), "…and change nothing");
        assert!(
            sparq_engine::with_load_base(&dir, || sparq_engine::update(
                &Graph::new(),
                "LOAD <file://absent1.nt>"
            ))
            .is_err(),
            "without SILENT the same LOAD must be an error"
        );
    }

    // ── extension 4: RDF 1.2 triple terms ────────────────────────────────────────

    /// Triple terms round-trip identically through both engines' parsers, stores and
    /// serializers — in the default and a named graph, nested, and through DELETE DATA.
    #[test]
    fn triple_terms_round_trip_through_both_engines() {
        let op = "INSERT DATA { <http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/b> 3 )>> . \
                  <http://ex/s> <http://ex/q> \
                  <<( <http://ex/a> <http://ex/b> <<( <http://ex/c> <http://ex/d> \"x\"@en )>> )>> . \
                  GRAPH <http://ex/g0> { <http://ex/s2> <http://ex/p> \
                  <<( <http://ex/a> <http://ex/b> \"y\" )>> } } ;\n\
                  DELETE DATA { <http://ex/s> <http://ex/p> <<( <http://ex/a> <http://ex/b> 3 )>> }";
        let (r, ip, oxi) = apply(op);
        assert_eq!(r.len(), 2, "one of the three quads was deleted: {:?}", lines(&r));
        assert_eq!(lines(&r), lines(&oxi));
        assert_eq!(lines(&r), lines(&ip));
        assert!(
            lines(&r).iter().any(|l| l.contains("<<( <http://ex/c>")),
            "the nested triple term must survive: {:?}",
            lines(&r)
        );
    }

    /// A non-canonical lexical NESTED inside a triple term is normalized by the
    /// projection (so the adjudicated class still covers oxigraph's rewriting there) and
    /// is still counted by the preservation oracle (so sparq is still held to it).
    #[test]
    fn triple_term_objects_participate_in_both_numeric_oracles() {
        let op = format!(
            "INSERT DATA {{ <http://ex/s> <http://ex/p> \
             <<( <http://ex/a> <http://ex/b> \"0105\"^^<{}> )>> }}",
            XSD_INTEGER_IRI
        );
        let (r, _, oxi) = apply(&op);
        let written: BTreeSet<String> = ["0105".to_string()].into_iter().collect();
        assert!(unwritten_integer_lexicals(&r, &written).is_empty());
        assert_eq!(
            unwritten_integer_lexicals(&oxi, &written),
            vec!["105".to_string()],
            "the oracle must see inside triple terms"
        );
        assert!(
            cross_compare("sparq", &r, "oxigraph", &oxi, &permissive()).unwrap(),
            "and so must the normalizing projection"
        );
    }
}
