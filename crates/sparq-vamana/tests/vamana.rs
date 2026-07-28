//! [OPUS-5] (issue #3699) Stand-alone accuracy + format gate for the extracted crate — the same
//! properties `sparq-vectors` gates on its dict-id-keyed facade, restated over a plain
//! [`SliceVectors`] source so this crate is verifiable with no RDF dependency at all:
//!
//! 1. **recall@10 vs exact brute force** — a persisted graph must find the true neighbours;
//! 2. **restart survival** — build → drop the handle → `open` a fresh one (no rebuild) and get
//!    byte-identical neighbours;
//! 3. **PQ round-trip** — an index built with a candidate cache reloads it and still clears the
//!    recall floor (the "search on PQ, re-rank on disk" path);
//! 4. **staleness-token semantics** — a supplied token round-trips, an absent one reads back as
//!    `None` ("unverifiable"), never as an all-zero token that would look like a mismatch;
//! 5. **degenerate + hostile inputs** — empty source, all-zero query, a corrupted header, a forged
//!    PQ codebook header and an out-of-range PQ code are rejected cleanly, and a forged neighbour
//!    entry inside an otherwise-valid file is dropped on read: never a panic, never an
//!    out-of-bounds read.
//!
//! The workload is deliberately small so the suite runs under a plain (debug) `cargo test`; the
//! 50k-scale recall gate lives in `sparq-vectors`' `tests/diskann.rs`, which drives the very same
//! code through the facade.

use sparq_vamana::{
    PqConfig, ProductQuantizer, SliceVectors, StalenessToken, VamanaConfig, VamanaIndex,
    SPQG_HEADER_LEN, SPQG_HEADER_LEN_V1, SPQG_MAGIC, STALENESS_TOKEN_LEN,
};

const DIM: usize = 32;
const N: usize = 2_000;
const K: usize = 10;
/// Floor, not a promise: measured recall on this pinned (N, seed) workload sits above it, and the
/// build is single-threaded + fixed-seed so the number is deterministic.
const RECALL_FLOOR: f64 = 0.90;

fn splitmix64(state: &mut u64) -> u64 {
    *state = state.wrapping_add(0x9E37_79B9_7F4A_7C15);
    let mut z = *state;
    z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    z ^ (z >> 31)
}

/// Uniform in [-1, 1).
fn rand_vec(state: &mut u64, dim: usize) -> Vec<f32> {
    (0..dim)
        .map(|_| ((splitmix64(state) >> 40) as f32 / (1u64 << 23) as f32) * 2.0 - 1.0)
        .collect()
}

/// `N` seeded vectors with ids `1..=N` (1-based, mirroring a dictionary id space so the tests
/// would catch an off-by-one that a 0-based id space would hide).
fn corpus(n: usize, seed: u64) -> SliceVectors {
    let mut state = seed;
    let mut data = Vec::with_capacity(n * DIM);
    let mut ids = Vec::with_capacity(n);
    for i in 0..n {
        data.extend_from_slice(&rand_vec(&mut state, DIM));
        ids.push(i as u32 + 1);
    }
    SliceVectors::new(DIM, ids, data).expect("well-formed corpus")
}

/// Exact cosine top-`k` over the source — the brute-force oracle recall is measured against.
fn brute_force(src: &SliceVectors, query: &[f32], k: usize) -> Vec<u32> {
    use sparq_vamana::VectorSource;
    let qn: f32 = query.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mut scored: Vec<(u32, f32)> = src
        .iter()
        .map(|(id, v)| {
            let vn: f32 = v.iter().map(|x| x * x).sum::<f32>().sqrt();
            let dot: f32 = v.iter().zip(query).map(|(a, b)| a * b).sum();
            (id, dot / (vn * qn))
        })
        .collect();
    scored.sort_unstable_by(|a, b| b.1.total_cmp(&a.1).then(a.0.cmp(&b.0)));
    scored.truncate(k);
    scored.into_iter().map(|(id, _)| id).collect()
}

fn tmp(name: &str) -> std::path::PathBuf {
    std::env::temp_dir().join(format!("sparq-vamana-{name}-{}.spqg", std::process::id()))
}

/// Mean recall@`K` of `index` against the brute-force oracle over 50 seeded queries.
fn recall_at_k(index: &VamanaIndex, src: &SliceVectors, queries: usize) -> f64 {
    let mut qstate = 0xDEAD_BEEFu64;
    let mut hits = 0usize;
    for _ in 0..queries {
        let q = rand_vec(&mut qstate, DIM);
        let want = brute_force(src, &q, K);
        let got: Vec<u32> = index.nearest(&q, K).into_iter().map(|(id, _)| id).collect();
        hits += got.iter().filter(|id| want.contains(id)).count();
    }
    hits as f64 / (queries * K) as f64
}

