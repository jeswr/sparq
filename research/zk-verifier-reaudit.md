<!-- [OPUS-4.8] ZK verifier RE-AUDIT (post-remediation, AS LANDED) run + consolidated by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns. -->
# ZK verifier re-audit — post-remediation, AS LANDED

Independent adversarial re-audit of the merged sparq ZK verifier (`crates/sparq-zk`,
`crates/sparq-zk-compose`, `zk/compose`) on `main` **after** all 17 sq-1s2 remediation
commits, run for bead **sq-gbp4**. The only prior soundness audit
(`research/zk-soundness-audit.md`, 2026-06-13) predated every remediation commit and so
audited a verifier that no longer exists. This document re-runs the same 12-finding
adversarial pass against the verifier **as it is now landed**, with the highest scrutiny on
the ~3800-line hand-serialized public-input reconstruction in `verifier.rs` (the prior
audit's CRITICAL #1 — the place a field-order / endianness / arity / omitted-input slip
would silently re-open the whole estate).

This is a READ-ONLY analysis: no `.rs` was edited (sparq-zk is being worked concurrently on
another branch). Verdicts cite `file:line` on current `main`.

## Bottom-line verdict

**The v1 verifier is SOUND as landed for the threat model the prior audit assumed** (a
prover that fully controls its own side, presenting a manifest to a relying party that
supplies external trust anchors). Every one of the prior audit's 12 confirmed findings —
including all 5 CRITICALs — is **CLOSED with code evidence** on current `main`. The
verifier no longer has the three-disjoint-checks-that-never-meet structure the prior audit
condemned: the load-bearing binding gate (`reconstruct_public_inputs` byte-equality against
each proof's `public_inputs`, under the verifier's OWN nonce, with a verifier-recomputed
CANONICAL vk) now exists, is correct against the real circuits, and is anchored by an
EMPIRICAL ground-truth test captured from real `bb prove` output (non-circular). The
issuer-signature, replay/freshness, FILTER-binding, attribution, salt, and revocation gates
are real cryptography anchored on external relying-party inputs, not structural placeholders.

The reversal from the prior audit's "BROKEN — proves essentially nothing" to "sound as
landed" is justified by code that did not exist when that audit ran; this is not a
disagreement with the prior audit (it was correct about the verifier it saw) but a finding
that the remediation actually did the work.

**No finding is STILL-OPEN or RE-OPENED.** Two items are flagged **NEW** and both are
non-soundness (a test-coverage / toolchain-drift gap, and a documented privacy/binding
deferral) — neither admits a forge-and-verify. Recommended beads are listed at the end.

The one caveat a reader must hold: the cryptographic-chain forge tests (`forge_pubinput_*`,
the real bb prove/verify e2e cases) are `#[ignore]`d as slow and require the nargo/bb
toolchain, so closure rests on the code path being correct (verified here by reading it +
the empirical anchor) rather than on those tests running in default CI. Pinning them in a
toolchain-gated CI lane is the subject of sq-1gir (see below).

---

## How the remediation closes the prior structure

The prior audit's core diagnosis was that the verifier was three checks that never met:
stages 1–2 over prover-declared JSON, stage 3 a detached bb proof over a prover-chosen
public-input vector under a prover-chosen vk, with nothing tying them together. The
remediation introduces two interlocking binding layers that close the seam:

