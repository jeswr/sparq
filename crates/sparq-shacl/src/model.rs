//! The shapes model: parsing a shapes graph into [`Shape`] structs with their
//! targets, paths and constraint components.

use crate::path::Path;
use crate::view::{GraphView, RDF_TYPE};
use oxrdf::Term;
use rustc_hash::FxHashMap;
use sparq_core::Graph;

pub const SH: &str = "http://www.w3.org/ns/shacl#";
const RDFS_CLASS: &str = "http://www.w3.org/2000/01/rdf-schema#Class";

pub(crate) fn sh(local: &str) -> String {
    format!("{SH}{local}")
}

/// How a shape selects its focus nodes.
#[derive(Debug, Clone)]
pub enum Target {
    /// sh:targetNode — the node itself (need not occur in the data graph).
    Node(Term),
    /// sh:targetClass — all SHACL instances of the class in the data graph.
    Class(Term),
    /// Implicit class target: the shape is itself an rdfs:Class.
    ImplicitClass(Term),
    /// sh:targetSubjectsOf — all subjects of the predicate.
    SubjectsOf(String),
    /// sh:targetObjectsOf — all objects of the predicate.
    ObjectsOf(String),
}

/// One occurrence of a constraint component on a shape.
#[derive(Debug, Clone)]
pub enum Component {
    Class(Term),
    Datatype(String),
    NodeKind(String),
    MinCount(u64),
    MaxCount(u64),
    MinExclusive(Term),
    MinInclusive(Term),
    MaxExclusive(Term),
    MaxInclusive(Term),
    MinLength(u64),
    MaxLength(u64),
    Pattern {
        source: String,
        flags: Option<String>,
    },
    LanguageIn(Vec<String>),
    UniqueLang,
    Equals(String),
    Disjoint(String),
    LessThan(String),
    LessThanOrEquals(String),
    Not(usize),
    And(Vec<usize>),
    Or(Vec<usize>),
    Xone(Vec<usize>),
    Node(usize),
    /// sh:property — a child property shape validated against the same focus.
    Property(usize),
    Qualified {
        shape: usize,
        min: Option<u64>,
        max: Option<u64>,
        disjoint: bool,
        /// Sibling qualified value shapes (for sh:qualifiedValueShapesDisjoint).
        siblings: Vec<usize>,
    },
    Closed {
        ignored: Vec<Term>,
    },
    HasValue(Term),
    In(Vec<Term>),
    /// sh:sparql — a SPARQL-based constraint (SHACL §5.2). The index is into
    /// [`ShapesModel::sparql`].
    Sparql(usize),
    /// [OPUS-4.8] A SPARQL-based constraint COMPONENT (SHACL §6) that activated on
    /// this shape because the shape uses the component's parameter predicates.
    /// `component` indexes [`ShapesModel::components`]; `args` are the bound
    /// parameter values, parallel to the component's `parameters` (one term each
    /// — the first object found for a mandatory parameter; `None` for an absent
    /// optional one). The validator pre-binds each as `$paramName`.
    CustomSparql {
        component: usize,
        args: Vec<Option<Term>>,
    },
    /// [OPUS-4.8] (sq-mk9n, `shacl-af`) `sh:expression` — the SHACL-AF
    /// *Expression Constraint* (`sh:ExpressionConstraintComponent`): a value node
    /// `v` is violated when the node expression does NOT evaluate to `{ true }`
    /// for `v` as focus. The index is into [`ShapesModel::expressions`] (the
    /// parsed node expression is held there, off the public `Component` enum).
    #[cfg(feature = "shacl-af")]
    Expression(usize),
}

/// [OPUS-4.8] A declared `sh:parameter` of a SPARQL-based constraint component
/// (SHACL §6.2): its predicate (`sh:path`), the pre-bound variable name and
/// whether it is `sh:optional`.
#[derive(Debug, Clone)]
pub(crate) struct ComponentParameter {
    /// The parameter's predicate IRI (`sh:path` of the parameter) — a shape
    /// "uses" the parameter by carrying a triple with this predicate.
    pub predicate: String,
    /// The pre-bound variable name (`$name`): the parameter's `sh:name`, else the
    /// local name of its predicate IRI (SHACL §6.2.1).
    pub var: String,
    /// `sh:optional true` — the parameter need not be present for the component
    /// to activate (a mandatory parameter must be present).
    pub optional: bool,
}

