<!-- [OPUS-4.8] Design-for-review authored by Opus 4.8 (1M context); Fable 5 unavailable — re-review when Fable returns. -->
# W3C standards & emerging-spec gaps for sparq

Research / design-for-review record. **No production code lands from this doc** — it
inventories which W3C standards sparq already implements (verified against the actual
code, not the briefs), then proposes the genuine remaining gaps as future beads for the
maintainer to triage.

> **Honesty / scope (read first).** Every "already have" claim below was checked against
> the crate source on `main` as of 2026-06-19, not taken from prior `research/` docs. The
> two RDF 1.2 / SPARQL 1.2 implementer references (`research/rdf12-parser.md`,
> `research/sparql12-engine.md`) are *spec digests*; the engine has since implemented the
> surface they describe and passes the vendored `w3c/rdf-tests` SPARQL 1.2 suite
> (`crates/sparq-conformance/FINDINGS.md`: 1225 pass / 0 fail / 4 documented divergences
> over 1229 in-scope tests, at rdf-tests `f25dbc0`). So **RDF 1.2 + SPARQL 1.2 are NOT a
> gap** — that is the main correction to the brief's premise (see §0). The ZK/MPC estate
> is research-stage and **NOT externally audited** (open `sq-qhy4` / `sq-9hrn`; epic
> `sq-1s2`); nothing here is a soundness or production-guarantee claim, and any candidate
> that touches the ZK pipeline inherits that caveat. No performance numbers are asserted.

---

## 0. What sparq ALREADY implements in this lens (verified inventory)

The brief lists a long menu of specs "to check". Most are already done. Checked against
source:

| Spec area | Status in sparq (verified) | Evidence |
|---|---|---|
| **SPARQL 1.1 query + update** | Implemented; full W3C suite | `sparq-engine`; conformance 1225/1229 |
| **SPARQL 1.2 — triple terms** | Implemented (`TRIPLE`/`isTRIPLE`/`SUBJECT`/`PREDICATE`/`OBJECT`), parser via vendored spargebra 0.4.6 (`[patch.crates-io]`) | `crates/sparq-engine/src/exec.rs:7744,9776`; `crates/sparq-conformance/FINDINGS.md` |
| **SPARQL 1.2 — dir-lang functions** | Implemented (`LANGDIR`/`hasLANG`/`hasLANGDIR`/`STRLANGDIR`), `sameValue` | `crates/sparq-engine/src/exec.rs:7765-7790` |
| **RDF 1.2 — triple terms, base direction** | Parsed + indexed + serialized; `{"type":"triple"}` JSON; `its:dir` | `research/rdf12-parser.md`, `rdf12-indexing.md`; conformance suite vendored at `tests/w3c/rdf-tests/sparql/sparql12/` |
| **RDFC-1.0 (RDF Dataset Canonicalization)** | Implemented for **RDF 1.1** datasets via zkp-ld `rdf-canon` 0.15.3 + a clean public crate | `crates/sparq-canon/` — **but rejects triple terms** (see §1.A) |
| **SHACL Core** | Full component set incl. SHACL-1.2 disjunctive-list + list constraints | `crates/sparq-shacl/`, `skills/shacl-validation/SKILL.md` |
| **SHACL-SPARQL (§5.2) + custom components (§6)** | Implemented (`sh:sparql`, `sh:select`, `sh:ConstraintComponent`, `sh:SPARQLAskValidator`) | `crates/sparq-shacl/src/sparql.rs`, `model.rs:249` |
| **SHACL-AF rules** | `sh:rule` (`sh:TripleRule` + `sh:SPARQLRule`), node-expr algebra, `sh:expression`, `sh:values`, function registry | feature `shacl-af`; `skills/shacl-validation/SKILL.md` §277-346 |
| **GeoSPARQL 1.0/1.1 functions** | sf / eh / rcc8 relation families, `geof:buffer`/`boundary`/`convexHull`/`envelope`/`difference`/`distance`/`getSRID`/`relate`, WKT + GML literal parse | `crates/sparq-geo/src/` (registry, provider, literal) |
| **OWL 2 RL** | Broad subset: `cls-*` (some/all/has-value/oneOf/intersection/union), `prp-spo2` (propertyChain), `hasKey`, cardinality, `scm-*` | `crates/sparq-reason/src/owl.rs` (2895 LOC) |
| **RDFS entailment** | Forward-chaining materialization; passes RDF(S) entailment conformance | `crates/sparq-reason/src/rdfs.rs`, `crates/sparq-conformance/src/inference/` |
| **N3 / Notation3** | Forward-chaining; N3 builtins | `crates/sparq-reason` |
| **SPARQL 1.1 Service Description + VoID** | `sd:Service` at endpoint (no `query`), VoID at `/.well-known/void`, `sd:resultFormat`/`sd:inputFormat`/`sd:feature` | `crates/sparq-server/src/descriptors.rs` |
| **SPARQL 1.1 Protocol + Graph Store Protocol** | Protocol full; GSP `GET`/`HEAD`/`PUT`/`POST`/`DELETE` (indirect + direct identification) | `crates/sparq-server/src/http.rs`, `graph.rs` |
| **PROV-O** | Lineage records for derived data (CONSTRUCT, reasoner materialization) | `crates/sparq-prov/` |
| **Federated Query (SERVICE) + TPF/brTPF** | `SERVICE`, native HTTP FragmentTransport, capability-aware pushdown | `crates/sparq-fedclient`, `sparq-fedplan` |
| **Solid WAC/ACP** | Authorization oracle (read + write path), conformance harness | `crates/sparq-solid/` |
| **JSON-LD** | Ingest (oxjsonld) + serialize (expand-style writer, `@list` collapsing, compact-IRI prefixes) | `crates/sparq-engine/src/serialize.rs`; framing/full-compaction **deferred, no consumer** (`research/jsonld-pretty-compaction-scope.md`) |
| **SKOS** | **Only a namespace prefix + a taxonomy-walk in `sparq-policy`** — NOT an inference or integrity profile (see §1.B) | `crates/sparq-policy/src/eval.rs:91` |
| **VC / DID** | `did:` IRIs stored as **opaque strings** in the ZK issuer registry; **no DID resolution, no VC Data Integrity verify** (see §1.C) | `crates/sparq-zk/src/registry.rs` |
| **LDP (Linked Data Platform)** | Not implemented as a server surface (LDP container semantics live in PSS/TypeScript by design) | n/a |

