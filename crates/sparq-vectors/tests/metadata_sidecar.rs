#![cfg(feature = "metadata-sidecar")]

use sparq_vectors::{nearest_exact, nearest_exact_with_meta, VectorStore, SPQV_VERSION};

fn tmp_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!(
        "sparq-vectors-metadata-{}-{:?}",
        std::process::id(),
        std::thread::current().id()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn exact_ranking_and_metadata_survive_finalize_and_open_from_bytes() {
    let path = tmp_path("roundtrip.spqv");
    let mut store = VectorStore::create(&path, 2).unwrap();
    store
        .put_with_meta(30, &[0.0, 1.0], "tag:\0beta-β")
        .unwrap();
    store.put(10, &[1.0, 0.0]).unwrap();
    store.put_with_meta(20, &[-1.0, 0.0], "").unwrap();

    assert_eq!(store.meta(30), Some("tag:\0beta-β"));
    assert_eq!(store.meta(10), None);
    assert_eq!(store.meta(20), Some(""));

    store.finalize().unwrap();
    let bytes = std::fs::read(&path).unwrap();
    assert_eq!(u32::from_le_bytes(bytes[4..8].try_into().unwrap()), 4);

    let plain = nearest_exact(&store, &[1.0, 0.0], 3);
    let decorated = nearest_exact_with_meta(&store, &[1.0, 0.0], 3);
    let decorated_ranking: Vec<_> = decorated
        .iter()
        .map(|&(id, score, _)| (id, score))
        .collect();
    assert_eq!(
        decorated_ranking, plain,
        "metadata must not change ranking or scores"
    );
    assert_eq!(
        decorated,
        vec![
            (10, 1.0, None),
            (30, 0.0, Some("tag:\0beta-β".to_owned())),
            (20, -1.0, Some(String::new())),
        ]
    );

    let reopened = VectorStore::open_from_bytes(bytes).unwrap();
    assert_eq!(reopened.meta(30), Some("tag:\0beta-β"));
    assert_eq!(reopened.meta(10), None);
    assert_eq!(reopened.meta(20), Some(""));
    assert_eq!(
        nearest_exact_with_meta(&reopened, &[1.0, 0.0], 3),
        decorated,
        "finalize + open_from_bytes must preserve tags and exact ranking"
    );
}

#[test]
fn enabling_the_feature_without_writing_metadata_keeps_the_default_format() {
    let path = tmp_path("untagged.spqv");
    let mut store = VectorStore::create(&path, 2).unwrap();
    store.put(7, &[1.0, 0.0]).unwrap();
    store.finalize().unwrap();

    let bytes = std::fs::read(path).unwrap();
    assert_eq!(
        u32::from_le_bytes(bytes[4..8].try_into().unwrap()),
        SPQV_VERSION,
        "an untagged store must keep the default v2 format"
    );
    assert_eq!(store.meta(7), None);
}

#[cfg(feature = "spqv-provenance")]
#[test]
fn metadata_composes_with_embedding_provenance() {
    use sparq_vectors::{EmbeddingMetric, EmbeddingProvenance, Normalization};

    let path = tmp_path("provenance.spqv");
    let provenance =
        EmbeddingProvenance::new("model-a", EmbeddingMetric::Cosine, Normalization::L2);
    let mut store = VectorStore::create(&path, 2)
        .unwrap()
        .with_provenance(provenance.clone());
    store.put_with_meta(5, &[1.0, 0.0], "source-a").unwrap();
    store.finalize().unwrap();

    let reopened = VectorStore::open(&path).unwrap();
    assert_eq!(reopened.provenance(), Some(&provenance));
    assert_eq!(reopened.meta(5), Some("source-a"));
}