/// [OPUS-4.8] A SPARQL-based constraint component declaration (SHACL §6.2): its
/// parameters and a validator. The validator is chosen by shape kind at
/// evaluation time: `sh:nodeValidator` for node shapes, `sh:propertyValidator`
/// for property shapes, falling back to the generic `sh:validator`. Each carries
/// a `sh:ask` or `sh:select` query (compiled into [`PreparedValidator`]).
#[derive(Debug, Clone)]
pub(crate) struct ComponentDef {
    /// The component's node (for diagnostics / `sh:sourceConstraintComponent`).
    pub node: Term,
    pub parameters: Vec<ComponentParameter>,
    /// Generic validator (`sh:validator`) — used when no kind-specific one fits.
    pub validator: Option<PreparedComponentValidator>,
    /// `sh:nodeValidator` — preferred for node shapes.
    pub node_validator: Option<PreparedComponentValidator>,
    /// `sh:propertyValidator` — preferred for property shapes.
    pub property_validator: Option<PreparedComponentValidator>,
}

impl ComponentDef {
    /// The validator to run for a shape of the given kind: the kind-specific one
    /// if present, else the generic `sh:validator` (SHACL §6.2.2).
    pub fn validator_for(&self, is_property_shape: bool) -> Option<&PreparedComponentValidator> {
        let specific = if is_property_shape {
            self.property_validator.as_ref()
        } else {
            self.node_validator.as_ref()
        };
        specific.or(self.validator.as_ref())
    }
}

/// [OPUS-4.8] A compiled component validator: the parsed ASK/SELECT query plus
/// its own `sh:message` template (SHACL §6.3 uses the validator's `sh:message`
/// for the produced results).
#[derive(Debug, Clone)]
pub(crate) struct PreparedComponentValidator {
    pub prepared: crate::sparql::PreparedValidator,
    pub message: Option<String>,
}

/// A `sh:sparql` constraint's components (SHACL §5.2): the `sh:select` query,
/// its `sh:prefixes` declarations and an optional constraint-level `sh:message`.
/// The parsed/validated query is held in [`crate::sparql::PreparedSparql`]; a
/// `None` prepared form means the query was ill-formed (and the constraint is
/// skipped, matching this crate's lenient handling of ill-formed shapes).
#[derive(Debug, Clone)]
pub(crate) struct SparqlConstraint {
    /// The raw `sh:select` text.
    pub select: String,
    /// PREFIX declarations assembled from `sh:prefixes` (`sh:declare` →
    /// `sh:prefix` / `sh:namespace`), prepended to `select` before parsing.
    pub prefixes: String,
    /// The constraint's own `sh:message` template, if any (takes precedence over
    /// a `?message` binding and over the shape's `sh:message`).
    pub message: Option<String>,
    /// `sh:deactivated true` on the constraint node.
    pub deactivated: bool,
    /// The parsed query; `None` if `select` did not parse as a SELECT — the
    /// constraint is then skipped.
    pub prepared: Option<crate::sparql::PreparedSparql>,
}

#[derive(Debug)]
pub struct Shape {
    /// The shape's node in the shapes graph.
    pub node: Term,
    /// `Some` for property shapes (subjects of sh:path).
    pub path: Option<Path>,
    pub targets: Vec<Target>,
    pub components: Vec<Component>,
    /// Severity IRI (default sh:Violation).
    pub severity: String,
    /// sh:message literals, copied into results.
    pub messages: Vec<Term>,
    pub deactivated: bool,
    /// Child property shapes (sh:property) — also feeds sh:closed.
    pub property_children: Vec<usize>,
}

/// All shapes parsed from a shapes graph, indexed densely (cycle-safe).
pub struct ShapesModel {
    pub shapes: Vec<Shape>,
    by_node: FxHashMap<Term, usize>,
    /// Shapes that have at least one target — the validation entry points.
    pub targeted: Vec<usize>,
    /// SPARQL-based constraints (`sh:sparql`), referenced by [`Component::Sparql`].
    pub(crate) sparql: Vec<SparqlConstraint>,
    /// [OPUS-4.8] SPARQL-based constraint COMPONENTS (`sh:ConstraintComponent`,
    /// SHACL §6) declared in the shapes graph, referenced by
    /// [`Component::CustomSparql`]. The registry is keyed (for activation) on the
    /// mandatory parameter predicates each component declares.
    pub(crate) components: Vec<ComponentDef>,
    /// [OPUS-4.8] (sq-mk9n, `shacl-af`) Parsed `sh:expression` node expressions,
    /// referenced by [`Component::Expression`]. Kept off the public `Component`
    /// enum so the (crate-private) node-expression type is not exposed.
    #[cfg(feature = "shacl-af")]
    pub(crate) expressions: Vec<crate::rules::NodeExpr>,
}

