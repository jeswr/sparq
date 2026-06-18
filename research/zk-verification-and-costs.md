# ZK verification and costs

What the sparq ZK circuit family actually verifies, and where the proving time and
gates go. This is a plain-language companion to the circuit relations in
`zk/compose/compose_core/src/*.nr`, the orchestration crate
`crates/sparq-zk-compose`, and the committed gate-count snapshot
`crates/sparq-zk-compose/tests/gate_count_snapshot.json`.

> **Standing caveats (load-bearing, not waivable).** The composition verifier is
> **NOT-yet-sound** (`sq-qhy4` / `sq-9hrn`; remediation epic `sq-1s2`). The whole
> estate is **research-grade, single-model self-audited, and NOT externally
> audited**. The separate MPC layer is **semi-honest**. A passing — or fast —
> proof is **not** a soundness or privacy guarantee under an adversarial prover.
> Every "proves" statement below is *as designed / as landed*, not an audited
> guarantee. Do not treat a "verified" result as production-trustworthy.

## What the ZK proofs verify

The family is a set of per-property Noir circuits over **committed** RDF graphs.
Each circuit binds its statement to a public commitment and proves one SPARQL-shaped
primitive. The orchestration crate's `ProofManifest` can carry several sub-proofs
(scan + filter + join + issuer + revocation + holder PoP) and `verify_manifest`
re-checks the structural binding edges between them, then runs `bb verify` on each
sub-proof.

### Live vs designed — what is proven *today*

Only **one** statement is proven live, in your browser tab, on the deployed demo:
the **age-gate filter** (`filter_int_d2`). Everything else is compiled,
gate-counted and exercised by native tests, but is **not** run in-browser, and the
six-member car-hire flagship has **never been assembled and verified as one unit**
(composition is exercised pairwise, not end-to-end).

| Member (family) | Primitive | Public inputs | Hidden (private) | Status |
| --- | --- | --- | --- | --- |
| `filter_int` (`d1..d4`) | range / comparison over a hidden integer | `challenge, operand_enc, op, bound, expected` | the value's decimal digits | **LIVE** (`d2`); `d1/d3/d4` wired (compiled, real prove/verify tests) |
| `filter_f64` (raw + `d1..d4`) | comparison over IEEE-754 doubles | `challenge, operand_enc, op, b_bits, expected` | the value's decimal digits | WIRED (composable `d1..d4`); raw `filter_f64` is a non-composable building block |
| `scan` (`k{1,2}_n{16,64}_r{4,8}`, 8 members) | set-membership **+ completeness** (BGP scan over K committed graphs) | `challenge, commitments[K], pattern, disclosed rows[R], row_count, attribution[K]` | per-graph counts + term encodings | WIRED (all 8 compiled, prove/verify/tamper tests) |
| `join_eq` (`na{16,64}_nb{16,64}`, 4 members) | equality-join (hidden cross-credential JOIN, single-prover) | `challenge, commit_a, commit_b, join_commitment, slot_a, slot_b` | both graphs' enc/counts, the two joined rows, the join value, a blinder | WIRED (4 compiled; the joined entity never appears in a public input) |
| `hidden_issuer` (`d4`) | signature-verification + set-membership | `challenge, m, key_set_root` | issuer `pk`, signature `(R,s)`, reduced challenge, membership index+path | WIRED/DESIGNED (gadget + binding complete; inherits the open soundness audit; **additive only**) |
| `holder_pok` | proof-of-possession / proof-of-knowledge | `challenge, holder_pk_digest` | `hsk`, `hpk_x`, `hpk_y` | WIRED/DESIGNED (compiled; in-circuit hidden-key tier B2 deferred — landed binding is the clear-key B1 tier) |
| `revoke_unset` (`d10`) | non-membership / hidden-index status | `challenge, root, index_commitment` | `index`, `bit(=0)`, blinding, Merkle siblings | WIRED/DESIGNED (compiled; clear-index path still leaks the index unless the committed-index path is used) |

All 24 members compile and have gate-count baselines; the family-completeness gate
(`every_derivable_id_has_a_compiled_member`) enforces no silent unprovability.

### What the live demo proves today (exactly one statement)

The deployed car-hire page proves **only** the age-gate `FILTER` via the
`filter_int_d2` circuit (relation `filter_int::filter_int_check`, `op = OP_GE (3)`,
`bound = 25`). It is a genuine in-browser UltraHonk proof
(`@aztec/bb.js` + `@noir-lang/noir_js`, WASM, `verifierTarget:'evm'` keccak flavour,
`disableZk:false` so it stays zero-knowledge), generated **and** self-verified in the
tab against the ACIR shipped at `site/public/zk/filter_int_d2.json`.

