# ZKP query-correctness proofs for sparq — derived credentials over issuer-signed named graphs (OPTIONAL module)

Status: **design v2 for review — nothing implemented, no code touched.**
Author: research agent, 2026-06-12 (v1), revised same day (v2) per Jesse's
review. Reviewer: Jesse Wright.
Inputs: `research/zkp-noir-context.md` (reconstruction of the sparql_noir /
sparql_noir_modular line), sparq source (`sparq-core`, `sparq-serve`,
`sparq-reason`, `sparq-solid`), `research/concurrent-serving.md`,
`research/solid-access-control-design.md`, the zkp-sparql-workspace research
notes (reused, not re-derived), and fresh web research (June 2026). Every
uncited number is marked **[judgement]**. Open questions for Jesse are
numbered §9 and referenced inline as (Q*n*). Proving-performance landscape
(hardware, proof systems beyond Noir/UltraHonk, ACIR reuse) is being surveyed
separately into `research/zkp-performance-landscape.md` (forthcoming);
this plan keeps v1's measured anchors and defers proof-system selection
questions to that document.

## v2 changelog (what changed since v1, and why)

Jesse's review reshaped the threat and data model:

1. **Primary use case flipped.** v1 made T1 (untrusted Solid server proving
   to a client over whole-database commitments) primary. v2's primary
   scenario is the holder storing credentials **issued by other services —
   such as drivers licenses, tickets** — and proving **that a certain set of
   facts holds as a "derived credential"**. These credentials will **largely
   be verifiable credentials** (W3C VCs), Jesse's semantic reservations about
   that data model notwithstanding. This is v1's T2 (his DPhil RQ1 setting),
   sharpened; the untrusted-server model is downgraded to a secondary,
   optional track (§1.3).
2. **Storage mapping fixed by feedback.** "Store the content of the
   credential in a named graph — which would correspond to the document
   location. The proof graph can then be considered metadata. This can live
   in separate metadata graph(s) similar to how access controls are
   currently handled." §2.1 mirrors `solid-access-control-design.md`'s
   conventions (per-document named graphs + a reserved synthesized graph).
3. **The proof statement changed (the central change).** "The proof is then
   not a proof over the whole database state, it is that a query result
   holds over union of some set of undisclosed named graphs which have been
   signed by issuers with public keys X/Y/Z." Commitments become
   **per named graph** (graph IRI = credential document location); the
   disclosed scope is the **issuer-key set**, not the graph set (§1.1, §2).
4. **Consequences.** Per-graph commitments are *small* (a driver's license is
   tens of triples), which collapses the cost model: tiny Merkle trees or
   flat full-graph hashing, browser-side proving becomes the primary
   envelope, and v1's sorted-leaf adjacency machinery is mostly superseded
   for the primary path (§4, §6). v1 open questions Q3 (threat priority),
   Q4 (leaf encoding), Q5 (RDFC10/bnodes) and Q10 (scale ambition) are
   resolved by the feedback (§9 records how); new questions around VC
   interop, union sizing, revocation, and holder binding replace them.
5. **What survives.** The commitment/trace seams in sparq, the modular
   architecture-B recommendation, the SOTA survey, the optimisation
   discipline, and the staging shape all survive with updated content. The
   serving integration is recast: per-NAMED-GRAPH commitment maintenance and
   metadata-graph bookkeeping at ingest, rather than per-pod roots in the
   generation ring.

---

## 1. Problem statement and threat model

### 1.1 Primary scenario — holder-side derived credentials

A holder's personal data store (sparq as the storage/query tier, possibly in
the browser via the wasm build) contains **credentials issued by other
services — drivers licenses, tickets, attestations — largely W3C Verifiable
Credentials**. Each credential's content lives in one named graph whose name
is the credential document's location (§2.1). Each such graph has a
**per-graph commitment** that an **issuer has signed**.

The holder runs a SPARQL query over their credential store and presents the
result to a verifier as a **derived credential**: a disclosed result `R`
plus a proof of the statement

> **S(R, Q, K, k):** there exist named graphs `G_1 … G_k`, commitments
> `C(G_1) … C(G_k)`, signatures `σ_1 … σ_k`, and keys `pk_1 … pk_k` such
> that (a) for every `i`, `pk_i ∈ K` and `Verify(pk_i, σ_i, C(G_i))` holds,
> where `C` is the agreed commitment function over the canonical form of
> `G_i` (§2.2); (b) `R = eval(Q, merge(G_1, …, G_k))` under the
> Pérez–Arenas–Gutiérrez algebra — **sound and complete** w.r.t. the merge;
> and (c) nothing else is revealed: not the graphs, not their commitments,
> not the signatures, not which key in `K` signed which graph.

Public inputs: `Q`, `R`, the disclosed issuer-key set `K = {X, Y, Z}`, the
union arity `k` (or an upper bound, §2.4), a verifier-supplied challenge
(replay binding, §2.5), and revocation epoch data if used (§2.6). Private
witnesses: the graphs, commitments, signatures, and key selectors.

Two properties of this statement deserve emphasis:

- **Completeness is still load-bearing.** "No matching row is missing" is
  quantified over the *merge of the selected graphs*, not over the holder's
  whole store. A holder may freely choose *which* credentials to query —
  that choice is the holder's privacy right, not a soundness hole. What the
  proof must prevent is the holder suppressing rows *within* the selected
  graphs (e.g. hiding a "license: suspended" triple inside the same signed
  license graph). Almost all prior art stops at soundness
  (Braun/Wright/Käfer ESWC 2026 explicitly proves soundness *only* [BWK26]);
  completeness-within-the-merge is the differentiator, and at credential
  scale it is *cheap* (§2.2, §6.4).
- **Graph-set privacy inverts a v1 principle.** v1 followed Jesse's rule
  that scope is disclosed data, checked verifier-side (the ACL-visible
  PodRoot set was public). Here the *graph set is the secret* — which
  credentials the holder used must not leak (a disclosed commitment is
  linkable by an issuer–verifier collusion, since the issuer saw `C(G)` at
  issuance). So the disclosed scope becomes the **issuer-key set** `K`, and
  graph membership-under-signature moves in-circuit (§2.5). The principle
  survives in its corrected form: *disclosed* properties of the result
  (DISTINCT, ORDER BY, …) stay verifier-side; the *scope witness* does not.

