# ZK verifier soundness remediation — STATUS

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

## DONE
- Read audit + test-bench design + circuits + verifier/driver/build/toml/manifest.
- Determined bb public_inputs byte format empirically (above).
- Measured vk recompute determinism + timing; chose recompute-at-verify.
- #1 IMPLEMENTED: `reconstruct_public_inputs()` + byte-compare in
  `verify_manifest`; 2 unit tests pin it to REAL bb blobs (byte-match), +3
  hardening/sensitivity unit tests. Commit 6c236dd.
- #2 IMPLEMENTED: `CircuitProver::canonical_vk()` + `verify_with()`;
  `verify_manifest` uses the recomputed canonical vk, never `art.vk`. Commit
  6c236dd.
- Hardening: `hex_decode`/`take_lp`/`decode_artifacts` -> `Option`, routed
  through `CheckError::MalformedProof`/`MalformedField`/`PublicInputMismatch`.
- Existing 10 e2e + new 5 unit tests green.

## IN-FLIGHT
- Forge-and-verify NEGATIVE e2e tests (toolchain-gated): honest proof of
  statement A under manifest B => REJECT (#1); prover non-canonical vk =>
  REJECT (#2); + positive control through verify_manifest.

## NEXT COMMAND
- cargo test -p sparq-zk -p sparq-zk-compose --release -- --include-ignored