1. **JSON → proof (audit #1/#2/#4).** `reconstruct_public_inputs`
   (`verifier.rs:3309-3421`) rebuilds the bb `public_inputs` byte vector from the DECLARED
   `ProofInputs`, in each member's `main` declaration order, using the VERIFIER'S nonce as
   field 0, and `verify_manifest` byte-compares it against the proof
   (`verifier.rs:3237-3240`, `PublicInputMismatch`). The vk is recomputed verifier-side from
   the canonical compiled member keyed on the re-derived `CircuitId`
   (`driver.rs::canonical_vk:240-255`), and `bb verify` runs over (prover proof, OUR
   reconstructed inputs, OUR canonical vk) (`verifier.rs:3247-3262`). The prover's bundled
   vk and public-input bytes are NEVER trusted (the legacy `driver.rs::verify` that trusted
   them is explicitly documented as unused by `verify_manifest`, `driver.rs:293-302`).

2. **query → JSON (audit #5/#6/#7/#8/#10).** `bind_query_correctness`
   (`verifier.rs:2722-2799`) re-parses the query, extracts BGP constants + FILTER
   `(variable, op, bound)` (`verify.rs` `fragment_patterns`/`fragment_filters`/`variable_slots`),
   and requires the declared JSON (now byte-bound to the proofs by layer 1) to match the
   query the relying party reads. `bind_attributions` (`verifier.rs:2826-2863`) ties the
   prover's `manifest.attributions` to each scan's proof-bound per-graph attribution.

Because layer 1 binds JSON→proof and layer 2 binds query→JSON, the chain query→JSON→proof
is closed: a proof of one statement can no longer be presented under a different query, and
a JSON statement can no longer diverge from what the proof attests.

---

## Per-finding dispositions

### 1. (CRITICAL) bb-verified `public_inputs` never reconstructed from / compared to the declared statement — **CLOSED**

`reconstruct_public_inputs` (`verifier.rs:3309-3421`) serializes the public-input field
vector from the declared `ProofInputs` in `main` declaration order and `verify_manifest`
asserts byte-equality with `art.public_inputs` (`verifier.rs:3237-3240`). The serialization
was verified EXACTLY against the Noir circuits' `main` signatures:

- `scan_k{k}_n{n}_r{r}`: challenge, commitments[k], pattern_is_const[3], pattern_const_enc[3],
  rows[r][3] (row-major, padded to r), row_count, attribution[k] — matches
  `zk/compose/scan_k1_n16_r4/src/main.nr:10-16` exactly; all listed params are genuinely
  `pub`, the private `counts`/`enc` are correctly omitted, and there is NO `-> pub` return
  to append (the highest-risk omission — confirmed absent across all members).
- `filter_int_d{d}`: challenge, operand_enc, op, bound, expected — matches
  `zk/compose/filter_int_d1/src/main.nr:10-14`.
- `filter_f64_d{d}`: challenge, operand_enc, op, b_bits, expected — matches
  `zk/compose/filter_f64_d1/src/main.nr:12-16`.

bb's per-element serialization (one 32-byte big-endian field element per public input;
arrays/structs flattened row-major; `bool`→{0,1}; `u32`/`u64`→the integer value, no header,
no length prefix) is modeled correctly by `field_to_be_bytes_32`
(`crates/sparq-zk/src/field.rs:48-58`, left-zero-padded BE) and `push_uint`
(`verifier.rs:3329-3333`). The `op` code mapping is bit-exact between Rust
`FilterOp::code()` (`manifest.rs:54-63`: Lt=0…Ne=5) and the circuit globals
(`compose_core/src/filter_int.nr` OP_LT=0…OP_NE=5).

**Non-circular ground truth:** the unit tests `reconstruct_filter_int_matches_real_bb_public_inputs`
(`verifier.rs:3525-3545`, 160 bytes) and `reconstruct_scan_matches_real_bb_public_inputs`
(`verifier.rs:3556-3616`, 704 bytes) compare the reconstruction against bytes captured from
a REAL `bb prove` (the `probe_scan_public_inputs_hex` test at `e2e.rs:2985-3008` reads
`art.public_inputs` from an actual `prover.prove_in(...)`, not from the reconstruction). So
the byte layout is pinned to bb's actual output, not to itself. A toolchain bump that changes
the serialization breaks these tests loudly — exactly the tripwire the binding needs.

Forge defeated: an honest proof over statement A re-labeled with a false `ProofInputs` B
yields a reconstruction ≠ the proof's `public_inputs` → `PublicInputMismatch`. Tested:
`forge_pubinput_statement_substitution_rejected` (`forge_gates.rs:573`).

### 2. (CRITICAL) bb verify used the prover-supplied vk — **CLOSED**

`verify_manifest` recomputes the canonical vk verifier-side from the compiled member named
by the re-derived `CircuitId` and uses THAT for bb verify (`verifier.rs:3247-3262` →
`driver.rs::canonical_vk:240-255`, which runs `nargo compile` + `bb write_vk` on the
canonical member). The prover's `art.vk` is decoded but flagged `#[allow(dead_code)]`,
never used by the verifier (`verifier.rs:3437-3442`). The legacy bundled-vk `verify` carries
an explicit NOTE that `verify_manifest` does NOT call it (`driver.rs:293-302`). The false
"we recompute the vk from the compiled member" comment the prior audit flagged is now TRUE.
A trivial attacker circuit + attacker vk fails because bb verify runs against the canonical
member vk, not the attacker's.

### 3. (CRITICAL) No issuer-signature / key-set membership check — **CLOSED**

`bind_issuer_attestations` (`verifier.rs:1709`) verifies, for every scan `commitments[g]`, a
REAL cryptographic signature — Schnorr-over-Baby-JubJub `s·G == R + e·pk` with identity-key
rejection and prime-order-subgroup checks (`sparq-zk/src/sig.rs::verify:486-511`) — over the
message `commitment_message_with_status(commitment_fr, salt_fr, status_ref)`
(`verifier.rs:1882-1883`), where `commitment_fr` is the SAME `commitments[g]` value that the
audit-#1 reconstruction byte-binds into the proof (`verifier.rs:3350-3352`). The signing key
MUST be a member of the EXTERNAL relying-party `trusted_key_set` (`verifier.rs:1788-1792`,
`IssuerKeyNotInKeySet`), never `manifest.key_set` (which may only NARROW, never widen — a
declared key outside K is `UntrustedDeclaredKey`, `verifier.rs:1717-1723`). A commitment with
neither a clear attestation nor a hidden-issuer entry is `UnattestedCommitment`
(`verifier.rs:1778-1781`). The suppression forge (drop a triple, recommit) is dead: the
truncated `C(G')` differs, so no valid signature over its message exists. The hidden-issuer
path is genuine crypto — `bind_hidden_issuer_attestations` derives the key-set Merkle root
from the relying party's OWN KeySet, recomputes the signed message itself (never trusting
`hi.message`), byte-binds `[challenge, m, key_set_root]`, and runs bb verify against the
canonical vk (`verifier.rs:2469-2564`). Tested: `forge_commitment_unsigned_rejected`,
`forge_issuer_invalid_signature_rejected`, `forge_issuer_key_not_in_external_k_rejected`
(`forge_gates.rs:221-256`).

