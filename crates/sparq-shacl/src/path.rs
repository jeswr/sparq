//! SHACL property paths: the parsed form, evaluation by direct graph walks, and
//! serialisation back to a Turtle path expression for validation reports.

use crate::view::{dedup, GraphView, RDF_FIRST};
use oxrdf::Term;
use rustc_hash::FxHashSet;

const SH: &str = "http://www.w3.org/ns/shacl#";

/// A SHACL property path (sh:path), structurally faithful to the shapes graph so
/// reports can serialise the same expression back out.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Path {
    /// A predicate path: the IRI of the property.
    Predicate(String),
    Inverse(Box<Path>),
    Sequence(Vec<Path>),
    Alternative(Vec<Path>),
    ZeroOrMore(Box<Path>),
    OneOrMore(Box<Path>),
    ZeroOrOne(Box<Path>),
}

impl Path {
    /// Parses a path expression rooted at `node` in a shapes graph.
    pub fn parse(g: &GraphView, node: &Term) -> Result<Path, String> {
        Self::parse_guarded(g, node, &mut FxHashSet::default())
    }

    fn parse_guarded(
        g: &GraphView,
        node: &Term,
        seen: &mut FxHashSet<Term>,
    ) -> Result<Path, String> {
        match node {
            Term::NamedNode(n) => Ok(Path::Predicate(n.as_str().to_string())),
            Term::BlankNode(_) => {
                // Chain-based cycle guard: a node may legitimately occur twice as a
                // SIBLING (e.g. the sequence `( _:inv _:inv )`); only re-entry on the
                // CURRENT parse chain is a cycle, so the node is removed again after
                // its subtree parses (see the `seen.remove` below).
                if !seen.insert(node.clone()) {
                    return Err("cyclic path expression".into());
                }
                let parsed = Self::parse_blank(g, node, seen);
                seen.remove(node);
                parsed
            }
            Term::Literal(_) => Err("literal in path expression".into()),
            #[allow(unreachable_patterns)]
            _ => Err("unsupported path term".into()),
        }
    }

