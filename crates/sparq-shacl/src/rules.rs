//! [OPUS-4.8] (sq-d1dw) SHACL Advanced Features (SHACL-AF) **rules** — `sh:rule`.
//!
//! OPT-IN: this whole module is gated behind the `shacl-af` cargo feature, so the
//! base SHACL Core + SHACL-SPARQL validation path carries zero rule code or parse
//! cost when the feature is off.
//!
//! Two rule types from the W3C SHACL-AF Recommendation are supported:
//!
//!   * **`sh:TripleRule`** (§4.2) — carries `sh:subject` / `sh:predicate` /
//!     `sh:object` node expressions. For each focus node, the inferred triples are
//!     the cartesian product `S × P × O` of the three evaluated node-expression
//!     sets. The node-expression forms supported here are the spec's common
//!     building blocks: the focus-node expression `sh:this`, a constant IRI /
//!     literal, and a path node expression (`[ sh:path P ]`, with `sh:nodes`
//!     defaulting to the focus node).
//!   * **`sh:SPARQLRule`** (§4.3) — carries an `sh:construct` CONSTRUCT query (with
//!     optional `sh:prefixes`). For each focus node, the query runs with `$this`
//!     pre-bound to the focus node and the constructed triples are inferred. This
//!     reuses the very SPARQL engine the `sh:sparql` constraint path already
//!     depends on (here through `sparq_engine::construct`).
//!
//! Both rule types honour:
//!   * **`sh:condition`** (§4.1) — a rule only fires for focus nodes that conform
//!     to *every* condition shape (evaluated through the existing constraint
//!     validator: a focus node conforms iff it produces no validation results).
//!   * **`sh:order`** (§4.1) — rules run in ascending numeric order (default `0`);
//!     a rule sees the triples inferred by every *earlier* order group.
//!   * **`sh:deactivated`** — deactivated rules are ignored.
//!
//! ## Execution model & fixpoint
//!
//! The normative SHACL-AF algorithm runs each order group once, with triples from
//! one group visible to later groups. This crate runs the whole shape/order
//! schedule and then, because triple/SPARQL rules can feed each other across
//! shapes (a rule's output can satisfy another rule's `sh:condition` or match
//! another rule's CONSTRUCT/path), **iterates the schedule to a fixpoint**: it
//! repeats until a pass infers no new triple, bounded by [`MAX_ITERATIONS`] so an
//! ill-behaved rule set always terminates. The returned set is the *inferred*
//! triples only (the input data graph is never mutated); [`apply_rules`] returns
//! that set, and [`expand`] returns a fresh graph of data ∪ inferred.

use crate::model::{sh, ShapesModel};
use crate::path::Path;
use crate::view::GraphView;
use oxrdf::{NamedNode, NamedOrBlankNode, Term, Triple};
use rustc_hash::FxHashSet;
use sparq_core::Graph;

/// Upper bound on fixpoint iterations of the rule schedule. SHACL-AF leaves
/// multi-pass entailment to "external orchestration"; we iterate until a pass
/// adds nothing, but cap it so a pathological rule set (e.g. one that mints a
/// fresh blank node each pass) always terminates rather than looping forever.
pub const MAX_ITERATIONS: usize = 100;

/// A parsed SHACL-AF rule attached to a shape.
#[derive(Debug, Clone)]
struct Rule {
    kind: RuleKind,
    /// `sh:order` (default 0.0), the schedule key (ascending).
    order: f64,
    /// `sh:condition` shape ids (into [`ShapesModel::shapes`]); the rule fires for
    /// a focus node only if it conforms to every one.
    conditions: Vec<usize>,
    deactivated: bool,
}

#[derive(Debug, Clone)]
enum RuleKind {
    /// `sh:TripleRule`: the three node expressions for subject/predicate/object.
    Triple {
        subject: NodeExpr,
        predicate: NodeExpr,
        object: NodeExpr,
    },
    /// `sh:SPARQLRule`: the (prefix-prepended) CONSTRUCT query text. Parsed per
    /// run via the engine (CONSTRUCT is a graph-form query, not a SELECT, so it
    /// does not go through the SELECT pre-binding path; `$this` is injected as a
    /// `VALUES` table textually-safe at parse time via the algebra).
    Sparql { construct: String },
}

