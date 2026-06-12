# ZKP query-correctness proofs for sparq — design plan (OPTIONAL module)

Status: **design for review — nothing implemented, no code touched.**
Author: research agent, 2026-06-12. Reviewer: Jesse Wright.
Inputs: `research/zkp-noir-context.md` (reconstruction of the sparql_noir /
sparql_noir_modular line), sparq source (`sparq-core`, `sparq-serve`,
`sparq-reason`, `sparq-solid`), `research/concurrent-serving.md`,
`research/solid-access-control-design.md`, the zkp-sparql-workspace research
notes (reused, not re-derived), and fresh web research (June 2026). Every
uncited number is marked **[judgement]**. Open questions for Jesse are
numbered §9 and referenced inline as (Q*n*).

---

## 1. Problem statement and threat models

sparq is a read-optimised SPARQL engine deployed (among other targets) as the
storage/query tier of a Solid pod server (`sparq-solid`,
concurrent-serving.md §2.10: named graph per resource, WAC/ACP gating, the
prod-solid-server contract). The question: can sparq optionally emit a
**cryptographic proof that a query result is correct with respect to a
committed dataset state**, and what is the cheapest sound way to do it?

"Correct" = the disclosed result is exactly `eval(Q, D_g)` under the
Pérez–Arenas–Gutiérrez algebra, where `D_g` is the dataset at a specific
published generation `g`, restricted to the requester's visibility scope.
Soundness AND completeness — not just "these rows exist" but "no row is
missing" (this is where almost all prior art stops; Braun/Wright/Käfer
ESWC 2026 explicitly proves soundness *only* [BWK26]).

### Threat-model options

- **T1 — untrusted server proving to client (integrity + scoped privacy).**
  The Solid client does not trust the pod server (or a replica/CDN/cache in
  front of it). The server proves the answer is sound and complete w.r.t. a
  commitment whose root the client trusts (signed by the pod owner, or pinned
  from a previous interaction). "ZK" is load-bearing even here: the proof must
  not leak triples outside the requester's ACL scope — completeness must be
  proven *relative to the visible graph set* without revealing the invisible
  remainder. This is the vSQL/IntegriDB/Proof-of-SQL setting [VSQL17]
  [INTEGRIDB15] [SXT] transplanted to RDF, plus access-control scoping that
  none of those systems have.

- **T2 — client-side selective disclosure over signed credentials (Jesse's
  original DPhil RQ1 setting).** The *holder* proves to a *verifier* that a
  query over credentials signed by third-party issuers yields a disclosed
  result, revealing nothing else. This is exactly what `sparql_noir` and
  `sparql_noir_modular` already do (zkp-noir-context.md §c.1).

**Which fits sparq-in-Solid: T1 is the native fit; T2 is served by the same
commitment layer for free.** sparq is a server; its writer/generation model
produces exactly the versioned commitments T1 needs. But the two models share
the entire mechanism below the signature: if pod roots are per-named-graph
Merkle commitments signed by the pod owner, then (a) the server can run T1
proofs against them, and (b) a client can export a pod (or subset) plus its
signed root and run Jesse's existing T2 holder-side stack *unchanged* — the
pod root plays the role of the VC `datasetCommit`. Jesse's past work therefore
serves T2 directly and supplies the proving machinery T1 reuses. Recommended
priority: design the commitment layer to serve both, implement T1 first
(it exercises sparq), keep T2 export as a cheap by-product. (Q1, Q3)