### 1.2 Why sparq is involved at all

sparq is the holder's credential store and query engine. Its roles:

1. **Ingest-time bookkeeping**: when a credential document is loaded, store
   its content graph, compute its canonical form + commitment, record
   `(graph, commitment, issuer key, signature, status)` in a reserved
   metadata graph (§2.1) — mirroring how `sparq-solid` synthesizes
   `<urn:sparq:auth>`.
2. **Witness generation from the real plan**: the engine can emit a query
   trace — matched leaf/slot indices per row, scan boundaries, executed
   join order — so witness building is index arithmetic, not re-evaluation
   in a foreign stack (today `sparql_noir`'s TS pipeline re-evaluates with
   Comunica/n3 to build witnesses **[judgement on current pipeline
   detail]**). The sparq wasm build makes this available *in the browser*,
   next to bb.js proving.
3. **The proving stack itself comes from Jesse's existing repos**
   (`sparql_noir`, `sparql_noir_modular`); sparq supplies commitments,
   metadata, and traces — not circuits.

### 1.3 Secondary (optional) track — untrusted server proofs

v1's T1 — the Solid client distrusting the pod server and demanding proofs
that answers are sound and complete w.r.t. a server-published commitment —
is **downgraded to optional**. Honest assessment of what it still buys:

- The **same per-named-graph commitments** serve it: a per-pod (or
  per-server) super-root over `(graph_iri_hash, C(G))` pairs restores the
  whole-database statement when wanted, and the generation ring
  (`sparq-serve` ArcSwap ring, group-commit batches) remains the natural
  commitment-epoch mechanism — now maintaining per-graph commitments in the
  apply path, with the super-root as an optional extra fold.
- The serving integration that matters *now* is the part both tracks share:
  **metadata-graph bookkeeping and per-named-graph commitment maintenance
  at ingest/write time**. That is what stages 1–2 build (§7); the
  super-root, ACL-scoped completeness, and signed-generation story are
  parked behind an explicit trigger in stage 4.
- The SQL-integrity prior art (vSQL/IntegriDB/Proof-of-SQL, §3) remains the
  literature anchor for this track only.

