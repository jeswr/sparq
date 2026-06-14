// [OPUS-4.8] sq-hbg7 (CI coverage ratchet): host-side unit/integration tests for
// the PROVER-FREE surface of sparq-zk-compose — the Prover.toml emission
// (`toml.rs`), the manifest accessors / serde (`manifest.rs`), and the driver's
// error / path helpers (`driver.rs`). These run with NO `nargo`/`bb` toolchain on
// PATH, which is exactly the environment CI's per-crate coverage job measures in
// (the heavy `#[ignore]`d bb prove/verify tests in `e2e.rs` are skipped there, so
// the toolchain-gated paths they cover do not count toward the CI number). The
// gap CI saw (61.25% vs the 76 floor) was almost entirely the 100%-pure
// `toml.rs` rendering (0% without the prover) plus manifest accessors and driver
// Display/From impls. Every test below asserts REAL behaviour — exact rendered
// bytes, accessor correctness, fail-closed defaults, serde round-trips — not
// trivial `assert!(true)` coverage padding.
//
// NB: deliberately NO `bb`/`nargo` here. A test that drove the CircuitProver
// prove/verify path would either (a) be a no-op under the toolchain-gate (so it
// would not raise the CI/no-toolchain number) or (b) spawn a heavy real prover on
// a box that HAS the toolchain. The honest fix for the CI number is to cover the
// host logic that runs WITHOUT the prover; the prover round-trips are already
// covered by the `#[ignore]`d e2e tests for runs that have `bb`.

use sparq_zk_compose::manifest::{
    AttestedStatusRef, BindingEdge, BindingMode, CircuitId, EntailmentRegime, FieldHex, FilterOp,
    ProofInputs, ProofManifest, RevocationStatus, StatusListSnapshot, SubProof,
};
use sparq_zk_compose::toml::{
    canonical_digits, filter_f64_prover_toml, filter_int_prover_toml, pad_hex, pad_rows,
    prover_toml_for, scan_prover_toml,
};

// --------------------------------------------------------------------------
// helpers
// --------------------------------------------------------------------------

fn fh(s: &str) -> FieldHex {
    FieldHex(s.to_string())
}

// =========================================================================
// toml.rs — Prover.toml emission (196 lines that are 0% without the prover)
// =========================================================================

#[test]
fn filter_int_prover_toml_renders_every_field_in_declaration_order() {
    // Order MUST match filter_int_d{d}/src/main.nr: challenge, operand_enc, op,
    // bound, expected, digits.
    let toml = filter_int_prover_toml(
        &fh("0xchal"),
        &fh("0xoperand"),
        FilterOp::Gt.code(),
        18,
        true,
        b"18",
    );
    let expected = "challenge = \"0xchal\"\n\
                    operand_enc = \"0xoperand\"\n\
                    op = \"2\"\n\
                    bound = \"18\"\n\
                    expected = true\n\
                    digits = [\"49\", \"56\"]\n";
    assert_eq!(toml, expected);
    // declaration order is significant — the keys must appear in this sequence.
    let order = ["challenge", "operand_enc", "op", "bound", "expected", "digits"];
    let mut last = 0usize;
    for k in order {
        let at = toml.find(&format!("{k} =")).unwrap_or_else(|| panic!("missing {k}"));
        assert!(at >= last, "{k} out of declaration order");
        last = at;
    }
}

#[test]
fn filter_int_prover_toml_false_verdict_and_empty_digits() {
    let toml = filter_int_prover_toml(&fh("0x1"), &fh("0x2"), FilterOp::Lt.code(), 0, false, &[]);
    assert!(toml.contains("op = \"0\"\n"));
    assert!(toml.contains("expected = false\n"));
    assert!(toml.contains("digits = []\n"), "empty digit list renders as []");
}

#[test]
fn filter_f64_prover_toml_renders_b_bits_not_bound() {
    // The composable f64 member carries `b_bits` (the IEEE pattern) where the int
    // member carries `bound`; the rest of the layout matches.
    let bits = 42.0f64.to_bits();
    let toml = filter_f64_prover_toml(
        &fh("0xc"),
        &fh("0xop"),
        FilterOp::Ge.code(),
        bits,
        true,
        b"42",
    );
    assert!(toml.contains(&format!("b_bits = \"{bits}\"\n")));
    assert!(!toml.contains("bound ="), "f64 member uses b_bits, never bound");
    assert!(toml.contains("op = \"3\"\n"));
    assert!(toml.contains("digits = [\"52\", \"50\"]\n"));
}

