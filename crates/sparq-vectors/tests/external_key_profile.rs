//! [SONNET-4.6] sq-lhcot.2 (issue #2789) — the cross-repository conformance runner for the DRAFT
//! external-key `.spqv` interoperability profile (KERN boundary #1746).
//!
//! The corpus in `tests/fixtures/external-key/` was written by an INDEPENDENT stdlib-Python encoder
//! (`generate.py`), not by this crate, so these tests compare two implementations rather than
//! confirming one is self-consistent. Every accepted fixture is re-encoded and asserted
//! byte-identical to the committed bytes; every rejected fixture must fail with an error naming the
//! reason the manifest records.
//!
//! The manifest is the shared artifact: a Kern/PSS-side implementation reads `MANIFEST.tsv` and the
//! `.bin` files, never this file.

#![cfg(feature = "external-key")]

use sparq_vectors::external_key::{
    parse_multihash, ExternalKeyTable, EXTERNAL_KEY_PROFILE_VERSION,
};
use sparq_vectors::spqv_provenance::{Metric, Normalization};
use std::path::{Path, PathBuf};

fn corpus_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("tests/fixtures/external-key")
}

struct Row {
    fixture: String,
    accept: bool,
    entries: Option<usize>,
    hash_code: Option<u32>,
    key_len: Option<usize>,
    detail: String,
}

fn manifest() -> Vec<Row> {
    let raw = std::fs::read_to_string(corpus_dir().join("MANIFEST.tsv")).expect("MANIFEST.tsv");
    let mut rows = Vec::new();
    for line in raw.lines() {
        if line.starts_with('#') || line.starts_with("fixture\t") || line.trim().is_empty() {
            continue;
        }
        let cols: Vec<&str> = line.split('\t').collect();
        assert_eq!(cols.len(), 6, "malformed manifest row: {:?}", line);
        let opt = |s: &str| if s == "-" { None } else { Some(s.to_string()) };
        rows.push(Row {
            fixture: cols[0].to_string(),
            accept: match cols[1] {
                "accept" => true,
                "reject" => false,
                other => panic!("unknown expect column {:?}", other),
            },
            entries: opt(cols[2]).map(|s| s.parse().expect("entries")),
            hash_code: opt(cols[3])
                .map(|s| u32::from_str_radix(s.trim_start_matches("0x"), 16).expect("hash_code")),
            key_len: opt(cols[4]).map(|s| s.parse().expect("key_len")),
            detail: cols[5].to_string(),
        });
    }
    assert!(!rows.is_empty(), "the manifest listed no fixtures");
    rows
}

fn fixture(name: &str) -> Vec<u8> {
    std::fs::read(corpus_dir().join(name)).unwrap_or_else(|e| panic!("read {}: {}", name, e))
}

/// Every `.bin` in the corpus directory is listed in the manifest, and every manifest row names a
/// file that exists. Without this, adding a fixture and forgetting the manifest row would leave it
/// silently unexercised — the corpus would grow while the conformance surface did not.
#[test]
fn manifest_and_corpus_directory_agree() {
    let rows = manifest();
    let listed: Vec<String> = rows.iter().map(|r| r.fixture.clone()).collect();
    let mut on_disk: Vec<String> = std::fs::read_dir(corpus_dir())
        .expect("corpus dir")
        .map(|e| e.expect("dir entry").file_name().to_string_lossy().into_owned())
        .filter(|n| n.ends_with(".bin"))
        .collect();
    on_disk.sort();
    let mut listed_sorted = listed.clone();
    listed_sorted.sort();
    assert_eq!(
        listed_sorted, on_disk,
        "MANIFEST.tsv and the .bin files on disk disagree"
    );
    assert!(
        listed.iter().filter(|n| n.starts_with("pos-")).count() >= 4,
        "the corpus must keep positive fixtures, not only rejections"
    );
    assert!(
        listed.iter().filter(|n| n.starts_with("neg-")).count() >= 10,
        "the corpus must keep the adversarial half"
    );
}

/// Each `accept` fixture parses, reports the header the manifest records, and **re-encodes to the
/// exact committed bytes**. That last clause is the cross-implementation property: the Rust writer
/// and the independent Python writer must agree byte-for-byte on the canonical form.
#[test]
fn positive_fixtures_parse_and_re_encode_byte_identically() {
    let mut checked = 0;
    for row in manifest().into_iter().filter(|r| r.accept) {
        let bytes = fixture(&row.fixture);
        let table = ExternalKeyTable::from_bytes(&bytes)
            .unwrap_or_else(|e| panic!("{} must parse: {}", row.fixture, e));
        assert_eq!(Some(table.len()), row.entries, "{} entry count", row.fixture);
        assert_eq!(Some(table.hash_code()), row.hash_code, "{} hash code", row.fixture);
        assert_eq!(Some(table.key_len()), row.key_len, "{} key length", row.fixture);
        assert_eq!(
            table.to_bytes(),
            bytes,
            "{} re-encoded to different bytes than the independent generator wrote — the two \
             implementations disagree on the canonical form",
            row.fixture
        );
        assert!(!row.detail.is_empty(), "{} needs a rationale", row.fixture);
        checked += 1;
    }
    assert!(checked >= 4, "expected the positive half of the corpus, ran {}", checked);
}

