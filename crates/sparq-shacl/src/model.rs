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
    /// [OPUS-4.8] (sq-sx15d) `sh:class` with a SHACL-list object
    /// (`sh:class ( ex:A ex:B )`, SHACL 1.2): a value node conforms iff it is a
    /// SHACL instance of ANY listed class (`sh:ClassConstraintComponent`).
    /// Mirrors the disjunctive `sh:datatype` / `sh:nodeKind` list spelling; a
    /// single IRI object stays [`Component::Class`].
    ClassIn(Vec<Term>),
    /// `sh:datatype` — the allowed-datatype set (SHACL §4.5.2). A single IRI
    /// object is a singleton set; the SHACL-1.2 disjunctive list form
    /// `sh:datatype ( xsd:string rdf:langString )` is the multi-element set. A
    /// value node conforms iff it is a literal whose (well-formed) datatype is in
    /// the set. [OPUS-4.8] (sq-vg3y) extended from a single IRI to the set form.
    Datatype(Vec<String>),
    /// `sh:nodeKind` — the allowed node-kind set (SHACL §4.6.1). A single IRI is a
    /// singleton set; the SHACL-1.2 disjunctive list form
    /// `sh:nodeKind ( sh:BlankNode sh:IRI )` is the multi-element set. A value
    /// node conforms iff its kind matches ANY listed kind. [OPUS-4.8] (sq-vg3y).
    NodeKind(Vec<String>),
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
    /// [OPUS-4.8] (sq-sx15d) `sh:equals` — the value set of the shape's path must
    /// equal the value set of the comparand. SHACL 1.2 lets the comparand be a
    /// full property [`Path`] (often an RDF-list sequence), not just a predicate
    /// IRI; a bare IRI parses to a trivial [`Path::Predicate`], so the SHACL 1.0
    /// `-001` predicate form stays backward-compatible.
    Equals(Path),
    /// [OPUS-4.8] (sq-sx15d) `sh:disjoint` — the value set of the shape's path
    /// must be disjoint from the comparand's. The comparand is a [`Path`] (SHACL
    /// 1.2 list/path form; a bare IRI is a trivial path).
    Disjoint(Path),
    /// [OPUS-4.8] (sq-sx15d) `sh:lessThan` — every path value must be `<` every
    /// comparand value. The comparand is a [`Path`] (SHACL 1.2).
    LessThan(Path),
    /// [OPUS-4.8] (sq-sx15d) `sh:lessThanOrEquals` — every path value must be
    /// `<=` every comparand value. The comparand is a [`Path`] (SHACL 1.2).
    LessThanOrEquals(Path),
    /// [OPUS-4.8] (sq-sx15d) `sh:subsetOf` (SHACL 1.2) — the value set of the
    /// shape's path must be a SUBSET of the comparand path's value set
    /// (`sh:SubsetOfConstraintComponent`). One result per path value absent from
    /// the comparand set. The comparand is a [`Path`].
    SubsetOf(Path),
    /// [OPUS-4.8] (sq-sx15d) `sh:someValue` (SHACL 1.2) — EXISTENTIAL: at least
    /// one value node must conform to the nested shape
    /// (`sh:SomeValueConstraintComponent`). The index is into
    /// [`ShapesModel::shapes`]. One result on the focus/path when NONE conform.
    SomeValue(usize),
    /// [OPUS-4.8] (sq-sx15d) `sh:singleLine true` (SHACL 1.2) — each string value
    /// must contain no line-break characters (LF/CR/FF/VT)
    /// (`sh:SingleLineConstraintComponent`). `sh:singleLine false` imposes no
    /// constraint (not parsed into a component).
    SingleLine,
    /// [OPUS-4.8] (sq-sx15d) `sh:rootClass` (SHACL 1.2) — each value node must be
    /// the named class or a transitive `rdfs:subClassOf`-descendant of it
    /// (`sh:RootClassConstraintComponent`). Reuses the `sh:class` subclass
    /// closure.
    RootClass(Term),
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
        /// [OPUS-4.8] (sq-vg3y) SHACL-1.2 "close by types" mode: `sh:closed
        /// sh:ByTypes`. When `false` (`sh:closed true`), the allowed predicate set
        /// P is the IRIs reachable from THIS shape via `sh:property/sh:path`. When
        /// `true`, P is recomputed PER value node from its `rdf:type`s via the
        /// `collectProperties` algorithm (SHACL §4.8.1), plus `rdf:type`.
        by_types: bool,
    },
    HasValue(Term),
    In(Vec<Term>),
    /// [OPUS-4.8] (sq-vg3y) `sh:memberShape` — SHACL-1.2 list-member shape
    /// (`sh:MemberShapeConstraintComponent`, SHACL §4.x). Each value node must be a
    /// well-formed SHACL list, and every member of that list must conform to the
    /// referenced shape. The index is into [`ShapesModel::shapes`].
    MemberShape(usize),
    /// [OPUS-4.8] (sq-vg3y) `sh:uniqueMembers true` — value nodes must be SHACL
    /// lists whose members are pairwise distinct (`sh:UniqueMembersConstraintComponent`).
    UniqueMembers,
    /// [OPUS-4.8] (sq-vg3y) `sh:maxListLength` — value nodes must be SHACL lists
    /// with at most N members (`sh:MaxListLengthConstraintComponent`).
    MaxListLength(u64),
    /// [OPUS-4.8] (sq-vg3y) `sh:minListLength` — value nodes must be SHACL lists
    /// with at least N members (`sh:MinListLengthConstraintComponent`).
    MinListLength(u64),
    /// [OPUS-4.8] (sq-vg3y) `sh:uniqueValuesFor` — the values of the listed
    /// properties of a value node must be unique across all target nodes of the
    /// shape (`sh:UniqueValuesForConstraintComponent`, SHACL §4.x). One or more
    /// property IRIs (a single IRI is a singleton; a SHACL list gives a composite
    /// key).
    UniqueValuesFor(Vec<String>),
    /// sh:sparql — a SPARQL-based constraint (SHACL §5.2). The index is into
    /// `ShapesModel::sparql`.
    Sparql(usize),
    /// [OPUS-4.8] A SPARQL-based constraint COMPONENT (SHACL §6) that activated on
    /// this shape because the shape uses the component's parameter predicates.
    /// `component` indexes `ShapesModel::components`; `args` are the bound
    /// parameter values, parallel to the component's `parameters` (one term each
    /// — the first object found for a mandatory parameter; `None` for an absent
    /// optional one). The validator pre-binds each as `$paramName`.
    CustomSparql {
        component: usize,
        args: Vec<Option<Term>>,
        /// [OPUS-4.8] Index into the model's `path_validators` store of a
        /// per-shape validator with the shape's property path substituted for `$PATH`
        /// query variable (SHACL §6.3), present only when the shape is a PROPERTY
        /// shape, the chosen validator references `$PATH`, and the substituted
        /// query re-parses. When set it OVERRIDES the component's shared
        /// (path-free) validator for this occurrence; otherwise the shared
        /// validator is used as-is (node shapes, or `$PATH`-free validators). An
        /// index (not the value) keeps the crate-private validator type off this
        /// public enum.
        path_validator: Option<usize>,
    },
    /// [OPUS-4.8] (sq-mk9n, `shacl-af`) `sh:expression` — the SHACL-AF
    /// *Expression Constraint* (`sh:ExpressionConstraintComponent`): a value node
    /// `v` is violated when the node expression does NOT evaluate to `{ true }`
    /// for `v` as focus. The index is into `ShapesModel::expressions` (the
    /// parsed node expression is held there, off the public `Component` enum).
    #[cfg(feature = "shacl-af")]
    Expression(usize),
    /// [OPUS-4.8] (sq-3w6n, `shacl-af`) `sh:nodeByExpression` — the SHACL-AF
    /// *Node-by-Expression Constraint* (`sh:NodeByExpressionConstraintComponent`):
    /// for each value node `v`, the node expression is evaluated against `v` as
    /// focus to a set of node-shape terms `s`; `v` is violated when it does NOT
    /// conform to some `s`. (Like `sh:node`, but the shape is computed by the
    /// expression rather than fixed.) The index is into
    /// `ShapesModel::expressions` (shared store with `sh:expression`).
    #[cfg(feature = "shacl-af")]
    NodeByExpression(usize),
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