#[test]
fn scan_prover_toml_renders_arrays_and_nested_enc() {
    let toml = scan_prover_toml(
        &fh("0xchal"),
        &[fh("0xc0"), fh("0xc1")],         // 2 graphs
        &[true, false, true],              // pattern_is_const
        &[fh("0xs"), fh("0x0"), fh("0xo")],
        &[[fh("0xr00"), fh("0xr01"), fh("0xr02")]], // one disclosed row
        1,
        &[true, false],                    // attribution (length k=2)
        &[1, 0],                           // per-graph counts
        &[
            vec![[fh("0xe00"), fh("0xe01"), fh("0xe02")]], // graph 0
            vec![],                                         // graph 1 (empty)
        ],
    );
    // commitments array
    assert!(toml.contains("commitments = [\"0xc0\", \"0xc1\"]\n"));
    // bool array (no quotes on bools)
    assert!(toml.contains("pattern_is_const = [true, false, true]\n"));
    // pattern_const_enc hex array
    assert!(toml.contains("pattern_const_enc = [\"0xs\", \"0x0\", \"0xo\"]\n"));
    // rows: nested triple array
    assert!(toml.contains("rows = [[\"0xr00\", \"0xr01\", \"0xr02\"]]\n"));
    assert!(toml.contains("row_count = \"1\"\n"));
    assert!(toml.contains("attribution = [true, false]\n"));
    // counts: quoted decimals
    assert!(toml.contains("counts = [\"1\", \"0\"]\n"));
    // enc: [[[..]],[]]  — graph 0 has one row, graph 1 is empty
    assert!(
        toml.contains("enc = [[[\"0xe00\", \"0xe01\", \"0xe02\"]], []]\n"),
        "nested enc rendering mismatch:\n{toml}"
    );
}

#[test]
fn pad_hex_pads_with_zero_and_leaves_longer_untouched() {
    let padded = pad_hex(vec![fh("0xa")], 3);
    assert_eq!(padded, vec![fh("0xa"), fh("0x0"), fh("0x0")]);
    // already at/over length => unchanged
    let same = pad_hex(vec![fh("0xa"), fh("0xb")], 2);
    assert_eq!(same, vec![fh("0xa"), fh("0xb")]);
    let over = pad_hex(vec![fh("0xa"), fh("0xb"), fh("0xc")], 2);
    assert_eq!(over.len(), 3, "pad never truncates");
}

#[test]
fn pad_rows_pads_with_zero_rows() {
    let padded = pad_rows(vec![[fh("0x1"), fh("0x2"), fh("0x3")]], 2);
    assert_eq!(padded.len(), 2);
    assert_eq!(padded[1], [fh("0x0"), fh("0x0"), fh("0x0")]);
}

#[test]
fn canonical_digits_are_ascii_byte_codes() {
    assert_eq!(canonical_digits(0), vec![b'0']);
    assert_eq!(canonical_digits(18), vec![b'1', b'8']);
    assert_eq!(canonical_digits(1024), vec![b'1', b'0', b'2', b'4']);
    // each byte is an ASCII digit
    for d in canonical_digits(987654321) {
        assert!(d.is_ascii_digit());
    }
}

#[test]
fn prover_toml_for_filter_int_uses_op_code_and_digits() {
    let inputs = ProofInputs::FilterInt {
        id: CircuitId::FilterInt { d: 1 },
        operand_enc: fh("0xop"),
        op: FilterOp::Ne,
        bound: 7,
        expected: false,
    };
    let (id, toml) = prover_toml_for(&inputs, &fh("0xchal"), &[], &[], b"7");
    assert_eq!(id, CircuitId::FilterInt { d: 1 });
    assert!(toml.contains("op = \"5\"\n"), "FilterOp::Ne code is 5");
    assert!(toml.contains("bound = \"7\"\n"));
    assert!(toml.contains("expected = false\n"));
    assert!(toml.contains("digits = [\"55\"]\n"));
}

#[test]
fn prover_toml_for_filter_f64_uses_b_bits() {
    let bits = 18.0f64.to_bits();
    let inputs = ProofInputs::FilterF64 {
        id: CircuitId::FilterF64 { d: 2 },
        operand_enc: fh("0xop"),
        op: FilterOp::Le,
        b_bits: bits,
        expected: true,
    };
    let (id, toml) = prover_toml_for(&inputs, &fh("0xchal"), &[], &[], b"18");
    assert_eq!(id, CircuitId::FilterF64 { d: 2 });
    assert!(toml.contains(&format!("b_bits = \"{bits}\"\n")));
    assert!(toml.contains("op = \"1\"\n"), "FilterOp::Le code is 1");
}