### 4. (CRITICAL) No replay / freshness binding — **CLOSED**

`verify_manifest` now takes a verifier-issued `nonce: &VerifierNonce` and a `seen: &dyn
SeenNonces` store (`verifier.rs:3144-3154`). The nonce — NOT the prover JSON — is the
challenge fed into `reconstruct_public_inputs` as public-input field 0
(`verifier.rs:3199, 3237`), so a proof committed under any other challenge cannot byte-match
(closing the single-JSON-substitution rebind the prior audit specifically called out). The
declared `manifest.binding` challenge is additionally required to EQUAL the nonce as a field
element (`verifier.rs:3200-3209`, `NonceBindingMismatch`). Single-use is enforced FIRST,
fail-closed, burn-on-mismatch (`verifier.rs:3185-3187`, `NonceReplay`); the durable
`FileSeenNonces` (flock + fsync, restart-surviving, sq-aih) is the production impl, with
`InMemorySeenNonces` honestly labeled test-only (`verifier.rs:605-686`). Tested:
`forge_nonce_binding_mismatch_rejected`, `forge_nonce_replay_rejected`
(`forge_gates.rs:413-439`).

### 5. (CRITICAL) FILTER operator/bound/verdict never bound to the query's FILTER — **CLOSED**

The query FILTER is parsed into `(variable, op, bound)` (`verify.rs` `fragment_filters` →
`comparison_filter`), and `bind_query_correctness` requires a `filter_int` sub-proof whose
`(op, bound)` equal the query's and whose `expected == true` (`filter_edge_true`,
`verifier.rs:2871-2883`: `f_op.code() == op.code() && *f_bound == bound && *expected`). These
same `op/bound/expected` are folded into the reconstructed public inputs
(`verifier.rs:3403-3408`) and byte-compared, so the binding is cryptographic, not JSON-only.
The exact age=17-vs-`>=18`-proven-as-`17>=17` forge fails: a `>=18` query requires
`op=Ge, bound=18`; a `17>=17` proof declares `bound=17`, failing `*f_bound == bound` →
`UnboundFilter`. FILTERs the binding layer cannot vouch for (float, var-var, arithmetic,
disjunction, non-canonical integer) fail CLOSED as `UnsupportedFragment` rather than being
silently ignored. Tested: `forge_pubinput_verdict_substitution_rejected`
(`forge_gates.rs:599`).

