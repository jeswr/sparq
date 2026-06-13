// [OPUS-4.8] GlobalJoin trait + disclosed-key equi-join over GLOBAL IRIs (M2).
//! The cross-holder join protocol.
//!
//! Architecture refs: §2 convention #6 (GLOBAL IRIs as cross-credential join
//! keys — the distinguishing feature vs all prior graph-MPC), §4.3 step 4 (the
//! join model: key-on-key → (circuit-)PSI; non-key → oblivious join with
//! bounded intermediate size), §3.1 (PSI does NOT compose for free into a
//! multi-pattern BGP), and OPEN QUESTION **§5.2 Q3** (BGP-join obliviousness
//! cost).
//!
//! ## Why this is its own boundary
//!
//! GOOSE/SMPG/PPMQ all join on *node-local* (Cypher) identifiers, which
//! disqualifies them for federation: a node id is meaningful only inside one
//! database. The contribution here is joining on **global IRIs** that mean the
//! same thing across independent holders (architecture §2 convention #6). That
//! makes the join key a public, dereferenceable identifier — which interacts
//! directly with the no-proof-of-revealed-properties rule (#4): where the join
//! KEY is disclosed, the join can be checked OUTSIDE the cryptographic core (a
//! plaintext check over disclosed IRIs); only where joined VALUES must stay
//! hidden does the join go into the MPC. Q3 is exactly how much of the
//! per-pattern obliviousness padding that out-of-circuit handling collapses,
//! and for which SPARQL fragment (RQ2b).
//!
//! ## Status — M2: the disclosed-key path is REAL
//!
//! [`DisclosedKeyJoin`] implements [`GlobalJoin`] for the
//! `JoinPlan::key_disclosed == true` regime: a crypto-free equi-join over the
//! disclosed global-IRI key. Because a global IRI is a stable, public
//! cross-holder identifier, joining on it needs NO MPC primitive — it is a
//! plaintext check OUTSIDE the cryptographic core (convention #4), and is
//! *invariant to both design forks* (Q1/Q2). This is the PLAN's first M2 target.
//!
//! The **hidden-value** path (`key_disclosed == false`, where the join values
//! are PRIVATE and must enter a circuit-PSI / oblivious join) stays an honest
//! [`MpcError::NotYetImplemented`]: it is gated on a chosen
//! [`crate::backend::MpcBackend`] (M3) and on the Q2 trust-model fork + the Q3
//! BGP-join obliviousness-cost analysis (RQ2b). No fake crypto here.

use crate::partial::{HolderId, MpcError, PartialResult};
use oxrdf::{Term, Variable};
use std::collections::BTreeMap;

/// Describes ONE cross-holder join: the variable to join on and whether its
/// bound values are disclosed (so the join can be checked in the clear) or
/// hidden (so it must run inside the MPC). This is pure data — produced by the
/// (untrusted, §4.1) planner and consumed by a [`GlobalJoin`] impl. The planner
/// is untrusted: a [`GlobalJoin`] must not rely on this plan for *soundness*,
/// only as a hint for *which* protocol to run.
#[derive(Debug, Clone)]
pub struct JoinPlan {
    /// The shared variable the holders' partials are joined on.
    pub join_var: Variable,
    /// `true` if `join_var`'s bound values are disclosed global IRIs (key-on-key
    /// equi-join checkable in the clear, §4.3 step 4); `false` if they are
    /// hidden values requiring an oblivious / PSI join inside the MPC.
    pub key_disclosed: bool,
}