So the genuine gaps are a much shorter list than the brief implies. The candidates below
are only items I confirmed are **absent or materially incomplete** in the code.

---

## 1. The confirmed gaps (with evidence)

### 1.A — RDFC-1.0 canonicalization cannot handle RDF 1.2 triple terms

`crates/sparq-canon/src/lib.rs` **explicitly fails closed** on any dataset containing a
triple term:

```rust
// crates/sparq-canon/src/lib.rs (verbatim)
"RDF 1.2 triple terms cannot be canonicalized (RDFC-1.0 data model)"
```

The bridge serializes sparq's oxrdf-0.3 quads to N-Quads, parses into oxrdf-0.2, and runs
the zkp-ld `rdf-canon` 0.15.3 crate — whose published algorithm is defined over the RDF 1.1
abstract syntax. This is a *correct* conservative choice today, but it is now a **live
interop gap**: sparq ingests, indexes, queries, and serializes triple terms end-to-end, yet
the moment a dataset (e.g. a reified-statement graph, or a Verifiable-Credential graph with
annotation syntax) carries a triple term, it **cannot be canonicalized at all** — so it
cannot be hashed/committed by the ZK pipeline (`sparq-zk` depends on `sparq-canon`), cannot
be isomorphism-compared, and cannot get a stable C14N digest. The W3C position is that
RDFC-1.0 "is based on the abstract syntax of RDF 1.1, and the additions in RDF 1.2
(prominently triple terms and language direction) have consequences for this specification"
([rdf-star-wg #114](https://github.com/w3c/rdf-star-wg/issues/114)) — i.e. the spec gap is
acknowledged upstream and a profile/extension is the expected path.

### 1.B — SKOS has no integrity-validation or inference surface

`grep` confirms SKOS appears only as (a) a registered prefix in the serializer/server/NLQ
label list, and (b) a `skos:broader`/`skos:narrower` taxonomy walk inside `sparq-policy`'s
purpose-matching. There is **no SKOS integrity-condition checking** (the SKOS Reference's
S-constraints: S13 `prefLabel`/`altLabel`/`hiddenLabel` pairwise-disjoint, S14 ≤1
`prefLabel` per language, S27 `skos:related` disjoint with `skos:broaderTransitive`, cycle
detection, dangling references), and **no SKOS-specific inference** (S-rules: `skos:broader`
⊑ `skos:broaderTransitive`, `broaderTransitive`/`narrowerTransitive` are inverse and each
transitive, `narrowMatch`/`broadMatch`/`exactMatch` semantics). SKOS quality tooling is a
well-established niche (qSKOS, Skosify, SHACL Play's SKOS shapes catalog) but there is no
Rust-native, embeddable one — and sparq already has both an OWL-RL/RDFS forward-chainer and
a full SHACL stack to host it.

### 1.C — VC Data Integrity verification + DID resolution are absent

The ZK issuer registry (`crates/sparq-zk/src/registry.rs`) stores `zk:issuerKey
<did:example:dmv#key-1>` as an **opaque IRI** and a separate `zk:issuerPublicKey` with the
raw Baby-JubJub key material; it never *resolves* the DID or *verifies* a W3C Verifiable
Credential proof. The maintainer's clear ZK/privacy/Solid direction is built around
issuer-signed credentials, so the inability to (a) resolve `did:key`/`did:web` to a
verification method and (b) verify a `DataIntegrityProof` (`eddsa-2022`, `ecdsa-sd-2023`,
`bbs-2023`) on an incoming VC graph is a real gap between the standards stack and what the
engine actually trusts. The Rust prior art is mature (spruceid's `ssi` family covers DID
resolution + VC Data Integrity), so this is an *integration*, not an invention.

### 1.D — SPARQL 1.2 Service Description version negotiation

`descriptors.rs` advertises `sd:supportedLanguage sd:SPARQL11Query` (and `SPARQL11Update`)
only. SPARQL 1.2 Service Description **collapses `sd:Language` to just Query/Update and adds
`sd:supportedVersion`** to carry the language version
([w3c/sparql-service-description](https://w3c.github.io/sparql-service-description/spec/)).
Since sparq now evaluates SPARQL 1.2, the descriptor under-advertises the engine's actual
capability — a federation client cannot discover that triple-term queries are supported.

### 1.E — Graph Store Protocol PATCH

`http.rs` explicitly **405s `PATCH`** on the GSP route (`"PATCH which we 405"`,
`crates/sparq-server/src/http.rs:4287`). The Solid Protocol and SPARQL 1.2 GSP both define
a graph-level `PATCH` (Solid uses `text/n3` patches / `application/sparql-update`; SPARQL
1.2 GSP discusses PATCH semantics). A graph-scoped `PATCH` with a SPARQL-Update or
N3-Patch body is the one GSP verb sparq does not serve.

---

## 2. The candidate features

Each candidate is an **opt-in crate / cargo-feature** (the lean-core rule). None touch the
default `sparq-core`/`sparq-engine` build.

### C1 — `sparq-canon`: RDFC-1.0 profile for RDF 1.2 triple terms (HIGH)

**What.** Extend `sparq-canon` to canonicalize datasets containing triple terms, instead of
failing closed. Triple terms are *transparent ground/structured terms* with no blank-node
identity of their own except via nested blank nodes; the natural approach is to define a
deterministic flattening — encode each triple-term object into a canonical token over its
already-canonicalized components and feed it through the existing N-Quads bridge — or, if a
recursive blank node appears inside a triple term, extend the HNDQ first-/n-degree hashing
to descend into the triple-term structure. This must be a **named, documented profile**
("RDFC-1.0 + sparq triple-term extension"), explicitly NOT claimed as standard RDFC-1.0,
because W3C has not yet standardized the RDF 1.2 extension.

**Why.** Unblocks the ZK commitment pipeline and isomorphism/digest paths for any dataset
that uses reification/annotation syntax — which is precisely the credential-graph shape the
ZK/Solid direction targets. Today such a graph is un-canonicalizable, so it is silently
outside the trust pipeline.

**Decision ask.** Implement a **sparq-local extension profile now** (clearly labelled
non-standard), or **wait for the W3C RDF 1.2 canonicalization work** and keep failing
closed? If the former, do we constrain it to triple terms with **no nested blank nodes**
(the common credential case, much simpler) as v1?

### C2 — `sparq-skos`: SKOS integrity validation + inference (opt-in crate) (MED)

**What.** A new opt-in crate that ships (a) a curated SHACL shapes pack encoding the SKOS
Reference integrity conditions (S13/S14/S27, disjointness, ≤1 prefLabel/lang, dangling
refs) plus the most-cited qSKOS quality checks (cycles in broader/narrower, orphan
concepts, missing top concepts, relation clashes), runnable through the existing
`sparq-shacl` engine; and (b) a small set of SKOS S-rules for the OWL-RL/RDFS forward-
chainer (`broader` ⊑ `broaderTransitive`, transitive closure, inverse pairs). Output is the
standard SHACL validation report.

**Why.** Controlled-vocabulary publishers (libraries, government taxonomies, EU DPV) need
SKOS QA; there is no Rust-native embeddable tool. It composes cleanly with sparq's GUI ("load
a thesaurus, get a quality report") and reuses two surfaces sparq already has.

**Decision ask.** Worth a dedicated crate, or just **ship the shapes pack as a data asset /
example** alongside `sparq-shacl` (zero new code, the shapes run on the existing engine)?
The latter is near-zero-effort and may be the right first step.

### C3 — `sparq-vc`: VC Data Integrity verify + DID resolution (opt-in crate) (MED→HIGH)

**What.** An opt-in crate wrapping the spruceid `ssi` stack to: resolve `did:key` and
`did:web` to a verification method (W3C Controlled Identifiers / DID Core), and **verify** a
W3C VC 2.0 `DataIntegrityProof` (`eddsa-2022` first; `ecdsa-sd-2023`/`bbs-2023` selective-
disclosure later) over a VC graph loaded into a sparq `Graph`. It exposes a typed
`VerifiedCredential` (issuer DID, subject, validity window, verification result) the ZK
registry and Solid oracle can consume — turning today's *opaque trusted-issuer string* into
a *cryptographically verified* issuer fact.

**Why.** This is the missing link between the standards stack the maintainer cares about
(VC/DID, Solid, ZK-over-credentials) and what the engine actually trusts. The ZK pipeline's
issuer-signature layer (`sparq-zk/src/sig.rs`) already assumes a trusted issuer key; this
makes the *upstream* "is this credential genuinely from that issuer" step real, with vetted
crypto.

**Honesty constraint.** This verifies *standard VC Data Integrity* (audited upstream crypto)
— it is **separate from and must not be conflated with** the sparq ZK estate (which is
NOT externally audited). Keep the crate docs explicit that VC-DI verification is the
mainstream, audited path and the ZK query-proof layer is the research path.

**Decision ask.** Take the `ssi` dependency (large surface, but the standard Rust choice),
or keep VC verification out-of-engine in PSS/TypeScript (mirroring the Solid security-in-PSS
rule)? Which suites in v1 — `eddsa-2022` only, or include `ecdsa-sd-2023`/`bbs-2023` for
selective disclosure?

### C4 — SPARQL 1.2 Service Description: `sd:supportedVersion` + 1.2 language IRIs (S)

**What.** Update `sparq-server`'s descriptor to advertise `sd:supportedVersion` and the
SPARQL 1.2 language posture (per the 1.2 SD WD), gated by the engine's actual 1.2 feature
state so the advertisement is never a fiction (the same honesty boundary the descriptor
already applies to `BasicFederatedQuery`/`provenance`).

**Why.** Federation source-selection can then discover that an endpoint speaks SPARQL 1.2 /
triple terms. Small, self-contained, directly improves the federation story.

**Decision ask.** Advertise 1.2 now (the WD is stable-in-direction but not a Rec), or wait
for the SD to reach CR? Honesty gate: only advertise features actually compiled in.

### C5 — Graph Store Protocol `PATCH` (graph-scoped update) (M)

**What.** Implement GSP `PATCH` on the existing graph route: accept an
`application/sparql-update` (and optionally Solid `text/n3` N3-Patch) body scoped to the
target graph, executed atomically through the existing WAL-durable update path. Replaces the
current hard `405`.

**Why.** Completes the GSP verb set and is the one piece of graph-level write semantics PSS
cannot delegate to sparq today. Useful for incremental graph edits without a full
`PUT`-replace.

**Decision ask.** Is graph-level `PATCH` wanted at the sparq-server layer at all, given the
Solid "writes go through PSS" rule? If yes, SPARQL-Update body only, or also N3-Patch (which
pulls in more parsing)?

### C6 — `sparq-reason`: OWL 2 QL / EL profiles (opt-in feature) (L)

**What.** Add the OWL 2 QL (query-rewriting / DL-Lite$_R$) and/or EL (`ELK`-style
classification) profiles alongside the existing OWL-RL forward-chainer, as opt-in features.
QL is the natural fit for *query rewriting* over large ABoxes (no materialization), EL for
*ontology classification* (biomedical ontologies: SNOMED CT, GO).

**Why.** OWL-RL is one of three OWL 2 profiles; QL and EL serve disjoint use-cases (QL =
OBDA-style rewriting, EL = large terminology classification) that RL cannot. This is the
single biggest *reasoning* standards gap.

**Honesty note.** This is genuinely **L/XL effort** — QL rewriting and EL classification are
substantial algorithms, not rule additions. Likely the lowest-priority candidate unless a
concrete consumer (e.g. a SNOMED-CT classification demo for the GUI) drives it.

**Decision ask.** Is either QL or EL on the roadmap, or is OWL-RL deemed sufficient? If one,
which — QL (federation/OBDA fit) or EL (biomedical-ontology fit)?

### C7 — `sparq-canon`: canonical N-Triples/N-Quads 1.2 output + c14n digest API (S→M)

**What.** Smaller sibling of C1: even without full triple-term *blank-node* canonicalization,
expose a **canonical N-Quads 1.2 serializer** (the RDF 1.2 canonical-form token rules for
triple terms + `@lang--dir`) and a stable content-hash over a *ground* (blank-node-free)
dataset including triple terms. Many credential/ZK graphs are ground; a ground-dataset digest
is well-defined without the HNDQ machinery and unblocks the common case immediately.

**Why.** A pragmatic first slice of C1 that closes the ZK-pipeline blocker for ground
graphs with near-zero risk, deferring the hard blank-node-in-triple-term case.

**Decision ask.** Ship the ground-dataset digest as the v1 (and treat full C1 as a later
phase), or go straight for full C1?

---

## 3. Recommendation

In maintainer-direction priority order (ZK/privacy, Solid, federation, GUI):

1. **C7 then C1** — the RDF 1.2 canonicalization gap is the only one that *blocks an
   existing pipeline* (ZK over reified/credential graphs). C7 (ground-dataset digest) is a
   small, low-risk first slice; C1 (full profile) follows. **Highest value-per-effort.**
2. **C3 (`sparq-vc`)** — directly advances the VC/DID/Solid/ZK direction with audited
   upstream crypto; turns the opaque trusted-issuer assumption into a verified fact. Gate on
   the honesty boundary (audited VC-DI ≠ unaudited ZK).
3. **C2 (`sparq-skos`)** — high fit, low effort *if* shipped first as a SHACL shapes asset
   (decision ask in C2). Good GUI demo.
4. **C4 (SD 1.2)** — small, self-contained federation win; can land independently.
5. **C5 (GSP PATCH)** and **C6 (OWL QL/EL)** — only if a concrete consumer appears;
   C6 especially is L/XL and should wait for a driving use-case.

## 4. Phased plan (each phase = a future bead for the orchestrator)

1. **Bead: canon-ground-digest (C7)** — `sparq-canon`: canonical N-Quads 1.2 serializer +
   stable content-hash for *ground* datasets containing triple terms / dir-lang literals;
   tests over reified-credential fixtures. (area: `sparq-canon`; effort S→M)
2. **Bead: canon-rdf12-profile (C1)** — extend HNDQ to descend into triple terms with nested
   blank nodes; ship as a clearly-labelled non-standard "RDFC-1.0 + RDF 1.2 extension"
   profile; W3C-tracking note. Depends on bead 1. (area: `sparq-canon`; effort M→L)
3. **Bead: sparq-vc-verify (C3)** — opt-in `sparq-vc` crate: `did:key`/`did:web` resolution +
   `eddsa-2022` VC Data Integrity verify over a sparq `Graph`, typed `VerifiedCredential`;
   honesty-boundary docs (audited VC-DI vs unaudited ZK). (area: new `sparq-vc`; effort M→L)
4. **Bead: sparq-vc-sd (C3 follow-up)** — add `ecdsa-sd-2023` / `bbs-2023` selective-
   disclosure suites; wire `VerifiedCredential` into the ZK issuer registry as a verified
   issuer fact. Depends on bead 3. (area: `sparq-vc` + `sparq-zk`; effort M)
5. **Bead: skos-shapes (C2 step 1)** — curated SKOS integrity + qSKOS SHACL shapes pack as a
   data asset runnable on `sparq-shacl`; example + report fixtures. (area: `sparq-shacl`
   assets; effort S)
6. **Bead: sparq-skos-crate (C2 step 2)** — promote to an opt-in `sparq-skos` crate adding
   SKOS S-rules to the forward-chainer (transitive closure, inverse pairs) if the shapes pack
   earns a consumer. Depends on bead 5. (area: new `sparq-skos`; effort M)
7. **Bead: sd-sparql12 (C4)** — `sd:supportedVersion` + 1.2 language posture in the
   descriptor, feature-gated to the engine's real 1.2 state. (area: `sparq-server`; effort S)
8. **Bead: gsp-patch (C5)** — GSP `PATCH` with `application/sparql-update` body on the graph
   route, atomic via the WAL update path; optional N3-Patch later. (area: `sparq-server`;
   effort M)
9. **Bead: owl-ql-el-spike (C6)** — *spike only*: feasibility + consumer-need assessment for
   OWL 2 QL (rewriting) and EL (classification) before any implementation commitment.
   (area: `sparq-reason`; effort S for the spike, L→XL if greenlit)

## 5. Open questions that genuinely need the maintainer

- **C1/C7:** Implement a sparq-local non-standard RDF 1.2 canonicalization profile now, or
  wait for W3C? (The ZK pipeline is blocked on triple-term graphs until one exists.)
- **C3:** Does VC Data Integrity verification belong *in the engine* (`sparq-vc`), or does
  the Solid "security-critical paths run through vetted TypeScript in PSS" rule extend to VC
  verification too — making this a PSS concern and out of scope for sparq?
- **C3:** Take the large `ssi` dependency, or hand-roll only `eddsa-2022` + `did:key`/
  `did:web` (smaller surface, more maintenance)?
- **C2:** Dedicated `sparq-skos` crate, or just a SHACL shapes asset on the existing engine?
- **C6:** Is any OWL 2 profile beyond RL on the roadmap, and if so QL or EL?
- **General:** All of the SPARQL 1.2 / RDF 1.2 *surface* specs are still Working Drafts
  (only Concepts + Semantics are CR). Do we advertise/ship 1.2-derived features now, or hold
  surface-level conformance claims until the docs reach CR/Rec?

## Sources

Internal (verified on `main`, 2026-06-19): `crates/sparq-canon/src/lib.rs`,
`crates/sparq-shacl/`, `crates/sparq-reason/src/owl.rs`, `crates/sparq-geo/src/`,
`crates/sparq-server/src/descriptors.rs`, `crates/sparq-server/src/http.rs`,
`crates/sparq-zk/src/registry.rs`, `crates/sparq-policy/src/eval.rs`,
`crates/sparq-conformance/FINDINGS.md`, `research/rdf12-parser.md`,
`research/sparql12-engine.md`, `research/jsonld-pretty-compaction-scope.md`,
`research/sparq-solid-scope.md`, `skills/shacl-validation/SKILL.md`.

External:
- W3C VC Data Model 2.0 — <https://www.w3.org/TR/vc-data-model-2.0/>
- W3C Data Integrity ECDSA Cryptosuites v1.0 — <https://www.w3.org/TR/vc-di-ecdsa/>
- W3C Data Integrity BBS Cryptosuites v1.0 — <https://www.w3.org/TR/vc-di-bbs/>
- W3C Data Integrity EdDSA Cryptosuites v1.0 — <https://www.w3.org/TR/vc-di-eddsa/>
- spruceid `ssi` (DID resolution + VC Data Integrity, Rust) — <https://github.com/spruceid/ssi>, <https://docs.rs/ssi/>
- W3C RDF Dataset Canonicalization (RDFC-1.0) — <https://www.w3.org/TR/rdf-canon/>
- RDF 1.2 effect on canonicalization (rdf-star-wg #114) — <https://github.com/w3c/rdf-star-wg/issues/114>
- SKOS Reference (integrity conditions S1–S46) — <https://www.w3.org/TR/skos-reference/>
- qSKOS quality issues — <https://github.com/cmader/qSKOS/wiki/Quality-Issues>
- SHACL Play SKOS shapes catalog — <https://shacl-play.sparna.fr/play/shapes-catalog>
- SPARQL 1.2 Service Description (ED) — <https://w3c.github.io/sparql-service-description/spec/>
- SPARQL 1.2 Graph Store Protocol (ED) — <https://w3c.github.io/sparql-graph-store-protocol/spec/>
- GeoSPARQL 1.1 (OGC 22-047r1) — <https://docs.ogc.org/is/22-047r1/22-047r1.html>
- OWL 2 Profiles (RL/QL/EL) — <https://www.w3.org/TR/owl2-profiles/>
