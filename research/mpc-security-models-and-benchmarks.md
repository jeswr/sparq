<!-- [OPUS-4.8] MPC security-models taxonomy + protocol/operator/leakage/benchmark synthesis, Opus 4.8 (Fable unavailable) — design-for-review; re-review when Fable returns. -->

# MPC Security Models, Protocols, Operators, Leakage & Benchmarks for sparq-mpc

**Status:** Deep-research design record (no implementation; doc-only). Author: Opus 4.8
(Fable unavailable — flag for re-review). Date: 2026-06-15.
**Scope:** the *deep-understanding* artifact for what is achievable with MPC over
(federated) SPARQL, grounded in BOTH the literature AND the live `crates/sparq-mpc`
crate. Companion blueprint: [`mpc-zkp-research-and-architecture.md`](./mpc-zkp-research-and-architecture.md)
(the protocol/threat-model architecture); [`mpc-m4-distributed-sig-feasibility.md`](./mpc-m4-distributed-sig-feasibility.md)
(the attestation/collaborative-proof verdict). Build plan: `crates/sparq-mpc/PLAN.md`.

This record covers seven things, in order: (1) the security-model taxonomy + a
recommended **configurable** API; (2) the protocol-family → (security-model, N)
performance map; (3) the per-SPARQL-operator best-secure-realization matrix; (4) the
privacy/leakage analysis + the privacy-vs-performance frontier; (5) the benchmark
matrix (security × N × query class × data scale) + metrics + methodology; (6) the
SOTA-systems comparison + the realistic ACHIEVABLE ENVELOPE; (7) the other-properties
analysis (fairness/GOD/identifiable-abort/composability/preprocessing/randomness/PQ).

---

## 0. Where the crate is today (ground truth, verified against source)

The crate is a **Milestone-3 honest-majority Shamir backend** and an **in-process
multi-party simulation** — there is NO real network, NO broadcast, NO round counter,
NO preprocessing phase, and NO communication-cost model. Every "party" is a function
call in one process. This is faithful and honestly documented (no fake crypto), but it
means the dominant real-world MPC cost — rounds × RTT, bytes on the wire — is *not
observable in a single line of running code today*. Concretely:

- **`field.rs`** — `F_p`, `p = 2^61 − 1` (Mersenne; `u128` products folded by the
  `2^61 ≡ 1` identity).
- **`shamir.rs`** — Shamir `t`-of-`n`, `t = ⌊(n−1)/2⌋` (honest majority). Free local
  linear ops (`add_shares`, `scale`); `run_secure` cumulative SUM is **zero-round**
  (the flatmate aggregate). `mul_shares_raw` produces a **degree-`2t` product with NO
  degree reduction** — only a *single* multiplication is supported (the equality test);
  any multiplication *chain* (general arithmetic circuits) is unbuilt. Masking via a
  ChaCha20 CSPRNG (`SecureRng::from_os`, `rng.rs`); a deterministic PRNG is `cfg`-gated
  behind `insecure-test-rng` (off by default).
- **`robust.rs`** — Reed-Solomon / Berlekamp-Welch reconstruction: **detect** any
  tampering at `n > degree+1`, **correct** up to `e = ⌊(n−t−1)/2⌋` cheaters. Sound
  detection; `MpcError::Tampered{cheaters}` attribution is **best-effort/heuristic**
  on the abort path (named against an arbitrary first-`t+1` reference subset — *can
  blame an honest party*).
- **`join.rs`** — `DisclosedKeyJoin` (crypto-free plaintext equi-join over disclosed
  global IRIs; differentially tested == union-store PAG eval) and `HiddenValueJoin`
  (all-pairs `O(|L|·|R|)` secret-shared equality `secure_equal`: `d=a−b`, mask `m=d·r`,
  open `m`; `m==0 ⇔ keys equal; leaks only the match bit per pair`).
- **`backend.rs`** — the `MpcBackend` trait + two enums: `TrustModel{HonestMajority,
  DishonestMajority}` and `MaliciousSecurity{SemiHonestOnly, HonestMajorityAbort,
  HonestMajorityRobust{max_cheaters}}`, surfaced via `BackendInfo`/`info()`.
- **`proof.rs`** — `CollaborativeProof`/`Attestation`/`ProofStatement` are honest
  `MpcError::NotYetImplemented` stubs naming the Q1 gate. No crypto.
- **`holder.rs`** — real per-holder local SPARQL sub-evaluation via `sparq-engine`.

**The single most load-bearing code fact for this whole record:** at the honest-majority
default `n = 2t+1` (odd `n`: 3,5,7,9) the **degree-`2t` equality open has ZERO RS
redundancy** (`shamir.rs reconstruct_degree` doc, pinned by a boundary test), so a
forged product share silently flips a match verdict and is information-theoretically
undetectable. The RS robustness advertised by `HonestMajorityRobust` is TRUE for the
degree-`t` linear aggregate and **FALSE for the hidden-value-join equality path at
minimal N**. The fix is an information-theoretic MAC, not RS redundancy (the deferred
WI-4 / `sq-6d6g` seam). A federation reading a single backend-level
`malicious_security` bit could over-trust the join.

---

## 1. Security-model taxonomy + the recommended CONFIGURABLE API

### 1.1 The six models, placed precisely (adversary / threshold / guarantee / min N)

| # | Model | Adversary | Corruption threshold | Output guarantee | Min N | In `sparq-mpc`? |
|---|---|---|---|---|---|---|
| 1 | **Semi-honest / passive** | follows protocol, infers from view | any `t<n` (privacy is a setup param) | correct (only because all follow) | 2 | `SemiHonestOnly`; the v1 model |
| 2 | **Covert (ε-deterrence)** | deviates, caught w.p. ε; PVC adds a public cheating cert | dishonest-majority-capable (≤ n−1) | correct EXCEPT w.p. 1−ε (+PVC blame cert) | 2 | **absent** (a genuine middle tier) |
| 3 | **Malicious-with-abort** | arbitrary; correct-or-all-abort | dishonest majority (≤ n−1) — the SPDZ regime | correct or abort; no fairness/liveness | 2 | `HonestMajorityAbort` (name wrongly soldered to honest-majority) |
| 4 | **Malicious-with-identifiable-abort (IA)** | arbitrary; on abort honest parties AGREE on a cheater | dishonest majority (≤ n−1) | abort + sound cheater attribution | 2 (meaningful ≥3) | **proto only** (`Tampered{cheaters}`, heuristic) |
| 5 | **Honest-majority robust / GOD** | arbitrary; honest parties ALWAYS get the right output | `t<n/3` perfect (BGW); `t<n/2` statistical (+broadcast) | correct + fairness + liveness | 3 (stat); 4 (perfect) | `HonestMajorityRobust{max_cheaters}` — collapses perfect vs statistical |
| 6 | **Dishonest-majority malicious (SPDZ family)** | ≤ n−1 corrupt; IT-MACs on additive shares | dishonest majority (no honest majority) | correct or abort (NOT GOD — Cleve) | 2 | `DishonestMajority` is an empty marker; **no backend** |

