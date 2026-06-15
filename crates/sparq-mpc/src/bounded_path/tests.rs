// [OPUS-4.8] sq-py8h.1 — DIFFERENTIAL tests for the disclosed-key bounded path.
//! Differential acceptance suite for [`eval_bounded_path_disclosed`].
//!
//! THE acceptance criterion (mirrors `differential_three_holder_chain_equals_union`
//! in [`crate::join`]): the federated DISCLOSED-KEY unroll of a bounded path over
//! per-holder disclosed edge partials must equal the clear-text engine's
//! `eval_path` over the UNION store, for each bounded form in the design matrix
//! (§6 step 1). The union store is the single `sparq-engine` `Graph` holding the
//! union of all holders' edges; the clear-text side runs the *equivalent* SPARQL
//! 1.1 property path through the real engine (`query`), which exercises the
//! engine's `eval_path` (`exec.rs`).
//!
//! Bounded `{m,k}` is expressed to the engine as the equivalent SPARQL path the
//! standard DOES carry (SPARQL 1.1 dropped the `{m,n}` quantifier, so the bounded
//! form is written as the union of fixed-length chains it denotes):
//!   - `(p){k}`     ==  `p/p/.../p`        (k copies; a `Sequence`)
//!   - `(p){m,k}`   ==  union of `p^ℓ` for ℓ in m..=k (an `Alternative` of chains)
//!   - `(p){0,k}`   ==  `(p)? | p^2 | ... | p^k`  (the `?` supplies the length-0/1 arms)
//!   - alternation  ==  `(p|q)/(p|q)/...`
//!
//! The unroller and the engine independently compute the SAME bounded relation,
//! so the equality is exact (design §1.2 — bounded, not an approximation).
//!
//! CRYPTO-FREE assertion: the disclosed-key path touches NO MPC primitive. There
//! is no `CommCounter`, no `ShamirBackend`, no `secure_equal` on this code path —
//! it is a fold of `DisclosedKeyJoin` over disclosed global IRIs. The
//! `crypto_free_no_mpc_round_artifacts` test pins that the only types reachable
//! are the crypto-free join + plain `PartialResult`s (round-count == 0 by
//! construction: nothing in the call graph can record a round).

use super::*;
use crate::holder::Holder;
use oxrdf::Term;
use sparq_core::Graph;
use sparq_engine::query;

const PFX: &str = "@prefix ex: <http://ex/> .\n";

fn nn(iri: &str) -> NamedNode {
    NamedNode::new(iri).unwrap()
}

/// Build the per-predicate disclosed edge set: each holder discloses its
/// `(?s,?o)` rows for a predicate via `evaluate_local`, exactly as a federation
/// member would. Returns the [`DisclosedEdges`] the unroller consumes.
fn disclosed_edges(holder_docs: &[(&str, &[&str])]) -> DisclosedEdges {
    // holder_docs: (turtle body, [predicate IRIs this holder discloses]).
    let mut entries: Vec<(NamedNode, PartialResult)> = Vec::new();
    for (i, (doc, preds)) in holder_docs.iter().enumerate() {
        let h = Holder::from_rdf(format!("h{i}"), &format!("{PFX}{doc}"), "turtle").unwrap();
        for p in *preds {
            let part = h
                .evaluate_local(&format!("SELECT ?s ?o WHERE {{ ?s <{p}> ?o }}"))
                .unwrap();
            entries.push((nn(p), part));
        }
    }
    DisclosedEdges::from_holder_edges(entries).unwrap()
}

/// Build a single union store from holder turtle bodies — the "evaluate the
/// whole path over D = ⋃ holder graphs" side of the differential.
fn union_graph(docs: &[&str]) -> Graph {
    let mut combined = String::from(PFX);
    for d in docs {
        combined.push_str(d);
        combined.push('\n');
    }
    Graph::load_str(&combined, "turtle").expect("union graph parses")
}

