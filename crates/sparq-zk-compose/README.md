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
- **filter_int** `filter_int_d{d}`: `d` = digit count of the hidden value.

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
   edges are checked as field equalities.
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

**Deferred (documented, schema-stable placeholders where relevant):**
- **`xsd:double` FILTER composition** — `filter_f64` is a gate-counted, tested
  comparison building block, but binding an `f64` to a committed literal needs
  in-circuit float→canonical-decimal printing (unbudgeted). The comparison
  itself (`sparq_ieee754`, IEEE/NaN-correct) is done.
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
- **Inference / entailment** — only `EntailmentRegime::Simple` is proved.
- **HolderPoP binding** — schema field reserved; v1 uses `Challenge`.

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
