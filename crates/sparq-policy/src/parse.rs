//! Parse an ODRL policy expressed in RDF into the typed [`Policy`] model.
//!
//! The parser runs SPARQL queries (via [`sparq_engine::query`]) over the loaded
//! policy graph — ODRL evaluation *is* a SPARQL/SHACL/N3 workload, so extracting
//! the model with the engine sparq already ships keeps this crate in-family and
//! dependency-light (no bespoke RDF walker).
//!
//! Rule and constraint nodes are matched by **variable** (never by a literal
//! blank-node label), because real ODRL policies overwhelmingly express rules,
//! constraints and duties as *blank nodes*; the whole rule structure is joined
//! in one query per (rule-kind, attribute) so a blank-node constraint is bound
//! through its incident edge, not re-named.
//!
//! Input forms accepted by [`parse_policy_str`]: any RDF serialization
//! `sparq_core::Graph::load_str` accepts (`turtle`, `ntriples`, …). The policy
//! IRI is whichever subject is `a odrl:Policy`/`Set`/`Offer`/`Agreement`, or —
//! if none is typed — whichever subject carries `odrl:permission`/`prohibition`.
//! [OPUS-4.8]

use crate::model::{
    Action, ConflictStrategy, Constraint, ConstraintNode, Duty, LogicalConstraint, LogicalOperator,
    Operator, Policy, Rule, Value, ODRL_NS,
};
use oxrdf::{Literal, Term};
use sparq_core::Graph;
use std::collections::BTreeMap;

const XSD: &str = "http://www.w3.org/2001/XMLSchema#";

/// Parse an ODRL policy from an RDF string in `format` (e.g. `"turtle"`).
///
/// # Errors
///
/// Returns `Err` if the RDF does not parse, or if a query over it fails. A
/// well-formed RDF document with no ODRL rules parses to an empty [`Policy`]
/// (which then denies everything — fail-closed).
pub fn parse_policy_str(rdf: &str, format: &str) -> Result<Policy, String> {
    let graph = Graph::load_str(rdf, format)?;
    parse_policy(&graph)
}

/// Parse an ODRL policy from an already-loaded [`Graph`].
///
/// # Errors
///
/// Returns `Err` if a query over the graph fails.
pub fn parse_policy(graph: &Graph) -> Result<Policy, String> {
    let iri = policy_iri(graph)?;
    let permissions = rules(graph, "permission", true)?;
    let prohibitions = rules(graph, "prohibition", false)?;
    let conflict = policy_conflict(graph)?;
    Ok(Policy {
        iri,
        permissions,
        prohibitions,
        conflict,
    })
}

/// Extract the policy's declared `odrl:conflict` conflict-resolution strategy, if any.
/// [OPUS-4.8] sq-ihqbl.
///
/// The value is classified via [`ConflictStrategy::from_iri`], so an unrecognised term
/// is preserved as [`ConflictStrategy::Unknown`] (and later *refused*) rather than
/// silently dropped.
///
/// **Fail-closed on ambiguity.** For an authorization guard it is unsound to take only
/// the first-sorted `?c`: a benign strategy that happens to sort first (e.g.
/// `odrl:prohibit`) would mask a co-asserted unimplementable one (`odrl:perm`, an unknown
/// IRI), and the graph would be mis-classified as admissible. So we gather **every**
/// distinct declared strategy and *refuse* (`Err`) when more than one is present — a graph
/// declaring multiple conflicting resolution strategies is ambiguous and cannot be honoured
/// deterministically. (Any multi-value set necessarily contains a non-`Prohibit` strategy,
/// so this only ever refuses a graph that declares a non-default strategy, never a benign
/// deny-overrides one.) We deliberately do **not** tie `?p` to a specific policy node: an
/// unrelated subject asserting `odrl:conflict` then contributes to the refusal set, which is
/// strictly the *more* fail-closed direction. [OPUS-4.8] sq-ihqbl.
fn policy_conflict(graph: &Graph) -> Result<Option<ConflictStrategy>, String> {
    let res = sparq_engine::query(
        graph,
        &format!("SELECT DISTINCT ?c WHERE {{ ?p <{ODRL_NS}conflict> ?c }} ORDER BY ?c"),
    )?;
    let mut strategies: Vec<ConflictStrategy> = res
        .rows
        .into_iter()
        .filter_map(|r| r.into_iter().next().flatten())
        .map(|t| ConflictStrategy::from_iri(&term_str(&t)))
        .collect();
    strategies.dedup();
    match strategies.len() {
        0 => Ok(None),
        1 => Ok(Some(strategies.remove(0))),
        _ => Err(format!(
            "policy declares multiple conflicting `odrl:conflict` strategies ({strategies:?}); \
             an ambiguous conflict-resolution set is refused (fail-closed) rather than resolved \
             to whichever term sorts first"
        )),
    }
}