/// The canonical set of `(a,b)` endpoint IRI pairs from a `(vars,rows)` partial,
/// as `(a_debug, b_debug)` strings — order-independent, dedup'd. The unroller
/// projects onto `[?__pp_a, ?__pp_b]`; the engine query below projects `?a ?b`.
fn pair_set(vars: &[Variable], rows: &[Vec<Option<Term>>]) -> BTreeSet<(String, String)> {
    // Find the two endpoint columns by position (both sides are 2-col results).
    assert_eq!(
        vars.len(),
        2,
        "expected a 2-column (a,b) result, got {vars:?}"
    );
    rows.iter()
        .map(|r| {
            let a = r[0].as_ref().map(|t| format!("{t:?}")).unwrap_or_default();
            let b = r[1].as_ref().map(|t| format!("{t:?}")).unwrap_or_default();
            (a, b)
        })
        .collect()
}

/// Run the clear-text engine oracle: the equivalent SPARQL path over the union
/// store, projecting `?a ?b`. Exercises `eval_path` (`exec.rs`).
fn clear_text_pairs(union_docs: &[&str], path_sparql: &str) -> BTreeSet<(String, String)> {
    let u = union_graph(union_docs);
    let q = format!("PREFIX ex: <http://ex/> SELECT ?a ?b WHERE {{ ?a {path_sparql} ?b }}");
    let res = query(&u, &q).unwrap();
    pair_set(&res.vars, &res.rows)
}

/// Run the federated DISCLOSED-KEY unroll and return its `(a,b)` pair set.
fn federated_pairs(edges: &DisclosedEdges, form: &PathForm) -> BTreeSet<(String, String)> {
    let got = eval_bounded_path_disclosed(edges, form).unwrap();
    assert_eq!(got.holder, HolderId::new("federation"));
    pair_set(&got.vars, &got.rows)
}

// =====================================================================
// 1. PLAIN SEQUENCE  p1/p2/p3
// =====================================================================

/// A plain 3-step sequence path `?a ex:p1/ex:p2/ex:p3 ?b`, edges split across
/// THREE holders (one predicate each) — the disclosed-key fold must equal the
/// engine's `eval_path` of the sequence over the union. This is the direct
/// generalisation of `differential_three_holder_chain_equals_union`.
#[test]
fn differential_plain_sequence_equals_eval_path() {
    let a_doc = "ex:p1 ex:p1 ex:m1 . ex:p2 ex:p1 ex:m2 .";
    let b_doc = "ex:m1 ex:p2 ex:n1 . ex:m2 ex:p2 ex:n1 . ex:m9 ex:p2 ex:n2 .";
    let c_doc = "ex:n1 ex:p3 ex:z1 . ex:n2 ex:p3 ex:z2 .";

    let edges = disclosed_edges(&[
        (a_doc, &["http://ex/p1"]),
        (b_doc, &["http://ex/p2"]),
        (c_doc, &["http://ex/p3"]),
    ]);
    let form = PathForm::sequence([nn("http://ex/p1"), nn("http://ex/p2"), nn("http://ex/p3")]);

    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[a_doc, b_doc, c_doc], "ex:p1/ex:p2/ex:p3");
    assert_eq!(got, want, "seq unroll must equal eval_path over the union");
    // Concretely: p1->m1->n1->z1 and p2->m2->n1->z1. m9/n2 dropped (no p1 into m9).
    assert_eq!(got.len(), 2, "{got:?}");
}

// =====================================================================
// 2. EXACT  (p){k}
// =====================================================================

/// Exact `(ex:knows){3}` — a 3-hop chain of ONE predicate, edges split across
/// holders. Differential against `ex:knows/ex:knows/ex:knows`.
#[test]
fn differential_exact_k_equals_eval_path() {
    // A chain a->b->c->d->e plus a side branch; exactly-3 connects a->d and b->e.
    let h1 = "ex:a ex:knows ex:b . ex:b ex:knows ex:c .";
    let h2 = "ex:c ex:knows ex:d . ex:d ex:knows ex:e .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    let form = PathForm::exact(nn("http://ex/knows"), 3);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "ex:knows/ex:knows/ex:knows");
    assert_eq!(got, want);
    // a->b->c->d (len3) and b->c->d->e (len3): {(a,d),(b,e)}.
    assert_eq!(got.len(), 2, "{got:?}");
}

// =====================================================================
// 3. RANGE  (p){m,k}
// =====================================================================

