# sparq-zk-compose

ZK proof **composition** for sparq — stage 2 of the query-proof design
(`research/zkp-query-proofs-plan.md` v3, §S4.E modules ii/iii). Drives the
per-property Noir circuit family at [`zk/compose/`](../../zk/compose) into a
full query-result proof and verifies one.

> Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable
> returns). This crate was authored by Opus 4.8 on top of a Fable-authored,
> Opus-validated circuit scaffold.

## FIPS posture (CR-G4)

This crate's cryptography (BN254 / Poseidon2 / Schnorr over Baby-JubJub, via the
`sparq-zk` primitives) is **not** FIPS-approved and is **not** inside any FIPS
140-3 / CMVP-validated module — sparq makes **no FIPS claim and no CMVP claim**.
The primitives are chosen for in-circuit efficiency, not regulatory approval. The
crate is `publish = false` and is not in the default dependency graph, so a
FIPS-constrained operator can keep it **out of FIPS scope** simply by not opting
in. See `compliance/cryptoreview/fips-posture.md`.

## Architecture

```text
sparq-zk (stage 1)                 sparq-zk-compose (stage 2, this crate)
─────────────────────              ──────────────────────────────────────
canonicalize + commit  ──leaves──▶ build::build_scan      ──┐
encode_term/encode_triple          build::build_filter_int   │
verify::recheck (Q6 guard) ◀──────  verifier (re-check)       │
                                                              ▼
                                   driver::CircuitProver  ── nargo + bb
                                   (subprocess proving)      subprocess
                                                              │
                                   manifest::ProofManifest ◀──┘
                                   (public inputs + bb proof bytes)
```