/// A stable string key identifying a rule/constraint/duty node, used to group
/// the flat rows of a join query back into structured rules. IRIs and blank
/// nodes both get a distinct key.
fn node_key(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => format!("<{}>", n.as_str()),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Literal(l) => format!("\"{}\"", l.value()),
        #[allow(unreachable_patterns)]
        _ => String::new(),
    }
}

fn term_str(t: &Term) -> String {
    match t {
        Term::NamedNode(n) => n.as_str().to_owned(),
        Term::BlankNode(b) => format!("_:{}", b.as_str()),
        Term::Literal(l) => l.value().to_owned(),
        #[allow(unreachable_patterns)]
        _ => String::new(),
    }
}

fn policy_iri(graph: &Graph) -> Result<Option<String>, String> {
    let typed = sparq_engine::query(
        graph,
        &format!(
            "SELECT ?p WHERE {{ ?p a ?t . \
             VALUES ?t {{ <{ODRL_NS}Policy> <{ODRL_NS}Set> <{ODRL_NS}Offer> <{ODRL_NS}Agreement> }} }}"
        ),
    )?;
    if let Some(Term::NamedNode(n)) = typed
        .rows
        .into_iter()
        .next()
        .and_then(|r| r.into_iter().next().flatten())
    {
        return Ok(Some(n.into_string()));
    }
    let withrule = sparq_engine::query(
        graph,
        &format!(
            "SELECT ?p WHERE {{ ?p ?r ?x . \
             VALUES ?r {{ <{ODRL_NS}permission> <{ODRL_NS}prohibition> }} }}"
        ),
    )?;
    Ok(
        match withrule
            .rows
            .into_iter()
            .next()
            .and_then(|r| r.into_iter().next().flatten())
        {
            Some(Term::NamedNode(n)) => Some(n.into_string()),
            _ => None,
        },
    )
}

/// Index `?rule -> attribute` rows from a single query into per-rule lists.
fn group_by_first(graph: &Graph, sparql: &str) -> Result<BTreeMap<String, Vec<Term>>, String> {
    let res = sparq_engine::query(graph, sparql)?;
    let mut out: BTreeMap<String, Vec<Term>> = BTreeMap::new();
    for row in res.rows {
        let mut it = row.into_iter();
        let (Some(Some(key_t)), Some(Some(val_t))) = (it.next(), it.next()) else {
            continue;
        };
        out.entry(node_key(&key_t)).or_default().push(val_t);
    }
    Ok(out)
}