#[test]
fn prover_toml_for_scan_pads_rows_and_enc_to_circuit_dimensions() {
    // n=3 slots/graph, r=2 disclosed rows; supply ONE active row + one short graph
    // and assert the renderer pads up to (r, n).
    let inputs = ProofInputs::Scan {
        id: CircuitId::Scan { k: 1, n: 3, r: 2 },
        commitments: vec![fh("0xc0")],
        pattern_is_const: [true, false, false],
        pattern_const_enc: [fh("0xp"), fh("0x0"), fh("0x0")],
        rows: vec![[fh("0xr0"), fh("0xr1"), fh("0xr2")]], // 1 row -> padded to r=2
        row_count: 1,
        attribution: vec![true],
    };
    // graph 0 supplies 2 active triples; should pad up to n=3.
    let scan_enc = vec![vec![
        [fh("0xe00"), fh("0xe01"), fh("0xe02")],
        [fh("0xe10"), fh("0xe11"), fh("0xe12")],
    ]];
    let (id, toml) = prover_toml_for(&inputs, &fh("0xchal"), &[2], &scan_enc, &[]);
    assert_eq!(id, CircuitId::Scan { k: 1, n: 3, r: 2 });
    // rows padded from 1 -> 2 (the second row is the zero row).
    assert!(
        toml.contains("rows = [[\"0xr0\", \"0xr1\", \"0xr2\"], [\"0x0\", \"0x0\", \"0x0\"]]\n"),
        "rows must pad to r=2:\n{toml}"
    );
    // enc padded from 2 -> 3 rows in the single graph (third is the zero row).
    assert!(
        toml.contains("[\"0xe10\", \"0xe11\", \"0xe12\"], [\"0x0\", \"0x0\", \"0x0\"]]]\n"),
        "enc must pad to n=3:\n{toml}"
    );
    assert!(toml.contains("counts = [\"2\"]\n"));
}

// =========================================================================
// manifest.rs — accessors, fail-closed helpers, serde
// =========================================================================

#[test]
fn filter_op_codes_are_the_circuit_globals() {
    // value = the `op` public input the circuit expects (OP_* globals).
    assert_eq!(FilterOp::Lt.code(), 0);
    assert_eq!(FilterOp::Le.code(), 1);
    assert_eq!(FilterOp::Gt.code(), 2);
    assert_eq!(FilterOp::Ge.code(), 3);
    assert_eq!(FilterOp::Eq.code(), 4);
    assert_eq!(FilterOp::Ne.code(), 5);
}

#[test]
fn circuit_id_package_names_every_member_family() {
    assert_eq!(CircuitId::Scan { k: 2, n: 4, r: 1 }.package(), "scan_k2_n4_r1");
    assert_eq!(CircuitId::FilterInt { d: 1 }.package(), "filter_int_d1");
    assert_eq!(CircuitId::FilterF64 { d: 3 }.package(), "filter_f64_d3");
    assert_eq!(CircuitId::RevokeUnset { depth: 10 }.package(), "revoke_unset_d10");
    assert_eq!(CircuitId::HiddenIssuer { depth: 4 }.package(), "hidden_issuer_d4");
}

#[test]
fn proof_inputs_circuit_id_accessor_matches_each_variant() {
    let scan = ProofInputs::Scan {
        id: CircuitId::Scan { k: 1, n: 1, r: 1 },
        commitments: vec![fh("0xc")],
        pattern_is_const: [false, false, false],
        pattern_const_enc: [fh("0x0"), fh("0x0"), fh("0x0")],
        rows: vec![],
        row_count: 0,
        attribution: vec![false],
    };
    assert_eq!(scan.circuit_id(), &CircuitId::Scan { k: 1, n: 1, r: 1 });

    let fi = ProofInputs::FilterInt {
        id: CircuitId::FilterInt { d: 2 },
        operand_enc: fh("0x1"),
        op: FilterOp::Eq,
        bound: 1,
        expected: true,
    };
    assert_eq!(fi.circuit_id(), &CircuitId::FilterInt { d: 2 });

    let ff = ProofInputs::FilterF64 {
        id: CircuitId::FilterF64 { d: 2 },
        operand_enc: fh("0x1"),
        op: FilterOp::Eq,
        b_bits: 0,
        expected: true,
    };
    assert_eq!(ff.circuit_id(), &CircuitId::FilterF64 { d: 2 });
}

#[test]
fn binding_mode_challenge_accessor_for_both_variants() {
    let plain = BindingMode::Challenge { challenge: fh("0xaa") };
    assert_eq!(plain.challenge(), &fh("0xaa"));

    let holder = BindingMode::HolderPop {
        challenge: fh("0xbb"),
        holder: "deadbeef".to_string(),
        pop: "cafe".to_string(),
        cryptosuite: "https://sparq.dev/ns/zk#poseidon2-schnorr-v1".to_string(),
    };
    assert_eq!(holder.challenge(), &fh("0xbb"));
}