### 6. (HIGH) Binding edge ignored which slot the FILTER constrains / didn't prune by verdict — **CLOSED**

`variable_slots` (`verify.rs`) gives the `(pattern, slot)` the FILTER variable binds to, and
`bind_query_correctness` requires a gating edge whose `from_slot` equals exactly that slot
(`verifier.rs:2769-2784`, `UnmappableFilterVar` if the var binds no scanned column). The
salary-slot-for-age forge fails: pointing `from_slot` at the salary object is not `?age`'s
slot, so no gating edge is found → `UnboundFilter`. Verdict-pruning is the load-bearing part
and is present: EVERY active disclosed row (`0..row_count.min(rows.len())`) of EVERY scan
answering the FILTER's pattern must carry a true-verdict edge at the FILTER slot
(`verifier.rs:2779-2789`); combined with the scan circuit's in-circuit completeness
constraint (`scan.nr:114-138` — every matching active slot MUST be disclosed) and the
proof-bound `expected`, a failing row can neither be silently dropped nor presented as
passing. The empty-result evasion is closed (`any_scan_answered` / no-scan → `UnboundFilter`,
`verifier.rs:2791-2796`).

### 7. (HIGH) Composition seam (operand substitution / kind confusion) was JSON-only — **CLOSED**

Stage 2 still string-equates the scanned slot to the filter `operand_enc`
(`verifier.rs:1586-1604`), BUT both sides are now cryptographically bound: the scan's
`rows[from_row][from_slot]` is in the scan proof's public inputs
(`verifier.rs:3367-3377`) and the filter's `operand_enc` is in the filter proof's public
inputs (`verifier.rs:3404`), both via `to_field()` → `field_to_be_bytes_32`. A passing
stage-2 equality therefore means byte-identical strings → identical field value → identical
32-byte word bound into BOTH proofs. The "attach a filter proof over a different operand
O′" forge fails the audit-#1 byte-compare. (Note: the stage-2 `FieldHex` comparison is a raw
String compare, `manifest.rs:24-26` — a representation mismatch makes stage 2 REJECT
[fail-closed], it cannot create a false accept because the proof-binding underneath is over
the reduced field value.)

### 8. (CRITICAL) Cross-graph bnode join guard driven by prover-declared, proof-unbound attributions — **CLOSED (in-circuit)**