Out of scope for this plan: proving *updates* were authorised (that is the
Solid auth layer's job), MPC/federation (DPhil RQ2), and proof of
*non-tampering of the server binary* (zkVM territory; baseline only).

---

## 2. Mapping sparq's structures onto commitments

### 2.1 Dict ids vs term hashes — the committed-dictionary bridge

Jesse's circuits encode terms as `Enc_t(term) = h_2(type_code, h_s(value))`
with `h_s` = Blake3 (off-circuit) and `h_2` = Pedersen/Poseidon2
(sparql_noir `spec/encoding.md`). sparq stores `u32` dict-local ids
(`sparq-core/src/dict.rs`): dense, insertion-ordered in the current code
(the ARCHITECTURE blueprint moves to lexicographic-rank ids in M4), with
**inline ids** — canonical non-negative `xsd:integer` ≤ 2³⁰−1 encoded as
`id = 2³¹ + value`, no dictionary entry.

Ids are meaningless outside one store instance, so a commitment over raw id
triples proves nothing to an external verifier. The bridge is a **committed
dictionary**: a second Merkle/vector commitment `DictRoot_g` over the mapping
`id → Enc_t(term)` (leaf `i` = Poseidon2(type_code, blake3_field(term_i))).
Then a committed triple leaf can be `h_4` over *ids*, and any disclosed
binding carries a dict-opening proving `id ↔ term`. Two design points:

- **Inline ids are self-certifying.** `is_inline(id)` and
  `value = id − 2³¹` are pure arithmetic — in-circuit this is a range check +
  subtraction, ~tens of gates **[judgement]**, no dict opening at all. Numeric
  FILTER/range predicates over inline ids run entirely in id space in-circuit,
  mirroring exactly the engine's own optimisation. This is a genuine sparq
  advantage over the term-hash-only encoding: `filter_lt` measured 2,925 gates
  in the modular bench (u64 range-check dominated, HANDOFF-WAVE17); over a
  30-bit inline id the comparison shrinks substantially **[judgement; needs
  measurement]**.
- **Id stability.** Insertion-order ids never get reassigned (id 0 sentinel,
  no reclamation in the current dict), so `DictRoot` grows append-only —
  cheap incremental maintenance (§6.3). If M4 moves to lexicographic ids,
  global re-ranking on rebuild invalidates `DictRoot` wholesale; the ZK
  module should treat dict commitment epochs as tied to index rebuilds. (Q4)
- **Blank nodes / canonicalization.** Jesse's pipeline canonicalises with
  RDFC10 before hashing; sparq's dict labels blank nodes store-locally. For
  T1 (server proving against its own committed state) store-local labels are
  fine — the commitment *defines* the dataset. For T2 export / cross-store
  comparability, RDFC10 canonical labels must be computed at export time.
  Flagged, not solved here. (Q5)

### 2.2 Sorted permutations vs Merkle leaf ordering

sparq's SPO permutation is already a **sorted sequence of triples** — exactly
the "leaf-hash sorted" commitment shape Jesse's modularity note names as the
first tree variant (zkp-noir-context.md §c.4). Commit:

```
leaf_i  = h_4(g_id, s_id, p_id, o_id)   for the i-th quad in G-SPO order
Root_g  = Poseidon2-Merkle(leaves)
```

Sorted-by-id leaves buy three things his work already wants:

1. **Non-membership = adjacency**: prove leaves `i, i+1` are adjacent and the
   absent triple falls strictly between them in sort order — the
   `bgp_nonmember_prefix3` sentinel design lands naturally, which unlocks
   NOT EXISTS / MINUS / completeness proofs (§4, architecture B).
2. **Range proofs**: a sorted contiguous run of leaves `[lo, hi]` *is* the
   complete answer to a bound-prefix triple pattern — completeness of a scan
   is two adjacency proofs (boundaries) + the run, not per-row work.
3. **The engine's scan output order matches commitment order**, so the query
   trace (§4) indexes leaves directly.

Caveat: sorted-by-*id* order is only canonical given the dict; with
insertion-order ids the leaf order is store-specific. That is acceptable for
T1 and for T2-with-exported-dict; it is *not* the RDFC10 sorted-term-hash
order of sparql_noir. Bridging choice (commit over id-triples + DictRoot, vs
re-hash terms per leaf at export) is a real fork: id-leaves make commitment
maintenance cheap and witness generation free; term-hash leaves make the
commitment store-independent but cost a Blake3 per term per rebuild. (Q4)

### 2.3 Generation ring = commitment versioning

`sparq-serve` publishes immutable `Generation`s through an ArcSwap ring; a
single sequenced writer group-commits batches (default 3 ms / 256 updates)
into one generation. **A published generation is a natural commitment epoch**:
extend `Generation` (stage 2, §7) with `{ Root_g, DictRoot_g }`. Properties
that fall out for free:

- Proofs are pinned to a generation number — the exact freshness/consistency
  token the ring already gives readers. A client can demand "prove against
  generation ≥ N" (fork/rollback detection requires the owner-signature or a
  transparency log over roots; out of scope, noted).
- The group-commit batch is the unit of incremental commitment maintenance:
  `b` changed quads ⇒ `O(b · log N)` Poseidon2 evaluations *off-circuit*
  (§6.3), amortised exactly like the rest of the write path.

### 2.4 Per-pod epochs = scoped commitments for access control

Pods are named graphs (`PodId` = graph IRI, `sparq-serve/src/epoch.rs`);
ACL evaluation yields an **accessible graph set per (session, mode)**
(`AuthIndex::accessible`, solid-access-control-design.md). Commit per pod:

```
PodRoot_{p,e}  = Merkle over pod p's quads at pod-epoch e
Root_g         = Merkle over sorted (pod_id_hash, PodRoot) pairs
```

- The per-pod epoch vector tells the maintainer *which* pod subtrees to
  recompute — unrelated writes never touch another pod's root, mirroring the
  cache-invalidation design.