/// The protocol that joins multiple holders' [`PartialResult`]s on a global IRI.
///
/// Architecture §4.3 step 4: cross-source joins on global IRIs are the heart of
/// federated evaluation. Two regimes the eventual impl must distinguish:
///
/// - **Disclosed-key join** (`JoinPlan::key_disclosed == true`): a plaintext
///   equi-join over the disclosed IRIs, computed OUTSIDE the cryptographic core
///   per convention #4. This sub-case is *invariant to Q1/Q2* (no secret data)
///   and is the natural first target within M2.
/// - **Hidden-value join** (`key_disclosed == false`): requires circuit-PSI /
///   oblivious join inside a [`crate::backend::MpcBackend`]; gated on Q2/Q3 and
///   far heavier.
///
/// ## Status — M2
/// [`DisclosedKeyJoin`] implements this for the disclosed-key regime (crypto-
/// free, the PLAN's first M2 target). The hidden path returns
/// [`MpcError::NotYetImplemented`] until the backend (M3) and Q2/Q3 land.
pub trait GlobalJoin {
    /// Join holders' partials according to `plan`, returning the combined
    /// disclosed result.
    ///
    /// A `GlobalJoin` must not rely on the (untrusted, §4.1) `plan` for
    /// *soundness* — only as a hint for *which* protocol to run. The disclosed-
    /// key implementation therefore independently checks that the named
    /// `join_var` actually appears in every partial it is asked to join, and
    /// performs full SPARQL compatible-mapping semantics over *all* shared
    /// columns (not just the planner-named key), so a malicious planner cannot
    /// induce a result that disagrees with PAG evaluation over the union.
    fn join(&self, partials: &[PartialResult], plan: &JoinPlan) -> Result<PartialResult, MpcError>;
}

/// The disclosed-key cross-holder equi-join (M2; crypto-free).
///
/// Architecture §4.3 step 4 "key-on-key": each holder has already disclosed a
/// [`PartialResult`] (via [`crate::holder::Holder::evaluate_local`]) carrying
/// only the join key + the columns its fragment projects — minimise data
/// sharing (§4.2). Because the join key is a GLOBAL IRI (convention #6) and is
/// disclosed, joining is a plain equi-join over plaintext IRIs, done OUTSIDE
/// any cryptographic core (convention #4 no-proof-of-revealed-properties). No
/// MPC primitive is needed; the result is *invariant* to the Q1/Q2 forks.
///
/// ## Semantics — a faithful SPARQL join, not a naive key-merge
///
/// The result is the natural inner join of the partials under SPARQL
/// *compatible-mappings* semantics (Pérez–Arenas–Gutiérrez): two rows combine
/// iff they agree on EVERY shared variable they both bind. The planner names
/// one `join_var` (the global-IRI key it expects to be present and disclosed),
/// but soundness does not trust that name — the join still enforces agreement
/// on any *other* variable two partials happen to share, exactly as evaluating
/// the whole BGP over the union of the holders' graphs would. This is what
/// makes the differential test (join == union-store evaluation) hold.
///
/// Join is folded left-to-right over `partials`; SPARQL join is associative and
/// commutative up to row order, and this impl sorts its output canonically so
/// the result is order-independent (the disclosed multiset is what convention
/// #4 lets the verifier recompute aggregates/ordering over).
#[derive(Debug, Default, Clone, Copy)]
pub struct DisclosedKeyJoin;

impl DisclosedKeyJoin {
    pub fn new() -> Self {
        DisclosedKeyJoin
    }
}

impl GlobalJoin for DisclosedKeyJoin {
    fn join(&self, partials: &[PartialResult], plan: &JoinPlan) -> Result<PartialResult, MpcError> {
        // The hidden-value path is the deferred, fork-gated one: PRIVATE join
        // values that must enter a circuit-PSI / oblivious join. No fake crypto.
        if !plan.key_disclosed {
            return Err(MpcError::not_yet(
                "hidden-value (private-key) cross-holder join — circuit-PSI / oblivious join",
                "M3 MpcBackend + Q2 (trust model) + Q3 (BGP-join obliviousness cost, RQ2b)",
            ));
        }

        // --- Preconditions (real, M2-checkable; NOT trusting the planner). ---
        if partials.is_empty() {
            return Err(MpcError::Protocol(
                "disclosed-key join needs at least one partial".into(),
            ));
        }
        // Soundness obligation (§4.1): independently verify the named key is
        // actually projected by every partial — a planner that names a key a
        // holder did not disclose must FAIL, not silently yield an empty join.
        for p in partials {
            if !p.vars.contains(&plan.join_var) {
                return Err(MpcError::Protocol(format!(
                    "holder {} did not disclose the join key ?{} (vars: {})",
                    p.holder,
                    plan.join_var.as_str(),
                    p.vars
                        .iter()
                        .map(|v| format!("?{}", v.as_str()))
                        .collect::<Vec<_>>()
                        .join(", "),
                )));
            }
        }

        // --- Fold the equi-join left-to-right. ---
        // The accumulator is itself a PartialResult; the federated result is
        // attributed to a synthetic "federation" holder (it is no single
        // holder's data — it is the disclosed cross-holder result).
        let mut acc = partials[0].clone();
        for next in &partials[1..] {
            acc = join_two(&acc, next)?;
        }

        // Canonical row order so the disclosed multiset is independent of holder
        // / row ordering (convention #4: the verifier recomputes ordering).
        canonicalize_rows(&mut acc.rows);
        acc.holder = HolderId::new("federation");
        Ok(acc)
    }
}