/// All rules of a kind (`"permission"`/`"prohibition"`). `with_duties` controls
/// whether duty obligations are parsed (permissions only in the base case).
fn rules(graph: &Graph, kind: &str, with_duties: bool) -> Result<Vec<Rule>, String> {
    // Enumerate the rule nodes first (keyed), then attach attributes by join.
    // (BIND a copy so the projection has two distinct variables — the engine
    // rejects `SELECT ?rule ?rule`.)
    let nodes = group_by_first(
        graph,
        &format!("SELECT ?rule ?rule2 WHERE {{ ?policy <{ODRL_NS}{kind}> ?rule . BIND(?rule AS ?rule2) }}"),
    )?;

    let actions = group_by_first(
        graph,
        &format!("SELECT ?rule ?a WHERE {{ ?policy <{ODRL_NS}{kind}> ?rule . ?rule <{ODRL_NS}action> ?a }}"),
    )?;
    let targets = group_by_first(
        graph,
        &format!("SELECT ?rule ?t WHERE {{ ?policy <{ODRL_NS}{kind}> ?rule . ?rule <{ODRL_NS}target> ?t }}"),
    )?;
    let assignees = group_by_first(
        graph,
        &format!("SELECT ?rule ?p WHERE {{ ?policy <{ODRL_NS}{kind}> ?rule . ?rule <{ODRL_NS}assignee> ?p }}"),
    )?;
    let assigners = group_by_first(
        graph,
        &format!("SELECT ?rule ?p WHERE {{ ?policy <{ODRL_NS}{kind}> ?rule . ?rule <{ODRL_NS}assigner> ?p }}"),
    )?;

    let constraints = constraints_for(graph, kind, "constraint")?;
    let logical_constraints = logical_constraints_for(graph, kind)?;
    let duties = if with_duties {
        duties_for(graph, kind)?
    } else {
        BTreeMap::new()
    };

    let mut out = Vec::new();
    for (rule_key, rule_terms) in nodes {
        let node = rule_terms.first().cloned().expect("group has >=1 row");
        let action = first_str(&actions, &rule_key)
            .map(Action)
            .unwrap_or_else(Action::use_);
        out.push(Rule {
            id: term_str(&node),
            action,
            target: first_str(&targets, &rule_key),
            assignee: first_str(&assignees, &rule_key),
            assigner: first_str(&assigners, &rule_key),
            constraints: constraints.get(&rule_key).cloned().unwrap_or_default(),
            logical_constraints: logical_constraints
                .get(&rule_key)
                .cloned()
                .unwrap_or_default(),
            duties: duties.get(&rule_key).cloned().unwrap_or_default(),
        });
    }
    Ok(out)
}

fn first_str(m: &BTreeMap<String, Vec<Term>>, key: &str) -> Option<String> {
    m.get(key).and_then(|v| v.first()).map(term_str)
}

/// Parse the *atomic* constraints attached (via `odrl:<pred>`) to each rule of
/// `kind`, keyed by rule node. Constraint *nodes* are bound by variable, so
/// blank-node constraints are handled correctly.
///
/// A `?c` that is a compound `odrl:LogicalConstraint` (it carries an
/// `odrl:and`/`odrl:or`/`odrl:xone` refining set rather than a direct
/// `leftOperand`/`operator`/`rightOperand`) is **skipped here** — it is parsed by
/// [`logical_constraints_for`] into the rule's `logical_constraints` instead, so it
/// is *not* mis-read as a structurally-incomplete atomic constraint (which would
/// wrongly fail the whole rule closed). [OPUS-4.8] sq-a0zef.
fn constraints_for(
    graph: &Graph,
    kind: &str,
    pred: &str,
) -> Result<BTreeMap<String, Vec<Constraint>>, String> {
    // One row per (rule, constraint-node, left, operator, right, is-logical).
    // LEFT/OP/RIGHT are OPTIONAL so a structurally incomplete constraint still
    // surfaces (and is turned into an unsatisfiable guard — fail-closed). `?and`
    // binds iff the node is a LogicalConstraint (any of the three combinators), so
    // such a node is routed to logical_constraints_for instead of here.
    let q = format!(
        "SELECT ?rule ?c ?left ?op ?right ?and WHERE {{ \
           ?policy <{ODRL_NS}{kind}> ?rule . \
           ?rule <{ODRL_NS}{pred}> ?c . \
           OPTIONAL {{ ?c <{ODRL_NS}leftOperand> ?left }} \
           OPTIONAL {{ ?c <{ODRL_NS}operator> ?op }} \
           OPTIONAL {{ ?c <{ODRL_NS}rightOperand> ?right }} \
           OPTIONAL {{ ?c <{ODRL_NS}rightOperandReference> ?right }} \
           OPTIONAL {{ ?c ?logop ?and . \
             VALUES ?logop {{ <{ODRL_NS}and> <{ODRL_NS}or> <{ODRL_NS}xone> }} }} \
         }}"
    );
    let res = sparq_engine::query(graph, &q)?;
    let mut out: BTreeMap<String, Vec<Constraint>> = BTreeMap::new();
    // De-dup by (rule, constraint-node) so multiple OPTIONAL combos don't double-count.
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for row in res.rows {
        let mut it = row.into_iter();
        let rule_t = it.next().flatten();
        let c_t = it.next().flatten();
        let left = it.next().flatten();
        let op = it.next().flatten();
        let right = it.next().flatten();
        let is_logical = it.next().flatten().is_some();
        let (Some(rule_t), Some(c_t)) = (rule_t, c_t) else {
            continue;
        };
        let rkey = node_key(&rule_t);
        let ckey = format!("{rkey}|{}", node_key(&c_t));
        if seen.insert(ckey, ()).is_some() {
            continue;
        }
        // A compound LogicalConstraint is parsed by logical_constraints_for, not as
        // a malformed atomic constraint. [OPUS-4.8] sq-a0zef.
        if is_logical {
            continue;
        }
        out.entry(rkey)
            .or_default()
            .push(build_constraint(left, op, right));
    }
    Ok(out)
}

