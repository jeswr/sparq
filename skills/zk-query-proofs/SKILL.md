---
name: zk-query-proofs
description: Prove and verify a SPARQL query result over committed RDF Verifiable Credentials in zero knowledge with sparq-zk + sparq-zk-compose — per-graph Poseidon2 commitments, BGP scan + integer FILTER Noir proofs, issuer Schnorr attestation (incl. hidden-key set membership), status-list revocation (clear- or hidden-index), verifier nonces / single-use replay defence, and the ProofManifest/circuit family. Use when building or driving the zk-query-proofs surface (proving a query answer, verifying a manifest, attesting an issuer, checking revocation). Requires the Noir toolchain (nargo + bb).
---

# sparq-zk-query-proofs

Zero-knowledge proofs that a SPARQL query result is correct over RDF held in named-graph Verifiable Credentials. **`sparq-zk`** (stage 1) canonicalizes (RDFC10) and commits each named graph to a Poseidon2-BN254 commitment `C(G)`, encodes terms, and signs commitments with a Schnorr-over-Baby-JubJub issuer key. **`sparq-zk-compose`** (stage 2) builds per-property Noir circuit inputs (BGP **scan** + hidden-operand integer **FILTER**), drives `nargo`/`bb` to produce/verify proofs, and bundles everything into a serializable `ProofManifest` that a relying party verifies against its own trust anchors (issuer key-set, status list, fresh nonce).

> **Research-stage / experimental — NOT-yet-sound.** The composition verifier's soundness is the subject of an open audit (sq-qhy4 / sq-9hrn; remediation epic sq-1s2): a passing proof is NOT a guarantee the SPARQL statement holds under an adversarial prover. Read the "Honest scope" section before relying on a guarantee — only `Simple` entailment is proved; circuit members are fixed buckets. The query fragment covers BGP scans, integer FILTER (and the integer-valued `xsd:double` fragment), and a single-prover hidden cross-credential JOIN.

## Prerequisites

- **Noir toolchain on `PATH`** (the only way to prove/verify): `nargo` **1.0.0-beta.21** and Barretenberg `bb` **5.0.0-nightly.20260324** (bb target `noir-recursive`). Other versions may change the bb public-input byte layout the verifier reconstructs against. If `nargo`/`bb` are absent, the structural pre-filter and all host-side helpers still work, but `verify_manifest` / `CircuitProver` cannot.
- **Compiled circuit family** lives at `zk/compose/` in the repo (a Nargo workspace, NOT linked — driven by subprocess). `CircuitProver::from_crate_root()` locates it as `../../zk/compose` relative to the crate.
- **Cargo deps** (both crates are `publish = false`, non-default workspace members — nothing else in sparq depends on them, and there is **no `zk` cargo feature on these crates** to enable; just add them as path/git deps):
  ```toml
  sparq-zk = { path = "crates/sparq-zk" }
  sparq-zk-compose = { path = "crates/sparq-zk-compose" }
  ```

## Quickstart

A self-contained integer-FILTER proof (`5 < 10`) and full verify. Compiles and runs against the current API when `nargo`+`bb` are present (else `prove_in`/`verify` error).

```rust
use sparq_zk_compose::build::{build_filter_int, encode_int_literal};
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::toml::prover_toml_for;
use sparq_zk_compose::manifest::{CircuitId, FieldHex, FilterOp};

// 1. Build inputs: prove the hidden operand 5 satisfies `?v < 10`, verdict true.
let operand_enc = encode_int_literal(5);                       // term encoding of "5"^^xsd:integer
let (inputs, digits) =
    build_filter_int(operand_enc, /*value*/ 5, FilterOp::Lt, /*bound*/ 10, /*expected*/ true)
        .expect("a compiled filter_int member fits");

// 2. Render the Prover.toml (challenge = the verifier's nonce; 0x2a here).
let challenge = FieldHex("0x2a".into());
let (id, toml) = prover_toml_for(&inputs, &challenge, &[], &[], &digits);
assert_eq!(id, CircuitId::FilterInt { d: 1 });

// 3. Prove + verify via nargo/bb subprocesses (tag isolates concurrent provers).
let prover = CircuitProver::from_crate_root();
let out = std::env::temp_dir().join("sparq_zk_quickstart");
let art = prover.prove_in(&id, &toml, &out, "quickstart").expect("prove");
assert!(prover.verify(&art, &out.join("verify")).expect("verify runs"));
```