/// Join two partials under SPARQL compatible-mapping semantics. The output
/// schema is the union of the two variable lists (shared vars appear once, in
/// left-then-new order); two rows combine iff they agree on every shared var
/// they both bind (SPARQL treats an unbound var as compatible with anything).
fn join_two(left: &PartialResult, right: &PartialResult) -> Result<PartialResult, MpcError> {
    // Output schema: left vars, then right vars not already present.
    let mut out_vars = left.vars.clone();
    for v in &right.vars {
        if !out_vars.contains(v) {
            out_vars.push(v.clone());
        }
    }

    // Positions of each shared variable in left and right rows.
    // (var, left_idx, right_idx) for every var both sides project.
    let shared: Vec<(usize, usize)> = left
        .vars
        .iter()
        .enumerate()
        .filter_map(|(li, v)| right.vars.iter().position(|rv| rv == v).map(|ri| (li, ri)))
        .collect();

    // Index the right side by its tuple of shared-var values so the join is
    // O(|left|+|right|) hash-join rather than O(|left|·|right|). `oxrdf::Term`
    // is `Eq + Hash` but NOT `Ord`, so we key on a deterministic string render
    // of the shared-column terms (the same canonical render used to sort the
    // output). None = unbound, which SPARQL treats as compatible with anything —
    // handled below by also scanning right rows with an unbound shared slot and
    // by the explicit compatibility check. For the disclosed-key regime the
    // global-IRI key is always bound, so the common path is a clean hash hit.
    let mut right_by_key: BTreeMap<String, Vec<usize>> = BTreeMap::new();
    for (ri, row) in right.rows.iter().enumerate() {
        let key = render_key(row, &shared.iter().map(|&(_, rj)| rj).collect::<Vec<_>>());
        right_by_key.entry(key).or_default().push(ri);
    }
    // Rows of the right side whose shared key has at least one unbound slot must
    // be considered against every left row (unbound is compatible with all), so
    // collect them once.
    let right_has_unbound_key: Vec<usize> = right
        .rows
        .iter()
        .enumerate()
        .filter(|(_, row)| shared.iter().any(|&(_, rj)| row[rj].is_none()))
        .map(|(ri, _)| ri)
        .collect();

    let mut out_rows: Vec<Vec<Option<Term>>> = Vec::new();
    for lrow in &left.rows {
        let left_shared_idx: Vec<usize> = shared.iter().map(|&(lj, _)| lj).collect();
        let left_has_unbound = left_shared_idx.iter().any(|&lj| lrow[lj].is_none());
        let lkey = render_key(lrow, &left_shared_idx);

        // Candidate right rows: exact-key matches, plus any right row with an
        // unbound shared slot, plus (if the LEFT key has an unbound slot) every
        // right row. The explicit compatibility check below is authoritative;
        // these are just the candidate set.
        let mut candidates: Vec<usize> = right_by_key.get(&lkey).cloned().unwrap_or_default();
        candidates.extend(right_has_unbound_key.iter().copied());
        if left_has_unbound {
            candidates = (0..right.rows.len()).collect();
        }
        candidates.sort_unstable();
        candidates.dedup();

        for ri in candidates {
            let rrow = &right.rows[ri];
            if !compatible(lrow, rrow, &shared) {
                continue;
            }
            // Merge: left row, then the right columns not already in the schema.
            let mut merged = lrow.clone();
            for (rj, rv) in right.vars.iter().enumerate() {
                if !left.vars.contains(rv) {
                    merged.push(rrow[rj].clone());
                } else if let Some(slot) = out_vars.iter().position(|v| v == rv) {
                    // Shared var: if the left side had it unbound but the right
                    // side bound it, take the bound value (SPARQL merge).
                    if merged[slot].is_none() {
                        merged[slot] = rrow[rj].clone();
                    }
                }
            }
            out_rows.push(merged);
        }
    }

    Ok(PartialResult {
        // Provisional holder; the top-level join() overwrites it with the
        // synthetic federation id once the whole fold completes.
        holder: left.holder.clone(),
        vars: out_vars,
        rows: out_rows,
    })
}

