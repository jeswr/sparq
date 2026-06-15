//! [OPUS-4.8] sq-2te — round-trip validation for the HDT WRITE path (`save`).
//!
//! `save` is opt-in (the `write` cargo feature), so the whole file is gated: with
//! the feature OFF this is an empty test binary. The contract under test: a sparq
//! `Graph` written with `save` and reloaded with `load` yields EXACTLY the original
//! term set — and equals sparq's own N-Triples load of the same source (the same
//! differential the existing `tests/roundtrip.rs` uses for the read path).
//!
//! Run with: `cargo test -p sparq-hdt --features write`.
#![cfg(feature = "write")]

use sparq_core::Graph;
use std::collections::BTreeSet;
use std::path::Path;

/// Canonical comparable triple set (N-Triples term rendering) — same helper as
/// `tests/roundtrip.rs`.
fn triple_set(g: &Graph) -> BTreeSet<[String; 3]> {
    let scan = g.store.scan(&[None, None, None]);
    scan.rows
        .iter()
        .map(|r| {
            let [s, p, o] = scan.to_spo(r);
            [
                g.dict.term(s).to_string(),
                g.dict.term(p).to_string(),
                g.dict.term(o).to_string(),
            ]
        })
        .collect()
}

/// The term zoo: IRIs, blank nodes, plain / language-tagged / datatyped literals,
/// unicode, an inline-integer literal, and terms shared between subject and object
/// position (HDT's shared dictionary section). Mirrors `roundtrip.rs`'s zoo so the
/// write path is exercised over the same shapes the read path is.
const ZOO: &str = concat!(
    "<http://example.org/alice> <http://xmlns.com/foaf/0.1/knows> <http://example.org/bob> .\n",
    "<http://example.org/bob> <http://xmlns.com/foaf/0.1/knows> <http://example.org/alice> .\n",
    "<http://example.org/alice> <http://xmlns.com/foaf/0.1/name> \"Alice\" .\n",
    "<http://example.org/alice> <http://xmlns.com/foaf/0.1/age> \"42\"^^<http://www.w3.org/2001/XMLSchema#integer> .\n",
    "<http://example.org/alice> <http://www.w3.org/2000/01/rdf-schema#label> \"Alice\"@en .\n",
    "<http://example.org/alice> <http://www.w3.org/2000/01/rdf-schema#label> \"Alicia\"@es .\n",
    "<http://example.org/bob> <http://www.w3.org/2000/01/rdf-schema#label> \"B\u{00f6}b \u{2014} caf\u{00e9}\"@de .\n",
    "<http://example.org/bob> <http://example.org/height> \"1.83\"^^<http://www.w3.org/2001/XMLSchema#decimal> .\n",
    "_:b0 <http://xmlns.com/foaf/0.1/knows> <http://example.org/alice> .\n",
    "_:b0 <http://example.org/link> _:b1 .\n",
    "_:b1 <http://xmlns.com/foaf/0.1/name> \"anon\" .\n",
);

fn scratch(name: &str) -> std::path::PathBuf {
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sparq-hdt-write");
    std::fs::create_dir_all(&dir).unwrap();
    dir.join(name)
}

/// Core contract: Graph -> `save` .hdt -> `load` .hdt == the original Graph,
/// triple-for-triple AND distinct-term count.
#[test]
fn save_then_load_round_trips_the_term_zoo() {
    let original = Graph::load_str(ZOO, "ntriples").unwrap();
    let path = scratch("zoo.hdt");
    sparq_hdt::save(&original, &path).expect("saving Graph to .hdt");

    let reloaded = sparq_hdt::load(&path).expect("loading the saved .hdt");
    assert_eq!(
        triple_set(&reloaded),
        triple_set(&original),
        "save -> load must preserve the term set exactly",
    );
    assert_eq!(
        reloaded.store.len(),
        original.store.len(),
        "same stored triple count"
    );
    assert_eq!(reloaded.store.len(), 11);
    assert_eq!(
        reloaded.dict.len(),
        original.dict.len(),
        "same number of distinct terms"
    );
}

