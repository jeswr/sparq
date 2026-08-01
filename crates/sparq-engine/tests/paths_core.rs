#![cfg(feature = "paths")]

use oxrdf::{NamedNode, Term};
use sparq_core::Graph;
use sparq_engine::{enumerate_paths, Endpoint, PathMode, PathSolution, PathSpec, Via};
use std::collections::{BTreeMap, BTreeSet};

const EX: &str = "http://example/";

fn iri(local: &str) -> Term {
    Term::NamedNode(NamedNode::new(format!("{EX}{local}")).unwrap())
}

fn via() -> NamedNode {
    NamedNode::new(format!("{EX}p")).unwrap()
}

fn graph(edges: &[(&str, &str)]) -> Graph {
    let nt = edges
        .iter()
        .map(|(from, to)| format!("<{EX}{from}> <{EX}p> <{EX}{to}> .\n"))
        .collect::<String>();
    Graph::load_str(&nt, "ntriples").unwrap()
}

fn spec(mode: PathMode, start: &str, end: Option<&str>, max_length: Option<usize>) -> PathSpec {
    PathSpec {
        mode,
        cyclic: end.is_none(),
        start: Some(Endpoint::Node(iri(start))),
        end: end.map(|term| Endpoint::Node(iri(term))),
        via: Via::Predicate(via()),
        max_length,
    }
}

fn node_names(solution: &PathSolution) -> Vec<String> {
    solution
        .nodes
        .iter()
        .map(|term| {
            term.to_string()
                .trim_start_matches('<')
                .trim_end_matches('>')
                .trim_start_matches(EX)
                .to_owned()
        })
        .collect()
}

#[test]
fn shortest_returns_both_diamond_paths() {
    let g = graph(&[("a", "b"), ("b", "d"), ("a", "c"), ("c", "d")]);
    let got = enumerate_paths(&g, &spec(PathMode::Shortest, "a", Some("d"), None)).unwrap();
    assert_eq!(got.len(), 2);
    assert_eq!(
        got.iter().map(node_names).collect::<Vec<_>>(),
        vec![vec!["a", "b", "d"], vec!["a", "c", "d"]]
    );
    assert!(got
        .iter()
        .all(|path| path.nodes.len() == path.edges.len() + 1));
}

#[test]
fn shortest_cuts_long_route_but_all_includes_it() {
    let g = graph(&[
        ("a", "b"),
        ("b", "d"),
        ("a", "c"),
        ("c", "d"),
        ("a", "x"),
        ("x", "y"),
        ("y", "d"),
    ]);
    let shortest = enumerate_paths(&g, &spec(PathMode::Shortest, "a", Some("d"), None)).unwrap();
    assert_eq!(shortest.len(), 2);
    let all = enumerate_paths(&g, &spec(PathMode::All, "a", Some("d"), Some(3))).unwrap();
    assert_eq!(all.len(), 3);
    assert_eq!(node_names(&all[2]), vec!["a", "x", "y", "d"]);
}

#[test]
fn cyclic_triangle_returns_to_start() {
    let g = graph(&[("a", "b"), ("b", "c"), ("c", "a")]);
    let got = enumerate_paths(&g, &spec(PathMode::Shortest, "a", None, None)).unwrap();
    assert_eq!(got.len(), 1);
    assert_eq!(node_names(&got[0]), vec!["a", "b", "c", "a"]);
}

#[test]
fn all_without_max_length_errs() {
    let g = graph(&[("a", "b")]);
    let error = enumerate_paths(&g, &spec(PathMode::All, "a", Some("b"), None)).unwrap_err();
    assert_eq!(error, "PATHS ALL requires MAX LENGTH");
}

fn reference_paths(
    edges: &[(usize, usize)],
    mode: PathMode,
    maximum: usize,
) -> BTreeSet<Vec<usize>> {
    let mut adjacency: BTreeMap<usize, Vec<usize>> = BTreeMap::new();
    let starts: BTreeSet<_> = edges.iter().map(|&(from, _)| from).collect();
    let ends: BTreeSet<_> = edges.iter().map(|&(_, to)| to).collect();
    for &(from, to) in edges {
        adjacency.entry(from).or_default().push(to);
    }
    for next in adjacency.values_mut() {
        next.sort_unstable();
        next.dedup();
    }

    fn visit(
        adjacency: &BTreeMap<usize, Vec<usize>>,
        end: usize,
        maximum: usize,
        path: &mut Vec<usize>,
        found: &mut Vec<Vec<usize>>,
    ) {
        if path.len() - 1 == maximum {
            return;
        }
        if let Some(nexts) = adjacency.get(path.last().unwrap()) {
            for &next in nexts {
                if next == end {
                    path.push(next);
                    found.push(path.clone());
                    path.pop();
                } else if !path.contains(&next) {
                    path.push(next);
                    visit(adjacency, end, maximum, path, found);
                    path.pop();
                }
            }
        }
    }

    let mut answer = BTreeSet::new();
    for &start in &starts {
        for &end in &ends {
            if start == end {
                continue;
            }
            let mut candidates = Vec::new();
            visit(&adjacency, end, maximum, &mut vec![start], &mut candidates);
            if mode == PathMode::Shortest {
                if let Some(minimum) = candidates.iter().map(Vec::len).min() {
                    candidates.retain(|path| path.len() == minimum);
                }
            }
            answer.extend(candidates);
        }
    }
    answer
}

#[test]
fn seeded_random_graph_agrees_with_recursive_reference() {
    let mut state = 0x5eed_cafe_u32;
    let mut edges = BTreeSet::new();
    while edges.len() < 40 {
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let from = (state as usize) % 20;
        state = state.wrapping_mul(1_664_525).wrapping_add(1_013_904_223);
        let to = (state as usize) % 20;
        if from != to {
            edges.insert((from, to));
        }
    }
    let edges: Vec<_> = edges.into_iter().collect();
    let labels: Vec<_> = edges
        .iter()
        .map(|(from, to)| (format!("n{from}"), format!("n{to}")))
        .collect();
    let borrowed: Vec<_> = labels
        .iter()
        .map(|(from, to)| (from.as_str(), to.as_str()))
        .collect();
    let g = graph(&borrowed);

    for mode in [PathMode::Shortest, PathMode::All] {
        let request = PathSpec {
            mode,
            cyclic: false,
            start: None,
            end: None,
            via: Via::Predicate(via()),
            max_length: (mode == PathMode::All).then_some(4),
        };
        let got: BTreeSet<Vec<usize>> = enumerate_paths(&g, &request)
            .unwrap()
            .iter()
            .map(node_names)
            .map(|nodes| {
                nodes
                    .into_iter()
                    .map(|node| node.trim_start_matches('n').parse().unwrap())
                    .collect()
            })
            .collect();
        let expected = reference_paths(&edges, mode, if mode == PathMode::All { 4 } else { 20 });
        assert!(
            !expected.is_empty(),
            "seeded fixture must exercise successful paths"
        );
        assert_eq!(got, expected, "differential mismatch in {mode:?} mode");
    }
}