The composition pattern is **modular per-property proofs** (after the
`sparql_noir` / `zkp-sparql-workspace` reference architectures): each BGP
pattern and each numeric FILTER is its own circuit proof; composition is the
verifier checking each sub-proof AND the **binding-consistency edges** between
them (a shared term encoding appearing in both a scan proof's disclosed rows
and a filter proof's `operand_enc` is a plain public-input equality).

### Circuit-family id derivation (dynamic (k, n) sizing)

The prover and verifier BOTH derive the circuit-family member from the proof
shape — never trust a declared id:

- **scan** `scan_k{k}_n{n}_r{r}`: `k` = number of committed graphs, `n` =
  smallest compiled slot-bucket ≥ the largest graph, `r` = smallest compiled
  row-bucket ≥ the disclosed match count. See `build::derive_scan_id`. The lattice
  is `k ∈ {1,2}`, `n ∈ {16,64}`, `r ∈ {4,8}`, and ALL EIGHT `(k,n,r)` combinations
  are now compiled ([OPUS-4.8] sq-pzet wired the remaining five — `k1_n16_r8`,
  `k1_n64_r4`, `k1_n64_r8`, `k2_n16_r4`, `k2_n64_r4` — closing the
  silent-unprovability gap where `derive_scan_id` returned an id with no package).
- **filter_int** `filter_int_d{d}`: `d` = the hidden value's decimal digit count,
  which must EXACTLY equal a compiled member (the circuit's `digits: [u8; d]`
  witness pins the count). Compiled members: `d ∈ {1,2,3,4}` (the contiguous
  1..=4 range; [OPUS-4.8] sq-wto added `d=3`). `build::derive_filter_int_id`
  requires an exact match and returns `None` for any other count (e.g. 5..=19),
  so `build_filter_int` cleanly declines an out-of-family operand rather than
  deriving a wrong-`d` member that would be silently unprovable (sq-wto). NOTE:
  `filter_int` handles the canonical NON-negative integer fragment only; see
  `filter_signed_int` / `filter_decimal` below for the signed + decimal extension.
- **filter_signed_int** `filter_signed_int_d{md}` and **filter_decimal**
  `filter_decimal_i{id}_f{fd}` ([OPUS-4.8] sq-1q9h): extend `filter_int` to
  NEGATIVE `xsd:integer` (`md` = magnitude digit count, plus an optional leading
  `-`) and to `xsd:decimal` (`id` integer digits + `fd` fraction digits, fixed-
  point compared at `fd` places against a host-prescaled bound). Same operand
  binding as `filter_int` — the canonical `"[-]?<lexical>"^^<datatype>` token is
  rebuilt in-circuit and asserted equal to `operand_enc` (oxrdf passes the lexical
  form through verbatim, so the in-circuit token byte-matches the committed one;
  cross-checked against the REAL `sparq_zk::encode` pipeline in
  `tests/filter_signed_binding.rs`). The numeric value is a constrained function
  of the SAME witnessed bytes, so the comparison is sound. `bb gates`: 17416 each
  (blake3-block-bound, identical to `filter_int`). Host encoders:
  `build::encode_signed_int_literal` / `encode_decimal_literal`. As of [OPUS-4.8]
  sq-7lrq these are now MANIFEST-COMPOSABLE: `CircuitId::FilterSignedInt { md }` /
  `FilterDecimal { id, fd }` + the matching `ProofInputs` variants +
  `build_filter_signed_int` / `build_filter_decimal` (returning a
  `FilterSignedWitness` of the private sign + digits) + `prover_toml_for`'s
  `filter_signed_witness` arg + the verifier's binding-edge operand extraction and
  public-input reconstruction (empirically anchored against real `bb prove` output
  in `reconstruct_filter_signed_int_matches_real_bb_public_inputs` /
  `reconstruct_filter_decimal_matches_real_bb_public_inputs`; composable witness
  soundness in `filter_{signed_int,decimal}_composable_witness_honest_proves_lie_rejected`).
  `md` / `(id, fd)` are EXACT-match (an out-of-family shape returns `None`, never a
  silently-unprovable member). The GENERAL fractional/scientific `xsd:double`
  fragment (an in-circuit decimal→IEEE-754 RNE parser over an arbitrary lexical
  form) remains DEFERRED — bead sq-lxi7.
- **join_eq** `join_eq_na{n_a}_nb{n_b}`: the two graph-size buckets select the
  member, exactly as scan's `n` does. The buckets are `{16,64}` (aligned with
  scan's `n`), and ALL FOUR `(n_a,n_b)` combinations are compiled ([OPUS-4.8]
  sq-pzet added the `64` bucket — `na16_nb64`, `na64_nb16`, `na64_nb64` — so a
  hidden join composes with a scan over the `n=64` bucket). See
  `build::derive_join_eq_id`; out-of-family (`> 64`) returns `None`.

The verifier re-derives and rejects on mismatch (`CircuitIdMismatch`), so a
proof can only verify against the member its public inputs fit.