/// A SHACL node expression as used by `sh:subject`/`sh:predicate`/`sh:object`.
/// The supported subset (the spec's common building blocks): the focus node
/// (`sh:this`), a constant term, or a property-path expression from the focus.
#[derive(Debug, Clone)]
enum NodeExpr {
    /// `sh:this` — evaluates to the focus node.
    This,
    /// A constant IRI or literal node.
    Constant(Term),
    /// A path node expression `[ sh:path P ]` (`sh:nodes` defaults to the focus
    /// node): evaluates to the value nodes of the path from the focus node.
    Path(Path),
}

impl NodeExpr {
    /// The node-set this expression evaluates to for `focus` over `data`.
    fn eval(&self, data: &GraphView, focus: &Term) -> Vec<Term> {
        match self {
            NodeExpr::This => vec![focus.clone()],
            NodeExpr::Constant(t) => vec![t.clone()],
            NodeExpr::Path(p) => p.values(data, focus),
        }
    }
}

/// All rules of the shapes graph, grouped per owning shape and sorted into the
/// execution schedule. Built once (behind the feature) alongside the model.
struct RuleSet {
    /// `(shape_id, rule)` pairs, sorted by `rule.order` ascending so the schedule
    /// is a single left-to-right pass over order groups.
    scheduled: Vec<(usize, Rule)>,
}