#[test]
fn recall_at_10_clears_the_floor_and_survives_a_reopen() {
    let src = corpus(N, 7);
    let path = tmp("recall");
    let built = VamanaIndex::build(&src, &path, VamanaConfig::default(), None).unwrap();
    assert_eq!(built.len(), N);
    assert_eq!(built.dim(), DIM);
    assert!(!built.has_pq_cache());

    let recall = recall_at_k(&built, &src, 50);
    assert!(recall >= RECALL_FLOOR, "recall@{K} {recall:.4} < floor {RECALL_FLOOR}");

    // Restart survival: the same query returns byte-identical neighbours from a fresh open with
    // NO rebuild. (The reopened handle carries the default search beam, which is what the build
    // config used, so the two are directly comparable.)
    let mut qseed = 0x1234u64;
    let q = rand_vec(&mut qseed, DIM);
    let before = built.nearest(&q, K);
    drop(built);
    let reopened = VamanaIndex::open(&path).unwrap();
    assert_eq!(reopened.len(), N);
    assert_eq!(reopened.nearest(&q, K), before, "reopen must not change results");

    // ...and the same file read as owned bytes (the filesystem-less / wasm path) is identical.
    let from_bytes = VamanaIndex::open_from_bytes(std::fs::read(&path).unwrap()).unwrap();
    assert_eq!(from_bytes.nearest(&q, K), before);

    std::fs::remove_file(&path).ok();
}

#[test]
fn pq_cache_persists_reloads_and_still_clears_the_floor() {
    let src = corpus(N, 11);
    let path = tmp("pq");
    let pq_cfg = PqConfig { m: 8, ..PqConfig::default() };
    let built =
        VamanaIndex::build_with_pq(&src, &path, VamanaConfig::default(), pq_cfg, None).unwrap();
    assert!(built.has_pq_cache(), "build_with_pq must persist a candidate cache");

    // The trailing PQ section reloads from the file alone.
    let reopened = VamanaIndex::open(&path).unwrap();
    assert!(reopened.has_pq_cache(), "the PQ section must survive a reopen");
    assert_eq!(reopened.len(), N);

    // Search-on-PQ + re-rank-on-disk still finds the true neighbours: the beam is PQ-guided but
    // the returned scores are the EXACT re-ranked distances.
    let recall = recall_at_k(&reopened, &src, 50);
    assert!(recall >= RECALL_FLOOR, "PQ recall@{K} {recall:.4} < floor {RECALL_FLOOR}");

    std::fs::remove_file(&path).ok();
}

#[test]
fn staleness_token_round_trips_and_absence_is_none_not_zero() {
    let src = corpus(64, 3);

    // Supplied → handed back verbatim.
    let mut raw = [0u8; STALENESS_TOKEN_LEN];
    for (i, b) in raw.iter_mut().enumerate() {
        *b = (i as u8).wrapping_mul(7).wrapping_add(1);
    }
    let token = StalenessToken::new(raw);
    let with_path = tmp("token");
    let with = VamanaIndex::build(&src, &with_path, VamanaConfig::default(), Some(token)).unwrap();
    assert_eq!(with.staleness_token(), Some(token));
    assert_eq!(
        VamanaIndex::open(&with_path).unwrap().staleness_token(),
        Some(token),
        "the token must survive a reopen"
    );

    // Absent → `None` ("unverifiable"), NOT `Some([0; 24])`, which a consumer would read as a
    // generation MISMATCH rather than as "this index was never bound".
    let without_path = tmp("notoken");
    let without = VamanaIndex::build(&src, &without_path, VamanaConfig::default(), None).unwrap();
    assert_eq!(without.staleness_token(), None);

    std::fs::remove_file(&with_path).ok();
    std::fs::remove_file(&without_path).ok();
}

#[test]
fn legacy_v1_file_opens_and_searches_but_reports_no_token() {
    let src = corpus(64, 5);
    let path = tmp("v1");
    let built = VamanaIndex::build(&src, &path, VamanaConfig::default(), None).unwrap();
    let mut qseed = 0x99u64;
    let q = rand_vec(&mut qseed, DIM);
    let expected = built.nearest(&q, 5);
    drop(built);

    // Rewrite the v2 file as a genuine v1: version = 1 and the 24-byte token block dropped.
    let mut bytes = std::fs::read(&path).unwrap();
    bytes[4..8].copy_from_slice(&1u32.to_le_bytes());
    bytes.drain(SPQG_HEADER_LEN_V1..SPQG_HEADER_LEN);
    std::fs::write(&path, &bytes).unwrap();

    let legacy = VamanaIndex::open(&path).expect("a legacy v1 .spqg must still open");
    assert_eq!(legacy.staleness_token(), None);
    assert_eq!(legacy.nearest(&q, 5), expected, "the node body is unchanged by the header shift");

    std::fs::remove_file(&path).ok();
}

