use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests/fixtures")
        .join(name)
}

#[test]
fn snikmeta_stats_match_known_cardinalities_and_full_load() {
    let path = fixture("snikmeta.hdt");
    let stats = sparq_hdt::stats(&path).expect("reading fixture metadata");
    let graph = sparq_hdt::load(&path).expect("fully loading fixture");

    assert_eq!(stats.shared, 43);
    assert_eq!(stats.subjects_only, 6);
    assert_eq!(stats.objects_only, 133);
    assert_eq!(stats.predicates, 23);
    assert_eq!(stats.distinct_subjects(), 49);
    assert_eq!(stats.distinct_objects(), 176);
    assert_eq!(stats.triples, graph.len());
}

#[cfg(feature = "write")]
#[test]
fn stats_reader_matches_written_graph_and_full_reader_load() {
    let source = concat!(
        "<http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/bob> .\n",
        "<http://example.org/bob> <http://xmlns.com/foaf/0.1/knows> <http://example.org/alice> .\n",
        "<http://example.org/alice> <http://xmlns.com/foaf/0.1/name> \"Alice\" .\n",
        "_:b0 <http://example.org/link> _:b1 .\n",
    );
    let graph = sparq_core::Graph::load_str(source, "ntriples").expect("building source graph");
    let dir = tempfile::tempdir().expect("creating scratch directory");
    let path = dir.path().join("stats.hdt");
    sparq_hdt::save(&graph, &path).expect("writing HDT archive");
    let bytes = std::fs::read(path).expect("reading HDT bytes");

    let stats = sparq_hdt::stats_reader(std::io::Cursor::new(bytes.clone()))
        .expect("reading metadata from memory");
    let loaded =
        sparq_hdt::load_reader(std::io::Cursor::new(bytes)).expect("fully loading the same bytes");

    assert_eq!(stats.triples, 4);
    assert_eq!(stats.triples, loaded.len());
    assert_eq!(stats.shared, 2);
    assert_eq!(stats.subjects_only, 1);
    assert_eq!(stats.objects_only, 2);
    assert_eq!(stats.predicates, 3);
    assert_eq!(stats.distinct_subjects(), 3);
    assert_eq!(stats.distinct_objects(), 4);
}
