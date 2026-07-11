//! Full-path enumeration over a single RDF predicate.

use oxrdf::{NamedNode, Term};
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

/// Parameters for path enumeration over one predicate.
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
    /// Predicate defining the directed edge relation.
    pub via: NamedNode,
    /// Maximum edge count. Required for [`PathMode::All`].
    pub max_length: Option<usize>,
}

/// One materialized path.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PathSolution {
    /// Path nodes, including both endpoints.
    pub nodes: Vec<Term>,
    /// Edge terms, one fewer than `nodes`; currently each is `PathSpec::via`.
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

    let Some(via_id) = graph.id_of(&Term::NamedNode(spec.via.clone())) else {
        return Ok(Vec::new());
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
    let mut adjacency: BTreeMap<Id, Vec<Id>> = BTreeMap::new();
    let mut sinks = BTreeSet::new();
    for row in scan.rows.iter() {
        let [subject, _, object] = scan.to_spo(row);
        adjacency.entry(subject).or_default().push(object);
        sinks.insert(object);
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
    let edge = Term::NamedNode(spec.via.clone());
    Ok(paths
        .into_iter()
        .map(|ids| PathSolution {
            edges: vec![edge.clone(); ids.len() - 1],
            nodes: ids.into_iter().map(|id| graph.dict.term(id)).collect(),
        })
        .collect())
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