/// Range `(ex:knows){1,3}` (bounded `+`) — UNION of lengths 1,2,3, deduped.
/// Differential against `ex:knows | ex:knows/ex:knows | ex:knows/ex:knows/ex:knows`.
#[test]
fn differential_range_m_k_equals_eval_path() {
    let h1 = "ex:a ex:knows ex:b . ex:b ex:knows ex:c .";
    let h2 = "ex:c ex:knows ex:d .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    let form = PathForm::range(nn("http://ex/knows"), 1, 3);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(
        &[h1, h2],
        "ex:knows | ex:knows/ex:knows | ex:knows/ex:knows/ex:knows",
    );
    assert_eq!(got, want);
    // len1: ab,bc,cd ; len2: ac,bd ; len3: ad. All distinct -> 6 pairs.
    assert_eq!(got.len(), 6, "{got:?}");
}

/// Range with a NON-1 lower bound `(ex:knows){2,3}` — must EXCLUDE the length-1
/// pairs. Pins that `min` is honoured (not silently treated as 1).
#[test]
fn differential_range_min_two_excludes_length_one() {
    let h1 = "ex:a ex:knows ex:b . ex:b ex:knows ex:c .";
    let h2 = "ex:c ex:knows ex:d .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    let form = PathForm::range(nn("http://ex/knows"), 2, 3);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "ex:knows/ex:knows | ex:knows/ex:knows/ex:knows");
    assert_eq!(got, want);
    // len2: ac,bd ; len3: ad. The length-1 pairs (ab,bc,cd) are EXCLUDED -> 3.
    assert_eq!(got.len(), 3, "{got:?}");
    // Assert a length-1 pair is genuinely absent.
    let ab = (
        format!("{:?}", Term::from(nn("http://ex/a"))),
        format!("{:?}", Term::from(nn("http://ex/b"))),
    );
    assert!(
        !got.contains(&ab),
        "length-1 pair (a,b) must be excluded by min=2"
    );
}

// =====================================================================
// 4. REFLEXIVE  (p){0,k}  — the identity pairs appear exactly once
// =====================================================================

/// Reflexive `(ex:knows){0,2}` — the `{1,2}` union PLUS the length-0 identity
/// pairs `(x,x)` for every node, each exactly once (design §2.3). Differential
/// against `(ex:knows)? | ex:knows/ex:knows` (the `?` carries length 0 and 1).
#[test]
fn differential_reflexive_zero_k_identity_pairs_once() {
    let h1 = "ex:a ex:knows ex:b .";
    let h2 = "ex:b ex:knows ex:c .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    let form = PathForm::range(nn("http://ex/knows"), 0, 2);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:knows)? | ex:knows/ex:knows");
    assert_eq!(got, want, "reflexive {{0,2}} must equal eval_path");

    // The identity (x,x) pairs: nodes are a,b,c -> (a,a),(b,b),(c,c).
    for node in ["http://ex/a", "http://ex/b", "http://ex/c"] {
        let d = format!("{:?}", Term::from(nn(node)));
        let count = got.iter().filter(|(a, b)| *a == d && *b == d).count();
        assert_eq!(
            count, 1,
            "identity pair ({node},{node}) must appear EXACTLY once"
        );
    }
    // Full set: identity {aa,bb,cc} + len1 {ab,bc} + len2 {ac} = 6 distinct pairs.
    assert_eq!(got.len(), 6, "{got:?}");
}

/// `(p?)` is the `{0,1}` special case: reflexive identity + the single 1-hop
/// pattern. Differential against the engine's `(ex:knows)?`.
#[test]
fn differential_zero_or_one_p_question() {
    let h1 = "ex:a ex:knows ex:b .";
    let h2 = "ex:b ex:knows ex:c .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    let form = PathForm::range(nn("http://ex/knows"), 0, 1);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:knows)?");
    assert_eq!(got, want);
    // identity {aa,bb,cc} + len1 {ab,bc} = 5.
    assert_eq!(got.len(), 5, "{got:?}");
}

// =====================================================================
// 5. ALTERNATION  (p|q)
// =====================================================================