impl ShapesModel {
    pub fn parse(shapes_graph: &Graph) -> ShapesModel {
        let g = GraphView::new(shapes_graph);
        let mut m = ShapesModel {
            shapes: Vec::new(),
            by_node: FxHashMap::default(),
            targeted: Vec::new(),
            sparql: Vec::new(),
            components: Vec::new(),
            #[cfg(feature = "shacl-af")]
            expressions: Vec::new(),
        };

        // [OPUS-4.8] SHACL §6: discover SPARQL-based constraint components FIRST, so
        // shape parsing can activate them against the parameter predicates a shape uses.
        m.components = discover_components(&g);

        // Top-level shape discovery: explicitly typed shapes plus anything with a target.
        let mut roots: Vec<Term> = Vec::new();
        for class in ["NodeShape", "PropertyShape"] {
            for t in g.triples(None, Some(RDF_TYPE), Some(&iri(&sh(class)))) {
                roots.push(t[0].clone());
            }
        }
        for pred in [
            "targetNode",
            "targetClass",
            "targetSubjectsOf",
            "targetObjectsOf",
        ] {
            roots.extend(g.subjects_of(&sh(pred)));
        }
        // [OPUS-4.8] Implicit class shapes: a node that is an rdfs:Class with SHACL constraints is
        // itself a node shape (with an implicit class target) per the SHACL spec. Root discovery
        // previously only collected explicitly-typed shapes and shapes with sh:target* — so an
        // implicit class shape with neither was never parsed and its constraints silently ignored.
        // Include rdfs:Class subjects as root candidates; parse_shape only attaches the implicit
        // target to nodes actually typed rdfs:Class (and a constraint-free class parses to a
        // no-op shape, which validates nothing). See review 1616.
        for t in g.triples(None, Some(RDF_TYPE), Some(&iri(RDFS_CLASS))) {
            roots.push(t[0].clone());
        }
        for root in crate::view::dedup(roots) {
            m.shape_id(&g, &root);
        }

        // Fix up qualified-value-shape sibling lists: the qualified shapes of the
        // OTHER property shapes sharing a parent node shape.
        for parent in 0..m.shapes.len() {
            let children = m.shapes[parent].property_children.clone();
            if children.len() < 2 {
                continue;
            }
            let mut quals: Vec<(usize, Vec<usize>)> = Vec::new(); // (child, its qualified shapes)
            for &c in &children {
                let qs: Vec<usize> = m.shapes[c]
                    .components
                    .iter()
                    .filter_map(|comp| match comp {
                        Component::Qualified { shape, .. } => Some(*shape),
                        _ => None,
                    })
                    .collect();
                quals.push((c, qs));
            }
            for (child, _) in &quals {
                let siblings: Vec<usize> = quals
                    .iter()
                    .filter(|(c, _)| c != child)
                    .flat_map(|(_, qs)| qs.iter().copied())
                    .collect();
                for comp in &mut m.shapes[*child].components {
                    if let Component::Qualified { siblings: s, .. } = comp {
                        *s = siblings.clone();
                    }
                }
            }
        }

        // [OPUS-4.8] (sq-mk9n, `shacl-af`) Second pass: parse each shape's
        // `sh:expression` node expression into a component. Done after shape
        // discovery so a filter expression can reference any declared shape; an
        // inline anonymous filter shape inside the expression is registered on
        // demand here so it is parsed and conformance-checkable at eval time.
        #[cfg(feature = "shacl-af")]
        m.parse_expression_constraints(shapes_graph);

        // Validation entry points: shapes with targets.
        m.targeted = (0..m.shapes.len())
            .filter(|&i| !m.shapes[i].targets.is_empty())
            .collect();
        m
    }

