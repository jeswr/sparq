# sparq-zk-compose — STATUS

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable returns).

ZK stage 2: composition prover + verifier for sparq. Worktree `sparq-zkcompose`,
branch `zk-compose`.

## Milestone log

### M0 — inherited scaffold validation (DONE)

Fable-authored scaffold at `zk/compose/` reviewed critically and validated:

- `nargo check` clean (nargo 1.0.0-beta.21).
- `nargo test --package sparq_zk_compose_core`: **22/22 pass** (accept + adversarial).
- Bit-compatibility CONFIRMED: the cross-vectors in `compose_core/src/tests.nr`
  are byte-for-byte identical to `crates/sparq-zk/tests/poseidon2_noir_cross.rs`
  fixture outputs (same poseidon lib tag v0.3.0, same `Poseidon2::hash`,
  length-bearing IV). h2/h3/commit_fold reproduce `0x168758…`, `0x038682…`,
  `0x23864a…`, `0x130bf2…`, `0x0ceb49…`, `0x046964…`.
- `filter_int` operand binding CONFIRMED against `sparq-zk/src/encode.rs` +
  `field.rs`: `field_from_hash_bytes = bytes[1..]` (low 31 bytes BE) matches the
  in-circuit `for i in 1..32 { hs = hs*256 + digest[i] }`; literal token
  `"<digits>"^^<…#integer>` matches oxrdf `Literal::to_string()`.

Verdict: scaffold is GOOD; committed as baseline unchanged. Added `.gitignore`
(target/) and this STATUS.

### Circuit family inventory (compiles)

| member            | K | N  | R | relation        |
|-------------------|---|----|---|-----------------|
| scan_k1_n16_r4    | 1 | 16 | 4 | scan_check      |
| scan_k2_n16_r8    | 2 | 16 | 8 | scan_check      |
| scan_k2_n64_r8    | 2 | 64 | 8 | scan_check      |
| filter_int_d1     | — | —  | — | filter_int (D=1)|
| filter_int_d2     | — | —  | — | filter_int (D=2)|
| filter_int_d4     | — | —  | — | filter_int (D=4)|
| filter_f64        | — | —  | — | filter_f64 (building block, not manifest-composable v1) |
| revoke_unset_d10  | — | —  | — | revoke_unset (D=10, hidden-index revocation; sq-3e5/sq-h2v) |

### Hidden-index revocation (sq-3e5 + sq-h2v) — DONE (representative, depth-10)

[OPUS-4.8] `compose_core::revoke::revoke_unset_check<D>` + `revoke_unset_d10`:
a depth-D Poseidon2 (`h2`) MERKLE inclusion + bit-unset proof. PUBLIC = challenge
+ status-list Merkle root; PRIVATE = index, leaf bit, sibling path. Proves "the
bit at my HIDDEN index is 0 (active)" without disclosing the index or other bits
— closing the clear-index linkability channel. Directions derive from `index`
in-circuit (`to_le_bits`), binding the path to the leaf position and range-bounding
`index < 2^D`. Gate count: 822 (bb gates ultra_honk). Rust side: `revocation.rs`
(dense `merkle_root` / `merkle_witness` / `revoke_prover_toml`, `h2` cross-vector
confirmed identical to the circuit), `manifest::HiddenIndexRevocation` +
`CircuitId::RevokeUnset{depth}`, `verifier::bind_hidden_revocation` (binds the
proof's public root to the relying party's OWN authoritative root — preserves the
audit-#12 re-audit anchor — then bb verify). Tests: 6 nargo + 5 Rust unit + 3
real prove/verify e2e (unrevoked verifies & index-not-public; revoked unprovable;
forged-root rejected). The clear-index `bind_revocation` path is UNCHANGED.

SCOPE (honest): `merkle_root` is a DENSE 2^D-leaf builder and only `d10` (1024
indices) is compiled — a production status list (2^17+) needs sparse/compressed
inclusion (the circuit relation is depth-generic; only the host builder + single
member bound the size). RESIDUAL PRIVACY GAP: `bind_issuer_attestations` still
mandates the clear `RevocationStatus` (issuer-bound `status_ref_digest` embeds the
clear index), so a holder still leaks its index via that mandatory reference even
when presenting the hidden proof. Fully closing it needs the issuer attestation to
bind a COMMITMENT to the index (a `sparq_zk::sig` change) — the ZK-hard part
(circuit + root-binding) is done; this is the follow-up.

### M1 — Rust orchestration crate `crates/sparq-zk-compose` (DONE)

- `manifest`: ProofManifest serde (query, commitments, did:key issuers,
  attributions, EntailmentRegime, BindingMode challenge/holder-PoP,
  CircuitId, RevocationStatus placeholder, SubProof, BindingEdge). Round-trips.
- `build`: commitments + BGP pattern -> scan ProofInputs + witness; circuit-id
  derivation (k,n,r / d) shared by prover + verifier.
- `toml`: Prover.toml emission for every member.
- `driver`: nargo + bb subprocess prover/verifier. KEY FIX: nargo execute
  exits 0 even on failed assertion — detect unsatisfiability by witness-file
  absence. bb prove uses --write_vk (one pass -> proof+public_inputs+vk).
- `verifier`: structural gate (sparq_zk::verify::recheck Q6 bnode guard +
  circuit-id re-derivation + binding-edge field equalities) then bb verify.
- Non-default workspace member (nothing depends on it).

### M2 — tests + benches (DONE)

- `tests/e2e.rs`: 11 tests (10 run + 1 ignored slow). serde round-trip,
  4 structural-tamper cases, 3 witness-gen (incl. false-verdict rejection),
  1 NON-ignored full bb prove->verify->byte-tamper (filter_int_d1), 1 ignored
  full scan manifest prove->verify. All pass.
- `bench/zk-compose/`: gate counts (bb gates ultra_honk) + prove/verify
  wall-clock + README + regen scripts.

## Gate counts (ultra_honk circuit_size)

scan_k1_n16_r4=5958  scan_k2_n16_r8=11011  scan_k2_n64_r8=34379
filter_int_d{1,2,4}=17416 (blake3-block-bound, d-invariant)  filter_f64=3113

## Prove/verify (small e2e, darwin arm64)

filter_int_d1: prove 1.13s, verify 0.16s, proof 14656 B
scan_k1_n16_r4: prove 1.62s, verify 0.95s, proof 14656 B

## DONE / verified

- compose_core: 22/22 nargo tests. compose crate: 10/10 + 1 ignored.
- Poseidon2 + filter_int encoding bit-compatible with sparq-zk (verified).
- wasm byte gate: see commit (must stay 1,643,095).

### Exact next command (if resuming)

    cd /Users/jesght/Documents/GitHub/rdfjs/sparq-zkcompose && cargo test -p sparq-zk-compose
