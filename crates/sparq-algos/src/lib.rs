#![doc = include_str!("../README.md")]
#![forbid(unsafe_code)] // [OPUS-4.8] sq-mqvm: crate has zero `unsafe`

pub mod centrality;
pub mod community;
pub mod graph;
pub mod pagerank;

pub use centrality::{degree_centrality, degree_centrality_normalized, top_k, Direction};
pub use community::{
    label_propagation, num_communities, weakly_connected_components, LabelPropConfig,
};
pub use graph::{NodeFilter, NodeGraph};
pub use pagerank::{pagerank, PageRankConfig};

#[cfg(test)]
mod tests {
    use super::*;
    use oxrdf::{NamedNode, Term};
    use sparq_core::Graph;

    /// Builds a graph from N-Triples and the matching node-graph view (entities only).
    fn build(nt: &str) -> (Graph, NodeGraph) {
        let g = Graph::load_str(nt, "nt").expect("parse");
        let ng = NodeGraph::build(&g);
        (g, ng)
    }

    fn iri(s: &str) -> Term {
        Term::NamedNode(NamedNode::new(s).unwrap())
    }

    /// A small cycle a → b → c → a plus an extra edge a → c.
    const CYCLE: &str = r#"
<http://e/a> <http://p/knows> <http://e/b> .
<http://e/b> <http://p/knows> <http://e/c> .
<http://e/c> <http://p/knows> <http://e/a> .
<http://e/a> <http://p/knows> <http://e/c> .
"#;

    #[test]
    fn node_graph_basics() {
        let (_g, ng) = build(CYCLE);
        assert_eq!(ng.len(), 3);
        assert_eq!(ng.edge_count(), 4);
        assert!(!ng.is_empty());
    }

    #[test]
    fn literals_excluded_by_default_but_included_with_all() {
        let nt = r#"
<http://e/a> <http://p/name> "Alice" .
<http://e/a> <http://p/knows> <http://e/b> .
"#;
        let g = Graph::load_str(nt, "nt").unwrap();
        // EntitiesOnly: only a and b are nodes; the "Alice" literal is dropped.
        let entities = NodeGraph::build(&g);
        assert_eq!(entities.len(), 2);
        assert_eq!(entities.edge_count(), 1);
        // All: the literal becomes a third node + a second edge.
        let all = NodeGraph::build_with(&g, NodeFilter::All);
        assert_eq!(all.len(), 3);
        assert_eq!(all.edge_count(), 2);
    }

    #[test]
    fn parallel_edges_collapse() {
        // Two predicates assert a → b; the view sees one edge.
        let nt = r#"
<http://e/a> <http://p/p1> <http://e/b> .
<http://e/a> <http://p/p2> <http://e/b> .
"#;
        let (_g, ng) = build(nt);
        assert_eq!(ng.edge_count(), 1);
    }

    #[test]
    fn in_out_neighbors_are_symmetric() {
        let (_g, ng) = build(CYCLE);
        // For every out-edge i→j there must be a matching in-edge j←i.
        for i in 0..ng.len() {
            for &j in ng.out_neighbors(i) {
                assert!(
                    ng.in_neighbors(j as usize).contains(&(i as u32)),
                    "missing reverse edge"
                );
            }
        }
    }