    /// [OPUS-4.8] (sq-mk9n, `shacl-af`) Parses `sh:expression` on every shape into
    /// a [`Component::Expression`]. Inline filter shapes used by the expression are
    /// registered first (so they are conformance-checkable), then the expressions
    /// are parsed and attached.
    #[cfg(feature = "shacl-af")]
    fn parse_expression_constraints(&mut self, shapes_graph: &Graph) {
        let g = GraphView::new(shapes_graph);
        // Collect (shape_id, expression_term) for every shape carrying sh:expression.
        let mut pending: Vec<(usize, Term)> = Vec::new();
        for sid in 0..self.shapes.len() {
            let node = self.shapes[sid].node.clone();
            for expr in g.objects(&node, &sh("expression")) {
                pending.push((sid, expr));
            }
        }
        // Register any inline filter shapes the expressions reference (best-effort)
        // so `sh:filterShape` / function filter shapes resolve at eval time.
        for (_, expr) in &pending {
            self.register_expression_shapes(shapes_graph, expr, 0);
        }
        // Parse each expression (immutable borrow) and attach the component.
        let parsed: Vec<(usize, crate::rules::NodeExpr)> = pending
            .into_iter()
            .filter_map(|(sid, expr)| {
                crate::rules::parse_node_expr(&g, self, &expr).map(|ne| (sid, ne))
            })
            .collect();
        for (sid, expr) in parsed {
            let idx = self.expressions.len();
            self.expressions.push(expr);
            self.shapes[sid].components.push(Component::Expression(idx));
        }
    }

    /// Walks an expression term registering inline `sh:filterShape` / function
    /// filter shapes (`shnex:`/`sh:` `findFirst`/`matchAll`/`nodesMatching`) as
    /// shapes so they resolve. Depth-bounded against pathological cyclic graphs.
    #[cfg(feature = "shacl-af")]
    fn register_expression_shapes(&mut self, shapes_graph: &Graph, term: &Term, depth: usize) {
        if depth > 64 || matches!(term, Term::Literal(_)) {
            return;
        }
        let g = GraphView::new(shapes_graph);
        const SHNEX: &str = "http://www.w3.org/ns/shacl-node-expr#";
        for local in ["filterShape", "findFirst", "matchAll", "nodesMatching"] {
            for pred in [format!("{SHNEX}{local}"), sh(local)] {
                if let Some(shape_term) = g.object(term, &pred) {
                    self.ensure_shape(shapes_graph, &shape_term);
                }
            }
        }
        // Recurse into nested operand expressions / list members.
        for (_, obj) in g.predicate_objects(term) {
            if matches!(obj, Term::BlankNode(_)) {
                self.register_expression_shapes(shapes_graph, &obj, depth + 1);
            }
        }
    }

    pub fn by_node(&self, node: &Term) -> Option<usize> {
        self.by_node.get(node).copied()
    }

    /// [OPUS-4.8] (sq-mk9n, `shacl-af`) Ensures `node` is parsed as a shape and
    /// returns its id, parsing it on demand from `shapes_graph` if it was not
    /// discovered by top-level root discovery. SHACL-AF node-expression *filter
    /// shapes* (`sh:filterShape`, `shnex:findFirst`/`matchAll`/`nodesMatching`)
    /// may be **inline anonymous** shapes (e.g. `[ sh:minInclusive 3 ]`) that
    /// carry no `rdf:type`/target, so they are not roots; this lets the function
    /// registry register such a shape and then check conformance against it.
    /// `None` only if `node` is a literal (literals are never shapes).
    #[cfg(feature = "shacl-af")]
    pub(crate) fn ensure_shape(&mut self, shapes_graph: &Graph, node: &Term) -> Option<usize> {
        if matches!(node, Term::Literal(_)) {
            return None;
        }
        if let Some(id) = self.by_node(node) {
            return Some(id);
        }
        let g = GraphView::new(shapes_graph);
        Some(self.shape_id(&g, node))
    }

    /// The id of the shape rooted at `node`, parsing it (and, recursively, the
    /// shapes it references) on first sight. A placeholder breaks cycles.
    fn shape_id(&mut self, g: &GraphView, node: &Term) -> usize {
        if let Some(&id) = self.by_node.get(node) {
            return id;
        }
        let id = self.shapes.len();
        self.by_node.insert(node.clone(), id);
        self.shapes.push(Shape {
            node: node.clone(),
            path: None,
            targets: Vec::new(),
            components: Vec::new(),
            severity: sh("Violation"),
            messages: Vec::new(),
            deactivated: false,
            property_children: Vec::new(),
        });
        let parsed = self.parse_shape(g, node);
        self.shapes[id] = parsed;
        id
    }

