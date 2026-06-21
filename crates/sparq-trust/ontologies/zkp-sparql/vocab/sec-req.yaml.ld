# yaml-ld source for the three regulatory frameworks the paper is
# answerable to: eIDAS 2.0 (EU), NIST PQC (US), UK DVS (UK Digital
# Verification Services). Each is an instance of `sec-req:Requirement`
# carrying one or more deadlines and references to the
# `sec-prop:SecurityProperty` items it pulls on.
"@context":
  "@vocab": "https://w3id.org/zkp-sparql/sec-req#"
  sec-req:  "https://w3id.org/zkp-sparql/sec-req#"
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
  rationale:       { "@id": "sec-req:rationale", "@language": "en-GB" }
  jurisdiction:    { "@id": "sec-req:jurisdiction" }
  pulls:           { "@id": "sec-req:pulls", "@type": "@id", "@container": "@set" }
  hasDeadline:     { "@id": "sec-req:hasDeadline", "@type": "@id", "@container": "@set" }
  on:              { "@id": "sec-req:on", "@type": "xsd:date" }
  obligation:      { "@id": "sec-req:obligation", "@language": "en-GB" }
  reference:       { "@id": "dcterms:source", "@type": "@id" }
  wasDerivedFrom:  { "@id": "prov:wasDerivedFrom", "@type": "@id", "@container": "@set" }
  type:            "@type"
  id:              "@id"

"@graph":

  # ─────────── Class definitions ───────────
  - id: sec-req:Requirement
    type: [owl:Class, rdfs:Class]
    label: "Requirement"
    comment: >-
      A regulatory or normative requirement that the ZKP-of-SPARQL
      system must (or aspires to) satisfy. Pulls on one or more
      `sec-prop:SecurityProperty` items.

  - id: sec-req:Deadline
    type: [owl:Class, rdfs:Class]
    label: "Deadline"
    comment: >-
      A dated obligation under a requirement — e.g. NIST 2030
      "deprecate", NIST 2035 "disallow", eIDAS 2026/2027 wallet
      readiness milestones.

  # ─────────── Predicate definitions ───────────
  - id: sec-req:pulls
    type: owl:ObjectProperty
    label: "Pulls on"
    rdfs:domain: { id: sec-req:Requirement }
    rdfs:range:  { id: sec-prop:SecurityProperty }
    comment: >-
      Indicates that satisfying the requirement depends on attaining
      the named security property.
  - id: sec-req:hasDeadline
    type: owl:ObjectProperty
    label: "Has deadline"
    rdfs:domain: { id: sec-req:Requirement }
    rdfs:range:  { id: sec-req:Deadline }
  - id: sec-req:jurisdiction
    type: owl:DatatypeProperty
    label: "Jurisdiction"
    rdfs:domain: { id: sec-req:Requirement }
    rdfs:range:  { id: xsd:string }
  - id: sec-req:on
    type: owl:DatatypeProperty
    label: "On (date)"
    rdfs:domain: { id: sec-req:Deadline }
    rdfs:range:  { id: xsd:date }
  - id: sec-req:obligation
    type: owl:DatatypeProperty
    label: "Obligation"
    rdfs:domain: { id: sec-req:Deadline }
    rdfs:range:  { id: rdf:langString }

  # ─────────── eIDAS 2.0 ───────────
  - id: sec-req:Eidas20
    type: sec-req:Requirement
    label: "eIDAS 2.0 — EU Digital Identity Framework"
    jurisdiction: "EU"
    rationale: >-
      Regulation (EU) 2024/1183 amends eIDAS to introduce the European
      Digital Identity Wallet, with deployment obligations for member
      states and large online platforms. The wallet is the deployment
      target for ZKP-of-SPARQL presentations over verifiable
      credentials.
    pulls:
      - sec-prop:Unlinkability
      - sec-prop:SourceCredentialDisclosure
      - sec-prop:PostQuantumForgery
      - sec-prop:ValidityPeriodLeakage
    hasDeadline:
      - id: sec-req:eidas-deadline-2026
        type: sec-req:Deadline
        on: "2026-11-21"
        obligation: >-
          Member states must offer the EUDI Wallet to citizens
          (Regulation (EU) 2024/1183, deployment obligation).
      - id: sec-req:eidas-deadline-2027
        type: sec-req:Deadline
        on: "2027-05-21"
        obligation: >-
          Very Large Online Platforms must accept the EUDI Wallet for
          authentication.
    reference: bibo:eu-2024-1183
    wasDerivedFrom:
      - bibo:eu-2024-1183
      - bibo:eu-arf
      - bibo:european_digital_identity

  # ─────────── NIST PQC migration ───────────
  - id: sec-req:NistPqc
    type: sec-req:Requirement
    label: "NIST PQC migration timeline"
    jurisdiction: "US"
    rationale: >-
      NIST IR 8547 sets the migration schedule for legacy public-key
      cryptography: deprecate at the 112-bit security level by 2030,
      disallow by 2035. FIPS 204 (ML-DSA) is the standardised PQ
      digital-signature target.
    pulls:
      - sec-prop:PostQuantumForgery
      - sec-prop:PostQuantumSnooping
    hasDeadline:
      - id: sec-req:nist-deprecate-2030
        type: sec-req:Deadline
        on: "2030-12-31"
        obligation: >-
          Deprecate use of legacy public-key cryptography at the
          112-bit security level (RSA-2048, ECC-256, etc.) per
          NIST IR 8547.
      - id: sec-req:nist-disallow-2035
        type: sec-req:Deadline
        on: "2035-12-31"
        obligation: >-
          Disallow use of legacy public-key cryptography at the
          112-bit security level per NIST IR 8547.
    reference: bibo:nist-ir8547-2024
    wasDerivedFrom:
      - bibo:nist-fips204-2024
      - bibo:nist-ir8547-2024

  # ─────────── UK DVS ───────────
  - id: sec-req:UkDvs
    type: sec-req:Requirement
    label: "UK Digital Verification Services (DVS) framework"
    jurisdiction: "GB"
    rationale: >-
      The UK DVS framework (under DIATF 2024 + the Data
      (Use and Access) Bill 2024) sets certification and trust-mark
      requirements for digital verification providers. The framework
      pulls on unlinkability and source-credential disclosure for
      privacy compliance, and on validity-period semantics for the
      digital driving licence rollout.
    pulls:
      - sec-prop:Unlinkability
      - sec-prop:SourceCredentialDisclosure
      - sec-prop:ValidityPeriodLeakage
      - sec-prop:CircuitAudit
    hasDeadline:
      - id: sec-req:ukdvs-bill-royal-assent
        type: sec-req:Deadline
        on: "2025-06-19"
        obligation: >-
          Royal Assent of the Data (Use and Access) Act 2025
          (formerly the Data Bill 2024); statutory framework for DVS
          enters force.
      - id: sec-req:ukdvs-driving-licence-2025
        type: sec-req:Deadline
        on: "2025-12-31"
        obligation: >-
          Indicative target for the UK digital driving licence rollout
          announced 2025; first DVS-certified holders.
    reference: bibo:ukgov-data-bill-2024
    wasDerivedFrom:
      - bibo:ukgov-data-bill-2024
      - bibo:ukgov-diatf-2024
      - bibo:govuk_digital_driving_licence_2025