Out of scope for this plan: proving *updates* were authorised (the Solid
auth layer's job), MPC/federation (DPhil RQ2), proof of non-tampering of
the server binary (zkVM territory; baseline only), and the verifier-side
trust establishment for issuer keys themselves (DID resolution / trust
registries — manifest format hook only, Q12).

---

## 2. Data model: credentials as named graphs, proofs as metadata

### 2.1 Storage mapping (mirrors the access-control design)

`solid-access-control-design.md` establishes the house pattern: every pod
document is a named graph whose name is the resource IRI; ACL/ACR documents
are ordinary named graphs under a naming convention (`<R + ".acl">`); and
one **reserved synthesized graph** `<urn:sparq:auth>` holds the
materialized view, with the `urn:sparq:` IRI space reserved, loader-stripped
from incoming datasets, and writable only by its own installer. Mirror all
three for credentials:

- **Credential content**: the asserted triples of credential document `D`
  (for a VC: the credential body including `credentialSubject`) live in
  named graph `<D>` — graph IRI = document location, exactly as pod
  resources already work.
- **The credential's proof graph is metadata, not content.** A VC-DI proof
  is graph-valued (`sec:proof` is an `@graph` container in JSON-LD), so it
  arrives as a separate graph naturally. It is stored as its own named
  graph adjacent to `<D>` (naming convention analogous to `.acl` — exact
  convention Q13) and is **excluded from the dataset the proof statement
  quantifies over**: `merge(G_1…G_k)` is over content graphs only. Signature
  bytes are not facts the holder asserts; they are evidence *about* `<D>`.
- **One reserved synthesized graph `<urn:sparq:zk>`** (vocabulary
  `zk:` = `https://sparq.dev/ns/zk#`, mirroring `auth:`) holds the
  registry the prover needs, maintained only by the ingest path:

  ```turtle
  @prefix zk: <https://sparq.dev/ns/zk#> .

  <https://dmv.example/vc/lic-123>
      zk:commitment      "0x1a2b…"^^zk:field ;
      zk:scheme          zk:poseidon2-rdfc10-v1 ;
      zk:issuerKey       <did:example:dmv#key-1> ;
      zk:signatureGraph  <https://dmv.example/vc/lic-123?proof> ;
      zk:status          zk:active ;
      zk:ingested        "2026-06-12T…"^^xsd:dateTime .
  ```

  Same hardening rules as `<urn:sparq:auth>`: `urn:sparq:zk` is in the
  reserved IRI space, pre-existing copies are stripped on load, only the
  ingest path writes it, and it is excluded from query-visible datasets
  (the verifier never sees it; the prover reads it as its witness index).
  Reading it through the access-controlled path can be gated like `.acl`
  graphs are (Control-holders only) — it reveals exactly which credentials
  the holder possesses, the most sensitive inventory in the store.

This is deliberately the access-control design's shape: content graphs stay
ordinary and queryable (D1 of that doc), derived/bookkeeping state lives in
one reserved graph, and the security boundary is "only designated inputs
feed the synthesizer".

### 2.2 Per-graph commitments: construction, canonicalization, size

**Commitment target = the RDFC10-canonicalized content graph.** v1 left a
fork open (Q4: store-local id-leaves vs store-independent term-hash leaves;
Q5: RDFC10 at export vs skolemized-only). The new model closes both:

- Three parties (issuer signs, holder proves, verifier checks) means the
  commitment **must** be store-independent — dict-local ids are useless
  across trust boundaries. Term-hash leaves win.
- Rebuild cost, the argument for id-leaves, evaporates at credential scale:
  RDFC10-canonicalizing and re-hashing a 40-triple graph at ingest is
  microseconds-to-milliseconds off-circuit, paid once per credential.
- RDFC10 runs **per graph at ingest**, giving canonical bnode labels within
  each graph. (RDFC10 has pathological-input blow-ups; credentials are
  small but ingest should still cap canonicalization work — §8.)

Encoding stays Jesse's: `Enc_t(term) = h_2(type_code, h_s(value))` with
`h_s` = Blake3 off-circuit and `h_2` = Poseidon2 (flip the spec's Pedersen
default, Q10). **Bnode encoding gains graph scoping**: encode a blank node
as `h_2(BlankNode_code, h_2(C_pre(G), blake3(canonical_label)))` (with
`C_pre` a pre-commitment over the label-free structure, or simply the
issuer-assigned document IRI hash — exact construction to be specced). This
makes bnodes from different graphs *distinct by construction*, which is
precisely RDF **merge** semantics — see §2.4.

**Two commitment shapes, chosen by graph size:**

1. **Flat hash (primary, for credential-sized graphs).** Sort the encoded
   triples; `C(G)` = Poseidon2 chain/sponge over the sequence of
   `h_3(s,p,o)` leaf hashes. To use the graph in-circuit, the prover
   supplies *all* triples as witness and the circuit **recomputes `C(G)`
   from scratch**: at ~1 Poseidon2 permutation per triple for the leaf plus
   ~1 for the chain step ≈ **~150 gates/triple [judgement on the 74-gate
   Poseidon2 anchor [BBGATES]]**, a 40-triple license costs ~6 k gates and
   a k=3 union of ~100 triples ~15 k gates. The payoff is structural: the
   **entire graph is in-circuit**, so membership is array indexing,
   **completeness of any scan is a linear sweep, and non-membership /
   NOT EXISTS / MINUS are trivial** — no sorted-adjacency sentinels, no
   non-membership circuits, no boundary proofs. The single biggest
   machinery item of v1 (§6.4 there) disappears for the primary path.
2. **Per-graph Merkle (fallback, for large graphs).** A credential that is
   itself a dataset dump (10³–10⁶ triples — e.g. a signed registry extract)
   reverts to the v1 design *per graph*: sorted-leaf Poseidon2 Merkle tree,
   membership by path (depth-10 tree = 740 gates/triple — already worse
   per-triple than flat recompute below ~2¹⁰ triples when several triples
   are touched **[judgement]**), completeness by boundary adjacency, the
   `bgp_nonmember_prefix3` sentinel design for non-membership. Break-even
   between the two shapes is roughly where (triples touched × path cost)
   exceeds (graph size × 150 gates) — i.e. flat wins whenever the query
   touches more than ~`n/10` of an `n`-triple graph or `n ≲` a few hundred
   **[judgement; measure in stage 1]**.

The committed-dictionary bridge of v1 (§2.1 there, `DictRoot` over
`id → Enc_t`) is **demoted to the optional server track**: it existed to
avoid re-hashing terms in a mutable 10⁸-quad store. Holder-side, terms are
hashed once at ingest and the commitment is term-level already. Likewise
the inline-id arithmetic lever survives only as a server-track note (§6.8).

### 2.3 Issuer signatures and VC pragmatics

**What the issuer signs is the crux.** The statement S needs
`Verify(pk_i, σ_i, C(G_i))` *in-circuit* (the signature and commitment are
private). Two issuance realities:

- **(a) Cooperative / zkSPARQL-aware issuers** sign the Poseidon2 graph
  commitment `C(G)` directly, with a circuit-friendly scheme (Schnorr over
  the embedded curve is the cheap default — order(10³–10⁴) gates
  **[judgement; pin via performance doc]**; ECDSA-secp256k1 is the
  measured expensive anchor at **42,838 gates** [BBGATES]). This is the
  envelope all cost sketches in §4 assume, and it is a *legitimate* VC: a
  Data Integrity proof is parameterized by cryptosuite, and "sign the
  Poseidon2-RDFC10 graph commitment" is exactly a custom cryptosuite
  [VC-DI] — the credential remains a conforming W3C VC document, which
  matches Jesse's "largely verifiable credentials" framing without
  swallowing the parts of the data model he dislikes.
- **(b) Standard VC-DI issuers** (the deployed world: `eddsa-rdfc-2022`,
  `ecdsa-rdfc-2019`, `bbs-2023`) sign SHA-256 digests of RDFC10
  **N-Quads bytes**. Verifying *that* in-circuit means hashing the actual
  serialized bytes (a ~2–4 kB credential ≈ 40–64 SHA-256 blocks ×
  6,703 gates ≈ **270–430 k gates [judgement arithmetic on cited
  constants]**) plus a non-native-curve signature verify (Ed25519/P-256 on
  BN254: expensive, unpinned here — **defer to
  `zkp-performance-landscape.md`**). This violates the house rule "never
  hash strings in-circuit" and blows the browser ceiling at k ≥ 2. It is
  the **interop cost cliff**: real third-party credentials won't carry
  cooperative signatures on day one. Mitigations, in declining preference:
  a custom cryptosuite issuers can adopt (a); a **re-signing bridge** (a
  party the verifier trusts checks the standard VC-DI proof off-circuit and
  signs `C(G)` — honest trust-model cost: the bridge is trusted for
  issuance integrity, not for query correctness); a **commitment bridge**
  from a BBS+ presentation (prove knowledge of a `bbs-2023` signature
  outside the circuit while binding a hidden message equal to a Pedersen
  commitment shared with the Noir circuit — the zkp-ld/rdf-proofs
  neighborhood [RDFPROOFS]); or paying the in-circuit cliff natively for
  k=1 flows. Which to target first is Q3.

**Does the named-graph-commitment approach subsume BBS+ selective
disclosure? Largely yes — with one honest caveat.** BBS+ in the VC stack
buys two things: (i) *selective disclosure* — reveal a subset of signed
messages; (ii) *unlinkable multi-show* — each presentation is a fresh proof
of knowledge of the signature, so presentations cannot be correlated
through signature values. The graph-commitment model provides strictly
more of (i): "reveal subset of triples" is the special case
`Q = SELECT/CONSTRUCT those triples`, and arbitrary derived statements
(joins across credentials, filters over *hidden* values, EXISTS booleans,
inference-backed facts — §5) are exactly what BBS+ message-level disclosure
cannot express. And it provides (ii) *for any signature scheme*: since
`σ` and `C(G)` are private witnesses, nothing signature-correlated is ever
revealed — unlinkability does not depend on BBS+'s algebraic structure.
So functionally, **per-graph commitments + ZK query proofs subsume BBS+**;
BBS+'s residual roles are (α) ecosystem interop — verifiers that only speak
`vc-di-bbs` presentations [VC-DI-BBS] won't accept Noir manifests, and (β)
the commitment-bridge mitigation above, where BBS+ is the cheapest way to
*link out of* a standard credential without in-circuit byte hashing. State
this in the paper; it reconciles v1's RDFC10 open question (the
canonicalized form is what gets committed, per graph, at ingest) with the
signature story (the issuer signs the commitment, not the quads).