    fn parse_shape(&mut self, g: &GraphView, node: &Term) -> Shape {
        let mut shape = Shape {
            node: node.clone(),
            path: g
                .object(node, &sh("path"))
                .and_then(|p| Path::parse(g, &p).ok()),
            targets: Vec::new(),
            components: Vec::new(),
            severity: match g.object(node, &sh("severity")) {
                Some(Term::NamedNode(n)) => n.as_str().to_string(),
                _ => sh("Violation"),
            },
            messages: g.objects(node, &sh("message")),
            deactivated: matches!(
                g.object(node, &sh("deactivated")),
                Some(Term::Literal(l)) if l.value() == "true"
            ),
            property_children: Vec::new(),
        };

        // Targets.
        for t in g.objects(node, &sh("targetNode")) {
            shape.targets.push(Target::Node(t));
        }
        for t in g.objects(node, &sh("targetClass")) {
            shape.targets.push(Target::Class(t));
        }
        for t in g.objects(node, &sh("targetSubjectsOf")) {
            if let Term::NamedNode(n) = t {
                shape
                    .targets
                    .push(Target::SubjectsOf(n.as_str().to_string()));
            }
        }
        for t in g.objects(node, &sh("targetObjectsOf")) {
            if let Term::NamedNode(n) = t {
                shape
                    .targets
                    .push(Target::ObjectsOf(n.as_str().to_string()));
            }
        }
        // Implicit class target: the shape node is itself an rdfs:Class.
        if matches!(node, Term::NamedNode(_)) && g.has_type(node, RDFS_CLASS) {
            shape.targets.push(Target::ImplicitClass(node.clone()));
        }

        let c = &mut shape.components;
        for o in g.objects(node, &sh("class")) {
            c.push(Component::Class(o));
        }
        for o in g.objects(node, &sh("datatype")) {
            if let Term::NamedNode(n) = o {
                c.push(Component::Datatype(n.as_str().to_string()));
            }
        }
        for o in g.objects(node, &sh("nodeKind")) {
            if let Term::NamedNode(n) = o {
                c.push(Component::NodeKind(n.as_str().to_string()));
            }
        }
        for (pred, ctor) in [
            ("minCount", Component::MinCount as fn(u64) -> Component),
            ("maxCount", Component::MaxCount as fn(u64) -> Component),
            ("minLength", Component::MinLength as fn(u64) -> Component),
            ("maxLength", Component::MaxLength as fn(u64) -> Component),
        ] {
            for o in g.objects(node, &sh(pred)) {
                if let Term::Literal(l) = &o {
                    if let Ok(n) = l.value().parse::<u64>() {
                        c.push(ctor(n));
                    }
                }
            }
        }
        for (pred, ctor) in [
            (
                "minExclusive",
                Component::MinExclusive as fn(Term) -> Component,
            ),
            (
                "minInclusive",
                Component::MinInclusive as fn(Term) -> Component,
            ),
            (
                "maxExclusive",
                Component::MaxExclusive as fn(Term) -> Component,
            ),
            (
                "maxInclusive",
                Component::MaxInclusive as fn(Term) -> Component,
            ),
        ] {
            for o in g.objects(node, &sh(pred)) {
                c.push(ctor(o));
            }
        }
        let flags = g.str_object(node, &sh("flags"));
        for o in g.objects(node, &sh("pattern")) {
            if let Term::Literal(l) = &o {
                c.push(Component::Pattern {
                    source: l.value().to_string(),
                    flags: flags.clone(),
                });
            }
        }
        for o in g.objects(node, &sh("languageIn")) {
            let tags: Vec<String> = g
                .list(&o)
                .into_iter()
                .filter_map(|t| match t {
                    Term::Literal(l) => Some(l.value().to_string()),
                    _ => None,
                })
                .collect();
            c.push(Component::LanguageIn(tags));
        }
        if matches!(g.object(node, &sh("uniqueLang")), Some(Term::Literal(l)) if l.value() == "true")
        {
            c.push(Component::UniqueLang);
        }
        for (pred, ctor) in [
            ("equals", Component::Equals as fn(String) -> Component),
            ("disjoint", Component::Disjoint as fn(String) -> Component),
            ("lessThan", Component::LessThan as fn(String) -> Component),
            (
                "lessThanOrEquals",
                Component::LessThanOrEquals as fn(String) -> Component,
            ),
        ] {
            for o in g.objects(node, &sh(pred)) {
                if let Term::NamedNode(n) = o {
                    c.push(ctor(n.as_str().to_string()));
                }
            }
        }
        for o in g.objects(node, &sh("hasValue")) {
            c.push(Component::HasValue(o));
        }
        for o in g.objects(node, &sh("in")) {
            let members = g.list(&o);
            c.push(Component::In(members));
        }
        if matches!(g.object(node, &sh("closed")), Some(Term::Literal(l)) if l.value() == "true") {
            let ignored = match g.object(node, &sh("ignoredProperties")) {
                Some(list) => g.list(&list),
                None => Vec::new(),
            };
            shape.components.push(Component::Closed { ignored });
        }

        // Shape-referencing components (recursive).
        let nots = g.objects(node, &sh("not"));
        for o in nots {
            let id = self.shape_id(g, &o);
            shape.components.push(Component::Not(id));
        }
        for (pred, ctor) in [
            ("and", Component::And as fn(Vec<usize>) -> Component),
            ("or", Component::Or as fn(Vec<usize>) -> Component),
            ("xone", Component::Xone as fn(Vec<usize>) -> Component),
        ] {
            for o in g.objects(node, &sh(pred)) {
                let ids: Vec<usize> = g.list(&o).iter().map(|s| self.shape_id(g, s)).collect();
                shape.components.push(ctor(ids));
            }
        }
        for o in g.objects(node, &sh("node")) {
            let id = self.shape_id(g, &o);
            shape.components.push(Component::Node(id));
        }
        for o in g.objects(node, &sh("property")) {
            let id = self.shape_id(g, &o);
            shape.components.push(Component::Property(id));
            shape.property_children.push(id);
        }
        for o in g.objects(node, &sh("qualifiedValueShape")) {
            let id = self.shape_id(g, &o);
            let num = |p: &str| -> Option<u64> {
                match g.object(node, &sh(p)) {
                    Some(Term::Literal(l)) => l.value().parse().ok(),
                    _ => None,
                }
            };
            shape.components.push(Component::Qualified {
                shape: id,
                min: num("qualifiedMinCount"),
                max: num("qualifiedMaxCount"),
                disjoint: matches!(
                    g.object(node, &sh("qualifiedValueShapesDisjoint")),
                    Some(Term::Literal(l)) if l.value() == "true"
                ),
                siblings: Vec::new(),
            });
        }

        // sh:sparql — SPARQL-based constraints (SHACL §5.2). The object is a node
        // carrying sh:select (required), sh:prefixes, sh:message, sh:deactivated.
        // On a property shape, $PATH in the query is pre-bound to the path.
        let shape_path = shape.path.clone();
        for sp in g.objects(node, &sh("sparql")) {
            if let Some(idx) = self.parse_sparql_constraint(g, &sp, shape_path.as_ref()) {
                shape.components.push(Component::Sparql(idx));
            }
        }

        // [OPUS-4.8] SHACL §6: activate each declared constraint component whose
        // MANDATORY parameter predicates the shape all uses. The bound parameter
        // values (one object per parameter; `None` for an absent optional one)
        // are captured now and pre-bound as `$paramName` at evaluation time.
        for (cidx, comp) in self.components.iter().enumerate() {
            let mut args: Vec<Option<Term>> = Vec::with_capacity(comp.parameters.len());
            let mut activates = true;
            for p in &comp.parameters {
                let value = g.object(node, &p.predicate);
                if value.is_none() && !p.optional {
                    activates = false;
                    break;
                }
                args.push(value);
            }
            if activates && !comp.parameters.is_empty() {
                shape.components.push(Component::CustomSparql {
                    component: cidx,
                    args,
                });
            }
        }

        shape
    }