/// Parse the compound `odrl:LogicalConstraint` refinements attached (via
/// `odrl:constraint`) to each rule of `kind`, keyed by rule node. [OPUS-4.8] sq-a0zef.
///
/// A parsed `odrl:LogicalConstraint` node's combinator + operand node-keys, before
/// recursive assembly into a [`LogicalConstraint`]. [OPUS-4.8] sq-a0zef.
struct LcDef {
    operator: LogicalOperator,
    /// Operand node-keys in graph order (de-duplicated).
    operands: Vec<String>,
    /// De-dup set for the operand node-keys.
    seen: BTreeMap<String, ()>,
}

/// A `LogicalConstraint` node carries one combinator property
/// (`odrl:and`/`odrl:or`/`odrl:xone`) whose objects are the operand nodes (`odrl:and
/// <c1>, <c2>` — several objects of the one property). Each operand is parsed into a
/// [`ConstraintNode`]: a nested `LogicalConstraint` recurses, anything else becomes an
/// atomic [`Constraint`] via [`build_constraint`] (a structurally-incomplete atomic
/// operand becomes the unsatisfiable guard — fail-closed, never a silent pass).
///
/// Two bulk queries build node tables for the *whole* policy graph (all combinator
/// edges; all atomic constraint fields); the per-rule compound constraints are then
/// assembled recursively in-memory, with cycle protection so a malformed self-/mutually
/// referential `LogicalConstraint` cannot loop (it short-circuits to the unsatisfiable
/// guard — fail-closed).
fn logical_constraints_for(
    graph: &Graph,
    kind: &str,
) -> Result<BTreeMap<String, Vec<LogicalConstraint>>, String> {
    // (1) Every combinator edge in the graph: (lc-node, combinator, operand-node), in
    // graph order. A node appearing as a subject here is a LogicalConstraint.
    let edges_q = format!(
        "SELECT ?lc ?logop ?operand WHERE {{ \
           ?lc ?logop ?operand . \
           VALUES ?logop {{ <{ODRL_NS}and> <{ODRL_NS}or> <{ODRL_NS}xone> }} \
         }}"
    );
    let edges_res = sparq_engine::query(graph, &edges_q)?;
    let mut lc_defs: BTreeMap<String, LcDef> = BTreeMap::new();
    for row in edges_res.rows {
        let mut it = row.into_iter();
        let (Some(Some(lc_t)), Some(Some(logop_t)), Some(Some(operand_t))) =
            (it.next(), it.next(), it.next())
        else {
            continue;
        };
        let Some(operator) = LogicalOperator::from_iri(&term_str(&logop_t)) else {
            continue;
        };
        let lckey = node_key(&lc_t);
        let okey = node_key(&operand_t);
        let def = lc_defs.entry(lckey).or_insert_with(|| LcDef {
            operator,
            operands: Vec::new(),
            seen: BTreeMap::new(),
        });
        if def.seen.insert(okey.clone(), ()).is_none() {
            def.operands.push(okey);
        }
    }

    // (2) Every atomic constraint node's fields, keyed by node. A node with no
    // combinator edge but with these fields is an atomic operand.
    let atoms_q = format!(
        "SELECT ?c ?left ?op ?right WHERE {{ \
           ?c <{ODRL_NS}leftOperand> ?left . \
           OPTIONAL {{ ?c <{ODRL_NS}operator> ?op }} \
           OPTIONAL {{ ?c <{ODRL_NS}rightOperand> ?right }} \
           OPTIONAL {{ ?c <{ODRL_NS}rightOperandReference> ?right }} \
         }}"
    );
    let atoms_res = sparq_engine::query(graph, &atoms_q)?;
    let mut atoms: BTreeMap<String, Constraint> = BTreeMap::new();
    for row in atoms_res.rows {
        let mut it = row.into_iter();
        let c_t = it.next().flatten();
        let left = it.next().flatten();
        let op = it.next().flatten();
        let right = it.next().flatten();
        let Some(c_t) = c_t else { continue };
        let ckey = node_key(&c_t);
        // First binding wins (a rightOperand/rightOperandReference pair can double rows).
        atoms
            .entry(ckey)
            .or_insert_with(|| build_constraint(left, op.clone(), right.clone()));
    }

    // (3) The rule → direct-constraint-node map (which of a rule's `odrl:constraint`
    // objects are LogicalConstraint nodes), in rule/graph order.
    let rule_lc_q = format!(
        "SELECT ?rule ?c WHERE {{ \
           ?policy <{ODRL_NS}{kind}> ?rule . \
           ?rule <{ODRL_NS}constraint> ?c . \
         }}"
    );
    let rule_lc_res = sparq_engine::query(graph, &rule_lc_q)?;
    let mut out: BTreeMap<String, Vec<LogicalConstraint>> = BTreeMap::new();
    let mut seen_rule_c: BTreeMap<String, ()> = BTreeMap::new();
    for row in rule_lc_res.rows {
        let mut it = row.into_iter();
        let (Some(Some(rule_t)), Some(Some(c_t))) = (it.next(), it.next()) else {
            continue;
        };
        let rkey = node_key(&rule_t);
        let ckey = node_key(&c_t);
        if seen_rule_c.insert(format!("{rkey}|{ckey}"), ()).is_some() {
            continue;
        }
        // Only assemble compound nodes here (atomic direct constraints are handled by
        // constraints_for); a node is compound iff it has a combinator edge.
        if lc_defs.contains_key(&ckey) {
            let mut stack = BTreeMap::new();
            let lc = assemble_logical(&ckey, &lc_defs, &atoms, &mut stack);
            out.entry(rkey).or_default().push(lc);
        }
    }
    Ok(out)
}