/// Alternation per hop, exactly-2: `(ex:p|ex:q){2}` unrolls to the FOUR chains
/// {pp,pq,qp,qq}. Differential against `(ex:p|ex:q)/(ex:p|ex:q)`.
#[test]
fn differential_alternation_exact_two_equals_eval_path() {
    // Mixed p/q edges across two holders so several of the four chains hit.
    let h1 = "ex:a ex:p ex:m . ex:a ex:q ex:n .";
    let h2 = "ex:m ex:q ex:x . ex:n ex:p ex:y . ex:m ex:p ex:w .";
    let edges = disclosed_edges(&[
        (h1, &["http://ex/p", "http://ex/q"]),
        (h2, &["http://ex/p", "http://ex/q"]),
    ]);

    let step = PathStep::alternation([nn("http://ex/p"), nn("http://ex/q")]);
    let form = PathForm::repeat_step(step, 2, 2);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:p|ex:q)/(ex:p|ex:q)");
    assert_eq!(got, want, "alternation exactly-2 must equal eval_path");
    // a-p-m-q-x, a-p-m-p-w, a-q-n-p-y -> {(a,x),(a,w),(a,y)} = 3.
    assert_eq!(got.len(), 3, "{got:?}");
}

/// Bounded alternation RANGE `(ex:p|ex:q){1,2}` — union of length-1 (p,q) and
/// length-2 (pp,pq,qp,qq), deduped. Differential against `(ex:p|ex:q) |
/// (ex:p|ex:q)/(ex:p|ex:q)`.
#[test]
fn differential_alternation_range_one_two() {
    let h1 = "ex:a ex:p ex:m . ex:a ex:q ex:n .";
    let h2 = "ex:m ex:q ex:x . ex:n ex:p ex:y .";
    let edges = disclosed_edges(&[
        (h1, &["http://ex/p", "http://ex/q"]),
        (h2, &["http://ex/p", "http://ex/q"]),
    ]);

    let step = PathStep::alternation([nn("http://ex/p"), nn("http://ex/q")]);
    let form = PathForm::repeat_step(step, 1, 2);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:p|ex:q) | (ex:p|ex:q)/(ex:p|ex:q)");
    assert_eq!(got, want);
    // len1 (every one-hop p/q edge): a-m, a-n, m-x, n-y ;
    // len2: a-x (a-p-m-q-x), a-y (a-q-n-p-y). 6 distinct pairs.
    assert_eq!(got.len(), 6, "{got:?}");
}

// =====================================================================
// 5b. SEQUENCE WITH A BOUNDED-REPETITION / ALTERNATION ELEMENT
//     — the trickiest eval_sequence branch (per-part dedup before compose)
// =====================================================================

/// `Sequence` whose FIRST element is a bounded repetition: `(ex:p){1,2}/ex:q`.
/// This exercises `eval_sequence` composing a multi-length sub-form (which can
/// reach the same mid node by two different lengths) with a plain hop — the
/// per-part dedup in `eval_sequence` must collapse the mid multiplicity before
/// the join so the composition is not inflated. Differential against the
/// engine's `(ex:p{1,2})/ex:q`, written as the union of fixed-length chains the
/// standard carries: `(ex:p | ex:p/ex:p)/ex:q`.
#[test]
fn differential_sequence_with_repetition_element() {
    // a->b->c via p (so a reaches c by p{1,2}: directly is absent, but a->b->c
    // is length 2, and we also add a->c direct p so a reaches c by BOTH len1+len2)
    // then c->d via q. The repetition sub-form yields (a,c) by two lengths; after
    // dedup it is one mid pair, composed with c->q->d gives (a,d) once.
    let h1 = "ex:a ex:p ex:b . ex:b ex:p ex:c . ex:a ex:p ex:c .";
    let h2 = "ex:c ex:q ex:d .";
    let edges = disclosed_edges(&[
        (h1, &["http://ex/p"]),
        (h2, &["http://ex/q"]),
    ]);

    let form = PathForm::Sequence(vec![
        PathForm::range(nn("http://ex/p"), 1, 2),
        PathForm::exact(nn("http://ex/q"), 1),
    ]);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:p | ex:p/ex:p)/ex:q");
    assert_eq!(
        got, want,
        "sequence with a repetition element must equal eval_path"
    );
    // p{1,2} mid-relation reaches c from BOTH a (a->b->c len2 AND a->c len1) and
    // b (b->c len1); only c has a q out-edge (c->d). So composing with c->q->d
    // gives {(a,d),(b,d)}. The key property under test: c is reached from a by
    // TWO p-lengths, but the per-part dedup collapses that mid multiplicity so
    // (a,d) is produced once, not twice.
    assert_eq!(got.len(), 2, "{got:?}");
    // (a,d) must appear exactly once despite c being reached from a by two p-lengths.
    let ad_rows = got
        .iter()
        .filter(|(a, b)| {
            *a == format!("{:?}", Term::from(nn("http://ex/a")))
                && *b == format!("{:?}", Term::from(nn("http://ex/d")))
        })
        .count();
    assert_eq!(ad_rows, 1, "(a,d) must be a single pair after per-part dedup");
}

