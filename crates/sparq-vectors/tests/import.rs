//! [OPUS-4.8] (sq-xsq9) Bulk-import round-trip + fail-closed tests for `VectorStore::import_npy` /
//! `import_numeric_dump`: write a small external matrix of KNOWN vectors, import it keyed by dict id,
//! reopen the `.spqv`, and assert `get`/`nearest_term_exact` return exactly those vectors; then assert
//! dimension/row/dtype/order/length mismatches and a malformed/oversized `.npy` header all fail closed
//! (no silent reinterpretation, no panic).

use sparq_core::Graph;
use sparq_vectors::{nearest_term_exact, ImportBinding, ImportSpec, VectorStore};

fn tmp(name: &str) -> std::path::PathBuf {
    let dir = std::env::temp_dir().join(format!("sparq-vectors-import-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Serialize `rows × dim` f32 (row-major) into a minimal valid NumPy v1.0 `.npy` (C-order, `<f4`).
fn write_npy_f32(path: &std::path::Path, rows: usize, dim: usize, data: &[f32]) {
    assert_eq!(data.len(), rows * dim);
    let dict = format!("{{'descr': '<f4', 'fortran_order': False, 'shape': ({rows}, {dim}), }}");
    let mut header = dict.into_bytes();
    let unpadded = 10 + header.len() + 1; // 6 magic + 2 version + 2 len + body + '\n'
    let pad = (64 - (unpadded % 64)) % 64;
    header.extend(std::iter::repeat_n(b' ', pad));
    header.push(b'\n');
    let mut out = Vec::new();
    out.extend_from_slice(b"\x93NUMPY");
    out.push(1);
    out.push(0);
    out.extend_from_slice(&(header.len() as u16).to_le_bytes());
    out.extend_from_slice(&header);
    for &x in data {
        out.extend_from_slice(&x.to_le_bytes());
    }
    std::fs::write(path, &out).unwrap();
}

/// Same matrix as f64 (`<f8`) to exercise the narrowing path.
fn write_npy_f64(path: &std::path::Path, rows: usize, dim: usize, data: &[f32]) {
    let dict = format!("{{'descr': '<f8', 'fortran_order': False, 'shape': ({rows}, {dim}), }}");
    let mut header = dict.into_bytes();
    let unpadded = 10 + header.len() + 1;
    let pad = (64 - (unpadded % 64)) % 64;
    header.extend(std::iter::repeat_n(b' ', pad));
    header.push(b'\n');
    let mut out = Vec::new();
    out.extend_from_slice(b"\x93NUMPY");
    out.push(1);
    out.push(0);
    out.extend_from_slice(&(header.len() as u16).to_le_bytes());
    out.extend_from_slice(&header);
    for &x in data {
        out.extend_from_slice(&(x as f64).to_le_bytes());
    }
    std::fs::write(path, &out).unwrap();
}

const ROWS: [[f32; 4]; 3] = [
    [1.0, 0.0, 0.0, 0.0], // alice
    [0.9, 0.1, 0.0, 0.0], // bob — close to alice
    [0.0, 0.0, 0.0, 1.0], // carol — far from alice
];

#[test]
fn npy_round_trip_get_and_nearest() {
    let npy = tmp("vecs.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&npy, 3, 4, &flat);

    let ids = [10u32, 20, 30]; // arbitrary, sparse dict ids; row i -> ids[i]
    let spqv = tmp("npy.spqv");
    let store = VectorStore::import_npy(
        ImportSpec {
            spqv_path: &spqv,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    )
    .expect("npy import");

    // Returned handle reads the imported vectors keyed by the supplied ids.
    assert_eq!(store.len(), 3);
    assert_eq!(store.get(10), Some(&ROWS[0][..]));
    assert_eq!(store.get(20), Some(&ROWS[1][..]));
    assert_eq!(store.get(30), Some(&ROWS[2][..]));
    assert_eq!(store.get(99), None);

    // ...and a fresh open from disk sees the same.
    let reopened = VectorStore::open(&spqv).unwrap();
    assert_eq!(reopened.dim(), 4);
    assert_eq!(reopened.get(20), Some(&ROWS[1][..]));

    // The imported geometry is preserved: querying with alice's own vector ranks alice (id 10,
    // cosine 1.0) first, then bob (id 20) as the nearest distinct neighbour — far ahead of carol.
    let near = sparq_vectors::nearest_exact(&reopened, &ROWS[0], 3);
    assert_eq!(near[0].0, 10, "alice (id 10) matches her own query vector");
    assert_eq!(
        near[1].0, 20,
        "bob (id 20) is alice's nearest distinct neighbour"
    );
    assert_eq!(near[2].0, 30, "carol (id 30) is farthest");
}

#[test]
fn numeric_dump_round_trip() {
    let dump = tmp("vecs.f32");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    let mut bytes = Vec::new();
    for &x in &flat {
        bytes.extend_from_slice(&x.to_le_bytes());
    }
    std::fs::write(&dump, &bytes).unwrap();

    let ids = [7u32, 8, 9];
    let spqv = tmp("dump.spqv");
    let store = VectorStore::import_numeric_dump(
        ImportSpec {
            spqv_path: &spqv,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &dump,
    )
    .expect("dump import");
    assert_eq!(store.get(7), Some(&ROWS[0][..]));
    assert_eq!(store.get(9), Some(&ROWS[2][..]));
    // Reopen confirms persistence.
    assert_eq!(VectorStore::open(&spqv).unwrap().get(8), Some(&ROWS[1][..]));
}

#[test]
fn npy_f64_is_narrowed_to_f32() {
    let npy = tmp("vecs64.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f64(&npy, 3, 4, &flat);
    let ids = [1u32, 2, 3];
    let spqv = tmp("npy64.spqv");
    let store = VectorStore::import_npy(
        ImportSpec {
            spqv_path: &spqv,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    )
    .expect("f64 npy import");
    // f64 values were exact f32 here, so the narrowed vectors equal the originals.
    assert_eq!(store.get(1), Some(&ROWS[0][..]));
    assert_eq!(store.get(3), Some(&ROWS[2][..]));
}

#[test]
fn fingerprint_binding_enables_check_graph() {
    let ttl = r#"
        @prefix ex: <http://example.org/> .
        ex:alice ex:knows ex:bob .
        ex:bob ex:knows ex:carol .
    "#;
    let g = Graph::load_str(ttl, "turtle").unwrap();
    let other = Graph::load_str(
        "@prefix ex: <http://example.org/> . ex:x ex:p ex:y .",
        "turtle",
    )
    .unwrap();
    let alice = g
        .id_of(&oxrdf::Term::NamedNode(
            oxrdf::NamedNode::new("http://example.org/alice").unwrap(),
        ))
        .unwrap();
    let bob = g
        .id_of(&oxrdf::Term::NamedNode(
            oxrdf::NamedNode::new("http://example.org/bob").unwrap(),
        ))
        .unwrap();
    let carol = g
        .id_of(&oxrdf::Term::NamedNode(
            oxrdf::NamedNode::new("http://example.org/carol").unwrap(),
        ))
        .unwrap();

    let npy = tmp("fp.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&npy, 3, 4, &flat);
    let ids = [alice, bob, carol];
    let spqv = tmp("fp.spqv");
    let store = VectorStore::import_npy(
        ImportSpec {
            spqv_path: &spqv,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Graph(&g),
        },
        &npy,
    )
    .unwrap();

    // Bound to g: check_graph passes for g, fails for a different graph generation.
    assert!(store.fingerprint().is_some());
    assert!(store.check_graph(&g).is_ok());
    assert!(store.check_graph(&other).is_err());

    // ...and a term-keyed query against the build graph finds bob nearest to alice.
    let near = nearest_term_exact(
        &store,
        &g,
        &oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://example.org/alice").unwrap()),
        1,
    );
    assert_eq!(near.len(), 1);
    assert_eq!(
        near[0].0,
        oxrdf::Term::NamedNode(oxrdf::NamedNode::new("http://example.org/bob").unwrap())
    );
}

// --------------------------------------------------------------- fail-closed paths

#[test]
fn npy_dimension_mismatch_fails_closed() {
    let npy = tmp("dimx.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&npy, 3, 4, &flat); // dim 4 in the file
    let ids = [1u32, 2, 3];
    // store dim 8 != array dim 4
    let Err(err) = VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("dimx.spqv"),
            dim: 8,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    ) else {
        panic!("dim mismatch must fail closed");
    };
    assert!(err.contains("dim"), "err: {err}");
}

#[test]
fn npy_row_count_mismatch_fails_closed() {
    let npy = tmp("rowx.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&npy, 3, 4, &flat); // 3 rows
    let ids = [1u32, 2]; // only 2 ids
    let Err(err) = VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("rowx.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    ) else {
        panic!("row/id count mismatch must fail closed");
    };
    assert!(err.contains("rows") || err.contains("id"), "err: {err}");
}

#[test]
fn dump_wrong_length_fails_closed() {
    let dump = tmp("lenx.f32");
    // 3 rows x dim 4 x 4 bytes = 48 bytes expected; write 47 (truncated) and 49 (padded).
    std::fs::write(&dump, vec![0u8; 47]).unwrap();
    let ids = [1u32, 2, 3];
    assert!(VectorStore::import_numeric_dump(
        ImportSpec {
            spqv_path: tmp("lenx1.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound
        },
        &dump,
    )
    .is_err());
    std::fs::write(&dump, vec![0u8; 49]).unwrap();
    assert!(VectorStore::import_numeric_dump(
        ImportSpec {
            spqv_path: tmp("lenx2.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound
        },
        &dump,
    )
    .is_err());
}

#[test]
fn npy_malformed_header_fails_closed_bounded() {
    let ids = [1u32, 2, 3];

    // (1) Oversized declared HEADER_LEN: must be rejected by the cap, never pre-allocated.
    let huge = tmp("huge-hdr.npy");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"\x93NUMPY");
    bytes.push(1);
    bytes.push(0);
    bytes.extend_from_slice(&u16::MAX.to_le_bytes()); // claims a 65535-byte header...
    bytes.extend(std::iter::repeat_n(b' ', 100)); // ...but the file is tiny
    std::fs::write(&huge, &bytes).unwrap();
    let Err(err) = VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("huge.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &huge,
    ) else {
        panic!("oversized header must fail closed");
    };
    assert!(
        err.contains("cap") || err.contains("past the"),
        "err: {err}"
    );

    // (2) Garbage / bad magic.
    let garbage = tmp("garbage.npy");
    std::fs::write(&garbage, b"this is not a numpy file").unwrap();
    assert!(VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("g.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound
        },
        &garbage,
    )
    .is_err());

    // (3) Valid header but the body is truncated below the declared shape: decode fails closed.
    let trunc = tmp("trunc.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&trunc, 3, 4, &flat);
    let mut t = std::fs::read(&trunc).unwrap();
    t.truncate(t.len() - 4); // drop one f32 from the body
    std::fs::write(&trunc, &t).unwrap();
    assert!(VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("trunc.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound
        },
        &trunc,
    )
    .is_err());
}

#[test]
fn empty_matrix_imports_to_an_empty_store() {
    // [OPUS-4.8] (sq-xsq9) Edge case: a zero-row matrix + empty ids slice is a VALID import that
    // produces an empty (but well-formed, reopenable) `.spqv` — not an error, not a panic.
    let ids: [u32; 0] = [];

    // (a) .npy with shape (0, 4): a real header, an empty body.
    let npy = tmp("empty.npy");
    write_npy_f32(&npy, 0, 4, &[]);
    let spqv = tmp("empty-npy.spqv");
    let store = VectorStore::import_npy(
        ImportSpec {
            spqv_path: &spqv,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    )
    .expect("empty npy import");
    assert_eq!(store.len(), 0);
    assert!(store.is_empty());
    assert_eq!(store.get(0), None);
    // Reopens cleanly from disk as an empty store of the declared dim.
    let reopened = VectorStore::open(&spqv).unwrap();
    assert_eq!(reopened.dim(), 4);
    assert_eq!(reopened.len(), 0);
    // A query over an empty store yields nothing rather than panicking.
    assert!(sparq_vectors::nearest_exact(&reopened, &[1.0, 0.0, 0.0, 0.0], 5).is_empty());

    // (b) header-less dump: a genuinely empty (0-byte) file is the 0-row dump.
    let dump = tmp("empty.f32");
    std::fs::write(&dump, b"").unwrap();
    let spqv2 = tmp("empty-dump.spqv");
    let store2 = VectorStore::import_numeric_dump(
        ImportSpec {
            spqv_path: &spqv2,
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &dump,
    )
    .expect("empty dump import");
    assert_eq!(store2.len(), 0);
}

#[test]
fn npy_duplicate_id_fails_closed() {
    let npy = tmp("dupid.npy");
    let flat: Vec<f32> = ROWS.iter().flatten().copied().collect();
    write_npy_f32(&npy, 3, 4, &flat);
    let ids = [5u32, 5, 6]; // duplicate id -> put() rejects
    let Err(err) = VectorStore::import_npy(
        ImportSpec {
            spqv_path: tmp("dupid.spqv"),
            dim: 4,
            ids: &ids,
            binding: ImportBinding::Unbound,
        },
        &npy,
    ) else {
        panic!("duplicate id must fail closed");
    };
    assert!(
        err.contains("already has a vector") || err.contains("id"),
        "err: {err}"
    );
}