    /// Parses one `sh:sparql` constraint node into a [`SparqlConstraint`],
    /// interning it into [`Self::sparql`] and returning its index. `None` when the
    /// node has no `sh:select` literal (an ill-formed constraint — skipped).
    /// `shape_path` is the enclosing (property) shape's path, used to pre-bind the
    /// `$PATH` query variable (SHACL §5.2.1) when present.
    fn parse_sparql_constraint(
        &mut self,
        g: &GraphView,
        node: &Term,
        shape_path: Option<&Path>,
    ) -> Option<usize> {
        let raw_select = match g.object(node, &sh("select")) {
            Some(Term::Literal(l)) => l.value().to_string(),
            _ => return None,
        };
        // $PATH pre-binding: substitute the property path's SPARQL property-path
        // form for $PATH / ?PATH in the query text (a property-shape feature).
        let select = match shape_path.and_then(Path::to_sparql_property_path) {
            Some(pp) => substitute_path_var(&raw_select, &pp),
            None => raw_select,
        };
        let prefixes = self.collect_prefixes(g, node);
        let message = match g.object(node, &sh("message")) {
            Some(Term::Literal(l)) => Some(l.value().to_string()),
            _ => None,
        };
        let deactivated = matches!(
            g.object(node, &sh("deactivated")),
            Some(Term::Literal(l)) if l.value() == "true"
        );
        let mut constraint = SparqlConstraint {
            select,
            prefixes,
            message,
            deactivated,
            prepared: None,
        };
        constraint.prepared = crate::sparql::PreparedSparql::build(&constraint);
        let idx = self.sparql.len();
        self.sparql.push(constraint);
        Some(idx)
    }