## Key APIs

Stage 1 (`sparq-zk`):
- `encode::salt_from_bytes(&[u8;32]) -> Fr` — a per-graph bnode salt. In trusted ingest use `ingest::IngestedDataset` instead (mints a fresh globally-unique salt per graph).
- `commit::commit_triples(triples: &[Triple], salt: Fr) -> Result<GraphCommitment, CommitError>` and `commit::commit_graph_content(&sparq_core::Graph, salt) -> Result<GraphCommitment, _>`. `GraphCommitment { canonical, leaves, commitment: Fr, salt: Fr }`.
- `ingest::IngestedDataset::ingest(store: &Graph, names: &[NamedNode]) -> Result<Self, IngestError>` — per-named-graph commitments, each under a fresh OS-random salt; `.commitments()`, `.salts()`, `.names()`.
- `sig::SecretKey::{from_seed(u64) /*test/tooling only*/, public_key(), sign_commitment_with_status(&Fr c, &Fr salt, &Fr status_ref) -> String}`; free fns `commitment_message_with_status`, `status_ref_digest(&Fr list_id, index, version)`, `status_list_id_to_field(iri)`, `public_key_to_hex` / `public_key_from_hex`, `verify`, and `SignatureScheme::Poseidon2SchnorrV1`.

Stage 2 (`sparq-zk-compose`):
- `build::{Pattern, Slot::{Const(Term), Var}, build_scan(&[GraphCommitment], &Pattern) -> Option<BuiltScan>, build_filter_int(operand_enc, value, op, bound, expected) -> Option<(ProofInputs, Vec<u8>)>, encode_int_literal(u64) -> FieldHex}`. `BuiltScan { inputs: ProofInputs, witness: ScanWitness { counts, enc } }`.
- `toml::prover_toml_for(&ProofInputs, &FieldHex challenge, scan_counts: &[u32], scan_enc: &[Vec<[FieldHex;3]>], filter_digits: &[u8]) -> (CircuitId, String)`.
- `driver::CircuitProver::{from_crate_root(), compile(&CircuitId), prove_in(&CircuitId, toml: &str, out_dir: &Path, tag: &str) -> Result<ProofArtifacts, DriverError>, gen_witness_tagged, canonical_vk, verify_with, verify}`.
- `verifier::encode_artifacts(&ProofArtifacts) -> String` — the `proof_hex` blob (`len|proof|len|public_inputs|vk`) to put in a `SubProof`.
- `verifier::verify_manifest(&ProofManifest, &CircuitProver, work_dir: &Path, &KeySet, &RevocationPolicy, &VerifierNonce, &dyn SeenNonces) -> Result<(), CheckError>` — **the full-binding entry point** (the only path that runs every gate; an internal re-audit finds it sound-as-landed under its stated threat model, but it is **pending external cryptographer sign-off — treat it as NOT-yet-sound for production**, see [SECURITY.md](../../SECURITY.md) / sq-qhy4). `prefilter_manifest_structure(&ProofManifest, &KeySet, &RevocationPolicy)` runs only the fast structural gate (no bb, binds nothing to a proof, enforces no freshness — **NOT a sound verifier on its own**). <!-- privacy-claims-allow: negative usage ("NOT a sound verifier on its own") + pending-external-audit caveat; sq-toze.35 -->
- Trust anchors / freshness: `KeySet::{empty, from_hex_keys(I), with_hidden_issuer_depth(u32)}`, `RevocationPolicy::{up_to(now, window), accept_version(v), with_snapshot(StatusListSnapshot), with_hidden_index_depth(u32)}`, `VerifierNonce::{from_hex, from_field}`, `FileSeenNonces::open(path)` (durable), `InMemorySeenNonces::new()` (test-only).
- Manifest model: `manifest::{ProofManifest, ProofInputs::{Scan, FilterInt}, SubProof { inputs, proof_hex }, BindingEdge, BindingMode::Challenge { challenge }, CommitmentAttestation, AttestedStatusRef, RevocationStatus, StatusListSnapshot, EntailmentRegime::Simple, FilterOp, CircuitId, FieldHex}`. `ProofManifest::{to_json, from_json}` round-trip via serde.
- Privacy upgrades (opt-in): `issuer::{key_set_root, key_membership_witness, hidden_issuer_prover_toml, HiddenIssuerWitness}`, `holder::{holder_set_root, holder_set_membership_witness, holder_set_prover_toml, HolderSetWitness}` (hidden-holder-SET tier, sq-3c00), and `revocation::{merkle_root, merkle_witness, revoke_prover_toml, MerkleWitness}`.
- Large-registry scaling (sq-8k3h, host-side only): `issuer::{key_set_root_sparse, key_membership_witness_sparse}` and `holder::{holder_set_root_sparse, holder_set_membership_witness_sparse}` build the BIT-IDENTICAL root + authentication path in `O(n·depth)` (no `2^depth` materialisation), so a very large issuer/holder registry commits at any depth. The in-circuit relation is depth-generic and UNCHANGED — these are a drop-in for the dense builders, asserting NO new soundness/privacy property.

