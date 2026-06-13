<!-- [OPUS-4.8] ===================================================================
     ACTIVE SESSION: sparq-mpc Milestone 0 (scaffold + per-holder local eval).
     The ZK-remediation STATUS below is a PRESERVED prior-session handoff — left
     intact for crash-resilience; this MPC section is the current work.
     Worktree off live main (base 72722b3). DO NOT PUSH.
========================================================================= -->
# sparq-mpc — Milestone 0 — STATUS (current session)

## Done
- New native-only crate `crates/sparq-mpc` added to root `Cargo.toml` workspace
  members. NOT added to `sparq-wasm` deps — wasm bundle untouched.
- Module tree (each docstring cites architecture §):
  - `partial.rs` — `HolderId`, `PartialResult`, `MpcError` (the single honest
    `NotYetImplemented { what, gated_on }` deferral channel).
  - `holder.rs` — **REAL** per-holder local SPARQL sub-evaluation via
    `sparq-engine` (`Holder::evaluate_local`); 3 unit tests.
  - `backend.rs` — `MpcBackend` trait + `TrustModel`/`BackendInfo` (Q2 decision
    point; no primitive chosen).
  - `join.rs` — `GlobalJoin` trait + `JoinPlan` (global-IRI join; impl deferred M2).
  - `proof.rs` — `CollaborativeProof`/`Attestation` + `ProofStatement` (gated on
    ZK foundation #3/#4/#5/#6/#8/#9/#12 + Q1; honest stubs; 2 contract tests).
- `crates/sparq-mpc/PLAN.md` — M0…M6, Q1/Q2/Q3/Q4 decision points, hard dep on
  the ZK foundation.

## Build / test gate
- `cargo build -p sparq-mpc` — clean.
- `cargo test -p sparq-mpc` — clean.
- `cargo tree -p sparq-wasm` — does NOT contain sparq-mpc (verified).

## Stubbed pending foundation / forks (NO fake crypto)
- `MpcBackend` crypto methods → `NotYetImplemented` (M3 + Q2).
- `GlobalJoin::join` → `NotYetImplemented` (M2; disclosed-key path first).
- `CollaborativeProof` + `Attestation` → `NotYetImplemented` (M1 ZK foundation
  + Q1 spike at M4).

## Disk
- `df` at start: 275G free on `/`. No `/tmp` scratch produced.

---

# ZK verifier soundness remediation — STATUS (PRESERVED prior session)

Model: Opus 4.8 (Fable 5 unavailable — re-review when Fable returns).
Worktree: isolated off `main`. Do NOT push/merge (orchestrator merges).

Mission: remediate the two load-bearing CRITICAL soundness gaps in the v1 ZK
verifier (audit issues #1 + #2), folding in #7 and #11 insofar as #1/#2 enable
them; plus the cheap malformed-`proof_hex` hardening.

## DESIGN

### bb `public_inputs` byte format (determined EMPIRICALLY)

Probed by `bb prove --write_vk -t noir-recursive` on the already-compiled
`filter_int_d1` and `scan_k1_n16_r4` members and hexdumping the emitted
`public_inputs` blob (bb 5.0.0-nightly.20260324, nargo 1.0.0-beta.21):

- **Each public input is exactly one 32-byte big-endian field element.** No
  header, no length prefix, no separator. The blob length is always
  `(#public field elements) * 32`.
- **Order = `main` parameter declaration order**, with structs/arrays
  **flattened in index order (row-major)**. A `[[Field;3];R]` rows array emits
  `R*3` consecutive 32-byte words: row 0 slot 0,1,2, then row 1, …
- **Type lowering:** `bool` → 32-byte `0` / `1`; `u32`/`u64` → the integer value
  as a 32-byte big-endian field element; `Field` → the field element's 32-byte
  big-endian repr (exactly `sparq_zk::field::field_to_hex` minus the `0x`).

Verified against the two members:

- `filter_int_d1` (5 pub fields → 160 bytes): `challenge`, `operand_enc`, `op`,
  `bound`, `expected`. Probe: challenge=0x2a, operand_enc=0x0831…943b, op=0,
  bound=10 (0x0a), expected=1. Matched byte-for-byte.
- `scan_k1_n16_r4` (21 pub fields → 672 bytes): `challenge`,
  `commitments[1]`, `pattern_is_const[3]`, `pattern_const_enc[3]`,
  `rows[4][3]` (=12 words, row-major), `row_count`. Matched byte-for-byte
  (row 0 = the one active match, rows 1–3 = zero words, row_count=1).

This is the field vector the verifier reconstructs from the **declared
`ProofInputs`** (using the verifier's own challenge) and byte-compares against
`art.public_inputs`. Reconstruction order is the single source of truth in each
`zk/compose/<member>/src/main.nr` and is mirrored 1:1 by `toml.rs`.

### vk authenticity: recompute-at-verify-time (chosen) vs pinned store

Measured (ACIR already compiled): `bb write_vk` = **~40–60 ms**; full
`nargo compile` (cold ACIR) + `bb write_vk` = **~350 ms**. Both are fully
**deterministic**: `bb write_vk` over a freshly-recompiled ACIR produces a
byte-identical vk to the original `bb prove --write_vk` output (verified by
`cmp`).

**Chosen:** recompute the canonical vk verifier-side from the compiled member
named by the re-derived `CircuitId` (`nargo compile` if the ACIR is stale, then
`bb write_vk`) and pass THAT vk to `bb verify` — never `art.vk`. Cheap enough
(<<1s) and needs no separate provisioning; determinism means a content-addressed
store keyed by CircuitId is a drop-in later optimisation if compile latency ever
matters. The prover-supplied `art.vk` is dropped (and additionally byte-compared
in a negative test to prove a non-canonical vk is rejected).

This pins the vk to the FULL re-derived `CircuitId` (k,n,r / d), which is what
subsumes audit #11 (n/d/r relabel): a proof produced by a different family
member has a different vk and fails `bb verify`.

### How #1/#2 subsume #7 and #11

- #7 (operand-slot / kind confusion at the scan→filter seam): once each
  sub-proof's `operand_enc` (scan disclosed row slot AND filter operand) is part
  of the byte-compared public-input vector, the stage-2 JSON equality is now an
  equality over **bb-bound** values, not declared JSON. (The deeper FILTER
  *semantics* binding — which slot the FILTER variable maps to, verdict pruning
  — is #5/#6, deferred to a later agent.)
- #11 (n/d/r relabel): vk pinned to the full CircuitId (see above).

### Seams left for later agents (designed to fold in cleanly)

- **#4 replay/freshness:** `verify_manifest` now reconstructs the challenge into
  field 0 of every vector from `manifest.binding.challenge()`. The next agent
  adds a `nonce: &FieldHex` param and asserts it == the binding challenge before
  reconstruction (and a seen-nonce store). The byte-binding is already done; only
  the freshness *source* + single-use remain.
- **#5/#6 FILTER semantics + #10 query digest:** the reconstructed vector already
  byte-binds op/bound/expected/operand_enc; a later agent parses the query FILTER
  to `(var, op, const)`, maps the var to a scanned slot, and cross-checks against
  the now-bound values (no new crypto seam needed).
- **#3 issuer sig, #8/#9 attribution/salt:** orthogonal to #1/#2; untouched.

## DONE (all committed; do NOT push — orchestrator merges)
- Read audit + test-bench design + circuits + verifier/driver/build/toml/manifest.
- Determined bb public_inputs byte format empirically (above).
- Measured vk recompute determinism + timing; chose recompute-at-verify.
- #1 IMPLEMENTED (commit 6c236dd): `reconstruct_public_inputs()` + byte-compare
  in `verify_manifest`. 2 unit tests pin it to REAL bb blobs (byte-match for
  filter_int_d1 + scan_k1_n16_r4), +3 sensitivity/hardening unit tests.
- #2 IMPLEMENTED (commit 6c236dd): `CircuitProver::canonical_vk()` +
  `verify_with()`; `verify_manifest` recomputes the canonical member vk and uses
  it, never `art.vk`. Removed the false vk-recompute comment (was verifier.rs
  204-209). vk pinned to FULL CircuitId => subsumes #11; operand_enc now in the
  byte-compared vector => subsumes #7.
- Hardening (commit 6c236dd): `hex_decode`/`take_lp`/`decode_artifacts` ->
  `Option`, routed through `CheckError::MalformedProof`/`MalformedField`/
  `PublicInputMismatch` (no panic on prover-controlled bytes).
- Forge-and-verify NEGATIVE e2e tests (commit 5c40f14, toolchain-gated):
  statement-substitution + verdict-substitution + challenge-rebind => REJECT
  (#1); a trivial attacker-circuit proof under its own vk => REJECT via
  canonical vk (#2); art.vk-is-ignored corollary; + positive control. The exact
  audit #2 attack was reproduced and confirmed defeated.
- wasm dep tree confirmed FREE of sparq-zk* (no regression).

## TEST RESULTS (cargo test -p sparq-zk -p sparq-zk-compose --release, threads=1)
- sparq-zk lib: 25 passed (incl. new field be_bytes_32 test)
- sparq-zk integration suites: 4 + 2 + 7 + 3 passed
- sparq-zk-compose lib: 5 passed (reconstruction byte-match + hardening)
- sparq-zk-compose e2e: 16 passed, 1 ignored (slow scan); the ignored
  full_manifest_prove_verify_scan also passes when run with --ignored
- 0 failed across the gate.

## DEFERRED (designed to fold in — see DESIGN "Seams")
- #4 replay/freshness (fresh-nonce param + single-use store): challenge is now
  byte-bound into field 0; only the freshness SOURCE + seen-nonce store remain.
- #5/#6 FILTER semantics + #10 query digest: op/bound/expected/operand_enc are
  now byte-bound; a later agent maps the query FILTER var->slot and cross-checks.
- #3 issuer signature, #8/#9 attribution/salt, #12 revocation: orthogonal to
  #1/#2; untouched. The scan.nr replay note + verify.rs:20-23 join-safety
  comment are left for the #4/#8 agents (the guarantee they describe is still
  not delivered, so removing them now would be its own false-assurance edit).

## RESIDUAL NOTE on #11 (n/d)
`n` cannot be re-derived from public data (graph size is private) and `d`
round-trips its bucket; the member is selected by the declared (stage-1b-checked)
CircuitId. This is sound because the canonical vk is recomputed for THAT full id
— a proof from a different compiled member fails bb verify against it (demonstrated
by forge_reject_noncanonical_vk). Per the audit this relabel is otherwise
bucket-invariant (same statement). No further action needed for #1/#2 scope.

## TEST-ISOLATION FIX (roborev codex job 2180, Medium) — [OPUS-4.8]
The toolchain-backed prove/witness e2e tests shared one `Prover.toml` + one
`target/<pkg>_w.gz` witness per Noir member, so default parallel `cargo test`
could interleave them and prove/verify the WRONG statement (only `--test-threads=1`
was reliable). Fixed by threading a unique per-test `tag` through the prove path:
new `CircuitProver::gen_witness_tagged` / `prove_in` write `Prover_<tag>.toml`
(selected via `nargo execute --prover-name`) and `target/<pkg>_w_<tag>.gz`; bb
artifacts already landed in the caller's isolated `out_dir`. All d1-sharing tests
(forge_*, full_prove_verify_filter_int_d1, witness_gen_*) now pass a distinct tag.
`zk/compose/.gitignore` ignores the generated `Prover*.toml`. e2e now passes under
DEFAULT parallelism (no --test-threads=1), confirmed over 3 consecutive runs:
`test result: ok. 16 passed; 0 failed; 1 ignored` (~4.5-7.3s).