Linkability fine print (for the paper's privacy claims): hiding `σ` and
`C(G)` removes *cryptographic* linkage; the disclosed result `R` itself can
still identify the holder (a name, a license number). Disclosure analysis
of `R` is the application's responsibility — same boundary as Jesse's
"don't ZK-prove revealed properties" rule, which continues to govern
DISTINCT/ORDER BY/LIMIT/COUNT-over-disclosed (verifier-side over the
manifest).

### 2.4 Union semantics in-circuit

`merge(G_1…G_k)` — RDF **merge**, not naive union: blank nodes are
graph-scoped, so identically-labelled bnodes in different credentials must
*not* be identified. The §2.2 graph-scoped bnode encoding makes this hold
by construction: cross-graph joins happen on IRIs and literals (the only
terms with cross-credential identity), and a cross-graph bnode join is
impossible — which is semantically correct. If a use case genuinely needs
cross-credential bnode identity it needs issuer-side skolemization, not
circuit machinery (Q6).

**k roots, not one super-structure.** The circuit takes `k` per-graph
witness blocks; there is no benefit to a holder-built super-tree over the
`k` commitments, because the signatures are per-commitment anyway and the
graph set is private. Concretely, with flat commitments: witness layout =
`k_max` blocks of `n_max` triple slots (padding slots carry a sentinel
encoding); the circuit recomputes each `C(G_i)`, verifies each
`(σ_i, pk_i)` with `pk_i` constrained to lie in the disclosed set `K`
(a `k × |K|` selector — trivial gates), and the query modules then operate
over the concatenated slot array with per-slot graph tags. Sizing: e.g.
`k_max = 4`, `n_max = 64` ⇒ 256 slots ⇒ ~40 k gates of hashing — well
inside the browser ceiling (~2¹⁹–2²⁰ constraints [V8WASM] [NOIR2543]).
Fixed-`k_max`-with-padding leaks only an upper bound on the union arity;
whether to ship a small circuit family (k ∈ {1,2,4,8}) instead is Q5.

**GRAPH visibility**: the merge is presented to the query as the default
graph; `GRAPH ?g` over undisclosed graph names is contradictory with
graph-set privacy and is excluded from the supported fragment (a query
whose *answer* depends on graph names would disclose them — redesign the
disclosure, per the house rule).

### 2.5 Graph-selection privacy, duplicates, and replay

- **Selection privacy**: which `k` of the holder's `m` stored credentials
  were used is hidden because commitments, signatures, and key-selectors
  are private witnesses; the verifier learns only `K` and `k_max`. The
  `<urn:sparq:zk>` registry is the prover's private index for witness
  selection — it never leaves the store.
- **Duplicate-inclusion pitfall (new, real).** With graph-scoped bnode
  encodings, including the *same* credential twice at different block
  indices would mint distinct bnodes per inclusion only if the scope tag
  were the block index — so the scope tag must be commitment-derived
  (§2.2), making double-inclusion idempotent on bnodes; and the circuit
  additionally enforces strict ordering `C(G_1) < C(G_2) < … < C(G_k)`
  (numeric, on the field representative) to force pairwise-distinct graphs.
  Without this, COUNT-style derived claims ("I hold ≥ 2 tickets") are
  forgeable by including one ticket twice. Aggregates are not in the v1
  fragment, but the ordering constraint is ~free and closes the class now.
- **Replay / holder binding.** A derived credential proven once could be
  replayed by anyone who obtains the manifest. Minimum: the verifier's
  challenge (nonce + audience) is a public input baked into every
  constituent proof's claim hash — proofs are per-presentation. This makes
  **holder-side proving latency the binding UX constraint**, which the
  small-circuit envelope is sized for. Whether to *also* bind a holder key
  in-circuit (proof-of-possession, enabling offline-presentable derived
  credentials with their own lifetime) is Q8.

### 2.6 Revocation hooks (open question, flagged not solved)

Issuer-signed commitments are forever; real credentials get revoked. The VC
ecosystem's standard answer — Bitstring Status List [VC-STATUS] — has the
verifier fetch a list and check an index, which is unusable here directly
(the index identifies the credential). ZK-compatible hooks, in rough order
of pragmatism:

1. **Hidden-index status-list inclusion**: commit the (issuer-published,
   issuer-signed) status bitstring as a Merkle tree; the circuit proves
   "bit at my credential's (hidden) index = 0" against the *disclosed*
   status-list root — one extra Merkle path (~1–2 k gates **[judgement]**)
   plus one extra issuer-signature check per credential. Freshness = the
   status root's epoch is a public input.
2. **Cryptographic accumulators** (CL02 dynamic accumulators [CL02],
   AnonCreds-style pairing/VB accumulators): non-membership witnesses,
   issuer-maintained; stronger privacy, heavier in-circuit cost on BN254,
   witness-update burden on holders.

Both change the statement S (conjoin per-credential non-revocation at a
disclosed epoch). Which mechanism, and what freshness semantics verifiers
actually demand, is Q7 — coordinate with what the VC ecosystem converges on
rather than inventing here.

### 2.7 What survives from v1's server-side machinery

- **Generation ring as commitment epochs** (v1 §2.3): still the right
  write-path hook, now maintaining *per-named-graph* commitments in the
  group-commit apply path — recompute `C(G)` for each graph touched by a
  batch (cheap: graphs are documents; a batch touches few). The per-pod
  super-root, owner signatures over roots, and ACL-scoped completeness
  move to the optional server track (stage 4).
- **Per-pod epoch vector** (v1 §2.4): its real content — "unrelated writes
  never touch another graph's commitment" — is exactly the per-graph model;
  it survives as the invalidation map for commitment maintenance.
- **Committed dictionary + inline-id circuits** (v1 §2.1): server-track
  only, as noted in §2.2.

---

## 3. State of the art (web research, June 2026 — full citations §10)

**Verifiable SQL/DB** (now anchoring the *secondary* track, kept for the
paper's related-work and because the commit-then-prove pattern carries
over). vSQL [VSQL17]: interactive proofs + polynomial delegation; TPC-H Q6
prover 3,851 s vs 0.67–4.16 s plaintext (~10³× overhead); not ZK in the
published version. IntegriDB [INTEGRIDB15]: ADS (m² authenticated interval
trees), setup 25,272 s and 1.85 GB ADS for a 30 MB table — the O(m²n)
blow-up is disqualifying for RDF. ZKSQL [ZKSQL23]: VOLE-based interactive
ZK, designated-verifier — interactivity clashes with credential
presentation (offline verifiability is mandatory here). PoneglyphDB
[PONE25]: non-interactive Halo2 per-operator circuits + recursion; concedes
full recommitment on every update — irrelevant at credential scale (a
"recommit" is one small graph), still the contrast for the server track.
FalconDB [FALCON20]: ADS + blockchain; proof generation "seconds to hours".
Space and Time "Proof of SQL" [SXT]: production Rust prover over per-table
commitments, vendor-reported >1 M rows < 1 s on GPU — per-table commitments
are structurally the cousin of per-graph commitments; closed query
fragment, vendor numbers. Reef [REEF24]: "commit once, prove many
predicates" over committed documents via Nova folding — the per-document
framing is *closer* to v2's model than any SQL system; documents ≈
credentials.

**RDF/SPARQL/credential-specific.** Confirmed near-empty at algebra level:
term-level selective disclosure (Yamamoto et al. zkp-ld/rdf-proofs
[RDFPROOFS] — BBS+-based, per-statement, no query algebra; now also the
nearest *credential-model* relative of v2 and the source of the
commitment-bridge idea in §2.3); Braun & Käfer ESWC 2025 [BK25];
VeriDKG [VERIDKG24] (verifiable-but-not-ZK SPARQL over a Merkle prefix
trie); Braun/Wright/Käfer ESWC 2026 — soundness only [BWK26]; and the
zkSPARQL line (Wright et al., ISWC 2026 submission, zksparql.org) — Jesse's
own work — the only algebra-level soundness+completeness system found. The
gap claim survives adversarial search. **The derived-credential framing
strengthens the paper's positioning**: it is the FOSDEM-2025 vision
statement made concrete, and no found system proves query results over
*multiple independently-issued* signed graphs with graph-set privacy.

**VC ecosystem pragmatics.** W3C VC Data Model 2.0 with Data Integrity
proofs [VC-DI]; cryptosuites sign SHA-256 digests of RDFC10 N-Quads
[RDFC10]; `bbs-2023` [VC-DI-BBS] is the standardized selective-disclosure
suite; Bitstring Status List [VC-STATUS] is the standardized revocation
mechanism. Consequences for this design are worked through in §2.3/§2.6.

**Primitives (numbers used in §4/§6 cost models; v1's measured anchors,
kept).** Deeper/fresher proving-performance data — GPU/hardware provers,
proof systems beyond UltraHonk, ACIR portability — is the forthcoming
`zkp-performance-landscape.md`'s remit; nothing below should be re-derived
there, and nothing there is anticipated here.

- Poseidon2 permutation = **74 UltraHonk gates** (pinned constant in
  Barretenberg master [BBGATES]); Blake3 compression 2,159; SHA-256 6,703;
  ECDSA-secp256k1 verify 42,838.
- UltraHonk recursive verification of one UltraHonk proof inside an Ultra
  circuit = **~682 k gates** [BBGATES]; inside a Mega/Goblin circuit
  **11,848 gates**; `--scheme chonk` (ClientIVC) non-Aztec usability
  **unverified** [AZTEC-ROADMAP] [BBCLI].
- Lookup arguments: cq/logUp/Lasso considerations from v1 are now largely
  moot for the primary path (graphs are in-circuit wholesale); they return
  only for the server track and intra-circuit tables [CQ22] [LOGUP22]
  [LASSO23] [LOOKUPSOK25].
- Prover throughput: ~50 k gates/s native laptop (order-of-magnitude,
  [NOIRBENCH]); ~25–40 k gates/s in-browser bb.js on M-series; wasm 4 GB
  cap, practical browser ceiling ~2¹⁹–2²⁰ constraints [V8WASM] [NOIR2543].
  **The browser numbers are now the primary envelope** (holders prove
  client-side); refinements belong to the performance doc.
- Maintainable-commitment literature (Hyperproofs [HYPER22], BalanceProofs
  [BALANCE23], LVMT [LVMT23]): server-track only.

---

## 4. Candidate architectures

sparq's unique contribution is unchanged in kind: **the engine emits a query
trace** (per-row matched slot indices, scan boundaries, executed join
order), so witness generation is index arithmetic over structures sparq
maintains anyway — now per-named-graph and mostly trivial (flat-committed
graphs are their own witness). A `TraceSink` on the executor behind a
feature flag, zero cost when disabled.

### A. Monolith per-query circuit (reuse `sparql_noir`)

One compiled Noir circuit per query shape: recompute the k graph
commitments, verify k signatures, evaluate the query, expose `R` + claim
hash. **A's standing improves in v2**: per-query `nargo` compile was poison
for a server loop, but derived-credential query shapes are few and reusable
(an "over-18" proof is compiled once, cached, and proven per presentation
with fresh challenge), and the monolith's known 10⁶+-gate join-machinery
profile shrinks dramatically when the whole dataset is ~10² in-circuit
triples.

- **Cost model**: k=3 cooperative-issuer example (§2.4 layout): ~40 k gates
  hashing + 3 Schnorr **[judgement]** + query machinery ≈ **~10⁵ gates ⇒
  ~2 s native, ~3–5 s browser [judgement on cited throughput anchors]**.
  Standard-suite issuers (§2.3 cliff): +270–430 k gates *per credential* —
  native-only, minutes-territory at k=3 **[judgement]**.
- **Pros**: single proof object (a derived credential wants to *be* one
  artifact); existing coverage (167/236 conformance); single-circuit trust
  boundary. **Cons**: per-shape compile cache to manage; circuit-family
  sizing (k_max, n_max) multiplies shapes; no incremental story (fine
  here).

### B. Modular per-property proofs + verifier-side composition (extend `sparql_noir_modular`) — **recommended, unchanged**

Per-atomic-property circuits (measured: `filter_eq` 132 gates, `filter_lt`
2,925, `bgp_match` depth-8 1,410, `binding_consistency` 281; 5 proofs ≈
5.2 s prove / 3.2 s verify, manifest ~164 kB — HANDOFF-WAVE17), composed by
a plain verifier over the manifest. New module work v2 needs:
`graph_commit_recompute[k_max][n_max]` (flat-hash + signature-set check,
§2.4) replacing per-triple `bgp_match` Merkle openings for small graphs;
`bgp_match` retained for the Merkle fallback; everything downstream
(filters, binding consistency, claim hashes) reused as-is.

- **Cost model** (worked example, **[judgement]** arithmetic on cited
  constants): k=3 credentials (license 40 + ticket 15 + ticket 25 = 80
  triples), `k_max=4 / n_max=64` commit circuit ≈ 40 k gates + 3 Schnorr;
  query = 3 patterns × 3 rows + 1 hidden filter + consistency over the
  in-circuit slot array ≈ 5–10 k gates; total **~60–80 k gates across ~3–4
  module proofs ⇒ ~2–4 s native, browser-feasible**; verifier 3–4 verifies
  + manifest checks ~1–3 s. Completeness/NOT EXISTS: linear sweep over 256
  slots inside the commit-holding circuit — no extra proofs (contrast v1's
  per-scan adjacency pairs).
- **Aggregation, honestly (recalculated)**: Ultra-in-Ultra recursion at
  ~682 k gates per inner verify [BBGATES] still costs more prover time than
  it saves — but the derived-credential setting adds a real consumer for
  compression: a *re-presentable* derived credential (Q8) wants one small
  artifact. Posture unchanged: manifest-of-proofs first; recursion as
  opt-in compression; CHONK/Goblin (11.8 k gates inner verify) the
  watch-item — performance doc to adjudicate. (Q9)
- **Pros/cons**: as v1 (parallel proving, compile-once-ever modules,
  G1–G5 soundness programme, Lean story per-module / composition cons),
  plus: one new module family is the *entire* circuit delta for v2.

### C. Folding/IVC per-row predicates (Nova/HyperNova/Protostar, or CHONK)

**Downgraded further.** Folding's asymptotic edge needs long row streams
(≥10⁴ rows); derived credentials disclose a handful of rows over ~10²
triples. Reef's per-document folding [REEF24] remains the pattern to cite.
Verdict unchanged: not now; re-evaluate on CHONK general-Noir availability
(performance doc) or if dataset-dump credentials (§2.2 fallback) with huge
result sets materialise.

### D. zkVM re-execution (RISC Zero / SP1) — the baseline to beat

Unchanged: 23 triples `SELECT *` ≈ 7.5 min on M1 [CEUR4085]; modern GPU
zkVMs faster (**[judgement, vendor-influenced]** — performance doc will
pin) but per-instruction proving of an engine stays orders of magnitude
above circuit cost at credential scale. Keep as paper baseline; note the
baseline comparison is now *more* favourable since v2's circuits got
smaller while the zkVM's engine-execution floor did not.

**Architecture fit summary**: B recommended; A upgraded to a serious
single-artifact alternative (and stays the conformance oracle: same
witness, both provers, results must agree); C parked harder; D baseline.

---

## 5. The inference question (open — flagged for Jesse, not decided)

Unchanged in structure from v1, sharpened by the credential framing:
**I1 (commit the materialized closure) is now clearly unacceptable for the
primary path** — the issuer signed base facts, not the holder's closure;
a commitment over holder-side derived triples proves nothing to a verifier.
That was v1's own caveat; v2 makes it the default situation. So:

- **I2 — prove derivations on demand** is the natural fit and a genuine
  contribution: a derived row carries a derivation witness (rule id + base
  triple inclusions in the *signed* graphs + TBox inclusion — and "TBox"
  here means an ontology graph that is itself one of the signed/committed
  graphs in the union, possibly issued by a *different* authority: schema
  publishers become issuers in K). `sparq-reason`'s counting-engine
  `emit_consequences` and the N3 `ProofStep` trees are exactly the witness
  generators. Cost at credential scale: ≈ a few array lookups + ~10² gates
  per derived row **[judgement]**.
- **I3 — dual commitment** (base + closure, plus an epoch proof that one is
  the closure of the other) only makes sense holder-side if verifiers
  demand *completeness under entailment* ("no derivable answer missing"),
  which for k small graphs can alternatively be done by materializing the
  closure of the merge *inside the witness* and sweeping it — feasible at
  this scale, unlike v1's server setting **[judgement; needs a worked
  bound on closure size]**.

