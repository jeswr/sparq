//! Round-trip validation: an HDT archive loaded via `sparq_hdt::load` must yield
//! EXACTLY the triple set sparq's own N-Triples loader produces from the same data.

use sparq_core::Graph;
use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

fn fixture(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures").join(name)
}

/// Dumps a graph as a canonical, comparable triple set (N-Triples term rendering).
fn triple_set(g: &Graph) -> BTreeSet<[String; 3]> {
    let scan = g.store.scan(&[None, None, None]);
    scan.rows
        .iter()
        .map(|r| {
            let [s, p, o] = scan.to_spo(r);
            [g.dict.term(s).to_string(), g.dict.term(p).to_string(), g.dict.term(o).to_string()]
        })
        .collect()
}

/// snikmeta.hdt is a real-world archive vendored from the `hdt` crate's test suite
/// (MIT; SNIK meta ontology) — i.e. NOT produced by this code path. Its N-Triples
/// ground truth (snikmeta.nt, 328 triples — the same count the `hdt` crate's own
/// tests assert) must load to the identical triple set.
#[test]
fn snikmeta_hdt_matches_ntriples() {
    let hdt_path = fixture("snikmeta.hdt");
    if !hdt_path.exists() {
        eprintln!("skipping: fixture {} absent", hdt_path.display());
        return;
    }
    let from_hdt = sparq_hdt::load(&hdt_path).expect("loading snikmeta.hdt");
    let nt = std::fs::read_to_string(fixture("snikmeta.nt")).expect("reading snikmeta.nt");
    let from_nt = Graph::load_str(&nt, "ntriples").expect("loading snikmeta.nt");

    let h = triple_set(&from_hdt);
    let n = triple_set(&from_nt);
    assert_eq!(h.len(), 328, "snikmeta is 328 distinct triples");
    assert_eq!(h, n, "HDT and N-Triples loads must agree triple-for-triple");
    assert_eq!(from_hdt.store.len(), from_nt.store.len());
}