- **ACL-scoped proofs**: a T1 proof for a session quantifies over exactly the
  pods in its visibility scope — the proof discloses the set of PodRoots used
  (or a commitment to it) and proves completeness within them, leaking
  nothing about invisible pods. This is the clean answer to "complete w.r.t.
  what?" under access control, and per Jesse's principle the *scope itself*
  is disclosed data, checked verifier-side, not ZK-proven.
- Pod-owner signatures attach here: owner signs `(pod_iri, e, PodRoot)`.
  That single object serves T1 (client checks owner sig, not server) and T2
  (exported pod = signed credential). Signature scheme stays modular per the
  modularity memo (Schnorr default; BBS+/SD-JWT-VC/ML-DSA as modules). (Q6)

---

## 3. State of the art (web research, June 2026 — full citations §10)

**Verifiable SQL/DB.** vSQL [VSQL17]: interactive proofs + polynomial
delegation; TPC-H Q6 prover 3,851 s vs 0.67–4.16 s plaintext (~10³×
overhead); not ZK in the published version. IntegriDB [INTEGRIDB15]: ADS
(m² authenticated interval trees), setup 25,272 s and 1.85 GB ADS for a
30 MB table — the O(m²n) blow-up is disqualifying for RDF (three "columns"
but the join structure is the whole workload). ZKSQL [ZKSQL23]: VOLE-based
interactive ZK, designated-verifier, set-equality operator proofs — minutes
per TPC-H query at 60–240 k rows; interactivity and designated-verifier
clash with Solid's offline-verifiability needs. PoneglyphDB [PONE25]:
non-interactive Halo2 per-operator circuits + recursion, no trusted setup;
concedes **full recommitment on every update** — the gap sparq's
generation/epoch model is positioned to close. FalconDB [FALCON20]:
ADS + blockchain; proof generation "seconds to hours", decoupled from
answering. Space and Time "Proof of SQL" [SXT]: production Rust prover over
per-table Dory/HyperKZG commitments, vendor-reported >1 M rows < 1 s on a
GPU — evidence that commit-then-prove SQL at interactive speed is reachable
with sumcheck-style provers and GPU help; closed query fragment, vendor
numbers. Reef [REEF24]: "commit once, prove many predicates" over committed
documents via Nova folding + lookups — the design pattern for per-row
predicate streams.

**RDF/SPARQL-specific.** Confirmed near-empty at algebra level: term-level
selective disclosure (Yamamoto et al. rdf-proofs [RDFPROOFS]; Braun & Käfer
ESWC 2025 [BK25]); VeriDKG [VERIDKG24] is verifiable-but-not-ZK SPARQL over a
blockchain-maintained Merkle prefix trie (the closest ADS analogue);
Braun/Wright/Käfer ESWC 2026 proves *soundness only* of results over
selectively disclosed datasets [BWK26]; and the zkSPARQL line (Wright et al.,
ISWC 2026 submission, zksparql.org) — Jesse's own work — is the only
algebra-level soundness+completeness system found. The gap claim in his
paper notes survives adversarial search. **Implication: no external system
to adopt; the design space is his two architectures plus the primitives
below.**

**Primitives (numbers used in §5 cost models).**
- Poseidon2 permutation = **74 UltraHonk gates** (pinned constant in
  Barretenberg master [BBGATES]); Blake3 compression 2,159; SHA-256 6,703;
  ECDSA-secp256k1 verify 42,838. Poseidon2-in-circuit is settled — ~30×
  cheaper than Blake3.
- UltraHonk recursive verification of one UltraHonk proof inside an Ultra
  circuit = **~682 k gates** [BBGATES]; inside a Mega/Goblin circuit only
  **11,848 gates**. bb CLI exposes `--scheme chonk` (ClientIVC: HyperNova-
  style folding + Goblin) but non-Aztec usability is **unverified**
  [AZTEC-ROADMAP] [BBCLI].
- Lookup arguments: plookup/logUp are Ω(table) per proof; cq has
  table-independent O(k log k) proving but needs an SRS ≥ N (infeasible past
  ~2²⁸) and has no Noir/bb implementation; Lasso's sublinearity needs table
  structure an arbitrary triple set lacks [CQ22] [LOGUP22] [LASSO23]
  [LOOKUPSOK25]. **Break-even vs Merkle paths ≈ k ≳ 4–5 k lookups per proof
  at N = 10⁸ [judgement, derived from cited constants]** — Merkle wins for
  realistic per-query row counts.
- Prover throughput: ~50 k gates/s native laptop (order-of-magnitude,
  [NOIRBENCH]); ~25–40 k gates/s in-browser bb.js on M-series; wasm 4 GB cap,
  practical browser ceiling around 2¹⁹–2²⁰ constraints [V8WASM] [NOIR2543].