## Common recipes

### 1. Commit a credential graph and attest it as an issuer
```rust
use sparq_zk::commit::commit_triples;
use sparq_zk::encode::salt_from_bytes;
use sparq_zk::sig::{SecretKey, status_list_id_to_field, status_ref_digest, public_key_to_hex, SignatureScheme};
use sparq_zk_compose::manifest::{AttestedStatusRef, CommitmentAttestation, FieldHex};

let salt = salt_from_bytes(&[7u8; 32]);                 // production: use IngestedDataset (OS-random)
let commit = commit_triples(&credential_triples, salt).unwrap();

let issuer = SecretKey::from_seed(1);                   // production: generate from OS entropy, NOT a seed
let (list, index, version) = ("http://ex/status/1", 3u64, 1u64);
let status_ref = status_ref_digest(&status_list_id_to_field(list), index, version);

// A scan-covering attestation MUST bind salt AND status reference (fail-closed otherwise).
let attestation = CommitmentAttestation {
    commitment: FieldHex::from_field(&commit.commitment),
    issuer_public_key: public_key_to_hex(&issuer.public_key()),
    signature: issuer.sign_commitment_with_status(&commit.commitment, &salt, &status_ref),
    cryptosuite: SignatureScheme::Poseidon2SchnorrV1.cryptosuite_iri().to_string(),
    salt: Some(FieldHex::from_field(&salt)),
    status: Some(AttestedStatusRef { index, version }),
};
```

### 2. Build a BGP scan proof for `{ ?s <http://ex/age> ?o }`
```rust
use oxrdf::{NamedNode, Term};
use sparq_zk_compose::build::{build_scan, Pattern, Slot};
use sparq_zk_compose::manifest::ProofInputs;

let pattern = Pattern {
    s: Slot::Var,
    p: Slot::Const(Term::NamedNode(NamedNode::new("http://ex/age").unwrap())),
    o: Slot::Var,
};
let scan = build_scan(&[commit /* GraphCommitment */], &pattern).expect("a compiled scan member fits");
// scan.inputs is ProofInputs::Scan { id, commitments, rows, attribution, .. };
// scan.witness has the private per-graph encodings prover_toml_for needs.
let operand_enc = match &scan.inputs {
    ProofInputs::Scan { rows, .. } => rows[0][2].clone(), // the object column to feed a FILTER
    _ => unreachable!(),
};
```