- **Public inputs** (confirmed against the ACIR ABI): `challenge` (Field),
  `operand_enc` (Field), `op = 3` (u32), `bound = 25` (u64), `expected` (bool).
- **Private witness:** `digits[2]` — the two ASCII decimal digits of the exact age.
- **Statement proven:** *"I know a 2-digit canonical `xsd:integer` whose canonical
  N-Triples token `"<digits>"^^xsd:integer` blake3-hashes (low-31-bytes BE) to a
  value `hs` with `h2(LITERAL, hs) == operand_enc`, AND `value >= 25 == expected`"* —
  without disclosing the age.
- An under-age claim (e.g. 24 trying to prove `>= 25` true) is **unsatisfiable**:
  `noir.execute` throws and no proof is produced. That unforgeability of the verdict
  is the soundness the demo shows live.

**Honest scope of the live demo:** it does **not** verify any signature, scan,
join, revocation or holder-key — those are not run in-browser. The `operand_enc`
values are **pre-computed native fixtures** (ages 24 / 25 / 30 / 42 hard-coded), so
the demo does not itself bind `operand_enc` to a committed graph via a live scan
proof; `challenge` is fixed to `0x1`. The proof is age-hiding, but the chosen circuit
member `D = 2` leaks `ceil(log10(value))` — i.e. that the age is 2 digits. The
car-hire component labels the other five rows "composed" (not "live") and treats
wiring them into the browser as a follow-up.

## Where the time and gates go

Gate counts below are the **deterministic** committed snapshot
(`bb gates -s ultra_honk` `circuit_size`; bb `5.0.0-nightly.20260324`; nargo
`1.0.0-beta.21`; 3% regression tolerance absorbs ~0.5–0.75% cross-platform variance).
Four members were independently re-run on this box and matched the snapshot to the
gate: `filter_int_d2 = 17416`, `revoke_unset_d10 = 899`, `scan_k1_n16_r4 = 5991`,
`hidden_issuer_d4 = 16932`.

> **All timings below are INDICATIVE / NON-CANONICAL** (native `bb prove` or Node
> `generateProof` on a shared EC2 work box), not a benchmark of record. Per project
> policy, EC2 measurements are not baked into docs or tests. Gate counts are the only
> deterministic figures.

### Per-member gate counts

| Member | Gates (`circuit_size`) | Note |
| --- | ---: | --- |
| `scan_k2_n64_r8` | 34821 | **Heaviest.** K=2 × N=64 × R=8; commitment-recompute sweep + completeness double-loop dominate |
| `scan_k2_n64_r4` | 27054 | 2nd heaviest |
| `scan_k1_n64_r8` | 18850 | |
| `join_eq_na64_nb64` | 18681 | Largest hidden cross-credential join (64×64) |
| `filter_int_d1..d4` | 17416 | Flat in D — blake3 over the canonical token fits one 64-byte block |
| `filter_f64_d1..d4` | 17416 | Same token-binding cost as `filter_int` (tie) |
| `hidden_issuer_d4` | 16932 | In-circuit signature verification — heaviest non-scan / non-filter operator |
| `scan_k1_n64_r4` | 14923 | |
| `join_eq_na16_nb64` / `na64_nb16` | 12885 | Equal by bucket-size symmetry |
| `scan_k2_n16_r8` | 11261 | |
| `holder_pok` | 10334 | One ~251-bit scalar-mul |
| `scan_k2_n16_r4` | 9254 | |
| `scan_k1_n16_r8` | 7038 | |
| `join_eq_na16_nb16` | 7025 | Smallest join |
| `scan_k1_n16_r4` | 5991 | Smallest scan |
| `filter_f64` (raw) | 3113 | Raw-bits building block — no string hashing; cheapest non-revoke member |
| `revoke_unset_d10` | 899 | **Cheapest.** Depth-10 Merkle bit-unset |

Scaling: scan members scale ~linearly in `k*n`, with `r` adding a row-soundness pass.
Filter is **flat in `D`** (the digit count only changes what leaks, not the gate
count). The signature member is dominated by elliptic-curve scalar multiplication,
not hashing.

### Hotspots

1. **Scan is the heavy end** (`scan_k2_n64_r8 = 34821`): the per-graph Poseidon2
   commitment recompute plus the completeness double-loop dominate.
2. **In-circuit signature verification is expensive** (`hidden_issuer_d4 = 16932`):
   two ~251-bit twisted-Edwards scalar muls implemented in explicit Field constraints
   (the `embedded_curve_*` blackboxes are Grumpkin — the wrong curve). This is the
   single heaviest non-scan / non-filter operator — *comparable to a full filter
   member*. The brief's expectation that signature verification is expensive is
   **borne out, with a nuance:** on the actual snapshot the largest scan bucket
   (34821) and the composable filter members (17416) edge it out, because only
   depth-4 / 16-issuer membership is compiled; a larger trusted-issuer set would push
   the signature member higher.