- Maintainable commitments: Hyperproofs — all n opening proofs maintained in
  O(log n)/update, aggregatable [HYPER22]; BalanceProofs improves aggregation
  [BALANCE23]; LVMT — O(1) group ops per root update [LVMT23]. Relevant only
  if sparq must serve *precomputed* openings at scale; plain Merkle root
  maintenance is O(b log N) hashes per batch and needs none of this.

---

## 4. Candidate architectures

What sparq uniquely adds to every option: **the engine can emit a query
trace** — for each result row, the permutation, block, and leaf indices of
every matched triple; the scan boundaries for every bound-prefix pattern; and
the join order actually executed. Witness generation becomes index arithmetic
plus Merkle-path extraction from a tree sparq maintains anyway, instead of
re-executing the query in a foreign stack (today `sparql_noir`'s TS pipeline
re-evaluates with Comunica/n3 to build witnesses **[judgement on current
pipeline detail]**). For completeness proofs the trace's scan boundaries are
*precisely* the adjacency witnesses §2.2 needs. This is cheap to expose: a
`TraceSink` on the executor behind a feature flag, zero cost when disabled.

### A. Monolith out-of-process (reuse `sparql_noir` as sidecar prover)

sparq exports `(quads, Root_g, DictRoot_g, trace)` for the touched scope; a
sidecar process runs the existing per-query compiled Noir circuit;
verification with the existing npm verifier.

- **Cost model**: per-query circuit ~10⁶+ gates (join machinery dominated,
  modular decision doc) ⇒ ~20–60 s native proving at 50 k gates/s
  **[judgement]** + per-query-shape `nargo` compile (cacheable). Verifier:
  one UltraHonk verify, ~ms–s. zkVM bar (7.5 min) beaten ~10×; the 1–2
  orders-of-magnitude target is marginal at the low end.
- **Pros**: zero new circuit work; full existing coverage (167/236
  conformance); one proof object. **Cons**: per-query compile latency is
  poison for a server loop; gate count scales with dataset-relevant Merkle
  depth × patterns × rows in one circuit — OOM-prone exactly like the
  monolith's known 10⁶+ profile; no incremental story.

### B. Modular per-property proofs + verifier-side composition (extend `sparql_noir_modular`), optional recursion later — **recommended**

Per-atomic-property circuits (measured: `filter_eq` 132 gates, `filter_lt`
2,925, `bgp_match` depth-8 1,410, `binding_consistency` 281; 5 proofs ≈ 5.2 s
prove / 3.2 s verify, manifest ~164 kB — HANDOFF-WAVE17), composed by a plain
verifier over the manifest. sparq's role: commitment maintenance + trace →
witnesses; circuits and verifier come from the existing repo.

- **Cost model** (worked example, **[judgement]** arithmetic on cited
  constants): pod of 10⁶ quads ⇒ depth-20 tree ⇒ `bgp_match` ≈ 1,410 +
  12·74 ≈ 2.3 k gates. Query: 3 patterns × 10 rows + 2 filters + consistency
  ≈ 35 module instances ≈ ~80 k gates total. Naively 35 proofs × ~1 s fixed
  overhead dominates (~35 s sequential, ~5–10 s parallel); **batching rows
  per module type** (one `bgp_match_batch[32]` circuit ≈ 75 k gates ≈ 1.5–2 s
  native) collapses this to ~4 proofs, ~3–5 s total. Verifier: 4 verifies +
  manifest checks, ~1–3 s, or browser-feasible. Completeness: + 2 adjacency
  proofs per scan boundary (same `bgp_match` machinery on neighbouring
  leaves).
- **Aggregation, honestly**: Ultra-in-Ultra recursion costs ~682 k gates per
  inner verify [BBGATES] — aggregating 4–10 proofs costs *more* prover time
  than it saves until verifier cost or proof size is the binding constraint
  (e.g. proofs posted to a ledger, or mobile verifiers). So: ship
  manifest-of-proofs first (his v0.3 status quo), add a recursion tree as an
  *optional compression stage*, and treat CHONK/Goblin (11.8 k gates per
  inner verify, if it becomes usable outside Aztec) as the upgrade that
  changes this calculus. (Q7)
- **Pros**: embarrassingly parallel; per-module circuits compile once ever
  (no per-query compile); maps 1:1 onto the trace; soundness-gap programme
  (G1–G5) already underway; Lean story per-module. **Cons**: manifest size
  O(modules); verifier does real work; G5-class composition soundness must
  be finished and ideally Lean-checked.

### C. Folding/IVC per-row predicates (Nova/HyperNova/Protostar, or CHONK)

Fold one step per result row (or per trace step): step circuit = "row r
matches pattern, filters pass, bindings consistent, leaf ∈ Root". Recursion
overhead ~10 k R1CS constraints (Nova) down to ~1 scalar mul (HyperNova)
[NOVA21] [HYPERNOVA23] — asymptotically the right shape for *long* row
streams (Reef proves the pattern works for regex-over-committed-docs
[REEF24]).

