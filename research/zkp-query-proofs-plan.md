# ZKP query-correctness proofs for sparq — derived credentials over issuer-signed named graphs (OPTIONAL module)

Status: **design v3 for review — nothing implemented, no code touched.**
Author: research agent, 2026-06-12 (v1), revised same day (v2, v3) per
Jesse's reviews. Reviewer: Jesse Wright.
Inputs: `research/zkp-noir-context.md` (reconstruction of the sparql_noir /
sparql_noir_modular line), sparq source (`sparq-core`, `sparq-serve`,
`sparq-reason`, `sparq-solid`), `research/concurrent-serving.md`,
`research/solid-access-control-design.md`, the zkp-sparql-workspace research
notes (reused, not re-derived), and fresh web research (June 2026). Every
uncited number is marked **[judgement]**. Open questions for Jesse are
numbered §9 and referenced inline as (Q*n*). Two companion documents landed
between v2 and v3 and are cited throughout: `zkp-performance-landscape.md`
(proving-performance synthesis — verdict: stay on Noir/UltraHonk/bb.js,
with Longfellow/Ligero and data-parallel GKR as named hedges) and
`zkp-noir-inventory-2026-06-12.md` (verified on-disk inventory of the Noir
estate — it corrects several memory-based assumptions, and this plan
defers to it wherever memory and disk disagree).

## v3 changelog (what changed since v2, and why)

Jesse's v2 review answered most of §9 and reset the target. The headline
items:

1. **Architecture fixed: B with a two-module split** — a `zk-trace` module
   in sparq that efficiently identifies the minimal input sets per
   per-property proof obligation, plus a composition package modeled on
   `sparql_noir_modular` (§4.E). Work proceeds **in the sparq repo**
   (`sparq-zk`); the zkp-sparql-workspace is frozen as an archive (Q1
   resolved).
2. **Noir dependency estate pinned against the verified inventory** —
   which repos become dependencies, the state change each needs (test-lib
   push = Jesse action; noir_XPath toolchain bump), and a recommendation
   on the IEEE754 float-API fork: finish PR-#39 now, defer the test_lib
   migration — grounded in test_lib's missing comparison ops (§4.E).
3. **Inference resolved: support both, recorded in the proof object** — an
   `entailmentRegime` field, ontology-graph commitments in the signed-input
   set, derivation witnesses from sparq's `proof-trees` branch, and the
   `sparql_noir_modular` extension surface spelled out (§5; Q2 resolved).
4. **Issuer-signature recommendation (Q3)**: custom VC-DI cryptosuite over
   the Poseidon2 commitment as primary (the live service is its own first
   issuer), in-circuit standard-suite verification as a per-credential
   interop fallback, Longfellow/Ligero as the hedge (§2.3).
5. **Commitment scope recommendation (Q4)**: commit the full VC envelope;
   validity-window checks in-circuit against a verifier-supplied "now"
   (§2.2).
6. **Union sizing is dynamic (Q5)**: a circuit *family* over a small
   (k, n) lattice; the verifier re-derives the required circuit id from
   the public statement and rejects mismatches; prover and verifier use
   different pinned artifacts per bucket (§2.4).
7. **Blank nodes are not skolemised (Q6, confirmed)**: protections in both
   prover and verifier against cross-graph bnode correlation — graph-scoped
   id domains in-circuit, a zk-trace plan check, a verifier re-derivation
   check, and per-graph salting of RDFC10 canonical labels (§2.2, §2.4).
8. **Revocation designed and promoted to v1-include (Q7)**: hidden-index
   status-list inclusion; accumulators as the upgrade path (§2.6).
9. **Holder binding explained (Q8)**: challenge-binding vs holder
   proof-of-possession are distinct concerns; both shipped as per-proof
   `binding` modes, use-case dependent (§2.5).
10. **Poseidon2 default confirmed (Q10)**: Pedersen `h_2`/`h_4` retired
    everywhere (§2.2, §6.3).
11. **ISWC framing dropped (Q11)**: the target is an actual live Solid
    server service; §7 gains an operational-posture section (async proving
    jobs, key rotation, generation-bound proof caching, versioned
    manifests, metrics) and the sparq house rules (roborev every commit,
    orchestrator merge-gates).
12. **Trust establishment (Q12)**: manifests carry `did:` key references +
    cryptosuite ids; the issuer set becomes a trust-framework reference —
    a signed issuer-registry graph, itself a named graph in the store
    (§2.8).
13. **Metadata conventions fixed (Q13)**: `<D>?proof` proof graphs
    inheriting the document ACL; an enriched `<urn:sparq:zk>` registry,
    Control-only readable (§2.1).
14. **§9 rewritten**: twelve questions move to the resolved block with
    one-line dispositions; the genuinely open remainder is aggregation
    posture (Q9, reframed), trust-framework specifics, test-lib completion
    ownership, and Jesse's veto window on the v3 recommendations.

