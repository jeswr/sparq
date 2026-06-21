# yaml-ld source for the eight security properties from §7.7 of the
# ZKP-SPARQL paper. Round-trips to JSON-LD via `scripts/render-jsonld.sh`,
# Turtle via `scripts/render-turtle.sh`, validated by `shapes/sec-prop.shapes.ttl`.
#
# Each property is an instance of `sec-prop:SecurityProperty` (a subclass of
# `skos:Concept`). prov:wasDerivedFrom links each property to one or more
# `bibo:Document` references; those references resolve to BibTeX keys in
# `notes/research/08-bibliography.md` of the parent workspace.
#
# Stability note: IRIs under https://w3id.org/zkp-sparql/ are placeholder
# while the repo is private (see README and LICENSE-DRAFT.md).
"@context":
  "@vocab": "https://w3id.org/zkp-sparql/sec-prop#"
  sec-prop: "https://w3id.org/zkp-sparql/sec-prop#"
  rdf:      "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  rdfs:     "http://www.w3.org/2000/01/rdf-schema#"
  owl:      "http://www.w3.org/2002/07/owl#"
  skos:     "http://www.w3.org/2004/02/skos/core#"
  prov:     "http://www.w3.org/ns/prov#"
  dcterms:  "http://purl.org/dc/terms/"
  bibo:     "http://purl.org/ontology/bibo/"
  xsd:      "http://www.w3.org/2001/XMLSchema#"
  label:           { "@id": "rdfs:label",   "@language": "en-GB" }
  comment:         { "@id": "rdfs:comment", "@language": "en-GB" }
  prefLabel:       { "@id": "skos:prefLabel", "@language": "en-GB" }
  threat:          { "@id": "sec-prop:threat",         "@language": "en-GB" }
  defence:         { "@id": "sec-prop:defence",        "@language": "en-GB" }
  openQuestion:    { "@id": "sec-prop:openQuestion",   "@language": "en-GB" }
  category:        { "@id": "sec-prop:category",       "@type": "@id" }
  paperSection:    { "@id": "sec-prop:paperSection" }
  wasDerivedFrom:  { "@id": "prov:wasDerivedFrom",     "@type": "@id", "@container": "@set" }
  type:            "@type"
  id:              "@id"