/// Each `reject` fixture is refused, **and** the error names the reason the manifest records — so a
/// parser cannot pass this suite by rejecting everything for the wrong reason.
#[test]
fn negative_fixtures_are_rejected_for_the_recorded_reason() {
    let mut checked = 0;
    for row in manifest().into_iter().filter(|r| !r.accept) {
        let bytes = fixture(&row.fixture);
        let err = match ExternalKeyTable::from_bytes(&bytes) {
            Ok(_) => panic!("{} must be REJECTED but parsed", row.fixture),
            Err(e) => e,
        };
        assert!(
            err.contains(&row.detail),
            "{} rejected for the wrong reason: expected an error containing {:?}, got {:?}",
            row.fixture,
            row.detail,
            err
        );
        checked += 1;
    }
    assert!(checked >= 10, "expected the adversarial half of the corpus, ran {}", checked);
}

/// A table built in the *worst* insertion order emits the canonical fixture bytes. This is what
/// makes the corpus shareable: the file is a function of the logical table, not of how it was built.
#[test]
fn canonical_bytes_are_independent_of_insertion_order() {
    let bytes = fixture("pos-three-keys.bin");
    let parsed = ExternalKeyTable::from_bytes(&bytes).expect("parse");
    let keys: Vec<(Vec<u8>, u32)> = parsed.entries().map(|(k, s)| (k.to_vec(), s)).collect();

    let mut rebuilt = ExternalKeyTable::new(parsed.hash_code(), parsed.key_len()).expect("new");
    for (digest, slot) in keys.iter().rev() {
        rebuilt.insert(digest, *slot).expect("insert");
    }
    assert_eq!(rebuilt.to_bytes(), bytes, "descending insertion must emit ascending bytes");
    assert_eq!(rebuilt, parsed);
}

/// The fixtures resolve through the lookup API, and a lookup under a different multihash code is an
/// **error** rather than a miss — the substitution guard the profile requires.
#[test]
fn fixture_keys_resolve_and_a_foreign_hash_code_is_refused() {
    let bytes = fixture("pos-three-keys.bin");
    let table = ExternalKeyTable::from_bytes(&bytes).expect("parse");
    let (first, slot) = table.entries().next().expect("an entry");
    let (first, slot) = (first.to_vec(), slot);

    assert_eq!(table.lookup(table.hash_code(), &first).unwrap(), Some(slot));
    // The same digest bytes declared under another code must NOT resolve.
    let err = table.lookup(table.hash_code() + 1, &first).unwrap_err();
    assert!(err.contains("code mismatch"), "got: {}", err);

    // Same key, reached through a whole binary multihash.
    let mut multihash = vec![table.hash_code() as u8, table.key_len() as u8];
    multihash.extend_from_slice(&first);
    assert_eq!(parse_multihash(&multihash).unwrap().0, table.hash_code());
    assert_eq!(table.lookup_multihash(&multihash).unwrap(), Some(slot));

    // An absent key is a clean miss, distinguishable from the refusals above.
    let absent = vec![0xFFu8; table.key_len()];
    assert_eq!(table.lookup(table.hash_code(), &absent).unwrap(), None);
}

/// The provenance fixture carries the embedding-pipeline binding through the block boundary. An
/// external key is generation-independent, not embedding-space-independent.
#[test]
fn provenance_fixture_binds_the_embedding_pipeline() {
    let table = ExternalKeyTable::from_bytes(&fixture("pos-provenance.bin")).expect("parse");
    let prov = table.provenance().expect("the fixture declares provenance");
    assert_eq!(prov.model_id, "text-embedding-3-small");
    assert_eq!(prov.model_version, "2024-01");
    assert_eq!(prov.content_version, "verb-v2");
    assert_eq!(prov.verbalization, "entity-verbalized");
    assert_eq!(prov.metric, Metric::Cosine);
    assert_eq!(prov.normalization, Normalization::L2);

    // A table with no provenance says so, rather than inventing a default pipeline.
    let bare = ExternalKeyTable::from_bytes(&fixture("pos-three-keys.bin")).expect("parse");
    assert_eq!(bare.provenance(), None);
}

/// The signature fixture round-trips OPAQUE bytes. Nothing here verifies them, and the accessor is
/// named so a caller cannot read presence as an integrity claim.
#[test]
fn signature_area_round_trips_unverified() {
    let table = ExternalKeyTable::from_bytes(&fixture("pos-signature.bin")).expect("parse");
    assert_eq!(table.unverified_signature(), &[0xDE, 0xAD, 0xBE, 0xEF]);
    let none = ExternalKeyTable::from_bytes(&fixture("pos-three-keys.bin")).expect("parse");
    assert!(none.unverified_signature().is_empty());
}

/// The corpus is written under the DRAFT profile. If this constant ever moves to the frozen
/// profile's version, these fixtures are stale and must be regenerated from the frozen document —
/// this assertion is the tripwire that forces that, instead of the corpus silently rotting.
#[test]
fn corpus_is_pinned_to_the_draft_profile_version() {
    assert_eq!(
        EXTERNAL_KEY_PROFILE_VERSION, 0,
        "the corpus in tests/fixtures/external-key/ was generated under DRAFT profile version 0; \
         regenerate it from the frozen #1746 profile before bumping this constant"
    );
    let bytes = fixture("pos-three-keys.bin");
    assert_eq!(
        u16::from_le_bytes([bytes[8], bytes[9]]),
        EXTERNAL_KEY_PROFILE_VERSION
    );
}