Recommendation deferred (Q2). Stage 1 ships with inference **off**.

---

## 6. Optimisation lever inventory ("optimise as much as possible")

Reordered for the v2 cost structure — at credential scale the dominant
costs are **signature verification and fixed per-proof overhead**, not
membership machinery.

1. **Flat full-graph recomputation for small graphs** (§2.2) — the v2
   headline lever. Kills Merkle paths, adjacency sentinels, and
   non-membership circuits in one move for the primary path; makes
   completeness a linear sweep. Per-graph Merkle only past the size
   break-even (measure it, stage 1).
2. **Signature scheme choice dominates the gate budget.** Schnorr-embedded
   ≈ 10³–10⁴ gates **[judgement]** vs ECDSA-secp256k1 42,838 [BBGATES] vs
   standard-suite RDFC10+SHA-256+non-native-curve ≈ 3×10⁵+ per credential
   **[judgement]**. This is where the cooperative-cryptosuite decision (Q3)
   is worth more than every other lever combined. Precise non-native-curve
   and alternative-proof-system numbers: `zkp-performance-landscape.md`.
3. **Hash split unchanged**: Poseidon2 in-circuit (74 gates), Blake3
   off-circuit, retire the Pedersen `h_2`/`h_4` default (Q10). Never hash
   strings in-circuit — which is exactly why the standard-suite cliff
   (§2.3) is a cliff.
