<!-- [OPUS-4.8] MPC+ZKP research + first-pass architecture, Opus 4.8 (Fable unavailable) — design-for-review; re-review when Fable returns. -->

# MPC over (Federated) SPARQL with Zero-Knowledge Proof of Correct, Attested-Source Results

**Status:** First-pass architecture for review (design only, no implementation). Author: Opus 4.8 (Fable unavailable — flag for re-review). Date: 2026-06-13.

---

## 1. Abstract

This document specifies a first-pass architecture for **Secure Multi-Party Computation (MPC) over federated SPARQL**, in which a set of mutually-distrusting holders jointly evaluate a single SPARQL query over the *union* of their privately-held, issuer-signed RDF named graphs, and produce **one verifiable response** carrying a zero-knowledge proof that the result is (a) the correct evaluation of the query under Pérez–Arenas–Gutiérrez (PAG) semantics, and (b) **derived only from named graphs signed by issuers in a disclosed key-set**, while disclosing the minimum inter-source information needed to compute it. This is exactly Jesse Wright's **RQ2** [Wright, ISWC 2025 DC, CEUR Vol-4085 paper19 — https://ceur-ws.org/Vol-4085/paper19.pdf]: the federated, MPC layer above the single-holder ZK-over-VCs work (**RQ1**). The honest headline finding: the *correctness* half is achievable today only under honest-majority, LAN-scale, small-data regimes; the *attested-source* half rests on essentially **one** honest-majority building block [Dutta et al., Asiacrypt 2024 — https://eprint.iacr.org/2022/1648]; and **no published system composes both for graph/SPARQL queries**. The composition is the contribution *and* the principal research risk. This build is additionally **hard-blocked** on completing the outstanding RQ1 verifier-soundness remediation (issuer-signature, replay, FILTER-binding, attribution, revocation) before any MPC layer can mean anything.

---

## 2. Jesse's vision & requirements (faithful)

Jesse's unifying construct is a **verifiable data sublanguage**: a declarative query language returning answers *together with* zero-knowledge-verifiable provenance (sourcing, integrity, derivations), minimal-disclosure ("Is Jesse over 21 according to facts issued by EU or UK governments" → reveals only "yes") [Wright, CEUR Vol-4085; Transfer of Status report, Hilary 2025 — https://blog.jeswr.org/2025/05/06/transfer-of-status]. It is realised as **SPARQL 1.1 → 1.2 over RDF**, decomposed into three research questions whose decomposition *is* the ZK/MPC composition:

