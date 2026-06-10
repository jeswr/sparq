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