4. **Batch modules to amortise fixed per-proof overhead (~1 s observed).**
   With total gates ~10⁵, the ~1 s/proof constant is the *largest single
   line item* — fold the whole union+signature check into one
   `graph_commit_recompute` circuit, and batch filter/consistency rows per
   module type; target ≤ 4 proofs per presentation. Keep every circuit
   under the browser ceiling (~2¹⁹) — **holder-side browser proving is the
   primary deployment**, not a nice-to-have. **[judgement]**
5. **Verifier-side checks (Jesse's principle, enforced, scope-corrected).**
   DISTINCT, ORDER BY, LIMIT/OFFSET, COUNT-over-disclosed, join edges over
   disclosed bindings: plain code over the manifest. The v1 ACL-scope item
   leaves this list — graph scope is now a private witness (§1.1); the
   *issuer-key set* and challenge are the verifier-checked publics.
6. **Trace-driven witness minimisation**: the executor trace indexes
   directly into the slot array; no re-evaluation, no second engine; the
   sparq wasm build does this in-browser next to bb.js.
7. **Ingest-time canonicalization amortisation**: RDFC10 + encoding +
   commitment once per credential at load, recorded in `<urn:sparq:zk>`;
   proving-time witness prep is pure lookup. (The v1 write-path
   amortisation story, relocated from group-commit batches to ingest.)
8. **Server-track levers, parked**: committed dictionary, inline-id
   arithmetic, sorted-leaf adjacency at scale, Hyperproofs-class
   maintainable openings, per-pod super-roots. Re-enter with stage 4 only.
9. **Aggregation only when it pays** (§4B): manifest now; recursion as
   compression for re-presentable derived credentials; CHONK watch-item.
   (Q9)

---

## 7. Recommended architecture and staged adoption

**Recommendation: B on per-named-graph commitments.** RDFC10-canonical,
Poseidon2 flat-hash commitments per credential graph (Merkle fallback for
large graphs), recorded with issuer signatures in `<urn:sparq:zk>` at
ingest; modular Noir proofs — one union-commitment/signature circuit plus
the existing property modules — produced holder-side (browser bb.js or
native sidecar) from sparq's executor trace; composition + disclosed-result
checks verifier-side; challenge-bound per presentation; recursion/folding
and the server track deferred behind explicit triggers.

### Stage 1 — holder flow end-to-end, zero engine impact

New optional crate `sparq-zk` (or a consumer in the zkp workspace — Q1)
using **existing public APIs only**: ingest 3–5 real-shaped W3C VCs
(cooperative-issuer mock: re-sign Poseidon2 commitments with Schnorr over
fixture keys), store content graphs + a hand-maintained `<urn:sparq:zk>`
registry, evaluate the query out-of-process, build witnesses, prove with
`sparql_noir_modular` extended by the `graph_commit_recompute` module, emit
manifest + challenge. No engine, wasm, or serve changes.
**Exit criteria**: (a) end-to-end derived credential — 3-pattern BGP + 1
hidden filter joining **k=2 credentials from different issuers** — at
≤ 5 s prove / ≤ 3 s verify native M-series, all circuits under 2¹⁹
constraints **[targets = judgement from the 5.2 s/3.2 s modular demo
anchor]**; (b) rejection tests: tampered triple, dropped row
(completeness), signature by a key ∉ K, same credential included twice
(§2.5 ordering), cross-graph bnode forgery; (c) measured flat-vs-Merkle
break-even table (§2.2); (d) zero diff in sparq's benchmark suite.

### Stage 2 — ingest bookkeeping + trace in the engine

(a) Ingest/write path computes per-named-graph RDFC10 + commitment and
maintains `<urn:sparq:zk>` (loader-synthesized, reserved-space-hardened,
mirroring `install_auth_view`), feature-gated; (b) executor `TraceSink`
(slot indices per matched row, scan boundaries), feature-gated, zero-cost
off; (c) prover consumes trace-fed witnesses; (d) wasm build exposes
witness export for in-browser proving.
**Exit criteria**: (a) ingest overhead with commitments on ≤ 10 % on a
credential-corpus load bench **[judgement target]**; (b) witness generation
≤ 10 ms per presentation at k ≤ 4 (vs re-evaluation); (c) browser
end-to-end (sparq-wasm evaluate + bb.js prove) under 15 s for the stage-1
query **[judgement]**; (d) feature-off builds byte-identical benchmarks.

### Stage 3 — union hardening, revocation prototype, honest benchmarks

k up to k_max=4 with padding; duplicate-ordering and bnode-scoping
adversarial tests promoted to CI; hidden-index status-list non-revocation
prototype (§2.6 option 1); NOT EXISTS / MINUS over the in-circuit sweep;
first honest benchmark table: v2-modular vs monolith-A (same witnesses,
conformance oracle) vs the zkVM baseline on identical derived-credential
queries.
**Exit criteria**: non-revocation adds ≤ 25 % prover time **[judgement]**;
≥ 10× beat vs the zkVM baseline on the 23-triple query and on a k=3
credential union; comparison table published.

### Stage 4 — research options, each behind a trigger

Standard VC-DI in-circuit verification or commitment-bridge (trigger: Q3
decision + `zkp-performance-landscape.md` findings on non-native-curve and
SHA-256 costs); recursion-tree compression / CHONK (trigger: re-presentable
derived credentials wanted, or documented non-Aztec ClientIVC); derivation
proofs I2 (trigger: Jesse's call on Q2); **server track** (v1's T1:
per-pod super-roots over per-graph commitments, generation-pinned proofs,
ACL-scoped completeness — trigger: an actual untrusted-server deployment
need); accumulator revocation (trigger: ecosystem convergence).

---

## 8. Honest risks

- **Composition soundness is still the crux of B.** G1–G4 closed, G5 in
  flight; the new `graph_commit_recompute` module adds union-layer
  obligations (slot-tag integrity, padding-sentinel non-matchability,
  ordering-constraint coverage) that must enter the G-series and ideally
  the Lean composition argument. Mitigation: A-as-oracle in CI.
- **The interop cliff is a product risk, not just a cost line.** If
  verifiers/issuers won't move off standard cryptosuites, every cost sketch
  in §4 triples-or-worse and browser proving dies (§2.3). The plan's bet is
  the custom-cryptosuite path (a); stage 4 holds the fallbacks. Decide
  early (Q3).
- **RDFC10 at ingest is attacker-facing**: pathological bnode structures
  blow up canonicalization; cap work per document at ingest (credentials
  are small; reject outliers, fail closed).
- **Privacy claims need a real analysis**: hiding σ/C(G) is necessary, not
  sufficient — k_max, query shape, timing, and R itself leak. The paper
  should scope its unlinkability claim precisely (cryptographic
  non-linkability of presentations to issuance, under disclosed-K).
- **bb/Noir churn** unchanged: nightly bb.js pin, beta toolchain, recursion
  "very much experimental" [BBREC]. The module boundary contains it.
- **Numbers are extrapolations** from ≤ 10⁴-quad measurements and v1
  anchors; flat-hash circuits, Schnorr-embedded costs, and browser
  end-to-end are unmeasured. Stage gates exist to falsify them early;
  `zkp-performance-landscape.md` will tighten the proving-side constants.
- **Dual evaluation drift** (stage 1 evaluates outside sparq): divergence
  would make proofs attest the wrong answer. Stage 2's trace removes the
  class.

## 9. Open questions for Jesse

Resolved by the v2 feedback (recorded, removed from the open list):
*v1 Q3* threat-model priority → holder-side derived credentials primary,
server track optional. *v1 Q4* leaf encoding → store-independent
term-hash/RDFC10 commitments (three-party trust boundary forces it;
rebuild cost is trivial at credential scale); id-leaves survive only in the
server track. *v1 Q5* bnode canonicalization → RDFC10 per graph at ingest,
graph-scoped bnode encoding (§2.2). *v1 Q10* scale ambition → eval at
credential scale (10¹–10³ triples/graph, k ≤ ~8), pod/server scale moves to
the optional track.

Open:

1. **Where does this live?** A `sparq-zk` crate here, or a consumer in
   `zkp-sparql-workspace` depending on sparq as a library? (Unchanged
   recommendation: commitment/trace/metadata seams in sparq, prover +
   circuits in the workspace — your call.)
2. **Inference semantics**: I1/I2/I3 (§5)? Is derivation-witnessing (I2,
   now over issuer-signed base graphs, with ontologies as signed graphs
   from schema-publisher issuers) an ISWC contribution or post-paper?
3. **Issuer-signature reality (the big one)**: target first — (a) custom
   VC-DI cryptosuite signing the Poseidon2 graph commitment (cheap, needs
   issuer adoption), (b) in-circuit standard-suite verification (no
   adoption needed, 3×10⁵+ gates/credential), or (c) a re-signing /
   BBS+-commitment bridge (new trust assumptions)? §2.3 lays out the
   trade; absorbs v1 Q6's scheme choice.
4. **Commitment content scope**: commit `credentialSubject` claims only, or
   the full VC envelope (issuer, validity window, type)? Where do
   validity-window checks live — in-circuit comparison against a disclosed
   "now", or disclosed dates checked verifier-side?
5. **Union sizing**: one `k_max/n_max` circuit with padding (leaks bounds
   only) vs a small circuit family (k ∈ {1,2,4,8} × n ∈ {16,64,256})?
   What envelope should the eval target?
6. **Cross-credential bnode identity**: confirm merge semantics (no bnode
   joins across graphs, §2.4) covers the intended use cases, or do any
   require issuer-side skolemization conventions?
7. **Revocation**: hidden-index status-list inclusion vs accumulators
   (§2.6); what freshness semantics do target verifiers demand, and does
   v1 of the eval include it at all?
8. **Holder binding & replay**: is challenge-binding per presentation
   enough, or should derived credentials be re-presentable artifacts with
   in-circuit holder proof-of-possession (which also strengthens the case
   for recursion-compression, Q9)?
9. **Aggregation posture**: manifest-of-proofs for the paper, or invest in
   the Ultra recursion tree / gamble on CHONK — now also weighing the
   re-presentable-derived-credential consumer (Q8)? Defer numbers to
   `zkp-performance-landscape.md`?
10. **Flip the spec's `h_2`/`h_4` default from Pedersen to Poseidon2**?
    Now lower-stakes: issuers sign fresh commitments under whatever suite
    we spec (Q3), so legacy-signed-dataset compatibility constrains less
    than v1 assumed.
11. **Timeline coupling**: does stage 1–3 feed the ISWC 2026 eval, or stay
    decoupled until after submission? Do the roborev/no-push constraints
    from the optimisation project apply to this module?
12. **Verifier trust establishment for K**: DID resolution / trust
    registries are out of circuit scope, but the manifest format must
    carry key references — adopt `did:` URLs + cryptosuite ids now?
13. **Metadata conventions**: exact naming for the stored VC-DI proof
    graph (`<D>?proof`-style convention vs registry-only retention in
    `<urn:sparq:zk>`), and the access rule for `<urn:sparq:zk>` itself
    (Control-holders only, mirroring `.acl` readability?).

## 10. Bibliography

- [VC-DI] W3C. Verifiable Credential Data Integrity 1.0.
  https://www.w3.org/TR/vc-data-integrity/
- [VC-DI-BBS] W3C. Data Integrity BBS Cryptosuites v1.0.
  https://www.w3.org/TR/vc-di-bbs/
- [VC-STATUS] W3C. Bitstring Status List v1.0.
  https://www.w3.org/TR/vc-bitstring-status-list/
- [RDFC10] W3C. RDF Dataset Canonicalization (RDFC-1.0).
  https://www.w3.org/TR/rdf-canon/
- [CL02] Camenisch, Lysyanskaya. Dynamic Accumulators and Application to
  Efficient Revocation of Anonymous Credentials. CRYPTO 2002.
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
`research/zkp-performance-landscape.md` (forthcoming — proving performance:
hardware, proof systems beyond Noir, ACIR reuse);
`zkp-sparql-workspace/{HANDOFF-WAVE17.md, decisions/sparql-noir-modular-alternative.md, notes/research/02,05,08}`;
`sparql_noir/spec/{encoding,algebra,proofs,preprocessing}.md`;
sparq `crates/{sparq-core/src/{dict,store}.rs, sparq-serve/src/{epoch,ring,writer}.rs, sparq-reason/src/{incremental,lib}.rs}`;
`research/{ARCHITECTURE.md, concurrent-serving.md §2.8–2.10, solid-access-control-design.md §2–3}`.
