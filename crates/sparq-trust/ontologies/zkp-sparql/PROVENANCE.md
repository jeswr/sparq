# PROVENANCE — vendored `sparql-zkp-ontologies`

> 🤖 **SPARQ agent** [OPUS-4.8] — provenance + attribution record for the
> ontology files vendored into this directory.

## What this is

This directory is a **verbatim vendored copy** of the ontology source from the
`sparql-zkp-ontologies` repository: a CI-validated, SHACL-shaped vocabulary set
describing the security/privacy properties of zero-knowledge-proof-of-SPARQL
systems and the signature implementations + regulatory requirements that pull on
them. It is the companion ontology to the **ISWC 2025** ZKP-SPARQL paper.

| Sub-vocabulary | Namespace | Scope |
|----------------|-----------|-------|
| `sec-prop:` | `https://w3id.org/zkp-sparql/sec-prop#` | The eight security properties from paper §7.7 (unlinkability, source-credential disclosure, PQ forgery, PQ snooping, signature-type leakage, proof-size leakage, circuit audit, validity-period leakage). |
| `sig-impl:` | `https://w3id.org/zkp-sparql/sig-impl#` | Four signature implementations (BBS+, SD-JWT-VC, ed25519, ECDSA), each carrying reified `sig-impl:Assertion` verdicts (`yes`/`no`/`partial`) about the properties it achieves/fails. |
| `sec-req:` | `https://w3id.org/zkp-sparql/sec-req#` | Three regulatory frameworks (eIDAS 2.0, NIST PQC migration, UK DVS) that pull on `sec-prop:` properties, with dated obligations. |
| `prov-ext:` | `https://w3id.org/zkp-sparql/prov-ext#` | Minimal `bibo:`/`dcterms:`/`prov:` provenance extension (one coined term, `prov-ext:bibtexKey`) backing the `prov:wasDerivedFrom` citation chains. |

## Origin

- **Origin repository:** `github.com/jeswr/sparql-zkp-ontologies` (private at
  time of vendoring; the SPARQ agent had read access via the maintainer's `gh`
  token).
- **Vendored-from commit SHA:** `0fe80ea7d858de9f02bd29df29f6e50cdada14a0`
  (authored 2026-05-04).
- **Authorship:** the unpublished prior ontology work of **Jesse Wright** (the
  sparq maintainer) and the co-authors of the accompanying ISWC 2025 paper,
  which include **Nigel Shadbolt, Jun Zhao, and Rui Zhao**. See the upstream
  `LICENSE-DRAFT.md` (copied verbatim alongside this file) and the paper for the
  full author and acknowledgement set.

## Why it is here (the maintainer's 2026-06-20 decision)

The original repository carried an open question — *"release `sparql-zkp-ontologies`
publicly?"* — which it tracked in its `LICENSE-DRAFT.md` and `README.md` (both
stating the repo was private and the IRIs were "placeholder pending a
public-flip decision").

On **2026-06-20** the maintainer **resolved that question** and directed:

1. **Do NOT make the external `jeswr/sparql-zkp-ontologies` repo public.**
2. Instead, **vendor its ontologies into the public sparq codebase** with full
   attribution and the license preserved.
3. **Archive** the external repository afterward.

sparq is a **public** repository, so this content becomes public — that is the
maintainer's **explicit intent**: the ontology accompanies a published (ISWC
2025) paper and is ready to be cited. The "do not redistribute / publish / cite"
language in the vendored `LICENSE-DRAFT.md` reflects the *pre-publication private*
state of the source repo; it is superseded for this vendored copy by the
maintainer's authoritative 2026-06-20 publication decision. The maintainer
(Jesse Wright) is the sole copyright holder and the licensor named in that draft,
so the decision to publish is itself the grant. The draft is preserved verbatim
for historical fidelity, not as an active prohibition on this copy.

## License handling

The upstream license was a **draft** (`LICENSE-DRAFT.md`) — an MIT placeholder
that was *"pending public release"* and *"not in force"* while the source repo
was private. Per the maintainer's instruction this vendoring **does NOT
relicense** the content: the original `LICENSE-DRAFT.md` is copied **verbatim**
into this directory (including its pending-MIT text and its private-state
caveats) so the original author intent and licensing trail are preserved intact.

The maintainer's 2026-06-20 decision is the authoritative act that authorises
this content's publication under sparq's public repository; activating the
pending MIT terms (or any other relicensing) is the maintainer's prerogative and
is **not** performed here.

## Namespace resolution (open question — RESOLVED)

The upstream repo flagged the `https://w3id.org/zkp-sparql/...` IRIs as
"placeholder while the repository is private." **Resolution (2026-06-20):** the
`https://w3id.org/zkp-sparql/...` IRIs are **kept as-is** in this vendored copy.
**w3id.org IRIs are independent of the source repository's visibility** — they
resolve via the w3id permanent-identifier redirect, not the GitHub repo — so they
remain stable even after the external repo is archived. No re-minting into a
sparq-local namespace is performed. The sparq security-properties ontology
(`research/security-properties-ontology-design.md`, epic `sq-0dksu`) **extends**
this `sec-prop:` namespace; it does not fork it.

## Authoring / pipeline notes (from upstream)

- **Source authoring is yaml-ld** (`vocab/*.yaml.ld`); upstream round-tripped it
  to JSON-LD → SHACL-validated → Turtle for the paper appendix. The vendored copy
  carries the **yaml-ld source** + the **SHACL shapes** (`shapes/*.shapes.ttl`);
  the generated JSON-LD/Turtle were build outputs (`build/`, gitignored upstream)
  and are not vendored — they are regenerable from the source.
- Provenance discipline: every claim carries `prov:wasDerivedFrom` pointing at a
  `bibo:Document` declared in `vocab/prov-ext.yaml.ld`.
- British English in prose; reified-assertion pattern for sig-impl ↔ sec-prop
  verdicts.

## Consumers within sparq

- `crates/sparq-trust` (this crate) — the trust-graph authorisation PoC; the
  planned secprop **property-admissibility pre-check** (`sq-dt5hv`) in its
  `admit.rs` will consume the `sec-prop:`/`sig-impl:` graphs.
- `research/security-properties-ontology-design.md` (epic `sq-0dksu`) — the
  sparq design record that extends `sec-prop:`.

These files are **data**, not Rust source: vendoring them does not change the
`sparq-trust` build (nothing `include_str!`s them today). Any future loader must
be feature-gated to keep the lean core unaffected (strict additivity, design
§2.2 G6).

## References

- Epic `sq-0dksu` · vocab bead `sq-5oru9` · secprop precheck `sq-dt5hv`
- Design record: `research/security-properties-ontology-design.md` (PR #972)
- Open maintainer decisions: **#1001** (default assurance level) · **#1002**
  (DPV alignment depth)