The scan circuit now binds per-row source attribution IN-CIRCUIT: `attribution[g]` is a
`pub` input asserted equal to the true matched-graph bit
(`compose_core/src/scan.nr:143` `assert(attribution[g] == graph_matches, ...)`, the
per-graph "any active slot matches the pattern" accumulator). It is surfaced as a public
input (`scan_k1_n16_r4/src/main.nr:16`), byte-bound by the audit-#1 reconstruction
(`verifier.rs:3398-3401`), and cross-checked by `bind_attributions`: the declared
`manifest.attributions[pi]` must be a SUPERSET of the scan's proof-bound matched-graph set
(`verifier.rs:2849-2856`, `AttributionUnderDeclared`). The `[[0],[0]]` collapse forge is
dead — a graph the proof shows contributing cannot be omitted from the declared set, so the
Q6 obligation gate (`verify.rs::cross_graph_join_obligations`) now operates over a
proof-bound attribution map. Attribution is required PRESENT and EXACTLY `k` bits — no
`serde(default)` omit/short bypass (`verifier.rs:1561-1573`, `AttributionMalformed`).
Tested: `forge_attribution_omitted_rejected`, `forge_attribution_under_declared_rejected`
(`forge_gates.rs:279-298`).

### 9. (HIGH) Cross-graph bnode salt separation unenforced — **CLOSED (in-circuit salt binding deferred, scope-documented)**

A globally-unique per-graph OS-random salt mint at trusted ingest now exists:
`SaltMint::mint` (`crates/sparq-zk/src/ingest.rs:136-149`) draws 32 CSPRNG bytes
(`getrandom`, a real non-dev dependency), folds to a 248-bit field element, and enforces
uniqueness within the session; `ingest_with_mint` mints one fresh salt per named graph
(`ingest.rs:206-227`). The salt is bound under the issuer signature
(`commitment_message_with_status`/`_with_salt`, `sig.rs:284-286, 410-417`), and a
scan-covering attestation MUST carry a salt (`ScanCommitmentSaltMissing`,
`verifier.rs:1842-1847`). The verifier ALSO checks salt-uniqueness across scan-referenced
commitments (`verifier.rs:1906-1932`, `SaltReused`). The encoding achieves the separation:
`encode_term` folds the salt into the bnode leaf (`encode.rs:56-59`), so distinct salts ⇒
distinct bnode encodings while IRIs/literals stay salt-independent. Honest scope caveat
(documented at `ingest.rs:27-38`, NOT a soundness hole): cross-PRESENTATION salt uniqueness
rests on the trusted issuer's CSPRNG entropy + optional registry seeding, since the
verifier's stateless `SaltReused` check only spans one manifest's scans; the in-circuit (vs
verifier-side-clear) salt binding — "fix (b)" — is the deferred privacy upgrade. Adequate
against the audit-#9 threat (a salt-reusing ingester correlating bnodes below the join
layer).

### 10. (HIGH) Query text not bound into the proof (FILTER-add / constant-swap replay) — **CLOSED**

The fix chose cross-checks over a query digest. Constant-swap: `scan_matches_pattern`
encodes each query BGP constant slot and requires byte-equality with the scan's proof-bound
`pattern_is_const`/`pattern_const_enc` (`verifier.rs:2680-2701`); every query pattern must
be bound by some scan (`verifier.rs:2729-2737`, `UnboundPattern`) — an age scan under a
salary query mismatches `pattern_const_enc`. FILTER-add: a query FILTER with no answering
true-verdict edge → `UnboundFilter` (`verifier.rs:2780-2796`). Projection-narrowing and
FILTER-drop remain (correctly, per the original finding) non-breaks. The out-of-fragment
FILTER fail-closed (`UnsupportedFragment`) closes the "silently ignore the FILTER" variant.

### 11. (HIGH) Circuit-id re-derivation trusts declared n (and d); r-bucket relabel slack — **CLOSED (subsumed by #1/#2)**

