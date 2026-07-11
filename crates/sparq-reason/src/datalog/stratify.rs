//! [FABLE-5] sq-6tykl.3 — the **stratification checker**: assigns every rule a
//! stratum such that each `NOT`-negated or `AGGREGATE`-read predicate is fully
//! derived in a strictly lower stratum, or rejects the program (a cycle through
//! negation/aggregation has no stratified model).
//!
//! Algorithm: the classic iterative stratum-raising fixpoint over the predicate
//! dependency graph (Ullman, *Principles of Database and Knowledge-Base Systems*,
//! §3.8 — equivalent to SCC condensation with a no-negative-edge-inside-an-SCC
//! check, but yields the tight stratum numbering directly). For every rule with
//! head predicate `h`:
//!
//! * `stratum(h) ≥ stratum(p)`     for each positive body atom predicate `p`;
//! * `stratum(h) ≥ stratum(q) + 1` for each `NOT` atom predicate `q` and each
//!   predicate inside an `AGGREGATE` body (aggregation is non-monotonic: adding a
//!   fact can change a count, so the aggregated relation must be complete);
//! * `stratum(h₁) = stratum(h₂)`   for co-head predicates of one rule (mutual
//!   positive edges), so a rule's stratum is well-defined.
//!
//! **Granularity:** dependency nodes are predicates, except `rdf:type` atoms with a
//! CONSTANT class, which get a per-CLASS node — RDF encodes unary relations as
//! classes, so `NOT [?x, a, ex:Hub]` must not collide with an unrelated
//! `[?y, a, ex:Leaf]` head. An `rdf:type` atom with a VARIABLE class is conservative:
//! reading one depends on EVERY class; deriving one feeds every class reader.
//!
//! A stratum exceeding the node count proves a positive/negative cycle with
//! at least one negative edge — reported with the offending predicate/class IRI.

use super::{Program, Rule};
use rustc_hash::FxHashMap;
use sparq_core::dict::{Dict, Id};

/// A valid stratification of a [`Program`]: the per-rule stratum assignment
/// computed by [`stratify`]. Consumed by the evaluator (strata run in order).
#[derive(Clone, Debug)]
pub struct Stratification {
    /// Stratum of each rule (index-parallel with the program's rules).
    pub(crate) rule_stratum: Vec<usize>,
    /// Number of strata (`max(rule_stratum) + 1`; `0` for an empty program).
    pub(crate) n_strata: usize,
}

impl Stratification {
    /// Number of strata the program evaluates in (a pure-positive program has 1).
    ///
    /// # Examples
    ///
    /// ```
    /// use sparq_reason::datalog::{parse_program, stratify};
    /// let mut dict = sparq_core::dict::Dict::new();
    /// let p = parse_program(
    ///     &mut dict,
    ///     "@prefix ex: <http://ex/> .
    ///      [?x, ex:lonely, \"y\"] :- [?x, a, ex:Node], NOT [?x, ex:knows, ?y] .",
    /// )?;
    /// // `ex:knows` is EDB-only (stratum 0); the NOT edge pushes `ex:lonely` to 1.
    /// assert_eq!(stratify(&dict, &p)?.n_strata(), 2);
    /// # Ok::<(), String>(())
    /// ```
    pub fn n_strata(&self) -> usize {
        self.n_strata
    }
}

const RDF_TYPE: &str = "http://www.w3.org/1999/02/22-rdf-syntax-ns#type";

/// A dependency-graph node: a predicate, one CLASS of the `rdf:type` relation, or
/// the conservative "any class" node for variable-class `rdf:type` atoms.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
enum Key {
    Pred(Id),
    Class(Id),
    TypeAny,
}

/// The dependency node an atom reads from / derives into.
fn atom_key(atom: &super::Atom, ty: Option<Id>) -> Key {
    if ty == Some(atom.pred) {
        match atom.t[2] {
            super::DTerm::Const(c) => Key::Class(c),
            super::DTerm::Var(_) => Key::TypeAny,
        }
    } else {
        Key::Pred(atom.pred)
    }
}

/// Per-rule dependency edges: `(body node, negative?)`.
fn rule_edges(rule: &Rule, ty: Option<Id>) -> Vec<(Key, bool)> {
    let mut edges = Vec::new();
    for a in &rule.positive {
        edges.push((atom_key(a, ty), false));
    }
    for a in &rule.negated {
        edges.push((atom_key(a, ty), true));
    }
    for agg in &rule.aggregates {
        for a in &agg.body {
            edges.push((atom_key(a, ty), true));
        }
    }
    edges
}