> **NOT-yet-sound (standing caveat — sq-qhy4 / sq-9hrn; remediation epic sq-1s2).**
> sq-pzet only BUILDS the remaining family members (constraint encoding + the
> prove/verify entry points, with prove→verify round-trip + invalid-witness-rejected
> tests per member). It makes NO soundness or zero-knowledge-security claim: the
> composition verifier's soundness is the subject of the open audit (`research/
> zk-soundness-audit.md` / `research/mpc-cozk-reaudit.md`). A passing proof here is
> NOT a guarantee that the SPARQL statement holds under an adversarial prover. The
> full-binding verifier entry point is still `verifier::verify_manifest`, and even it
> inherits the open verifier-soundness question.

### Manifest (`manifest::ProofManifest`)

The public, serializable proof metadata graph (credential model: content lives
in a named graph, the manifest is the proof graph). Carries: the SPARQL query
text (re-parsed, never trusted), per-graph commitments, `did:key` issuer refs,
per-pattern graph attributions, declared non-bnode join obligations, the
`EntailmentRegime`, the `BindingMode` (challenge / holder-PoP), an optional
revocation status-list placeholder, the composed sub-proofs (public inputs +
bb proof bytes), and the binding edges. JSON via serde; round-trips.

### Verification stages (`verifier`)

1. **Structural (no bb, fast):** `sparq_zk::verify::recheck` re-runs the
   stage-1 layer-3 gate (cross-graph bnode-join guard Q6 + attribution arity)
   from the query text; circuit ids are re-derived from public inputs; binding
   edges are checked as field equalities; the query-correctness, issuer-signature
   / key-set, and **per-graph source-attribution** binding gates run here.
   The attribution gate (audit #8) cross-checks `manifest.attributions` against
   the PROOF-BOUND per-graph `attribution[k]` each scan carries (`scan.nr` step 4,
   byte-bound by the audit #1 reconstruction): `manifest.attributions[pattern]`
   must be a superset of the proved matched-graph set, so a collapse-two-graphs
   `[[0],[0]]` lie that would drop a cross-graph bnode obligation is rejected.
   The issuer signature now binds the per-graph salt (audit #9) and the verifier
   rejects two distinct commitments sharing a salt.
2. **Cryptographic:** every sub-proof is verified via `bb verify`.

`verify_manifest_structure` is the fast gate; `verify_manifest` adds bb.

## v1 covers vs defers

**Covered (working e2e):**
- Per-pattern BGP **scan** proofs: in-circuit per-graph Poseidon2 commitment
  recompute (bit-compatible with `sparq-zk`), row soundness, and **scan
  completeness** (the disclosed match set is complete w.r.t. the committed
  graph union — the differentiator over soundness-only prior art).
- Hidden-operand numeric **FILTER** over `xsd:integer` literals
  (`filter_int`): re-derives the literal's term encoding in-circuit (blake3
  blackbox over the canonical N-Triples token) and proves the comparison
  verdict.
- **Manifest** serde, prover driver (nargo/bb subprocess), verifier with the
  sparq-zk re-check and binding-consistency edges.
- Full bb prove→verify on small credential graphs; tamper tests (modified
  result, wrong commitment, swapped operand, cross-graph bnode join, flipped
  proof byte) all fail.

**Composable now ([OPUS-4.8] sq-q7e / sq-tat):**
- **`xsd:double` FILTER composition** — IMPLEMENTED for the INTEGER-VALUED double
  fragment. The composable `filter_f64_d{d}` members (`CircuitId::FilterF64 { d }`
  / `ProofInputs::FilterF64`) bind the hidden operand to the committed literal via
  the canonical `"<digits>"^^xsd:double` token (blake3, the same mechanism as
  `filter_int`) and DERIVE the IEEE bits in-circuit from the bound value
  (`f64::from(value)`, exact for `value < 2^53`), so there is NO prover-free
  `a_bits`. A float FILTER now participates in a composed proof via a binding edge
  to a scan (e2e: `filter_f64_composes_end_to_end`). The raw `filter_f64` building
  block (free bits) remains for non-composed use. PARTIALLY ADVANCED ([OPUS-4.8]
  sq-mslu): the general decimal→IEEE-754 conversion CORE for a FRACTIONAL /
  SCIENTIFIC lexical form now exists as a tested, gate-counted building block —
  `filter_f64_dec` (`sparq_zk_compose_core::filter_float::filter_f64_decimal_check`
  / `decimal_to_f64_integer_valued`). It takes the lexically-parsed operand
  components `(sign, mantissa_int, dexp)` (value = `mantissa_int·10^dexp`) and
  converts to the correctly-rounded double via a SINGLE `f64::from(u128)` op —
  round-to-nearest-even, the IEEE default. This is SOUND for the integer-VALUED
  fragment (`dexp ≥ 0`, exact integer ≤ `u128`), so a fractional/scientific
  LEXICAL form whose VALUE is an integer (`1.5E3`, `100.0`, `1.0E1`) is now
  covered, including RNE rounding above `2^53` (`1E20`). MEASURED at **4380 gates**
  (`bb gates -s ultra_honk`, gate-count snapshot row), well within budget vs the
  17416 of a bound member. STILL DEFERRED, with two distinct remaining pieces:
  (a) the genuinely-FRACTIONAL-VALUE case (`dexp < 0`, e.g. `0.1`) — a composition
  of the library's individually-correctly-rounded ops DOUBLE-rounds and can miss
  the nearest double by 1 ULP, so it is REJECTED in-circuit rather than shipped
  unsound; a sound general parser needs the full strtod core (exact bignum
  `mantissa·2^?` vs `10^|dexp|` with an explicit RNE tie decision — Clinger /
  Eisel-Lemire), unbudgeted; and (b) MANIFEST COMPOSABILITY of even the
  integer-valued-fractional member, which additionally needs to rebuild the
  operand's canonical N-Triples token in-circuit — and the canonical xsd:double
  lexical form is the SHORTEST-ROUND-TRIP print (Grisu/Ryu), so binding it needs
  in-circuit float PRINTING (the "sparq-zk API gaps" item below), distinct from
  parsing. Both are expected to be subsumed by #514's value-lane, which signs the
  IEEE-754 bits directly and removes in-circuit literal parsing/printing entirely.
  Query-text FILTER→float mapping also deferred (sparq-zk `fragment_filters` parses
  only the xsd:integer FILTER fragment); a float FILTER composes via the binding
  edge + its own verified sub-proof.
- **SIGNED `xsd:integer` + `xsd:decimal` FILTER composition** ([OPUS-4.8] sq-7lrq)
  — IMPLEMENTED. The sq-1q9h members (`filter_signed_int_d{md}`,
  `filter_decimal_i{id}_f{fd}`) are now manifest-composable: `CircuitId` /
  `ProofInputs` variants + `derive_*` + `build_*` + `prover_toml_for`
  (`filter_signed_witness` arg) + the verifier binding edge + public-input
  reconstruction (same shape as the f64 work). The numeric value stays a
  constrained function of the witnessed lexical bytes (sound operand binding,
  inherited from sq-1q9h). DEFERRED: the GENERAL fractional/scientific `xsd:double`
  fragment (in-circuit decimal→IEEE-754 RNE parser over an arbitrary lexical form)
  — bead sq-lxi7; MEASURE `bb gates` before claiming it is affordable.

**Deferred (documented, schema-stable placeholders where relevant):**
- **Issuer signatures** over commitments — v1 carries `did:key` refs in the
  manifest but does not verify a signature in-circuit (commitments are
  disclosed manifest fields). The named-graph credential seam is in place.
- **Join consistency across patterns** is verifier-side over disclosed rows
  (the proof-vs-clear dispatch rule); multi-pattern joins are not yet proved
  in a single circuit.
- **Aggregation** — noted subject-to-change per the user; not in v1.
- **Trust framework** (issuer allow-lists, schema governance) — deferred.
- **Revocation** — `RevocationStatus` is a hidden-index status-list
  placeholder; v1 carries the index in the clear and does not check liveness.
- **Inference / entailment** — ENFORCED end-to-end ([OPUS-4.8] sq-314), not free
  metadata. The verifier checks `entailment_regime` against an external
  `EntailmentPolicy` (default `Simple`-only, fail-closed): a regime the policy
  rejects, a `Simple` manifest carrying inference steps, or a non-`Simple` regime
  whose `derivation_steps` do not STRUCTURALLY ground every derived triple to the
  disclosed base (or to an earlier step) all REJECT. The `derivation` module
  ships a `DerivationStep` capability + an RDFS rule subset (rdfs9 subClassOf-type,
  rdfs7 subPropertyOf) the verifier re-checks (`bind_entailment`). DEFERRED: the
  in-circuit ZK proof that an UNDISCLOSED antecedent is in the committed graph's
  closure (the inference-circuit deliverable) — until then a derivation is sound
  only over the DISCLOSED base (an ungrounded antecedent is rejected, never
  assumed). So `Rdfs`/`Owl` is accepted only for disclosed-base derivations under
  an explicit opt-in policy.
- **HolderPoP binding** — IMPLEMENTED ([OPUS-4.8] sq-cwq): the `HolderPop`
  binding carries a holder key + a challenge-bound Schnorr proof-of-possession
  (`pop`), and the verifier checks it FAIL-CLOSED against an external
  `HolderRegistry` (an empty registry, an untrusted holder, or a
  malformed/invalid/replayed PoP all REJECT — no silent-accept of an absent PoP,
  which was the prior placeholder behaviour).
- **Credential↔holder binding (clear-key tier, B1)** — IMPLEMENTED ([OPUS-4.8]
  sq-z8s7, `research/zk-holder-pop-design.md` §3.3 B1): the verifier now binds the
  presenter to the SPECIFIC credential the issuer issued, closing the
  trusted-holder gap (sq-cwq's nonce-only PoP let trusted holder A present trusted
  holder B's credential). When a credential's `CommitmentAttestation` carries an
  issuer-attested `AttestedHolderBinding` (the issuer folded
  `holder_key_digest(hpk)` into the signature via the `ZKSIG_C4`
  `commitment_message_with_holder` message), the verifier cross-checks that the
  PRESENTED holder key (the one the PoP was signed under) hashes to the
  issuer-attested `holder_pk_digest` and FAILS CLOSED on any mismatch
  (`HolderKeyMismatch`). A bearer credential (no holder binding) presented under
  `HolderPop` is rejected (`HolderBindingMissing`) when the relying party's
  `HolderBindingPolicy::require_binding()` mandates binding; the back-compatible
  `HolderBindingPolicy::allow_bearer()` default keeps the sq-cwq registry+PoP
  behaviour for bearer credentials. The issuer-attested digest is anchored in the
  issuer signature (verified under the external trusted `KeySet K`), never a free
  prover JSON field. See `verifier::bind_holder_pop` / `verifier::bind_holder_binding`.
- **Credential↔holder binding (hidden-key tier, B2)** — IMPLEMENTED ([OPUS-4.8]
  sq-c2ql, `research/zk-holder-pop-design.md` §3.3 B2 / step 6), opt-in,
  **NOT-yet-sound** (see below): the hidden-key analogue of B1 — the holder proves
  possession of the matching secret IN ZERO KNOWLEDGE, WITHOUT disclosing `hpk`. A
  manifest may carry `HolderPokProof`s, each a `bb` proof of the `holder_pok`
  circuit member (knowledge of `hsk` with `hpk = hsk·G` and
  `holder_key_digest(hpk) == holder_pk_digest`, `hsk`/`hpk` private). The verifier
  gate `verifier::bind_holder_pok` does NOT trust the proof's public
  `holder_pk_digest`: it reads the digest from the ISSUER-ATTESTED
  `AttestedHolderBinding` covering the PoK's scan-referenced commitment (anchored in
  the issuer signature, the same `ZKSIG_C4` / external-`K` anchor B1 uses),
  reconstructs the proof's public inputs from the verifier nonce + that digest, and
  byte-compares + `bb verify`s — **the binding edge** that ties the proven hidden
  holder key to the issuer-attested credential (a holder who does not hold the
  issuer-bound secret cannot satisfy the PoK, and cannot swap in its own digest
  without breaking the issuer's EUF-CMA signature). Fail-closed reasons:
  `HolderPokUnreferencedCommitment` / `HolderPokBindingMissing` /
  `HolderPokDigestMismatch` / `HolderPokProofRejected` / `HolderPokMalformedProof`;
  under the opt-in `HolderBindingPolicy::require_in_circuit_pok()`, a holder-bound
  credential with no matching PoK is `HolderPokMissing`. The clear-key (B1) path is
  unchanged and remains the default holder gate; B2 is the additive privacy layer.
- **Hidden-holder-SET anonymity tier** — IMPLEMENTED ([OPUS-4.8] sq-3c00), opt-in,
  **NOT-yet-sound** (see below): prove the holder is a MEMBER of a holder set WITHOUT
  revealing WHICH holder — the holder analogue of `hidden_issuer`'s
  `key_set_membership`, reusing the same Poseidon2 Merkle gadget + the `holder_pok`
  key-pair / on-curve / identity / `< L` gadgets. The privacy upgrade over B2's
  clear-digest `holder_pok` (which makes `holder_pk_digest` public, so a verifier
  still learns the holder is the specific hidden-key party). A manifest may carry
  `HolderSetProof`s, each a `bb` proof of the `holder_set_d{depth}` member (knowledge
  of `hsk` with `hpk = hsk·G` AND `holder_key_digest(hpk)` a Merkle member of the
  committed holder set; `hsk`/`hpk`/index/path all private — only `holder_set_root`
  is public). The verifier gate `verifier::bind_holder_set` does NOT trust the
  proof's public `holder_set_root`: it recomputes the AUTHORITATIVE root from its OWN
  `HolderRegistry` (opt in with `HolderRegistry::with_hidden_holder_set_depth`),
  reconstructs the public inputs from the verifier nonce + that root, byte-compares +
  `bb verify`s — so "in the set" is bound to the relying party's holder registry,
  only WHICH holder is hidden. Fail-closed reasons: `HolderSetNotEnabled` /
  `HolderSetDepthMismatch` / `HolderSetRootUnavailable` / `HolderSetRootMismatch` /
  `HolderSetUnreferencedCommitment` / `HolderSetProofRejected` /
  `HolderSetMalformedProof`. The clear/clear-digest holder paths are unchanged; this
  is the additive hidden-holder privacy layer.

## sparq-zk API gaps noted

Consuming sparq-zk's public API (`encode`, `commit`, `field`, `verify`)
required no internal rewrites. Two precise notes for stage-1 maintainers:

1. **No numeric value lane in the encoding.** `encode_term` commits a literal
   as `h2(LITERAL, blake3(token))` — a *string* hash, so a committed numeric
   value has no field-arithmetic binding. `filter_int` closes this by
   re-deriving the blake3 token binding in-circuit (a deliberate, measured
   exception to "never hash strings in-circuit", ~17.4k gates). A future
   additive `sparq-zk` API exposing a numeric value lane (or canonical-lexical
   helper) would let `filter_f64` compose without in-circuit float printing.
2. **`field.rs` doc comment is stage-1-scoped.** It states "the circuit never
   re-derives [`h_s`] from bytes"; stage 2 deliberately does, for the filter
   operand binding. Not a bug — the truncation contract (`bytes[1..]`, low 31
   bytes BE) is what makes the in-circuit re-derivation match. Worth a
   one-line note in that comment when convenient.

No additive `pub` items were required in `sparq-zk` for v1.

## Build & test

```sh
# fast tests (serde, structural verification, tamper) — no toolchain needed:
cargo test -p sparq-zk-compose

