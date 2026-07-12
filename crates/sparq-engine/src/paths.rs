//! Full-path enumeration over an RDF predicate or materialized graph pattern.

use oxrdf::vocab::xsd;
use oxrdf::{Literal, NamedNode, Term};
use spargebra::algebra::GraphPattern;
use spargebra::{Query, SparqlParser};
use sparq_core::{dict::Id, store::Perm, Graph};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

/// Selects shortest-path or bounded all-path enumeration.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PathMode {
    /// Return every path at the minimum length for each selected endpoint pair.
    Shortest,
    /// Return every path up to `PathSpec::max_length`.
    All,
}

/// Defines the directed edge relation used for path enumeration.
///
/// [GPT-5.6] `sq-lsp7k.3.3`: pattern relations are materialized once, then
/// consumed by the same deterministic T1 enumerator as predicate relations.
#[derive(Clone, Debug)]
pub enum Via {
    /// Every triple using this predicate is one directed edge.
    Predicate(NamedNode),
    /// A SPARQL group graph pattern whose reserved `?from` and `?to` variables
    /// designate each directed edge's endpoints.
    Pattern(String),
}

/// Parameters for path enumeration over an edge relation.
#[derive(Clone, Debug)]
pub struct PathSpec {
    /// Enumeration mode.
    pub mode: PathMode,
    /// If true, return only non-empty paths whose first and last nodes are equal.
    pub cyclic: bool,
    /// Optional fixed starting node; otherwise every edge subject is considered.
    pub start: Option<Term>,
    /// Optional fixed ending node; otherwise every edge object is considered.
    pub end: Option<Term>,
    /// Predicate or graph pattern defining the directed edge relation.
    pub via: Via,
    /// Maximum edge count. Required for [`PathMode::All`].
    pub max_length: Option<usize>,
}

/// One materialized path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathSolution {
    /// Path nodes, including both endpoints.
    pub nodes: Vec<Term>,
    /// Edge terms, one fewer than `nodes`. Pattern edges use the pattern source
    /// as a canonical, blank-node-free `xsd:string` marker.
    pub edges: Vec<Term>,
}

/// Enumerates paths selected by `spec` in deterministic `(length, node ids)` order.
///
/// `All` paths are simple unless `cyclic` is true, in which case the only repeated
/// node is the final return to the start. An unbounded `All` request is rejected.
pub fn enumerate_paths(graph: &Graph, spec: &PathSpec) -> Result<Vec<PathSolution>, String> {
    if spec.mode == PathMode::All && spec.max_length.is_none() {
        return Err("PATHS ALL requires MAX LENGTH".to_owned());
    }

    let (edge_pairs, edge) = edge_relation(graph, &spec.via)?;
    let mut adjacency: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    let mut sinks = BTreeSet::new();
    for (from, to) in edge_pairs {
        adjacency.entry(from).or_default().push(to);
        sinks.insert(to);
    }
    for next in adjacency.values_mut() {
        next.sort_unstable();
        next.dedup();
    }

    let starts = selected_ids(graph, spec.start.as_ref(), adjacency.keys().copied());
    let ends = selected_ids(graph, spec.end.as_ref(), sinks.iter().copied());
    let mut paths = Vec::new();
    if spec.cyclic {
        for start in starts {
            if spec.end.as_ref().is_some() && !ends.contains(&start) {
                continue;
            }
            enumerate_pair(&adjacency, start, start, spec, &mut paths);
        }
    } else {
        for start in starts {
            for &end in &ends {
                if start != end {
                    enumerate_pair(&adjacency, start, end, spec, &mut paths);
                }
            }
        }
    }

    paths.sort_by(|left, right| left.len().cmp(&right.len()).then_with(|| left.cmp(right)));
    Ok(paths
        .into_iter()
        .map(|ids| PathSolution {
            edges: vec![edge.clone(); ids.len() - 1],
            nodes: ids.into_iter().map(|id| graph.dict.term(id)).collect(),
        })
        .collect())
}