Anchoring facts: **Cleve (STOC'86)** — fairness/GOD is *impossible* without an honest
majority. **BGW** — `t<n/3` gives perfect, error-free GOD over point-to-point; `t<n/2`
gives statistical GOD requiring broadcast (Goyal-Song-Liu, "GOD Comes Free in Honest
Majority MPC", CRYPTO'20, eprint 2020/189; Goyal-Song, "Malicious Security Comes Free",
eprint 2020/134). **Covert** — Aumann-Lindell'07; **PVC** — Asharov-Orlandi, Asiacrypt'12.
**Identifiable abort** — Ishai-Ostrovsky-Zikas; Baum et al. CRYPTO'20 (eprint 2020/767);
pairwise-MAC IA CRYPTO'24 (eprint 2023/1548). **SPDZ family** — Damgård et al. SPDZ
CRYPTO'12, MASCOT (OT triples, eprint 2016/505), Overdrive (SHE triples, eprint 2017/1230).

### 1.2 The central finding: the taxonomy is THREE orthogonal axes; the code flattens two

The genuinely orthogonal axes are:

- **AXIS-1 ADVERSARY:** passive (semi-honest) | covert(ε) | active (malicious).
- **AXIS-2 OUTPUT GUARANTEE:** abort (selective | unanimous | identifiable) | fairness
  | guaranteed-output (GOD). (Fairness sits between abort and GOD.)
- **AXIS-3 CORRUPTION THRESHOLD:** dishonest-majority (`t<n`) | honest-majority
  (`t<n/2`, `n>2t`) | super-honest-majority (`t<n/3`, `n>3t`).

Today: `TrustModel` ≈ AXIS-3 but **binary** (loses the `n/3` vs `n/2` split that
separates perfect-GOD from statistical-GOD). `MaliciousSecurity` **bundles AXIS-1
(semi-honest vs active) WITH AXIS-2 (abort vs robust)** and hardcodes "HonestMajority"
into the active variant *names* — so dishonest-majority-malicious-abort (= SPDZ) is
**unnameable**, and covert and identifiable-vs-unanimous abort have NO representation.
This is fine for the single-backend v1; it is the precise thing to refactor to make
security a true configurable axis (architecture convention #7, "modularity is the
contribution", and Jesse's "configurable long-term" decision).

Two correctness/legibility bugs that fall out of the flattening, both *trust bugs* not
just missing features: (i) `HonestMajorityRobust` reads as whole-protocol GOD when it is
*per-reconstruction* and false for the degree-`2t` join path at minimal N; (ii)
`Tampered{cheaters}` reads as sound identifiable-abort when it is heuristic on the abort
path. **Guarantees should be reported PER-OPERATOR (linear aggregate vs equality/join vs
comparison) and PER-N, because they genuinely differ; one backend-level bit lies.**

### 1.3 Recommended configurable API (the deliverable)

A controlled enum refactor + a small negotiation type — *not* a rewrite. The trait
seam, the `BackendInfo` descriptor, and the architecture's A/B/C/D framing already exist.

**(a) Three-axis security descriptor** (replace the two entangled enums; keep
`MaliciousSecurity` as a back-compat *projection* so `ShamirBackend::info()` and existing
callers don't break):

```text
enum AdversaryModel { SemiHonest, Covert{ deterrence_num, deterrence_den }, Malicious }
enum OutputGuarantee { Abort(AbortKind), Fairness, GuaranteedOutput }   // AbortKind: Selective|Unanimous|Identifiable
enum CorruptionThreshold {                       // carries t so abort is expressible in BOTH majority regimes
    DishonestMajority{ t },                       // t < n
    HonestMajority{ t },                          // n > 2t  → statistical GOD (broadcast)
    SuperHonestMajority{ t },                     // n > 3t  → perfect GOD (point-to-point, BGW)
}
struct PublicVerifiability(bool)                  // PVC / public cheater cert / coZK-public-verify
```

**(b) A SELECTION/negotiation type + a fail-closed registry** — today a caller can only
inspect `BackendInfo` *post-hoc*; there is no requirement a federation states up front:

```text
struct SecurityRequirement {
    min_adversary: AdversaryModel,
    min_output_guarantee: OutputGuarantee,
    max_corruption: CorruptionThreshold,        // or required honest fraction
    require_cheater_attribution: bool,           // sound IA, not heuristic
    require_public_verifiability: bool,
}
impl SecurityRequirement { fn satisfies(&self, info: &BackendInfo) -> bool { ... } }

// registry.select(req) -> Result<&dyn MpcBackend, MpcError::NoBackendSatisfies>
// FAILS CLOSED (mirrors the NotYetImplemented honesty discipline): a request for
// dishonest-majority-malicious over the SPARQL pipeline is truthfully REFUSED,
// never silently downgraded.
```

**(c) Per-operator, per-N reporting** — `BackendInfo` (or a `JoinPlan`-adjacent report)
keyed by operator class {linear-aggregate, equality/join, comparison} so the degree-`t`
robust vs degree-`2t` no-redundancy distinction is legible, parameterised by `(n,t)`.

**(d) Cleve as a type-level invariant** — `DishonestMajority ⇒` at most
`OutputGuarantee::Abort` (optionally `Identifiable`); `Fairness`/`GuaranteedOutput` only
representable under honest majority. The API must *never* be able to advertise a
guarantee Cleve forbids.

**(e) Honesty-anchor (do not over-claim).** Malicious-secure correctness for query eval
(Senate/ORQ) AND authenticated inputs (Dutta) BOTH exist ONLY honest-majority;
dishonest-majority malicious correctness for SPARQL/graph query eval has **zero**
published instances. So the API must let a federation REQUEST dishonest-majority-malicious
but truthfully report that NO backend satisfies it for the SPARQL operator pipeline. The
shipped envelope remains honest-majority/semi-honest among cooperating holders, with the
RS-checked detect-and-abort hardening the one real active-security step taken so far.

---

## 2. Protocol families → (security model, N) performance map

sparq-mpc implements exactly **one** family — Shamir/BGW. Everything else here is a
literature finding, NOT something the code measures. The map below is for the
four-flatmates-and-a-landlord setting (any N, cooperating holders, LAN-first).

| Family | Security regime | N scaling | Mult cost | Best for | In crate? |
|---|---|---|---|---|---|
| **Shamir / BGW (DN07, ATLAS)** | honest-majority; malicious "free" (Goyal-Liu-Song) or RS-robust | **any N**; free linear (0 rounds) | DN07 ~6 field elts/mult/party (O(n) total); classic BGW O(n²) | semi-honest, general N, LAN, mostly-linear (the v1 sweet spot) | **YES** (no DN07 degree-reduction → only 1 product) |
| **Replicated 2-of-3 (ABY3, Falcon)** | honest-majority; Falcon malicious (~2×) | **fixed n=3** | 1 ring/field elt/party/mult (the empirical 3PC champion) | semi-honest/malicious at N=3 only | no |
| **Additive + SPDZ/MASCOT/Overdrive** | dishonest-majority malicious (abort) | online O(n); offline O(n²) channels | Beaver-triple-consumed; preprocessing dominates | hostile cross-org federation | no (documented slot-in behind the trait) |
| **GMW (Boolean/arith)** | semi/malicious; dishonest-majority | round-per-AND-DEPTH; O(n²) channels | OT/triple per AND | shallow Boolean (bit-level FILTER comparisons) | no |
| **BMR / multi-party garbled** | semi/malicious (HSS17) up to n−1 | **constant rounds** (WAN winner); O(n²) garbling material | garble-once, eval-local | WAN federation, deep circuits (in-circuit sig/hash) | no |
| **FSS / DPF** | cheap case is **2-server** semi-honest | sublinear comm; near-non-interactive online | n/a (function shares) | private point/range/equality lookup, PIR (RQ2a) | no |

**Most performant per regime (the answer the question asks):**

- **(semi-honest, N=2):** FSS/DPF for selection/comparison/PIR (sublinear comm,
  near-non-interactive) OR garbled circuits for general 2PC. *sparq has neither; its
  Shamir at n=2 has t=0 → no privacy threshold, effectively trivial.*
- **(semi-honest/malicious, N=3):** **replicated 2-of-3** (ABY3/Falcon) — 1 elt/party/
  mult. *sparq runs Shamir at n=3 (t=1), paying more than replicated would.*
- **(semi-honest, general N, LAN, mostly-linear):** **Shamir + DN07 mult** — exactly the
  family sparq chose, CORRECT for the linear aggregate (free addition). *sparq lacks
  DN07, so any non-linear N-party circuit is unbuilt.*
- **(malicious, general N, LAN):** malicious honest-majority Shamir (Goyal-Liu-Song
  "free" compiler) if honest majority; **SPDZ/MASCOT** if dishonest-majority (heavier).
  **Senate (USENIX'21)** is the relational SOTA: malicious n-party with circuit
  DECOMPOSITION (sub-circuits on party-subsets in parallel), up to 145× over generic
  AGMPC — directly relevant to federated-SPARQL source-combination planning.
- **(any model, WAN / deep circuits):** **BMR garbled** (constant-round) beats
  round-per-depth secret sharing. *sparq's round-per-depth Shamir is the WRONG family for
  WAN — honestly scoped out (v1 is LAN/datacenter only).*

**Load-bearing honesty point:** sparq's Shamir choice is OPTIMAL for the v1 target
(linear aggregate, cooperating holders, LAN, any N) and WRONG/unbuilt for everything
beyond it (WAN→BMR, N=3-speed→replicated, dishonest-majority→SPDZ, sublinear lookup→FSS,
multiplication chains→DN07 degree reduction). Because the crate is an in-process
simulation, NONE of the O(N) vs O(N²)-channel / round-complexity scaling that defines
this dimension is exercised by running code.

Sources: Damgård-Nielsen DN07 (CRYPTO'07); ATLAS (eprint 2021/833); ABY3 (CCS'18);
Falcon (PETS'21); MASCOT (eprint 2016/505); Overdrive (eprint 2017/1230); GMW'87;
HSS17 (Asiacrypt'17); row-reduction n-party garbling (eprint 2025/829); DPF
(Gilboa-Ishai EUROCRYPT'14; eprint 2018/707); Senate (USENIX'21); Secrecy (NSDI'23);
ORQ (SOSP'25, eprint 2025/1657); GORAM (VLDB'25); S&S 2022 survey.

---

## 3. Per-SPARQL-operator best-secure-realization matrix

The unifying insight from relational MPC (ORQ SOSP'25, Conclave, Secrecy, Senate, SMCQL):
secure relational evaluation is dominated by ONE primitive family — **oblivious
shuffle+sort** — and every set/join/dedup/aggregate operator should reduce to a
sort-then-linear-scan so the pipeline costs **O(n log n) work, O(log n) rounds, O(n)
memory** (n = total input rows). This is the cost model sparq-mpc must adopt for the
hidden regime and is the single biggest gap vs the current `O(|L|·|R|)` all-pairs join.

**The two-regime split (architecture §4.3 + convention #4) is load-bearing:** operators
split into (1) **DISCLOSED-multiset** operators recomputed by the verifier OUTSIDE the
crypto (DISTINCT, ORDER BY, LIMIT/OFFSET, COUNT/SUM/AVG/MIN/MAX, and joins/UNION/OPTIONAL
over disclosed values), and (2) **HIDDEN** operators that must run inside MPC. Many
operators thus have a *zero-crypto* realization when their operands are disclosed global
IRIs — `DisclosedKeyJoin` already ships this for inner joins. The expensive MPC
realizations are needed ONLY for the hidden regime.

| SPARQL operator | clear-text (sparq-engine) | best HIDDEN-regime MPC realization | cost | security models | sparq today |
|---|---|---|---|---|---|
| **BGP / multi-pattern join** | `eval_bgp*` + `hash_join`/`merge_join` | ORQ oblivious sort-merge join-aggregation (inner/outer/semi/anti, composes) | O(n log n) | semi-honest (ORQ 3PC); malicious honest-majority (ORQ 4PC Fantastic Four); SPDZ dishonest-maj | `HiddenValueJoin` O(\|L\|·\|R\|) all-pairs |
| **FILTER (=, ≠)** | `eval_expr`/`eval_function` | equality-to-zero (d=a−b, mask, open d·r) | 1 mult/cmp | semi-honest (built); malicious needs IT-MAC at n=2t+1 | `secure_equal` BUILT |
| **FILTER (<, ≤, >)** | comparison ops | Rabbit (eprint 2021/119) / edaBits (Crypto'20) bit-decomposition | ~log p bit-ops | honest- & dishonest-majority | **absent** (but disclosed→verifier recomputes) |
| **SUM / COUNT** | `group_aggregate` | LINEAR → free local share-addition (0 rounds) | free | all | BUILT (`run_secure`) |
| **AVG** | `eval_aggregate` | secure SUM + division by (disclosed/public) count | + 1 div | all | absent |
| **MIN / MAX** | `eval_aggregate` | secure-comparison tournament (log n rounds Rabbit) OR oblivious-sort + extreme | O(n log n) | all | absent |
| **GROUP BY (hidden key)** | hash-group | ORQ sort-on-key + segmented prefix-sum gated by secret is-new-group bit | O(n log n) | all | absent (disclosed-key → verifier; hidden forbidden v1) |
| **DISTINCT / dedup** | `distinct_bindings` | oblivious sort + adjacent-equality scan + oblivious compaction | O(n log n) | all | absent (disclosed → verifier recomputes) |
| **ORDER BY** | `order_bindings` | oblivious sorting network (bitonic O(n log²n) / Waksman O(n log n)) | O(n log n) | all | absent (disclosed → verifier recomputes) |
| **OPTIONAL / MINUS / UNION** | `join/minus/union_bindings` | ORQ outer/anti/concat (same sort-merge, different gating bits; UNION = oblivious concat + padding) | O(n log n) | all | inner-only disclosed-key |
| **Property paths** | `eval_path` (transitive-closure fixpoint) | WORST operator: transitive closure of a SECRET edge set leaks structure + super-linear; bounded length only (GORAM is confidentiality-only ego-centric, NOT correctness) | bounded only | n/a unbounded | absent; scope to bounded |
| **sub-SELECT** | nested eval | composes IFF inner result disclosed (recompute) or kept secret-shared (ORQ "compose with itself") | inherits | all | absent |
| **Cross-party (federated) join** | `eval_service` | **the headline:** global IRI is a public cross-holder id → disclosed-key join is crypto-free; only HIDDEN values enter circuit-PSI | O(\|L\|+\|R\|) disclosed | all | `DisclosedKeyJoin` BUILT (sparq's genuine lead) |

**Single hidden equi-join (alternative to sort-merge):** circuit-PSI via cuckoo+simple
hashing (Pinkas et al. EUROCRYPT'18; VOLE-PSI eprint 2021/266 — malicious 2²⁰ in 6.2s;
structure-aware FSS-PSI eprint 2025/907) gives ~linear cost and secret-shared match bits.
It does NOT compose for free into a multi-pattern BGP (use sort-merge there).

**The substrate gap.** The entire oblivious shuffle+sort foundation is ABSENT. Oblivious
radix sort (Hamada eprint 2014/121, no private comparisons) or Waksman-network shuffle
(Waks-On/Waks-Off CCS'23, eprint 2023/1236, 2.7–3.5× over bitonic — use Waksman not
Benes; Benes is biased). Building it unlocks ~6 operators at once and is the
highest-leverage single addition.

**Two more honest gaps:** (i) general Shamir multiplication needs degree reduction
(BGW/DN resharing) before any multiplication chain (comparison, segmented group-agg
scan, AVG) is buildable — `mul_shares_raw` only does one non-reducing product; (ii) the
degree-`2t` equality open at n=2t+1 has zero RS redundancy → needs an IT-MAC for
malicious security (guarantee D) on the hidden-join path.

---

## 4. Privacy / leakage analysis + the privacy-vs-performance frontier

### 4.1 What sparq hides today, and what it leaks

The two cross-holder paths sit at OPPOSITE ends of the frontier:

- **`DisclosedKeyJoin`** — the LEAKIEST configuration *by design* (convention #4): join
  keys, every joined value, the per-holder partial, the result cardinality, AND which
  holder contributed each row are all in cleartext. It hides only non-projected triples.
  Acceptable only when keys are genuinely public global IRIs and the
  no-proof-of-revealed-properties rule means the join was checkable in cleartext anyway.
- **`HiddenValueJoin`** — hides the join KEY values and their difference, leaking only
  the match bit per pair, and the aggregate hides per-holder addends opening only the sum.

But even `HiddenValueJoin` leaks four things RQ2a wants hidden:

- **(L1) Final result cardinality.** `out_rows` is exactly the true number of matches;
  `canonicalize_rows` sorts but does NOT pad. The classic oblivious-DB leak
  (ShrinkWrap/SAQE: an oblivious algorithm pads to worst-case; for joins worst case is
  `|L|·|R|`; sparq pads to NOTHING — reveals true size).
- **(L2) Per-pair match pattern / access pattern.** `secure_equal` runs a fixed `|L|·|R|`
  loop (the *number* of comparisons is input-independent — good), but each opened `m`
  reveals the match bit at (i,j). The set of matching (i,j) IS the bipartite match graph
  → the join-key fan-out/multiplicity distribution leaks (a strong fingerprint of the
  private key distribution even though keys stay hidden).
- **(L3) Input cardinalities `|L|`,`|R|`.** Loop bounds are public; standard MPC
  assumption, but the *number of credentials a holder has* can itself be sensitive.
- **(L4) Source provenance / row-linkability.** Disclosed payload columns are emitted
  as-is; M4 v1 *deliberately* gives up source-unlinkability (verifier checks `pk_i` in
  the clear). So today AND in planned M4 v1, the contributing source is linkable.

**The malicious × confidentiality interaction (D × A):** at n=2t+1 the degree-`2t`
equality open has zero RS redundancy, so a malicious party can flip a verdict
undetectably AND — per coZK eprint 2025/1026 — computing/proving on an inconsistent
witness can LEAK honest inputs. So malicious deviation is not only a correctness problem;
it is a confidentiality problem. IT-MAC (`sq-6d6g`) is the named fix.

### 4.2 The seven-channel leakage taxonomy and the frontier (cheapest → fullest mitigation)

| Channel | Cheapest mitigation (residual) | Full mitigation (cost) |
|---|---|---|
| **Query structure / which BGP** | publish query (no hiding) — acceptable: verifier ISSUES it | PFE / universal circuit — O(z log z) boolean, up to O(z⁵) arith (Mohassel-Sadeghian 2013/137); almost never worth it |
| **Which holder has a triple (membership)** | per-holder local eval discloses partial | BGP via MPC over committed data (Merkle-inclusion under MPC) — heavy |
| **Join KEY values** | disclose IRIs (`DisclosedKeyJoin`) | secret-shared equality (`HiddenValueJoin`) — 1 mult/pair |
| **Join match pattern (L2) + fan-out** | none today (per-pair open) | aggregate match bits inside MPC, never open per-pair; oblivious-shuffle then padded-prefix reveal |
| **Intermediate cardinalities** | none today (true sizes flow) | full obliviousness (pad to `\|L\|·\|R\|`) OR DP-padding to a noisy cardinality (ShrinkWrap) |
| **Final result size (L1)** | none today (true `\|result\|`) | pad to public bound (exact) OR DP result-size (Laplace/geometric noise + dummies) — cheap, leaks only ε |
| **Access pattern over a store** | none (in-proc sim has no RAM accesses) | DORAM (Duoram, Floram, 3PDORAM) — O((κ+D)log N)/access; only once data exceeds RAM-in-MPC |
| **Source provenance (L4)** | none (M4 v1 reveals `pk_i`) | in-circuit BBS-in-ZK key-set membership (unlinkable) — Q1, "the join nobody has built" |

### 4.3 The frontier, concretely (most-private configuration per scenario)

- **Aggregate-only query (the £100k use case):** current `run_secure` cumulative-sum +
  verifier recomputes `>£100k` outside crypto. **Already near the frontier** — hides
  addends, leaks only the disclosed boolean/sum; output is a scalar so NO oblivious
  shuffle/ORAM/padding needed. The one residual is the malicious flip at n=2t+1 (add
  IT-MAC). Privacy-optimal and cheap.
- **Private-key join returning a SET:** `HiddenValueJoin` + (missing) result-size padding
  + (missing) match-bit aggregation. To be fully oblivious: (a) pad output to a public or
  DP bound, and (b) do NOT open per-pair match bits — instead **oblivious-shuffle**
  (Waksman, O(n log n)) the matched rows, then reveal only a padded prefix. The standard
  oblivious-join output path, entirely absent from the crate.
- **Multi-pattern BGP (Q3/RQ2b):** obliviousness padding compounds per pattern. The
  escape hatch (convention #4: disclose global-IRI join keys, check joins out-of-circuit)
  keeps it tractable at the cost of leaking those keys. ORQ shows even with O(n log n)
  fusion the honest envelope is minutes-to-tens-of-minutes; joins are the cost center.

### 4.4 The highest-leverage, lowest-cost privacy upgrade

**Differentially-private result-size / output-cardinality protection** (ShrinkWrap, SAQE,
Doquet/Adore differential obliviousness). Full obliviousness pads joins to `|L|·|R|`
(catastrophic); DP-padding pads to a *noisy* cardinality that is provably (ε,δ)-private
and only modestly above the true size — Doquet/Adore report up to an order-of-magnitude
speedup vs full obliviousness. For the four-flatmate aggregate the result is a fixed-size
boolean so L1 is already closed for THAT query; the moment a query returns a *set* (any
non-aggregate SELECT projection), result-size leakage is live and unmitigated.

**Residual-leak honesty (state loudly per the empirical-honesty rule):** DP result-size
leaks the query exists + an (ε,δ)-noised size; over repeated queries the budget composes
(must track an ε budget — ShrinkWrap's optimizer). NOT information-theoretic. Differential
obliviousness on access patterns (Doquet/Adore) is (ε,δ)-DP, not perfect hiding (designed
for TEE; the length-DP idea transfers to MPC opened values). Disclosed-key join leaks
everything disclosed. Per-pair `secure_equal` opening leaks the full match graph (L2).

Sources: ShrinkWrap (arXiv 1810.01816); SAQE (PVLDB 2020); Doquet (PVLDB v16 p4160);
Adore (PVLDB v16 p842); Duoram (USENIX'23); Waks-On/Waks-Off (eprint 2023/1236);
Mohassel-Sadeghian PFE (eprint 2013/137); ORQ (eprint 2025/1657); Dutta (eprint 2022/1648).

---

## 5. Benchmark matrix (security × N × query class × data scale) + metrics + methodology

**Current state:** sparq-mpc has NO benchmarks (no `benches/`, not in
`bench/benchmarks.toml`, not in the dashboard). M6 is the empty milestone. All
performance evidence is qualitative prose. The `insecure-test-rng` feature exists
precisely for reproducible benches. CRITICAL HONESTY CONSTRAINT: the crate is an
in-process simulation, so the dominant real-world cost (rounds × RTT, bytes) is NOT
observable. The harness's #1 job is to make that cost MODELLED (tier 1) and then REAL
(tiers 2/3), and to NEVER let a single-process wall-clock masquerade as an MPC latency.

### 5.1 The matrix (stays inside the honest "viable regime", flags cells outside as no-data)

- **AXIS 1 — security model:** the *actual* `MaliciousSecurity` reachable from
  `ShamirBackend::new(n)`, recorded per cell via `info()` — NOT a requested one. For the
  degree-`t` aggregate: always has redundancy (Abort at even-N `e=0`, Robust at larger
  N). For the degree-`2t` equality/join: **SemiHonestOnly at n=2t+1** (odd N — zero RS
  redundancy), Abort at even N, Robust only at n≥2t+3. So the security axis is a *function
  of (n, primitive, t)*, not free to choose. `DishonestMajority` cells are honest
  "N/A — no backend" (never a faked number).
- **AXIS 2 — number of parties N ∈ {2,3,5,7,9,11}** (+ even {4,6,8} to exercise the
  Abort-vs-Robust boundary). `t=⌊(n−1)/2⌋`. Two sub-regimes: minimal `n=2t+1` (cheapest,
  weakest active security at degree-2t) vs over-provisioned `n=2t+3` (robust equality).
- **AXIS 3 — query class:** (a) cumulative-SUM aggregate (the £100k case — zero-round
  linear baseline); (b) disclosed-key equi-join (crypto-free — the cost FLOOR, isolates
  engine from MPC); (c) hidden-value equi-join (the COST CENTER — `|L|·|R|` equalities);
  (d) chained joins (the 3-holder differential); (e) BGP/multi-pattern (Q3/RQ2b — flag as
  not-SOTA-cost). Each maps to an existing differential test so correctness co-gates cost.
- **AXIS 4 — data scale:** rows/party ∈ {10, 100, 1000, 10000} for joins (`|L|=|R|` so
  pair-count = scale² — 10⁴ → 10⁸ equalities: a deliberate ceiling probe behind a heavy
  tier with df/time caps); for the aggregate, N *is* the scale. Cap at 10⁴/party; never
  extrapolate past it.

### 5.2 Metrics (smaller-is-better, to match the dashboard's `customSmallerIsBetter`)

- **Tier-1 (in-process):** online wall-clock (ms), peak RSS (MiB), and — load-bearing —
  **MODELLED communication**: `bytes_per_party` (counted, not timed: each `F_p` opened =
  8 bytes/party; instrument `share()`/`reconstruct_degree()`/open to count field elements
  crossing the abstract party boundary) and `round_count` (logical rounds: aggregate = 0
  mult rounds + 1 open; each `secure_equal` = 1 mult round + 1 open; all-pairs join =
  `|L|·|R|` opens — the round-count explosion the literature flags).
- **Tier-3 (real-process):** network-bound wall-clock under each tc/netem profile, so
  `latency = f(rounds, RTT, bytes, bandwidth)` becomes observable.
- **Offline/preprocessing:** currently ZERO for semi-honest Shamir — record it as 0
  EXPLICITLY so a future SPDZ backend's preprocessing shows up as a regression, not a
  hidden cost (the Ozdemir-Boneh / ORQ lesson: preprocessing is the number papers hide).
- **Throughput:** secure-equalities/sec, triples-aggregated/sec.

### 5.3 N-party simulation methodology — three escalating-fidelity tiers

- **TIER 1 (in-process, default, CI-cheap):** criterion benches in
  `crates/sparq-mpc/benches/` over `ShamirBackend` with `insecure-test-rng` (reproducible
  masks). Measures compute + MODELLED comm/rounds. Deterministic byte/round counts
  (load-robust); wall-clock quiet-box-sensitive. The only tier that runs per-commit.
  HONEST LABEL required: "single-process simulation; network cost MODELLED not measured."
- **TIER 2 (multi-process, single box, loopback):** N OS processes over 127.0.0.1 —
  real serialization + real socket syscalls, ~0 RTT. This requires a NEW transport layer
  the crate lacks (the backend is coordinator-free function calls today), so it is itself
  a build item. Gives real bytes-on-wire and real round latency at LAN-ideal.
- **TIER 3 (multi-process + tc/netem, single box):** the standard MPC-DB method (MP-SPDZ,
  Secrecy, ORQ all do this). Linux `tc qdisc netem` on loopback/veth. PROFILES from the
  literature: **LAN = 1 Gbps / 1 ms RTT** (some use 10 Gbps/0.01 ms); **WAN = 100–200
  Mbps / 20–100 ms RTT**, optional 0.1–1% loss. ORQ reports a WAN slowdown multiplier
  (1.2–6.9×) vs LAN for the same workload — emit that ratio. Needs root/CAP_NET_ADMIN →
  nightly/EC2 only, never per-commit. Multi-box (separate EC2, real inter-AZ/region) is a
  tier-4 stretch, cost-capped via the orphan-proof `ec2-bench.sh`; single-box tc/netem is
  the defensible WAN proxy for v1 (and is what the cited systems use for most numbers).

### 5.4 Prior MPC-DB benchmarking survey (how to be credible)

- **ORQ (SOSP'25):** full TPC-H SF10, LAN AND WAN, per-query wall-clock + LAN→WAN
  multiplier; default 3PC semi-honest, malicious = 4-party Fantastic Four; lesson: joins
  are the cost center, obliviousness forces worst-case padding (= the `HiddenValueJoin`
  all-pairs structure). Q21 ≈ 42 min LAN malicious — minutes-to-tens-of-minutes, NOT "a
  few minutes".
- **Secrecy (NSDI'23):** 3PC semi-honest, 1K rows/relation keeping TPC-H size RATIOS,
  AWS-LAN AND multi-cloud; >1000× over naive; emphasizes communication-round reduction
  (validates `round_count` as first-class).
- **Senate (USENIX'21):** genuinely-malicious n-party; reports scaling in N.
- **Conclave (EuroSys'19):** MPC+cleartext hybrid; reports leakage/speed tradeoff (=
  the `DisclosedKeyJoin` cost-floor path).
- **MP-SPDZ:** localhost-all-parties + tc/netem LAN(1Gbps/1ms)/WAN(~100-200Mbps/20ms) is
  the canonical setup; reports wall-clock + data-sent (bytes) + rounds per party — exactly
  the proposed metric triple.
- **Collaborative zk-SNARKs (Ozdemir-Boneh USENIX'22):** proving-time vs NUMBER-OF-PROVERS
  over 3 Gb/s; N−1 malicious ≈ 2×; EXCLUDES preprocessing — the cautionary precedent for
  the offline-cost metric and the M4 collaborative-proof bench (when it exists).
- **Scalable coZK (USENIX'25):** 128 servers / 2²¹ gates but proof-DELEGATION, a PoC —
  the precedent for honestly labelling a tier as "PoC, not production."

**Grounding-in-code the harness must respect:** `secure_equal` needs `n ≥ 2t+1`; the
equality open is degree-`2t` so its RS redundancy (and thus its security-axis value)
differs from the aggregate's degree-`t` — bench BOTH primitives and label each cell's
actual guarantee. Use the seeded test RNG for reproducibility, NEVER claimed as
production. Differential correctness (join == union-store eval; secure-sum == plaintext
sum) runs alongside every cost cell so a fast-but-wrong cell is caught. All scaled runs
obey the disk/EC2 discipline: df watchdog, /tmp scratch cleanup, dataset cap,
orphan-proof self-terminate.

---

## 6. SOTA-systems comparison + the realistic ACHIEVABLE ENVELOPE

### 6.1 Relational MPC analytics (the mature line — lessons for sparq's planner)

- **SMCQL (PVLDB'17):** 2-party HBC, garbled circuits + ORAM; the *split-execution* idea
  (public slice in plaintext + secure slice in MPC, a "split" annotation deciding the
  boundary) = sparq convention #4. Slow: >1h at 200k records.
- **Conclave (EuroSys'19):** N-party semi-honest hybrid MPC+cleartext; STP/hybrid
  annotations trade a *quantified* leak for ≥7× (often orders-of-magnitude); full TPC-H
  SF-10. Lesson: leakage-for-speed is real but must be a DECLARED bounded leak (RQ2a).
- **Senate (USENIX'21):** N-party MALICIOUSLY secure (the standout); MPC-decomposition +
  a planner that runs sub-circuits on party-subsets in parallel; up to 145× over prior
  malicious MPC. Lesson: malicious N-party relational analytics IS achievable, and
  decomposition+planning is what makes it tractable — directly relevant to sparq's
  untrusted-planner stance.
- **Secrecy (NSDI'23):** 3PC replicated SS semi-honest; oblivious operators with
  composition-level quadratic→linear rewrites; >1000× over naive. Lesson: the
  oblivious-operator cost model is the template `HiddenValueJoin` lacks.
- **SAQE (PVLDB'20):** jointly optimizes DP noise + AQP sampling + secure computation;
  accuracy bounds depend on sample size not raw size. A documented escape hatch for large
  aggregates when exact answers aren't required (orthogonal to sparq's exact-answer goal).
- **Cerebro (USENIX'21):** MPC for ML, but contributes compute/release POLICIES +
  cryptographic AUDITING (accountability) — the fallback tier (covert/PVC) when full
  malicious soundness is too costly.
- **ORQ (SOSP'25):** the SOTA + the honest performance anchor — first full TPC-H under
  MPC, "a few minutes/query on LAN", oblivious sort up to ~0.5B rows, join-aggregate
  fusion eliminating the quadratic secure-join cost. THE headline: even SOTA relational
  MPC is **minutes-to-tens-of-minutes per query on LAN**, joins are the cost center.
  Adjacent: Alchemy (oblivious-SQL optimizer), MapComp (view-based join-group-agg),
  Jodes (distributed oblivious join), SecretFlow-SCQL (production secure query platform).

### 6.2 Secure/verifiable federated-RDF/SPARQL & graph work (closest prior art + gaps)

- **GOOSE (DBSec'20):** SPARQL UCRPQ, honest broker + HBC DBs, NO COUNT/SUM/AVG;
  confidentiality-flavored, not correctness/attestation.
- **SMPG / PPMQ (DBSec'24 / SN CS'25):** Cypher on Neo4j, Shamir via JIFF, semi-honest,
  conjunctive SPJ only. Its sub-ms numbers are SINGLE-DESKTOP, CO-LOCATED, NO inter-party
  network — non-evidence for federation.
- **GORAM (PVLDB'25):** ego-centric queries on federated graphs over ABY3 (semi-honest
  honest-majority 3PC); 58 ms–35.7 s up to 41.6M vertices / 1.4B edges. BUT "no party
  learns the graph" is CONFIDENTIALITY ONLY (not correctness, not attestation) and it is
  ego-centric traversal, NOT general BGP/SPARQL.
- **VeriDKG (PVLDB'24):** VERIFIABLE (not confidential) SPARQL — RGB-Trie + accumulator,
  integrity against a cheating server, NO hiding. A different guarantee axis.
- **FedUP (WWW'24):** NON-crypto federated SPARQL planner — result-aware plans via
  provenance over quotient summaries; orders of magnitude over FedX/ANAPSID/CostFed.
  Relevant to sparq's planner: prune empty source-combinations BEFORE any MPC (cheap
  plaintext source-selection in front of expensive MPC), as the untrusted hint-producer.

### 6.3 The verdict (the realistic envelope of MPC+SPARQL)

**Every published graph/SPARQL crypto system is semi-honest AND/OR confidentiality-only
AND/OR conjunctive-only. Malicious-secure, attested-input, full-SPARQL federation has
ZERO published instances.** The relational line (Senate/Secrecy/ORQ) shows malicious +
multi-operator IS possible but at minutes-per-query on LAN with joins as the dominant
cost, and it carries neither attestation (guarantee C) nor global-IRI keys. sparq's
distinguishing bets — global IRIs as cross-credential join keys (vs node-local ids in
GOOSE/SMPG), attested-source derivation (the Dutta eprint 2022/1648 pillar), and a single
collaborative ZK proof of correctness+attestation — are in NO prior system; **the
composition is the contribution AND the principal research risk.**

- **REALISTIC TODAY (the viable regime):** honest-majority (or honest-but-curious among
  cooperating holders), LAN/datacenter, ≤10³–10⁴ triples/party, few-pattern BGPs,
  disclosed-key joins computed in plaintext (DONE), hidden-value equi-joins via
  secret-shared equality (DONE, all-pairs), disclosed-aggregate recompute by the verifier
  (M5, not built). Aligns with the sub-second-for-≤10³ RQ1 figure.
- **ASPIRATIONAL / unquantified research risk (budget as such, never "seconds"):**
  dishonest-majority malicious on a WAN; heavy multi-way BGP joins; and the in-circuit
  distributed signature-over-secret-shared-witness ("the join nobody has built", Q1) the
  attestation half needs. The full composition (coZK ⊕ malicious dishonest-majority MPC ⊕
  oblivious BGP joins ⊕ attested inputs ⊕ WAN) has ZERO performance data points.

---

## 7. Other desirable MPC properties × the configurable model and N

- **Fairness & GOD.** Cleve forbids both without honest majority — encode as a type-level
  invariant (§1.3d). `HonestMajorityRobust{max_cheaters}` is *per-reconstruction*
  robustness (a form of GOD on one open), NOT whole-protocol GOD, and it is FALSE for the
  degree-`2t` join path at n=2t+1. The model also collapses perfect-GOD (n>3t, BGW,
  point-to-point, error-free) and statistical-GOD (n>2t, broadcast, negligible error) —
  add `SuperHonestMajority{t}` and report perfect-vs-statistical per actual (n,t).
- **Identifiable abort (IA) & cheater attribution.** Real IA = honest parties AGREE on a
  cheater's identity (Baum CRYPTO'20 eprint 2020/767; pairwise-MAC IA CRYPTO'24 eprint
  2023/1548). sparq has detection (sound) + correction (sound within budget) + heuristic
  blame (UNSOUND on the abort path — can blame an honest party). Surfacing the BW-SUCCESS
  cheater set IS sound (recovered Q pins disagreeing points); the abort-path blame is not.
  Add an `AttributionQuality{Sound, Heuristic}` marker so callers never treat heuristic
  blame as IA. True IA additionally needs a broadcast channel + per-party authenticated
  transcripts (IT-MACs), which the in-process sim lacks — so IA should be explicitly
  deferred behind the dishonest-majority backend, not half-implemented via RS blame.
- **Input vs function/query privacy.** sparq delivers INPUT privacy (Shamir threshold)
  and deliberately NOT function/query privacy (query is public, digest bound, planner
  public; PFE/universal-circuit is the would-be capability, explicitly forgone by
  convention #4). The one genuine residual is the join MATCH STRUCTURE leak (L2/§4.1).
  Surface a `LeakageProfile` so a federation can reason about residual leakage.
- **Composability / UC.** The whole architecture is a COMPOSITION (holder → join → aggregate
  → proof). `secure_equal` OPENS a value mid-computation — exactly what naive sequential
  composition does not justify. Honest-majority protocols can achieve UC without a CRS/PKI trust
  root — but only GIVEN the theorems' communication-model resources (ideal authenticated/private
  channels, broadcast where required, the UC session/scheduling model; Canetti FOCS'01) — an
  argument FOR the default as a design target; applying this to today's in-process code is
  aspirational. The collaborative-proof layer inherits the coZK 2025/1026 pitfall (proving on
  an inconsistent extended witness leaks honest inputs) — a composition-security failure a UC
  treatment would catch. The composition/UC posture (which results apply + the per-stage
  obligations + the `secure_equal`-open justification) is now recorded in
  [`mpc-composition-uc-posture.md`](./mpc-composition-uc-posture.md) (sq-wj4k).
- **Offline/online (preprocessing) split.** Honest-majority Shamir needs NONE for its
  current ops (linear = zero-round, single mult = local product + open). A SPDZ/MASCOT/
  Overdrive backend adds an input-independent offline phase (Beaver triples + MACs); the
  online consumes them and MAC-checks before opening — the textbook split. KEY: `BackendInfo`
  has NO `requires_preprocessing`/cost field, so a federation cannot budget the
  dominant-usually-excluded cost. Chained multiplications (degree reduction or triples)
  also force this once comparison/group-agg/AVG are needed.
- **Randomness / CSPRNG + trusted setup.** sparq is honest at the SINGLE-DEALER level
  (OS-seeded ChaCha20, uniform rejection sampling, insecure PRNG cfg-gated out). Two gaps
  for real deployment: (a) no trusted dealer in a real federation — each holder must
  VSS-share its OWN input and parties must JOINTLY generate masks via distributed
  coin-tossing or, far cheaper in honest-majority, **PRSS** (replicated PRF seeds, eprint
  2021/1223, IETF draft-thomson-ppm-prss) — `dealer()` is a stand-in; a maliciously-fixed
  mask `r=0` would flip equality verdicts. **Design + code seam landed** for (a):
  `research/mpc-distributed-randomness-design.md` (PRSS-vs-coin-toss decision, dealer-less
  VSS, the `r=0` defense) + the `randomness` module (`DistributedRandomness` /
  `RandomnessModel`; the current dealer reports `TrustedDealerSim`, `deployable() == false`);
  the PRSS/coin-toss/VSS impl is follow-on beads behind the seam (sq-yyro). (b) The MPC core needs NO trusted setup
  (information-theoretic — a genuine trust-minimality advantage), but the collaborative-ZK
  layer does (a Groth16-style coSNARK needs a per-circuit CRS; a transparent system —
  UltraHonk/STARK, sparq's verifier target — avoids it). "Trusted-setup-free" is true for
  the SS core and FALSE/dependent for the proof layer — report separately.
- **Post-quantum.** The SS core is PQ for FREE at the information-theoretic level (Shamir
  over `F_p` has no computational assumption → PQ confidentiality + RS integrity). Caveats
  at the boundaries: ChaCha20 masking is PQ-fine; a dishonest-majority SPDZ/MASCOT backend
  must pick PQ-safe OT/SHE (lattice); the ATTESTATION + collaborative-proof layer is the PQ
  risk — EdDSA/BBS+ signatures and Pedersen commitments are pre-quantum, and no PQ
  collaborative-zkSNARK exists (lattice SNARKs Labrador/Greyhound 2025 make it conceivable
  long-horizon). Honest claim: **PQ confidentiality+integrity in the MPC core TODAY; PQ
  attestation/proof NOT today.** No per-component PQ-posture field exists anywhere.

---

## 8. Highest-leverage next steps (priority order)

1. **Oblivious shuffle+sort substrate (Waksman/radix)** — the single highest-leverage
   primitive; unlocks DISTINCT, ORDER BY, GROUP BY-over-hidden, MIN/MAX, OPTIONAL/MINUS,
   and ~linear joins at once. Without it the hidden regime is stuck at the all-pairs join.
2. **Tier-1 benchmark + comm/round COUNTING instrumentation** — make the N-scaling story
   empirical (deterministic byte/round counts) and per-cell security-guarantee-accurate;
   the prerequisite for honestly claiming ANY per-regime performance verdict (M6 is empty).
3. **The configurable 3-axis security API + fail-closed selection registry** — the
   "configurable long-term" deliverable; turns the trust model into a stated requirement a
   backend is matched against, refusing (not downgrading) dishonest-majority-malicious.
4. **DP result-size + oblivious-shuffle output path for set-returning joins** — the
   cheapest real privacy win (closes L1/L2 for any non-aggregate SELECT).
5. **IT-MAC for the degree-`2t` equality open at n=2t+1** — closes the one real
   active-security (and confidentiality, per 2025/1026) hole on the hidden-join path,
   promoting `secure_equal` from SemiHonestOnly to Abort at minimal N.
6. **DN07/ATLAS degree-reduction multiplication** — turns the backend from "linear + one
   equality" into a general honest-majority arithmetic engine (prereq for comparison,
   group-agg scan, AVG).
7. **M5 verifier-recompute path for disclosed-multiset operators** — makes the
   zero-crypto disclosed regime the fast default (convention #4 end-to-end).
8. **M4 collaborative-proof spike (v1 = Artemis commit-and-prove anchor + Dutta
   authenticated-input)** — the attested-source half; hard-blocked on the RQ1 ZK
   remediation and a fresh coZK soundness audit; the full in-circuit unlinkable version
   ("the join nobody has built") is two research steps out.

---

## 9. Open questions carried forward

1. Does the four-flatmates use case actually need dishonest-majority AMONG holders, or is
   honest-majority-among-cooperating-holders defensible? (architecture §5.2 Q2) — decides
   whether the SPDZ branch is ever built vs just being a truthfully-refused requirement.
2. Is v1 truly LAN (round-per-depth Shamir OK) or WAN (constant-round BMR garbled wins)?
   The architecture scopes v1 to LAN; RQ2 "federation" implies WAN.
3. Q3/RQ2b: does disclosing global-IRI join keys collapse enough per-pattern obliviousness
   padding to keep multi-pattern BGPs tractable, and for which exact SPARQL fragment?
4. Can Senate-style circuit DECOMPOSITION be applied to federated-SPARQL source-combination
   planning to avoid all-N participation in every operator (the most promising N-scaling
   lever, unexploited in sparq's all-N model)?
5. For source-unlinkability (L4): is the M4 v1 "verifier sees `pk_i`" leak acceptable, or
   is unlinkable in-circuit BBS-key-set membership (Q1) a hard requirement?
6. Is perfect-GOD (n>3t) ever a target, or does the linear-aggregate use case make
   statistical (n>2t) robust reconstruction always sufficient (perfect-vs-statistical
   distinction descriptive-only)?
7. Does DORAM's 2-/3-party specialization (Duoram) force a backend split if data-dependent
   private indexing is ever needed, conflicting with sparq's any-N Shamir commitment?

---

## Sources (consolidated)

Cleve (STOC'86); BGW; Aumann-Lindell covert (2007); Asharov-Orlandi PVC (Asiacrypt'12);
Baum et al. IA (CRYPTO'20, eprint 2020/767); pairwise-MAC IA (CRYPTO'24, eprint
2023/1548); Goyal-Song-Liu GOD-free (CRYPTO'20, eprint 2020/189); Goyal-Song
malicious-free (eprint 2020/134); Damgård-Nielsen DN07 (CRYPTO'07); ATLAS (eprint
2021/833); ABY3 (CCS'18); Falcon (PETS'21); SPDZ (CRYPTO'12); MASCOT (eprint 2016/505);
Overdrive (eprint 2017/1230); Le Mans (eprint 2021/1639); GMW'87; HSS17 (Asiacrypt'17);
n-party garbling row-reduction (eprint 2025/829); DPF (EUROCRYPT'14; eprint 2018/707);
Rabbit (eprint 2021/119); edaBits (Crypto'20); Pinkas circuit-PSI (EUROCRYPT'18);
VOLE-PSI (eprint 2021/266); structure-aware FSS-PSI (eprint 2025/907); Hamada oblivious
radix sort (eprint 2014/121); Waks-On/Waks-Off (eprint 2023/1236); PRSS (eprint
2021/1223; IETF draft-thomson-ppm-prss); Canetti UC (FOCS'01); Mohassel-Sadeghian PFE
(eprint 2013/137); ShrinkWrap (arXiv 1810.01816); SAQE (PVLDB'20); Doquet (PVLDB v16
p4160); Adore (PVLDB v16 p842); Duoram (USENIX'23); SMCQL (PVLDB'17); Conclave
(EuroSys'19); Senate (USENIX'21); Secrecy (NSDI'23); Cerebro (USENIX'21); ORQ (SOSP'25,
eprint 2025/1657); Alchemy/MapComp/Jodes/SecretFlow-SCQL (PVLDB'24/'25); GOOSE
(DBSec'20); SMPG/PPMQ (DBSec'24 / SN CS'25, PMC12662885); GORAM (PVLDB'25, arXiv
2410.02234); VeriDKG (PVLDB'24); FedUP (WWW'24); Ozdemir-Boneh coZK (USENIX'22, eprint
2021/1530); coZK malicious pitfalls (CRYPTO'25, eprint 2025/1026); scalable coZK
(USENIX'25, eprint 2024/940); Dutta authenticated-input MPC (Asiacrypt'24, eprint
2022/1648); Artemis commit-and-prove (arXiv 2409.12055); Labrador/Greyhound lattice
SNARKs (2025); S&S 2022 MPC survey; MP-SPDZ framework.

**In-repo ground truth:** `crates/sparq-mpc/src/{backend,shamir,robust,join,proof,holder,
partial,field,rng}.rs`, `crates/sparq-mpc/{PLAN.md,Cargo.toml}`;
`research/mpc-zkp-research-and-architecture.md` §3–5;
`research/mpc-m4-distributed-sig-feasibility.md`; `bench/{benchmarks.toml,ec2-bench.sh,
dashboard/}`; `crates/sparq-engine/src/exec.rs` (clear-text operator evaluators).
