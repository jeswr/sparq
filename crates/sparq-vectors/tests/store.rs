//! `.spqv` store round-trip, sparse coverage and error-path tests.

use sparq_vectors::VectorStore;

fn tmp_path(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sparq-vectors-test-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

#[test]
fn roundtrip_sparse_ids() {
    let path = tmp_path("roundtrip.spqv");
    let mut store = VectorStore::create(&path, 3).unwrap();
    // Sparse, unsorted ids — only some terms get embeddings.
    store.put(42, &[1.0, 2.0, 3.0]).unwrap();
    store.put(7, &[4.0, 5.0, 6.0]).unwrap();
    store.put(1_000_000, &[-1.5, 0.0, 2.5]).unwrap();

    // Reads work during the build phase…
    assert_eq!(store.get(7), Some(&[4.0f32, 5.0, 6.0][..]));
    assert_eq!(store.get(8), None);
    assert_eq!(store.len(), 3);

    store.finalize().unwrap();

    // …and from the finalized (mmap'd) handle…
    assert_eq!(store.get(42), Some(&[1.0f32, 2.0, 3.0][..]));
    assert_eq!(store.get(7), Some(&[4.0f32, 5.0, 6.0][..]));
    assert_eq!(store.get(1_000_000), Some(&[-1.5f32, 0.0, 2.5][..]));
    assert_eq!(store.get(8), None);
    assert_eq!(store.get(0), None);
    assert_eq!(store.get(u32::MAX), None);

    // …and from a fresh open.
    let reopened = VectorStore::open(&path).unwrap();
    assert_eq!(reopened.dim(), 3);
    assert_eq!(reopened.len(), 3);
    assert_eq!(reopened.get(42), Some(&[1.0f32, 2.0, 3.0][..]));
    assert_eq!(reopened.get(43), None);

    // iter() yields all pairs in insertion order.
    let pairs: Vec<(u32, Vec<f32>)> =
        reopened.iter().map(|(id, v)| (id, v.to_vec())).collect();
    assert_eq!(pairs[0], (42, vec![1.0, 2.0, 3.0]));
    assert_eq!(pairs[1], (7, vec![4.0, 5.0, 6.0]));
    assert_eq!(pairs[2], (1_000_000, vec![-1.5, 0.0, 2.5]));
}

#[test]
fn empty_store_roundtrips() {
    let path = tmp_path("empty.spqv");
    let mut store = VectorStore::create(&path, 8).unwrap();
    store.finalize().unwrap();
    let reopened = VectorStore::open(&path).unwrap();
    assert!(reopened.is_empty());
    assert_eq!(reopened.get(1), None);
    assert_eq!(reopened.iter().count(), 0);
}

#[test]
fn build_phase_errors() {
    let path = tmp_path("errors.spqv");
    assert!(VectorStore::create(&path, 0).is_err(), "dim 0 rejected");

    let mut store = VectorStore::create(&path, 2).unwrap();
    assert!(store.put(1, &[1.0]).is_err(), "dim mismatch rejected");
    assert!(store.put(1, &[1.0, f32::NAN]).is_err(), "non-finite rejected");
    assert!(store.put(1, &[0.0, 0.0]).is_err(), "all-zero rejected (cosine undefined)");
    store.put(1, &[1.0, 2.0]).unwrap();
    assert!(store.put(1, &[3.0, 4.0]).is_err(), "duplicate id rejected");
    store.finalize().unwrap();
    assert!(store.put(2, &[1.0, 2.0]).is_err(), "put after finalize rejected");
    store.finalize().unwrap(); // idempotent
}

#[test]
fn open_rejects_garbage() {
    let path = tmp_path("garbage.spqv");
    std::fs::write(&path, b"not a vector store at all").unwrap();
    assert!(VectorStore::open(&path).is_err());

    let short = tmp_path("short.spqv");
    std::fs::write(&short, b"SPQV").unwrap();
    assert!(VectorStore::open(&short).is_err());

    assert!(VectorStore::open(tmp_path("does-not-exist.spqv")).is_err());

    // Truncated body: valid header claiming more data than the file holds.
    let truncated = tmp_path("truncated.spqv");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SPQV");
    bytes.extend_from_slice(&1u32.to_le_bytes()); // version
    bytes.extend_from_slice(&4u32.to_le_bytes()); // dim
    bytes.extend_from_slice(&100u64.to_le_bytes()); // count
    bytes.extend_from_slice(&[0u8; 12]);
    std::fs::write(&truncated, &bytes).unwrap();
    assert!(VectorStore::open(&truncated).is_err());

    // Overflowing header: count×dim×4 wraps usize — must be rejected by checked
    // arithmetic, not pass a wrapped size check.
    let overflow = tmp_path("overflow.spqv");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SPQV");
    bytes.extend_from_slice(&1u32.to_le_bytes());
    bytes.extend_from_slice(&u32::MAX.to_le_bytes()); // dim
    bytes.extend_from_slice(&u64::MAX.to_le_bytes()); // count
    bytes.extend_from_slice(&[0u8; 12]);
    std::fs::write(&overflow, &bytes).unwrap();
    assert!(VectorStore::open(&overflow).is_err());
}

/// Builds a structurally well-sized 1-d, 2-vector file with a caller-chosen id→slot
/// index section, to exercise `open`'s index validation.
fn file_with_index(name: &str, index: &[(u32, u32)]) -> std::path::PathBuf {
    let path = tmp_path(name);
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SPQV");
    bytes.extend_from_slice(&1u32.to_le_bytes()); // version
    bytes.extend_from_slice(&1u32.to_le_bytes()); // dim
    bytes.extend_from_slice(&2u64.to_le_bytes()); // count
    bytes.extend_from_slice(&[0u8; 12]);
    bytes.extend_from_slice(&1.0f32.to_le_bytes()); // slot 0
    bytes.extend_from_slice(&2.0f32.to_le_bytes()); // slot 1
    for &(id, slot) in index {
        bytes.extend_from_slice(&id.to_le_bytes());
        bytes.extend_from_slice(&slot.to_le_bytes());
    }
    std::fs::write(&path, &bytes).unwrap();
    path
}

#[test]
fn open_rejects_corrupt_index() {
    // A valid index round-trips…
    let good = file_with_index("idx-good.spqv", &[(5, 1), (9, 0)]);
    let store = VectorStore::open(&good).unwrap();
    assert_eq!(store.get(5), Some(&[2.0f32][..]));
    assert_eq!(store.get(9), Some(&[1.0f32][..]));

    // …but unsorted ids, duplicate ids, out-of-range slots and duplicate slots are
    // all rejected at open (they would otherwise corrupt get/iter).
    assert!(VectorStore::open(file_with_index("idx-unsorted.spqv", &[(9, 0), (5, 1)])).is_err());
    assert!(VectorStore::open(file_with_index("idx-dup-id.spqv", &[(5, 0), (5, 1)])).is_err());
    assert!(VectorStore::open(file_with_index("idx-bad-slot.spqv", &[(5, 0), (9, 7)])).is_err());
    assert!(VectorStore::open(file_with_index("idx-dup-slot.spqv", &[(5, 0), (9, 0)])).is_err());
}