#[test]
fn field_hex_round_trips_and_rejects_malformed() {
    use sparq_zk::field::Fr;
    let f = Fr::from(0x1234u64);
    let hex = FieldHex::from_field(&f);
    assert_eq!(hex.to_field(), Some(f), "from_field/to_field round-trips");
    // malformed hex => None (fail-closed parse, never a panic).
    assert_eq!(FieldHex("not-hex".to_string()).to_field(), None);
    assert_eq!(FieldHex(String::new()).to_field(), None);
}

#[test]
fn status_list_snapshot_bit_is_lsb_first_and_out_of_range_reads_revoked() {
    // bits = 0b0000_0110 -> index 1 and 2 set, others unset (LSB-first within byte).
    let snap = StatusListSnapshot {
        status_list: "http://ex/list".to_string(),
        version: 1,
        bits: vec![0b0000_0110],
    };
    assert!(!snap.bit(0), "bit 0 unset");
    assert!(snap.bit(1), "bit 1 set");
    assert!(snap.bit(2), "bit 2 set");
    assert!(!snap.bit(3), "bit 3 unset");
    // crossing byte boundary: bit 8 is in byte 1, which is absent => fail-closed.
    assert!(snap.bit(8), "out-of-range index reads as SET (revoked), fail-closed");
    assert!(snap.bit(999_999), "far out-of-range also reads revoked");
}

#[test]
fn status_list_snapshot_second_byte_indexing() {
    let snap = StatusListSnapshot {
        status_list: "http://ex/list".to_string(),
        version: 2,
        bits: vec![0x00, 0b0000_0001], // bit 8 set (byte 1, offset 0)
    };
    assert!(!snap.bit(7));
    assert!(snap.bit(8), "bit 8 = byte 1 offset 0");
    assert!(!snap.bit(9));
}

#[test]
fn proof_manifest_json_round_trips_and_defaults_type() {
    // A minimal manifest with a HolderPop binding (exercises the binding serde +
    // the default-cryptosuite field) and a committed-index revocation reference.
    let manifest = ProofManifest {
        r#type: "urn:sparq:zk:ProofManifest".to_string(),
        query: "SELECT * WHERE { ?s ?p ?o }".to_string(),
        issuers: vec!["did:key:zABC".to_string()],
        key_set: vec!["aa".to_string()],
        commitment_attestations: vec![],
        attributions: vec![vec![0]],
        join_obligations: vec![("v".to_string(), 0, 1)],
        entailment_regime: EntailmentRegime::Simple,
        derivation_steps: vec![],
        binding: BindingMode::Challenge { challenge: fh("0x2a") },
        revocation: Some(RevocationStatus {
            status_list: "http://ex/list".to_string(),
            index: Some(3),
            version: 7,
            index_commitment: None,
        }),
        status_snapshots: vec![StatusListSnapshot {
            status_list: "http://ex/list".to_string(),
            version: 7,
            bits: vec![0x00],
        }],
        sub_proofs: vec![SubProof {
            inputs: ProofInputs::FilterInt {
                id: CircuitId::FilterInt { d: 1 },
                operand_enc: fh("0x1"),
                op: FilterOp::Gt,
                bound: 1,
                expected: true,
            },
            proof_hex: String::new(),
        }],
        binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
        hidden_revocation: None,
        hidden_issuer_attestations: vec![],
    };
    let json = manifest.to_json();
    let back = ProofManifest::from_json(&json).expect("round-trips");
    assert_eq!(back, manifest);
    // The default type-marker is applied when the field is absent from the JSON.
    let stripped = json.replace(
        "\"type\": \"urn:sparq:zk:ProofManifest\",\n  ",
        "",
    );
    let defaulted = ProofManifest::from_json(&stripped).expect("type defaults");
    assert_eq!(defaulted.r#type, "urn:sparq:zk:ProofManifest");
}

#[test]
fn holder_pop_cryptosuite_defaults_when_absent_in_json() {
    // A HolderPop binding JSON that omits `cryptosuite` must default to the v1
    // Schnorr suite (serde default fn), NOT fail to parse.
    let json = r#"{
      "mode": "holder-pop",
      "challenge": "0x2a",
      "holder": "deadbeef",
      "pop": "cafe"
    }"#;
    let bm: BindingMode = serde_json::from_str(json).expect("parses with default cryptosuite");
    match bm {
        BindingMode::HolderPop { cryptosuite, .. } => {
            assert_eq!(cryptosuite, "https://sparq.dev/ns/zk#poseidon2-schnorr-v1");
        }
        _ => panic!("expected HolderPop"),
    }
}