- **RQ1 — single graph database, ZK.** A holder proves a SPARQL SELECT result is correct against issuer-signed VC graphs. *Delivered* (feasibility): Risc0 zkVM + ed25519 baseline (~7.5 min for 23 triples, M1) [CEUR Vol-4085]; the Noir/UltraHonk successor (BGP/Join/Filter/OPTIONAL/UNION/bounded paths/EXISTS/NOT EXISTS/MINUS; sub-second for ≤10³ committed triples) [zksparql.org, ISWC 2026 submission; Braun, Wright, Käfer, ESWC 2026 — https://link.springer.com/chapter/10.1007/978-3-032-25156-5_16].
- **RQ2 — distributed/federated, MPC (+ZK).** Extend RQ1 to the *union across independent, potentially malicious graph databases*; sub-questions **RQ2a** (minimal disclosure between sources) and **RQ2b** (which logical profiles are efficiently supportable per configuration). *Unstarted; this document.*
- **RQ3 — auth/authz as emergent query planning.** Out of scope here; reuses RQ2's planner.

**Driving use case (his, not generic):** four flatmates, each with a Solid Pod of payslip VCs, jointly prove **cumulative salary > £100k** to a landlord. RQ1 handles per-holder "salary > £25k"; the union-aggregate "cumulative > £100k across four independent wallets" is precisely what requires MPC [CEUR Vol-4085, Minimal Use Case].

### Requirements & hard conventions the MPC layer MUST respect

1. **Named-graph credential model.** Each W3C VC (JSON-LD → RDF) is the named-graph unit; a holder's wallet is a set of such graphs. Provenance is intended to be first-class (SPARQL 1.2 reification / proof built-ins) [CEUR Vol-4085, sec:importance].
2. **Issuer-signed per-graph commitments.** Each named graph `G_i` carries a commitment `C(G_i)` bound to an issuer signature `Sign(pk_i, C(G_i))` with `pk_i` in a disclosed key-set `K`. Commitment scheme is **modular**: term → field element via Pedersen/Poseidon; quad → `hash4(...)`; default = sorted-leaf Merkle tree (non-membership via two-leaf bracket + sentinels), prefix-tree alternative [`04-commitment.tex`, zkp-sparql-workspace; verifiable-credentials-zk skill].
3. **Undisclosed-graph set-membership = "derived credential".** Proving "the answer derives from facts issued by *some* key in `K`" without revealing *which* graph/credential is an issuer-key set-membership proof over `K` — the core "derived from attested sources" relation.
4. **No-proof-of-revealed-properties.** Anything a deterministic function of the *disclosed* multiset (DISTINCT, ORDER BY, LIMIT/OFFSET, COUNT, SUM, AVG, MIN, MAX) is **recomputed by the verifier outside the circuit/MPC**, never proven in-circuit. Joins/UNIONs/OPTIONALs over disclosed values are out-of-circuit JS checks. HAVING / GROUP BY over hidden multisets are forbidden. **This is load-bearing for the MPC layer:** cross-source join/aggregation reasoning stays OUT of the cryptographic core wherever the joined values are disclosed [`feedback_zkp_no_proof_of_revealed_properties.md`].
5. **No-minted-IRIs.** Self-minted identifiers are blank nodes or real dereferenceable URLs (W3C/ORCID/ROR/GitHub/WebID); no `urn:`, no `https://id.jeswr.org/...`. RDF goes through the object mapper (`@solid/object`, `@jeswr/fetch-rdf`), never hand-rolled parsing. Any RDF repo ships the IRI dereferenceability lint [`feedback_no_minted_iris.md`, `feedback_iri_dereferenceability_lint.md`].
6. **Global IRIs as cross-credential join keys** — the distinguishing feature vs all prior graph-MPC (GOOSE/SMPG use node-local Cypher identifiers, disqualified for federation) [CEUR Vol-4085, sec on Cypher].
7. **Modularity is the contribution.** Commitment-shape and signature-scheme axes stay swappable; MPC primitives parameterise on the same `commit` / signature interfaces, not concrete schemes [`feedback_modular_commitment_signature_design.md`].
8. **Endpoint-independent, OWA-safe semantics** (PAG algebra); omitting credentials must not forge a valid result [CEUR Vol-4085, sec:importance].

---

## 3. Related-work landscape (terse, honest SOTA + trade-offs)

### 3.1 MPC primitives & relational query engines
- **Secret-sharing families.** Honest-majority 3PC replicated SS is the performance sweet spot (1 element/mult-gate; ~2× for malicious) but assumes ≥2-of-3 non-colluding compute parties — a *strong* assumption for a hostile-pod federation. Dishonest-majority malicious (SPDZ/MASCOT/Overdrive) is the realistic cross-org model but pays expensive input-independent preprocessing usually excluded from headline numbers [S&S 2022 survey — https://sands.edpsciences.org/articles/sands/full_html/2022/01/sands20210001/sands20210001.html; IACR 2022/417].
- **Network model dictates protocol:** secret-sharing (round-per-depth) wins on LAN; garbled/constant-round (Yao/BMR) wins on WAN — i.e. the federated setting.
- **Relational MPC SOTA.** Senate (USENIX'21, genuinely malicious n-party), Secrecy (NSDI'23, 3PC semi-honest, >1000× over naïve), Conclave (EuroSys'19, MPC+cleartext hybrid joins ≥7×, at a leakage cost), **ORQ (SOSP'25)** — first full TPC-H under MPC, O(n log n) join+aggregate fusion; honest envelope is *minutes-to-tens-of-minutes* (Q21 ≈ 42 min LAN, malicious) ×1.2–6.9 on WAN, **not** "a few minutes" [ORQ eprint 2025/1657 — https://eprint.iacr.org/2025/1657.pdf]. ORQ's malicious mode = 4-party honest-majority Fantastic Four; its default 3PC is semi-honest. **Joins are the cost center**; obliviousness forces worst-case padding at every intermediate step.
- **PSI for private joins.** VOLE-PSI: 2²⁰ elements, malicious, 6.2 s / <59 MB [eprint 2021/266]. But this is *two-party, single equi-join, key-on-key*; it does **not** compose for free into a multi-pattern SPARQL BGP or non-key/unbound-variable joins.

### 3.2 Graph / federated-SPARQL MPC (closest prior art — and its gaps)
- **GOOSE** (SPARQL UCRPQ fragment; honest broker; honest-but-curious DBs; no COUNT/SUM/AVG), **SMPG / PPMQ** (Cypher on Neo4j; Shamir via JIFF; semi-honest; conjunctive SPJ only). **PPMQ's sub-millisecond numbers are single-desktop, co-located, no inter-party network** — non-evidence for federation [PMC12662885 — https://pmc.ncbi.nlm.nih.gov/articles/PMC12662885/]. **GORAM** (VLDB'25, billion-scale) is ego-centric traversal on **ABY3 (semi-honest honest-majority 3PC)** — its "no party learns the graph" is *confidentiality only*, not correctness/attestation [arXiv 2410.02234].
- **Verdict:** every graph/SPARQL crypto system is semi-honest and/or confidentiality-only and/or conjunctive-only. **Malicious-secure, attested-input, full-SPARQL federation has zero published instances.**

### 3.3 Federated SPARQL (non-crypto, for the planner)
- Classic pipeline: source selection → decomposition → join ordering → execution (FedX, ANAPSID, CostFed). SOTA shift: **FedUP** (WWW'24) — result-aware plans via provenance over quotient summaries; 1–3 orders over classic on FedShop, parity on LargeRDFBench [https://hal.science/hal-04538238/document]. Relevant to RQ2's planner: minimise the *combinatorial* source-combination blow-up before any MPC is invoked.
- **VeriDKG** (PVLDB'24): verifiable (not confidential) SPARQL over decentralised KGs (RGB-Trie + accumulator) — integrity against a cheating server, no hiding. Distinct guarantee from this work.

### 3.4 ZK ⋂ MPC — the intersection this project lives in
- **Collaborative zk-SNARKs.** Multiple provers over a secret-shared witness emit *one* proof; verifier unchanged. Ozdemir–Boneh (USENIX'22): malicious-minority ≈ single-prover, N−1 malicious ≈ 2× — but **only at 3 Gb/s LAN**, honest-majority for the cheap case, preprocessing excluded [eprint 2021/1530]. Scalable coZK (USENIX'25): 128 servers / 2²¹ gates / >30× — but *proof delegation* (one client's witness split across its own helpers), PoC, not production [eprint 2024/940]. **A 2025 result shows the coZK template had exploitable soundness/privacy pitfalls** (proving on invalid witnesses leaks honest inputs; SOTA malicious compilers unsafe as-is) — "free malicious security" holds only honest-majority, post-patch [eprint 2025/1026 — https://eprint.iacr.org/2025/1026].
- **MPC over authenticated inputs — the attestation pillar.** Dutta et al. (Asiacrypt'24): generic compiler giving signature-authenticated inputs (the "derived from attested sources" half) — but **honest-majority, linear-secret-sharing only**, and it yields authenticated-MPC, *not* a succinct public ZK proof [eprint 2022/1648]. The Bontekoe survey's verdict: input authentication is the *under-explored* gap, with **only this one** work addressing verifiability + authentication together [arXiv 2309.08248].
- **MPC-in-the-head ≠ collaborative zk-SNARKs.** MPCitH/VOLEitH (Picnic/FAEST) are *single-prover* simulated MPC giving transparent PQ NIZKs (KB-sized, non-succinct) — do not conflate with real multi-prover coZK.
- **CP-SNARK linking glue** (LegoSNARK) binds signature-covered bytes to circuit-input bytes — standard for the single-prover case; **not demonstrated for distributed/secret-shared witnesses with in-circuit signature-opening** [eprint 2019/142].
- **Multi-key homomorphic signatures (MKHS).** A *direct alternative* to commit+SNARK for multi-issuer data: issuers' signatures homomorphically carry correctness of a function over data signed by distinct keys, publicly verifiable [eprint 2024/895]. Gives authenticity+correctness but **not hiding**; fully-succinct lattice MKHS practicality is unproven. Worth evaluating against the coZK route; complementary, not a replacement.

### 3.5 FHE / TEE — honest comparison as MPC+ZK alternatives
| Axis | FHE (+vFHE) | TEE (Nitro/TDX/SEV, +ZK) | **MPC + ZK (this work)** |
|---|---|---|---|
| Query maturity | research (~10K rows, ArcEDB) | **production** (DuckDB-SGX2) | high (Senate/ORQ); graph analog exists |
| Perf vs plaintext | 10³–10⁶× | **~1.5–2×** (46× if oblivious) | minutes; round-bound |
| Trust root | **lattice only (best)** | hardware vendor (**worst**) | honest-majority / k-of-n |
| Multi-owner | threshold/MHE | enclave sees plaintext | **native** |
| Correctness vs malicious | needs vFHE (secs–mins/bootstrap) | attestation **forgeable <$1k 2025** (WireTap) | malicious MPC + ZK |

- **TEE** is the only *interactive-today* option but raw attestation is **no longer a sound integrity proof** post-WireTap/Heracles/RMPocalypse (Intel rules physical attacks out-of-scope) [WireTap CCS'25 — https://dl.acm.org/doi/10.1145/3719027.3765204]. Viable only as TEE-for-speed + ZK-for-integrity; strictly worse on trust minimality.
- **FHE** has the strongest trust root and removes network rounds, but join-heavy SPARQL is its worst case (O(N²) oblivious padding), SOTA tops at ~10K rows, and verifiable-FHE for a whole query is impractical (per-bootstrap proofs). Long-horizon, small-data single-server sub-cases only.
- **Verdict:** keep MPC+ZK as the trust-minimal, multi-party, correctness-sound core; treat TEE as a pragmatic latency escape hatch (with a ZK integrity layer), FHE as a tracked long-horizon option.

---

## 4. Proposed architecture

### 4.1 Parties
- **Issuers** — sign each VC named graph; root of trust. Honest (trusted to sign honestly). Public keys resolvable into the disclosed key-set `K`.
- **Holders / data sources** (the four Pods) — each owns one or more signed named graphs; **mutually distrusting**, possibly malicious; act as the MPC compute parties and collaborative provers.
- **Verifier / relying party** (the landlord) — issues a fresh challenge, receives one verifiable response, checks the ZKP and recomputes all disclosed-property aggregates locally. Honest-but-curious.
- **(Optional) planner / coordinator** — produces the federated query plan. We treat it as *untrusted*: its plan is an input the cryptographic layer must not have to trust for soundness (contrast GOOSE/SMCQL's *honest broker*, which we explicitly reject).

### 4.2 Trust & threat model (state explicitly — do not paper over)
- **Honest issuers; malicious holder-provers; honest-but-curious verifier; passive network.** Out of scope: prover side-channels, verifier timing, active network.
- **Target security regime: honest-majority malicious.** This is an *honest admission of the literature ceiling*, not a preference: malicious-secure correctness for query eval (Senate/ORQ) and authenticated inputs (Dutta) both exist **only** honest-majority; dishonest-majority malicious correctness for query evaluation is *not demonstrated by any cited system*. The architecture states the honest-majority assumption as a first-class constraint and carries dishonest-majority as future research, not as shipped capability.
- **Four guarantees kept distinct** (the systematic conflation to avoid): (A) confidentiality, (B) computation correctness, (C) input authentication = "derived from attested sources", (D) malicious security. Three hard pitfalls encoded as assumptions: **commitment ≠ attestation** (proving over committed data says nothing about issuer signing — needs the CP-SNARK/Dutta link), **confidentiality ≠ correctness** (a "no party learns X" system can still return wrong answers), **honest-majority ≠ malicious-against-N−1**.

### 4.3 Protocol flow (one verifiable response)
1. **Setup.** Each holder ingests its VCs as named graphs; at *trusted ingest* draws per-graph OS-random salts (closes the cross-graph bnode salt-separation gap), commits `C(G_i)`, and obtains `Sign(pk_i, {C(G_i), true-triple-count, salt})` from the issuer (count + salt bound under the signature so short-count / salt-reuse views are detectable).
2. **Query + challenge.** Verifier sends SPARQL query `Q` + fresh nonce `N`. Planner produces a *result-aware* plan (FedUP-style) to minimise source combinations; **plan is untrusted**.
3. **Disclosure minimisation (RQ2a).** Decide per operator what is disclosed vs hidden under the no-proof-of-revealed-properties rule: disclosed-multiset functions (DISTINCT/ORDER/LIMIT/COUNT/SUM/AVG) are recomputed by the verifier; only the *secret* relation (which committed triples satisfy a hidden predicate, cross-source aggregate thresholds where intermediate values must stay private) goes into the MPC.
4. **MPC evaluation.** Holders run the SPARQL operator pipeline under MPC over secret-shared committed data: BGP via Merkle inclusion against signed roots; cross-source joins on **global IRIs** (key-on-key → (circuit-)PSI; non-key → oblivious join with bounded intermediate size); the £100k-style aggregate as a secure comparison whose intermediate per-source values never leave a source. **No in-circuit DISTINCT/sort/count.**
5. **Collaborative ZK proof.** The provers (= holders) jointly emit **one** proof binding: (i) `Disclosed(π) ⊆ Eval_PAG(Q,D)` (soundness); (ii) each contributing commitment is **issuer-signed under a key in `K`** (attestation / "derived credential" set-membership over `K`, revealing membership not identity); (iii) the query digest, FILTER operator/operand/verdict, per-row source-graph attribution, and challenge `N` are all bound into the proof's public inputs (closing the RQ1 replay/binding gaps). Aggregates and ordering are excluded — verifier recomputes them.
6. **Verification.** Verifier reconstructs public inputs from the declared statement and *canonical* circuit vk (never prover-supplied), checks the proof, recomputes disclosed-property aggregates, checks revocation non-membership + freshness window, and accepts a boolean/minimal answer.

### 4.4 Reuse of the existing RQ1 ZK estate
The MPC layer is designed to *wrap* the single-holder RQ1 estate, not replace it:
- **Commitment + signature design** ([`04-commitment.tex`]; modular `commit`/signature interfaces) is the per-source substrate; MPC primitives parameterise on the same interfaces (convention #7).
- **The ProofManifest / modular dispatcher** [`decisions/sparql-noir-modular-alternative.md`] is the closest existing seam: per-property circuits bound to disclosed rows, joins/UNIONs/OPTIONALs as out-of-circuit manifest edges. The clean architectural prize: the **composition obligation (manifest covers query) is a pure-data Lean theorem `ProofManifest × Query → Bool`, detached from crypto** — the natural place an MPC composition argument extends.
- **The #1/#2-hardened verifier** (public-input reconstruction from the canonical vk) is the *prerequisite* binding layer the collaborative proof plugs into — see §5 hard dependency.
- **Two RQ1 proving paths coexist** and the MPC layer must be aware of both: BBS+-direct (Braun/KIT) for BGP; Noir circuits for complex/string ops. Folding (Nova/SuperNova/HyperNova) for per-triple accumulation is a *research bet*, not a known-composable building block (no folding ⊕ collaborative-proving exists).

---

## 5. Open questions, risks & honest feasibility verdict

### 5.1 HARD DEPENDENCY (gating, not optional)
**No MPC+ZK build can mean anything to a relying party until the RQ1 verifier-soundness remediation is complete.** The v1 verifier proves *nothing* to a third party [`research/zk-soundness-audit.md`]. The MPC layer composes *on top of* the RQ1 binding layer, so the following remediation phases are **prerequisites**:
- **#3/#4 — issuer-signature + key-set membership, and replay/freshness binding.** Without #3, the prover is effectively the issuer of every fact (commitments are unsigned, prover-chosen) — the entire "attested sources" guarantee is absent. Without #4, every manifest is infinitely replayable. **The MPC attestation step (§4.3 step 5(ii)) IS issue #3 generalised to multiple sources** — it cannot be built before #3 lands single-source.
- **#5/#6 — FILTER operator/bound/verdict binding** (prover currently substitutes the comparison; the £25k/£100k threshold is the FILTER, so this is directly load-bearing for the use case).
- **#8/#9 — per-row source-graph attribution bound in-circuit + salt separation** (the cross-graph bnode correlation guard is exactly the *inter-source disclosure* control RQ2a is about; unbound attributions defeat it).
- **#12 — revocation** (status-list non-membership + freshness; a revoked credential must not contribute to a federated aggregate).

Until these land, the MPC layer would compose onto a verifier that returns `Ok(())` for forged results — i.e. it would be theatre. **Recommendation: do not start the MPC build until #3/#4/#5/#6/#8/#9/#12 are closed and have negative e2e tests.**

### 5.2 Open questions (research-grade)
1. **Distributed signature/commitment-opening inside the collaborative proof.** Verifying BBS+/EdDSA over a *secret-shared* witness in an MPC-friendly way is unsolved in the literature; the ~4.2K-constraint EdDSA-Poseidon figure is single-prover. *This is the join nobody has built.*
2. **Trust-model reconciliation.** Dutta (attestation) and ORQ-malicious (correctness) are both honest-majority; a hostile-pod federation arguably wants dishonest-majority. Does the use case (four *cooperating* flatmates vs the landlord) actually need dishonest-majority among holders, or is honest-majority-among-holders defensible? The answer reshapes the whole protocol choice.
3. **BGP-join obliviousness cost.** SPARQL BGPs are multi-way self-joins; obliviousness padding compounds per pattern. Does the no-proof-of-revealed-properties rule (disclose join keys, check joins out-of-circuit) collapse enough of this to stay tractable, and for which fragment (RQ2b)?
4. **coZK soundness post-2025/1026.** Any build must use patched constructions and re-audit; "collaborative zk-SNARK malicious security" is *not* settled.

### 5.3 Honest feasibility verdict
- **Impractical / expensive regime:** non-trivial BGP / multi-way join over multi-party RDF under **dishonest-majority malicious on a WAN** — *no published system demonstrates this*; realistic envelope is minutes-to-tens-of-minutes per query at best, and the full composition (coZK ⊕ malicious dishonest-majority MPC ⊕ oblivious BGP joins ⊕ attested inputs ⊕ WAN) has **zero performance data points**. Budget it as unquantified research risk, never "seconds."
- **Viable regime:** **honest-majority (or honest-but-curious among cooperating holders), LAN/datacenter co-location, small committed datasets (≤10³–10⁴ triples/party), few-pattern BGPs, disclosed-property aggregates recomputed by the verifier.** This aligns with the existing sub-second-for-≤10³-triples RQ1 figure and is a credible first target — far from "federated SPARQL at scale," and it should be presented as such.
- **The contribution is the composition, and so is the risk.** This is integration of three research lines (collaborative proving + authenticated-input MPC + the RQ1 SPARQL-correctness circuit with global-IRI join keys), **not** wiring of off-the-shelf malicious-secure parts. The realistic path: (a) finish RQ1 remediation; (b) prototype single-prover ZK-SPARQL-over-VCs (done); (c) *separately* prototype authenticated-input MPC over global-IRI joins in the viable regime; (d) budget real research time + a fresh soundness audit for the join.

---

### Key reference files
- RQ1 soundness gating (in-repo): [`research/zk-soundness-audit.md`](./zk-soundness-audit.md)
- The following are **external sources** (Jesse's private `zkp-sparql-workspace`, not in this repo — paths are relative to that workspace root):
  - Existing seam for MPC composition: `decisions/sparql-noir-modular-alternative.md`
  - Commitment/signature design: `paper/sections/04-commitment.tex`
  - RQ framing: `notes/research/03-jesse-prior-work.md`
  - Transfer report: Jesse's PhD transfer-of-status report (downloadable from blog.jeswr.org)