`derive_id` still trusts the declared `n`/`d` (it cannot derive private graph size,
`verifier.rs:1448-1455`), but this is now bucket-bound by the audit-#1/#2 fixes: `n` does NOT
appear in any public input (it sizes the PRIVATE `enc` array and the `scan_check::<K,N,R>`
generic, `scan_k1_n16_r4/src/main.nr:18-19, 26`), so two members differing only in `n` (the
only such pair compiled: `scan_k2_n16_r8` vs `scan_k2_n64_r8`) have DIFFERENT constraint
systems and therefore different vks. The canonical vk is recomputed for the declared
member's package (`driver.rs::canonical_vk` keyed on the full `CircuitId`), so a proof made
against the n=64 member declaring n=16 is verified against the n=16 canonical vk and bb
REJECTS. The r/k axes are re-derived and required to match (`derive_id` + `CircuitIdMismatch`,
`verifier.rs:1540-1548`), and a wrong r also produces a wrong-length reconstruction that
cannot byte-match. So the n/d/r relabel is closed once #1/#2 are closed, exactly as the
prior audit predicted.

### 12. (MEDIUM) Revocation unimplemented; status-list index disclosed in the clear — **CLOSED (hidden-index proof landed)**

The clear-index liveness gate (`bind_revocation`, `verifier.rs:2158`) is fail-closed:
freshness first (`StatusListStale`), missing authoritative snapshot (`StatusSnapshotMissing`),
revoked bit set (`CredentialRevoked`, reading the relying party's OWN authoritative bytes —
bit-out-of-range fails closed as SET), prover-snapshot disagreement (`StatusSnapshotTampered`,
checked across ALL matching snapshots per roborev #2263). The status reference is issuer-bound
(folded into the signed message, `sig.rs:410`; cross-checked against the issuer-signed
`AttestedStatusRef`, `verifier.rs:2003-2074`, `RevocationReferenceMismatch`/`...Missing`), so
it can't be swapped/omitted. The hidden-index in-circuit inclusion proof has LANDED
(`bind_hidden_revocation`, `verifier.rs:2306`): it derives the Merkle root from the relying
party's OWN snapshot (NOT a prover root), requires the proof's public root to byte-equal it
(`HiddenRevocationRootMismatch`), cross-binds the proof's `index_commitment` to the
issuer-signed one, byte-binds `[challenge, root, index_commitment]`, and runs bb verify; the
in-circuit `bit == 0` assertion makes a revoked credential's proof unsatisfiable. On the
committed-index path the clear index AND liveness bit are hidden (sq-ayv); the list IRI +
version remain disclosed in both paths (a documented privacy residual, NOT a soundness gap,
`verifier.rs:2150-2154`). Tested: `forge_revocation_revoked_bit_rejected`,
`forge_revocation_stale_version_rejected` (`forge_gates.rs:359-382`).

---

## Hardening recommendations from the prior audit — disposition

- **Malformed `proof_hex` → REJECT not panic — DONE.** `hex_decode` and `take_lp` now return
  `Option` and the caller maps `None` to `CheckError::MalformedProof`
  (`verifier.rs:3226-3228, 3465-3505`); the old `.expect("valid hex")`/OOB slice is gone.
- **Document `row_count` semantics — DONE** (the scan module docs state union/RDF-merge
  set-count semantics; the verifier uses it only to size the r bucket and to bound the
  per-row FILTER sweep).
- **Normalize / reject non-canonical integer literals — DONE** (`canonical_u64`,
  `verify.rs`, rejects leading zeros/signs/whitespace, matching `filter_int.nr` tokenization).
- **Bind the true triple count into the signed object — partially addressed** via the
  salt/status-bound commitment message; the count is the commit-fold IV (collision-resistant
  length separation). Acceptable; no forge.
- **Negative e2e tests for the binding gates — DONE** (`forge_gates.rs` +
  `differential_fuzz.rs` cover mismatched-statement, prover-vk, replay, `[[0],[0]]`,
  revoked/stale). The 1:1 finding→test map is sq-1gir (below).
- **Fix the false-assurance comments — DONE.** `verifier.rs:204-209` (the old false
  "recompute the vk" comment) now describes the real canonical-vk recompute; `verify.rs`
  module docs (the old flat "cannot smuggle a correlating join" claim) now correctly qualify
  the verifier-side check as "necessarily coarser" and ground it on the proof-bound
  attribution (audit #8); `prefilter_manifest_structure` carries a prominent "THIS IS NOT A
  VERIFIER" warning (`verifier.rs:1487-1505`).
- **Gate `HolderPop` behind unimplemented OR implement PoP — DONE (implemented +
  fail-closed).** `bind_holder_pop` (`verifier.rs:2918`) verifies a challenge-bound Schnorr
  PoP under a relying-party `HolderRegistry`, fail-closed on empty registry / untrusted
  holder / malformed / invalid (`HolderRegistryEmpty`/`HolderNotTrusted`/`HolderPopMalformed`/
  `HolderPopInvalid`). Honest deferral documented (`verifier.rs:2906-2916`): the PoP proves
  possession of a trusted holder key signing the nonce but does NOT yet bind that key to the
  SPECIFIC credential (issuer-attested holder binding deferred) — so trusted holder A could
  present trusted holder B's credential. Recorded as NEW-2 below.

---

## NEW findings (introduced or surfaced by the remediation)

Neither is a soundness break (no forge-and-verify). Both are recommended as beads.

### NEW-1. (LOW, test/CI) Missing empirical bb anchors for `filter_f64_d*` and the k=2 scan members; the crypto-chain forge tests are `#[ignore]`d

The audit-#1 reconstruction has EMPIRICAL ground-truth byte tests only for `filter_int_d1`
(160 B) and `scan_k1_n16_r4` (704 B) (`verifier.rs:3525-3616`). The `filter_f64_d*` members
and the k=2 scan members (`scan_k2_n16_r8`, `scan_k2_n64_r8`) rely on the (correct, generic)
layout reasoning + the shared bb serialization model, with NO captured golden vector — so a
toolchain bump could silently change their serialization without a failing anchor. Separately,
the cryptographic-chain forge tests (`forge_pubinput_*`, the real bb prove/verify e2e cases)
are `#[ignore]`d as slow and need nargo/bb, so they do not run in default CI; closure of #1–#12
currently rests on code-reading + the two anchors. **Impact:** detection/regression-coverage
gap, not a present hole. **Recommend:** add probe-captured anchors for f64 + k2 members, and a
toolchain-gated CI lane that runs the `#[ignore]`d forge/anchor suite. (Strongly complements
sq-1gir.)

### NEW-2. (LOW, privacy/binding deferral, already documented) HolderPoP is not yet credential-bound; clear-index + list-version are disclosed

Two honestly-documented deferrals that a relying party must understand: (a) `bind_holder_pop`
proves possession of a trusted holder key over the nonce but does NOT bind it to the specific
attested credential (`verifier.rs:2906-2916`), so among trusted holders one could present
another's credential — narrows "who may present at all" but not "whose credential"; (b) on the
non-committed revocation path the status-list index is disclosed, and on BOTH paths the list
IRI + snapshot version are disclosed (`verifier.rs:2150-2154`) — linkability handles. Neither
weakens soundness (a revoked/forged credential still fails; an untrusted holder still fails).
**Recommend:** track the issuer-attested holder-binding upgrade and the remaining disclosure
residuals as privacy beads.

---

## Recommended beads for the orchestrator to create

(Do NOT create here — for the orchestrator. The re-audit found no STILL-OPEN/RE-OPENED
soundness finding, so these are NEW + test-infra hardening.)

1. **[test/CI, P1] Toolchain-gated CI lane for the ZK forge/anchor suite + empirical anchors
   for `filter_f64_d*` and k=2 scan members** (NEW-1). `area:sparq-zk-compose, test, zk`.
   Captures probe golden vectors for the un-anchored members and runs the `#[ignore]`d
   `forge_pubinput_*` / real-bb e2e cases in a nargo/bb-available CI job so audit-#1
   serialization drift and the crypto-chain gates are regression-protected, not just
   code-read. Companion to sq-1gir.
2. **[feature, P2] Issuer-attested holder binding (credential-bound HolderPoP)** (NEW-2a).
   `area:sparq-zk-compose, zk`. Bind the holder key into the credential commitment message so
   the PoP attests possession of THIS credential's holder key, closing the
   trusted-holder-A-presents-B's-credential gap.
3. **[feature, P3] In-circuit salt binding + remaining revocation-disclosure privacy
   residuals** (NEW-1/#9 deferral + NEW-2b). `area:sparq-zk, zk`. The in-circuit (vs
   verifier-side-clear) salt binding ("fix (b)" at `ingest.rs:37-38`) and hiding the list IRI
   / snapshot-version linkability channels.

**Existing bead sq-1gir** already tracks the standing forge-and-verify regression MAP (one
permanent test per historical CRITICAL #1–#12). This re-audit confirms each finding's CLOSED
disposition and its current reject path / error variant (cited per-finding above), giving
sq-1gir the exact 1:1 mapping it needs:

| Finding | Forge | Reject path / error |
|---|---|---|
| #1 | honest proof over a different statement | `PublicInputMismatch` |
| #2 | prover-supplied / trivial-circuit vk | bb verify fails vs canonical vk |
| #3 | unsigned / prover-key / dropped-triple commitment | `UnattestedCommitment` / `IssuerKeyNotInKeySet` |
| #4 | replay / JSON-challenge rebind | `NonceReplay` / `NonceBindingMismatch` / `PublicInputMismatch` |
| #5 | `17>=17` proven for `>=18` query | `UnboundFilter` (bound mismatch) |
| #6 | salary slot as age operand / failing row disclosed | `UnboundFilter` (slot / verdict) |
| #7 | filter proof over a different operand | `PublicInputMismatch` (operand_enc) |
| #8 | `[[0],[0]]` collapse / omitted attribution | `AttributionUnderDeclared` / `AttributionMalformed` |
| #9 | salt reuse across scans | `SaltReused` (+ no issuer sig over reused salt) |
| #10 | FILTER-add / constant-swap | `UnboundFilter` / `UnboundPattern` |
| #11 | n/d/r bucket relabel | `CircuitIdMismatch` / bb verify fails vs canonical vk |
| #12 | revoked / stale credential | `CredentialRevoked` / `StatusListStale` |

---

## Methodology

Read the prior audit (`research/zk-soundness-audit.md`) and the current verifier/prover
estate on `main`: `verifier.rs` (`verify_manifest`, `prefilter_manifest_structure`,
`reconstruct_public_inputs`, `bind_query_correctness`, `bind_attributions`,
`bind_issuer_attestations`, `bind_revocation`, `bind_holder_pop`, the hidden-* gates,
`decode_artifacts`/`hex_decode`/`take_lp`, `derive_id`), `driver.rs`
(`canonical_vk`/`verify_with`/`verify`), `field.rs`, `sig.rs`, `ingest.rs`, the Noir circuits
under `zk/compose/` (`scan.nr`, `scan_k1_n16_r4/main.nr`, `filter_int_d1/main.nr`,
`filter_f64_d1/main.nr`), and the `forge_gates.rs` / `e2e.rs` test seams. The single
highest-risk area (the public-input reconstruction) was verified two ways: (1) the
reconstruction's declaration order/types/sizing were matched line-by-line against each
member's `main` `pub` signature (confirming no omitted `pub` input, no included private
param, no missed `-> pub` return, bit-exact `op` mapping); (2) the byte layout was confirmed
non-circular against the captured-from-real-bb anchor tests. Each prior finding was assigned
CLOSED only when a concrete reject path + (where present) a forge test was found; verdicts
prioritize empirical honesty over reassurance — this is a credential system, and the report
states plainly both that the verifier is now sound as landed AND the two residual
non-soundness gaps (test-coverage/toolchain-drift and privacy/binding deferrals).