#[test]
fn attested_status_ref_committed_index_serde_omits_clear_index() {
    // sq-ayv committed-index path: index = None, index_commitment = Some. The
    // `skip_serializing_if` keeps the clear `index` out of the JSON entirely.
    let r = AttestedStatusRef {
        index: None,
        version: 4,
        index_commitment: Some(fh("0xc0mm")),
    };
    let json = serde_json::to_string(&r).unwrap();
    assert!(!json.contains("\"index\""), "clear index omitted on committed path: {json}");
    assert!(json.contains("index_commitment"));
    let back: AttestedStatusRef = serde_json::from_str(&json).unwrap();
    assert_eq!(back, r);

    // clear-index path: index = Some, index_commitment omitted.
    let clear = AttestedStatusRef { index: Some(2), version: 1, index_commitment: None };
    let cjson = serde_json::to_string(&clear).unwrap();
    assert!(cjson.contains("\"index\":2"));
    assert!(!cjson.contains("index_commitment"));
    assert_eq!(serde_json::from_str::<AttestedStatusRef>(&cjson).unwrap(), clear);
}

// =========================================================================
// driver.rs — error Display / From, prover construction (no subprocess spawn)
// =========================================================================

#[test]
fn driver_error_display_is_informative_for_every_variant() {
    use sparq_zk_compose::driver::DriverError;
    let spawn = DriverError::Spawn {
        tool: "nargo".to_string(),
        source: std::io::Error::new(std::io::ErrorKind::NotFound, "no such file"),
    };
    let s = format!("{spawn}");
    assert!(s.contains("nargo") && s.contains("spawn"), "spawn display: {s}");

    let tool = DriverError::Tool {
        tool: "bb".to_string(),
        stderr: "assertion failed".to_string(),
    };
    let t = format!("{tool}");
    assert!(t.contains("bb") && t.contains("assertion failed"), "tool display: {t}");

    let io = DriverError::Io(std::io::Error::other("disk"));
    assert!(format!("{io}").contains("io error"), "io display");

    // std::error::Error is implemented (Display + Debug).
    let as_err: &dyn std::error::Error = &spawn;
    assert!(!as_err.to_string().is_empty());
}

#[test]
fn driver_error_from_io_error_maps_to_io_variant() {
    use sparq_zk_compose::driver::DriverError;
    let e: DriverError = std::io::Error::other("boom").into();
    assert!(matches!(e, DriverError::Io(_)), "From<io::Error> -> Io variant");
    assert!(format!("{e}").contains("boom"));
}

#[test]
fn circuit_prover_constructs_without_spawning_any_tool() {
    use sparq_zk_compose::driver::CircuitProver;
    // `new` and `from_crate_root` are pure path construction — no nargo/bb spawn,
    // so they run identically in CI's no-toolchain environment.
    let _p = CircuitProver::new("/nonexistent/zk/compose");
    let _p2 = CircuitProver::from_crate_root();
    // The constructed prover is just config; building it must not require the
    // toolchain to be present.
}

#[test]
fn proof_artifacts_clone_and_encode_blob_layout() {
    use sparq_zk_compose::driver::ProofArtifacts;
    use sparq_zk_compose::verifier::encode_artifacts;
    // `encode_artifacts` is the PUBLIC host encoder for the `proof_hex` blob: it is
    // `lp(proof) ‖ lp(public_inputs) ‖ vk`, where `lp` is a 4-byte big-endian
    // length prefix. The verifier's (private) decoder splits exactly this layout;
    // here we pin the encoder's wire format without needing a real bb proof.
    let art = ProofArtifacts {
        proof: vec![0xaa, 0xbb],          // 2 bytes
        public_inputs: vec![0x01],         // 1 byte
        vk: vec![0xff, 0xee, 0xdd],        // trailing vk (no length prefix)
    };
    let hex = encode_artifacts(&art);
    // lp(proof)        = 00000002 aabb
    // lp(public_inputs)= 00000001 01
    // vk               = ffeedd
    assert_eq!(hex, "00000002aabb0000000101ffeedd");
    // ProofArtifacts derives Clone — exercise it.
    let art2 = art.clone();
    assert_eq!(encode_artifacts(&art2), hex);

    // Empty segments still carry their (zero) length prefix.
    let empty = ProofArtifacts { proof: vec![], public_inputs: vec![], vk: vec![] };
    assert_eq!(encode_artifacts(&empty), "0000000000000000");
}