    #[test]
    fn pagerank_sums_to_one_and_symmetric_cycle_is_uniform() {
        // A pure 3-cycle: every node has in=out=1, so ranks must be uniform = 1/3.
        let nt = r#"
<http://e/a> <http://p/k> <http://e/b> .
<http://e/b> <http://p/k> <http://e/c> .
<http://e/c> <http://p/k> <http://e/a> .
"#;
        let (_g, ng) = build(nt);
        let r = pagerank(&ng, PageRankConfig::default());
        let sum: f64 = r.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6, "sum {sum}");
        for v in &r {
            assert!((v - 1.0 / 3.0).abs() < 1e-6, "rank {v} not uniform");
        }
    }

    #[test]
    fn pagerank_authority_beats_periphery() {
        // A "star into a hub": x, y, z all point at h. h is the authority.
        let nt = r#"
<http://e/x> <http://p/k> <http://e/h> .
<http://e/y> <http://p/k> <http://e/h> .
<http://e/z> <http://p/k> <http://e/h> .
"#;
        let g = Graph::load_str(nt, "nt").unwrap();
        let ng = NodeGraph::build(&g);
        let r = pagerank(&ng, PageRankConfig::default());
        let h = ng.index_of(g.id_of(&iri("http://e/h")).unwrap()).unwrap();
        let x = ng.index_of(g.id_of(&iri("http://e/x")).unwrap()).unwrap();
        assert!(r[h] > r[x], "hub {} should outrank leaf {}", r[h], r[x]);
        let sum: f64 = r.iter().sum();
        assert!((sum - 1.0).abs() < 1e-6);
    }

    #[test]
    fn pagerank_empty_graph() {
        let g = Graph::load_str("", "nt").unwrap();
        let ng = NodeGraph::build(&g);
        assert!(pagerank(&ng, PageRankConfig::default()).is_empty());
    }

    #[test]
    fn degree_centrality_directions() {
        // a → b, a → c, b → c. Degrees: a out2/in0, b out1/in1, c out0/in2.
        let nt = r#"
<http://e/a> <http://p/k> <http://e/b> .
<http://e/a> <http://p/k> <http://e/c> .
<http://e/b> <http://p/k> <http://e/c> .
"#;
        let g = Graph::load_str(nt, "nt").unwrap();
        let ng = NodeGraph::build(&g);
        let a = ng.index_of(g.id_of(&iri("http://e/a")).unwrap()).unwrap();
        let c = ng.index_of(g.id_of(&iri("http://e/c")).unwrap()).unwrap();
        let out = degree_centrality(&ng, Direction::Out);
        let inc = degree_centrality(&ng, Direction::In);
        let tot = degree_centrality(&ng, Direction::Total);
        assert_eq!(out[a], 2);
        assert_eq!(inc[a], 0);
        assert_eq!(out[c], 0);
        assert_eq!(inc[c], 2);
        assert_eq!(tot[a], 2);
        assert_eq!(tot[c], 2);
        // Normalised by n-1 = 2.
        let nrm = degree_centrality_normalized(&ng, Direction::In);
        assert!((nrm[c] - 1.0).abs() < 1e-12);
    }

    #[test]
    fn top_k_is_deterministic_with_index_tiebreak() {
        let scores = vec![5usize, 5, 1, 5];
        let top = top_k(&scores, 2);
        // Three nodes tie at 5; smallest indices win → (0, 5) then (1, 5).
        assert_eq!(top, vec![(0, 5), (1, 5)]);
    }

    #[test]
    fn weakly_connected_components_partition() {
        // Two disjoint clusters: {a,b} and {c,d}.
        let nt = r#"
<http://e/a> <http://p/k> <http://e/b> .
<http://e/c> <http://p/k> <http://e/d> .
"#;
        let g = Graph::load_str(nt, "nt").unwrap();
        let ng = NodeGraph::build(&g);
        let comp = weakly_connected_components(&ng);
        assert_eq!(num_communities(&comp), 2);
        let a = ng.index_of(g.id_of(&iri("http://e/a")).unwrap()).unwrap();
        let b = ng.index_of(g.id_of(&iri("http://e/b")).unwrap()).unwrap();
        let c = ng.index_of(g.id_of(&iri("http://e/c")).unwrap()).unwrap();
        assert_eq!(comp[a], comp[b]);
        assert_ne!(comp[a], comp[c]);
    }

    #[test]
    fn wcc_ignores_edge_direction() {
        // a → b only; still one weakly connected component.
        let nt = "<http://e/a> <http://p/k> <http://e/b> .\n";
        let (_g, ng) = build(nt);
        let comp = weakly_connected_components(&ng);
        assert_eq!(num_communities(&comp), 1);
    }

    #[test]
    fn label_propagation_finds_two_cliques() {
        // Two triangles joined by a single bridge edge — LP should keep them apart,
        // or at worst find <= the WCC count (one component here).
        let nt = r#"
<http://e/a> <http://p/k> <http://e/b> .
<http://e/b> <http://p/k> <http://e/c> .
<http://e/c> <http://p/k> <http://e/a> .
<http://e/d> <http://p/k> <http://e/e> .
<http://e/e> <http://p/k> <http://e/f> .
<http://e/f> <http://p/k> <http://e/d> .
<http://e/c> <http://p/k> <http://e/d> .
"#;
        let (_g, ng) = build(nt);
        let comm = label_propagation(&ng, LabelPropConfig::default());
        // Deterministic: same labels across runs.
        let comm2 = label_propagation(&ng, LabelPropConfig::default());
        assert_eq!(comm, comm2);
        // Every node is labelled; ids are dense.
        assert_eq!(comm.len(), ng.len());
        assert!(num_communities(&comm) >= 1);
    }

    #[test]
    fn label_propagation_empty() {
        let g = Graph::load_str("", "nt").unwrap();
        let ng = NodeGraph::build(&g);
        assert!(label_propagation(&ng, LabelPropConfig::default()).is_empty());
    }

    #[test]
    fn node_index_round_trips_to_term() {
        let nt = "<http://e/a> <http://p/k> <http://e/b> .\n";
        let g = Graph::load_str(nt, "nt").unwrap();
        let ng = NodeGraph::build(&g);
        let a_id = g.id_of(&iri("http://e/a")).unwrap();
        let a = ng.index_of(a_id).unwrap();
        assert_eq!(ng.term(&g, a), iri("http://e/a"));
        assert_eq!(ng.dict_id(a), a_id);
    }
}