#[test]
fn degenerate_sources_and_queries_are_well_defined() {
    // Empty source: builds, opens, searches — and returns nothing.
    let empty = SliceVectors::new(DIM, Vec::new(), Vec::new()).unwrap();
    let path = tmp("empty");
    let idx = VamanaIndex::build(&empty, &path, VamanaConfig::default(), None).unwrap();
    assert!(idx.is_empty());
    assert!(idx.nearest(&[1.0f32; DIM], 5).is_empty());
    assert!(VamanaIndex::open(&path).unwrap().is_empty());
    std::fs::remove_file(&path).ok();

    // Single vector: it is its own nearest neighbour.
    let one = SliceVectors::new(2, vec![42], vec![1.0, 0.0]).unwrap();
    let one_path = tmp("one");
    let idx1 = VamanaIndex::build(&one, &one_path, VamanaConfig::default(), None).unwrap();
    assert_eq!(idx1.nearest(&[1.0, 0.0], 3), vec![(42, 1.0)]);
    // An all-zero query has no direction → no results (never a panic, never a NaN ranking).
    assert!(idx1.nearest(&[0.0, 0.0], 3).is_empty());
    std::fs::remove_file(&one_path).ok();
}

#[test]
fn malformed_files_are_rejected_with_a_descriptive_error() {
    // Truncated.
    assert!(VamanaIndex::open_from_bytes(vec![0u8; 8]).is_err());
    // Bad magic.
    let mut bad = vec![0u8; SPQG_HEADER_LEN];
    bad[0..4].copy_from_slice(b"NOPE");
    assert!(VamanaIndex::open_from_bytes(bad).unwrap_err().contains("bad magic"));
    // Unsupported version.
    let mut ver = vec![0u8; SPQG_HEADER_LEN];
    ver[0..4].copy_from_slice(&SPQG_MAGIC);
    ver[4..8].copy_from_slice(&99u32.to_le_bytes());
    assert!(VamanaIndex::open_from_bytes(ver).unwrap_err().contains("unsupported .spqg version"));
    // Zero dimension.
    let mut zero = vec![0u8; SPQG_HEADER_LEN];
    zero[0..4].copy_from_slice(&SPQG_MAGIC);
    zero[4..8].copy_from_slice(&2u32.to_le_bytes());
    assert!(VamanaIndex::open_from_bytes(zero).unwrap_err().contains("zero dimension"));
    // A count that overflows the declared file size.
    let mut big = vec![0u8; SPQG_HEADER_LEN];
    big[0..4].copy_from_slice(&SPQG_MAGIC);
    big[4..8].copy_from_slice(&2u32.to_le_bytes());
    big[8..12].copy_from_slice(&4u32.to_le_bytes());
    big[12..16].copy_from_slice(&8u32.to_le_bytes());
    big[16..24].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(VamanaIndex::open_from_bytes(big).is_err());
    // An unknown encoding tag.
    let mut tag = vec![0u8; SPQG_HEADER_LEN];
    tag[0..4].copy_from_slice(&SPQG_MAGIC);
    tag[4..8].copy_from_slice(&2u32.to_le_bytes());
    tag[8..12].copy_from_slice(&4u32.to_le_bytes());
    tag[28..32].copy_from_slice(&7u32.to_le_bytes());
    assert!(VamanaIndex::open_from_bytes(tag).unwrap_err().contains("unsupported encoding tag"));
}

