<!-- [OPUS-4.8] Per-SPARQL-operator × per-MPC-configuration capability + performance matrix, Opus 4.8 (Fable unavailable) — design-for-review; re-review when Fable returns. -->

# MPC + SPARQL Capability + Performance Matrix (per-operator × per-configuration)

**Status:** Deep-research design record (no implementation; doc-only). Author: Opus 4.8
(Fable unavailable — flag for re-review). Date: 2026-06-15.

**What this is.** The project owner's standing directive for the MPC track asks for *"the
most performant ways of handling ALL queries at ALL configurations, the most
privacy-preserving way, any other desirable MPC properties, and a deep understanding of
exactly what can be achieved with MPC and SPARQL."* The existing records answer the
*systems* question (security taxonomy, protocol families, benchmark methodology, the M4
attestation verdict). They do **not** yet resolve it to a single **operator × configuration
cell**: for `OPTIONAL` under malicious dishonest-majority on a WAN at N=7, what is the most
performant known protocol, what does it leak, and where does sparq sit? That is this
document.

**Relationship to the other records (extend, do not duplicate).**
- [`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md) —
  the 3-axis taxonomy (AdversaryModel × OutputGuarantee × CorruptionThreshold), the
  protocol-family map, the benchmark tiers, and the achievable envelope. This record
  **adopts its vocabulary verbatim** and resolves its §3 operator table to per-cell
  granularity, adds the full configuration cross-product, and updates §0 ground truth
  (several gaps it flagged as missing are now CLOSED — see §1.1).
- [`mpc-zkp-research-and-architecture.md`](./mpc-zkp-research-and-architecture.md) — the
  four-guarantee framing (A confidentiality / B correctness / C attestation / D malicious
  security), the protocol flow, the RQ1-remediation hard gate.
- [`mpc-m4-distributed-sig-feasibility.md`](./mpc-m4-distributed-sig-feasibility.md) — the
  attestation half ("the join nobody has built"); §6 here ties each operator to it.

**Honesty contract (per the empirical-honesty rule).** Every cell is labelled with one of
four CAPABILITY TIERS, never blurred:

| Tier | Meaning |
|---|---|
| **BUILT** | realized in `crates/sparq-mpc` today (cite the file); a current-state claim |
| **KNOWN** | a published protocol exists and is implementable in sparq's model; cited by name; NOT in the crate |
| **OPEN** | active research; no off-the-shelf protocol for *this exact cell*; budget as research risk |
| **IMPOSSIBLE** | provably unachievable in this configuration (cite the impossibility) |

No fabricated performance numbers. Round/communication complexity is given as **qualitative
classes** (free / O(1) rounds / O(log n) rounds / O(depth) rounds; O(n) / O(n log n) /
O(n²) comm). Concrete cost is now partly observable: the in-process **counting tier** emits
*modelled* deterministic byte/round/multiplication counts, and the **loopback network tier**
emits *measured* wall-clock + bytes-on-wire over a real TCP transport (both BUILT — §1.2);
the *netem-shaped* LAN/WAN wall-clock is the only piece still gated (needs CAP_NET_ADMIN → a
privileged host / the EC2 bead). The per-cell figures below remain qualitative classes
because the harness has not yet been swept across the full operator × configuration grid, not
because the harness is absent. Literature is cited by name + eprint, consolidated with the
parent record's source list (§11).

---

## 1. Ground truth — where the crate is TODAY (verified against source, 2026-06-15)

### 1.1 What has landed since the security-models record (its §0 is now partly STALE)

The parent record's "where the crate is today" (its §0) was written before a wave of work
landed. The current state is materially more advanced; this record is the up-to-date
ground truth. Verified by reading the files:

| Capability | Status in parent §0 | Status NOW | Evidence |
|---|---|---|---|
| 3-axis security descriptor | "the code flattens two axes" | **BUILT** (`AdversaryModel`/`OutputGuarantee`/`CorruptionThreshold`/`SecurityDescriptor`) | `backend.rs:111–340`; bead **sq-mq8q CLOSED** |
| Cleve as type-invariant | recommended (§1.3d) | **BUILT** — `OutputGuarantee::fairness/guaranteed_output` return `None` for `DishonestMajority`; private witness types un-mintable otherwise | `backend.rs:240–296` |
| Per-operator, per-N reporting | recommended (§1.3c) | **BUILT** — `operator_descriptor(OperatorClass)` + `MpcBackend::operator_security` | `shamir.rs:216–252` (`LinearAggregate`/`EqualityJoin`/`Comparison`) |
| Fail-closed selection registry | recommended (§1.3b) | **BUILT** — `SecurityRequirement` + registry | bead **sq-a6p1 CLOSED** |
| RS consistency-checked open (degree-t) | proposed (WI-1) | **BUILT** — `reconstruct_robust` (Berlekamp–Welch; correct ≤ e, detect ≥1, abort) wired into every prod reconstruct | `robust.rs`, `shamir.rs:281`; **sq-m34i / sq-uu0u CLOSED** |
| Consistency-checked open (degree-2t equality) | gap | **BUILT** — `reconstruct_degree` routes the 2t open through the same RS checker | `shamir.rs:511–516`; **sq-7q9i CLOSED** |
| **Oblivious shuffle** (Waksman/Beneš) | "ABSENT — the substrate gap" | **BUILT and SOUND** at the honest-majority Shamir layer (re-randomise + permute; no degree reduction needed) | `oblivious.rs` `WaksmanNetwork`/`shuffle`; **sq-18lk CLOSED** |
| **Oblivious sort** network (Batcher) | "ABSENT" | **substrate BUILT** (network + access-pattern obliviousness real); **secure key-comparison NOT built** — `sort_with_keys` uses *disclosed* keys; the secret comparator (`SimulatedSecretComparator`) is INSECURE and cfg-gated test-only | `oblivious.rs:761,927,955`; gated on **sq-rrz4** |
| Result-size protection / match-bit aggregation | gap (L1/L2) | **partially BUILT** — bead **sq-jnkm CLOSED** (oblivious result-size protection + match-bit aggregation for set-returning joins) | bead sq-jnkm |
| CSPRNG masking | gap | **BUILT** (ChaCha20; insecure PRNG cfg-gated) | `rng.rs`; **sq-1vt CLOSED** |

**Single most important correction:** the parent record names *"the entire oblivious
shuffle+sort foundation is ABSENT"* as the highest-leverage gap. **The shuffle is now BUILT
and sound; the sort network is BUILT.** What remains is the **secure comparator** that the
sort and `MIN`/`MAX`/range-`FILTER` depend on — and that is blocked on **degree reduction
(sq-dvuc, OPEN)**. The leverage point has moved one layer down: *multiplication chaining*
is now the keystone, not shuffle/sort.

### 1.2 The benchmark harness — what is BUILT and what is still gated (corrected 2026-06-15)

> **CORRECTION (this supersedes an earlier draft of this section).** An earlier version of
> this record claimed the crate was *"in-process simulation; no network — no `transport.rs`,
> no `netprofiles.rs`, no `metrics.rs`, no round/byte counter … bench files do not exist."*
> **That is FALSE.** The tier-1/2/3 benchmark harness the parent §5 specified has LANDED
> (beads **sq-sxm** + **sq-tg6b**, both CLOSED). The files exist, are wired into `lib.rs`, and
> are tested. The accurate current state:

- **Tier-1 — in-process counting tier — BUILT.** `metrics.rs` (`CommCounter`) records the
  *modelled* communication a real deployment would pay, derived deterministically from
  `(n, t, sizes)`: `field_elements_opened` / `field_elements_shared`, `multiplications`,
  `mult_rounds` / `open_rounds` (sequential upper bound) **and** `batched_rounds` (lower
  bound when independent ops collapse into one network round), plus `bytes_per_party`
  (8 bytes/field-element = `FIELD_BYTES`) and `total_bytes`. `bench.rs` is the matrix runner
  that drives the representative operators across the `{2,3,5,7}` × query-class grid and reads
  these counters off, emitting a stable JSON shape (`MatrixResults::to_json`, labelled
  *"network cost MODELLED not measured"*) plus a human table; `examples/mpc_bench_matrix.rs`
  prints both. Every cost cell co-runs the REAL primitive (`check_aggregate` /
  `check_hidden_join` / `check_shuffle` / `check_sort`) so a fast-but-wrong cell is caught.
  (Beads **sq-sxm** CLOSED.)
- **Tier-2 — real loopback network transport — BUILT.** `transport.rs` is a genuine
  multi-process transport: each party is its OWN OS process exchanging the actual protocol
  messages over a `127.0.0.1` TCP socket (`examples/mpc_party.rs` is the party child binary;
  `examples/mpc_net_bench.rs` is the coordinator/driver that spawns N child processes). It
  ships a hand-written length-prefixed wire codec (`Message` / `StepCode`), a byte-counting
  `Channel`, a star-topology `Coordinator`, and emits a `NetworkCell` carrying the
  **MEASURED** wall-clock, **MEASURED** bytes-sent/recv on the wire, and the round count —
  validated by the load-bearing test that the networked result equals the in-process result
  (`run_cell_networked` for the aggregate + hidden-join cells). So real bytes-on-wire and real
  round latency ARE observable in running code today (at LAN-ideal ~0 RTT on raw loopback).
  (Bead **sq-tg6b** CLOSED.)
- **Tier-3 — netem-shaped LAN/WAN — PROFILES BUILT, execution privilege-gated.**
  `netprofiles.rs` defines the canonical literature profiles as data — `NetemProfile::lan()`
  (1 Gbit/s, 2 ms RTT), `::wan()` (100 Mbit/s, 100 ms RTT, 5 ms jitter, 0.1 % loss),
  `::loopback()` (unshaped) — plus the exact `tc qdisc … netem` command generator that
  `scripts/mpc-netem.sh` applies. The *shaped* wall-clock is the ONLY genuinely-not-yet-here
  piece: applying a netem qdisc needs `CAP_NET_ADMIN` (root), unavailable unprivileged on the
  dev box, so the shaped runs are gated to a privileged host / the orphan-proof EC2 bench
  (bead **sq-hoaj**, the heavy-ceiling run). The harness NEVER fakes a shaped number — a
  Tier-3 figure only exists once `tc` actually ran. (Profiles part of **sq-tg6b** CLOSED;
  scale-out execution **sq-hoaj** OPEN.)

**What this means for the cost columns below:** the round/comm classes are still given
qualitatively because the harness has not been *swept* across the whole operator ×
configuration grid yet, not because no harness exists. The dominant real-world cost
(rounds × RTT, bytes on wire) IS now observable for the representative aggregate + hidden-join
cells on the loopback tier; the shaped LAN/WAN multiplier awaits the privileged/EC2 run.
- **One protocol family only: Shamir/BGW, honest-majority,** `t = ⌊(n−1)/2⌋`
  (`shamir.rs`). No replicated 3PC, no SPDZ/MASCOT, no GMW, no garbled circuits, no FSS/DPF.
- **No degree reduction (on `main`).** `mul_shares_raw` yields a degree-`2t` product and the
  crate **only ever opens it once** (the equality test). Any multiplication *chain* — secure
  comparison, products of match bits, conjunctive hidden-pattern joins, segmented
  group-agg scans, AVG — is **unbuilt on `main`** (`shamir.rs:455–469`; bead **sq-dvuc OPEN**).
  `degree_reduce` is **in-flight in PR #119, not yet merged**, so it is correctly absent from
  the `main` tree at the time of writing — `mul_shares_raw` is still single-product here. This
  is the load-bearing blocker for ~half the operator surface until #119 lands. *(This is the
  one current-state claim from the earlier draft that was accurate — degree reduction genuinely
  is not on `main` yet; only the transport/metrics/bench "do not exist" claims were wrong.)*
- **The hidden equi-join is still all-pairs** `O(|L|·|R|)` `secure_equal` (`join.rs:411,
  443`). ORQ-style O(n log n) sort-merge is **OPEN** (bead **sq-ujz8**).
- **No collaborative ZK proof; no in-circuit signature.** `proof.rs` is honest
  `NotYetImplemented` stubs.
- **Single-dealer randomness simulation.** A real federation needs PRSS / dealer-less VSS
  (bead **sq-yyro OPEN**); `dealer()` is a stand-in.

### 1.3 The two-regime split (load-bearing; from architecture convention #4)

Every operator has up to TWO realizations, and the cheap one is often crypto-free:
- **DISCLOSED regime** — operands are public global IRIs / disclosed multiset values. The
  operator is recomputed by the **verifier OUTSIDE the crypto** (no MPC at all). `DISTINCT`,
  `ORDER BY`, `LIMIT/OFFSET`, `COUNT/SUM/AVG/MIN/MAX`, and joins/UNION/OPTIONAL over
  disclosed values all collapse to this. `DisclosedKeyJoin` BUILT (`join.rs:119`).
- **HIDDEN regime** — operands stay secret-shared; the operator must run inside MPC. This is
  where the cost lives and where the matrix below is non-trivial.

The matrix evaluates the **HIDDEN regime** unless noted, because the disclosed regime is the
same trivial "verifier recomputes, free, leaks-everything-disclosed" cell for almost every
operator (§4 covers the leakage of choosing disclosed).

---

## 2. The SPARQL operator surface for MPC (what each operator fundamentally REQUIRES)

The MPC cost of an operator is a function of the **primitive class** it reduces to. The
classes, cheapest → most expensive in the secret-sharing model:

| Primitive class | What it needs | Round cost | sparq status |
|---|---|---|---|
| **P0 Linear** | local add / scale of shares | **free (0 rounds)** | BUILT (`add_shares`/`scale`/`run_secure`) |
| **P1 Single product** | one share-multiply + one open (no chaining) | 1 mult round + 1 open | BUILT (`mul_shares_raw` + `secure_equal`) |
| **P2 Equality-to-zero** | P1 specialised (`d=a−b`, mask `d·r`, open) | 1 mult + 1 open | BUILT (`secure_equal`, `join.rs:411`) |
| **P3 Oblivious shuffle** | re-randomise + permute by hidden permutation | O(n log n) swaps, O(log n) rounds (deployed) | **BUILT & sound** (`oblivious.rs`) |
| **P4 Mult chaining** | degree reduction after each product (BGW/DN/ATLAS) | +1 round / mult depth | **OPEN (sq-dvuc)** — the keystone blocker |
| **P5 Secure comparison** (`<`,`≤`,`>`) | bit-decomposition / edaBits / Rabbit / DGK — mult-depth >1 | O(log p) bit-ops or O(1)-round constant-round | **OPEN (sq-rrz4, blocked on sq-dvuc)** |
| **P6 Oblivious sort** (secret key) | sort network + P5 comparator | O(n log²n) gates, O(log²n) rounds | substrate BUILT; comparator OPEN (needs P5) |
| **P7 Oblivious data-dependent access** | DORAM (Duoram/Floram/3PDORAM) | O((κ+D) log N)/access | **OPEN** (not needed until data > RAM-in-MPC) |

Now, per SPARQL 1.1 algebra node, the **minimum primitive class** it requires in the hidden
regime (this is the "what it fundamentally requires" the directive asks for):

| SPARQL operator / algebra node | Min primitive in HIDDEN regime | Why |
|---|---|---|
| **BGP / triple-pattern match** (single pattern) | P2 (equality per bound term) + membership | match a hidden term against a committed set = equality-to-zero; multi-term = conjunction of equalities → needs **P4** (product of match bits) |
| **BGP / multi-pattern (conjunctive join)** | **P4 + P6** | self-join across patterns; obliviousness ⇒ sort-merge (P6) or all-pairs P2; conjunction of per-pattern bits needs mult-chaining (P4) |
| **JOIN (inner, hidden key)** | P2 (all-pairs) **or** P6 (sort-merge) **or** circuit-PSI | key equality; SOTA is sort-merge (P6) or circuit-PSI |
| **OPTIONAL / left-join** | P6 + P4 | sort-merge with a per-left-row "had-any-match" bit (a P4 OR-fold) controlling NULL padding |
| **UNION** | P3 (oblivious concat + padding) | concatenate two secret multisets without revealing which side a row came from = shuffle/pad |
| **MINUS / anti-join** | P6 + P4 | sort-merge, emit left rows whose match bit is 0 |
| **FILTER (=, ≠)** | P2 | equality-to-zero; `≠` is its complement |
| **FILTER (<, ≤, >, BETWEEN)** | **P5** | order comparison over a prime field; bit-decomposition / edaBits |
| **FILTER (regex, string fns)** | P5 + Boolean circuit | char-wise comparison → GMW/garbled Boolean; worst case for SS |
| **FILTER (IN, logical AND/OR/NOT)** | P4 | AND = product of bits (chaining), OR = De Morgan, NOT = `1−b` |
| **BIND / expression eval** | P0–P4 by op | `+`/`−`/`×const` free; `×var` needs P1/P4; comparisons need P5 |
| **COUNT / SUM** | **P0 (free)** | linear over shares — the genuine sweet spot |
| **AVG** | P0 + 1 division | SUM (free) ÷ disclosed/public count; secret-count division needs P4/P5 |
| **MIN / MAX** | P5 tournament **or** P6 + pick-extreme | comparison-bound; O(log n) rounds (tournament) or sort then take end |
| **GROUP_CONCAT** | P6 + oblivious string concat | order-dependent, string-typed; sort-on-key then concat |
| **GROUP BY (hidden key)** | **P6 + P4** | sort-on-key, then segmented prefix-sum gated by a secret "is-new-group" bit (P4 chain) |
| **DISTINCT / dedup** | P6 | oblivious sort + adjacent-equality scan + oblivious compaction |
| **ORDER BY** | P6 | oblivious sorting network (Batcher / bitonic / Waksman+key) |
| **LIMIT / OFFSET / top-k** | P6 (partial) or P5 selection | after an oblivious sort, reveal a padded prefix; top-k = oblivious selection |
| **Property paths** (`*`,`+`, arbitrary length) | **OPEN** (super-linear; structure leak) | transitive closure of a SECRET edge set; only BOUNDED length is tractable |
| **SERVICE / federation** | P0–P2 (global-IRI key) | the headline: global IRI is a *public* cross-holder id → disclosed-key join is crypto-free; only hidden values enter circuit-PSI/P2 |
| **Subquery (sub-SELECT)** | inherits inner | composes IFF inner result disclosed (recompute) or kept secret-shared (compose ORQ "with itself") |

**The structural takeaway:** the SPARQL surface partitions into three cost zones —
- **CHEAP (P0–P3, BUILT):** SUM/COUNT, single equality FILTER, inner equi-join (all-pairs),
  UNION (via shuffle), and — newly — anything reducible to oblivious shuffle.
- **MEDIUM (P4–P6, mostly OPEN):** everything needing mult-chaining or a secure comparator —
  range FILTER, MIN/MAX, GROUP BY, DISTINCT, ORDER BY, OPTIONAL/MINUS, multi-pattern BGP,
  AVG-by-secret-count. **All blocked behind the single sq-dvuc degree-reduction keystone.**
- **OUT-OF-REACH (P7 / unbounded):** property paths of unbounded length over a secret graph,
  and data-dependent private indexing (DORAM) — bounded-length / scoped-fragment only.

---

## 3. The configuration space (the cells we evaluate)

Three orthogonal security axes (parent §1.2) × N × network tier. Using the crate's now-BUILT
vocabulary (`backend.rs`):

- **AXIS-1 AdversaryModel:** `SemiHonest` | `Covert{ε}` | `Malicious`.
- **AXIS-2 OutputGuarantee:** `Abort{Selective|Unanimous|Identifiable}` | `Fairness` | `GuaranteedOutput` (GOD).
- **AXIS-3 CorruptionThreshold:** `DishonestMajority{t<n}` | `HonestMajority{n>2t}` (statistical GOD, broadcast) | `SuperHonestMajority{n>3t}` (perfect GOD, point-to-point).
- **N (parties):** {2, 3, small-odd 5/7/9, even 4/6/8, large}. Two honest-majority sub-regimes
  matter: **minimal** `n=2t+1` (cheapest; degree-`2t` open has ZERO RS redundancy →
  semi-honest-only on the equality path) vs **over-provisioned** `n≥2t+3` (robust equality).
- **Network tier:** **LAN** (≈1 Gbps / ≈1 ms RTT) | **WAN** (≈100–200 Mbps / 20–100 ms RTT,
  optional loss). Network tier *dictates protocol family*, not just speed: round-per-depth
  SS wins on LAN; constant-round garbled/BMR wins on WAN.

**Cleve (STOC'86) constrains the cross-product:** Fairness and GOD are IMPOSSIBLE without an
honest majority. So the entire `DishonestMajority × {Fairness, GOD}` slab is **IMPOSSIBLE**
for every operator — the crate already enforces this as a type-invariant (`backend.rs:266,
276`). We therefore evaluate the **meaningful** cells, the cross-product of:

> **{SemiHonest, Covert, Malicious} × {HonestMajority, DishonestMajority} × {LAN, WAN} × {minimal-N, over-provisioned-N, large-N}**, with GOD/Fairness only on the honest-majority side.

The matrix in §5 collapses this to the cells where the *answer differs*: most operators have
the same capability tier across N and network (they differ in *cost*, captured by the
round/comm class), so we report **(AdversaryModel × Threshold)** capability per operator and
fold N/network into the cost column and the §4 cost notes.

---

## 4. Per-operator capability + most-performant-protocol matrix (THE CORE)

**Legend.** Tier ∈ {BUILT, KNOWN, OPEN, IMPOSSIBLE}. SH = semi-honest, Cov = covert,
Mal = malicious. HM = honest-majority, DM = dishonest-majority. "Most performant" names the
SOTA protocol for that cell with its round/comm class. "sparq" = where the crate sits.

### 4.1 The CHEAP zone (BUILT or near-BUILT)

#### SUM / COUNT (linear aggregate) — the sweet spot
| Config | Tier | Most performant known protocol | sparq today |
|---|---|---|---|
| SH, HM, any N, LAN/WAN | **BUILT** | linear secret-sharing → **free local addition, 0 mult rounds, O(n) total comm for the single open** | `run_secure` cumulative sum (`shamir.rs`), differentially tested == plaintext sum |
| Mal, HM, over-provisioned N | **BUILT** | same + RS/Berlekamp–Welch checked open (correct ≤ `e=⌊(n−t−1)/2⌋`, detect, abort) | `reconstruct_robust` at degree-`t` (has redundancy for every valid HM `(n,t)`) |
| Mal, HM, **minimal N** | BUILT (detect/abort at even N; **SH-only at odd `n=2t+1`** because degree-`t` still has redundancy here — aggregate is fine; only the *equality* path loses it) | + IT-MAC for full malicious at the bytes-on-wire level (KNOWN: SPDZ-style MAC) | degree-`t` open is robust at all valid HM N (the no-redundancy hole is the **degree-2t** path, not this one) |
| any, **DM** | KNOWN | **SPDZ/MASCOT/Overdrive** additive + IT-MAC; preprocessing-heavy (Beaver triples + MACs) | **no backend** — registry refuses (fail-closed, sq-a6p1) |
| GOD, HM | KNOWN | Goyal–Song–Liu "GOD comes free in honest majority" (eprint 2020/189) | not built (robust reconstruct is per-open, not whole-protocol GOD) |

**Verdict: the £100k aggregate is privacy-optimal and CHEAP today** — hides per-holder
addends, leaks only the disclosed sum/boolean, 0 mult rounds. The only residual is the
threshold *comparison* `>£100k`, which is currently disclosed-and-verifier-recomputed (free,
leaks the boolean) or needs P5 to keep the sum itself secret (OPEN — see §4.2 comparison).

#### FILTER (=, ≠) / single-term BGP equality
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| SH, HM, any N | **BUILT** | equality-to-zero: `d=a−b`, mask `m=d·r`, open `m`; `m==0 ⇔ equal`. 1 mult + 1 open. Leaks only the match bit. | `secure_equal` (`join.rs:411`) |
| Mal, HM, over-provisioned N | KNOWN→partial | same with checked open; full malicious needs IT-MAC | `reconstruct_degree` checks the 2t open where redundancy exists |
| Mal, HM, **minimal N=2t+1** | **OPEN (the one real hole)** | **IT-MAC** on the degree-2t product (NOT RS redundancy — there is none at degree 2t when n=2t+1) | documented gap; seam **sq-6d6g**. Per coZK eprint 2025/1026 this is also a *confidentiality* hole, not just correctness |
| any, DM | KNOWN | SPDZ equality (MAC-checked) | no backend |

#### Inner JOIN (hidden key) / cross-credential hidden join
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| SH, HM, any N | **BUILT (naive)** | sparq runs **all-pairs** `O(\|L\|·\|R\|)` `secure_equal`. **SOTA is O(n log n)** ORQ sort-merge join-aggregation OR ~linear **circuit-PSI** (cuckoo+simple hashing; VOLE-PSI eprint 2021/266) | `HiddenValueJoin` all-pairs (`join.rs:443`); SOTA = bead **sq-ujz8 OPEN** |
| SH, HM, **disclosed global-IRI key** | **BUILT, crypto-free** | hash-join `O(\|L\|+\|R\|)` in cleartext — sparq's genuine lead (global IRIs as cross-holder join keys) | `DisclosedKeyJoin` (`join.rs:119`) |
| Mal, HM, N=3 | KNOWN | **ORQ 4-party Fantastic Four** sort-merge; relational SOTA | not built |
| Mal, DM | KNOWN | malicious circuit-PSI (VOLE-PSI, malicious, single equi-join only — does NOT compose into multi-pattern BGP) | no backend |
| any, multi-pattern BGP | **OPEN** | sort-merge composed per pattern (P6) needs the secure comparator + mult-chaining | blocked on sq-dvuc/sq-rrz4/sq-ujz8 |

**Cost note (N, network):** all-pairs join is the **cost center** at every config (the parent
record's ORQ anchor: even SOTA O(n log n) joins are minutes-to-tens-of-minutes on LAN, ×1.2–6.9
on WAN). The all-pairs `O(\|L\|·\|R\|)` opens are also a **round-count explosion** on a real
network (one open per pair). The harness now *measures* this rather than only modelling it: the
hidden-join cell runs over the real loopback transport (`transport.rs::run_cell_networked`,
the `MulAndOpen` step) so the bytes/rounds are observed, and `CommCounter`'s
`batched_rounds` lower bound records that the `\|L\|·\|R\|` equalities are mutually independent
(they collapse into ~one network round if batched) — the difference between the sequential
upper bound and the batched lower bound IS the round-count-explosion risk, made explicit. The
shaped WAN slowdown awaits the netem/EC2 run (`netprofiles.rs::wan()` + sq-hoaj).

#### UNION
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| SH/Mal, HM, hidden | **KNOWN (substrate BUILT)** | oblivious **concat + pad + shuffle** (P3) so a row's source side is hidden — the shuffle is BUILT and sound | shuffle BUILT (`oblivious.rs`); the UNION wrapper (concat+pad) not wired |
| disclosed | BUILT (crypto-free) | verifier recomputes union of disclosed multisets | parent convention #4 |

### 4.2 The MEDIUM zone (mostly OPEN — all gated behind degree reduction sq-dvuc)

#### FILTER (<, ≤, >) — secure comparison
| Config | Tier | Most performant known protocol | sparq |
|---|---|---|---|
| disclosed operands | **BUILT (crypto-free)** | verifier recomputes the comparison | `operator_descriptor(Comparison)` honestly reports "no in-crypto guarantee" (`shamir.rs:250`) |
| SH/Mal, HM, hidden | **OPEN in sparq / KNOWN in literature** | **edaBits** (Crypto'20) for mixed arith/Boolean; **Rabbit** (eprint 2021/119) ~constant-round less-than; **DGK / bit-decomposition** for the classic O(log p)-bit route; constant-round (DGK-style) wins on **WAN**, bit-decomposition is fine on **LAN** | **not built — blocked on degree reduction (sq-dvuc)**; tracked as bead **sq-rrz4** |

**This is the single most-requested missing primitive:** the £100k verdict's `>` (if the sum
is to stay secret), every range FILTER, MIN/MAX, GROUP BY's is-new-group bit, top-k. **The
trade-off** is round-complexity: bit-decomposition is O(log p) rounds (bad on WAN), Rabbit/DGK
are ~constant-round (WAN-friendly) but heavier per-round. sparq should target **edaBits** (it
amortises the bit-Beaver generation and is the modern SOTA for mixed circuits over both HM and
DM), once degree reduction (P4) lands.

#### MIN / MAX
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| disclosed | BUILT (verifier) | recompute | convention #4 |
| hidden | **OPEN** | comparison **tournament** (O(log n) rounds, n−1 comparisons) for a single extreme; or oblivious-**sort** then take the end (reuses the BUILT sort network once P5 exists) | needs P5 (sq-rrz4) → P4 (sq-dvuc) |

#### GROUP BY (hidden key) / GROUP_CONCAT / HAVING
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| disclosed key | **BUILT-able (verifier)** | recompute group-agg over disclosed keys | convention #4 (HAVING/GROUP-BY-over-hidden is *forbidden* in RQ1 by design) |
| hidden key | **OPEN** | **ORQ**: oblivious sort-on-key (P6) + segmented prefix-sum gated by a secret is-new-group bit (P4 chain) — O(n log n) work, O(log n) rounds | sort substrate BUILT; the segmented scan + is-new-group bit need P4/P5 (sq-dvuc/sq-rrz4) |

#### DISTINCT / ORDER BY / LIMIT (top-k)
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| disclosed | **BUILT (verifier)** | recompute over disclosed multiset (no-proof-of-revealed-properties) | convention #4 — the fast default |
| ORDER BY, **disclosed key, secret payload** | **BUILT** | Batcher sort network over secret-shared rows keyed by a **disclosed** sort key — already wired | `sort_with_keys` (`oblivious.rs:927`) |
| ORDER BY / DISTINCT, **hidden key** | **OPEN** | oblivious sort with a **secure** comparator (Batcher / bitonic O(n log²n), or Waksman O(n log n) + key); DISTINCT = sort + adjacent-equality (P2) scan + oblivious compaction | sort network BUILT; secure comparator OPEN (sq-rrz4) |
| LIMIT/top-k, hidden | **OPEN** | oblivious **selection** (partial sort) or sort + reveal padded prefix | needs P5/P6 |

#### OPTIONAL (left-join) / MINUS (anti-join)
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| disclosed | BUILT (verifier) | recompute | convention #4 |
| hidden | **OPEN** | **ORQ** outer/anti: same sort-merge, different gating bits + a per-left-row "had-any-match" OR-fold (P4) to drive NULL padding (OPTIONAL) / row retention (MINUS) | needs sort-merge join (sq-ujz8) + P4 (sq-dvuc) |

#### BIND / expression eval / arithmetic
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| `+`,`−`,`× const` on secrets | **BUILT** | linear (free) | `add_shares`/`scale`/`add_constant` |
| `× var` (single) | **BUILT** | one product | `mul_shares_raw` |
| `× var` (chained), `/`, comparisons | **OPEN** | degree reduction (P4) for products; secure division via Newton/Goldschmidt over shares (needs P4); comparisons P5 | sq-dvuc / sq-rrz4 |
| string ops, hashing, regex | **OPEN/KNOWN** | Boolean circuit via **GMW** (round-per-AND-depth) or **garbled/BMR** (constant-round, WAN winner) — the worst case for Shamir SS | wrong family for sparq's Shamir core; would need a Boolean backend |

#### AVG
| Config | Tier | Most performant | sparq |
|---|---|---|---|
| disclosed/public count | **near-BUILT** | secure SUM (free) ÷ public count (free scale by inverse) | SUM BUILT; division-by-public-constant is a free scale |
| secret count | **OPEN** | secure division (P4/P5) | needs degree reduction |

### 4.3 The OUT-OF-REACH zone

#### Property paths (`*`, `+`, arbitrary length)
| Config | Tier | Note |
|---|---|---|
| unbounded length, hidden graph | **OPEN / effectively IMPOSSIBLE at scale** | transitive closure of a SECRET edge set leaks structure (each fixpoint iteration reveals reachability growth) and is super-linear; GORAM does ego-centric traversal but is *confidentiality-only*, not correct/attested. **Scope to BOUNDED length only** (unroll to a fixed hop count), where it reduces to a fixed BGP chain (P4+P6). |

#### Data-dependent private indexing (any operator over a store larger than RAM-in-MPC)
| Config | Tier | Note |
|---|---|---|
| private random access | **OPEN (KNOWN protocols exist, none in sparq)** | **DORAM** (Duoram USENIX'23 2-/3-party; Floram; 3PDORAM) — O((κ+D) log N)/access. Only needed once data exceeds in-MPC RAM; Duoram's 2-/3-party specialisation would force a **backend split** conflicting with sparq's any-N Shamir commitment (parent open-Q 7). |

#### Function/query privacy (hiding which BGP)
| Config | Tier | Note |
|---|---|---|
| hide the query | **KNOWN but deliberately FORGONE** | PFE / universal circuit (Mohassel–Sadeghian eprint 2013/137): O(z log z) Boolean, up to O(z⁵) arith — almost never worth it. sparq deliberately publishes the query (verifier ISSUES it). The only residual query-side leak is the **join match structure** (§5 L2). |

### 4.4 The cost of going semi-honest → malicious, per primitive class (AXIS-1 lift)

The directive asks the cost of hardening each operator. Per primitive:

| Primitive | SH→Mal cost (HM) | SH→Mal cost (DM) |
|---|---|---|
| P0 linear / SUM | ~free — RS-checked open is the same Lagrange on clean input; cost only on tamper (BUILT) | SPDZ MAC-check before open (adds the preprocessing tax) |
| P2 equality | **IT-MAC** at minimal N (the one real hole, sq-6d6g); otherwise RS-checked (BUILT) | SPDZ MAC |
| P3 shuffle | malicious shuffle needs a proof-of-permutation (verifiable shuffle) — KNOWN | heavier |
| P4 mult / P5 comparison | Goyal–Song "malicious comes free in honest majority" (eprint 2020/134) — **~no online overhead at HM** | Beaver triples + MAC-check — preprocessing dominates |
| P6 sort | inherits comparator + checked opens | inherits |

**Headline:** at **honest majority**, malicious security is **close to free** (Goyal–Song /
Goyal–Song–Liu) for the arithmetic operators *once the primitives exist*; the cost is in the
checked-open machinery (BUILT for degree-`t`/`2t`-with-redundancy) plus an IT-MAC for the one
no-redundancy cell. At **dishonest majority** there is NO free lunch: every malicious cell
pays the SPDZ preprocessing tax (Beaver triples + MACs), and — critically — **no published
system delivers dishonest-majority-malicious correctness for SPARQL/graph query eval at all**
(parent §6.3). The registry correctly **refuses** that request rather than downgrading.

---

## 5. Most privacy-preserving handling per operator (leakage + how to minimise)

Reusing the parent's seven-channel leakage taxonomy (§4.2 there), resolved per operator. For
each operator we give the **leakiest (cheapest)** and the **most-private (costliest)** variant
and what each leaks.

| Operator | Leakiest variant (what leaks) | Most-private variant (residual leak) | sparq today |
|---|---|---|---|
| **SUM/COUNT** | disclosed sum (the scalar) | secret sum, only threshold boolean opened (leaks 1 bit) | near-frontier (BUILT); only residual is the malicious flip at minimal N |
| **FILTER =** | per-comparison match bit opened | aggregate match bits inside MPC, never open per-pair | per-pair open today (L2 below) |
| **FILTER <** | disclosed operand (full value) | secure comparison opening only the verdict bit | disclosed today (P5 OPEN) |
| **JOIN (hidden)** | disclosed-key join leaks keys, values, cardinality, **and which holder contributed each row** (L4) | hidden key + match-bit aggregation + **result-size padding** + **oblivious shuffle of matched rows, reveal padded prefix** | all-pairs leaks full **match graph** (L2 / fan-out); result-size protection landed (sq-jnkm) but not on the all-pairs path |
| **GROUP BY** | disclosed keys + group sizes | hidden key, padded per-group output (DP-noised group count) | hidden forbidden today (convention #4) |
| **ORDER BY / DISTINCT** | disclosed multiset | oblivious sort/dedup, padded output | disclosed (verifier) today |
| **OPTIONAL** | which left rows matched (the optional-presence bit) | obliviously padded NULLs so presence is hidden | OPEN |
| **Property path** | reachability structure of the secret graph | bounded-length only; even then leaks path-existence bits | scope to bounded |

**The four live leaks sparq must close (from parent §4.1, status updated):**
- **L1 result cardinality** — the all-pairs `HiddenValueJoin` emits exactly the true match
  count. **Mitigation BUILT for set-returning joins** via bead **sq-jnkm** (oblivious
  result-size protection + match-bit aggregation); **not yet wired into the all-pairs path** —
  the gap is connecting sq-jnkm's output path to `HiddenValueJoin`/the sort-merge join (sq-ujz8).
- **L2 per-pair match graph / fan-out** — each opened `m` in the all-pairs loop reveals the
  match bit at `(i,j)`; the set of matches IS the bipartite match graph → join-key
  multiplicity distribution leaks (a strong fingerprint of the hidden key distribution).
  **Mitigation:** aggregate match bits inside MPC (P4) + oblivious-shuffle (P3, BUILT) +
  reveal only a padded prefix — i.e. the sort-merge join (sq-ujz8) consuming the BUILT shuffle.
- **L3 input cardinalities** `|L|,|R|` — loop bounds are public; standard MPC assumption but
  the *number of credentials a holder has* can be sensitive. Mitigation: pad inputs to a
  public bound (cheap, exact) or a DP bound.
- **L4 source provenance / row-linkability** — disclosed payload columns + M4 v1's clear
  `pk_i` make the contributing source linkable. **Mitigation:** in-circuit BBS-key-set
  membership (unlinkable) — the "join nobody has built" (§6 / sq-bwwl), two research steps out.

**The cheapest high-leverage privacy upgrade** remains **differential-privacy result-size /
output-cardinality** (ShrinkWrap/SAQE/Doquet/Adore): pad to a *noisy* (ε,δ)-private
cardinality instead of the catastrophic full-obliviousness `|L|·|R|`. Bead **sq-shk5 OPEN**.
For the aggregate query the result is a fixed-size boolean so L1 is already closed there; the
moment a query returns a *set*, L1 is live. **Residual-leak honesty:** DP result-size leaks
the query exists + an (ε,δ)-noised size, and the budget *composes* over repeated queries (must
track an ε budget) — NOT information-theoretic.

---

## 6. Other desirable MPC properties, resolved to operator granularity

- **Fairness & GOD.** IMPOSSIBLE without honest majority (Cleve) — **enforced as a
  type-invariant** today (`backend.rs:266,276`). `HonestMajorityRobust` is **per-reconstruction**
  robustness (a form of GOD on one open), now correctly reported PER-OPERATOR via
  `operator_descriptor` — robust for the degree-`t` aggregate, semi-honest-only for the
  degree-`2t` equality at minimal N. Whole-protocol GOD (Goyal–Song–Liu, free at HM) is KNOWN,
  not built. Perfect-GOD (`SuperHonestMajority`, n>3t, BGW point-to-point) vs statistical-GOD
  (`HonestMajority`, n>2t, broadcast) is now **expressible** in the threshold enum; whether
  perfect-GOD is ever a *target* is parent open-Q 6 (the linear-aggregate use case likely makes
  statistical sufficient).
- **Identifiable abort (IA).** Real IA = honest parties AGREE on the cheater (Baum CRYPTO'20
  eprint 2020/767; pairwise-MAC IA CRYPTO'24 eprint 2023/1548). sparq has sound *detection* +
  sound *correction within budget* (Berlekamp–Welch) + **heuristic blame on the abort path**
  (can blame an honest party). The `AbortKind::Identifiable` variant exists in the enum
  (`backend.rs:186`) but true IA needs a broadcast channel + per-party authenticated
  transcripts (IT-MACs) the in-process sim lacks → IA should stay **deferred behind the
  dishonest-majority backend**, not half-implemented via RS blame.
- **Input certification / attested sources (guarantee C).** Per-operator, this is *orthogonal*
  to the operator's arithmetic — it is a property of the *inputs* each holder feeds in. The
  attested-input pillar is **Dutta et al. (eprint 2022/1648)**: authenticated-input MPC, HM
  LSSS only. The M4 v1 path (verifier-side attestation, Artemis commit-and-prove anchor)
  attaches it to *any* operator's output without touching the operator (parent M4 doc §2).
  The full in-circuit unlinkable version ("the join nobody has built", sq-bwwl) is the only
  thing that makes the source-set membership hidden — two research steps out, audit-gated.
- **Composability / UC.** Honest majority enjoys UC without setup (Canetti) — an argument FOR
  the default. The danger operator is **equality** (and any operator that opens a value
  mid-computation): `secure_equal` OPENS a masked product mid-pipeline; naive sequential
  composition does not justify this, and per coZK eprint 2025/1026 proving/computing on an
  inconsistent witness can LEAK honest inputs. A UC/composition design record is itself a gap
  (see §7).
- **Robustness vs network.** Round-per-depth Shamir (sparq's family) degrades on WAN; the
  comparison/sort operators (O(log p) / O(log²n) rounds) are the WAN pain points. The
  constant-round answer (Rabbit/DGK for comparison; BMR garbled for deep Boolean) is a
  *different family* — honestly scoped out of v1 (LAN-first).
- **Preprocessing / offline-online.** HM Shamir needs NONE for P0–P3 (the BUILT operators).
  P4/P5 (degree reduction, comparison) and any DM/SPDZ backend introduce an input-independent
  offline phase (double-sharings / Beaver triples + MACs). `BackendInfo` has **no
  `requires_preprocessing` cost field** — a federation cannot yet budget the
  usually-hidden-dominant cost (the Ozdemir-Boneh / ORQ lesson). Gap below.
- **Post-quantum.** The SS core is PQ for free (information-theoretic Shamir over `F_p`); RS
  integrity is PQ; ChaCha20 masking is PQ-fine. PQ risk is at the boundaries: a DM SPDZ
  backend needs PQ-safe OT/SHE; the **attestation/collaborative-proof layer** is the PQ risk
  (EdDSA/BBS+ + Pedersen are pre-quantum; no PQ collaborative-zkSNARK exists). Honest claim:
  **PQ confidentiality + integrity in the MPC core TODAY; PQ attestation NOT today.**

---

## 7. The honest envelope, resolved to operator granularity

Restating the parent §6.3 verdict, now per operator and reflecting the post-sq-18lk/sq-mq8q/
sq-a6p1/sq-jnkm state:

**DEPLOYABLE TODAY in sparq (honest-majority, cooperating holders, LAN, ≤10³–10⁴ triples/party):**
- SUM/COUNT aggregate — **privacy-optimal, cheap, malicious-detect at over-provisioned N.**
- FILTER `=`/`≠` (single equality) — **BUILT** (semi-honest; malicious-detect except the
  minimal-N degree-2t hole that needs an IT-MAC).
- Inner equi-join over **disclosed global-IRI keys** — **BUILT, crypto-free** (sparq's lead).
- Inner equi-join over **hidden keys** — **BUILT but naive** all-pairs `O(|L|·|R|)` (correct;
  not SOTA cost; leaks the match graph L2).
- ORDER BY over **disclosed sort key** with secret payload — **BUILT** (Batcher network).
- Oblivious **shuffle** (and therefore the substrate for UNION, result-size protection,
  match-bit aggregation) — **BUILT and sound.**
- The disclosed regime for DISTINCT/ORDER BY/LIMIT/COUNT/SUM/AVG/MIN/MAX/joins/UNION/OPTIONAL
  (verifier recomputes outside the crypto) — **the fast default** (convention #4).

**ASPIRATIONAL (KNOWN protocols exist; NOT in sparq; the medium zone, all behind sq-dvuc):**
- Range FILTER, MIN/MAX, GROUP BY (hidden key), DISTINCT/ORDER BY (hidden key),
  OPTIONAL/MINUS (hidden), AVG-by-secret-count, multi-pattern BGP, top-k — every one needs
  degree reduction (P4) and/or a secure comparator (P5). **None is research-novel; all are
  standard once the keystone lands.**
- O(n log n) sort-merge join (replace all-pairs) — KNOWN (ORQ), bead sq-ujz8.
- DP result-size protection — KNOWN (ShrinkWrap), bead sq-shk5.

**RESEARCH-NOVEL / OPEN (budget as risk, never "seconds"):**
- In-circuit unlinkable attested-source membership ("the join nobody has built", sq-bwwl) —
  two steps out, audit-gated.
- Dishonest-majority-malicious correctness for *any* SPARQL operator — zero published instances.
- The full composition (coZK ⊕ malicious DM MPC ⊕ oblivious BGP joins ⊕ attested inputs ⊕ WAN)
  — zero performance data points.

**IMPOSSIBLE:**
- Fairness / GOD under dishonest majority (Cleve) — enforced.
- Unbounded property paths over a secret graph at scale — scope to bounded length.

---

## 8. Gap analysis → sequenced roadmap (cross-referenced to beads)

Ordered by leverage. Each gap names whether a bead already covers it.

1. **Degree reduction (BGW/DN/ATLAS) — `degree_reduce(shares_2t) → shares_t`.** THE keystone:
   unblocks P4–P6 → range FILTER, MIN/MAX, GROUP BY, DISTINCT/ORDER-BY (hidden), OPTIONAL,
   AVG-secret-count, multi-pattern BGP, and the secure sort comparator — **~half the operator
   surface in one primitive.** Bead **sq-dvuc (OPEN)** ✅ tracked.
2. **Secure comparison (edaBits / Rabbit) opening only the verdict bit.** The £100k `>` with a
   secret sum, every range FILTER, the secure sort comparator. Blocked on (1). Bead **sq-rrz4
   (OPEN)** ✅ tracked.
3. **ORQ-style O(n log n) sort-merge join-aggregation** (replace all-pairs `HiddenValueJoin`;
   consume the BUILT shuffle; emit shuffled padded prefix → closes L1/L2 on the join path).
   Bead **sq-ujz8 (OPEN)** ✅ tracked. **NOT-yet-beaded sub-gap:** wiring the CLOSED sq-jnkm
   result-size/match-bit aggregation path into the join operator (it landed standalone). →
   **FILED below.**
4. **DP result-size / output-cardinality + ε-budget.** Cheapest privacy win for any
   set-returning query. Bead **sq-shk5 (OPEN)** ✅ tracked.
5. **IT-MAC for the degree-2t equality open at minimal N=2t+1.** Promotes `secure_equal` from
   semi-honest-only to detect/abort at minimal N; also closes the coZK-2025/1026 confidentiality
   interaction. Seam noted in **sq-6d6g (OPEN, P3)** as a *deferred-seam doc*, but the actual
   IT-MAC *implementation* for the equality path is **not its own bead.** → **FILED below.**
6. **Distributed randomness (PRSS / dealer-less VSS).** Replace the single-dealer simulation so
   masks/correlated randomness are jointly generated — prerequisite for any real federation and
   for P4/P5 correlated randomness. Bead **sq-yyro (OPEN)** ✅ tracked.
7. **Network transport + round/byte instrumentation (tier-1 modelled, tier-2/3 real).** Make
   the round-count/comm cost — the dominant real-world cost — observable; prerequisite for ANY
   per-config performance verdict. **DONE (CLOSED):** tier-1 modelled counters
   (`metrics.rs::CommCounter`) + the matrix runner (`bench.rs`) landed as **sq-sxm**, and the
   real loopback multi-process transport (`transport.rs`) + the tc/netem LAN/WAN profiles
   (`netprofiles.rs` + `scripts/mpc-netem.sh`) landed as **sq-tg6b** — both CLOSED. The only
   remaining slice is *executing* the netem-shaped LAN/WAN runs at scale, which needs
   CAP_NET_ADMIN → tracked as **sq-hoaj (OPEN, P3)** (the orphan-proof EC2 ceiling run). The
   bead **sq-5gnv**, which re-filed "build transport + instrumentation", is therefore
   **REDUNDANT** and was closed pointing at sq-sxm/sq-tg6b/sq-hoaj. *(The only network-tier
   work genuinely not yet built is oblivious shuffle/sort OVER the transport — a noted
   sq-tg6b follow-up — and the scale-out itself, sq-hoaj.)*
8. **Bounded-length property-path operator** (unroll to fixed hop count → fixed BGP chain).
   The only tractable slice of property paths. **Not beaded.** → **FILED below.**
9. **WAN-tier protocol selection (constant-round comparison; BMR for Boolean/regex).** The
   round-per-depth family is wrong for WAN; needs a constant-round comparison and a Boolean
   backend for string ops. **Not beaded** (parent open-Q 2). → **FILED below.**
10. **Composition / UC design record** (the equality mid-pipeline open; coZK 2025/1026 witness
    validation). **Not beaded.** → **FILED below.**
11. **`BackendInfo.requires_preprocessing` cost field** so federations can budget the
    usually-hidden offline cost. **Not beaded** (small, but a real honesty gap). → **FILED below.**
12. **In-circuit unlinkable attested-source join** ("the join nobody has built"). Bead
    **sq-bwwl (OPEN)** ✅ tracked; gated on RQ1 remediation + a fresh coZK audit + the
    single-prover in-circuit sig gadget.

Already-CLOSED (do not re-file): sq-18lk (shuffle/sort substrate), sq-mq8q (3-axis descriptor),
sq-a6p1 (selection registry), sq-uu0u/sq-m34i/sq-7q9i (robust reconstruct), sq-jnkm (result-size
protection standalone), sq-1vt (CSPRNG), **sq-sxm (tier-1 modelled counters + matrix runner —
`metrics.rs`/`bench.rs`)**, **sq-tg6b (tier-2 loopback transport + tier-3 netem profiles —
`transport.rs`/`netprofiles.rs`/`scripts/mpc-netem.sh`)**.

Beads filed by this record for the genuinely-open sub-gaps (verified against source 2026-06-15):
sq-y32f (#3 wire sq-jnkm into the join), sq-km34 (#5 IT-MAC degree-2t at minimal N), sq-py8h
(#8 bounded property paths), sq-38zk (#9 WAN constant-round comparison + Boolean backend),
sq-aaop (#10 composition/UC record), sq-4i39 (#11 `BackendInfo.requires_preprocessing` field).
The earlier sq-5gnv (#7 "build transport + instrumentation") was filed on a mistaken premise
and has been **CLOSED as redundant** — that work already shipped (sq-sxm + sq-tg6b).

---

## 9. Headline findings (executive summary)

1. **CHEAP, deployable today:** SUM/COUNT (free, 0 mult rounds, privacy-optimal — the £100k
   case is near-frontier), single-equality FILTER, disclosed-global-IRI join (crypto-free —
   sparq's lead), and — newly landed — a **sound oblivious shuffle** plus the sort network and
   3-axis configurable security with Cleve as a type-invariant.
2. **EXPENSIVE / aspirational:** the hidden equi-join is the cost center (all-pairs today,
   O(n log n) ORQ sort-merge is the target; minutes-to-tens-of-minutes even at SOTA, ×1.2–6.9
   on WAN). The entire MEDIUM operator zone — range FILTER, MIN/MAX, GROUP BY, DISTINCT/ORDER
   BY (hidden), OPTIONAL, AVG-secret-count, multi-pattern BGP — is **all blocked behind ONE
   primitive: degree reduction (sq-dvuc) → secure comparison (sq-rrz4).** That is the
   highest-leverage next build; none of it is research-novel.
3. **OUT OF REACH:** dishonest-majority-malicious correctness for *any* SPARQL operator (zero
   published instances — the registry rightly refuses it); unbounded property paths over a
   secret graph; in-circuit *unlinkable* attested-source membership (two research steps out).
   Fairness/GOD under dishonest majority is **impossible** (Cleve) and is enforced in the type
   system.
4. **The leverage point has MOVED:** the prior record's "oblivious shuffle+sort is the #1 gap"
   is **resolved** (sq-18lk closed). The new keystone is **multiplication chaining**; building
   it unlocks roughly half the operator surface at once.

---

## 10. Open questions carried forward (operator-specific additions to the parent's)

1. Does the sort-merge join (sq-ujz8) consuming the BUILT shuffle + the CLOSED sq-jnkm
   result-size path fully close L1/L2 for the hidden join, or is DP padding (sq-shk5) still
   needed on top for the fan-out leak?
2. For the £100k verdict specifically: is disclosing the sum and recomputing `>` outside the
   crypto acceptable (cheap, leaks the sum), or must the sum stay secret and only the boolean
   open (needs sq-dvuc + sq-rrz4)? This decides whether the keystone build is on the *critical
   path* for the headline use case or only for general queries.
3. Can Senate-style circuit DECOMPOSITION (sub-circuits on party-subsets) be applied to
   federated-SPARQL source-combination planning to avoid all-N participation per operator (the
   best unexploited N-scaling lever)? — carried from parent open-Q 4.
4. (parent open-Qs 1–3, 5–7 still stand: dishonest-majority-among-holders necessity; LAN vs
   WAN v1 scope; multi-pattern BGP fragment; source-unlinkability requirement;
   perfect-vs-statistical GOD; DORAM backend-split.)

---

## 11. Sources (delta over the parent records — see those for the full list)

This record cites the same corpus as
[`mpc-security-models-and-benchmarks.md`](./mpc-security-models-and-benchmarks.md) §9 and
[`mpc-zkp-research-and-architecture.md`](./mpc-zkp-research-and-architecture.md) §3. The
load-bearing per-cell sources: Cleve (STOC'86, impossibility of fairness w/o honest majority);
BGW; Damgård–Nielsen DN07 (CRYPTO'07) + ATLAS (eprint 2021/833) (degree reduction);
Goyal–Song malicious-free (eprint 2020/134) + Goyal–Song–Liu GOD-free (eprint 2020/189);
edaBits (Crypto'20) + Rabbit (eprint 2021/119) + DGK (secure comparison); Hamada oblivious
radix sort (eprint 2014/121) + Waks-On/Waks-Off (eprint 2023/1236) (shuffle/sort);
Pinkas circuit-PSI (EUROCRYPT'18) + VOLE-PSI (eprint 2021/266); ORQ (SOSP'25, eprint 2025/1657)
+ Secrecy (NSDI'23) + Senate (USENIX'21) + Conclave (EuroSys'19) (relational MPC);
ShrinkWrap (arXiv 1810.01816) + SAQE (PVLDB'20) + Doquet/Adore (DP obliviousness);
Duoram (USENIX'23) + Floram (DORAM); SPDZ (CRYPTO'12) + MASCOT (eprint 2016/505) + Overdrive
(eprint 2017/1230) (dishonest-majority); Mohassel–Sadeghian PFE (eprint 2013/137);
Dutta authenticated-input MPC (eprint 2022/1648) + Artemis (arXiv 2409.12055) (attestation);
coZK soundness pitfalls (CRYPTO'25, eprint 2025/1026); Canetti UC (FOCS'01); GORAM (PVLDB'25)
(confidentiality-only graph traversal).

**In-repo ground truth (verified 2026-06-15):** `crates/sparq-mpc/src/{backend,shamir,robust,
join,oblivious,oblivious_join,proof,holder,partial,field,rng,metrics,bench,transport,
netprofiles}.rs` + `crates/sparq-mpc/examples/{mpc_bench_matrix,mpc_net_bench,mpc_party}.rs`
+ `scripts/mpc-netem.sh` — specifically `backend.rs:111–340` (3-axis descriptor + Cleve
invariant), `shamir.rs:216–252` (`operator_descriptor`), `shamir.rs:455–516`
(`mul_shares_raw` single-product + `reconstruct_degree`; **no `degree_reduce` on `main` — it
is in-flight in PR #119**), `oblivious.rs` (`WaksmanNetwork`/`shuffle` sound;
`SortingNetwork`/`sort_with_keys`; `SimulatedSecretComparator` INSECURE cfg-gated),
`join.rs:119,382,411,443` (`DisclosedKeyJoin` / `HiddenValueJoin` all-pairs).
**Benchmark harness (the §1.2 correction):** `metrics.rs` (`CommCounter` — byte/round/mult
counters, tier-1) + `bench.rs` (`run_matrix`/`MatrixResults`, the model×N×query matrix runner)
landed via **sq-sxm**; `transport.rs` (real loopback multi-process TCP transport — `Message`/
`Channel`/`Coordinator`/`NetworkCell`/`run_cell_networked`) + `netprofiles.rs`
(`NetemProfile::lan/wan/loopback` + `tc` generator) landed via **sq-tg6b** (both CLOSED).
Beads: sq-pwr / sq-0jsc (epics), sq-dvuc / sq-rrz4 / sq-ujz8 / sq-shk5 / sq-yyro / sq-bwwl /
sq-6d6g (OPEN gaps), sq-18lk / sq-mq8q / sq-a6p1 / sq-uu0u / sq-m34i / sq-7q9i / sq-jnkm /
sq-1vt / **sq-sxm / sq-tg6b** (CLOSED), sq-hoaj (OPEN — netem/EC2 scale-out).