fn edge_relation(graph: &Graph, via: &Via) -> Result<(Vec<(Id, Id)>, Term), String> {
    match via {
        Via::Predicate(predicate) => {
            let edge = Term::NamedNode(predicate.clone());
            let Some(via_id) = graph.id_of(&edge) else {
                return Ok((Vec::new(), edge));
            };
            let scan = graph
                .store
                .scan_perm(&[None, Some(via_id), None], Perm::Pso)
                .or_else(|| {
                    graph
                        .store
                        .scan_perm(&[None, Some(via_id), None], Perm::Pos)
                })
                .expect("a predicate-leading permutation is always built");
            let pairs = scan
                .rows
                .iter()
                .map(|row| {
                    let [subject, _, object] = scan.to_spo(row);
                    (subject, object)
                })
                .collect();
            Ok((pairs, edge))
        }
        Via::Pattern(pattern) => {
            let source = format!("SELECT ?from ?to WHERE {{ {pattern} }}");
            let parsed = SparqlParser::new()
                .parse_query(&source)
                .map_err(|error| format!("invalid PATHS VIA pattern: {error}"))?;
            let Query::Select {
                pattern: GraphPattern::Project { inner, .. },
                ..
            } = parsed
            else {
                return Err("invalid SELECT projection for PATHS VIA pattern".to_owned());
            };
            let mut binds_from = false;
            let mut binds_to = false;
            inner.on_in_scope_variable(|variable| match variable.as_str() {
                "from" => binds_from = true,
                "to" => binds_to = true,
                _ => {}
            });
            if !binds_from {
                return Err("PATHS VIA pattern must bind ?from".to_owned());
            }
            if !binds_to {
                return Err("PATHS VIA pattern must bind ?to".to_owned());
            }
            let result = crate::query(graph, &source)?;
            let from = result
                .vars
                .iter()
                .position(|var| var.as_str() == "from")
                .ok_or_else(|| "PATHS VIA pattern must bind ?from".to_owned())?;
            let to = result
                .vars
                .iter()
                .position(|var| var.as_str() == "to")
                .ok_or_else(|| "PATHS VIA pattern must bind ?to".to_owned())?;
            let mut pairs = Vec::with_capacity(result.rows.len());
            for row in result.rows {
                let from_term = row[from].as_ref().ok_or_else(|| {
                    "PATHS VIA pattern must bind ?from in every solution".to_owned()
                })?;
                let to_term = row[to].as_ref().ok_or_else(|| {
                    "PATHS VIA pattern must bind ?to in every solution".to_owned()
                })?;
                let Some(from_id) = graph.id_of(from_term) else {
                    continue;
                };
                let Some(to_id) = graph.id_of(to_term) else {
                    continue;
                };
                pairs.push((from_id, to_id));
            }
            let marker = Term::Literal(Literal::new_typed_literal(pattern.clone(), xsd::STRING));
            Ok((pairs, marker))
        }
    }
}

fn selected_ids(graph: &Graph, fixed: Option<&Term>, all: impl Iterator<Item = Id>) -> Vec<Id> {
    match fixed {
        Some(term) => graph.id_of(term).into_iter().collect(),
        None => all.collect(),
    }
}

fn enumerate_pair(
    adjacency: &BTreeMap<Id, Vec<Id>>,
    start: Id,
    end: Id,
    spec: &PathSpec,
    output: &mut Vec<Vec<Id>>,
) {
    match spec.mode {
        PathMode::Shortest => shortest(adjacency, start, end, output),
        PathMode::All => {
            let mut path = vec![start];
            all_paths(
                adjacency,
                end,
                spec.max_length.expect("validated above"),
                &mut path,
                output,
            );
        }
    }
}

fn shortest(adjacency: &BTreeMap<Id, Vec<Id>>, start: Id, end: Id, output: &mut Vec<Vec<Id>>) {
    let mut queue = VecDeque::from([vec![start]]);
    let mut found_length = None;
    while let Some(path) = queue.pop_front() {
        let edges = path.len() - 1;
        if found_length.is_some_and(|minimum| edges >= minimum) {
            continue;
        }
        let current = *path.last().expect("paths are non-empty");
        let Some(nexts) = adjacency.get(&current) else {
            continue;
        };
        for &next in nexts {
            if next == end {
                let mut found = path.clone();
                found.push(next);
                found_length = Some(edges + 1);
                output.push(found);
            } else if !path.contains(&next) {
                let mut extended = path.clone();
                extended.push(next);
                queue.push_back(extended);
            }
        }
    }
}

fn all_paths(
    adjacency: &BTreeMap<Id, Vec<Id>>,
    end: Id,
    maximum: usize,
    path: &mut Vec<Id>,
    output: &mut Vec<Vec<Id>>,
) {
    if path.len() > maximum {
        return;
    }
    let current = *path.last().expect("paths are non-empty");
    let Some(nexts) = adjacency.get(&current) else {
        return;
    };
    for &next in nexts {
        if next == end {
            path.push(next);
            output.push(path.clone());
            path.pop();
        } else if !path.contains(&next) {
            path.push(next);
            all_paths(adjacency, end, maximum, path, output);
            path.pop();
        }
    }
}