/// Deterministic string render of a row's terms at the given column indices,
/// used as a hash-join key (oxrdf `Term` is `Eq + Hash` but not `Ord`, so we
/// cannot key a `BTreeMap` on `Vec<Option<Term>>` directly). `{:?}` is stable
/// within a run and distinguishes terms that differ in lexical form, datatype,
/// or language tag — adequate for an equi-join key. The authoritative match is
/// still term `==` via [`compatible`]; this only buckets candidates.
fn render_key(row: &[Option<Term>], idxs: &[usize]) -> String {
    let mut s = String::new();
    for &i in idxs {
        match &row[i] {
            Some(t) => {
                s.push('\u{1}'); // bound marker
                s.push_str(&format!("{t:?}"));
            }
            None => s.push('\u{0}'), // unbound marker (distinct from any term)
        }
        s.push('\u{1f}'); // column separator
    }
    s
}

/// SPARQL compatibility: two rows are compatible iff for every shared variable
/// they BOTH bind, the bound terms are equal. An unbound slot on either side is
/// compatible with anything.
fn compatible(lrow: &[Option<Term>], rrow: &[Option<Term>], shared: &[(usize, usize)]) -> bool {
    shared.iter().all(|&(li, ri)| match (&lrow[li], &rrow[ri]) {
        (Some(a), Some(b)) => a == b,
        _ => true, // one side unbound → compatible
    })
}

/// Sort rows into a canonical order (by their `{:?}` term tuple) so the
/// disclosed result multiset is independent of holder/row ordering — the
/// verifier recomputes ORDER BY over the disclosed multiset (convention #4).
fn canonicalize_rows(rows: &mut [Vec<Option<Term>>]) {
    rows.sort_by(|a, b| format!("{a:?}").cmp(&format!("{b:?}")));
}

#[cfg(test)]
mod tests {
    //! M2 tests: the disclosed-key global-IRI equi-join. The load-bearing one is
    //! `differential_*`: the federated join of per-holder partials must equal
    //! evaluating the WHOLE query over the UNION of the holders' graphs in a
    //! single `sparq-engine` store. This is what makes the crypto-free join a
    //! faithful stand-in for PAG evaluation over `D = ⋃ holder graphs`.
    use super::*;
    use crate::holder::Holder;
    use sparq_core::Graph;
    use sparq_engine::query;

    const PFX: &str = "@prefix ex: <http://ex/> .\n";

    fn var(n: &str) -> Variable {
        Variable::new_unchecked(n)
    }

    /// Render a (vars, rows) result as a canonical, order-independent multiset of
    /// rows where each row is a sorted list of `(?var, term-debug)` pairs. Two
    /// results are equal as SPARQL solution multisets iff this is equal. Used to
    /// compare the federated join against the union-store evaluation regardless
    /// of column order or row order.
    fn canonical_multiset(vars: &[Variable], rows: &[Vec<Option<Term>>]) -> Vec<Vec<(String, String)>> {
        let mut out: Vec<Vec<(String, String)>> = rows
            .iter()
            .map(|row| {
                let mut pairs: Vec<(String, String)> = vars
                    .iter()
                    .zip(row.iter())
                    .map(|(v, t)| {
                        let val = match t {
                            Some(t) => format!("{t:?}"),
                            None => "<UNBOUND>".to_string(),
                        };
                        (v.as_str().to_string(), val)
                    })
                    .collect();
                pairs.sort();
                pairs
            })
            .collect();
        out.sort();
        out
    }