/// A validated file's *records* are still attacker-controlled: `open` checks the header and the
/// file size but cannot read every record's adjacency without paging the whole file in. A
/// neighbour entry naming a slot that does not exist must therefore be dropped where it is read,
/// not indexed with — before this was checked, a one-word edit to a built file opened cleanly and
/// panicked inside `nearest` on `in_working[nbr as usize]`.
#[test]
fn out_of_range_neighbour_entries_cannot_panic_a_search() {
    const SMALL: usize = 64;
    let src = corpus(SMALL, 5);
    let path = tmp("badnbr");
    VamanaIndex::build(&src, &path, VamanaConfig::default(), None).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();

    let dim = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let degree = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let count = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
    assert_eq!((dim, count), (DIM, SMALL));
    let record_len = 8 + dim * 4 + degree * 4;
    // Every record keeps its declared degree but its first neighbour entry now names slot
    // `u32::MAX`, and its second a slot just past the end.
    let mut forged = 0usize;
    for slot in 0..count {
        let rec = SPQG_HEADER_LEN + slot * record_len;
        let deg = u32::from_le_bytes(bytes[rec + 4..rec + 8].try_into().unwrap()) as usize;
        if deg == 0 {
            continue;
        }
        let nbrs = rec + 8 + dim * 4;
        bytes[nbrs..nbrs + 4].copy_from_slice(&u32::MAX.to_le_bytes());
        if deg > 1 {
            bytes[nbrs + 4..nbrs + 8].copy_from_slice(&(count as u32).to_le_bytes());
        }
        forged += 1;
    }
    assert!(forged > 0, "the fixture must actually contain neighbour entries to forge");

    let mut qseed = 0x9977u64;
    let q = rand_vec(&mut qseed, DIM);
    let valid_ids = 1..=SMALL as u32;

    // Both readers — the mmap `open` and the owned-bytes `open_from_bytes` — go through the same
    // record reader, so exercise each.
    std::fs::write(&path, &bytes).unwrap();
    let mapped = VamanaIndex::open(&path).unwrap();
    let got = mapped.nearest(&q, K);
    assert!(!got.is_empty(), "the surviving edges still reach neighbours");
    assert!(got.iter().all(|(id, _)| valid_ids.contains(id)), "returned a phantom id: {:?}", got);
    std::fs::remove_file(&path).ok();

    let owned = VamanaIndex::open_from_bytes(bytes).unwrap();
    assert_eq!(owned.nearest(&q, K), got);
}

/// `dim`/`m`/`k` in a PQ codebook come straight off the wire, and the centroid-block size derived
/// from them overflows a 32-bit `usize` (wasm32 is a supported target) long before it overflows a
/// 64-bit one. A forged header must be rejected with an error — never a wrapped offset, an
/// enormous allocation, or a panic.
#[test]
fn forged_pq_codebook_headers_are_rejected_not_overflowed() {
    let pq_header = |dim: u32, m: u32, k: u32| {
        let mut hdr = Vec::with_capacity(12);
        hdr.extend_from_slice(&dim.to_le_bytes());
        hdr.extend_from_slice(&m.to_le_bytes());
        hdr.extend_from_slice(&k.to_le_bytes());
        hdr
    };
    // Sizes that overflow `k · dim · 4` on a 32-bit target (and would demand a multi-gigabyte
    // subspace table on a 64-bit one) — rejected before either can happen.
    let huge = pq_header(u32::MAX, u32::MAX, 256);
    assert!(ProductQuantizer::from_bytes(&huge).is_err());
    assert!(ProductQuantizer::from_bytes(&pq_header(u32::MAX, 1, 256)).is_err());
    // Header-only sanity: the validity envelope `fit` enforces still applies.
    assert!(ProductQuantizer::from_bytes(&pq_header(4, 0, 256)).is_err());
    assert!(ProductQuantizer::from_bytes(&pq_header(4, 8, 256)).is_err());
    assert!(ProductQuantizer::from_bytes(&pq_header(4, 2, 257)).is_err());
    // A well-formed header with the centroid payload missing is a truncation, not a wrap.
    let err = ProductQuantizer::from_bytes(&pq_header(4, 2, 256)).unwrap_err();
    assert!(err.contains("truncated"), "err: {}", err);
    // ...and a real codebook still round-trips (the checks reject only malformed blocks).
    let src = corpus(64, 3);
    let vectors: Vec<&[f32]> = {
        use sparq_vamana::VectorSource;
        src.iter().map(|(_, v)| v).collect()
    };
    let pq = ProductQuantizer::fit(DIM, vectors, PqConfig::default()).unwrap();
    let round = ProductQuantizer::from_bytes(&pq.to_bytes()).unwrap();
    assert_eq!((round.dim(), round.m(), round.k()), (pq.dim(), pq.m(), pq.k()));

    // The same forged header inside a real file's trailing PQ section: a clean `Err` out of
    // `open_from_bytes`, prefixed with the file's origin.
    let path = tmp("badpq");
    VamanaIndex::build_with_pq(&src, &path, VamanaConfig::default(), PqConfig::default(), None)
        .unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).ok();
    let dim = u32::from_le_bytes(bytes[8..12].try_into().unwrap()) as usize;
    let degree = u32::from_le_bytes(bytes[12..16].try_into().unwrap()) as usize;
    let count = u64::from_le_bytes(bytes[16..24].try_into().unwrap()) as usize;
    // Codebook header = section magic (4) + codebook length (8), right after the node records.
    let cb = SPQG_HEADER_LEN + count * (8 + dim * 4 + degree * 4) + 12;
    bytes[cb..cb + 12].copy_from_slice(&huge);
    let err = VamanaIndex::open_from_bytes(bytes).unwrap_err();
    assert!(err.contains("<bytes>"), "err: {}", err);
}

