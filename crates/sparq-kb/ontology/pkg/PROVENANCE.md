# PROVENANCE — the PKG (`pkg:`) ontology

> 🤖 **SPARQ agent** [OPUS-4.8] — reuse + alignment-verification record for the
> Project-Knowledge-Graph ontology. sq-2m6zm.1 (epic sq-2m6zm); design record
> `research/dogfooding-sparq-knowledge-graph.md` (PR #1063). Written while Fable
> unavailable; flag for re-review when Fable returns.

## What this is

`pkg.ttl` is the machine-readable form of the **reuse-first** Project-Knowledge-Graph
vocabulary of `research/dogfooding-sparq-knowledge-graph.md` §2.3, with `pkg.shapes.ttl`
(in `../../shapes/`) the SHACL **write-time guardrails** of §2.4 + §4.4. The
design principle is copied verbatim from the maintainer's own `sec-prop` discipline
(`research/security-properties-ontology-design.md`): **mint almost no net-new
terms.** PROV-O carries who/what/when/derived-from; SKOS carries concepts/topics;
DCAT + FaBiO/FRBR + DC carry the source catalog; CiTO carries the citation edges;
schema.org fills gaps; nanopublications package an assertion + its provenance; and
the vendored `zkp-sparql` `sig-impl:Assertion` pattern is **generalised** (not
forked) into `pkg:Finding`.

## Net-new vs reused

**Only four terms are genuinely net-new**, plus the single task-dependency inverse
pair and the bd/source/technique scaffolding the design names:

| net-new term | why it cannot be reused |
|---|---|
| `pkg:exploredStatus` | no FAIR-friendly DCAT/DC term carries the *explored-vs-unexplored* role for project knowledge (so follow-up can target the un-explored). Aligned to `schema:ActionStatusType` via `skos:closeMatch`. |
| `pkg:followUpPriority` | no term carries a *targeted-follow-up ordering* over un-explored sources. |
| `pkg:confidence` | no 0..1 numeric *confidence/epistemic-weight* literal exists in `schema:` / DPV / DC; minted as a bounded `xsd:decimal`, aligned to `schema:Rating`. Orthogonal to `pkg:assurance` (the enum epistemic-basis). |
| `pkg:couldBeMergedWith` | a forward-looking *merge-candidacy* hint with no established precedent; `skos:related` is weaker and not symmetric-by-design. Minted as an `owl:SymmetricProperty`. |
| `pkg:dependsOn` / `pkg:blockedBy` | the **single** task-dependency `owl:inverseOf` pair. There is deliberately **NO** `pkg:blocks`: bd's `blocks` edge is the *inverse-of-`pkg:dependsOn`* direction, modelled as `pkg:blockedBy`. Every shape / query names only these two predicates so a constraint cannot target an undefined property (design §2.2, must_fix-corrected). |

The rest reuses external vocabulary:

| PKG term | reuses (verified IRI) |
|---|---|
| `pkg:Source` | `fabio:Expression` + `dcat:CatalogRecord` + `dcterms:*` + `bibo:doi` + `frbr:realizationOf` |
| `pkg:Document` | `fabio:Expression` |
| topics / concepts | `skos:Concept` + `skos:inScheme` + `dcterms:subject` (no mint) |
| `pkg:Finding` | `rdfs:subClassOf sig-impl:Assertion , skos:Concept` |
| `pkg:about` | `rdfs:subPropertyOf dcterms:subject` |
| `pkg:verdict` | `rdfs:subPropertyOf sig-impl:verdict`; values `sig-impl:yes/no/partial` |
| `pkg:assurance` | values `secx:Proven` ⊐ `secx:Claimed` ⊐ `secx:Conjectured` (see *dependency* below) |
| supporting / refuting | `cito:supports` / `cito:citesAsEvidence` / `cito:disagreesWith` |
| provenance | `prov:wasDerivedFrom` / `wasGeneratedBy` / `wasAttributedTo` / `generatedAtTime` |
| `pkg:discoveredFrom` | `rdfs:subPropertyOf prov:wasDerivedFrom` |
| nanopub packaging | `np:hasAssertion` / `hasProvenance` / `hasPublicationInfo` |
| supersedes | `dcterms:replaces` / `dcterms:isReplacedBy` |
| alternative-to | `skos:related` |
| `pkg:implementedBy` | `schema:SoftwareSourceCode` (+ implementing PR as `prov:Activity`) |
| `pkg:Task` | `rdfs:subClassOf schema:Action` |
| parent-child | `dcterms:isPartOf` |

## Live-ontology alignment verification (design §2.5 requirement)

The design §2.5 flags that the `skos:closeMatch` alignments were cited from
knowledge of the SPAR/W3C-community vocabularies and **must be checked against the
live published ontology before shipping**. Each was verified against its live source
on **2026-06-21**:

| alignment used in `pkg.ttl` | live-ontology check | result |
|---|---|---|
| `schema:PotentialActionStatus`, `schema:ActiveActionStatus`, `schema:CompletedActionStatus`, `schema:FailedActionStatus` | `https://schema.org/ActionStatusType` | **confirmed** — all four are `ActionStatusType` enumeration members. |
| `cito:supports`, `cito:citesAsEvidence`, `cito:disagreesWith` | `http://purl.org/spar/cito/` (SPAR CiTO) | **confirmed** at `http://purl.org/spar/cito/{supports,citesAsEvidence,disagreesWith}`. |
| `fabio:Expression`, `fabio:ConferencePaper` | `http://purl.org/spar/fabio/` (SPAR FaBiO) | **confirmed**; FaBiO is FRBR-structured. |
| `np:Nanopublication`, `np:hasAssertion`, `np:hasProvenance`, `np:hasPublicationInfo` | `http://www.nanopub.org/nschema#` | **confirmed** (namespace + local names). |
| `schema:Rating` | `https://schema.org/Rating` | **confirmed** (used only as a `skos:closeMatch` soft pointer for `pkg:confidence`). |

### Honest note — `schema.org` HTTP vs HTTPS

schema.org's **canonical** namespace is `https://schema.org/` (HTTPS). This ontology
uses the **`http://schema.org/`** form, following (a) the design's stated convention
and (b) the existing sparq repo convention — `crates/sparq-trust` (`vocab.rs`,
`wire.rs`, `admit.rs`, the e2e tests) and the wider tree use `http://schema.org/`
(68 occurrences vs 25 for the HTTPS form). Because the schema.org references here are
only soft `skos:closeMatch` pointers (not `owl:equivalentClass`/hard imports), the
`http://` vs `https://` choice does not affect validation; it is a stylistic
alignment-with-the-repo decision, recorded here for auditability. A future
consistency pass could canonicalise the whole repo to `https://schema.org/`.

### Honest note — the `secx:` assurance axis is a forward dependency

`pkg:assurance` reuses `secx:Proven` / `secx:Claimed` / `secx:Conjectured`
(namespace `https://w3id.org/zkp-sparql/sec-prop#`; the design's `secx:` prose-prefix
= the sec-prop *extension* axis). **These three IRIs are defined in the DESIGN record
`research/security-properties-ontology-design.md` §4.2.2 (epic `sq-0dksu`) and are
NOT yet shipped as a committed `.ttl`/`.yaml.ld`** — the vendored
`crates/sparq-trust/ontologies/zkp-sparql/vocab/sec-prop.yaml.ld` defines the eight
*security properties* but not this assurance axis. They are referenced here by their
stable `w3id.org` namespace so they unify automatically when `sq-0dksu` ships the
extension. SHACL `sh:in` checks IRI identity only, so the guardrail fires correctly
today regardless; but a downstream consumer that *dereferences* `secx:Proven` will
not resolve it until `sq-0dksu` lands. A bead should track shipping the `secx:`
assurance individuals (captured as discovered work).

## Namespace

The `pkg:` namespace (`https://sparq.dev/ns/pkg#`) is a **sparq-local** namespace,
consistent with `trust:` (`https://sparq.dev/ns/trust#`) and `zk:`
(`https://sparq.dev/ns/zk#`). It is NOT minted/resolvable today; a future
standardisation pass would rehome the net-new terms. Every `pkg:` IRI is mirrored as
a Rust constant in `../../src/vocab.rs` and byte-pinned against `pkg.ttl` by the
`ttl_pins_match_rust_constants` sync test (the `sparq-trust` discipline).

## Consumers within sparq

- `crates/sparq-kb` (this crate) — ships the ontology + shapes + example as data, and
  the `validate` feature drives `sparq-shacl` over them (the dogfooding test).
- Phase-2 ingestion PoC (`sq-2m6zm.2`) — projects `.beads/issues.jsonl` + the
  AGENTS.md gate matrix + Skills frontmatter into `pkg:`-typed triples, gated on
  these shapes.

## References

- Design record: `research/dogfooding-sparq-knowledge-graph.md` (PR #1063).
- Precedent: `research/security-properties-ontology-design.md` (epic `sq-0dksu`);
  `crates/sparq-trust/ontologies/zkp-sparql/` + `crates/sparq-trust/src/vocab.rs`.
- Beads: epic `sq-2m6zm`; this task `sq-2m6zm.1`; blocks ingestion `sq-2m6zm.2`.