    /// Build a single union store from several turtle documents (the "evaluate
    /// the whole query over D = ⋃ holder graphs" side of the differential).
    fn union_graph(docs: &[&str]) -> Graph {
        let mut combined = String::from(PFX);
        for d in docs {
            combined.push_str(d);
            combined.push('\n');
        }
        Graph::load_str(&combined, "turtle").expect("union graph parses")
    }

    /// THE differential test (two holders): Holder A has `?p ex:knows ?x`,
    /// Holder B has `?x ex:name ?n`. Each evaluates its OWN fragment locally;
    /// the disclosed-key join on the shared global IRI `?x` must equal the whole
    /// query `{ ?p ex:knows ?x . ?x ex:name ?n }` over the UNION store.
    #[test]
    fn differential_two_holder_join_equals_union_eval() {
        // Holder A: a "knows" graph. ?x values are GLOBAL IRIs shared with B.
        let a_doc = "ex:p1 ex:knows ex:x1 . ex:p1 ex:knows ex:x2 . ex:p2 ex:knows ex:x2 .";
        // Holder B: a "name" graph. Note ex:x3 has a name but nobody knows it,
        // and ex:x1/ex:x2 are the shared join keys.
        let b_doc = "ex:x1 ex:name \"Xena\" . ex:x2 ex:name \"Yuri\" . ex:x3 ex:name \"Zed\" .";

        let a = Holder::from_rdf("a", &format!("{PFX}{a_doc}"), "turtle").unwrap();
        let b = Holder::from_rdf("b", &format!("{PFX}{b_doc}"), "turtle").unwrap();

        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let pb = b
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?x ?n WHERE { ?x ex:name ?n }")
            .unwrap();

        let plan = JoinPlan { join_var: var("x"), key_disclosed: true };
        let joined = DisclosedKeyJoin.join(&[pa, pb], &plan).expect("disclosed-key join ok");

        // Federated result is attributed to the synthetic federation holder, not
        // to any single source.
        assert_eq!(joined.holder, HolderId::new("federation"));

        // Union-store evaluation of the whole query.
        let u = union_graph(&[a_doc, b_doc]);
        let expected = query(
            &u,
            "PREFIX ex: <http://ex/> SELECT ?p ?x ?n WHERE { ?p ex:knows ?x . ?x ex:name ?n }",
        )
        .unwrap();

        // DIFFERENTIAL: identical solution multisets.
        assert_eq!(
            canonical_multiset(&joined.vars, &joined.rows),
            canonical_multiset(&expected.vars, &expected.rows),
            "federated join must equal union-store evaluation"
        );
        // And concretely: p1-x1-Xena, p1-x2-Yuri, p2-x2-Yuri (x3 dropped: nobody
        // knows it; so 3 rows).
        assert_eq!(joined.rows.len(), 3);
    }

    /// Empty-result holder: if one holder discloses NO rows for its fragment, the
    /// inner join is empty — and that equals the union-store evaluation (an inner
    /// BGP join with an unsatisfied pattern yields nothing).
    #[test]
    fn empty_holder_yields_empty_join_matching_union() {
        let a_doc = "ex:p1 ex:knows ex:x1 .";
        let b_doc = "ex:p1 ex:age 30 ."; // no ex:name triples at all

        let a = Holder::from_rdf("a", &format!("{PFX}{a_doc}"), "turtle").unwrap();
        let b = Holder::from_rdf("b", &format!("{PFX}{b_doc}"), "turtle").unwrap();

        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let pb = b
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?x ?n WHERE { ?x ex:name ?n }")
            .unwrap();
        assert!(pb.is_empty(), "Holder B discloses no rows for its fragment");

        let plan = JoinPlan { join_var: var("x"), key_disclosed: true };
        let joined = DisclosedKeyJoin.join(&[pa, pb], &plan).unwrap();
        assert!(joined.is_empty(), "join with an empty partial is empty");

        let u = union_graph(&[a_doc, b_doc]);
        let expected = query(
            &u,
            "PREFIX ex: <http://ex/> SELECT ?p ?x ?n WHERE { ?p ex:knows ?x . ?x ex:name ?n }",
        )
        .unwrap();
        assert!(expected.is_empty());
        assert_eq!(
            canonical_multiset(&joined.vars, &joined.rows),
            canonical_multiset(&expected.vars, &expected.rows),
        );
    }