impl PreparedComponentValidator {
    /// Whether the validator query text references the `$PATH` / `?PATH` query
    /// variable — i.e. it must be re-parsed per property shape with the shape's
    /// path substituted (SHACL §6.3). A cheap textual probe (the whole-token
    /// match is done by `substitute_path_var`); a false positive only costs a
    /// re-parse that produces the same query.
    pub fn references_path(&self) -> bool {
        self.raw.contains("PATH")
    }

    /// Re-parses this validator with `$PATH` / `?PATH` substituted by the SPARQL
    /// property-path expression `pp` (SHACL §6.3 pre-binds `$PATH` to the property
    /// shape's path). Returns `None` if the substituted query no longer parses
    /// (ill-formed → the component occurrence is then skipped, lenient).
    pub fn with_path(&self, pp: &str) -> Option<PreparedComponentValidator> {
        let substituted = substitute_path_var(&self.raw, pp);
        let prepared = crate::sparql::PreparedValidator::build(&substituted, self.is_ask)?;
        Some(PreparedComponentValidator {
            prepared,
            message: self.message.clone(),
            raw: substituted,
            is_ask: self.is_ask,
        })
    }
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
    /// The validator's full query text (prefixes already prepended) and whether
    /// it is an `sh:ask`. Retained so a `sh:propertyValidator` can be RE-PARSED
    /// per property shape with the shape's path substituted for the `$PATH`
    /// query variable (SHACL §6.3 pre-binds `$PATH` to the shape's property path,
    /// which — being a property PATH, not a term — cannot go through the VALUES
    /// table the other pre-bindings use, so it is a textual substitution like the
    /// §5.2 `sh:sparql` path). `None` for blank text would never compile.
    pub raw: String,
    pub is_ask: bool,
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
    /// [OPUS-4.8] (sq-wys) Per-shape `sh:propertyValidator`s re-parsed with the
    /// shape's path substituted for the `$PATH` query variable (SHACL §6.3),
    /// referenced by [`Component::CustomSparql`]'s `path_validator` index. Kept
    /// off the public `Component` enum so the (crate-private) prepared-validator
    /// type is not exposed (same pattern as `sparql` / `expressions`).
    pub(crate) path_validators: Vec<PreparedComponentValidator>,
    /// [OPUS-4.8] (sq-vg3y) Precomputed `sh:closed sh:ByTypes` property closures
    /// (SHACL-1.2 §4.8.1 `collectProperties`): for each shapes-graph node, the
    /// IRI properties reachable via `sh:property/sh:path` transitively through
    /// `rdfs:subClassOf` / inbound `sh:targetClass` / `sh:node`. This
    /// shapes-graph traversal is data-independent, so it is resolved ONCE here
    /// (only when at least one ByTypes-closed shape exists) and unioned per value
    /// node at eval time. Empty when no shape uses `sh:closed sh:ByTypes`.
    pub(crate) by_types_closures: FxHashMap<Term, Vec<String>>,
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
            path_validators: Vec::new(),
            by_types_closures: FxHashMap::default(),
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

