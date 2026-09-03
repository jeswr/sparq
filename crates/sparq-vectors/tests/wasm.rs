// [GPT-5.2-CODEX] sq-139hh — genuine wasm32 runtime coverage for the filesystem-less readers.
//
// The writers are intentionally filesystem-based, so this test constructs tiny version-2
// fixtures from the documented stable `.spqv` / `.spqg` layouts. That keeps the wasm test free of
// host filesystem setup while exercising the production validators, aligned owned-byte backing,
// exact scan, graph traversal, and f32 record reads end to end under `wasm-pack test --node`.

#![cfg(target_arch = "wasm32")]

use sparq_vectors::{nearest_exact, DiskAnnIndex, VectorStore};
use wasm_bindgen_test::*;

const DIM: u32 = 2;
const IDS: [u32; 3] = [7, 42, 99];
const VECTORS: [[f32; 2]; 3] = [[1.0, 0.0], [0.0, 1.0], [-1.0, 0.0]];

fn u32_le(out: &mut Vec<u8>, value: u32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn u64_le(out: &mut Vec<u8>, value: u64) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn f32_le(out: &mut Vec<u8>, value: f32) {
    out.extend_from_slice(&value.to_le_bytes());
}

fn spqv_fixture() -> Vec<u8> {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SPQV");
    u32_le(&mut bytes, 2);
    u32_le(&mut bytes, DIM);
    u64_le(&mut bytes, IDS.len() as u64);
    bytes.extend_from_slice(&[0; 12]);
    bytes.extend_from_slice(&[0; 24]); // unbound graph fingerprint
    for vector in VECTORS {
        for value in vector {
            f32_le(&mut bytes, value);
        }
    }
    // The id index is sorted by id; insertion slots already follow that order here.
    for (slot, id) in IDS.into_iter().enumerate() {
        u32_le(&mut bytes, id);
        u32_le(&mut bytes, slot as u32);
    }
    bytes
}

fn spqg_fixture() -> Vec<u8> {
    const DEGREE: u32 = 2;
    let mut bytes = Vec::new();
    bytes.extend_from_slice(b"SPQG");
    u32_le(&mut bytes, 2);
    u32_le(&mut bytes, DIM);
    u32_le(&mut bytes, DEGREE);
    u64_le(&mut bytes, IDS.len() as u64);
    u32_le(&mut bytes, 0); // medoid slot
    u32_le(&mut bytes, 0); // no PQ cache
    bytes.extend_from_slice(&[0; 24]); // unbound graph fingerprint
    for (slot, (id, vector)) in IDS.into_iter().zip(VECTORS).enumerate() {
        u32_le(&mut bytes, id);
        u32_le(&mut bytes, DEGREE);
        for value in vector {
            f32_le(&mut bytes, value);
        }
        for neighbour in 0..IDS.len() {
            if neighbour != slot {
                u32_le(&mut bytes, neighbour as u32);
            }
        }
    }
    bytes
}

#[wasm_bindgen_test]
fn owned_vector_and_diskann_bytes_run_end_to_end() {
    let store = VectorStore::open_from_bytes(spqv_fixture()).expect("valid .spqv fixture");
    assert_eq!(store.dim(), DIM as usize);
    assert_eq!(store.len(), IDS.len());
    assert_eq!(store.get(42), Some(&[0.0_f32, 1.0][..]));

    let exact = nearest_exact(&store, &[1.0, 0.0], IDS.len());
    assert_eq!(exact.iter().map(|hit| hit.0).collect::<Vec<_>>(), IDS);

    let index = DiskAnnIndex::open_from_bytes(spqg_fixture()).expect("valid .spqg fixture");
    assert_eq!(index.dim(), DIM as usize);
    assert_eq!(index.len(), IDS.len());
    assert_eq!(index.nearest(&[1.0, 0.0], IDS.len()), exact);
}