- **Cost model**: step circuit ≈ bgp_match-batch + filter ≈ 5–10 k
  constraints ⇒ thousands of rows/minute, constant memory, one final SNARK
  **[judgement]**.
- **The problem is the toolchain, not the math**: Sonobe's Noir frontend is
  explicitly experimental/unaudited [SONOBE]; bb's ClientIVC/CHONK is built
  for Aztec's kernel structure and its general-Noir availability is
  unverified [BBCLI]; leaving the bb/UltraHonk world (e.g. Nova on Pasta
  curves) abandons the Lampe→Lean and noir_IEEE754/noir_XPath investments.
  **Verdict: not now; re-evaluate when CHONK is documented for arbitrary
  Noir programs or when workloads with ≥10⁴ result rows materialise.** (Q7)

### D. zkVM re-execution (RISC Zero / SP1) — the baseline to beat

Compile a verifier-shaped sparq (or Oxigraph, as in the ISWC 2025 DC paper)
to a zkVM, prove "I ran the engine on committed input and got R". Measured
bar: 23 triples `SELECT *` ≈ 7.5 min on M1 [CEUR4085]. Modern SP1/RISC Zero
with GPU provers are faster (~10–100× on some workloads **[judgement,
vendor-influenced numbers]**) but per-instruction proving of a full engine
remains orders of magnitude above circuit cost for small queries. Keep as
the paper baseline and as the only option covering 100 % of SPARQL
semantics with zero circuit engineering. Not a product path.

**Architecture fit summary**: B dominates A on latency and serverability, C
on toolchain maturity, D on cost. A remains valuable as a *conformance
oracle* (same witness, both provers, results must agree) and for one-shot
proofs where a single proof object is required.

---

## 5. The inference question (open — flagged for Jesse, not decided)

`sparq-reason` materialises RDFS (counting-based incremental: every derived
triple is **one rule application from a base triple + closed TBox**), with
OWL-RL batch and an N3 engine that already emits proof trees
(`reason_n3_proof` → `ProofStep`). When inference is on, what does "the
query result is correct" mean? Options:

- **I1 — commit the materialized closure.** `Root_g` covers base ∪ derived;
  proofs are oblivious to inference. Honest but weak: the verifier trusts
  the materializer; the commitment no longer certifies "the owner asserted
  this", only "the server derived this". For T2/credentials this is likely
  unacceptable (an issuer signed base facts, not the server's closure).
- **I2 — commit base only; prove derivations on demand.** A derived triple in
  a result carries a *derivation witness*: for RDFS the depth-1 property
  makes this tiny — one rule id + one base-triple inclusion + one TBox-triple
  inclusion + an equality constraint, ≈ 2 Merkle paths + ~10² gates
  **[judgement]** per derived row, a new modular circuit family
  (`derive_rdfs_rule`). The counting engine's deterministic
  `emit_consequences` is exactly the witness generator. N3's `ProofStep`
  trees extend this to deeper derivations (cost ∝ proof-tree size).
  Completeness under inference ("no derivable answer missing") is the hard
  part — it needs the closure committed anyway, or a saturation argument.
- **I3 — dual commitment.** Commit base and closure separately
  (`BaseRoot_g`, `ClosureRoot_g`); soundness of disclosed rows via I2
  witnesses against `BaseRoot`; completeness via scan adjacency against
  `ClosureRoot`; optionally (expensive, research-grade) prove
  `ClosureRoot` is *the* closure of `BaseRoot` once per epoch via folding
  over rule applications — this is where architecture C re-enters.

Recommendation deferred (Q2): I1 ships free, I2 is the credible
research contribution and fits the modular architecture, I3 is the complete
story at epoch-proof cost. Stage 1 below ships with inference **off**.

---

## 6. Optimisation lever inventory ("optimise as much as possible")

1. **Membership: Merkle paths now, lookup arguments never (at this scale).**
   74-gate Poseidon2 ⇒ depth-27 (10⁸ quads) path ≈ 2 k gates; k = 100
   memberships ≈ 0.2 M gates ≈ 4–8 s native **[judgement]**. cq/logUp/Lasso
   all fail the N = 10⁸–10¹⁰ test (Ω(N) per proof, SRS cap, or structure
   requirement) [LOOKUPSOK25] [CQ22]; break-even ≈ 4–5 k lookups/proof at
   N = 10⁸ — beyond any realistic disclosed result. Re-examine only for
   *intra-circuit* tables (e.g. char-class tables in regex — noir_XPath
   already benefits from Noir's native lookup gates).
2. **Hash split: Poseidon2 in-circuit, Blake3 off-circuit, retire Pedersen
   default.** Poseidon2 74 gates vs Blake3 2,159 in-circuit [BBGATES] —
   never hash strings in-circuit (his `h_s` = Blake3-to-field design already
   ensures this; keep it). The spec's `h_2`/`h_4` Pedersen default should
   flip to Poseidon2 for new sparq-side commitments — bb gives Poseidon2 a
   dedicated gate; Pedersen's current bb cost is unpinned/unclear. (Q8)
3. **Commitment maintenance under updates — sparq's structural edge.**
   Group-commit batch of b quads in pod p: resort/update only p's subtree;
   O(b · log N_p) Poseidon2 evals off-circuit (~1 µs each native
   **[judgement]**) ⇒ ~20 µs per quad at N_p = 10⁶ — negligible against the
   write path. PoneglyphDB's full-recommit weakness [PONE25] is the explicit
   contrast. *Proving* the N→N+1 transition (PCD-style, Insarisa-like
   [HARISA22]) is **not needed** in the owner-signs-root trust model (the
   signature, not a proof, authenticates the new root); it becomes relevant
   only if a third party must audit update legality — park it. Hyperproofs/
   BalanceProofs [HYPER22] [BALANCE23] only if precomputed-opening serving
   at scale appears.
4. **Sorted-leaf adjacency for completeness and non-membership.** Two
   boundary adjacency proofs per scan replace per-candidate non-membership;
   this is the sentinel design generalised, and the single biggest lever for
   proving *completeness* cheaply — the thing no prior RDF system does.
5. **Verifier-side checks (his stated principle, enforced).** DISTINCT,
   ORDER BY, LIMIT/OFFSET, COUNT-over-disclosed, join edges over disclosed
   bindings, ACL scope membership: all plain code over the manifest. The
   modular dispatcher's `'proof' | 'clear'` obligation split already encodes
   this; sparq's contribution is emitting the obligations from the *real*
   plan rather than a re-derived one.
6. **Trace-driven witness minimisation.** Witness = leaf indices + paths +
   row bindings, extracted from the executor trace; no re-evaluation, no
   second engine. Also enables *proving the executed plan* (join order as
   public metadata) — cheap auditability without extra gates.
7. **Batch rows per module instance.** Fixed per-proof overhead (~1 s
   observed) dominates 10²–10³-gate circuits; `*_batch[K]` variants amortise
   it (§4B). Pick K to keep circuits under the browser ceiling (~2¹⁹
   constraints) so T2 holders can prove client-side. **[judgement]**
8. **Inline-id arithmetic in-circuit** (§2.1): numeric FILTERs over inline
   ids skip dict openings entirely; range checks shrink from u64 to 30-bit.
9. **Aggregation only when it pays** (§4B): manifest now; Ultra recursion
   tree as opt-in compression; Goblin/CHONK watch-item. (Q7)

---

## 7. Recommended architecture and staged adoption

**Recommendation: B on sparq's commitments.** Per-pod sorted-id-leaf
Poseidon2 Merkle commitments + committed dict, maintained incrementally by
the sparq-serve writer per generation; modular per-property Noir proofs
(batched per module type) produced by an out-of-process prover fed by an
executor trace; composition + revealed-property checks verifier-side;
recursion/folding deferred behind explicit triggers.

### Stage 1 — sidecar, zero engine impact

New optional crate `sparq-zk` (or a binary in Jesse's zkp workspace — Q1)
that consumes **existing public APIs only**: dump a pod's quads at a pinned
generation, build `PodRoot`/`DictRoot` out-of-process, run the query
*itself* (own evaluation over the dump is acceptable at this stage), build
witnesses, call the `sparql_noir_modular` prover, emit
manifest + generation number. No engine, wasm, or serve changes; not
compiled into any default build.
**Exit criteria**: (a) end-to-end prove+verify of a 3-pattern BGP + 1 hidden
filter over a 10⁴-quad pod in ≤ 10 s prove / ≤ 3 s verify on M-series
**[targets = judgement from measured 5.2 s/3.2 s demo]**; (b) tampered-row,
missing-row, and out-of-scope-row manifests all rejected; (c) zero diff in
sparq's benchmark suite (nothing changed).

### Stage 2 — commitments in the writer, trace in the engine

(a) `Generation` gains optional `{pod_roots, dict_root}` maintained
incrementally in the group-commit apply path, feature-gated; (b) executor
gains a `TraceSink` (leaf/block indices per matched row, scan boundaries),
feature-gated, zero-cost when off; (c) sidecar switches to trace-fed
witnesses and signed roots.
**Exit criteria**: (a) write-path overhead with commitments on ≤ 10 % on the
update_stream bench **[judgement target]**; (b) witness generation ≤ 100 ms
for 10²-row results (vs re-evaluation); (c) proof pinned to generation
number verifies against the root the writer published; (d) feature-off
builds byte-identical benchmarks.

### Stage 3 — completeness, scope, batching

Adjacency-based scan-completeness proofs; ACL-scoped multi-pod proofs
(scope = disclosed PodRoot set); `*_batch[K]` circuits; first honest
benchmark table vs the zkVM baseline and vs monolith-A on identical queries
(the "not captured" monolith-vs-modular comparison, now with a third
column).
**Exit criteria**: completeness proof for a bound-prefix scan at ≤ 2×
soundness-only cost **[judgement]**; ≥ 10× beat vs zkVM baseline on the
23-triple query and on a 10⁶-quad pod query; comparison table published.

### Stage 4 — research options, each behind a trigger

Recursion tree (trigger: verifier cost or manifest size becomes binding);
CHONK/Goblin aggregation (trigger: documented non-Aztec support);
derivation proofs I2/I3 (trigger: Jesse's call on Q2); T2 pod-export
credential flow (trigger: alignment with zkSPARQL paper needs).

---

## 8. Honest risks

- **Composition soundness is the crux of B.** G1–G4 closed, G5 in flight;
  until the manifest composition argument is finished (ideally Lean-checked
  over `ProofManifest × Query → Bool`), B is weaker than A's single-circuit
  trust boundary. Mitigation: A-as-oracle in CI.
- **bb/Noir churn**: nightly bb.js pin, beta toolchain, recursion "very much
  experimental" per Aztec docs [BBREC]. The sidecar boundary contains this.
- **Numbers above at 10⁸+ quads are extrapolations** from ≤ 10⁴-quad
  measurements; depth-27 trees, 100-row batches, and dict commitments at
  Wikidata scale are unmeasured. Stage gates exist to falsify them early.
- **Dual evaluation drift** (stage 1 evaluates the query outside sparq):
  divergence between sparq's answer and the sidecar's would make proofs
  attest the *wrong* answer. Stage 2's trace removes this class.

## 9. Open questions for Jesse

1. **Where does this live?** A `sparq-zk` crate in this repo, or a consumer
   in `zkp-sparql-workspace` that depends on sparq as a library? (Workspace
   keeps circuit/Lean tooling together; sparq crate keeps the trace/commit
   seams honest. Recommendation: commitment+trace seams in sparq, prover in
   the workspace — but your call.)
2. **Inference semantics**: I1 / I2 / I3 (§5)? Does the ISWC 2026 paper
   want derivation proofs as a contribution, or is inference-off the right
   scope for v1?
3. **Threat-model priority**: is T1 (untrusted Solid server) actually a
   target you want, or is sparq's role purely to be the
   commitment-maintaining substrate + witness generator for T2/zkSPARQL?
4. **Commitment leaf encoding**: id-triples + committed dict (cheap
   maintenance, store-local) vs term-hash triples (store-independent,
   RDFC10-compatible, costlier rebuilds)? This also interacts with the M4
   lexicographic-id migration.
5. **Blank-node canonicalization** for T2 export: RDFC10 at export time, or
   restrict v1 to skolemized/bnode-free pods?
6. **Who signs roots in Solid?** Pod owner key, server key, or both
   (owner-over-server)? And which signature module first (Schnorr is
   cheapest in-circuit; BBS+ aligns with the VC story)?
7. **Aggregation posture**: accept manifest-of-proofs verifier cost for the
   paper, or invest in the Ultra recursion tree now / gamble on CHONK?
8. **Flip the spec's `h_2`/`h_4` default from Pedersen to Poseidon2** for
   sparq-side commitments? (74 vs unpinned-but-larger gates; affects
   cross-compatibility with existing signed datasets.)
9. **Timeline coupling**: should sparq integration target the ISWC 2026
   paper (i.e. stage 1–2 benchmarks feeding the eval section), or stay
   decoupled until after submission? The roborev/no-push constraints from
   the optimisation project — do they apply to this module too?
10. **Scale ambition for the eval**: 10⁴ quads (credential-sized, matches
    zkSPARQL bench) or push to 10⁶–10⁸ (pod/server-sized, where the
    commitment-maintenance story differentiates against PoneglyphDB/SXT)?

## 10. Bibliography

- [VSQL17] Zhang, Genkin, Katz, Papadopoulos, Papamanthou. vSQL. IEEE S&P
  2017. https://eprint.iacr.org/2017/1145
- [INTEGRIDB15] Zhang, Katz, Papamanthou. IntegriDB. CCS 2015.
  https://dl.acm.org/doi/10.1145/2810103.2813711
- [ZKSQL23] Li, Weng, Xu, Wang, Rogers. ZKSQL. PVLDB 16(8), 2023.
  https://www.vldb.org/pvldb/vol16/p1804-li.pdf
- [PONE25] Gu, Fang, Nawab. PoneglyphDB. SIGMOD/PACMMOD 2025.
  https://arxiv.org/abs/2411.15031
- [FALCON20] Peng et al. FalconDB. SIGMOD 2020.
  https://users.cs.utah.edu/~lifeifei/papers/falcondb.pdf
- [SXT] Space and Time, Proof of SQL (vendor).
  https://github.com/spaceandtimefdn/sxt-proof-of-sql
- [REEF24] Angel et al. Reef. USENIX Security 2024.
  https://eprint.iacr.org/2023/1886
- [VERIDKG24] Zhou et al. VeriDKG. PVLDB 17(5), 2024.
  https://www.vldb.org/pvldb/vol17/p912-zhou.pdf
- [BK25] Braun, Käfer. ESWC 2025.
  https://link.springer.com/chapter/10.1007/978-3-031-94575-5_21
- [BWK26] Braun, Wright, Käfer. Proving Soundness of SPARQL Query Results…
  ESWC 2026. https://link.springer.com/chapter/10.1007/978-3-032-25156-5_16
- [RDFPROOFS] Yamamoto et al. zkp-ld/rdf-proofs.
  https://github.com/zkp-ld/rdf-proofs
- [CEUR4085] Wright. ISWC 2025 Doctoral Consortium, CEUR Vol-4085 paper 19.
- [ZKSPARQL] Wright, Shadbolt, J. Zhao, R. Zhao, Braun. zkSPARQL (ISWC 2026
  submission). https://zksparql.org/
- [HYPER22] Srinivasan et al. Hyperproofs. USENIX Security 2022.
  https://eprint.iacr.org/2021/599
- [BALANCE23] Wang, Ulichney, Papamanthou. BalanceProofs. USENIX Security
  2023. https://eprint.iacr.org/2022/864
- [LVMT23] Li et al. LVMT. OSDI 2023.
  https://people.iiis.tsinghua.edu.cn/~weixu/Krvdro9c/li-osdi23.pdf
- [HARISA22] Campanelli et al. Harisa/Insarisa. CCS 2022.
  https://eprint.iacr.org/2021/1672
- [BBF19] Boneh, Bünz, Fisch. Accumulator batching. CRYPTO 2019.
  https://eprint.iacr.org/2018/1188
- [CQ22] Eagen, Fiore, Gabizon. cq. https://eprint.iacr.org/2022/1763
- [LOGUP22] Haböck. logUp. https://eprint.iacr.org/2022/1530
- [LASSO23] Setty, Thaler, Wahby. Lasso. https://eprint.iacr.org/2023/1216
- [LOOKUPSOK25] SoK: Lookup Table Arguments.
  https://eprint.iacr.org/2025/1876
- [POSEIDON2] Grassi, Khovratovich, Schofnegger.
  https://eprint.iacr.org/2023/323
- [BBGATES] Barretenberg pinned gate-count constants (primary source).
  https://github.com/AztecProtocol/aztec-packages/blob/master/barretenberg/cpp/src/barretenberg/dsl/acir_format/gate_count_constants.hpp
- [BBREC] bb recursive aggregation guide.
  https://barretenberg.aztec.network/docs/how_to_guides/recursive_aggregation
- [BBCLI] bb CLI reference (`--scheme chonk`).
  https://barretenberg.aztec.network/docs/bb-cli-reference
- [AZTEC-ROADMAP] Aztec roadmap (CHONK = HyperNova-style folding + Goblin).
  https://aztec.network/blog/aztec-network-roadmap-update
- [NOVA21] Kothapalli, Setty, Tzialla. Nova. https://eprint.iacr.org/2021/370
- [HYPERNOVA23] Kothapalli, Setty. HyperNova.
  https://eprint.iacr.org/2023/573
- [SONOBE] PSE Sonobe (experimental Noir frontend).
  https://github.com/privacy-scaling-explorations/sonobe
- [NOIRBENCH] Savio-Sou/noir-benchmarks (order-of-magnitude only).
  https://github.com/Savio-Sou/noir-benchmarks
- [V8WASM] V8: up to 4 GB wasm memory. https://v8.dev/blog/4gb-wasm-memory
- [NOIR2543] noir-lang/noir#2543 (browser proving ceiling).
  https://github.com/noir-lang/noir/issues/2543

Internal sources: `research/zkp-noir-context.md`;
`zkp-sparql-workspace/{HANDOFF-WAVE17.md, decisions/sparql-noir-modular-alternative.md, notes/research/02,05,08}`;
`sparql_noir/spec/{encoding,algebra,proofs,preprocessing}.md`;
sparq `crates/{sparq-core/src/{dict,store}.rs, sparq-serve/src/{epoch,ring,writer}.rs, sparq-reason/src/{incremental,lib}.rs}`;
`research/{ARCHITECTURE.md, concurrent-serving.md §2.8–2.10, solid-access-control-design.md}`.