    /// Multi-row fan-out: one join key shared by many rows on both sides exercises
    /// the cartesian product within a key bucket; still must equal the union eval.
    #[test]
    fn multi_row_fanout_join_equals_union() {
        // Two people both know x1; x1 has two names (multi-valued). Expect 2×2=4
        // joined rows on the x1 key.
        let a_doc = "ex:p1 ex:knows ex:x1 . ex:p2 ex:knows ex:x1 .";
        let b_doc = "ex:x1 ex:name \"A\" . ex:x1 ex:name \"B\" .";

        let a = Holder::from_rdf("a", &format!("{PFX}{a_doc}"), "turtle").unwrap();
        let b = Holder::from_rdf("b", &format!("{PFX}{b_doc}"), "turtle").unwrap();
        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let pb = b
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?x ?n WHERE { ?x ex:name ?n }")
            .unwrap();

        let plan = JoinPlan { join_var: var("x"), key_disclosed: true };
        let joined = DisclosedKeyJoin.join(&[pa, pb], &plan).unwrap();
        assert_eq!(joined.rows.len(), 4, "2 knowers × 2 names on the x1 key");

        let u = union_graph(&[a_doc, b_doc]);
        let expected = query(
            &u,
            "PREFIX ex: <http://ex/> SELECT ?p ?x ?n WHERE { ?p ex:knows ?x . ?x ex:name ?n }",
        )
        .unwrap();
        assert_eq!(
            canonical_multiset(&joined.vars, &joined.rows),
            canonical_multiset(&expected.vars, &expected.rows),
        );
    }

    /// A 3-holder chain: A `?p knows ?x`, B `?x supervisedBy ?y`, C `?y name ?n`.
    /// Folded join on the chain must equal the 3-pattern BGP over the union of
    /// all three holders' graphs. (Each fold step joins on the shared var.)
    #[test]
    fn differential_three_holder_chain_equals_union() {
        let a_doc = "ex:p1 ex:knows ex:x1 . ex:p2 ex:knows ex:x2 .";
        let b_doc = "ex:x1 ex:supervisedBy ex:y1 . ex:x2 ex:supervisedBy ex:y1 . ex:x9 ex:supervisedBy ex:y2 .";
        let c_doc = "ex:y1 ex:name \"Boss\" . ex:y2 ex:name \"Other\" .";

        let a = Holder::from_rdf("a", &format!("{PFX}{a_doc}"), "turtle").unwrap();
        let b = Holder::from_rdf("b", &format!("{PFX}{b_doc}"), "turtle").unwrap();
        let c = Holder::from_rdf("c", &format!("{PFX}{c_doc}"), "turtle").unwrap();

        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let pb = b
            .evaluate_local(
                "PREFIX ex: <http://ex/> SELECT ?x ?y WHERE { ?x ex:supervisedBy ?y }",
            )
            .unwrap();
        let pc = c
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?y ?n WHERE { ?y ex:name ?n }")
            .unwrap();

        // The fold: A⋈B on ?x, then (A⋈B)⋈C on ?y. The JoinPlan names the key the
        // planner expects to be present in *all* partials; here we fold pairwise,
        // each step naming its shared var. (A single JoinPlan models one join;
        // the chain is two joins.)
        let ab = DisclosedKeyJoin
            .join(&[pa, pb], &JoinPlan { join_var: var("x"), key_disclosed: true })
            .unwrap();
        let abc = DisclosedKeyJoin
            .join(&[ab, pc], &JoinPlan { join_var: var("y"), key_disclosed: true })
            .unwrap();

        let u = union_graph(&[a_doc, b_doc, c_doc]);
        let expected = query(
            &u,
            "PREFIX ex: <http://ex/> SELECT ?p ?x ?y ?n WHERE { \
                 ?p ex:knows ?x . ?x ex:supervisedBy ?y . ?y ex:name ?n }",
        )
        .unwrap();
        assert_eq!(
            canonical_multiset(&abc.vars, &abc.rows),
            canonical_multiset(&expected.vars, &expected.rows),
        );
        // p1→x1→y1→Boss and p2→x2→y1→Boss; x9/y2 dropped (no knower). 2 rows.
        assert_eq!(abc.rows.len(), 2);
    }