### 3. Assemble + prove a full manifest (scan + bound integer FILTER)
```rust
use sparq_zk_compose::build::build_filter_int;
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::toml::prover_toml_for;
use sparq_zk_compose::verifier::encode_artifacts;
use sparq_zk_compose::manifest::*;

let challenge = FieldHex("0x2a".into());                 // == the verifier's nonce
let prover = CircuitProver::from_crate_root();
let out = std::env::temp_dir().join("sparq_zk_manifest");

// scan proof
let (scan_id, scan_toml) = prover_toml_for(&scan.inputs, &challenge,
    &scan.witness.counts, &scan.witness.enc, &[]);
let scan_art = prover.prove_in(&scan_id, &scan_toml, &out, "scan").unwrap();

// filter proof `?o >= 18`, verdict true
let (filter_inputs, digits) = build_filter_int(operand_enc, 25, FilterOp::Ge, 18, true).unwrap();
let (f_id, f_toml) = prover_toml_for(&filter_inputs, &challenge, &[], &[], &digits);
let filter_art = prover.prove_in(&f_id, &f_toml, &out, "filter").unwrap();

let manifest = ProofManifest {
    r#type: "urn:sparq:zk:ProofManifest".into(),
    query: "SELECT ?s ?o WHERE { ?s <http://ex/age> ?o FILTER(?o >= 18) }".into(),
    issuers: vec!["did:key:zIssuer".into()],
    key_set: vec![public_key_to_hex(&issuer.public_key())], // declared (narrowing) — NOT the trust anchor
    commitment_attestations: vec![attestation],
    attributions: vec![vec![0]],                            // one BGP pattern, from graph 0
    join_obligations: vec![],
    entailment_regime: EntailmentRegime::Simple,
    binding: BindingMode::Challenge { challenge: FieldHex("0x2a".into()) },
    revocation: Some(RevocationStatus { status_list: "http://ex/status/1".into(), index: 3, version: 1 }),
    status_snapshots: vec![],                               // prover copy is only a tripwire; see recipe 4
    sub_proofs: vec![
        SubProof { inputs: scan.inputs,   proof_hex: encode_artifacts(&scan_art) },
        SubProof { inputs: filter_inputs, proof_hex: encode_artifacts(&filter_art) },
    ],
    binding_edges: vec![BindingEdge { from_proof: 0, from_row: 0, from_slot: 2, to_proof: 1 }],
    hidden_revocation: None,
    hidden_issuer_attestations: vec![],
};
let json = manifest.to_json(); // ships to the verifier
```

### 4. Verify a manifest as a relying party (the full-binding path)
```rust
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::verifier::{verify_manifest, KeySet, RevocationPolicy, VerifierNonce, FileSeenNonces};
use sparq_zk_compose::manifest::{ProofManifest, StatusListSnapshot};
use sparq_zk::sig::public_key_to_hex;

let manifest = ProofManifest::from_json(&json).unwrap();

// External trust anchors — resolved + authenticated OUT OF BAND, never from the manifest:
let trusted = KeySet::from_hex_keys([public_key_to_hex(&issuer.public_key())]); // issuers you trust
let policy = RevocationPolicy::accept_version(1)        // freshness window; or ::up_to(now, window)
    .with_snapshot(StatusListSnapshot {                // AUTHORITATIVE bitstring (bit decision reads HERE)
        status_list: "http://ex/status/1".into(), version: 1, bits: vec![0u8] /* index 3 unset = active */,
    });

let nonce = VerifierNonce::from_hex("0x2a").unwrap();  // mint fresh per session, hand to prover BEFORE proving
let seen = FileSeenNonces::open("/var/lib/sparq/seen_nonces").unwrap(); // durable single-use store

let prover = CircuitProver::from_crate_root();
match verify_manifest(&manifest, &prover, std::path::Path::new("/tmp/verify"),
                      &trusted, &policy, &nonce, &seen) {
    Ok(())  => { /* result is correct, issuer-attested, live, fresh */ }
    Err(e)  => eprintln!("rejected: {e:?}"), // CheckError variant pinpoints the failed gate
}
```
The nonce is **burned on presentation** (consumed even if verification then fails) — a rejection is never a free retry; mint a new nonce for the next attempt.