3. **Filter token-binding** (`17416`): the blake3 blackbox over the canonical
   `"<digits>"^^xsd:integer` token is the cost driver and fits one 64-byte block — so
   `D` does not move gates (it only leaks `ceil(log10(value))`).

### Proving time (indicative, Node / native, non-canonical)

Prove time tracks gate count: scan members slowest, filter members fastest. From the
full-family cost curve (`bench/zk-compose/family_cost_curve.json`, darwin/aarch64,
t8): `scan_k2_n64_r8 ~1.828 s`, `scan_k2_n16_r8 ~1.061 s`,
`filter_int_d{1,2,4} ~1.05–1.07 s`, `filter_f64 (raw) ~0.777 s` (fastest),
`scan_k1_n16_r4 ~0.902 s`.

**Proof size, vk size, and verify time are CONSTANT across the whole family**
(UltraHonk succinctness): **14656 B** proof (noir-recursive / poseidon2 flavour) or
**8384 B** (evm / keccak flavour, still zero-knowledge), **3680 B** vk, **~12 ms**
verify on aarch64. (An older 2-member timing file reported a ~0.95 s scan verify on
darwin/8-thread; that looks like a cold-vk artefact and is superseded by the ~12 ms
uniform verify in the cost curve.)

### Browser single-thread overhead (~4×, the COI / multithreading lever)

The in-browser prover (`site/src/lib/zk-prover.ts`, proving the shipped ACIR
`site/public/zk/filter_int_d2.json`) is **forced single-threaded** on the deployed
GitHub Pages demo — roughly a **~4× overhead** versus the multithreaded path.

- **Root cause** (source-confirmed, `zk-prover.ts` `maxThreads()` ~line 183): bb.js
  worker-thread fan-out is gated on `SharedArrayBuffer`, which requires cross-origin
  isolation (COOP `same-origin` + COEP `require-corp` / `credentialless`).
  `maxThreads()` returns `1` unless `window.crossOriginIsolated` is `true`, and GitHub
  Pages cannot set those response headers, so the deployed demo is permanently
  single-threaded.
- **Magnitude** (indicative, Node, non-canonical, this EC2 box, from
  `research/zk-browser-perf-assessment.md`, same `filter_int_d2` circuit + witness,
  warm `generateProof`):

  | Threads | Prove | Verify |
  | ---: | ---: | ---: |
  | 1 | ~1285–1295 ms | ~174 ms |
  | 2 | ~766 ms | |
  | 4 | ~474 ms | |
  | 8 | ~306–358 ms | ~63 ms |

  Measured speedup t1 → t8 ≈ **4.2×**. Proof bytes are **identical (14656 B)** at
  every thread count — threads change only speed, never the proof. A corroborating
  native `bb prove` run on this box (wall-clock, includes process startup + CRS/SRS
  load, so it overstates pure proving) gave t1 ~1314–1340 ms, consistent with the
  Node t1.

So a user on the deployed Pages demo pays roughly the full single-threaded cost
(~1.3 s-equivalent native, **slower in real-browser WASM** — not separately measured),
i.e. ~4× slower than the multithreaded path they would get with cross-origin
isolation (e.g. via a COI service worker). **This is a perf / config axis only — it
does not change what is proven or the zero-knowledge (age-hiding) property.**

## Sources

- Circuit relations: `zk/compose/compose_core/src/{scan,filter_int,filter_float,join,issuer,holder,revoke,hashes,lib}.nr`
- Member public-input shapes: `zk/compose/<member>/src/main.nr`
- Live circuit ACIR ABI: `site/public/zk/filter_int_d2.json`
- Browser prover: `site/src/lib/zk-prover.ts`; demo: `site/src/components/zk-car-hire.tsx`; fixture: `site/src/data/zk-car-hire.ts`
- Orchestration: `crates/sparq-zk-compose/{README.md,src/*.rs}`; tests: `crates/sparq-zk-compose/tests/*.rs`
- Gate snapshot (authoritative, `sq-pzet`/`sq-mj2z` baseline): `crates/sparq-zk-compose/tests/gate_count_snapshot.json` (mirror: `bench/zk-compose/gate_counts_latest.json`)
- Cost curve (indicative): `bench/zk-compose/family_cost_curve.json`; browser perf: `research/zk-browser-perf-assessment.md`
- Per-member machine-readable cost + semantics table: `site/src/data/zk-circuit-costs.json`
- Soundness status: `research/zk-soundness-audit.md` (`sq-qhy4`/`sq-9hrn`/`sq-1s2`); `zk/compose/STATUS.md` (its hard-coded gate-count block has been replaced by a pointer to the authoritative catalog above, so the gate counts here are the single source of truth)