    /// Assembles SPARQL `PREFIX` declarations from a constraint node's
    /// `sh:prefixes` (SHACL §5.2.1): each `sh:prefixes` object is a prefix
    /// declarations resource that, directly or via `owl:imports`, declares
    /// `sh:declare` nodes carrying `sh:prefix` (the short name) and `sh:namespace`
    /// (the IRI). owl:imports chasing is followed one level (cycle-guarded).
    fn collect_prefixes(&self, g: &GraphView, node: &Term) -> String {
        collect_prefixes_from(g, &g.objects(node, &sh("prefixes")))
    }
}

/// [OPUS-4.8] (sq-d1dw, `shacl-af`) Crate-internal accessor to the shared
/// `sh:prefixes` collector for the SHACL-AF rules module (`sh:SPARQLRule`'s
/// `sh:construct` reuses the same `sh:declare`/`owl:imports` prefix machinery as
/// `sh:sparql`). Gated to the feature so it adds nothing when SHACL-AF is off.
#[cfg(feature = "shacl-af")]
pub(crate) fn collect_prefixes_for(g: &GraphView, prefix_roots: &[Term]) -> String {
    collect_prefixes_from(g, prefix_roots)
}

/// [OPUS-4.8] Assembles SPARQL `PREFIX` declarations from a set of `sh:prefixes`
/// declaration resources (SHACL §5.2.1 / §6.3): each root, directly or via
/// `owl:imports`, declares `sh:declare` nodes carrying `sh:prefix` (short name)
/// and `sh:namespace` (IRI). owl:imports chasing is followed transitively
/// (cycle-guarded). Shared by the `sh:sparql` and constraint-component paths.
fn collect_prefixes_from(g: &GraphView, prefix_roots: &[Term]) -> String {
    const OWL_IMPORTS: &str = "http://www.w3.org/2002/07/owl#imports";
    let mut out = String::new();
    let mut seen_decls: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
    let mut roots: Vec<Term> = prefix_roots.to_vec();
    let mut visited_roots: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
    let mut i = 0;
    while i < roots.len() {
        let root = roots[i].clone();
        i += 1;
        if !visited_roots.insert(root.clone()) {
            continue;
        }
        // Follow owl:imports transitively (cycle-guarded by visited_roots).
        for imp in g.objects(&root, OWL_IMPORTS) {
            roots.push(imp);
        }
        for decl in g.objects(&root, &sh("declare")) {
            if !seen_decls.insert(decl.clone()) {
                continue;
            }
            let prefix = match g.object(&decl, &sh("prefix")) {
                Some(Term::Literal(l)) => l.value().to_string(),
                _ => continue,
            };
            let ns = match g.object(&decl, &sh("namespace")) {
                Some(Term::Literal(l)) => l.value().to_string(),
                Some(Term::NamedNode(n)) => n.as_str().to_string(),
                _ => continue,
            };
            out.push_str(&format!("PREFIX {prefix}: <{ns}>\n"));
        }
    }
    out
}

fn iri(s: &str) -> Term {
    Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
}

/// [OPUS-4.8] SHACL §6.2: discover the `sh:ConstraintComponent` declarations in
/// the shapes graph and compile their parameters + validators. A component is
/// kept only if it has at least one parameter and at least one usable validator
/// (a generic / node / property validator with a parsable `sh:ask`/`sh:select`).
fn discover_components(g: &GraphView) -> Vec<ComponentDef> {
    let mut out = Vec::new();
    // SHACL §6.2: a component node is a SHACL instance of sh:ConstraintComponent —
    // i.e. typed sh:ConstraintComponent OR any rdfs:subClassOf-descendant of it
    // (the W3C suite declares `ex:ConstraintComponent rdfs:subClassOf
    // sh:ConstraintComponent` and types components with that subclass).
    for node in g.instances_of(&iri(&sh("ConstraintComponent"))) {
        let parameters = parse_component_parameters(g, &node);
        if parameters.is_empty() {
            continue; // a parameter-less component cannot activate by predicate use
        }
        // Validators parse under the component's own `sh:prefixes` (SHACL §6.3),
        // reusing the `sh:declare`/`owl:imports` chasing of the `sh:sparql` path.
        let prefixes = collect_prefixes_from(g, &g.objects(&node, &sh("prefixes")));
        let validator = parse_validator(g, &node, &sh("validator"), &prefixes);
        let node_validator = parse_validator(g, &node, &sh("nodeValidator"), &prefixes);
        let property_validator = parse_validator(g, &node, &sh("propertyValidator"), &prefixes);
        if validator.is_none() && node_validator.is_none() && property_validator.is_none() {
            continue; // no usable validator — skip (lenient)
        }
        out.push(ComponentDef {
            node,
            parameters,
            validator,
            node_validator,
            property_validator,
        });
    }
    out
}

