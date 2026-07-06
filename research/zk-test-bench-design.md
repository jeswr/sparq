<!-- [OPUS-4.8] authored by Opus 4.8 (Fable 5 unavailable) — re-review when Fable returns -->
# ZK test-suite & benchmark design — first-pass hand-off

<!-- [OPUS-4.8] sq-5reoy (#1599): the in-tree `zk/ieee754` and `zk/xpath` Noir trees were externalized to the `sparq-org/noir_IEEE754` (v0.10.0) and `sparq-org/noir_XPath` (v0.2.0) face repos and REMOVED from this repo; their test suites, oracle harnesses, and benches (`zk/ieee754/bench`, `zk/ieee754/scripts/*`, `zk/xpath/test_packages`, `zk/xpath/VENDOR.md`, `numeric_types.nr`) now live in the face repos. `zk/compose` remains in-tree and consumes the released `sparq_ieee754` as a pinned Nargo git dependency. Any `zk/xpath/…` / `zk/ieee754/…` path below is a HISTORICAL in-tree reference — the live source is the corresponding face repo. -->

A structured summary of how the test suites and benchmarks across the sparq ZK
estate are **designed** and what they **derive from**, plus an honest,
audit-driven gap analysis and a recommended go-forward plan.

This is a **first pass for Jesse to comment on**. It describes what *exists*
accurately; recommendations are marked as such. The cautionary example
throughout is `research/zk-soundness-audit.md` — the adversarial audit that
found v1 verifier soundness is broken precisely because the existing tests gave
false assurance. Read the gap analysis (§4) as the load-bearing section.

Scope studied (read-only): `crates/sparq-zk`, `crates/sparq-zk-compose`,
`zk/compose`, `zk/ieee754`, `zk/xpath`, `bench/zk*`, and the differential fuzzer
in `crates/sparq-bench`.

---

## 1. The ZK estate — layers and what each proves

| Layer | Path | What it proves / provides | Test anchor |
|---|---|---|---|
| **Stage 1: commit** | `crates/sparq-zk` | RDFC10-canonicalize a named graph → encode leaves → Poseidon2 fold `C(G)`. Bit-compatible with the Noir circuit's in-circuit commitment recompute. | W3C rdf-canon manifest + nargo Poseidon2 cross-vectors |
| **Stage 1: trace** | `crates/sparq-zk` (`trace.rs`, `verify.rs`) | Q6 blank-node anti-correlation: prover-side cross-graph bnode-join rejection + verifier-side static re-check + named-graph attribution. | `trace_guard.rs`, `trace_named_graph.rs` |
| **Circuit family** | `zk/compose` (Noir) | `scan_check` (commitment recompute + row soundness + completeness), `filter_int` (hidden-operand integer FILTER over the canonical literal token), `filter_f64` (IEEE comparison building block). | `compose_core/src/tests.nr` (22 in-circuit accept+adversarial) |
| **Stage 2: compose** | `crates/sparq-zk-compose` | Manifest serde, prover driver (nargo+bb subprocess), verifier (structural gate + bb verify), binding-consistency edges. | `tests/e2e.rs` |
| **IEEE-754 kernels** | `zk/ieee754` (Noir) | Generated `f16/f32/f64/f128` arithmetic, comparison, rounding, sqrt, int casts. The numeric substrate for `filter_f64` and XPath floats. | `tests/generated_arithmetic` + oracle harness |
| **XPath F&O** | `zk/xpath` (Noir, vendored) | SPARQL-builtin function layer (numeric/boolean/string/date arithmetic & comparison) over XPath 2.0 F&O. | qt3-derived `test_packages` + `xpath_unit_tests` |

The composition story (stage 2) is **modular per-property proofs**: each BGP
pattern and each numeric FILTER is its own circuit proof, and "composition" is
the verifier checking each sub-proof plus the **binding edges** between them.
That seam is exactly where the soundness audit found the system is unbound — see
§4.

---

## 2. Per-layer **test** design — what each suite tests and what it derives from

Correctness in this estate is anchored four ways; every suite below uses one or
more:

- **Oracle** — an independent reference model computes the expected answer
  (W3C rdf-canon, exact-rational/MPFR/FPgen float model, elementpath for qt3).
- **Byte-identity / cross-vector** — the Rust and Noir implementations must
  produce the same field elements bit-for-bit (Poseidon2, operand encoding).
- **Differential** — re-run over a reduced input and require the same result
  (traced-triples sufficiency; sparq vs Oxigraph in the engine).
- **Adversarial / negative** — a constructed lie must be *rejected* (in-circuit
  `should_fail_with`, structural tamper tests, RDFC10 negative-eval).

### 2.1 `crates/sparq-zk` — RDFC10 suite (`tests/rdf_canon_suite.rs`)

- **Derives from:** the W3C **rdf-canon** test suite, vendored at
  `tests/rdf-canon-testdata/` (snapshot commit `15619df…`, 2026-02-24; see
  `PROVENANCE.md`). Files are byte-identical to upstream; do not edit.
- **Manifest-driven, 86 entries**, asserted as `(eval, map, neg) = (64, 21, 1)`
  so suite-composition drift fails loudly. The manifest is parsed *with sparq
  itself* (dogfooding the engine).
- **Three test classes** (the W3C taxonomy):
  - `RDFC10EvalTest` (64): canonical N-Quads must match **byte-for-byte**.
  - `RDFC10MapTest` (21): the issued blank-node identifier map must match.
  - `RDFC10NegativeEvalTest` (1): a poison graph (HNDQ call-limit) must
    **fail closed** — canonicalization returning `Ok` is a failure.
- **Bridge design (important):** the suite validates the `rdf-canon`
  dependency *through* sparq-zk's own oxrdf-0.3 → text → oxrdf-0.2 → canonical
  bridge, so a bridge bug fails exactly like a canonicalization bug. SHA-384
  parameterization (test075) is honoured. A second test
  (`default_graph_tests_via_crate_api`, asserts `covered >= 40`) re-runs the
  default-graph-only eval tests through `canon::canonicalize_triples` — the
  *actual* single-graph entry point the commitment pipeline calls.

### 2.2 `crates/sparq-zk` — Poseidon2 nargo cross-test (`tests/poseidon2_noir_cross.rs`)

- **The bit-compatibility gate.** Vectors were produced by `nargo execute`
  (nargo 1.0.0-beta.21) on the fixture package `tests/fixtures/noir_poseidon2/`
  (noir-lang/poseidon v0.3.0, `std::hash::poseidon2_permutation`, bn254
  blackbox). They are **hardcoded** so the gate is deterministic.
- **Two-mode design:** when `nargo` is on PATH, `nargo_live_cross_check`
  re-executes the fixture and asserts the live output still equals both the
  vendored vectors *and* the Rust `poseidon2::hash` — so a toolchain bump that
  changed hashing semantics fails loudly instead of silently invalidating every
  recorded commitment. When `nargo` is absent, the static vectors still run
  (minimal CI stays green). This is the canonical "skip cleanly if toolchain
  absent" pattern reused everywhere in the estate.
- **Coverage:** the 4-element permutation, sponge hashes over `[1]…[1,2,3,4]`
  and `[5..11]`, a 40-leaf credential-scale fold, plus a padding-collision guard
  (`hash([a,b]) != hash([a,b,0])` — length-bearing IV).
- `commit.rs` adds the Rust-side commitment unit tests:
  input-order-independence, **salt-separation** (identical canonical content in
  two graphs yields different *leaves*, not just commitments — the Q6 privacy
  property), and leaf-index resolution. These cross-vectors are mirrored exactly
  in `compose_core/src/tests.nr` (`0x168758…`, `0x046964…`), closing the
  Rust↔Noir loop.

### 2.3 `crates/sparq-zk` — trace guard + named-graph (`trace_guard.rs`, `trace_named_graph.rs`)

- **`trace_guard.rs` (Q6 bnode anti-correlation).** Two fixtures: `CLEAN`
  (cross-graph joins on IRIs only — must succeed) and `COLLIDING` (same bnode
  label `_:addr` in two graphs — distinct nodes under RDF merge, must fail
  closed). Tests cover: IRI cross-graph join traced+resolved; **traced-input
  sufficiency** (a *differential* test — re-running the query over only the
  traced triples reproduces the rows); same-graph bnode join allowed;
  cross-graph bnode join rejected by the prover (`CrossGraphBnodeJoin`); the
  subtler label-identified-union leak caught; the **prover↔verifier handshake**
  (verifier independently *requires* an obligation and rejects a manifest
  declaring none, `MissingObligation`); and GRAPH-keyword rejection.
- **`trace_named_graph.rs` (attribution).** Complements the guard: focus on
  *correct attribution*, not rejection. Each pattern's consumed triples resolve
  to the correct named graph (by predicate); single-graph queries attribute to
  one graph; every resolved leaf index is in range of its graph's commitment.

### 2.4 Circuit family — in-circuit tests (`zk/compose/compose_core/src/tests.nr`)

- **Design convention (noir-testing):** one accept path plus one
  `should_fail_with` per *distinct rejection clause*, message-matched so each
  test pins *which* clause caught the lie ("adversarial-prover testing"). 22
  tests total.
- **`scan_check` (K=2,N=4,R=2 monomorphisation):** accepts a complete match set
  and a constant-only (ASK-shaped) pattern; rejects tampered commitment,
  tampered graph content (the "license: suspended" suppression case),
  suppressed row, fabricated row, off-pattern row, padding-as-disclosure,
  oversized count, oversized row_count. Padding-slot invariance and the
  count-domain-separation property are asserted directly.
- **`filter_int`:** all six ops over all verdicts + boundary cases; rejects a
  lying verdict, a swapped operand (different committed value), non-digit bytes,
  non-canonical leading zero, unknown operator. The operand-binding replica
  (`enc_int_literal`) is explicitly documented as **non-circular** — its
  anchor is the Rust e2e, which computes `operand_enc` via `sparq_zk::encode`.
- **`filter_f64`:** ordered comparisons + IEEE edge cases (NaN unordered,
  `ne(NaN)=true`, `-0 == +0`); rejects a lying verdict.

### 2.5 Stage 2 — `crates/sparq-zk-compose/tests/e2e.rs`

- **Fast tests (default, no toolchain):** manifest serde round-trip; structural
  acceptance; and four **structural tamper/negative** cases —
  `BindingInconsistent` (operand points at a different encoding than the
  scanned column), arity mismatch, `CircuitIdMismatch` (declared `k` the
  commitments don't support), and the sparq-zk Q6 cross-graph-bnode-join guard
  firing.
- **Toolchain-gated (skip cleanly if `nargo`/`bb` absent):** witness-gen
  satisfiability for `filter_int` and `scan`, **plus a false-verdict rejection**
  (17≥18 claimed true → witness generation must fail; the driver detects
  unsatisfiability by witness-file absence, since `nargo execute` exits 0 even
  on a failed assertion).
- **One NON-ignored full prove→verify:** `full_prove_verify_filter_int_d1`
  runs a real bb prove on the smallest member, verifies, then flips a proof byte
  and asserts bb rejects.
- **`#[ignore]` slow pattern:** `full_manifest_prove_verify_scan` does the full
  scan-member prove→verify through `verify_manifest`; run with
  `cargo test -- --ignored`. Concurrency note: subprocess proving shares one
  `Prover.toml` per package, so parallel tests must target *distinct* members.

### 2.6 `zk/ieee754` — generated oracle vectors

- **Oracle design (`scripts/generate_float_vectors.py`):** a self-contained
  reference model using **exact rational arithmetic** (`fractions.Fraction`) and
  round-to-nearest-even packing, so it produces vectors for f16/f32/f64/f128
  **without NumPy or MPFR**. (The brief's "MPFR/numpy oracle" framing is
  aspirational; the *committed* oracle is exact-rational — this matters because
  exact-rational is its own ground truth and needs no external lib to be
  trustworthy.) Covers arithmetic (`+ - * /`), comparisons (`eq…ge`),
  round-to-integral (`floor/ceil/trunc/round_ties_even`), `sqrt`, and XPath
  float→int casts (`to_u64`/`to_i64`, with `should_fail` for out-of-range).
- **Corpus sources, tagged per vector** (`curated:` / `random:<seed>:` /
  `fpgen:<file>:<line>`): curated interesting-value pairs (zeros, subnormals,
  infinities, NaN, ulp boundaries), seeded-random finite pairs (default 8/op,
  deterministic via `random.Random(seed+bits)`), and **IBM FPgen** ingestion —
  a bounded subset of the public `.fptest` arithmetic cases from
  `sergev/ieee754-test-suite`, downloaded+cached on demand (`--include-fpgen`).
- **Honest sizing note:** `tests/generated_arithmetic/src/lib.nr` has **121
  `#[test]` functions** (the brief's "121 vectors") but each batches many
  assertions — the source-comment tally is ~1,320 curated + ~526 random
  individual vectors. Describe it as "121 generated test functions covering
  ~1.8k vectors," not "121 vectors."
- **Public-API discipline:** `test_public_api.sh` checks external packages can
  import the generated `f*` types and *cannot* import helper internals;
  `lint_private_function_usage.py` enforces the private-helper boundary.

### 2.7 `zk/xpath` — qt3-derived taxonomy (`VENDOR.md`)

- **Derives from:** the W3C **qt3tests** suite, auto-converted to Noir test
  packages by upstream's `generate_tests.py` (using **elementpath** as the
  evaluation oracle). Vendored byte-identical to `jeswr/noir_XPath@fe88a5d`
  except a toolchain dep-swap (`Nargo.toml`) for beta.21.
- **The three-class taxonomy is itself a coverage map** (the suite was never
  fully green upstream — that is by design):
  - **Real converted qt3** (71 pkgs): execute library code — 63 green / 8 red
    (all 8 pre-existing upstream: type-generator gaps + timezone-semantics
    gaps, not toolchain drift).
  - **Stub-by-design** (152 pkgs): call `stub_*()` → `assert(false, "… not
    available in ZK")` — red on purpose, documenting unimplemented features
    (regex, XML/document model, env/context, higher-order, collation,
    format-*).
  - **Placeholder** (18 pkgs): "no converted tests" — 7 red, 11 vacuously
    green (upstream generator inconsistency).
- **Float/double oracle packages** (21, non-member upstream): run green on
  beta.21 (283 tests, incl. `fnround_double` 87/87, `fnround_float` 88/88).
  These are the oracle that validated the `sparq_ieee754` float-API migration
  (numeric_types.nr now sits on `zk/ieee754` rather than the old vendored lib).
- **Honesty marker:** `VENDOR.md` carefully labels every function as `qt3 n` /
  `unit` / `untested` / `STUB`. A large set of string functions and aggregates
  are **`impl` but `untested`** — this is documented, not hidden.

### 2.8 The differential-fuzz pattern (`crates/sparq-bench/src/fuzz.rs`)

- **Not ZK** — it is the engine's correctness fuzzer, included here because it
  is the estate's strongest *differential* design and the template for what ZK
  is missing.
- **Design:** deterministic SplitMix64 RNG (seed → fully reproducible repro),
  random small graphs with a **deliberately mixed-datatype** numeric column,
  random queries across every plan shape (bgp/filter/optional/union/minus/
  limit/distinct/order/equality), checked against **Oxigraph** (an independent
  mature SPARQL engine): `sparq query().len() == oxigraph solution count`, plus
  order-sensitive sequence checks, a `query_json` multiset differential, and a
  `count()` self-differential. A mismatch prints `seed + query + graph`.
- **Why it matters for ZK:** "agreement over thousands of random cases against
  an independent implementation" is exactly the assurance the ZK verifier
  *lacks* — see §5.

---

## 3. Per-layer **benchmark** design — gate counts, wall-clock, throughput, regression gating

Three kinds of ZK benchmark exist, with different ground truths and gating.

### 3.1 Gate counts (the primary ZK cost metric)

- **Ground truth:** `bb gates -s ultra_honk` `circuit_size` (per the
  noir-optimisation skill). `nargo info` alone is **not** accepted as evidence —
  stated explicitly in `zk/ieee754/AGENTS.md` ("Use `bb gates` as the source of
  truth").
- **Amplification harness (`zk/ieee754/scripts/benchmark_float_ops.py`):**
  builds temporary binary packages, runs `nargo compile`, reads
  `functions[0].circuit_size`, measures a **small-N and big-N** circuit and
  estimates **per-call cost from the difference** so fixed padding/setup does
  not dominate. Each op is measured as repeated `acc = acc op b`, matching the
  reference library's amortised method. Kernels (comparisons, rounding, sqrt,
  casts) use an XOR-fold pattern that keeps every call live (so their per-call
  estimate includes one `new()` decode).
- **Regression gating (`compare_float_benchmarks.py --max-regression`):**
  compares candidate `per_call_estimate` against the committed baseline
  (`bench/float_ops_latest.json`) op-by-op and **exits 1 if any delta exceeds
  the threshold** (default 1 gate/call). Baselines are committed JSON
  (`float_ops_baseline-*.json`, `*_latest.json`), and `AGENTS.md` codifies the
  promote-baseline workflow (`cp candidate → latest`, then update
  `bench/README.md`).
- **Compose gate counts (`bench/zk-compose/`):** same `circuit_size` convention,
  emitted to `gate_counts_latest.json` by `scripts/gate_counts.sh`. Current
  family: `scan_k1_n16_r4=5,958`, `scan_k2_n16_r8=11,011`,
  `scan_k2_n64_r8=34,379`, `filter_int_d{1,2,4}=17,416` (blake3-block-bound,
  d-invariant — the `d` param leaks `ceil(log10(value))`, not gates),
  `filter_f64=3,113`. Notably the compose bench is **README-tabulated, not
  `--max-regression`-gated** — unlike ieee754, there is no automated
  regression guard on the compose circuit sizes yet.

### 3.2 Prove / verify wall-clock

- `bench/zk-compose/scripts/prove_verify.sh <member>` → `prove_verify_timing.json`.
  Measures bb prove (incl. `--write_vk`) + verify + proof size. Current
  small-e2e numbers (darwin arm64): `filter_int_d1` prove 1.13 s / verify
  0.16 s; `scan_k1_n16_r4` prove 1.62 s / verify 0.95 s; proof 14,656 B
  (constant for ultra_honk). Verify time grows with public-input count (scan
  carries commitment+rows vectors). **Coverage is only 2 of the 7 members** —
  no sweep across the family, no prove-time-vs-(k,n,r) curve.

### 3.3 Canonicalization / commitment throughput (`bench/zk`)

- Criterion benches for the stage-1 pipeline: RDFC10 canonicalization, leaf
  encode + Poseidon2 fold, end-to-end commitment, raw Poseidon2 primitives.
  Standalone cargo project (own `[workspace]`, never touches the wasm build).
- Graph shapes `iri` (ground triples, cheap RDFC10 path) and `bnode` (canonical
  labeling). The README reads the baseline **honestly**: canonicalization is
  *not* the bottleneck (~100k triples/s); Poseidon2 is (~5–8k triples/s
  end-to-end), explicitly flagged as a correctness-first port with known
  headroom that should *not* be optimised before a stage needs it.

### 3.4 Trace overhead (`bench/zk-trace`)

- Criterion benches of the zk-trace seam: per-operator proof-input capture cost,
  **traced** (recorder armed) vs **untraced**, across every plan shape. Honest
  read: capture is a constant-factor ~2.5–4x cost when armed (it materializes
  the consumed input set and disables result-changing pushdowns). The
  `untraced` arm pins the disarmed floor.

---

## 4. HONEST gap analysis (audit-driven)

This section is the point of the document. `research/zk-soundness-audit.md`
found **v1 verifier soundness is broken**: `verify_manifest` returns `Ok(())`
over arbitrary forged results. The reason the existing tests did **not** catch
this is a *test-design* failure, and it is the central lesson here.

### 4.1 Why the current tamper-test design was insufficient

The verifier is three disjoint checks that never meet:

- **Stages 1–2** re-parse the query, re-derive circuit ids, and check
  binding-edge equalities — all over the **prover-declared `ProofInputs` JSON**.
- **Stage 3** hands the **prover-supplied `(proof, public_inputs, vk)`** triple
  (decoded verbatim from `proof_hex`) to `bb verify`.

Nothing checks that the JSON statement and the cryptographic proof describe the
**same** statement, that the vk is the **canonical circuit's** vk, that the
commitments are **signed by an issuer**, or that the challenge is **fresh**.

The tamper tests in `e2e.rs` (§2.5) exercise **only the declared-JSON layer**:

- `structure_rejects_inconsistent_binding_edge` mutates a *JSON field* and
  asserts the *JSON equality* check fires.
- `structure_rejects_circuit_id_mismatch` mutates a *declared id* and asserts
  the *re-derivation over declared inputs* fires.
- `full_prove_verify_filter_int_d1`'s byte-flip tests that **bb itself** rejects
  a corrupted proof — a property of bb, not of `verify_manifest`'s binding.

**None of these constructs a genuine bb proof over statement A and then asks the
verifier to accept it under a manifest claiming statement B.** Because no test
ever decoupled "the real proof" from "the declared claim," the catastrophic gap
(the public-input vector is never reconstructed from the declared statement and
byte-compared; the vk is taken from the prover) was invisible. The in-circuit
`should_fail_with` tests (§2.4) *are* sound — `scan_check`/`filter_int` reject
lies **in-circuit** — but they prove the *relation* is correctly constrained,
not that the *verifier binds a real proof of that relation to the declared
claim*. The estate had strong **relation** tests and zero **binding** tests, and
mistook the former for the latter. The README/STATUS lines "tamper tests …
all fail" and "verifier with the sparq-zk re-check" read as end-to-end
soundness assurance they do not provide — the audit's hardening item
"remove/correct misleading comments" applies to the test narrative too.

The class of test that **would** have caught it:

> **forge-and-verify**: construct a *genuine* bb proof over a true statement A
> the prover legitimately holds; swap the manifest's declared `ProofInputs` to a
> false statement B (or swap the query, the operand, the attribution) while
> leaving `proof_hex` pointing at the real proof of A; assert `verify_manifest`
> returns **Err**. Every confirmed audit issue is an instance the current suite
> *passes* but a forge-and-verify test would *fail*.

### 4.2 The binding gaps (the audit's confirmed issues, restated as missing tests)

| # | Confirmed issue (audit) | Missing test class |
|---|---|---|
| 1 | Public inputs never reconstructed from declared statement | forge-and-verify: honest proof of A under manifest B → must reject |
| 2 | `bb verify` uses prover-supplied vk | vk-substitution: attacker-compiled trivial-circuit vk → must reject |
| 3 | No issuer signature / key-set membership | unsigned/forged-commitment & subset-suppression → must reject |
| 4 | No replay/freshness binding | cross-session replay + challenge-rebind (JSON-only) → must reject |
| 5 | FILTER op/bound/verdict unbound to query | comparison-substitution (17≥17 presented as ≥18) → must reject |
| 6 | Binding edge ignores which slot / verdict | wrong-slot operand (salary read as age), unpruned false verdict → must reject |
| 7 | Composition seam is JSON-only | operand-substitution / kind-confusion at scan→filter seam → must reject |
| 8 | Cross-graph bnode attributions are proof-unbound | attribution-lie (`[[0],[0]]` collapsing two graphs) → must reject |

(Issues 9–12: salt-separation unenforced, query-text unbound, n/d/r-relabel
slack, revocation absent — covered in the go-forward plan, §5.)

### 4.3 Other coverage gaps (not in the audit's critical set, but real)

- **No f128 oracle hardening end-to-end:** ieee754 generates f128 vectors and
  `filter_f64` is gate-counted, but `filter_f64` is **not manifest-composable**
  (binding an `f64` to a committed literal needs in-circuit float→canonical-
  decimal printing, unbudgeted). So there is **zero** float-FILTER e2e
  coverage in compose — the float comparison is tested in isolation only.
- **No multi-pattern join proof:** join consistency across patterns is
  verifier-side over disclosed rows; multi-pattern joins are not proved in a
  single circuit and have no e2e test.
- **Inference/entailment regime untested beyond `Simple`:** only
  `EntailmentRegime::Simple` is proved; no test exists for any other regime.
- **No differential ZK testing:** the engine has a powerful sparq-vs-Oxigraph
  fuzzer (§2.8); the ZK layer has **nothing analogous** — no "the proven result
  set equals the cleartext engine's result set over random graphs+queries"
  differential. This would catch completeness/soundness divergences the fixed
  in-circuit vectors miss.
- **Compose gate counts are not regression-gated:** unlike ieee754's
  `--max-regression`, the compose circuit sizes live only in a README table.
- **Prove/verify benched on 2 of 7 members:** no family sweep, no
  prove-time-vs-(k,n,r) curve, no scan-vs-filter-vs-compose end-to-end timing.

---

## 5. Recommended go-forward test/bench plan (recommendations)

These are **recommendations** for Jesse to weigh, ordered so the test classes
map onto the audit's six critical issues. They presuppose the remediation
actually *implements* the missing binding gates — a test cannot assert a gate
that does not exist.

### 5.1 Binding-soundness test classes (must ship with the remediation)

Each is a **forge-and-verify** test: build a real proof, then lie in the
manifest, and assert **Err**. Mapping to the audit's criticals:

1. **Public-input reconstruction (audit #1, #7, #10, #11).** Generate an honest
   bb proof over statement A; mutate each declared field in turn (commitments,
   `pattern_*`, rows, row_count, operand_enc, op, bound, expected, and the query
   text/projection/constants) while keeping `proof_hex`; assert each mutation is
   rejected by byte-mismatch against the reconstructed public-input vector. One
   parameterized test per field is the cleanest shape (mirrors the per-clause
   `should_fail_with` convention).
2. **vk authenticity (audit #2).** Compile a trivial zero-constraint circuit
   with the right public-input arity, `bb prove --write_vk`, bundle the attacker
   vk; assert the verifier rejects because it recomputes/loads the canonical vk
   keyed by the re-derived `CircuitId` rather than trusting `art.vk`. Add a
   companion positive test that the canonical vk *does* verify an honest proof.
3. **Issuer signature / key-set membership (audit #3).** Assert an unsigned
   (or wrong-key-signed) commitment is rejected; assert the **subset-suppression**
   attack (drop the revocation/suspension triple, recompute `C` over truncated
   leaves, re-prove) is rejected because the commitment is not signed by a key
   in the disclosed set `K`.
4. **Replay / freshness (audit #4).** Verifier issues a fresh nonce; assert a
   captured manifest replayed under a *new* nonce is rejected; assert the
   challenge-rebind (substitute the JSON `binding.challenge` string only) is
   rejected because the challenge is byte-bound inside the reconstructed public
   inputs; assert single-use (seen-nonce store) rejects a second presentation.
5. **FILTER binding (audit #5, #6).** Parse the query FILTER into
   `(variable, op, constant)`; assert a `filter_int` sub-proof whose declared
   `(op, bound)` differ from the query's is rejected; assert the wrong-slot
   operand (salary value satisfying an age filter) is rejected because the
   binding edge must reference the FILTER-variable's actual scanned slot;
   assert `expected==false` rows are required to be excluded.
6. **Cross-graph attribution binding (audit #8).** Once per-row source-graph
   attribution is bound in-circuit and surfaced as a public input, assert the
   `attributions=[[0],[0]]` collapse-two-graphs lie is rejected by reconstructing
   and comparing the in-circuit attribution against `manifest.attributions`
   (today the guard trusts the JSON).

Plus the audit's explicit hardening tests: **malformed `proof_hex` → REJECT
(not panic)** via a `CheckError::MalformedProof` variant; **revocation: a
revoked/suspended credential → REJECT** once S2.6 exists; **HolderPoP → explicit
unimplemented error** until proof-of-possession exists.

### 5.2 Differential ZK testing (new, recommended)

Port the §2.8 fuzzer pattern to ZK: deterministic-seeded random
credential-graphs + random ZK-supportable queries; **prove**, then **verify**,
then assert the *proven, disclosed result set equals the cleartext sparq
engine's result set* (and, where the query is Oxigraph-supportable, equals
Oxigraph). This is the single highest-leverage addition — it catches
completeness/soundness divergences across the whole pipeline that fixed vectors
cannot, and it produces a seed-based repro. Gate it toolchain-on-PATH like the
existing bb tests; shard by category across agents as the engine fuzzer already
does.

### 5.3 Benchmark additions (recommended)

- **End-to-end prove/verify across the full circuit family** (all 7 members,
  not 2), emitted to JSON like the gate counts.
- **Prove-time-vs-(k,n,r)** curve for scan and **vs-d** for filter_int, so the
  cost model is empirical, not extrapolated from the gate-count linearity note.
- **Regression-gate the compose gate counts** with a `--max-regression`-style
  guard against `gate_counts_latest.json`, matching ieee754's discipline (today
  compose is README-only).
- **f128 / float-FILTER e2e** once `filter_f64` becomes manifest-composable
  (blocked on in-circuit float→canonical-decimal printing) — currently the float
  comparison is bench/test-covered in isolation but has no e2e path.

---

## 6. Open questions for Jesse

1. **Oracle provenance for floats.** The brief described an "MPFR/numpy oracle,"
   but the committed oracle is **exact-rational** (no external lib). Is the
   exact-rational model the intended long-term ground truth, or do you want an
   MPFR/numpy cross-check layered on top (and FPgen ingestion turned on in CI,
   which it currently is not by default)?
2. **Binding-test placement.** Should the forge-and-verify suite live in
   `crates/sparq-zk-compose/tests/` (Rust, drives bb) — the natural home — or do
   you want a dedicated `tests/soundness/` tree so the binding tests are
   unmistakably separated from the relation tests (the conflation of which
   caused the false assurance)?
3. **Differential-ZK scope.** Is a full prove→verify→compare-to-cleartext
   differential affordable in CI given bb prove times (~1–2 s/proof), or should
   it be a sharded, nightly, toolchain-gated job rather than per-PR?
4. **Compose regression gating.** Adopt ieee754's `--max-regression` JSON gate
   for compose circuit sizes now, or keep compose README-tabulated until the
   family stabilises?
5. **What does v1 claim?** Given the audit, should the test suite assert a
   **documented, narrowed claim** for v1 (e.g. "proves the in-circuit relations,
   makes no third-party soundness claim") so the green suite cannot again be
   read as end-to-end soundness — i.e. encode the honesty in an asserted test,
   not just prose?
6. **f128 in scope?** ieee754 fully generates and tests f128, but nothing
   downstream composes it. Is f128 a real target, or test-only headroom we
   should stop benchmarking against compose budgets?

---

*Model: Opus 4.8 (Fable 5 unavailable). First-pass design hand-off for comment;
recommendations marked as recommendations. The soundness audit
(`research/zk-soundness-audit.md`) is the governing cautionary input — do not
read any existing green suite as third-party soundness assurance until the
binding-soundness classes in §5.1 exist and pass.*