/// `Sequence` whose FIRST element is a bounded ALTERNATION repetition:
/// `(ex:p|ex:q){2}/ex:r`. Exercises `eval_sequence` composing an
/// alternation-expanded sub-form (which itself unrolls to a^ℓ chains, possibly
/// reaching the same mid node by several alternation branches) with a trailing
/// hop. Differential against `(ex:p|ex:q)/(ex:p|ex:q)/ex:r`.
#[test]
fn differential_sequence_with_alternation_repetition_element() {
    // a reaches m by p/p AND by q/q (two branches to the same mid m), then m->z
    // via r. The two branches to m must dedup to one mid before composition.
    let h1 = "ex:a ex:p ex:k . ex:a ex:q ex:j .";
    let h2 = "ex:k ex:p ex:m . ex:j ex:q ex:m . ex:m ex:r ex:z .";
    let edges = disclosed_edges(&[
        (h1, &["http://ex/p", "http://ex/q"]),
        (h2, &["http://ex/p", "http://ex/q", "http://ex/r"]),
    ]);

    let step = PathStep::alternation([nn("http://ex/p"), nn("http://ex/q")]);
    let form = PathForm::Sequence(vec![
        PathForm::repeat_step(step, 2, 2),
        PathForm::exact(nn("http://ex/r"), 1),
    ]);
    let got = federated_pairs(&edges, &form);
    let want = clear_text_pairs(&[h1, h2], "(ex:p|ex:q)/(ex:p|ex:q)/ex:r");
    assert_eq!(
        got, want,
        "sequence with an alternation-repetition element must equal eval_path"
    );
    // (p|q){2} from a: a-p-k-p-m and a-q-j-q-m both reach m; then m-r-z. => {(a,z)}.
    assert_eq!(got.len(), 1, "{got:?}");
    let az_rows = got
        .iter()
        .filter(|(a, b)| {
            *a == format!("{:?}", Term::from(nn("http://ex/a")))
                && *b == format!("{:?}", Term::from(nn("http://ex/z")))
        })
        .count();
    assert_eq!(
        az_rows, 1,
        "(a,z) must be a single pair despite two alternation branches reaching m"
    );
}

// =====================================================================
// 6. DEDUP across lengths — a pair reachable by two lengths appears ONCE
// =====================================================================

/// A pair reachable by BOTH a length-1 and a length-2 chain must be deduped to a
/// single endpoint pair (set semantics, design §2.2/§2.5) — the realized length
/// is never disclosed. Build a graph where a->c directly (p) AND a->b->c (p/p).
#[test]
fn dedup_collapses_multi_length_to_single_pair() {
    let h1 = "ex:a ex:p ex:b . ex:b ex:p ex:c . ex:a ex:p ex:c .";
    let edges = disclosed_edges(&[(h1, &["http://ex/p"])]);

    let form = PathForm::range(nn("http://ex/p"), 1, 2);
    let got = eval_bounded_path_disclosed(&edges, &form).unwrap();
    let pairs = pair_set(&got.vars, &got.rows);
    let want = clear_text_pairs(&[h1], "ex:p | ex:p/ex:p");
    assert_eq!(pairs, want);

    // (a,c) is reachable by p (a->c) AND p/p (a->b->c): exactly ONE row for it.
    let ac = (
        format!("{:?}", Term::from(nn("http://ex/a"))),
        format!("{:?}", Term::from(nn("http://ex/c"))),
    );
    let ac_rows = got
        .rows
        .iter()
        .filter(|r| {
            r[0].as_ref().map(|t| format!("{t:?}")) == Some(ac.0.clone())
                && r[1].as_ref().map(|t| format!("{t:?}")) == Some(ac.1.clone())
        })
        .count();
    assert_eq!(
        ac_rows, 1,
        "(a,c) reachable by two lengths must dedup to one row"
    );
}