        // [OPUS-4.8] (sq-vg3y) Resolve the data-independent `sh:closed sh:ByTypes`
        // property closures once (only if any shape uses that mode).
        if m.shapes.iter().any(|s| {
            s.components
                .iter()
                .any(|c| matches!(c, Component::Closed { by_types: true, .. }))
        }) {
            m.by_types_closures = compute_by_types_closures(&g);
        }

        // Validation entry points: shapes with targets.
        m.targeted = (0..m.shapes.len())
            .filter(|&i| !m.shapes[i].targets.is_empty())
            .collect();
        m
    }

    /// [OPUS-4.8] (sq-vg3y) The precomputed `sh:closed sh:ByTypes` property closure
    /// for a shapes-graph node, or `None` if the node pulls in no properties (e.g.
    /// a data class with no shapes-graph footprint). See [`Self::by_types_closures`].
    pub(crate) fn by_types_closure(&self, node: &Term) -> Option<&Vec<String>> {
        self.by_types_closures.get(node)
    }

    /// [OPUS-4.8] (sq-mk9n / sq-3w6n, `shacl-af`) Parses the two SHACL-AF
    /// node-expression constraints — `sh:expression`
    /// (→ [`Component::Expression`]) and `sh:nodeByExpression`
    /// (→ [`Component::NodeByExpression`]) — on every shape. Inline filter shapes
    /// used by the expressions, and the node shapes a `sh:nodeByExpression`
    /// expression names as a constant, are registered first (so they are
    /// conformance-checkable), then the expressions are parsed and attached.
    #[cfg(feature = "shacl-af")]
    fn parse_expression_constraints(&mut self, shapes_graph: &Graph) {
        let g = GraphView::new(shapes_graph);
        // Collect (shape_id, expression_term, is_node_by_expr) for every shape
        // carrying sh:expression or sh:nodeByExpression.
        let mut pending: Vec<(usize, Term, bool)> = Vec::new();
        for sid in 0..self.shapes.len() {
            let node = self.shapes[sid].node.clone();
            for expr in g.objects(&node, &sh("expression")) {
                pending.push((sid, expr, false));
            }
            for expr in g.objects(&node, &sh("nodeByExpression")) {
                pending.push((sid, expr, true));
            }
        }
        // Register any inline filter shapes the expressions reference (best-effort)
        // so `sh:filterShape` / function filter shapes resolve at eval time.
        for (_, expr, is_nbe) in &pending {
            self.register_expression_shapes(shapes_graph, expr, 0);
            // A `sh:nodeByExpression` whose expression names a node shape as a
            // constant IRI must have that shape parsed so conformance can be
            // checked at eval time (it may not be a target-bearing root).
            if *is_nbe && matches!(expr, Term::NamedNode(_)) {
                self.ensure_shape(shapes_graph, expr);
            }
        }
        // Parse each expression (immutable borrow) and attach the component.
        let parsed: Vec<(usize, crate::rules::NodeExpr, bool)> = pending
            .into_iter()
            .filter_map(|(sid, expr, is_nbe)| {
                crate::rules::parse_node_expr(&g, self, &expr).map(|ne| (sid, ne, is_nbe))
            })
            .collect();
        for (sid, expr, is_nbe) in parsed {
            let idx = self.expressions.len();
            self.expressions.push(expr);
            self.shapes[sid].components.push(if is_nbe {
                Component::NodeByExpression(idx)
            } else {
                Component::Expression(idx)
            });
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
        // [OPUS-4.8] (sq-sx15d) `sh:class` accepts a single class IRI/blank node or
        // the SHACL-1.2 disjunctive SHACL-list form `( ex:A ex:B )` — a value node
        // conforms iff it is an instance of ANY listed class. A blank-node object
        // that is a well-formed SHACL list (≥1 member) is the disjunctive form;
        // any other object is a single class. Mirrors the `sh:datatype` /
        // `sh:nodeKind` disjunctive handling above.
        for o in g.objects(node, &sh("class")) {
            match &o {
                Term::BlankNode(_) => {
                    let members = g.list(&o);
                    if members.is_empty() {
                        c.push(Component::Class(o));
                    } else {
                        c.push(Component::ClassIn(members));
                    }
                }
                _ => c.push(Component::Class(o)),
            }
        }
        // [OPUS-4.8] (sq-vg3y) `sh:datatype` / `sh:nodeKind` accept either a single
        // IRI or the SHACL-1.2 disjunctive SHACL-list form (`( a b )`). `iri_set`
        // returns the IRI set for either spelling; an empty set (e.g. a literal
        // object, or an ill-formed list) contributes no constraint.
        for o in g.objects(node, &sh("datatype")) {
            let dts = iri_set(g, &o);
            if !dts.is_empty() {
                c.push(Component::Datatype(dts));
            }
        }
        for o in g.objects(node, &sh("nodeKind")) {
            let kinds = iri_set(g, &o);
            if !kinds.is_empty() {
                c.push(Component::NodeKind(kinds));
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
        // [OPUS-4.8] (sq-sx15d) `sh:equals` / `sh:disjoint` / `sh:lessThan` /
        // `sh:lessThanOrEquals` / `sh:subsetOf` carry a comparand SHACL property
        // PATH in SHACL 1.2 (often an RDF-list sequence), not just a predicate
        // IRI. Parse the comparand with the same `Path` parser used for `sh:path`
        // (a bare NamedNode → `Path::Predicate`, so the SHACL-1.0 predicate forms
        // stay backward-compatible); an ill-formed comparand is dropped (lenient).
        for (pred, ctor) in [
            ("equals", Component::Equals as fn(Path) -> Component),
            ("disjoint", Component::Disjoint as fn(Path) -> Component),
            ("lessThan", Component::LessThan as fn(Path) -> Component),
            (
                "lessThanOrEquals",
                Component::LessThanOrEquals as fn(Path) -> Component,
            ),
            ("subsetOf", Component::SubsetOf as fn(Path) -> Component),
        ] {
            for o in g.objects(node, &sh(pred)) {
                if let Ok(path) = Path::parse(g, &o) {
                    c.push(ctor(path));
                }
            }
        }
        // [OPUS-4.8] (sq-sx15d) `sh:rootClass` (SHACL 1.2): each value node must be
        // the named class or a transitive `rdfs:subClassOf`-descendant of it. The
        // object is a single class term.
        for o in g.objects(node, &sh("rootClass")) {
            c.push(Component::RootClass(o));
        }
        // [OPUS-4.8] (sq-sx15d) `sh:singleLine true` (SHACL 1.2): string values must
        // contain no line-break characters. `sh:singleLine false` (or any non-true
        // object) imposes no constraint.
        if matches!(g.object(node, &sh("singleLine")), Some(Term::Literal(l)) if l.value() == "true")
        {
            c.push(Component::SingleLine);
        }
        for o in g.objects(node, &sh("hasValue")) {
            c.push(Component::HasValue(o));
        }
        for o in g.objects(node, &sh("in")) {
            let members = g.list(&o);
            c.push(Component::In(members));
        }
        // [OPUS-4.8] (sq-vg3y) `sh:maxListLength` / `sh:minListLength` (SHACL-1.2):
        // value nodes must be SHACL lists with at most / at least N members.
        for (pred, ctor) in [
            (
                "maxListLength",
                Component::MaxListLength as fn(u64) -> Component,
            ),
            (
                "minListLength",
                Component::MinListLength as fn(u64) -> Component,
            ),
        ] {
            for o in g.objects(node, &sh(pred)) {
                if let Term::Literal(l) = &o {
                    if let Ok(n) = l.value().parse::<u64>() {
                        c.push(ctor(n));
                    }
                }
            }
        }
        // [OPUS-4.8] (sq-vg3y) `sh:uniqueMembers true` (SHACL-1.2): SHACL-list value
        // nodes must have pairwise-distinct members.
        if matches!(g.object(node, &sh("uniqueMembers")), Some(Term::Literal(l)) if l.value() == "true")
        {
            c.push(Component::UniqueMembers);
        }
        // [OPUS-4.8] (sq-vg3y) `sh:uniqueValuesFor` (SHACL-1.2): one property IRI or
        // a SHACL list of property IRIs forming a composite uniqueness key.
        for o in g.objects(node, &sh("uniqueValuesFor")) {
            let props = iri_set(g, &o);
            if !props.is_empty() {
                c.push(Component::UniqueValuesFor(props));
            }
        }
        // `sh:closed`: SHACL-1.0 boolean (`true`) or SHACL-1.2 `sh:ByTypes`
        // (close-by-types). Any other object (or `false`) is not a closing form.
        // [OPUS-4.8] (sq-vg3y) added the `sh:ByTypes` spelling.
        match g.object(node, &sh("closed")) {
            Some(Term::Literal(l)) if l.value() == "true" => {
                shape.components.push(Component::Closed {
                    ignored: closed_ignored(g, node),
                    by_types: false,
                });
            }
            Some(Term::NamedNode(n)) if n.as_str() == sh("ByTypes") => {
                shape.components.push(Component::Closed {
                    ignored: closed_ignored(g, node),
                    by_types: true,
                });
            }
            _ => {}
        }
        // [OPUS-4.8] (sq-vg3y) `sh:memberShape` (SHACL-1.2): each value node must be
        // a well-formed SHACL list whose members all conform to the referenced
        // shape. Recursive (the member shape is parsed/interned like sh:node).
        for o in g.objects(node, &sh("memberShape")) {
            let id = self.shape_id(g, &o);
            shape.components.push(Component::MemberShape(id));
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
        // [OPUS-4.8] (sq-sx15d) `sh:someValue` (SHACL 1.2): EXISTENTIAL — at least
        // one value node must conform to the referenced (nested) shape. Parsed/
        // interned recursively like `sh:node`; the quantifier is inverted at eval
        // time (a violation iff NO value conforms).
        for o in g.objects(node, &sh("someValue")) {
            let id = self.shape_id(g, &o);
            shape.components.push(Component::SomeValue(id));
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
        // The shape's SPARQL property-path form, used for `$PATH` pre-binding on
        // property-shape component activations (computed once).
        let path_pp = shape_path.as_ref().and_then(Path::to_sparql_property_path);
        // Collect activations first: the `self.components` borrow below is
        // immutable, so the per-shape `$PATH` re-parse (which pushes into
        // `self.path_validators`) is deferred to after the loop.
        let mut activations: Vec<(usize, Vec<Option<Term>>, Option<PreparedComponentValidator>)> =
            Vec::new();
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
                // [OPUS-4.8] On a property shape, pre-bind `$PATH` (SHACL §6.3):
                // re-parse the chosen validator with the shape's property-path
                // expression substituted for the `$PATH` query variable. `$PATH`
                // is a property PATH (not a term), so — like the §5.2 `sh:sparql`
                // path — it is a textual substitution rather than a VALUES row.
                let path_validator = path_pp.as_deref().and_then(|pp| {
                    comp.validator_for(true)
                        .filter(|v| v.references_path())
                        .and_then(|v| v.with_path(pp))
                });
                activations.push((cidx, args, path_validator));
            }
        }
        for (cidx, args, path_validator) in activations {
            let path_validator = path_validator.map(|v| {
                let idx = self.path_validators.len();
                self.path_validators.push(v);
                idx
            });
            shape.components.push(Component::CustomSparql {
                component: cidx,
                args,
                path_validator,
            });
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

/// [OPUS-4.8] (sq-vg3y) The set of IRIs an object denotes when a constraint
/// parameter accepts "an IRI or a SHACL list of IRIs" (`sh:datatype` /
/// `sh:nodeKind` / `sh:uniqueValuesFor`, SHACL-1.2). A single IRI → singleton; a
/// SHACL-list head → its IRI members (non-IRI members dropped). Anything else →
/// empty (no constraint contributed).
fn iri_set(g: &GraphView, o: &Term) -> Vec<String> {
    match o {
        Term::NamedNode(n) => vec![n.as_str().to_string()],
        Term::BlankNode(_) => g
            .list(o)
            .into_iter()
            .filter_map(|m| match m {
                Term::NamedNode(n) => Some(n.as_str().to_string()),
                _ => None,
            })
            .collect(),
        _ => Vec::new(),
    }
}

/// The `sh:ignoredProperties` SHACL list of a (closed) shape node, or empty.
fn closed_ignored(g: &GraphView, node: &Term) -> Vec<Term> {
    match g.object(node, &sh("ignoredProperties")) {
        Some(list) => g.list(&list),
        None => Vec::new(),
    }
}

/// [OPUS-4.8] (sq-vg3y) Precompute the `sh:closed sh:ByTypes` `collectProperties`
/// closure (SHACL-1.2 §4.8.1) for every node mentioned in the shapes graph. For a
/// starting node S the closure is the union of the IRI properties reachable via
/// `sh:property/sh:path` from S and, recursively, from S's `rdfs:subClassOf`
/// objects, the shapes that target S via `sh:targetClass`, and the node shapes S
/// references via `sh:node`. The traversal is cycle-guarded (each node expanded at
/// most once per starting node) and reads ONLY the shapes graph (the per-value
/// `rdf:type` step happens at eval time). Nodes whose closure is empty are
/// omitted, so a `get` miss means "no properties".
fn compute_by_types_closures(g: &GraphView) -> FxHashMap<Term, Vec<String>> {
    // Candidate starting nodes: every subject and object in the shapes graph (a
    // data class T is looked up here too, so we must cover object positions).
    let mut nodes: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
    for [s, _, o] in g.triples(None, None, None) {
        if !matches!(s, Term::Literal(_)) {
            nodes.insert(s);
        }
        if !matches!(o, Term::Literal(_)) {
            nodes.insert(o);
        }
    }
    let mut out: FxHashMap<Term, Vec<String>> = FxHashMap::default();
    for start in nodes {
        let mut props: rustc_hash::FxHashSet<String> = rustc_hash::FxHashSet::default();
        let mut visited: rustc_hash::FxHashSet<Term> = rustc_hash::FxHashSet::default();
        let mut stack = vec![start.clone()];
        while let Some(s) = stack.pop() {
            if !visited.insert(s.clone()) {
                continue;
            }
            collect_properties(g, &s, &mut props, &mut stack);
        }
        if !props.is_empty() {
            out.insert(start, props.into_iter().collect());
        }
    }
    out
}

/// One step of the `collectProperties` algorithm (SHACL-1.2 §4.8.1): add the IRI
/// properties reachable from `s` via `sh:property/sh:path`, and push the nodes the
/// recursion continues into (`rdfs:subClassOf` objects, inbound `sh:targetClass`
/// subjects, `sh:node` objects) onto `stack`. Cycle-avoidance is the caller's
/// `visited` set. Reads only the shapes graph.
fn collect_properties(
    g: &GraphView,
    s: &Term,
    props: &mut rustc_hash::FxHashSet<String>,
    stack: &mut Vec<Term>,
) {
    const RDFS_SUBCLASS_OF: &str = "http://www.w3.org/2000/01/rdf-schema#subClassOf";
    // IRI properties reached via sh:property/sh:path.
    for ps in g.objects(s, &sh("property")) {
        if let Some(Term::NamedNode(n)) = g.object(&ps, &sh("path")) {
            props.insert(n.as_str().to_string());
        }
    }
    // rdfs:subClassOf objects (superclasses).
    for sup in g.objects(s, RDFS_SUBCLASS_OF) {
        stack.push(sup);
    }
    // Shapes that target s via sh:targetClass.
    for sub in g.subjects(&sh("targetClass"), s) {
        stack.push(sub);
    }
    // Node shapes referenced via sh:node.
    for n in g.objects(s, &sh("node")) {
        stack.push(n);
    }
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
        // [OPUS-4.8] The pre-bound variable name is the LOCAL NAME of the
        // parameter's `sh:path` IRI (SHACL §6.2.1: "the values of these parameters
        // [...] pre-bound [...] using the local name of the IRI of sh:path"), NOT
        // `sh:name` — which is only a human-readable display label. The W3C
        // `propertyValidator-select-001` test makes this load-bearing: its
        // parameter is `sh:path ex:lang ; sh:name "language"` yet the validator
        // query references `$lang` (the path local name), so binding `$language`
        // would leave `$lang` unbound and the constraint would never fire.
        let var = local_name(&predicate);
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
    Some(PreparedComponentValidator {
        prepared,
        message,
        raw: full,
        is_ask,
    })
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