/// Recursively assemble a [`LogicalConstraint`] from the pre-built node tables, with
/// cycle protection (`active` tracks the ancestor chain). A malformed node — not in the
/// combinator table and not a known atomic, or part of a cycle — becomes the
/// unsatisfiable-guard atomic constraint (fail-closed). [OPUS-4.8] sq-a0zef.
fn assemble_logical(
    lc_key: &str,
    lc_defs: &BTreeMap<String, LcDef>,
    atoms: &BTreeMap<String, Constraint>,
    active: &mut BTreeMap<String, ()>,
) -> LogicalConstraint {
    let def = lc_defs
        .get(lc_key)
        .expect("assemble_logical called on a non-LC node");
    active.insert(lc_key.to_owned(), ());
    let mut operands = Vec::with_capacity(def.operands.len());
    for okey in &def.operands {
        let node = if lc_defs.contains_key(okey) {
            if active.contains_key(okey) {
                // A cycle in the LogicalConstraint graph — fail closed rather than loop.
                ConstraintNode::Atomic(unsatisfiable_constraint())
            } else {
                ConstraintNode::Compound(assemble_logical(okey, lc_defs, atoms, active))
            }
        } else if let Some(c) = atoms.get(okey) {
            ConstraintNode::Atomic(c.clone())
        } else {
            // An operand that is neither a recognised compound nor a well-formed atomic
            // constraint → unsatisfiable guard (fail-closed).
            ConstraintNode::Atomic(unsatisfiable_constraint())
        };
        operands.push(node);
    }
    active.remove(lc_key);
    LogicalConstraint {
        id: lc_key.to_owned(),
        operator: def.operator,
        operands,
    }
}