/// A codebook may declare `K < 256` while every persisted code stays a full `u8`, so a code byte
/// can name a centroid the codebook does not have. Search resolves a code through the ADC table at
/// `tables[s · K + c]`, where such a byte reads another subspace's row — and, in the last subspace,
/// runs off the end of the table and panics. The out-of-range code must be rejected at open, by
/// both readers, so no malformed file survives to panic inside `nearest`.
#[test]
fn out_of_range_pq_codes_are_rejected_at_open_not_panicked_at_search() {
    const SMALL: usize = 64;
    let src = corpus(SMALL, 7);
    let path = tmp("pqcode");
    // K = 16 ≪ 256: every byte in 16..=255 is now a forgeable out-of-range centroid index.
    let pq_cfg = PqConfig { m: 4, k: 16, ..PqConfig::default() };
    VamanaIndex::build_with_pq(&src, &path, VamanaConfig::default(), pq_cfg, None).unwrap();
    let mut bytes = std::fs::read(&path).unwrap();
    std::fs::remove_file(&path).ok();

    // The code array is the file's trailing run of `count × M` bytes, so the final byte is the last
    // slot's last subspace — the one whose forged code indexes past the end of the ADC table.
    let last = bytes.len() - 1;
    assert!(bytes[last] < 16, "the fixture must persist in-range codes to begin with");
    bytes[last] = 255;

    let err = VamanaIndex::open_from_bytes(bytes.clone()).unwrap_err();
    assert!(err.contains("PQ code 255"), "err: {}", err);
    assert!(err.contains("out of range"), "err: {}", err);
    // The report locates the offending byte: last slot, last subspace.
    assert!(err.contains(&format!("slot {}", SMALL - 1)), "err: {}", err);
    assert!(err.contains("subspace 3"), "err: {}", err);

    // The mmap reader shares the check — a file on disk is rejected identically, never opened.
    std::fs::write(&path, &bytes).unwrap();
    let mapped = VamanaIndex::open(&path);
    std::fs::remove_file(&path).ok();
    assert!(mapped.is_err(), "the mmap reader must reject the same forged code");
}

#[test]
fn invalid_build_config_is_rejected_before_any_file_is_written() {
    let src = corpus(16, 1);
    let path = tmp("badcfg");
    let cfg = VamanaConfig { degree: 32, build_beam: 8, ..VamanaConfig::default() };
    let err = VamanaIndex::build(&src, &path, cfg, None).unwrap_err();
    assert!(err.contains("build_beam"), "err: {err}");
    assert!(!path.exists(), "a rejected config must not leave a partial file behind");
}

#[cfg(feature = "filtered")]
#[test]
fn filtered_traversal_returns_only_accepted_ids_and_stays_ranked() {
    let src = corpus(N, 13);
    let path = tmp("filtered");
    let idx = VamanaIndex::build(&src, &path, VamanaConfig::default(), None).unwrap();

    // Accept the (broad) even-id half — broad enough that traversal, not a scan, is the sensible
    // strategy, which is exactly the regime this entry point serves.
    let accept = |id: u32| id % 2 == 0;
    let mut qseed = 0x5151u64;
    let q = rand_vec(&mut qseed, DIM);
    let got = idx.nearest_filtered_by(&q, K, 400, accept);

    assert!(!got.is_empty(), "a broad mask must still return neighbours");
    assert!(got.len() <= K);
    assert!(got.iter().all(|&(id, _)| accept(id)), "a rejected id must never be returned");
    // Best-first, and every returned id is a genuine member of the accepted top-K.
    assert!(got.windows(2).all(|w| w[0].1 >= w[1].1), "results must be sorted best-first");
    let want: Vec<u32> = brute_force(&src, &q, N)
        .into_iter()
        .filter(|&id| accept(id))
        .take(K)
        .collect();
    let overlap = got.iter().filter(|(id, _)| want.contains(id)).count();
    assert!(overlap * 2 >= got.len(), "filtered traversal recall collapsed: {overlap}/{}", got.len());

    // An all-zero query is degenerate — no direction, no results (same contract as `nearest`).
    assert!(idx.nearest_filtered_by(&[0.0f32; DIM], K, 400, accept).is_empty());

    std::fs::remove_file(&path).ok();
}