"@graph":

  # ─────────── Class definitions ───────────
  - id: sec-prop:SecurityProperty
    type: [owl:Class, rdfs:Class]
    label: "Security property"
    comment: >-
      A property a ZKP-of-SPARQL system may attain, fail, or partially
      attain. Each property is paired with a defence and an open question
      in paper §7.7.
    rdfs:subClassOf: { id: skos:Concept }

  - id: sec-prop:Category
    type: [owl:Class, rdfs:Class]
    label: "Property category"
    comment: >-
      Coarse grouping of properties: formal-analysis priority,
      information-disclosure, or deployment/spec.

  - id: sec-prop:formalAnalysisPriority
    type: sec-prop:Category
    label: "Formal-analysis priority"
    comment: "Properties whose mechanised proof is the priority deliverable."
  - id: sec-prop:informationDisclosure
    type: sec-prop:Category
    label: "Information disclosure"
    comment: "Properties about what a verifier learns from a proof."
  - id: sec-prop:deploymentSpec
    type: sec-prop:Category
    label: "Deployment / specification"
    comment: "Properties whose primary obligation is on the spec or deployment."

  # ─────────── Predicate definitions ───────────
  - id: sec-prop:threat
    type: owl:DatatypeProperty
    label: "Threat"
    rdfs:domain: { id: sec-prop:SecurityProperty }
    rdfs:range:  { id: rdf:langString }
  - id: sec-prop:defence
    type: owl:DatatypeProperty
    label: "Defence"
    rdfs:domain: { id: sec-prop:SecurityProperty }
    rdfs:range:  { id: rdf:langString }
  - id: sec-prop:openQuestion
    type: owl:DatatypeProperty
    label: "Open question"
    rdfs:domain: { id: sec-prop:SecurityProperty }
    rdfs:range:  { id: rdf:langString }
  - id: sec-prop:category
    type: owl:ObjectProperty
    label: "Category"
    rdfs:domain: { id: sec-prop:SecurityProperty }
    rdfs:range:  { id: sec-prop:Category }
  - id: sec-prop:paperSection
    type: owl:DatatypeProperty
    label: "Paper section"
    rdfs:domain: { id: sec-prop:SecurityProperty }
    rdfs:range:  { id: xsd:string }

  # ─────────── Properties (1–8) ───────────

  - id: sec-prop:Unlinkability
    type: sec-prop:SecurityProperty
    label: "Unlinkability (pre+post quantum)"
    prefLabel: "Unlinkability"
    paperSection: "§7.7 #1"
    category: sec-prop:formalAnalysisPriority
    threat: >-
      Two presentations of the same prover are identifiable as the same
      prover; two presentations involving the same credential are
      identifiable as such.
    defence: >-
      Pre-quantum: PoK-of-signature randomisation (BBS+ unlinkable;
      Schnorr over fresh nonce; PLONK proof-randomness fresh per
      presentation). Post-quantum: hiding-side of all commitments is
      statistical, so unlinkability survives a CRQC for already-issued
      proofs.
    openQuestion: >-
      Cross-presentation linkage when issuer signs with a deterministic
      scheme (ECDSA / ed25519 without a PoK lift) — must be lifted;
      document explicitly.
    wasDerivedFrom:
      - bibo:adler-2024-personhood
      - bibo:firstperson-network-2024
      - bibo:ietf-bbs-signatures-2025

  - id: sec-prop:SourceCredentialDisclosure
    type: sec-prop:SecurityProperty
    label: "Source-credential disclosure"
    prefLabel: "Disclosure of sources / credentials used in a result"
    paperSection: "§7.7 #2"
    category: sec-prop:informationDisclosure
    threat: >-
      Verifier learns which credentials in the holder's wallet
      contributed to a result, beyond what the answer entails.
    defence: >-
      Multiset-Merkle commitment is over the full signed dataset, not
      per-credential roots; the BGP match reveals only the bound terms,
      not the credential boundaries. Subject to the
      `spec/disclosure.md` "issuer-set" reveal toggle.
    openQuestion: >-
      Default policy is to disclose the issuer set (audit requirement);
      a "no issuer disclosure" mode requires an issuer-set accumulator
      (future work, links to AnonCreds revocation registries).
    wasDerivedFrom:
      - bibo:anoncreds-spec
      - bibo:w3c-rdfc10-2024

  - id: sec-prop:PostQuantumForgery
    type: sec-prop:SecurityProperty
    label: "Post-quantum forgery resistance"
    prefLabel: "Post-quantum forgery"
    paperSection: "§7.7 #3"
    category: sec-prop:formalAnalysisPriority
    threat: >-
      A CRQC adversary forges new credentials by recovering issuer keys,
      then constructs valid proofs.
    defence: >-
      Bounded by the issuer signature scheme. Today: Schnorr / EdDSA /
      BBS+ — all Shor-broken. Migration: ML-DSA (FIPS 204) or
      lattice-BBS (Tessaro-Zhu 2023, Boschini et al.). The encoding spec
      is signature-suite-agnostic, so the swap is local.
    openQuestion: >-
      No deployable lattice-BBS yet; ML-DSA-on-VC needs a W3C
      cryptosuite.
    wasDerivedFrom:
      - bibo:nist-fips204-2024
      - bibo:nist-ir8547-2024
      - bibo:tessaro-zhu-2023
      - bibo:cloudflare-pq-anoncreds-2024

  - id: sec-prop:PostQuantumSnooping
    type: sec-prop:SecurityProperty
    label: "Post-quantum snooping resistance (harvest-now-decrypt-later)"
    prefLabel: "Post-quantum snooping"
    paperSection: "§7.7 #4"
    category: sec-prop:formalAnalysisPriority
    threat: >-
      A CRQC adversary harvests stored proofs / commitments today and
      recovers hidden inputs (e.g. the credential's full triple set)
      once a CRQC arrives.
    defence: >-
      The hiding side of Pedersen, Poseidon2, and Blake3 is
      statistical / information-theoretic; pre-CRQC proofs maintain
      hiding indefinitely. PLONK witnesses preserve statistical HVZK.
    openQuestion: >-
      Pedersen binding breaks under CRQC, so retro-soundness fails (a
      CRQC can produce new fake proofs that look valid against an old
      vk) — verifiers should archive proofs alongside trust signals.
    wasDerivedFrom:
      - bibo:nist-ir8547-2024
      - bibo:gabizon-2019-plonk

  - id: sec-prop:SignatureTypeLeakage
    type: sec-prop:SecurityProperty
    label: "Signature-type leakage (PoKoS reveals signature type)"
    prefLabel: "Signature-type leakage"
    paperSection: "§7.7 #5"
    category: sec-prop:informationDisclosure
    threat: >-
      The proof of knowledge of signature reveals which signature scheme
      the input credential used; an attacker can scope the issuer set
      by knowing which issuers use BBS+ vs ECDSA vs ML-DSA.
    defence: >-
      Configurable suite-blinding via a generic PoK-of-discrete-log
      adapter inside the circuit (every supported suite reduces to a
      Schnorr-like proof of secret-knowledge over the appropriate
      group).
    openQuestion: >-
      Efficient circuit encoding for every suite the spec lists;
      today some suites add gate-count overhead disproportionate to
      the privacy gain.
    wasDerivedFrom:
      - bibo:ietf-bbs-signatures-2025
      - bibo:w3c-vc-di-bbs-2025
      - bibo:w3c-vc-di-ecdsa-2025

  - id: sec-prop:ProofSizeLeakage
    type: sec-prop:SecurityProperty
    label: "Proof-size leakage (e.g. path length on recursive paths)"
    prefLabel: "Proof-size leakage"
    paperSection: "§7.7 #6"
    category: sec-prop:informationDisclosure
    threat: >-
      Variable-size circuits leak structural information about the
      input — most acutely for property-path queries where the matched
      path length is encoded in the circuit shape.
    defence: >-
      Bounded unrolling via `path_segment_max` in `spec/config.md`; the
      path length is explicitly disclosed per the
      disclose-and-verify-externally principle. Optional patterns
      padded to `optional_cap_default` (= 4). All circuits compile to a
      fixed-size verifier-key per query template.
    openQuestion: >-
      Whether bucket-rounding query-template parameters reduces leakage
      further; engineering trade-off vs prover time.
    wasDerivedFrom:
      - bibo:w3c-sparql11-2013
      - bibo:perez-arenas-gutierrez-2009-semantics

  - id: sec-prop:CircuitAudit
    type: sec-prop:SecurityProperty
    label: "Circuit audit / proof-of-correctness"
    prefLabel: "Circuit audit"
    paperSection: "§7.7 #7"
    category: sec-prop:formalAnalysisPriority
    threat: >-
      A bug in the circuit generator could produce circuits whose
      accepting witness does not imply the claimed SPARQL semantics.
    defence: >-
      Mechanised soundness / completeness in Lean 4 (paper §6);
      per-primitive `*_unique` and `*_correct` lemmas in `proofs/`;
      cross-checked against the W3C SPARQL 1.0 evaluation test suite.
      Open obligations (`sorry`) are explicitly enumerated.
    openQuestion: >-
      Closing the full algebra-fragment proof; NAVe-based ACIR-level
      SMT verification once the toolchain pins a current Noir release.
    wasDerivedFrom:
      - bibo:lampe-2025
      - bibo:nave-2025
      - bibo:perez-arenas-gutierrez-2009-semantics

  - id: sec-prop:ValidityPeriodLeakage
    type: sec-prop:SecurityProperty
    label: "Validity-period leakage"
    prefLabel: "Validity-period leakage"
    paperSection: "§7.7 #8"
    category: sec-prop:deploymentSpec
    threat: >-
      Including credential validity periods in the result reveals (or
      narrows) which credentials sourced the answer.
    defence: >-
      Two modes — (a) reveal the intersection of validity periods only
      (information-theoretically minimal under composition); (b) prove
      validity-at-presentation-time inside the circuit and disclose only
      "valid as of timestamp T". `spec/disclosure.md` "validity-policy"
      toggle.
    openQuestion: >-
      Mode (a) is provably minimal but interacts non-trivially with
      revocation-list checks (which themselves leak); recommend
      mode (b) as the default for production.
    wasDerivedFrom:
      - bibo:w3c-vcdm20-2025
      - bibo:eu-2024-1183