### 5. Fast structural pre-check without the toolchain
```rust
use sparq_zk_compose::verifier::{prefilter_manifest_structure, KeySet, RevocationPolicy};
// Re-parses the query (Q6 cross-graph bnode-join guard + arity), re-derives circuit ids,
// checks binding edges + issuer/key-set + revocation reference. NO bb, NO freshness.
// NOT a sound verifier on its own — always follow with verify_manifest. privacy-claims-allow: negative usage; sq-toze.35
let _required = prefilter_manifest_structure(&manifest, &KeySet::empty(), &RevocationPolicy::accept_version(1))?;
```

### 6. Hidden-index revocation proof (privacy upgrade — index never disclosed)
```rust
use sparq_zk_compose::revocation::{merkle_root, merkle_witness, revoke_prover_toml};
use sparq_zk_compose::manifest::{CircuitId, HiddenIndexRevocation, FieldHex};
use sparq_zk_compose::driver::CircuitProver;
use sparq_zk_compose::verifier::encode_artifacts;

let depth = 10;                                  // compiled member: revoke_unset_d10 (<= 1024 indices)
let root = merkle_root(&authoritative_snapshot, depth).unwrap();
let witness = merkle_witness(&authoritative_snapshot, depth, /*hidden index*/ 3).unwrap(); // witness.bit must be 0
let toml = revoke_prover_toml(&nonce_field, &root, 3, &witness);
let prover = CircuitProver::from_crate_root();
let art = prover.prove_in(&CircuitId::RevokeUnset { depth }, &toml, std::path::Path::new("/tmp/rev"), "rev").unwrap();
// attach to manifest.hidden_revocation; enable on the verifier with
// RevocationPolicy::...with_hidden_index_depth(depth). (Hidden-issuer attestations mirror this
// with sparq_zk_compose::issuer::* + KeySet::with_hidden_issuer_depth.)
```

## Honest scope (what is and isn't supported)

