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
}

impl ShapesModel {
    pub fn parse(shapes_graph: &Graph) -> ShapesModel {
        let g = GraphView::new(shapes_graph);
        let mut m = ShapesModel {
            shapes: Vec::new(),
            by_node: FxHashMap::default(),
            targeted: Vec::new(),
        };

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

        // Validation entry points: shapes with targets.
        m.targeted = (0..m.shapes.len())
            .filter(|&i| !m.shapes[i].targets.is_empty())
            .collect();
        m
    }

    pub fn by_node(&self, node: &Term) -> Option<usize> {
        self.by_node.get(node).copied()
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

        shape
    }
}

fn iri(s: &str) -> Term {
    Term::NamedNode(oxrdf::NamedNode::new_unchecked(s))
}