    fn parse_blank(g: &GraphView, node: &Term, seen: &mut FxHashSet<Term>) -> Result<Path, String> {
        // An rdf:list head is a sequence path.
        if g.object(node, RDF_FIRST).is_some() {
            let items = g.list(node);
            if items.len() < 2 {
                return Err("sequence path with fewer than two members".into());
            }
            let parts = items
                .iter()
                .map(|i| Self::parse_guarded(g, i, seen))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Path::Sequence(parts));
        }
        if let Some(p) = g.object(node, &format!("{SH}inversePath")) {
            return Ok(Path::Inverse(Box::new(Self::parse_guarded(g, &p, seen)?)));
        }
        if let Some(p) = g.object(node, &format!("{SH}alternativePath")) {
            let items = g.list(&p);
            if items.len() < 2 {
                return Err("alternative path with fewer than two members".into());
            }
            let parts = items
                .iter()
                .map(|i| Self::parse_guarded(g, i, seen))
                .collect::<Result<Vec<_>, _>>()?;
            return Ok(Path::Alternative(parts));
        }
        if let Some(p) = g.object(node, &format!("{SH}zeroOrMorePath")) {
            return Ok(Path::ZeroOrMore(Box::new(Self::parse_guarded(
                g, &p, seen,
            )?)));
        }
        if let Some(p) = g.object(node, &format!("{SH}oneOrMorePath")) {
            return Ok(Path::OneOrMore(Box::new(Self::parse_guarded(g, &p, seen)?)));
        }
        if let Some(p) = g.object(node, &format!("{SH}zeroOrOnePath")) {
            return Ok(Path::ZeroOrOne(Box::new(Self::parse_guarded(g, &p, seen)?)));
        }
        Err("ill-formed path blank node".into())
    }

    /// The value nodes reachable from `start` along this path (a set, in
    /// discovery order — SHACL value nodes are distinct).
    pub fn values(&self, g: &GraphView, start: &Term) -> Vec<Term> {
        self.step(g, start, true)
    }

    /// One application of the path from `node`; `forward` flips under inversion.
    fn step(&self, g: &GraphView, node: &Term, forward: bool) -> Vec<Term> {
        match self {
            Path::Predicate(p) => {
                if forward {
                    g.objects(node, p)
                } else {
                    g.subjects(p, node)
                }
            }
            Path::Inverse(inner) => inner.step(g, node, !forward),
            Path::Sequence(parts) => {
                let mut nodes = vec![node.clone()];
                let order: Vec<&Path> = if forward {
                    parts.iter().collect()
                } else {
                    parts.iter().rev().collect()
                };
                for p in order {
                    nodes = dedup(nodes.iter().flat_map(|n| p.step(g, n, forward)));
                    if nodes.is_empty() {
                        break;
                    }
                }
                nodes
            }
            Path::Alternative(parts) => dedup(parts.iter().flat_map(|p| p.step(g, node, forward))),
            Path::ZeroOrOne(inner) => {
                dedup(std::iter::once(node.clone()).chain(inner.step(g, node, forward)))
            }
            Path::ZeroOrMore(inner) => closure(g, node, inner, forward, true),
            Path::OneOrMore(inner) => closure(g, node, inner, forward, false),
        }
    }

    /// Serialises the path as a Turtle path expression usable as the object of
    /// `sh:resultPath` (predicate IRIs, `( … )` sequences, `[ sh:inversePath … ]`
    /// etc. for the other forms).
    pub fn to_turtle(&self) -> String {
        match self {
            Path::Predicate(p) => format!("<{p}>"),
            Path::Inverse(p) => format!("[ <{SH}inversePath> {} ]", p.to_turtle()),
            Path::Sequence(ps) => {
                let inner: Vec<String> = ps.iter().map(|p| p.to_turtle()).collect();
                format!("( {} )", inner.join(" "))
            }
            Path::Alternative(ps) => {
                let inner: Vec<String> = ps.iter().map(|p| p.to_turtle()).collect();
                format!("[ <{SH}alternativePath> ( {} ) ]", inner.join(" "))
            }
            Path::ZeroOrMore(p) => format!("[ <{SH}zeroOrMorePath> {} ]", p.to_turtle()),
            Path::OneOrMore(p) => format!("[ <{SH}oneOrMorePath> {} ]", p.to_turtle()),
            Path::ZeroOrOne(p) => format!("[ <{SH}zeroOrOnePath> {} ]", p.to_turtle()),
        }
    }

    /// [OPUS-4.8] Serialises the path as a **SPARQL property-path expression**
    /// (for `$PATH` pre-binding in SHACL-SPARQL property constraints, §5.2.1):
    /// `<iri>`, `^p`, `p1/p2`, `p1|p2`, `p*`, `p+`, `p?`, fully parenthesised so
    /// it composes safely when textually spliced into a query's BGP. Returns
    /// `None` for an empty/degenerate sequence or alternative (no SPARQL form).
    pub fn to_sparql_property_path(&self) -> Option<String> {
        Some(match self {
            Path::Predicate(p) => format!("<{p}>"),
            Path::Inverse(p) => format!("^({})", p.to_sparql_property_path()?),
            Path::Sequence(ps) => {
                if ps.is_empty() {
                    return None;
                }
                let inner: Vec<String> = ps
                    .iter()
                    .map(|p| p.to_sparql_property_path())
                    .collect::<Option<_>>()?;
                format!("({})", inner.join("/"))
            }
            Path::Alternative(ps) => {
                if ps.is_empty() {
                    return None;
                }
                let inner: Vec<String> = ps
                    .iter()
                    .map(|p| p.to_sparql_property_path())
                    .collect::<Option<_>>()?;
                format!("({})", inner.join("|"))
            }
            Path::ZeroOrMore(p) => format!("({})*", p.to_sparql_property_path()?),
            Path::OneOrMore(p) => format!("({})+", p.to_sparql_property_path()?),
            Path::ZeroOrOne(p) => format!("({})?", p.to_sparql_property_path()?),
        })
    }
}

/// Breadth-first reachability closure of `inner` from `start`; `reflexive`
/// includes `start` itself (zeroOrMore vs oneOrMore).
fn closure(g: &GraphView, start: &Term, inner: &Path, forward: bool, reflexive: bool) -> Vec<Term> {
    let mut seen: FxHashSet<Term> = FxHashSet::default();
    let mut out: Vec<Term> = Vec::new();
    let mut queue: Vec<Term> = vec![start.clone()];
    if reflexive {
        seen.insert(start.clone());
        out.push(start.clone());
    }
    let mut i = 0;
    while i < queue.len() {
        let n = queue[i].clone();
        i += 1;
        for next in inner.step(g, &n, forward) {
            if seen.insert(next.clone()) {
                out.push(next.clone());
                queue.push(next);
            }
        }
    }
    out
}