# the toolchain-gated tests (witness-gen + one full prove+verify+tamper) run
# automatically when nargo + bb are on PATH; they skip cleanly otherwise.

# the slow full-scan manifest prove→verify:
cargo test -p sparq-zk-compose -- --ignored

# circuit unit + adversarial tests:
cd zk/compose && nargo test --package sparq_zk_compose_core
```

`sparq-zk-compose` is a **non-default** workspace member: like `sparq-zk`,
nothing in the workspace depends on it, so default builds and the wasm artifact
are byte-identical with or without it.

### Concurrency note

Subprocess proving shares one `target/<pkg>.gz` witness and (by default) one
`Prover.toml` per circuit package, so concurrent provers targeting the **same**
family member would race on those files. To make this safe under default
parallel `cargo test`, the driver exposes tag-isolated entry points —
`gen_witness_tagged` / `prove_in` — which write `Prover_<tag>.toml` (selected via
`nargo execute --prover-name`) and `target/<pkg>_w_<tag>.gz`; bb artifacts land in
the caller's own `out_dir`. Concurrent toolchain-gated tests pass a per-test
`tag`, so they no longer need to target distinct members or run with
`--test-threads=1`. <!-- [OPUS-4.8] roborev job 2180 -->. The untagged `prove`
/`gen_witness` wrappers keep the legacy shared names and are only safe
single-threaded against a given member.

## Reproduction (benchmarks)

See [`bench/zk-compose/`](../../bench/zk-compose) for gate counts and
prove/verify timing, with regeneration scripts.