Empirical-honesty notes carried from the inventory: the remembered "trace
module" **does not exist as code** anywhere in the Noir estate — what
exists is sparq's explain-analyze operator trace (rows/timings only) and
`compileQuery`'s AST walk; the zk-trace module is *new work* on those two
seams (§4). And `test_lib`, the preferred float library, currently has
**no comparison operators at all** — the single capability SPARQL FILTER
needs most (§4.E).

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
(replay binding, §2.5), and revocation status-list versions (§2.6 —
v1-include). Private
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
3. **The proving stack builds on Jesse's existing Noir repos** — but as of
   v3 the integration code lives *here*: a `sparq-zk` crate hosts the
   zk-trace module and the composition package, consuming the
   `sparql_noir_modular`-lineage circuits and the float/XPath libraries as
   clean pushed dependencies (§4.E). sparq supplies commitments, metadata,
   and traces; the Noir repos supply circuits.

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
trust establishment for issuer keys themselves (DID resolution and
registry operation — the manifest's key references and the trust-framework
hook are specced in §2.8 (Q12); the resolution machinery is not).

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
  graph **`<D + "?proof">`** (Q13 — resolved convention: a query-string
  suffix on the document IRI, mirroring the document-location convention
  for the content itself; addressable, collision-free against real pod
  resources, trivially derivable from `<D>`), **inheriting the ACL of
  `<D>`**, and is **excluded from the dataset the proof statement
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
      zk:cryptosuite     zk:poseidon2-schnorr-v1 ;
      zk:issuerKey       <did:example:dmv#key-1> ;
      zk:signatureGraph  <https://dmv.example/vc/lic-123?proof> ;
      zk:statusList      <https://dmv.example/status/3#94> ;
      zk:rdfc10Salt      "0x9f3c…"^^zk:field ;
      zk:status          zk:active ;
      zk:ingested        "2026-06-12T…"^^xsd:dateTime .
  ```

  Same hardening rules as `<urn:sparq:auth>`: `urn:sparq:zk` is in the
  reserved IRI space, pre-existing copies are stripped on load, only the
  ingest path writes it, and it is excluded from query-visible datasets
  (the verifier never sees it; the prover reads it as its witness index).
  Access rule (Q13 — resolved): `<urn:sparq:zk>` is readable only by
  **Control-holders of the referenced documents** (mirroring `.acl`
  readability in the access-control design), and proof graphs inherit the
  ACL of their document. The individual commitments are *not* secret —
  they are issuer-signed public values — but the registry as a whole is
  the catalogue of every credential the holder possesses, the most
  sensitive inventory in the store; hence Control-only.

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
`h_s` = Blake3 off-circuit and `h_2` = Poseidon2 (the spec's Pedersen
default is retired — Q10 resolved, §6.3). **Bnode encoding gains graph
scoping and a per-graph salt**: encode a blank node as
`h_2(BlankNode_code, h_2(salt_G, blake3(canonical_label)))`, where
`salt_G` is a per-graph salt fixed at issuance/ingest and recorded as
`zk:rdfc10Salt` in `<urn:sparq:zk>` (§2.1). The salt matters because
RDFC10 produces *canonical* labels (`c14n0`, `c14n1`, …) that recur across
unrelated graphs: unsalted, identical canonical labels in different graphs
would hash to equal leaf components, and equal leaf-level material across
commitments is exactly the cross-graph correlation channel Q6 closes
(§2.4). Salted, bnodes from different graphs are *distinct by
construction*, which is precisely RDF **merge** semantics — see §2.4.

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

**Commitment scope (Q4 — resolved by recommendation, pending Jesse veto).**
Commit the **full VC envelope** — issuer, validity window, credential
type, and `credentialSubject` — not the claims alone. Rationale:
validity-window and type checks are things verifiers will demand *proven*,
and they are only cheap if the relevant terms are inside the commitment.
Validity windows are checked **in-circuit against a disclosed "now"**: the
verifier's challenge (§2.5) carries a timestamp, and the circuit
range-checks `validFrom ≤ now ≤ validUntil` over the committed values —
two field comparisons, noise next to the hashing budget — **without
disclosing the credential's actual dates**, which would otherwise be a
fingerprinting channel (issuance dates are nearly unique per credential).
Claims-only commitments are recorded as a **non-goal**: they save a
handful of triples per credential and forfeit provable validity and type.

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
  k=1 flows.

**Q3 — resolved by recommendation, pending Jesse veto: target (a) first,
keep (b) as a flagged fallback.** The adoption argument has changed now
that the target is a live Solid service rather than an ISWC eval (Q11):
the service controls both ends initially — it can act as its **own first
issuer** (signing pod-resident facts and service-issued attestations as
credentials under the custom cryptosuite) and verify its own suite, so the
cold-start problem that made (a) look optimistic in v2 does not block
launch; third-party issuers join one at a time. Keep **(b) in-circuit
standard-suite verification behind a per-credential flag** as the interop
fallback: a credential whose registry entry carries a standard cryptosuite
routes to the expensive circuit family (~270–430 k gates/credential,
native-only), everything else stays on the cheap path — the flag is paid
per credential, not by the whole presentation. For exactly that hash-heavy
case, `zkp-performance-landscape.md` (§2.7, Hedge 1) names the hedge:
**Longfellow-zk/Ligero** [LONGFELLOW] — shipping in Google Wallet, proving
ECDSA-P256+SHA-256 credential statements in hundreds of ms on phones — is
the system to spike *before* writing any in-circuit SHA-256. The
re-signing bridge and BBS+ commitment bridge remain stage-4 options
unchanged.

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
*not* be identified. Jesse confirms (Q6): **blank nodes are not
skolemised**, so accidental cross-graph correlation must be engineered
out, in *both* prover and verifier. Four layers:

1. **In-circuit, by construction**: bnode identifiers are effectively
   *(graph, local-id)* pairs — the §2.2 salted, graph-scoped encoding
   means equal local labels in different graphs *cannot* produce equal
   field elements, so they cannot unify in any join the circuit evaluates.
   Cross-graph joins happen on IRIs and literals only (the only terms with
   cross-credential identity) — which is semantically correct merge
   behaviour.
2. **Prover-side plan check**: the zk-trace module (§4.E) rejects, at
   witness-build time, any query plan that would join on a bnode-valued
   variable across graph boundaries — failing closed with a diagnostic
   rather than emitting a proof whose join silently matches nothing (or
   leaks, via shape or timing, which graphs were tried).
3. **Verifier-side re-derivation check**: the verifier re-runs the same
   static check over the manifest (it already re-derives obligations from
   the query text, §4.E) and rejects any manifest whose join edges assume
   a cross-graph bnode equality. Prover and verifier enforce the rule
   independently; a malicious prover cannot smuggle a correlating join
   past an honest verifier.
4. **Commitment-layer salting** (§2.2): RDFC10's per-graph canonical
   labels recur across graphs (`c14n0` appears everywhere), so leaf hashes
   are salted per graph — identical canonical labels in different graphs
   never yield equal leaf hashes, closing the correlation channel *below*
   the join layer too (an issuer–verifier coalition comparing leaf-level
   material across commitments learns nothing).

If a use case genuinely needs cross-credential bnode identity it needs
issuer-side skolemization (mint IRIs at issuance) — an issuer convention,
not circuit machinery.

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

**Union sizing is dynamic (Q5 — resolved).** Jesse asked whether the
sizing can be dynamic; it can — as a circuit **family**, not a
dynamically-shaped circuit (UltraHonk circuits are fixed-shape): compile
the commit/union circuit over a small (k, n) lattice — **k ∈ {1, 2, 4, 8}
× n ∈ {16, 64, 256}** — and have the proof manifest carry the **circuit
identifier** of the bucket used. Prover and verifier therefore use
*different artifacts per bucket*, exactly as Jesse anticipated:
verification keys are pinned per family member and distributed
**content-addressed in the manifest format**. The soundness rule is that
the verifier **re-derives the required circuit id from the same public
statement** (the declared k, the query shape, the disclosed result) and
rejects mismatches — so a prover can neither pick a smaller-padding bucket
to leak less than the statement implies nor a larger one to hide more
graphs than the statement admits. What padding leaks is only the envelope
bucket — a (4, 64) proof says "at most 4 graphs of at most 64 triples",
nothing finer — and that is the deliberate trade. Operationally, the live
server (Q11) compiles family members **lazily and caches them**; the
12-member lattice keeps the artifact set enumerable, and the largest
member ((8, 256) ⇒ 2,048 slots ≈ ~300 k gates of hashing **[judgement,
same per-triple anchor]**) is native-only by design — browser flows live
in the small buckets.

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
  were the block index — so the scope tag must be graph-derived (§2.2's
  per-graph salt, fixed at ingest, never the block index), making
  double-inclusion idempotent on bnodes; and the circuit
  additionally enforces strict ordering `C(G_1) < C(G_2) < … < C(G_k)`
  (numeric, on the field representative) to force pairwise-distinct graphs.
  Without this, COUNT-style derived claims ("I hold ≥ 2 tickets") are
  forgeable by including one ticket twice. Aggregates are not in the v1
  fragment, but the ordering constraint is ~free and closes the class now.
- **Replay / holder binding (Q8 — explained; resolved as
  use-case-dependent, both modes supported).** Jesse asked what this
  question is really about, and whether it is proof of knowledge of a
  signature. It is **two distinct concerns** that v2 folded into one:

  - **Challenge-binding** answers *replay*: the verifier's challenge
    (nonce + audience + timestamp — the same "now" that §2.2's validity
    check consumes) is a public input baked into every constituent proof's
    claim hash, making the proof **single-use**. Anyone replaying the
    manifest fails the next verifier's fresh nonce. Cost: nothing
    in-circuit beyond hashing the challenge into the claim. This makes
    holder-side proving latency the binding UX constraint, which the
    small-circuit envelope is sized for.
  - **Holder proof-of-possession** answers *theft/transfer*: the
    credential carries the holder's public key in the VC `cnf`
    (confirmation) claim, and the circuit proves knowledge of the matching
    secret — so yes, Jesse's reading is right: this is **proof of
    knowledge of a signature/secret** (a Schnorr-style PoK over the
    embedded curve, the same cost class as one issuer-signature check).
    It is what makes a derived credential *re-presentable*: an artifact
    with its own lifetime that only the legitimate holder can present.

  Which a presentation needs is **use-case dependent, exactly as Jesse
  says** — so the proof object carries a **`binding` mode**:
  `binding: challenge` for one-shot interactive presentations (the live
  service's default) and `binding: holder-pop` for re-presentable derived
  credentials (also the consumer that makes recursion-compression
  interesting, Q9). The two compose: a holder-pop derived credential is
  still challenge-bound at each presentation.

### 2.6 Revocation (Q7 — resolved by recommendation: hidden-index status list, v1-include)

Issuer-signed commitments are forever; real credentials get revoked. The VC
ecosystem's standard answer — Bitstring Status List [VC-STATUS] — has the
verifier fetch a list and check an index, which is unusable here directly
(the index identifies the credential). **v1 design — hidden-index
status-list inclusion**, the concrete proposal Jesse asked for:

- The issuer maintains a **signed status-list credential**: a bitstring
  (bit *i* = revocation state of the credential issued with status index
  *i*), committed as a Poseidon2 Merkle tree over fixed-size chunks, with
  the root plus a version/timestamp signed by the issuer.
- Each credential's committed envelope (Q4, §2.2) carries its status
  index. The circuit proves, **without revealing the index**: Merkle
  inclusion of the chunk containing the index against the **disclosed**
  status-list version/commitment, plus an in-circuit bit-extraction check
  that the bit is 0. Cost: one Merkle path + bit arithmetic ≈ **a few
  thousand gates per credential [judgement]** — small next to the
  signature checks.
- **Freshness is verifier-side and in the clear**: the issuer's signed
  timestamp on the status-list version is disclosed, and the verifier
  demands a commitment **no older than its policy window** (24 h for
  tickets, 30 days for diplomas — its call). No circuit cost; freshness
  policy is the verifier's business, not the proof's.
- The registry (§2.1, `zk:statusList`) records each credential's
  status-list reference; the live service refreshes status lists in the
  background so proving never blocks on an issuer fetch.

**Why v1-include rather than a stage-4 option**: the target is a live
service (Q11). A service issuing derived credentials without revocation
cannot honestly claim those credentials are trustworthy — the first
suspended driver's license that still proves "valid license" is a
product-killing failure, not a research footnote.

**Upgrade path, explicitly deferred**: dynamic universal accumulators
(CL02 [CL02], AnonCreds-style pairing/VB accumulators) give non-membership
without list structure and stronger privacy at scale, at real costs:
issuer-maintained witness updates pushed to every holder on every
revocation, heavier non-native arithmetic on BN254, and a less
standardized trust story. Adopt only if status lists break (revisit
trigger: an issuer with >10⁶ outstanding credentials or sub-hour
revocation SLAs). Either way the statement S conjoins per-credential
non-revocation at a disclosed list version.

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

### 2.8 Verifier trust establishment and the manifest's key material (Q12 — resolved)

Yes (per Jesse): the manifest carries **key references as `did:` URLs plus
cryptosuite identifiers** — never bare key bytes. Verification resolves
the DID document, extracts the verification method, and checks the
cryptosuite id against the circuit family the manifest names.
DID-method-agnostic by design; **`did:key` and `did:web` are the required
minimum** (did:key for fixtures and offline tests; did:web because every
issuer in the Solid setting already has a web origin).

**Trust frameworks (Jesse is actively thinking about this — a starting
shape, not a final design).** The disclosed issuer set `K` should not be a
bare key list the verifier eyeballs; express it as a **trust-framework
reference**: a signed **issuer-registry graph** — itself an ordinary named
graph in the store, with its own commitment and signature, **dogfooding
exactly the credential model this plan builds** — listing member issuer
DIDs, their roles, and key-validity windows. The manifest then asserts
"all k signatures verify against keys in framework F (registry-graph
commitment X, signed by framework operator Y)", and the **verifier's
policy decides which frameworks it accepts** — the proof attests
membership; the verifier attests trust. This keeps trust establishment out
of circuit scope: the in-circuit check stays `pk_i ∈ K`, and K's
*derivation* from F starts verifier-side (it can move in-circuit later if
framework-membership privacy is ever needed). Specifics — framework
vocabulary, registry update semantics, cross-framework bridging — remain
an open design area (§9, open item 2), to be co-designed with Jesse's
trust-framework thinking.

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
kept).** The proving-performance survey has since landed
(`zkp-performance-landscape.md`); its verdict: **stay on
Noir/UltraHonk/bb.js**. The neutral M1 benchmark (PSE csp-benchmarks) puts
Barretenberg at 610 ms for a comparable-size hash circuit; in-browser
bb.js has independently proven ~2M-constraint circuits in <3 s on an M1
Air (~20× our budget); GPU, zkVM, and Nova-family folding alternatives are
disqualified *by measurement* at this scale; and Longfellow-zk/Ligero is
the named hedge for the standard-suite interop cliff (§2.3). The constants
below are unchanged by it.

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
feature flag, zero cost when disabled. Honesty note, per
`zkp-noir-inventory-2026-06-12.md` §d: the "trace module" remembered from
the workspace **does not exist as code anywhere** — what exists today is
sparq's explain-analyze operator trace (`crates/sparq-engine/src/exec.rs`,
`pub(crate) mod trace`: per-operator labels, row counts, timings — *not*
proof-input sets) and `sparql_noir_modular`'s `compileQuery` AST walk. The
zk-trace module specced in §4.E is **new work grafted onto those two
seams**, not a port of something that exists.

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
  compression: a *re-presentable* derived credential (`binding:
  holder-pop`, §2.5) wants one small artifact. Posture unchanged:
  manifest-of-proofs first; recursion as opt-in compression; CHONK/Goblin
  (11.8 k gates inner verify) the watch-item —
  `zkp-performance-landscape.md` adjudicated: CHONK is **confirmed
  unusable for non-Aztec Noir programs as of mid-2026**, so
  manifest-of-proofs stands until its revisit triggers fire. (Q9)
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
zkVMs are faster only on datacenter hardware — `zkp-performance-landscape.md`
pinned the client-side anchor (RISC0: 18.5 s + 1.47 GB for a *single small
hash* on M1, independent csp-benchmarks) — so per-instruction proving of
an engine stays 2–3 documented orders of magnitude above circuit cost at
credential scale. Keep as paper baseline; note the
baseline comparison is now *more* favourable since v2's circuits got
smaller while the zkVM's engine-execution floor did not.

**Architecture fit summary**: B recommended; A upgraded to a serious
single-artifact alternative (and stays the conformance oracle: same
witness, both provers, results must agree); C parked harder; D baseline.

### E. Chosen shape (v3): B as a two-module split — `zk-trace` + composition package

Jesse's v2 review fixes the shape: "a zero knowledge specific module —
similar to the trace module — which efficiently identifies the sets of
inputs that need to go into the per-property proofs per [architecture B].
Then this package can be something similar to the existing 'modular'
package to do the property proofs and compose a full proof."

**(i) `zk-trace` — input identification (new module in sparq).** Given an
*executed* query, it identifies the **minimal input set per per-property
proof obligation**. It grafts onto two existing seams: the explain-analyze
operator trace in `crates/sparq-engine/src/exec.rs` (which today records
only `{label, depth, rows, nanos}` per operator) and the Stage-2 executor
`TraceSink` this plan already designs. What it records per execution:

- **per-row matched leaf/slot indices** — for each disclosed result row
  and each triple pattern, which slot(s) in which graph's witness block
  matched;
- **scan boundaries** — the slot ranges each scan actually swept (the
  completeness witness: the linear-sweep circuits must know what
  "everything" was);
- **the executed join order** — so manifest edges mirror the real plan,
  not a re-derived one;
- **per-obligation witness sets** — the resolved deliverable: for every
  obligation the obligation compiler will emit (one filter application,
  one pattern match, one consistency edge, one derivation step §5, one
  non-revocation check §2.6), exactly the slot indices, salts, and
  signature selectors its witness builder needs, deduplicated across
  obligations;
- the **cross-graph bnode-join check** (§2.4, layer 2), failing closed at
  witness-build time.

Witness building then is array indexing over the `<urn:sparq:zk>` registry
plus the trace — no re-evaluation, no second engine — and it runs in the
browser via the sparq wasm build next to bb.js.

**(ii) The composition package — modeled on `sparql_noir_modular`.** The
inventory (§e) describes the existing architecture precisely, and it is
the right one — **extend, don't reinvent**: per-property circuits (one
tiny bin package per atomic obligation, poseidon-only deps);
**proof-vs-clear dispatch** (hidden operand → ZK proof, all-disclosed →
plain check — the repo's stated architectural innovation, kept); **manifest
composition** (`ProofManifest {disclosed, modules, edges}`; joins / UNION /
OPTIONAL as plain-checked edges); and **verifier re-derivation** (re-run
obligation compilation from the query text, verify every proof, recompute
public-input hashes, enforce complete-cover, re-classify to reject
proof→clear downgrade attacks). v3 deltas: the deterministic obligation
enumerator (`compileQuery`'s contract) stays the shared prover/verifier
TCB, but its *input identification* is fed by zk-trace's execution-derived
witness sets instead of an AST walk; new module families
`graph_commit_recompute[(k,n)]` (§2.4 lattice), `status_nonrevoked`
(§2.6), and `derivation_step` (§5) join the existing
`filter_*`/`bgp_match`/`binding_consistency` set; and the manifest gains
the circuit-id, `entailmentRegime`, `binding`-mode, and `did:`
key-reference fields specced across §2.

**(iii) Where the work lives (Q1 — resolved by Jesse).** In **this repo**:
a new `sparq-zk` crate (zk-trace, witness building, manifest types; the
TS/wasm prover-verifier surface starts as a package under it). Jesse: "the
zk-sparql-workspace got a little out of control … I would like to pick up
work here." The inventory (§f) confirms the diagnosis — ~27 working copies
in one repo, every checkout on a different feature branch, a fragile
`../../../../` cross-repo path dependency, unpushed crown jewels — so the
**workspace is frozen as an archive** (its notes, decisions, and Lean
proofs remain citable; no new development there), and the Noir libraries
are consumed as **clean pushed git dependencies, pinned by tag**. Required
state changes per dependency, from the inventory:

| Repo | Role | Required state change |
|---|---|---|
| `jeswr/sparql_noir_modular` | composition-package ancestor: circuits + TS prover/verifier patterns | confirm `v0.4-g5-filter-ne-lang` is merged to main; tag a release; extend per (ii) |
| `jeswr/test-lib` (pkg `test_lib`) | the better-abstractions float library (comptime-generated `f16`–`f128`, private fields, committed gate baselines) | **JESSE ACTION** (private repo; only he can): push the **76 unpushed commits** and commit the dirty `src/ops/kernels.nr` — the newest float work currently exists on one machine only |
| `jeswr/noir_XPath` | XPath 2.0 F&O for FILTER semantics | toolchain bump **beta.16 → current** (26 `nargo check` errors on beta.21 today; its version-check CI fails daily) + the float-API decision below |
| `jeswr/noir_IEEE754` | the pushed, complete float library (v0.3.1 tag; new `Float<E,M,RM>` API on branches) | land the new Float API on main and tag it — the PR-#39 blocker |
| `noir-lang/poseidon` v0.1.1 | the modular circuits' only dependency | none |
| `jeswr/sparql_noir` (monolith) | conformance oracle (architecture A) | none required; pin GitHub main (pushed 2026-05-23); ignore the stale local copies |

**The float-API fork — recommendation: finish PR-#39 now; defer the
test_lib migration.** Jesse flags that the best IEEE754 library is not the
pushed one but the one with better abstractions, living "under a folder by
a completely different name" — the inventory resolves it:
`~/Documents/GitHub/jeswr/test-lib`, package `test_lib`. He is right about
the abstractions (truly private fields, generated `f64`-style global type
names, f128 support, committed gate baselines + regression harness —
inventory §b). But the inventory's honest gap is decisive for
*sequencing*: **`test_lib` has no comparison operators at all** (no
Eq/Ord/lt anywhere), no sqrt, no rounding-mode surface in the public API,
and ~22 tests against noir_IEEE754's 44 MPFR-oracle packages — and
comparison predicates are precisely what XPath/FILTER consumes most (the
whole `filter_lt/gt` family; `filter_lt` is already the costliest shipped
module at 2,925 gates). Migrating XPath to test_lib today means writing
the very kernels FILTER needs before any proof can run, plus rewriting
~1.4 k lines of `numeric_types.nr` glue and regenerating float test
packages (inventory §c's estimate: a few hundred lines of new kernels +
tests, plus the glue rewrite). The competing path — **PR #39, migrating
noir_XPath to noir_IEEE754's new `Float<E,M,RM>` API — is already ~done**
on the workspace branch, blocked only on landing the API on noir_IEEE754
main and replacing a TODO relative-path dep with a tag. So: **ship PR-#39
first**, and treat **test_lib as the long-term destination** once it gains
comparisons / rounding-to-integral / casts — at which point the entire
XPath float surface is one file (`numeric_types.nr`, the only place
IEEE754 is referenced) and the second migration is cheap. This sequencing
reads against the grain of Jesse's preference for test_lib; the
measurement (missing comparators vs an almost-finished PR) is why, and it
is his call to override (§9, open item 3).

---

## 5. Inference (Q2 — resolved: support both, recorded in the proof object)

Jesse's call: **proofs must work with and without inference, and which one
applies is part of the proof object.** Concretely:

- The manifest carries an **`entailmentRegime` field**:
  `none | rdfs | owl-rl | n3`. `none` remains the stage-1 scope. The
  verifier's re-derivation (§4.E) treats the regime as part of the public
  statement: a proof under `rdfs` is *not* interchangeable with one under
  `none`, and the verifier's policy decides which regimes it accepts.
- **When the regime ≠ `none`, ontology graphs join the signed-input set**:
  their commitments enter the union exactly like credential graphs —
  schema publishers become issuers in `K` (v2's I2 insight, kept). A
  derivation is only as trustworthy as its TBox, and the proof says whose
  TBox it was.
- **The derivation witness comes from sparq's `proof-trees` branch**
  (feature `explain`, in `sparq-reason`): `Materialized*Graph::why(triple)`
  returns a `ProofTree` that is **flat, id-based, and deterministic** — a
  `Vec<ProofNode {conclusion, rule, premises}>` in
  premises-before-conclusion order, root last, shared sub-proofs
  deduplicated (a DAG), every internal choice point iterated sorted. That
  is exactly the linear-scan witness shape a derivation circuit wants —
  the branch's module docs were written against this plan's §5, and the
  shape was verified on the branch for v3. zk-trace (§4.E) pulls one
  `ProofTree` per derived row in the result and translates node indices to
  witness slot indices.
- **The `sparql_noir_modular` extension surface** (the inventory §e
  confirms the modular repo has *zero* inference code today and documents
  the drop-in extension pattern, so this is exactly the extension Jesse
  anticipated): one new property module, **`derivation_step`**, verifying
  a single rule application — public inputs anchor (rule id, premise
  slot/row hashes, conclusion hash) into a claim hash; the circuit checks
  the conclusion follows from the premises under the identified rule, with
  rule semantics as a small in-circuit table per supported regime — plus a
  witness builder in `src/modules/`, and an obligation-compiler
  classification branch emitting one `derivation_step` obligation per
  `ProofNode` of each derived row, with `binding_consistency`-style edges
  chaining premises to earlier conclusions. Cost at credential scale: a
  few array lookups + ~10² gates per node **[judgement, as v2]**. Proof
  construction is capped engine-side
  (`ExplainOpts {max_depth: 128, max_nodes: 65 536}`); realistic
  credential derivations are a handful of nodes.
- **I1 (commit the materialized closure) stays unacceptable** for the
  primary path — the issuer signed base facts, not the holder's closure.
  **I3-style completeness under entailment** ("no derivable answer
  missing") is retained as an opt-in for verifiers that demand it:
  materialize the closure of the merge *inside the witness* and sweep it —
  feasible at credential scale **[judgement; a worked bound on closure
  size is still owed]** — and it is the natural reading of completeness
  when `entailmentRegime ≠ none`.

Stage 1 still ships `entailmentRegime: none`; `derivation_step` lands in
stage 3 (§7).

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
3. **Hash split unchanged; Pedersen retired (Q10 — resolved: yes, flip
   everywhere)**: Poseidon2 in-circuit (74 gates), Blake3 off-circuit.
   The flip is now safe and worth taking: issuers sign fresh commitments
   under the suite we spec (§2.3), so the legacy-compatibility constraint
   is gone; Poseidon2 is the cheapest in-circuit hash in UltraHonk; and
   one uniform hash across `h_2`/`h_4`/commitments removes a whole class
   of cross-suite bugs. Never hash strings in-circuit — which is exactly
   why the standard-suite cliff (§2.3) is a cliff.
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

**Recommendation: B on per-named-graph commitments, in the §4.E two-module
shape (`sparq-zk` = zk-trace + composition package).** RDFC10-canonical,
Poseidon2 flat-hash commitments per credential graph (Merkle fallback for
large graphs), recorded with issuer signatures in `<urn:sparq:zk>` at
ingest; modular Noir proofs — one union-commitment/signature circuit plus
the existing property modules — produced holder-side (browser bb.js or
native sidecar) from sparq's executor trace; composition + disclosed-result
checks verifier-side; challenge-bound per presentation; recursion/folding
and the server track deferred behind explicit triggers.

### Stage 1 — holder flow end-to-end, zero engine impact

New optional crate `sparq-zk` in this repo (Q1 — resolved; the workspace
stays frozen, §4.E) using **existing public APIs only**: ingest 3–5 real-shaped W3C VCs
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

Circuit-family buckets beyond the stage-1 default (§2.4 lattice, lazy
compile + cache, verifier-side circuit-id re-derivation);
duplicate-ordering and bnode-scoping adversarial tests promoted to CI
(including the §2.4 prover/verifier cross-graph-bnode checks); hidden-index
status-list non-revocation prototype (§2.6 — v1-include, so this stage is
its landing slot); first `derivation_step` module +
`entailmentRegime: rdfs` end-to-end over a `proof-trees` witness (§5);
NOT EXISTS / MINUS over the in-circuit sweep; first honest benchmark
table: v3-modular vs monolith-A (same witnesses, conformance oracle) vs
the zkVM baseline on identical derived-credential queries.
**Exit criteria**: non-revocation adds ≤ 25 % prover time **[judgement]**;
≥ 10× beat vs the zkVM baseline on the 23-triple query and on a k=3
credential union; comparison table published.

### Stage 4 — research options, each behind a trigger

Standard VC-DI in-circuit verification behind the per-credential flag, or
the commitment-bridge (trigger: first real third-party-credential demand —
run the Longfellow/libzk spike first, per `zkp-performance-landscape.md`
Hedge 1); recursion-tree compression / CHONK (trigger:
`binding: holder-pop` re-presentable credentials wanted, or documented
non-Aztec ClientIVC); I3-style completeness under entailment (trigger: a
verifier demands it, §5); **server track** (v1's T1: per-pod super-roots
over per-graph commitments, generation-pinned proofs, ACL-scoped
completeness — trigger: an actual untrusted-server deployment need);
accumulator revocation (trigger: §2.6's scale/SLA thresholds).

### Live-service posture (Q11 — resolved: ISWC framing dropped)

Jesse: "I no longer care about ISWC, this is about building this for an
actual Solid server that is a live service." Consequences, spelled out:

- **Proof generation is an async job, not a blocking request.** Even 2–5 s
  of proving does not belong on a request thread; the service exposes
  create-job / progress / fetch-result, and the browser path mirrors it
  with a worker + progress events. SLO thinking applies from day one:
  queue depth, per-bucket latency percentiles, and a warm cache of circuit
  artifacts (§2.4's lazy-compiled family members).
- **Key rotation and issuer-key history.** Issuers rotate keys; proofs
  must verify against the key that was valid **at issuance time**, not at
  resolution time. The registry records the key reference per credential
  (§2.1); DID-document key history — or the trust-framework registry graph
  (§2.8), which carries key-validity windows — supplies the history; the
  verifier checks key-valid-at-issuance in the clear.
- **Proof caching is generation-bound.** A proof attests a result over
  specific graph commitments, so sparq's generation/time-travel machinery
  is the invalidation index: a cached proof is bound to the generation(s)
  of its constituent graphs and is invalidated when a write touches any of
  them — per-graph commitment maintenance in the apply path (§2.7) is
  exactly the hook. Status-list freshness (§2.6) invalidates on its own
  clock, independent of graph writes.
- **Versioned manifest format with a compatibility policy.** Manifests
  carry a format version, circuit-family ids, and cryptosuite ids
  (content-addressed VKs, §2.4); the service commits to verifying N−1
  manifest versions across upgrades; and circuit-id pinning means a
  bb/Noir toolchain bump produces a *new* family version, never a silent
  change — the bb-churn risk (§8) becomes an operational procedure instead
  of an ambush.
- **Operational metrics**: proving time by bucket and stage, witness-build
  time, verification failures by cause (signature / completeness /
  re-classification / binding / staleness), status-list refresh lag,
  artifact-cache hit rate.
- **House rules**: this is sparq-repo work — roborev review on every
  commit and orchestrator-controlled merge gates apply, as for the rest of
  the repo; the workspace freeze (§4.E) is part of the same discipline.

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
  the custom-cryptosuite path (a) — now the standing recommendation
  (§2.3), softened by the live-service setting (the service is its own
  first issuer) and hedged by the Longfellow/libzk spike; stage 4 holds
  the fallbacks.
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
  `zkp-performance-landscape.md` tightened the system-level constants but
  explicitly notes no published benchmark covers our exact workload —
  stage 1 measures it.
- **Dual evaluation drift** (stage 1 evaluates outside sparq): divergence
  would make proofs attest the wrong answer. Stage 2's trace removes the
  class.
- **The float-library estate is a live dependency risk.** The best float
  work (`test_lib`) sits unpushed on one machine (76 commits + a dirty
  `kernels.nr` — Jesse action, §4.E) and lacks the comparison kernels
  FILTER needs most; noir_XPath does not compile on the current toolchain
  (26 errors on beta.21; daily-failing version-check CI). Until PR-#39
  lands and the toolchain is bumped, FILTER-over-floats has no clean
  dependency path — which is exactly why §4.E sequences it first.
- **Live-service surface is new risk territory**: key-rotation mistakes,
  stale status lists, and cache-invalidation bugs are *soundness* failures
  as experienced by verifiers, not lab artifacts. §7's operational posture
  is the mitigation, and it needs the same adversarial-test discipline as
  the circuits.

## 9. Open questions for Jesse

**Resolved by the v2 feedback (recorded):** *v1 Q3* threat-model priority →
holder-side derived credentials primary, server track optional. *v1 Q4*
leaf encoding → store-independent term-hash/RDFC10 commitments; id-leaves
survive only in the server track. *v1 Q5* bnode canonicalization → RDFC10
per graph at ingest, graph-scoped bnode encoding (§2.2). *v1 Q10* scale
ambition → credential scale primary; pod/server scale optional track.

**Resolved by the v3 review (Jesse's answers + the recommendations he
asked for — dispositions one line each):**

- **Q1 (where it lives)** → resolved by Jesse: in the sparq repo
  (`sparq-zk`, §4.E); the zkp-sparql-workspace is frozen as an archive.
- **Q2 (inference)** → resolved by Jesse: support **both**, recorded in
  the proof object as `entailmentRegime`; ontology commitments join the
  signed-input set; `sparql_noir_modular` gains `derivation_step`, with
  `ProofTree` from the `proof-trees` branch as witness source (§5).
- **Q3 (issuer signatures)** → resolved by recommendation, pending veto:
  custom VC-DI cryptosuite over the Poseidon2 commitment primary (the
  live service is its own first issuer); in-circuit standard-suite
  verification as a per-credential interop fallback; Longfellow/libzk
  spike before any in-circuit SHA-256 (§2.3).
- **Q4 (commitment scope)** → resolved by recommendation, pending veto:
  commit the full VC envelope; validity windows checked in-circuit against
  the verifier's disclosed "now"; claims-only = non-goal (§2.2).
- **Q5 (union sizing)** → resolved: yes, it can be dynamic — a circuit
  family over the (k, n) lattice; prover and verifier use different
  per-bucket artifacts; the verifier re-derives the circuit id from the
  public statement and rejects mismatches; VKs content-addressed; lazy
  compile + cache on the server (§2.4).
- **Q6 (blank nodes)** → resolved by Jesse: **not skolemised**; four-layer
  protection — in-circuit (graph, local-id) scoping, zk-trace plan
  rejection, verifier re-derivation check, per-graph salting of RDFC10
  canonical labels (§2.2, §2.4).
- **Q7 (revocation)** → resolved by recommendation: hidden-index
  status-list inclusion, **v1-include** for the live service; verifier-
  side freshness policy; accumulators as the deferred upgrade path (§2.6).
- **Q8 (holder binding & replay)** → explained and resolved as
  use-case-dependent: challenge-binding (replay) and holder
  proof-of-possession (yes — proof of knowledge of a secret, bound via
  the VC `cnf` claim) are distinct concerns; both supported as per-proof
  `binding` modes (§2.5).
- **Q10 (Pedersen → Poseidon2)** → resolved by recommendation: yes, flip
  everywhere — legacy constraint gone, cheapest in-circuit hash, uniformity
  removes cross-suite bugs (§2.2, §6.3).
- **Q11 (timeline)** → resolved by Jesse: ISWC dropped; the target is an
  actual live Solid server service, with §7's operational posture; sparq
  house rules (roborev per commit, orchestrator merge-gates) apply.
- **Q12 (key references)** → resolved by Jesse: yes — `did:` URLs +
  cryptosuite ids in the manifest; issuer sets as trust-framework
  references over a signed issuer-registry graph (§2.8; specifics open
  below).
- **Q13 (metadata conventions)** → resolved by recommendation:
  `<D>?proof` proof graphs inheriting `<D>`'s ACL; enriched
  `<urn:sparq:zk>` registry, readable by Control-holders of the
  referenced documents only (§2.1).

**Genuinely still open:**

1. **Q9 — aggregation posture, reframed for the live service.**
   Manifest-of-proofs remains the posture; the open question is when (if
   ever) to invest in compression for `binding: holder-pop` re-presentable
   credentials. CHONK/ClientIVC is confirmed unusable for non-Aztec Noir
   as of mid-2026 (`zkp-performance-landscape.md` §2.6); its opening up is
   the revisit trigger.
2. **Trust-framework specifics** (Jesse actively thinking): framework
   vocabulary, registry-graph update semantics, cross-framework bridging,
   and whether framework membership ever needs to move in-circuit. §2.8 is
   the proposed starting shape, not the answer.
3. **test-lib completion: ownership and timeline.** Pushing the 76 commits
   + dirty `kernels.nr` is a Jesse action (private repo); the
   comparison/rounding/cast kernels that would make `test_lib` the XPath
   float backend are currently unowned work. Until decided, PR-#39 is the
   standing path (§4.E) — confirm or veto that sequencing.
4. **Veto window on the v3 recommendations**: Q3 (cryptosuite-first), Q4
   (full-envelope scope), Q7 (status-list v1-include), Q10 (Poseidon2
   flip), Q13 (naming/ACL conventions), and PR-#39-before-test_lib — all
   proceed as written unless overridden.

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
- [LONGFELLOW] Frigo, Shelat. Longfellow-zk / libzk (Google; deployed in
  Google Wallet). https://eprint.iacr.org/2024/2010 ·
  https://datatracker.ietf.org/doc/draft-google-cfrg-libzk/
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
`research/zkp-performance-landscape.md` (delivered — proving-performance
synthesis; the external evidence base behind §3/§4's system verdicts);
`research/zkp-noir-inventory-2026-06-12.md` (verified on-disk inventory of
the Noir estate; authoritative wherever memory and disk disagree);
`zkp-sparql-workspace/{HANDOFF-WAVE17.md, decisions/sparql-noir-modular-alternative.md, notes/research/02,05,08}` (frozen archive — citable, no new development);
`sparql_noir/spec/{encoding,algebra,proofs,preprocessing}.md`;
sparq `crates/{sparq-core/src/{dict,store}.rs, sparq-engine/src/exec.rs (mod trace), sparq-serve/src/{epoch,ring,writer}.rs, sparq-reason/src/{incremental,lib}.rs}`
and the `proof-trees` branch (`sparq-reason` feature `explain`:
`ProofTree`/`ProofNode`/`ExplainOpts`);
`research/{ARCHITECTURE.md, concurrent-serving.md §2.8–2.10, solid-access-control-design.md §2–3}`.
