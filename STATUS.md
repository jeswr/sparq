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

## PHASE 2 — QUERY-CORRECTNESS FILTER BINDING (#5/#6/#7 + achievable #10) [OPUS-4.8]
DONE (committed; do NOT push). Verifier-side only — NO circuit recompile (the
values are ALREADY bound by #1's reconstruct_public_inputs; this is the missing
verifier-side check that the bound values MATCH the query the RP reads).

### What landed
- `sparq-zk::verify` (commit 1): `FilterCmp` (neutral cmp op, `code()` ==
  filter_int OP_*), `QueryFilter { variable, op, bound }`, `fragment_filters()`
  (independent spargebra parse, normalizes `?var op const`, flattens AND,
  FAILS CLOSED on any unbindable FILTER — float/string/var-var/disjunction/
  arithmetic/non-canonical-integer), `variable_slots()` (var -> (pat,slot)),
  `fragment_pattern_consts()` (per-pattern constant Terms; returns oxrdf::Term so
  compose needs no sparq-engine dep), `canonical_u64()` (rejects leading-zero/
  signed ints, mirrors filter_int.nr digit-token constraints). +5 unit tests.
- `sparq-zk-compose::verifier` (commit 2): `verify_manifest_structure` gains
  stage 2b + 2c (run BEFORE bb, in the fast gate so a crypto failure can't mask
  them). New CheckError: `UnboundPattern` / `UnboundFilter` / `UnmappableFilterVar`.
  - 2b (#10 constant-swap): every query BGP pattern's constant slots must be
    bound by some scan sub-proof. `scan_matches_pattern` re-encodes the query
    constants (salt-independent for IRI/literal, `encode_term(.., Fr::from(0))`)
    and equates to the bb-bound `pattern_is_const`/`pattern_const_enc`.
  - 2c (#5/#6/#7 FILTER): each query FILTER `?v op c` is gated PER ROW — for
    EVERY scan answering the pattern `?v` binds in, EVERY active disclosed row
    (0..row_count) must have a binding edge to a filter_int sub-proof s.t. (1)
    the edge's scan answers the pattern (scan_matches_pattern), (2) `from_slot`
    == `?v`'s slot in that pattern (#6), (3) the filter's bound `(op,bound)`
    EQUAL the query's `(op,c)` (#5), (4) `expected==true` (verdict gating). Any
    ungated/false row, or no scan answering the pattern, => REJECT (#10
    FILTER-add). Per-row gating is the load-bearing part: the disclosed result IS
    the scans' rows, so a multi-row scan cannot disclose a FAILING row while only
    proving the passing one (a true #6 gap closed — see test
    `filter_reject_unproven_failing_row`).

### FILTER var->slot mapping APPROACH (the requested detail)
A FILTER variable is mapped to a *concrete scanned slot* WITHOUT a trusted
pattern->sub-proof index in the manifest:
  1. `variable_slots(query_patterns)` gives every `(variable, pattern_idx,
     slot_idx)` from an independent query parse.
  2. For FILTER `?v op c`, take `?v`'s `(pattern_idx, slot_idx)` positions.
  3. A binding edge satisfies it iff its `from_proof` scan's bound constants
     MATCH query pattern `pattern_idx` (so the scan is *identified as that
     pattern's scan by its constants*, not by a trusted index — this is exactly
     the #10 cross-check) AND `edge.from_slot == slot_idx` AND the `to_proof`
     filter's `(op,bound,expected)` match `(op, c, true)`.
The scan that answers a pattern is thus pinned by its (bb-bound) constants, and
the operand column is pinned to the FILTER variable's actual slot in that
pattern — closing the "point the operand at the salary slot for an age filter"
forge (#6) and the operand-substitution seam (#7, via stage-2's now-bb-bound
operand_enc equality + this slot check).

### FORGES NOW REJECTED (test names + `test result:` line)
All in `crates/sparq-zk-compose/tests/e2e.rs`; the 6 FILTER negatives are
STRUCTURAL (no toolchain) so they run in minimal CI and can't be masked by a
later bb failure:
- `filter_reject_comparison_substitution_17_vs_18` — the headline age-17-vs-`>=18`
  forge: a filter_int over (Ge, bound=17, expected=true) does NOT satisfy a query
  FILTER(?o>=18) => UnboundFilter(o). (#5)
- `filter_reject_filter_add_on_scan_only` — scan-only manifest under a
  FILTER-carrying query => UnboundFilter(o). (#10 FILTER-add)
- `filter_reject_constant_swap_age_as_salary` — age scan under a <salary> query
  => UnboundPattern(0). (#10 constant-swap)
- `filter_reject_operand_slot_substitution` — FILTER(?age>=65) with the edge
  pointing at the SALARY object slot => UnboundFilter(age). (#6)
- `filter_reject_false_verdict_row` — expected=false row presented as passing =>
  UnboundFilter(o). (#5/#6 verdict gating)
- `filter_reject_unbindable_filter_fragment` — string-literal FILTER fails closed
  => Sparqzk(UnsupportedFragment). (#10 fail-closed)
- `filter_reject_unproven_failing_row` — 2-row scan (age 25 passes, 15 fails)
  with a true-verdict proof only for row 0 => UnboundFilter(o). (#5/#6 per-row)
- `filter_binding_happy_path_structure` + `filter_two_rows_both_gated_verifies`
  — correct composed FILTER manifests (1- and 2-row, every row gated) pass.
- #1/#2 forge tests preserved (now carry an honest scan at idx 0; forged FILTER
  at idx 1): forge_reject_statement_substitution / _verdict_substitution
  (PublicInputMismatch{proof:1}), _challenge_rebind (proof:0, scan fails first),
  _noncanonical_vk (ProofRejected{proof:1}), _artvk_is_ignored, _positive.
Gate (default threads): `cargo test -p sparq-zk -p sparq-zk-compose --release`:
  sparq-zk lib `test result: ok. 30 passed; 0 failed`
  sparq-zk integration: 4 / 2 / 7 / 3 passed
  sparq-zk-compose lib `5 passed`
  sparq-zk-compose e2e `test result: ok. 25 passed; 0 failed; 1 ignored`
  0 failed across the gate (confirmed stable over multiple runs; e2e ~6.4s).
  Ignored slow `full_manifest_prove_verify_scan` also passes with --ignored.

### DEFERRED-TO-CIRCUIT (empirical-honesty deferral, NOT faked)
A FULL canonical-query-digest-as-public-input is NOT done — it needs a CIRCUIT
change (a new `query_digest: pub Field` per member => nargo recompile + new vk),
which is out of scope for this verifier-only phase. RESIDUAL forge it would
additionally close: the verifier-side cross-checks above are sound against the
*current* manifest shape (constants + FILTERs are matched by value), BUT they do
NOT bind the query's **projection/variable-naming** or **pattern ORDER/multiplicity**
into the proof. Concretely the residual is narrow: a query whose BGP constants
and integer FILTERs are byte-identical to an honest one but differs only in (a)
the SELECT projection list, or (b) duplicate/reordered patterns that re-use the
same constants, is not distinguished by 2b/2c (each pattern still finds *a*
matching scan; projection is not part of any bound value). Projection-NARROWING
and FILTER-DROP remain true-statement directions (not forgeries, per the audit).
The dangerous directions (FILTER-add, comparison-/constant-/operand-swap,
verdict mis-gating) ARE closed by this phase. A query digest would make the
binding total (exact-query, not value-wise); recommend it as the next circuit-
touching deliverable. NO circuit files were changed in this phase.

## DEFERRED (designed to fold in — see DESIGN "Seams")
- #4 replay/freshness (fresh-nonce param + single-use store): challenge is now
  byte-bound into field 0; only the freshness SOURCE + seen-nonce store remain.
- #5/#6/#7 + achievable #10: DONE this phase (above). The query-DIGEST part of
  #10 is deferred-to-circuit (residual noted above).
- #3 issuer signature, #8/#9 attribution/salt, #12 revocation: orthogonal;
  untouched. The scan.nr replay note + verify.rs:20-23 join-safety comment are
  left for the #4/#8 agents (the guarantee they describe is still not delivered,
  so removing them now would be its own false-assurance edit).

## STILL NOT FULLY SOUND AFTER THIS PHASE
This phase closes the FILTER/query-result *correctness* binding (#5/#6/#7 +
achievable #10). The verifier is STILL NOT fully sound: #3 (issuer-signature /
key-set membership — commitments are unsigned prover-chosen values; the prover is
still effectively the issuer of every fact), #4 (replay/freshness — no fresh
nonce / single-use), #8/#9 (cross-graph bnode attribution/salt are proof-unbound),
and #12 (revocation unimplemented) all remain. Do NOT present the verifier as
proving credential provenance, freshness, or non-revocation to a relying party.

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

## FILTER `!=` BINDING (roborev codex job 2207, Medium) — [OPUS-4.8]
`FilterCmp::Ne` had an op code (5) and was mapped through the compose layer, but
the verifier-side FILTER parser (`collect_filter_expr` in `crates/sparq-zk/src/
verify.rs`) never recognised SPARQL `!=`. spargebra has NO dedicated `NotEqual`
node — `?v != c` parses to `Expression::Not(Box::new(Expression::Equal(a, b)))`
(confirmed by probing the vendored spargebra: `Not(Equal(Variable, Literal))` for
`?v != c`, `Not(Equal(Literal, Variable))` for `c != ?v`, `Not(Equal(Var, Var))`
for var-var). So every valid integer `!=` FILTER fell through to the fail-closed
reject path and was UNSUPPORTED. Fixed by matching `E::Not(inner)` and, only when
`inner` is `E::Equal(a, b)`, routing through the existing `push_cmp(FilterCmp::Ne,
a, b)` (which reuses `comparison_filter`'s var/const + canonical-integer vetting,
both operand orders; `Ne` is symmetric so the flip is a no-op). Any other `Not`
payload (`Not(Greater)`, `Not(And)`, `Not(Equal(?a,?b))`, non-integer/non-canonical
operands) STILL fails closed — `!=` widens coverage without loosening the fragment.
Tests: 3 unit (`extracts_not_equal_filter_in_both_operand_orders`,
`not_equal_flattens_with_conjoined_comparisons`, `unbindable_not_equal_filters_
fail_closed`) + 3 e2e (`witness_gen_filter_int_ne_satisfiable`, `witness_gen_
filter_int_ne_rejects_false_verdict`, `full_prove_verify_filter_int_ne_d1` — full
bb prove+verify of an honest `!=` plus a tampered-byte rejection, tag-isolated).
Gate `cargo test -p sparq-zk -p sparq-zk-compose --release` (DEFAULT threads) green
over 2 consecutive runs: sparq-zk lib `33 passed; 0 failed`; compose e2e `28 passed;
0 failed; 1 ignored` (~6.8s).