/// Parses a component's `sh:parameter` list (SHACL §6.2.1). Each parameter node
/// carries `sh:path` (its predicate) and optionally `sh:optional`/`sh:name`. A
/// parameter with no IRI `sh:path` is skipped (a component cannot key on it).
fn parse_component_parameters(g: &GraphView, node: &Term) -> Vec<ComponentParameter> {
    let mut params = Vec::new();
    for p in g.objects(node, &sh("parameter")) {
        let predicate = match g.object(&p, &sh("path")) {
            Some(Term::NamedNode(n)) => n.as_str().to_string(),
            _ => continue,
        };
        // The pre-bound variable name: sh:name if a literal, else the predicate's
        // local name (after the last '#' or '/').
        let var = match g.object(&p, &sh("name")) {
            Some(Term::Literal(l)) => l.value().to_string(),
            _ => local_name(&predicate),
        };
        let optional = matches!(
            g.object(&p, &sh("optional")),
            Some(Term::Literal(l)) if l.value() == "true"
        );
        params.push(ComponentParameter {
            predicate,
            var,
            optional,
        });
    }
    params
}

/// The local name of an IRI: the substring after the last `#` or `/`.
fn local_name(iri: &str) -> String {
    iri.rsplit(['#', '/']).next().unwrap_or(iri).to_string()
}

/// Parses one validator (`pred` = `sh:validator` / `sh:nodeValidator` /
/// `sh:propertyValidator`) of a component: its `sh:ask` (ASK validator) or
/// `sh:select` (SELECT validator), with `prefixes` prepended and an optional
/// `sh:message`. Returns `None` if the validator carries neither query or the
/// query is unparsable / of the wrong form (ill-formed → skipped).
fn parse_validator(
    g: &GraphView,
    node: &Term,
    pred: &str,
    prefixes: &str,
) -> Option<PreparedComponentValidator> {
    let v = g.object(node, pred)?;
    // sh:ask takes precedence; fall back to sh:select.
    let (text, is_ask) = match g.object(&v, &sh("ask")) {
        Some(Term::Literal(l)) => (l.value().to_string(), true),
        _ => match g.object(&v, &sh("select")) {
            Some(Term::Literal(l)) => (l.value().to_string(), false),
            _ => return None,
        },
    };
    let full = format!("{prefixes}\n{text}");
    let prepared = crate::sparql::PreparedValidator::build(&full, is_ask)?;
    let message = match g.object(&v, &sh("message")) {
        Some(Term::Literal(l)) => Some(l.value().to_string()),
        _ => None,
    };
    Some(PreparedComponentValidator { prepared, message })
}

/// Substitutes the `$PATH` / `?PATH` query variable (a SHACL property-shape
/// SPARQL pre-binding) with the SPARQL property-path expression `pp`, replacing
/// only WHOLE variable tokens (so `$PATHWAY` is left alone). SHACL §5.2.1.
fn substitute_path_var(select: &str, pp: &str) -> String {
    let mut out = String::with_capacity(select.len());
    let mut rest = select;
    while let Some(pos) = rest.find(['$', '?']) {
        out.push_str(&rest[..pos]);
        let tail = &rest[pos..]; // starts with $ or ?
        let body = &tail[1..];
        // Whole-token match of "PATH" delimited by a non-identifier char.
        let is_path_var = body.strip_prefix("PATH").is_some_and(|after| {
            after
                .chars()
                .next()
                .map(|ch| !(ch.is_ascii_alphanumeric() || ch == '_'))
                .unwrap_or(true)
        });
        if is_path_var {
            out.push_str(pp);
            rest = &body[4..]; // past "PATH"
        } else {
            // Not the PATH var: keep the sigil and continue past it.
            out.push_str(&tail[..1]);
            rest = body;
        }
    }
    out.push_str(rest);
    out
}