/// Full binary round trip with the term zoo: N-Triples -> HDT archive (via the
/// hdt crate's writer) -> `sparq_hdt::load` -> compare against sparq's own load
/// of the SOURCE N-Triples. Covers IRIs, blank nodes, plain / language-tagged /
/// datatyped literals, unicode, integers (sparq's inline-literal path), and terms
/// shared between subject and object position (HDT's shared dictionary section).
#[test]
fn generated_hdt_round_trips() {
    let nt = concat!(
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

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sparq-hdt-roundtrip");
    std::fs::create_dir_all(&dir).unwrap();
    let nt_path = dir.join("zoo.nt");
    let hdt_path = dir.join("zoo.hdt");
    std::fs::write(&nt_path, nt).unwrap();

    // N-Triples -> HDT archive on disk (FourSectDict PFC + BitmapTriples).
    let written = hdt::Hdt::read_nt(&nt_path).expect("building HDT from N-Triples");
    let mut out = std::io::BufWriter::new(std::fs::File::create(&hdt_path).unwrap());
    written.write(&mut out).expect("writing HDT archive");
    drop(out);

    let from_hdt = sparq_hdt::load(&hdt_path).expect("loading generated HDT");
    let from_nt = Graph::load_str(nt, "ntriples").unwrap();

    assert_eq!(triple_set(&from_hdt), triple_set(&from_nt));
    assert_eq!(from_hdt.store.len(), 11);
}

/// `load_reader` is the same decode from any buffered source.
#[test]
fn load_reader_from_memory() {
    let hdt_path = fixture("snikmeta.hdt");
    if !hdt_path.exists() {
        eprintln!("skipping: fixture {} absent", hdt_path.display());
        return;
    }
    let bytes = std::fs::read(&hdt_path).unwrap();
    let g = sparq_hdt::load_reader(std::io::Cursor::new(bytes)).unwrap();
    assert_eq!(g.store.len(), 328);
}

/// Garbage input must error, not panic.
#[test]
fn rejects_non_hdt_input() {
    let r = sparq_hdt::load_reader(std::io::Cursor::new(b"not an hdt file".to_vec()));
    assert!(r.is_err());
}

/// GZips the fixture in memory (single member; the decoder also accepts
/// multi-member streams).
fn gzipped_snikmeta() -> Option<Vec<u8>> {
    use std::io::Write;
    let hdt_path = fixture("snikmeta.hdt");
    if !hdt_path.exists() {
        eprintln!("skipping: fixture {} absent", hdt_path.display());
        return None;
    }
    let bytes = std::fs::read(&hdt_path).unwrap();
    let mut enc = flate2::write::GzEncoder::new(Vec::new(), flate2::Compression::fast());
    enc.write_all(&bytes).unwrap();
    Some(enc.finish().unwrap())
}

/// `.hdt.gz` containers are detected by MAGIC BYTES (not file name) and
/// decompressed transparently, from both a path and a reader.
#[test]
fn gzipped_hdt_loads_transparently() {
    let Some(gz) = gzipped_snikmeta() else { return };
    // Through a reader…
    let g = sparq_hdt::load_reader(std::io::Cursor::new(gz.clone())).unwrap();
    assert_eq!(g.store.len(), 328);
    assert_eq!(triple_set(&g), triple_set(&sparq_hdt::load(fixture("snikmeta.hdt")).unwrap()));
    // …and through a path with NO .gz extension (content sniffing, not names).
    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sparq-hdt-gz");
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("disguised.hdt");
    std::fs::write(&path, &gz).unwrap();
    assert_eq!(sparq_hdt::load(&path).unwrap().store.len(), 328);
}

/// The HDT header (dataset metadata triples) is exposed as a queryable graph,
/// for plain and gzipped archives alike.
#[test]
fn header_exposes_dataset_metadata() {
    let hdt_path = fixture("snikmeta.hdt");
    if !hdt_path.exists() {
        eprintln!("skipping: fixture {} absent", hdt_path.display());
        return;
    }
    let h = sparq_hdt::header(&hdt_path).unwrap();
    assert!(!h.is_empty(), "snikmeta carries header metadata");
    // The VoID statistics agree with the actual triple count.
    let triples: Vec<[String; 3]> = triple_set(&h)
        .into_iter()
        .filter(|[_, p, _]| p == "<http://rdfs.org/ns/void#triples>")
        .collect();
    assert_eq!(triples.len(), 1);
    assert_eq!(triples[0][2], "\"328\"");
    // Same through a gzipped stream.
    if let Some(gz) = gzipped_snikmeta() {
        let h2 = sparq_hdt::header_reader(std::io::Cursor::new(gz)).unwrap();
        assert_eq!(triple_set(&h2), triple_set(&h));
    }
}

// ============================================================================
// [OPUS-4.8] Differential correctness: the direct sparq-side decoder (H1–H4,
// the default `load_reader` path) must produce EXACTLY the same triple set AND
// the same number of distinct dictionary terms as the upstream-backed path
// (`Hdt::read` + per-id `id_to_string` translation), on the same bytes. This is
// the load-bearing gate from the optimization plan's HDT §oracle.
// ============================================================================

/// Loads `bytes` both ways (direct decoder vs upstream oracle) and asserts the two
/// graphs are triple-for-triple identical and have the same distinct-term count.
fn assert_direct_matches_upstream(bytes: &[u8], min_triples: usize) {
    let direct = sparq_hdt::load_reader(std::io::Cursor::new(bytes.to_vec()))
        .expect("direct decoder load");
    let upstream = sparq_hdt::load_reader_via_upstream(std::io::Cursor::new(bytes.to_vec()))
        .expect("upstream-oracle load");

    let d = triple_set(&direct);
    let u = triple_set(&upstream);
    assert!(d.len() >= min_triples, "expected >= {min_triples} triples, got {}", d.len());
    assert_eq!(d, u, "direct decoder and upstream oracle must agree triple-for-triple");
    assert_eq!(direct.store.len(), upstream.store.len(), "same stored triple count");
    // The id-translation must intern the same set of distinct terms. The exact
    // sparq ids depend only on first-appearance order, which is identical because
    // both walk SPO order; comparing distinct-term counts plus the triple-set
    // equality above pins the dictionary translation.
    assert_eq!(direct.dict.len(), upstream.dict.len(), "same number of distinct interned terms");
}

/// snikmeta.hdt: a real archive (NOT produced by our writer), 328 triples, whose
/// object section (133 strings, block_size 16) already spans MULTIPLE PFC blocks
/// and whose shared section (43 strings) exercises the shared subject/object range.
#[test]
fn direct_decoder_matches_upstream_snikmeta() {
    let hdt_path = fixture("snikmeta.hdt");
    if !hdt_path.exists() {
        eprintln!("skipping: fixture {} absent", hdt_path.display());
        return;
    }
    let bytes = std::fs::read(&hdt_path).unwrap();
    assert_direct_matches_upstream(&bytes, 328);
}

/// Builds an HDT archive in-process from N-Triples designed to stress the levers:
///  * a SHARED-section-heavy graph: every entity is used as both subject and object
///    (`eN knows eN+1`), so almost all IRIs land in the shared section;
///  * MULTI-BLOCK PFC: > 16 distinct strings per section (block_size 16) so the
///    block-sequential decode (H3) crosses block boundaries with running prefixes;
///  * a term zoo across blocks: langtag/datatyped/plain literals, blank nodes.
///
/// Then asserts the direct decoder == upstream oracle on those exact bytes.
#[test]
fn direct_decoder_matches_upstream_generated_multiblock() {
    use std::fmt::Write as _;
    let mut nt = String::new();
    const N: usize = 200; // >> block_size 16 -> many blocks in every section
    let knows = "<http://xmlns.com/foaf/0.1/knows>";
    let label = "<http://www.w3.org/2000/01/rdf-schema#label>";
    let age = "<http://example.org/age>";
    let height = "<http://example.org/height>";
    for i in 0..N {
        // chain: e{i} knows e{i+1} -> e{i} appears as both subject and object (shared)
        writeln!(nt, "<http://example.org/e{i}> {knows} <http://example.org/e{}> .", i + 1).unwrap();
        // language-tagged + datatyped + plain literals, object-only section, many blocks
        writeln!(nt, "<http://example.org/e{i}> {label} \"name {i}\"@en .").unwrap();
        writeln!(nt, "<http://example.org/e{i}> {age} \"{i}\"^^<http://www.w3.org/2001/XMLSchema#integer> .").unwrap();
        writeln!(nt, "<http://example.org/e{i}> {height} \"{i}.5\"^^<http://www.w3.org/2001/XMLSchema#decimal> .").unwrap();
    }
    // A couple of blank nodes (subject + object positions).
    writeln!(nt, "_:anon0 {knows} <http://example.org/e0> .").unwrap();
    writeln!(nt, "<http://example.org/e0> {knows} _:anon1 .").unwrap();

    let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sparq-hdt-multiblock");
    std::fs::create_dir_all(&dir).unwrap();
    let nt_path = dir.join("multiblock.nt");
    std::fs::write(&nt_path, &nt).unwrap();

    let written = hdt::Hdt::read_nt(&nt_path).expect("building multi-block HDT");
    let mut buf: Vec<u8> = Vec::new();
    written.write(&mut buf).expect("serializing HDT");

    // Sanity: confirm the fixture really is multi-block + shared-heavy.
    assert!(written.dict.objects.num_strings > written.dict.objects.block_size, "objects must span >1 block");
    assert!(written.dict.shared.num_strings > written.dict.shared.block_size, "shared must span >1 block");

    assert_direct_matches_upstream(&buf, 4 * N);

    // The graph must also equal sparq's own N-Triples load of the SAME source.
    let from_direct = sparq_hdt::load_reader(std::io::Cursor::new(buf)).unwrap();
    let from_nt = Graph::load_str(&nt, "ntriples").unwrap();
    assert_eq!(triple_set(&from_direct), triple_set(&from_nt));
}

/// The direct decoder over a `.hdt.gz` stream yields the same triple set as over
/// the plain bytes (the streaming gzip path feeds the same decoder).
#[test]
fn direct_decoder_gzip_matches_plain() {
    let Some(gz) = gzipped_snikmeta() else { return };
    let from_gz = sparq_hdt::load_reader(std::io::Cursor::new(gz)).unwrap();
    let from_plain = sparq_hdt::load(fixture("snikmeta.hdt")).unwrap();
    assert_eq!(triple_set(&from_gz), triple_set(&from_plain));
    assert_eq!(from_gz.store.len(), from_plain.store.len());
}

/// Empty + single-triple HDT archives decode identically on both paths (exercises
/// the zero-section and single-block-of-one edge of the PFC + SPO walk).
#[test]
fn direct_decoder_matches_upstream_tiny() {
    for nt in [
        "", // empty graph: zero strings in every section, zero triples
        "<http://example.org/s> <http://example.org/p> <http://example.org/o> .\n",
    ] {
        let dir = Path::new(env!("CARGO_TARGET_TMPDIR")).join("sparq-hdt-tiny");
        std::fs::create_dir_all(&dir).unwrap();
        let nt_path = dir.join("tiny.nt");
        std::fs::write(&nt_path, nt).unwrap();
        let written = hdt::Hdt::read_nt(&nt_path).expect("building tiny HDT");
        let mut buf: Vec<u8> = Vec::new();
        written.write(&mut buf).expect("serializing tiny HDT");

        let direct = sparq_hdt::load_reader(std::io::Cursor::new(buf.clone())).unwrap();
        let upstream = sparq_hdt::load_reader_via_upstream(std::io::Cursor::new(buf)).unwrap();
        assert_eq!(triple_set(&direct), triple_set(&upstream));
        assert_eq!(direct.store.len(), upstream.store.len());
    }
}