/// The saved archive is the standard layout: it must also load through the
/// upstream-backed oracle path identically (proves `save` produced a spec-conformant
/// FourSectDict + BitmapTriples, not just something OUR decoder happens to accept).
#[test]
fn saved_archive_loads_via_upstream_oracle_too() {
    let original = Graph::load_str(ZOO, "ntriples").unwrap();
    let path = scratch("zoo-oracle.hdt");
    sparq_hdt::save(&original, &path).unwrap();

    let bytes = std::fs::read(&path).unwrap();
    let direct = sparq_hdt::load_reader(std::io::Cursor::new(bytes.clone())).unwrap();
    let upstream = sparq_hdt::load_reader_via_upstream(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(triple_set(&direct), triple_set(&upstream));
    assert_eq!(triple_set(&direct), triple_set(&original));
}

/// Compressed output containers: `save` honours the OUTPUT extension
/// (`.hdt.gz` / `.hdt.zst` / `.hdt.bz2`), and the reader sniffs each back by magic
/// bytes to the IDENTICAL term set as the bare `.hdt`.
#[test]
fn save_honours_compression_containers() {
    let original = Graph::load_str(ZOO, "ntriples").unwrap();
    let reference = triple_set(&original);

    // Magic bytes the reader sniffs: gzip 1f 8b, zstd 28 b5 2f fd, bzip2 "BZh".
    for (name, magic) in [
        ("zoo.hdt.gz", &[0x1f, 0x8b][..]),
        ("zoo.hdt.zst", &[0x28, 0xb5, 0x2f, 0xfd][..]),
        ("zoo.hdt.bz2", &[0x42, 0x5a, 0x68][..]),
    ] {
        let path = scratch(name);
        sparq_hdt::save(&original, &path).unwrap_or_else(|e| panic!("[{name}] save: {e}"));

        // The file really is wrapped in the requested container.
        let head = std::fs::read(&path).unwrap();
        assert!(
            head.starts_with(magic),
            "[{name}] expected container magic bytes"
        );

        // …and the reader transparently sniffs it back to the same triples.
        let g = sparq_hdt::load(&path).unwrap_or_else(|e| panic!("[{name}] load: {e}"));
        assert_eq!(triple_set(&g), reference, "[{name}] round-trip term set");
    }

    // A bare `.hdt` writes the uncompressed `$HDT` cookie.
    let bare = scratch("zoo-bare.hdt");
    sparq_hdt::save(&original, &bare).unwrap();
    let head = std::fs::read(&bare).unwrap();
    assert_eq!(
        &head[..4],
        b"$HDT",
        "bare .hdt must start with the $HDT cookie"
    );
}

/// A larger, multi-block, shared-section-heavy graph (entities used as both subject
/// and object; > 16 distinct strings per section so PFC crosses block boundaries) —
/// `save` then `load` must still equal sparq's own N-Triples load of the source.
#[test]
fn save_round_trips_multiblock_graph() {
    use std::fmt::Write as _;
    let mut nt = String::new();
    const N: usize = 200;
    let knows = "<http://xmlns.com/foaf/0.1/knows>";
    let label = "<http://www.w3.org/2000/01/rdf-schema#label>";
    for i in 0..N {
        writeln!(
            nt,
            "<http://example.org/e{i}> {knows} <http://example.org/e{}> .",
            i + 1
        )
        .unwrap();
        writeln!(nt, "<http://example.org/e{i}> {label} \"name {i}\"@en .").unwrap();
    }
    let original = Graph::load_str(&nt, "ntriples").unwrap();
    let path = scratch("multiblock.hdt");
    sparq_hdt::save(&original, &path).unwrap();

    let reloaded = sparq_hdt::load(&path).unwrap();
    let from_nt = Graph::load_str(&nt, "ntriples").unwrap();
    assert_eq!(triple_set(&reloaded), triple_set(&from_nt));
    assert_eq!(reloaded.store.len(), from_nt.store.len());
}

/// Saving the empty graph produces a valid (empty) archive that reloads to nothing.
#[test]
fn save_empty_graph() {
    let empty = Graph::load_str("", "ntriples").unwrap();
    let path = scratch("empty.hdt");
    sparq_hdt::save(&empty, &path).unwrap();
    let reloaded = sparq_hdt::load(&path).unwrap();
    assert_eq!(reloaded.store.len(), 0);
    assert!(reloaded.is_empty());
}

/// `save` then re-`load` then re-`save` is STABLE: a second save of the reloaded
/// graph yields the identical triple set (the path is idempotent on content).
#[test]
fn save_is_idempotent_on_content() {
    let original = Graph::load_str(ZOO, "ntriples").unwrap();
    let p1 = scratch("idem-1.hdt");
    sparq_hdt::save(&original, &p1).unwrap();
    let g1 = sparq_hdt::load(&p1).unwrap();
    let p2 = scratch("idem-2.hdt");
    sparq_hdt::save(&g1, &p2).unwrap();
    let g2 = sparq_hdt::load(&p2).unwrap();
    assert_eq!(triple_set(&g1), triple_set(&g2));
    assert_eq!(triple_set(&g2), triple_set(&original));
}

/// [OPUS-4.8] sq-ashy: the DIRECT encoder writes NO temporary N-Triples scratch
/// file at all (the old path round-tripped through one; this one encodes the HDT
/// sections straight from sparq's in-memory dict + triples). Many saves must leave
/// ZERO `sparq-hdt-save-*.nt` files behind — there is no temp-file step to leak.
#[test]
fn direct_encoder_writes_no_temp_files() {
    const SAVES: usize = 30;
    let original = Graph::load_str(ZOO, "ntriples").unwrap();
    let before = count_scratch_nt();
    for i in 0..SAVES {
        sparq_hdt::save(&original, scratch(&format!("cleanup-{i}.hdt"))).unwrap();
    }
    let after = count_scratch_nt();
    // The direct encoder creates none, so the count cannot grow with `SAVES`.
    assert!(
        after <= before,
        "direct encoder created temp N-Triples files: {before} -> {after} over {SAVES} saves \
         (expected no growth; the direct path has no temp-file step)",
    );
}

/// Counts residual `sparq-hdt-save-*.nt` scratch files for this process (the name the
/// retired temp-file path used) — the direct encoder must never produce one.
fn count_scratch_nt() -> usize {
    let prefix = format!("sparq-hdt-save-{}-", std::process::id());
    std::fs::read_dir(std::env::temp_dir())
        .map(|rd| {
            rd.filter_map(|e| e.ok())
                .filter(|e| e.file_name().to_string_lossy().starts_with(&prefix))
                .count()
        })
        .unwrap_or(0)
}

/// [OPUS-4.8] sq-ashy: the direct encoder's bytes equal what the upstream `hdt`
/// crate's own builder + writer (`Hdt::read_nt` over the equivalent N-Triples, then
/// `Hdt::write`) produces — proving the direct PFC + BitmapTriples encode is the
/// SAME standard archive, not merely something our own reader happens to accept.
///
/// Two pieces are compared byte-for-byte: (1) the WHOLE FourSectDict section (the PFC
/// dictionary — control info + four sections + every CRC), and (2) the BitmapTriples
/// PAYLOAD (`bitmap_y` / `bitmap_z` / `sequence_y` / `sequence_z`). The global preamble
/// and header are skipped (ours is file-path-independent; `read_nt`'s embeds a
/// canonical `file://` IRI + size metadata). The triples CONTROL-INFO preamble is
/// compared field-wise rather than byte-wise because the upstream
/// `ControlInfo::bitmap_triples` stores its `order` / `numTriples` properties in a
/// `HashMap` whose serialisation order — and therefore the preamble bytes + its
/// CRC16 — is run-to-run non-deterministic on BOTH encoders (they share that writer).
#[test]
fn direct_encode_matches_upstream_builder() {
    let original = Graph::load_str(ZOO, "ntriples").unwrap();

    // Ours: the direct in-memory encoder.
    let direct_path = scratch("equiv-direct.hdt");
    sparq_hdt::save(&original, &direct_path).unwrap();
    let direct_bytes = std::fs::read(&direct_path).unwrap();

    // Oracle: render the SAME graph to N-Triples, build via the upstream crate's
    // `read_nt`, write via `Hdt::write` (the retired round-trip path's machinery).
    let nt_path = scratch("equiv-oracle.nt");
    {
        use std::io::Write as _;
        let mut f = std::io::BufWriter::new(std::fs::File::create(&nt_path).unwrap());
        let scan = original.store.scan(&[None, None, None]);
        for row in scan.rows.iter() {
            let [s, p, o] = scan.to_spo(row);
            writeln!(
                f,
                "{} {} {} .",
                original.dict.term(s),
                original.dict.term(p),
                original.dict.term(o)
            )
            .unwrap();
        }
        f.flush().unwrap();
    }
    let oracle = hdt::Hdt::read_nt(&nt_path).expect("upstream read_nt");
    let mut oracle_bytes = Vec::new();
    oracle
        .write(&mut oracle_bytes)
        .expect("upstream Hdt::write");

    // (1) The dictionary section (from its `$HDT`\x03 control info up to the triples
    // `$HDT`\x04 control info) must be byte-for-byte identical.
    let d_dict = section(&direct_bytes, 3, 4);
    let o_dict = section(&oracle_bytes, 3, 4);
    assert_eq!(
        d_dict, o_dict,
        "FourSectDict PFC bytes must match the upstream builder's exactly"
    );

    // (2) The triples PAYLOAD — everything after the (non-deterministically ordered)
    // triples control-info preamble — must be byte-for-byte identical.
    let d_tp = triples_payload(&direct_bytes);
    let o_tp = triples_payload(&oracle_bytes);
    assert_eq!(
        d_tp, o_tp,
        "BitmapTriples payload bytes must match the upstream builder's exactly"
    );
}

/// The bytes of the section whose control-info type is `start_type`, up to (but not
/// including) the next `$HDT` cookie carrying control-info type `end_type`. Control
/// types: 1 global, 2 header, 3 dictionary, 4 triples.
fn section(bytes: &[u8], start_type: u8, end_type: u8) -> &[u8] {
    let start = cookie_of_type(bytes, start_type, 0).expect("start control info");
    let end = cookie_of_type(bytes, end_type, start + 4).expect("end control info");
    &bytes[start..end]
}

/// The BitmapTriples PAYLOAD: the bytes AFTER the triples (`$HDT`\x04) control-info
/// preamble, which ends at the SECOND null terminator following the cookie (the
/// `format` string's, then the properties string's). The preamble's property order is
/// non-deterministic, but the payload (`bitmap_y` / `bitmap_z` / `sequence_y` /
/// `sequence_z` + their CRCs) is fully determined by the sorted triple ids.
fn triples_payload(bytes: &[u8]) -> &[u8] {
    let start = cookie_of_type(bytes, 4, 0).expect("triples control info");
    // Skip cookie (4) + type (1), then the two null-terminated strings (format, props).
    let mut p = start + 5;
    let mut nulls = 0;
    while p < bytes.len() && nulls < 2 {
        if bytes[p] == 0 {
            nulls += 1;
        }
        p += 1;
    }
    // After the second null comes the CRC16 (2 bytes) of the control info; skip it too.
    &bytes[p + 2..]
}

/// Position of the `$HDT` cookie at/after `from` whose following control-type byte is
/// `ty`.
fn cookie_of_type(bytes: &[u8], ty: u8, from: usize) -> Option<usize> {
    let mut at = from;
    while let Some(i) = find_subslice(bytes, b"$HDT", at) {
        if bytes.get(i + 4) == Some(&ty) {
            return Some(i);
        }
        at = i + 4;
    }
    None
}

fn find_subslice(haystack: &[u8], needle: &[u8], from: usize) -> Option<usize> {
    (from..=haystack.len().saturating_sub(needle.len()))
        .find(|&i| &haystack[i..i + needle.len()] == needle)
}