/// The unsatisfiable-guard atomic constraint used for a malformed/unknown operand
/// (a constraint that can never be satisfied — fail-closed). Shared by [`build_constraint`]
/// and the compound-operand assembler. [OPUS-4.8] sq-a0zef.
fn unsatisfiable_constraint() -> Constraint {
    Constraint {
        left: "urn:sparq-policy:malformed".to_owned(),
        operator: Operator::Neq,
        right: Value::Iri("urn:sparq-policy:malformed".to_owned()),
    }
}

/// Build a [`Constraint`], turning anything malformed/unknown into an
/// unsatisfiable guard (fail-closed): the enclosing rule can then never match.
fn build_constraint(left: Option<Term>, op: Option<Term>, right: Option<Term>) -> Constraint {
    let (Some(left), Some(op), Some(right)) = (left, op, right) else {
        return unsatisfiable_constraint();
    };
    let operator = match Operator::from_iri(&term_str(&op)) {
        Some(o) => o,
        None => return unsatisfiable_constraint(),
    };
    Constraint {
        left: term_str(&left),
        operator,
        right: value_of(&right),
    }
}

/// Parse duties (and their actions/constraints) per rule node.
fn duties_for(graph: &Graph, kind: &str) -> Result<BTreeMap<String, Vec<Duty>>, String> {
    let q = format!(
        "SELECT ?rule ?d ?a WHERE {{ \
           ?policy <{ODRL_NS}{kind}> ?rule . \
           ?rule <{ODRL_NS}duty> ?d . \
           OPTIONAL {{ ?d <{ODRL_NS}action> ?a }} \
         }}"
    );
    let res = sparq_engine::query(graph, &q)?;
    let mut out: BTreeMap<String, Vec<Duty>> = BTreeMap::new();
    let mut seen: BTreeMap<String, ()> = BTreeMap::new();
    for row in res.rows {
        let mut it = row.into_iter();
        let rule_t = it.next().flatten();
        let d_t = it.next().flatten();
        let a_t = it.next().flatten();
        let (Some(rule_t), Some(d_t)) = (rule_t, d_t) else {
            continue;
        };
        let rkey = node_key(&rule_t);
        let dkey = format!("{rkey}|{}", node_key(&d_t));
        if seen.insert(dkey, ()).is_some() {
            continue;
        }
        let action = a_t
            .map(|t| Action(term_str(&t)))
            .unwrap_or_else(Action::use_);
        out.entry(rkey).or_default().push(Duty {
            id: term_str(&d_t),
            action,
            constraints: Vec::new(),
        });
    }
    Ok(out)
}

/// Classify a [`Term`] into a typed [`Value`] (numeric/dateTime/iri/string).
pub(crate) fn value_of(t: &Term) -> Value {
    match t {
        Term::NamedNode(n) => Value::Iri(n.as_str().to_owned()),
        Term::BlankNode(b) => Value::Str(format!("_:{}", b.as_str())),
        Term::Literal(l) => literal_value(l),
        #[allow(unreachable_patterns)]
        _ => Value::Str(String::new()),
    }
}

fn literal_value(l: &Literal) -> Value {
    let dt = l.datatype();
    let v = l.value();
    if is_datetime(dt) {
        Value::DateTime(v.to_owned())
    } else if is_numeric(dt) {
        match v.trim().parse::<f64>() {
            Ok(n) => Value::Num(n),
            Err(_) => Value::Str(v.to_owned()),
        }
    } else {
        Value::Str(v.to_owned())
    }
}

fn is_datetime(dt: oxrdf::NamedNodeRef<'_>) -> bool {
    matches!(
        dt.as_str().strip_prefix(XSD),
        Some("dateTime" | "date" | "dateTimeStamp")
    )
}

fn is_numeric(dt: oxrdf::NamedNodeRef<'_>) -> bool {
    matches!(
        dt.as_str().strip_prefix(XSD),
        Some(
            "integer"
                | "decimal"
                | "double"
                | "float"
                | "long"
                | "int"
                | "short"
                | "byte"
                | "nonNegativeInteger"
                | "positiveInteger"
                | "unsignedLong"
                | "unsignedInt"
        )
    )
}