- **Entailment:** only `EntailmentRegime::Simple` is proved — no in-circuit RDFS/OWL reasoning (`Rdfs`/`Owl` are stable schema placeholders).
- **Query fragment:** BGP triple-pattern **scan** (in-circuit per-graph Poseidon2 commitment recompute + row soundness + **scan completeness**) and hidden-operand numeric **FILTER over `xsd:integer`** (`filter_int`, non-negative). `xsd:double` FILTER (`filter_f64`) is composable for the integer-valued fragment (`filter_f64_d{d}`; general fractional/scientific forms deferred). **NEGATIVE `xsd:integer`** (`filter_signed_int`) and **`xsd:decimal`** (`filter_decimal`, fixed-point) are compiled, byte-bound CIRCUIT MEMBERS ([OPUS-4.8] sq-1q9h) with the same operand binding as `filter_int`; their manifest-composability wiring (`CircuitId`/`ProofInputs`/verifier edges) is a follow-up (sq-7lrq), so they are not yet assemblable into a `ProofManifest`. A **hidden cross-credential JOIN** is proved in-circuit (`join_eq`, single-prover; the join term stays private) — distinct from the verifier-side disclosed-row join. **No aggregation.** The Q6 cross-graph bnode-join guard runs from `manifest.attributions` / `join_obligations`.
- **Fixed circuit members only** (build returns `None` for shapes outside these): scan `k∈{1,2}`, `n∈{16,64}`, `r∈{4,8}` — **all eight `(k,n,r)` combinations compiled** ([OPUS-4.8] sq-pzet); `filter_int_d∈{1,2,3,4}`; `filter_f64_d∈{1,2,3,4}`; `filter_signed_int_d∈{2,4}` + `filter_decimal_i3_f2` (compiled, not yet manifest-composable — sq-1q9h/sq-7lrq); `join_eq` `n_a,n_b∈{16,64}` — **all four `(n_a,n_b)` combinations compiled** ([OPUS-4.8] sq-pzet); `revoke_unset_d10` (≤1024 status indices); `hidden_issuer_d4` (≤16 issuers); `holder_pok`. The buckets are derived from the data by the prover **and re-derived by the verifier** (a proof can only verify against the member its public inputs fit); an out-of-bucket shape returns `None` (a clean error, never a silently-unprovable wrong-bucket member).
- **Privacy defaults:** issuer attestation is checked in the **clear** (reveals which issuer signed) and `RevocationStatus.index` is **disclosed** (a linkability channel) unless you opt into the hidden-issuer / hidden-index circuits — those are **additive** layers; the clear-path checks always still run.
- **Holder binding (`HolderPop`):** a presentation may bind the proof to a holder key, cross-checked against the issuer-attested `AttestedHolderBinding.holder_pk_digest` (the issuer signed `commitment_message_with_holder` under the external `K`), so it closes the trusted-holder gap (holder A cannot present holder B's credential). Two tiers: **B1 (clear-key, default)** discloses the holder key and the verifier recomputes its digest host-side (`verifier::bind_holder_pop` / `bind_holder_binding`, gated by `HolderBindingPolicy::require_binding()`); **B2 (hidden-key, opt-in — [OPUS-4.8] sq-c2ql)** carries a `HolderPokProof` (a `holder_pok` bb proof) so the holder proves possession **in zero knowledge without disclosing the key** — `verifier::bind_holder_pok` binds the proof's public digest to the issuer-attested digest (the binding edge), gated by `HolderBindingPolicy::require_in_circuit_pok()`. B2 is **NOT-yet-sound** (sq-qhy4) like the rest of the verifier; it is the additive hidden-holder layer over B1.
- **Trust anchors are external, never the prover's manifest:** the trusted issuer key-set (`KeySet`) and the authoritative status bitstring (`RevocationPolicy::with_snapshot`) come from the relying party. `manifest.key_set` is only accepted as a *subset* of the external `K`; the prover's `status_snapshots` is only a tamper tripwire, never the bit-decision source.
- **Proving is subprocess-only** (`nargo`/`bb`), no embedded prover. **Concurrency:** against the *same* compiled member, use the tagged entry points (`prove_in` / `gen_witness_tagged` with a unique `tag`) — the untagged `prove` / `gen_witness` share one `Prover.toml`/witness and are only safe single-threaded.
- **Replay/freshness:** use `FileSeenNonces` (durable: `flock` + `fsync`, single-host) in production. `InMemorySeenNonces` is non-durable, test-only (a restart reopens the replay window). For multi-host, back `SeenNonces` with a DB UNIQUE constraint / CAS store.
- **Toolchain pin:** `nargo 1.0.0-beta.21`, `bb 5.0.0-nightly.20260324`, bb target `noir-recursive`. Other versions may change the public-input byte layout the audit-#1 reconstruction byte-compares against.
- **Maturity:** v1, authored by Opus 4.8 while Fable was unavailable; flagged for ZK re-review. Treat as a research seam, not a hardened product.

## See also

- `verifiable-credentials-zk` — credential signature schemes, commitment choices, and the credential↔circuit public-input contract.
- `noir-circuit-patterns` / `noir-optimisation` — writing/sizing the Noir circuits this crate drives (`zk/compose/`).
- `sparql-formal-semantics` — the Pérez–Arenas–Gutiérrez fragment + blank-node scoping the Q6 guard and `verify::recheck` enforce.
- `mpc-protocols` — the multi-party layer that composes with this single-prover ZK estate.