/// Check that `program` is stratifiable and compute the per-rule strata.
///
/// `dict` is read-only here — it renders predicate IRIs in the rejection message.
///
/// # Errors
///
/// Returns `Err` when the program's dependency graph has a cycle through a `NOT`
/// or `AGGREGATE` edge (no stratified model exists), naming a predicate on the
/// cycle.
///
/// # Examples
///
/// ```
/// use sparq_reason::datalog::{parse_program, stratify};
/// let mut dict = sparq_core::dict::Dict::new();
/// // win(X) :- move(X,Y), NOT win(Y) — the textbook NON-stratifiable program.
/// let p = parse_program(
///     &mut dict,
///     "@prefix ex: <http://ex/> .
///      [?x, ex:win, \"y\"] :- [?x, ex:move, ?y], NOT [?y, ex:win, \"y\"] .",
/// )?;
/// assert!(stratify(&dict, &p).is_err());
/// # Ok::<(), String>(())
/// ```
pub fn stratify(dict: &Dict, program: &Program) -> Result<Stratification, String> {
    // `rdf:type` is only special if the document mentions it (lookup, no interning —
    // `dict` is read-only here; an absent id can match no atom predicate).
    let ty = {
        let id = dict.lookup(&oxrdf::NamedNode::new_unchecked(RDF_TYPE).into());
        (id != 0).then_some(id)
    };
    // Collect every dependency node; strata live on nodes.
    let mut node_ix: FxHashMap<Key, usize> = FxHashMap::default();
    let mut nodes: Vec<Key> = Vec::new();
    let mut ix = |k: Key, node_ix: &mut FxHashMap<Key, usize>, nodes: &mut Vec<Key>| -> usize {
        *node_ix.entry(k).or_insert_with(|| {
            nodes.push(k);
            nodes.len() - 1
        })
    };
    // (head, body, negative) edges over node indices.
    let mut edges: Vec<(usize, usize, bool)> = Vec::new();
    for rule in &program.rules {
        let heads: Vec<usize> = rule
            .head
            .iter()
            .map(|a| ix(atom_key(a, ty), &mut node_ix, &mut nodes))
            .collect();
        for (b, neg) in rule_edges(rule, ty) {
            let b = ix(b, &mut node_ix, &mut nodes);
            for &h in &heads {
                edges.push((h, b, neg));
            }
        }
        // Co-head nodes must share a stratum: mutual positive edges.
        for &h1 in &heads {
            for &h2 in &heads {
                if h1 != h2 {
                    edges.push((h1, h2, false));
                }
            }
        }
    }
    // The conservative variable-class coupling: reading `TypeAny` depends on every
    // class; a `TypeAny` HEAD feeds every class reader.
    if let Some(&any) = node_ix.get(&Key::TypeAny) {
        let classes: Vec<usize> = nodes
            .iter()
            .enumerate()
            .filter(|(_, k)| matches!(k, Key::Class(_)))
            .map(|(i, _)| i)
            .collect();
        let any_is_head = program
            .rules
            .iter()
            .flat_map(|r| &r.head)
            .any(|a| atom_key(a, ty) == Key::TypeAny);
        for c in classes {
            edges.push((any, c, false)); // TypeAny reads every class
            if any_is_head {
                edges.push((c, any, false)); // every class may receive TypeAny facts
            }
        }
    }
    let n = nodes.len();
    let mut stratum = vec![0usize; n];
    loop {
        let mut changed = false;
        for &(h, b, neg) in &edges {
            let req = stratum[b] + usize::from(neg);
            if stratum[h] < req {
                if req > n {
                    return Err(format!(
                        "datalog program is NOT stratifiable: {} lies on a \
                         recursion cycle through NOT/AGGREGATE (no stratified model exists)",
                        node_name(dict, nodes[h])
                    ));
                }
                stratum[h] = req;
                changed = true;
            }
        }
        if !changed {
            break;
        }
    }
    // A rule's stratum is its head node's (co-heads converge to one value).
    let rule_stratum: Vec<usize> = program
        .rules
        .iter()
        .map(|r| stratum[node_ix[&atom_key(&r.head[0], ty)]])
        .collect();
    let n_strata = rule_stratum.iter().max().map_or(0, |m| m + 1);
    Ok(Stratification {
        rule_stratum,
        n_strata,
    })
}

fn node_name(dict: &Dict, k: Key) -> String {
    let iri = |id: Id| match dict.term(id) {
        oxrdf::Term::NamedNode(n) => n.as_str().to_string(),
        other => other.to_string(),
    };
    match k {
        Key::Pred(p) => format!("predicate `{}`", iri(p)),
        Key::Class(c) => format!("class `{}` (rdf:type atoms)", iri(c)),
        Key::TypeAny => "the variable-class `rdf:type` relation".to_string(),
    }
}