    /// SOUNDNESS: the join must NOT trust the untrusted planner. If the planner
    /// names a join key a holder did not actually disclose, the join FAILS with a
    /// Protocol error — it does not silently produce an empty (or wrong) result a
    /// later proof layer could not back (§4.1).
    #[test]
    fn join_key_absent_from_a_partial_is_a_protocol_error() {
        let a = Holder::from_rdf("a", &format!("{PFX}ex:p1 ex:knows ex:x1 ."), "turtle").unwrap();
        let b = Holder::from_rdf("b", &format!("{PFX}ex:x1 ex:name \"X\" ."), "turtle").unwrap();
        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let pb = b
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?x ?n WHERE { ?x ex:name ?n }")
            .unwrap();
        // Planner names ?wrong — present in NEITHER partial.
        let plan = JoinPlan { join_var: var("wrong"), key_disclosed: true };
        let err = DisclosedKeyJoin.join(&[pa, pb], &plan).unwrap_err();
        match err {
            MpcError::Protocol(m) => assert!(m.contains("did not disclose the join key")),
            other => panic!("expected Protocol error, got {other:?}"),
        }
    }

    /// An empty federation is a Protocol precondition violation, not a panic.
    #[test]
    fn empty_federation_is_a_protocol_error() {
        let plan = JoinPlan { join_var: var("x"), key_disclosed: true };
        let err = DisclosedKeyJoin.join(&[], &plan).unwrap_err();
        assert!(matches!(err, MpcError::Protocol(_)));
    }

    /// A single-holder "join" is the identity (its own disclosed partial,
    /// re-attributed to the federation) — and equals evaluating that fragment
    /// over the (single-holder) union. Edge case: the fold over one partial.
    #[test]
    fn single_holder_join_is_identity() {
        let a_doc = "ex:p1 ex:knows ex:x1 . ex:p2 ex:knows ex:x2 .";
        let a = Holder::from_rdf("a", &format!("{PFX}{a_doc}"), "turtle").unwrap();
        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        let plan = JoinPlan { join_var: var("x"), key_disclosed: true };
        let joined = DisclosedKeyJoin.join(std::slice::from_ref(&pa), &plan).unwrap();
        assert_eq!(joined.rows.len(), pa.rows.len());
        assert_eq!(
            canonical_multiset(&joined.vars, &joined.rows),
            canonical_multiset(&pa.vars, &pa.rows),
        );
    }

    /// HONEST DEFERRAL: the hidden-value (private-key) join is NOT faked. It
    /// returns a gate-naming `NotYetImplemented` citing M3 + Q2/Q3 — never a
    /// silent or fake result. This is the M2 boundary made explicit.
    #[test]
    fn hidden_value_join_is_honestly_deferred() {
        let a = Holder::from_rdf("a", &format!("{PFX}ex:p1 ex:knows ex:x1 ."), "turtle").unwrap();
        let pa = a
            .evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")
            .unwrap();
        // key_disclosed == false → the private-value regime.
        let plan = JoinPlan { join_var: var("x"), key_disclosed: false };
        let err = DisclosedKeyJoin.join(&[pa], &plan).unwrap_err();
        match err {
            MpcError::NotYetImplemented { gated_on, what } => {
                assert!(what.contains("hidden-value") || what.contains("PSI"));
                assert!(gated_on.contains("M3"), "must cite the backend gate M3");
                assert!(gated_on.contains("Q2") && gated_on.contains("Q3"));
            }
            other => panic!("expected NotYetImplemented, got {other:?}"),
        }
    }
}
