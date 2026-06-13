# sparq-zk-compose

ZK proof **composition** for sparq — stage 2 of the query-proof design
(`research/zkp-query-proofs-plan.md` v3, §S4.E modules ii/iii). Drives the
per-property Noir circuit family at [`zk/compose/`](../../zk/compose) into a
full query-result proof and verifies one.

> Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable
> returns). This crate was authored by Opus 4.8 on top of a Fable-authored,
> Opus-validated circuit scaffold.

## Architecture

```
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
  row-bucket ≥ the disclosed match count. See `build::derive_scan_id`.
- **filter_int** `filter_int_d{d}`: `d` = the hidden value's decimal digit count,
  which must EXACTLY equal a compiled member (the circuit's `digits: [u8; d]`
  witness pins the count). Compiled members: `d ∈ {1,2,3,4}` (the contiguous
  1..=4 range; [OPUS-4.8] sq-wto added `d=3`). `build::derive_filter_int_id`
  requires an exact match and returns `None` for any other count (e.g. 5..=19),
  so `build_filter_int` cleanly declines an out-of-family operand rather than
  deriving a wrong-`d` member that would be silently unprovable (sq-wto).

The verifier re-derives and rejects on mismatch (`CircuitIdMismatch`), so a
proof can only verify against the member its public inputs fit.

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
  block (free bits) remains for non-composed use. DEFERRED: the GENERAL fragment
  (fractional/scientific lexical forms, rounded values) needs a full in-circuit
  decimal→IEEE-754 parser with round-to-nearest-even over an arbitrary lexical
  form — unbudgeted; the integer-valued fragment is enforced by the digit-only
  token (an out-of-fragment operand is unprovable, never mis-bound). Query-text
  FILTER→float mapping also deferred (sparq-zk `fragment_filters` parses only the
  xsd:integer FILTER fragment); a float FILTER composes via the binding edge +
  its own verified sub-proof.

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
  binding now carries a holder key + a challenge-bound Schnorr proof-of-possession
  (`pop`), and the verifier checks it FAIL-CLOSED against an external
  `HolderRegistry` (an empty registry, an untrusted holder, or a
  malformed/invalid/replayed PoP all REJECT — no silent-accept of an absent PoP,
  which was the prior placeholder behaviour). DEFERRED: binding the holder key to
  a SPECIFIC credential (an issuer-attested credential↔holder binding) — until
  then the registry narrows "who may present" to authorised holders but a trusted
  holder is not yet tied to a particular credential. See
  `verifier::bind_holder_pop`.

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