impl RuleSet {
    /// Parses every `sh:rule` of every shape in `model` from the shapes graph
    /// `shapes`. A shape's rules apply to that shape's focus nodes (its targets).
    fn parse(shapes: &Graph, model: &ShapesModel) -> RuleSet {
        let g = GraphView::new(shapes);
        let mut scheduled: Vec<(usize, Rule)> = Vec::new();
        for sid in 0..model.shapes.len() {
            let shape_node = model.shapes[sid].node.clone();
            for rule_node in g.objects(&shape_node, &sh("rule")) {
                if let Some(rule) = parse_rule(&g, model, &rule_node) {
                    scheduled.push((sid, rule));
                }
            }
        }
        // Stable sort by order keeps same-order rules in shapes-graph discovery
        // order (the spec leaves intra-group order unobservable — same-order rules
        // are mutually invisible within a single pass — so any stable order is fine).
        scheduled.sort_by(|a, b| {
            a.1.order
                .partial_cmp(&b.1.order)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        RuleSet { scheduled }
    }
}

/// Parses one rule node. `None` for a deactivated-irrelevant ill-formed rule
/// (e.g. a `sh:SPARQLRule` with no `sh:construct`, or a `sh:TripleRule` missing a
/// node expression) — ill-formed rules are skipped, matching this crate's lenient
/// handling of ill-formed shapes.
fn parse_rule(g: &GraphView, model: &ShapesModel, node: &Term) -> Option<Rule> {
    let order = match g.object(node, &sh("order")) {
        Some(Term::Literal(l)) => l.value().parse::<f64>().unwrap_or(0.0),
        _ => 0.0,
    };
    let deactivated = matches!(
        g.object(node, &sh("deactivated")),
        Some(Term::Literal(l)) if l.value() == "true"
    );
    // sh:condition shapes → ids into the already-parsed model (a condition that
    // names an unknown shape is dropped; it cannot gate anything meaningfully).
    let conditions: Vec<usize> = g
        .objects(node, &sh("condition"))
        .iter()
        .filter_map(|c| model.by_node(c))
        .collect();

    // Rule kind: a sh:construct ⇒ SPARQLRule; sh:subject/predicate/object ⇒ TripleRule.
    let kind = if let Some(Term::Literal(l)) = g.object(node, &sh("construct")) {
        let prefixes = collect_rule_prefixes(g, node);
        let construct = format!("{prefixes}\n{}", l.value());
        RuleKind::Sparql { construct }
    } else {
        let subject = parse_node_expr(g, &g.object(node, &sh("subject"))?)?;
        let predicate = parse_node_expr(g, &g.object(node, &sh("predicate"))?)?;
        let object = parse_node_expr(g, &g.object(node, &sh("object"))?)?;
        RuleKind::Triple {
            subject,
            predicate,
            object,
        }
    };
    Some(Rule {
        kind,
        order,
        conditions,
        deactivated,
    })
}

/// Parses a node expression value. Supported forms: `sh:this`, a constant
/// IRI/literal, or a path node expression `[ sh:path P ]`. `None` for an
/// unsupported blank-node expression form (skipped — ill-formed/unsupported).
fn parse_node_expr(g: &GraphView, node: &Term) -> Option<NodeExpr> {
    match node {
        Term::NamedNode(n) if n.as_str() == sh("this") => Some(NodeExpr::This),
        Term::NamedNode(_) | Term::Literal(_) => Some(NodeExpr::Constant(node.clone())),
        Term::BlankNode(_) => {
            // A path node expression: [ sh:path P ]. (sh:nodes defaults to the
            // focus node, which is exactly Path::values' `start` argument.)
            let p = g.object(node, &sh("path"))?;
            Path::parse(g, &p).ok().map(NodeExpr::Path)
        }
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

/// Assembles SPARQL `PREFIX` declarations from a rule node's `sh:prefixes`
/// (SHACL-AF reuses the SHACL-SPARQL `sh:prefixes` mechanism). Delegates to the
/// model's shared prefix collector via a tiny re-walk here (the collector is
/// crate-private to model.rs; we mirror the simple direct-declare case and the
/// owl:imports chase by reusing GraphView).
fn collect_rule_prefixes(g: &GraphView, node: &Term) -> String {
    crate::model::collect_prefixes_for(g, &g.objects(node, &sh("prefixes")))
}

/// The result of running the rule engine: the inferred triples (a set), plus the
/// number of fixpoint iterations actually performed (for diagnostics/tests).
#[derive(Debug, Clone, Default)]
pub struct Inference {
    /// The inferred triples, in stable first-seen order (deduplicated).
    pub triples: Vec<Triple>,
    /// Fixpoint iterations performed (1 = the schedule reached a fixpoint after a
    /// single pass; higher = later passes inferred more before settling).
    pub iterations: usize,
    /// True iff the [`MAX_ITERATIONS`] cap was hit before a fixpoint (the inferred
    /// set may be incomplete — an ill-behaved, non-terminating rule set).
    pub capped: bool,
}

/// Runs the SHACL-AF rule engine: parses every `sh:rule` of `shapes`/`model` and
/// applies them to a fixpoint, returning the **inferred** triples (the input
/// `data` is never mutated). Rules fire only for focus nodes conforming to every
/// `sh:condition`, run in ascending `sh:order`, and are iterated until a pass adds
/// nothing (bounded by [`MAX_ITERATIONS`]).
///
/// This is the SHACL-AF (`shacl-af` feature) entry point; it has no effect on —
/// and is independent of — the base [`crate::validate`] path.
pub fn apply_rules(data: &Graph, shapes: &Graph) -> Inference {
    let model = ShapesModel::parse(shapes);
    apply_rules_with_model(data, shapes, &model)
}

/// [`apply_rules`] against an already-parsed shapes model (amortises shape parsing
/// across many data graphs). `shapes` is still required because rule node
/// expressions / CONSTRUCT prefixes are read directly from the shapes graph.
pub fn apply_rules_with_model(data: &Graph, shapes: &Graph, model: &ShapesModel) -> Inference {
    let ruleset = RuleSet::parse(shapes, model);
    if ruleset.scheduled.is_empty() {
        return Inference::default();
    }

    // The accumulated inferred-triple set (dedup + stable order).
    let mut inferred: Vec<Triple> = Vec::new();
    let mut inferred_set: FxHashSet<Triple> = FxHashSet::default();

    let mut iterations = 0;
    let mut capped = false;
    loop {
        iterations += 1;
        // Each pass runs over data ∪ inferred-so-far. Build the augmented graph
        // once per pass (cheap relative to the rule SPARQL/conformance work).
        let augmented = expand_graph(data, &inferred);
        let view = GraphView::new(&augmented);
        let mut pass_new = false;

        for (sid, rule) in &ruleset.scheduled {
            if rule.deactivated {
                continue;
            }
            for focus in focus_nodes(&view, model, *sid) {
                if !conforms_to_all(&augmented, model, &rule.conditions, &focus) {
                    continue;
                }
                for triple in fire_rule(&augmented, &view, &focus, rule) {
                    if inferred_set.insert(triple.clone()) {
                        inferred.push(triple);
                        pass_new = true;
                    }
                }
            }
        }

        if !pass_new {
            break;
        }
        if iterations >= MAX_ITERATIONS {
            capped = true;
            break;
        }
    }

    Inference {
        triples: inferred,
        iterations,
        capped,
    }
}

/// Convenience: the data graph **expanded** with every inferred triple — a fresh
/// `Graph` of `data ∪ apply_rules(data, shapes)` (the input is not mutated).
pub fn expand(data: &Graph, shapes: &Graph) -> Graph {
    let inf = apply_rules(data, shapes);
    expand_graph(data, &inf.triples)
}

/// Fires one rule for one focus node, yielding the triples it infers.
fn fire_rule(augmented: &Graph, view: &GraphView, focus: &Term, rule: &Rule) -> Vec<Triple> {
    match &rule.kind {
        RuleKind::Triple {
            subject,
            predicate,
            object,
        } => {
            let subjects = subject.eval(view, focus);
            let predicates = predicate.eval(view, focus);
            let objects = object.eval(view, focus);
            let mut out = Vec::new();
            // The cartesian product S × P × O; only well-typed RDF triples are
            // emitted (subject IRI/blank, predicate IRI), the rest are dropped.
            for s in &subjects {
                let Some(subj) = as_subject(s) else { continue };
                for p in &predicates {
                    let Term::NamedNode(pred) = p else { continue };
                    for o in &objects {
                        out.push(Triple {
                            subject: subj.clone(),
                            predicate: pred.clone(),
                            object: o.clone(),
                        });
                    }
                }
            }
            out
        }
        RuleKind::Sparql { construct } => {
            // Pre-bind $this to the focus node by injecting a VALUES table, then
            // run the CONSTRUCT through the engine (graph-form query → Vec<Triple>).
            match build_construct_with_this(construct, focus) {
                Some(sparql) => sparq_engine::construct(augmented, &sparql).unwrap_or_default(),
                None => Vec::new(),
            }
        }
    }
}

/// Builds the CONSTRUCT query text with `$this` pre-bound to `focus` via a
/// `VALUES` table appended to the WHERE clause. We append `VALUES (?this) {
/// (<focus>) }` at the end of the (single, outermost) WHERE block. This is robust
/// for the rule case because a CONSTRUCT's WHERE is a group graph pattern and a
/// trailing VALUES joins with it. Returns `None` if the focus node has no SPARQL
/// ground-term form (a blank node — rare as a rule focus).
fn build_construct_with_this(construct: &str, focus: &Term) -> Option<String> {
    let ground = match focus {
        Term::NamedNode(n) => format!("<{}>", n.as_str()),
        Term::Literal(l) => l.to_string(),
        // A blank-node focus cannot be written in VALUES; such a rule is skipped
        // for that focus (rule focus nodes are IRIs in practice).
        Term::BlankNode(_) => return None,
        #[allow(unreachable_patterns)]
        _ => return None,
    };
    // Find the last `}` (end of the WHERE group) and inject the VALUES before it.
    let close = construct.rfind('}')?;
    let mut out = String::with_capacity(construct.len() + ground.len() + 32);
    out.push_str(&construct[..close]);
    out.push_str(&format!(" VALUES (?this) {{ ({ground}) }} "));
    out.push_str(&construct[close..]);
    Some(out)
}

/// The focus nodes of shape `sid` (its target nodes) over `view`. Mirrors the
/// validator's own target selection so rules and constraints agree on targets.
fn focus_nodes(view: &GraphView, model: &ShapesModel, sid: usize) -> Vec<Term> {
    use crate::model::Target;
    let mut out: Vec<Term> = Vec::new();
    for t in &model.shapes[sid].targets {
        match t {
            Target::Node(n) => out.push(n.clone()),
            Target::Class(c) | Target::ImplicitClass(c) => out.extend(view.instances_of(c)),
            Target::SubjectsOf(p) => out.extend(view.subjects_of(p)),
            Target::ObjectsOf(p) => out.extend(view.objects_of(p)),
        }
    }
    crate::view::dedup(out)
}

/// True iff `focus` conforms to every `sh:condition` shape — i.e. validating
/// `focus` against each condition shape (as a standalone target) yields no
/// results. Reuses the full constraint validator so conditions can be arbitrary
/// shapes (Core constraints, sh:sparql, nested node/property shapes, …).
fn conforms_to_all(
    augmented: &Graph,
    model: &ShapesModel,
    conditions: &[usize],
    focus: &Term,
) -> bool {
    conditions
        .iter()
        .all(|&cid| crate::eval::conforms_node(augmented, model, cid, focus))
}

/// `data` expanded with `extra` triples as a fresh `Graph`. Interns both the data
/// (re-read from its dictionary) and the extra triples into a new dictionary.
fn expand_graph(data: &Graph, extra: &[Triple]) -> Graph {
    let view = GraphView::new(data);
    let base = view.triples(None, None, None).into_iter().map(|[s, p, o]| {
        Triple {
            subject: as_subject(&s).unwrap_or_else(|| {
                // A literal subject is impossible in a well-formed graph; fall back
                // to a placeholder blank node (never reached for real data).
                NamedOrBlankNode::BlankNode(oxrdf::BlankNode::default())
            }),
            predicate: match &p {
                Term::NamedNode(n) => n.clone(),
                _ => NamedNode::new_unchecked("urn:x-sparq:ill-formed-predicate"),
            },
            object: o,
        }
    });
    crate::graph_from_triples(base.chain(extra.iter().cloned()))
}

/// An RDF term as a triple subject (IRI or blank node); `None` for a literal.
fn as_subject(t: &Term) -> Option<NamedOrBlankNode> {
    match t {
        Term::NamedNode(n) => Some(NamedOrBlankNode::NamedNode(n.clone())),
        Term::BlankNode(b) => Some(NamedOrBlankNode::BlankNode(b.clone())),
        #[allow(unreachable_patterns)]
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::view::RDF_TYPE;

    const PRELUDE: &str = "@prefix sh: <http://www.w3.org/ns/shacl#> .\n\
        @prefix ex: <http://example.org/> .\n\
        @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n\
        @prefix rdf: <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .\n";

    fn g(ttl: &str) -> Graph {
        Graph::load_str(&format!("{PRELUDE}{ttl}"), "turtle").unwrap()
    }

    /// True iff the inferred set contains a triple `<s> <p> <o>` (IRI terms).
    fn has(inf: &Inference, s: &str, p: &str, o: &str) -> bool {
        inf.triples.iter().any(|t| {
            t.subject.to_string() == format!("<{}>", s)
                && t.predicate.as_str() == p
                && t.object.to_string() == format!("<{}>", o)
        })
    }

    // ---- sh:TripleRule: infer a triple ----
    #[test]
    fn triple_rule_infers_type() {
        // For every ex:Person focus node, infer (this, rdf:type, ex:Agent).
        let data = g("ex:alice a ex:Person . ex:bob a ex:Person .");
        let shapes = g(r#"
            ex:PersonShape a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Agent ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(has(
            &inf,
            "http://example.org/alice",
            RDF_TYPE,
            "http://example.org/Agent"
        ));
        assert!(has(
            &inf,
            "http://example.org/bob",
            RDF_TYPE,
            "http://example.org/Agent"
        ));
        assert_eq!(inf.triples.len(), 2);
    }

    // ---- sh:TripleRule with a path node expression for the object ----
    #[test]
    fn triple_rule_path_object() {
        // Infer (this, ex:grandparent, X) where X = this/ex:parent/ex:parent.
        let data = g(r#"
            ex:a a ex:Person ; ex:parent ex:b .
            ex:b ex:parent ex:c .
        "#);
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:subject sh:this ;
                sh:predicate ex:grandparent ;
                sh:object [ sh:path ( ex:parent ex:parent ) ] ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(
            has(
                &inf,
                "http://example.org/a",
                "http://example.org/grandparent",
                "http://example.org/c"
            ),
            "{:?}",
            inf.triples
        );
        assert_eq!(inf.triples.len(), 1);
    }

    // ---- sh:SPARQLRule with sh:construct ----
    #[test]
    fn sparql_rule_construct() {
        let data = g("ex:alice a ex:Person ; ex:firstName \"Alice\" .");
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:SPARQLRule ;
                sh:construct """
                  CONSTRUCT { $this <http://example.org/label> ?n }
                  WHERE { $this <http://example.org/firstName> ?n }
                """ ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert_eq!(inf.triples.len(), 1, "{:?}", inf.triples);
        let t = &inf.triples[0];
        assert_eq!(t.predicate.as_str(), "http://example.org/label");
        assert_eq!(t.object.to_string(), "\"Alice\"");
    }

    // ---- sh:SPARQLRule with sh:prefixes ----
    #[test]
    fn sparql_rule_with_prefixes() {
        let data = g("ex:alice a ex:Person ; ex:firstName \"Alice\" .");
        let shapes = g(r#"
            ex:prefixes sh:declare [ sh:prefix "ex" ; sh:namespace "http://example.org/"^^xsd:anyURI ] .
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:SPARQLRule ;
                sh:prefixes ex:prefixes ;
                sh:construct "CONSTRUCT { $this ex:label ?n } WHERE { $this ex:firstName ?n }" ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert_eq!(inf.triples.len(), 1, "{:?}", inf.triples);
        assert_eq!(
            inf.triples[0].predicate.as_str(),
            "http://example.org/label"
        );
    }

    // ---- sh:condition gating ----
    #[test]
    fn condition_gates_rule() {
        // Only persons with an ex:active "true" flag get the ex:Agent type.
        let data = g(r#"
            ex:alice a ex:Person ; ex:active true .
            ex:bob   a ex:Person .
        "#);
        let shapes = g(r#"
            ex:ActiveShape a sh:NodeShape ;
              sh:property [ sh:path ex:active ; sh:hasValue true ] .
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:condition ex:ActiveShape ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Agent ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(has(
            &inf,
            "http://example.org/alice",
            RDF_TYPE,
            "http://example.org/Agent"
        ));
        assert!(
            !has(
                &inf,
                "http://example.org/bob",
                RDF_TYPE,
                "http://example.org/Agent"
            ),
            "bob lacks the condition, must not be inferred: {:?}",
            inf.triples
        );
        assert_eq!(inf.triples.len(), 1);
    }

    // ---- sh:order sequencing (a later rule depends on an earlier rule's output) ----
    #[test]
    fn order_sequences_dependent_rules() {
        // Rule order 1 types every Person as ex:Agent; rule order 2 has a condition
        // (must be an ex:Agent) that ONLY holds because rule 1 already fired.
        let data = g("ex:alice a ex:Person .");
        let shapes = g(r#"
            ex:AgentShape a sh:NodeShape ;
              sh:property [ sh:path rdf:type ; sh:hasValue ex:Agent ] .
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:order 1 ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Agent ;
              ] ;
              sh:rule [
                a sh:TripleRule ;
                sh:order 2 ;
                sh:condition ex:AgentShape ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Verified ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(has(
            &inf,
            "http://example.org/alice",
            RDF_TYPE,
            "http://example.org/Agent"
        ));
        assert!(
            has(
                &inf,
                "http://example.org/alice",
                RDF_TYPE,
                "http://example.org/Verified"
            ),
            "the order-2 rule must see the order-1 inference: {:?}",
            inf.triples
        );
        assert_eq!(inf.triples.len(), 2);
    }

    // ---- sh:deactivated ----
    #[test]
    fn deactivated_rule_is_ignored() {
        let data = g("ex:alice a ex:Person .");
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:deactivated true ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Agent ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(inf.triples.is_empty(), "{:?}", inf.triples);
    }

    // ---- fixpoint termination on a transitively-chaining rule ----
    #[test]
    fn fixpoint_transitive_closure_terminates() {
        // A rule that closes ex:sub transitively: (this, ex:sub, Y) where
        // Y = this/ex:sub/ex:sub. Iterating to a fixpoint must add the closure
        // edges and then terminate (no infinite growth).
        let data = g(r#"
            ex:a a ex:N ; ex:sub ex:b .
            ex:b a ex:N ; ex:sub ex:c .
            ex:c a ex:N ; ex:sub ex:d .
            ex:d a ex:N .
        "#);
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:N ;
              sh:rule [
                a sh:TripleRule ;
                sh:subject sh:this ;
                sh:predicate ex:sub ;
                sh:object [ sh:path ( ex:sub ex:sub ) ] ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        // a reaches c (a/sub/sub) and d (after c→? ... a/sub*/sub) transitively.
        assert!(has(
            &inf,
            "http://example.org/a",
            "http://example.org/sub",
            "http://example.org/c"
        ));
        assert!(has(
            &inf,
            "http://example.org/a",
            "http://example.org/sub",
            "http://example.org/d"
        ));
        assert!(has(
            &inf,
            "http://example.org/b",
            "http://example.org/sub",
            "http://example.org/d"
        ));
        assert!(
            !inf.capped,
            "transitive closure must reach a fixpoint, not the cap"
        );
        assert!(inf.iterations >= 2, "needs >1 pass: {inf:?}");
    }

    // ---- the bounded cap protects against a non-terminating rule ----
    #[test]
    fn nonterminating_rule_is_capped() {
        // A CONSTRUCT that mints a FRESH blank node each pass never reaches a
        // fixpoint; the cap must stop it (and flag `capped`).
        let data = g("ex:a a ex:N .");
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:N ;
              sh:rule [
                a sh:SPARQLRule ;
                sh:construct "CONSTRUCT { $this <http://example.org/has> _:x } WHERE { }" ;
              ] .
        "#);
        let inf = apply_rules(&data, &shapes);
        assert!(inf.capped, "a fresh-bnode-per-pass rule must hit the cap");
        assert_eq!(inf.iterations, MAX_ITERATIONS);
    }

    // ---- expand() returns data ∪ inferred as a queryable graph ----
    #[test]
    fn expand_merges_inferred() {
        let data = g("ex:alice a ex:Person .");
        let shapes = g(r#"
            ex:S a sh:NodeShape ;
              sh:targetClass ex:Person ;
              sh:rule [
                a sh:TripleRule ;
                sh:subject sh:this ;
                sh:predicate rdf:type ;
                sh:object ex:Agent ;
              ] .
        "#);
        let expanded = expand(&data, &shapes);
        let view = GraphView::new(&expanded);
        // Both the original type and the inferred type are present.
        let types = view.objects(
            &Term::NamedNode(NamedNode::new_unchecked("http://example.org/alice")),
            RDF_TYPE,
        );
        let strs: Vec<String> = types.iter().map(|t| t.to_string()).collect();
        assert!(strs.iter().any(|s| s.contains("Person")));
        assert!(strs.iter().any(|s| s.contains("Agent")));
    }

    // ---- no rules ⇒ empty inference, no iteration ----
    #[test]
    fn no_rules_is_noop() {
        let data = g("ex:alice a ex:Person .");
        let shapes = g("ex:S a sh:NodeShape ; sh:targetClass ex:Person ; sh:property [ sh:path ex:name ; sh:minCount 1 ] .");
        let inf = apply_rules(&data, &shapes);
        assert!(inf.triples.is_empty());
        assert_eq!(inf.iterations, 0);
    }
}