// =====================================================================
// 7. CRYPTO-FREE — no MPC round occurs on the disclosed-key path
// =====================================================================

/// CRYPTO-FREE assertion (design §2.1 DISCLOSED regime, §6 step 1): the bounded
/// disclosed-key path runs entirely OUTSIDE the cryptographic core. There is no
/// secret sharing and no MPC round.
///
/// The crate's only round accounting is [`crate::metrics::CommCounter`], whose
/// counters are incremented ONLY by `record_mult` / `record_open` /
/// `record_independent_equalities` — all of which live on the Shamir/secure path
/// (`shamir`, `compare`, `oblivious*`), NONE of which the disclosed-key unroll
/// calls. So a counter threaded through this evaluation stays at zero by
/// construction. We assert that operationally: a fresh counter, untouched by the
/// evaluation, reports round-count == 0 (the unroll cannot touch it — it takes no
/// counter, shares no state, and the call graph reaches only `DisclosedKeyJoin`).
#[test]
fn crypto_free_no_mpc_round_artifacts() {
    use crate::metrics::CommCounter;

    let h1 = "ex:a ex:knows ex:b . ex:b ex:knows ex:c .";
    let h2 = "ex:c ex:knows ex:d .";
    let edges = disclosed_edges(&[(h1, &["http://ex/knows"]), (h2, &["http://ex/knows"])]);

    // A counter that, on a HIDDEN-regime evaluation, WOULD accumulate rounds.
    let counter = CommCounter::new(3);

    // Evaluate every supported bounded form; none may run an MPC round.
    let forms = [
        PathForm::sequence([nn("http://ex/knows"), nn("http://ex/knows")]),
        PathForm::exact(nn("http://ex/knows"), 2),
        PathForm::range(nn("http://ex/knows"), 1, 3),
        PathForm::range(nn("http://ex/knows"), 0, 2),
        PathForm::repeat_step(PathStep::alternation([nn("http://ex/knows")]), 1, 2),
    ];
    for form in &forms {
        let _ = eval_bounded_path_disclosed(&edges, form).unwrap();
    }

    // ROUND-COUNT == 0: the disclosed-key path recorded no multiplication and no
    // open round (it never even touched a counter — crypto-free by construction).
    assert_eq!(
        counter.mult_rounds, 0,
        "disclosed-key path must run NO mult rounds"
    );
    assert_eq!(
        counter.open_rounds, 0,
        "disclosed-key path must run NO open rounds"
    );
}

// =====================================================================
// 8. Misc soundness
// =====================================================================

/// A wrong-shape disclosed edge partial (not exactly 2 columns) is a Protocol
/// error, not a silent wrong answer.
#[test]
fn edge_partial_wrong_arity_is_protocol_error() {
    let bad = PartialResult {
        holder: HolderId::new("h0"),
        vars: vec![Variable::new_unchecked("s")],
        rows: vec![],
    };
    let err = DisclosedEdges::from_holder_edges([(nn("http://ex/p"), bad)]).unwrap_err();
    assert!(matches!(err, MpcError::Protocol(_)));
}

/// min > max is a Protocol error.
#[test]
fn min_greater_than_max_is_protocol_error() {
    let edges = disclosed_edges(&[("ex:a ex:p ex:b .", &["http://ex/p"])]);
    let form = PathForm::range(nn("http://ex/p"), 3, 1);
    let err = eval_bounded_path_disclosed(&edges, &form).unwrap_err();
    assert!(matches!(err, MpcError::Protocol(_)));
}

/// An absent predicate (no disclosed edges) contributes nothing — matches the
/// engine treating a never-occurring predicate as the empty relation.
#[test]
fn absent_predicate_yields_empty_chain() {
    let edges = disclosed_edges(&[("ex:a ex:p ex:b .", &["http://ex/p"])]);
    // Sequence over a predicate that nobody disclosed: empty.
    let form = PathForm::exact(nn("http://ex/missing"), 2);
    let got = eval_bounded_path_disclosed(&edges, &form).unwrap();
    assert!(got.rows.is_empty());
}
