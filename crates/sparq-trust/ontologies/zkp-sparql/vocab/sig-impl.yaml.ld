# yaml-ld source for the four signature implementations the paper considers
# at round 1: BBS+, SD-JWT-VC, ed25519, ECDSA. Each is an instance of
# `sig-impl:SignatureImplementation`. The relationship between an
# implementation and a `sec-prop:SecurityProperty` is captured by an
# `sig-impl:Assertion` reified node: this lets us record yes/no/partial
# verdicts plus a justification and a provenance pointer per assertion.
"@context":
  "@vocab": "https://w3id.org/zkp-sparql/sig-impl#"
  sig-impl: "https://w3id.org/zkp-sparql/sig-impl#"
  sec-prop: "https://w3id.org/zkp-sparql/sec-prop#"
  rdf:      "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  rdfs:     "http://www.w3.org/2000/01/rdf-schema#"
  owl:      "http://www.w3.org/2002/07/owl#"
  skos:     "http://www.w3.org/2004/02/skos/core#"
  prov:     "http://www.w3.org/ns/prov#"
  bibo:     "http://purl.org/ontology/bibo/"
  xsd:      "http://www.w3.org/2001/XMLSchema#"
  label:           { "@id": "rdfs:label",   "@language": "en-GB" }
  comment:         { "@id": "rdfs:comment", "@language": "en-GB" }
  justification:   { "@id": "sig-impl:justification", "@language": "en-GB" }
  achieves:        { "@id": "sig-impl:achieves",  "@type": "@id", "@container": "@set" }
  property:        { "@id": "sig-impl:property", "@type": "@id" }
  verdict:         { "@id": "sig-impl:verdict",  "@type": "@id" }
  postQuantum:     { "@id": "sig-impl:postQuantum", "@type": "xsd:boolean" }
  family:          { "@id": "sig-impl:family",   "@type": "@id" }
  wasDerivedFrom:  { "@id": "prov:wasDerivedFrom", "@type": "@id", "@container": "@set" }
  type:            "@type"
  id:              "@id"

"@graph":

  # ─────────── Class definitions ───────────
  - id: sig-impl:SignatureImplementation
    type: [owl:Class, rdfs:Class]
    label: "Signature implementation"
    comment: >-
      A concrete signature scheme (and its W3C / IETF / ISO binding)
      that the encoding layer of the paper accepts as an issuer
      signature suite.

  - id: sig-impl:Assertion
    type: [owl:Class, rdfs:Class]
    label: "Property assertion"
    comment: >-
      A reified statement that a signature implementation achieves /
      fails / partially achieves a security property. Reification lets
      us attach a justification and provenance per assertion.

  - id: sig-impl:Verdict
    type: [owl:Class, rdfs:Class]
    label: "Verdict"
    comment: "Assertion outcome: yes, no, partial."

  - id: sig-impl:yes
    type: sig-impl:Verdict
    label: "Yes — the property holds for this implementation."
  - id: sig-impl:no
    type: sig-impl:Verdict
    label: "No — the property does not hold for this implementation."
  - id: sig-impl:partial
    type: sig-impl:Verdict
    label: "Partial — holds under additional conditions or with caveats."

  - id: sig-impl:Family
    type: [owl:Class, rdfs:Class]
    label: "Signature family"
    comment: "Coarse cryptographic family (Schnorr-like, BBS, ECDSA, lattice)."

  - id: sig-impl:bbs
    type: sig-impl:Family
    label: "BBS family (pairing-based, group-signature lineage)"
  - id: sig-impl:schnorr
    type: sig-impl:Family
    label: "Schnorr-like family (ed25519, EdDSA)"
  - id: sig-impl:ecdsa
    type: sig-impl:Family
    label: "ECDSA family"
  - id: sig-impl:sdJwt
    type: sig-impl:Family
    label: "SD-JWT envelope (any underlying JOSE-supported algorithm)"

  # ─────────── Predicate definitions ───────────
  - id: sig-impl:achieves
    type: owl:ObjectProperty
    label: "Achieves"
    rdfs:domain: { id: sig-impl:SignatureImplementation }
    rdfs:range:  { id: sig-impl:Assertion }
  - id: sig-impl:property
    type: owl:ObjectProperty
    label: "Property"
    rdfs:domain: { id: sig-impl:Assertion }
    rdfs:range:  { id: sec-prop:SecurityProperty }
  - id: sig-impl:verdict
    type: owl:ObjectProperty
    label: "Verdict"
    rdfs:domain: { id: sig-impl:Assertion }
    rdfs:range:  { id: sig-impl:Verdict }
  - id: sig-impl:justification
    type: owl:DatatypeProperty
    label: "Justification"
    rdfs:domain: { id: sig-impl:Assertion }
    rdfs:range:  { id: rdf:langString }
  - id: sig-impl:postQuantum
    type: owl:DatatypeProperty
    label: "Post-quantum"
    rdfs:domain: { id: sig-impl:SignatureImplementation }
    rdfs:range:  { id: xsd:boolean }
  - id: sig-impl:family
    type: owl:ObjectProperty
    label: "Family"
    rdfs:domain: { id: sig-impl:SignatureImplementation }
    rdfs:range:  { id: sig-impl:Family }

  # ─────────── BBS+ ───────────
  - id: sig-impl:BbsPlus
    type: sig-impl:SignatureImplementation
    label: "BBS+ signatures (CFRG draft + W3C VC-DI-BBS Cryptosuites 1.0)"
    comment: >-
      Pairing-based signature scheme supporting selective disclosure
      and unlinkability natively.
    family: sig-impl:bbs
    postQuantum: false
    wasDerivedFrom:
      - bibo:ietf-bbs-signatures-2025
      - bibo:w3c-vc-di-bbs-2025
      - bibo:boneh-boyen-shacham-2004
    achieves:
      - id: sig-impl:assert-bbs-unlinkability
        type: sig-impl:Assertion
        property: sec-prop:Unlinkability
        verdict: sig-impl:yes
        justification: >-
          BBS+ supports randomisable proofs of knowledge of signature;
          two presentations of the same credential are unlinkable
          provided each presentation derives a fresh randomisation.
        wasDerivedFrom:
          - bibo:ietf-bbs-signatures-2025
          - bibo:tessaro-zhu-2023
      - id: sig-impl:assert-bbs-pq-forgery
        type: sig-impl:Assertion
        property: sec-prop:PostQuantumForgery
        verdict: sig-impl:no
        justification: >-
          BBS+ is Shor-broken; a CRQC can recover the issuer key from
          public-key material, and lattice-BBS analogues are not yet
          deployable in W3C cryptosuites.
        wasDerivedFrom:
          - bibo:nist-ir8547-2024
          - bibo:tessaro-zhu-2023
      - id: sig-impl:assert-bbs-pq-snooping
        type: sig-impl:Assertion
        property: sec-prop:PostQuantumSnooping
        verdict: sig-impl:partial
        justification: >-
          The hiding side of the PoK transcript is statistical, so
          archived BBS+ presentations remain hidden under CRQC; the
          binding side breaks, so retroactive forgery is possible.
        wasDerivedFrom:
          - bibo:nist-ir8547-2024
      - id: sig-impl:assert-bbs-sigtype-leakage
        type: sig-impl:Assertion
        property: sec-prop:SignatureTypeLeakage
        verdict: sig-impl:no
        justification: >-
          The PoK transcript is shaped distinctively (BLS12-381
          pairing-friendly group, multi-message structure); a verifier
          can identify a BBS+ presentation by transcript shape unless
          a generic PoK-of-discrete-log adapter is used.
        wasDerivedFrom:
          - bibo:ietf-bbs-signatures-2025

  # ─────────── SD-JWT-VC ───────────
  - id: sig-impl:SdJwtVc
    type: sig-impl:SignatureImplementation
    label: "SD-JWT-VC (selective-disclosure JWT VC)"
    comment: >-
      Salted-hash selective-disclosure envelope over a JWT. The
      underlying signature scheme is JOSE-determined; round-1 records
      ed25519/ECDSA-P256 as the deployed pair.
    family: sig-impl:sdJwt
    postQuantum: false
    wasDerivedFrom:
      - bibo:ietf-sd-jwt-vc-2025
      - bibo:ietf-sd-jwt-vc-format-2024
    achieves:
      - id: sig-impl:assert-sdjwt-unlinkability
        type: sig-impl:Assertion
        property: sec-prop:Unlinkability
        verdict: sig-impl:no
        justification: >-
          The same envelope (issuer signature over SD-JWT body) is
          replayed across presentations; verifiers can correlate
          presentations by signature value alone unless rewrapped per
          presentation, which is not standard.
        wasDerivedFrom:
          - bibo:ietf-sd-jwt-vc-2025
      - id: sig-impl:assert-sdjwt-pq-forgery
        type: sig-impl:Assertion
        property: sec-prop:PostQuantumForgery
        verdict: sig-impl:no
        justification: >-
          The default JOSE algorithms (ed25519, ECDSA-P256, RSA) are
          all Shor-broken. PQ-JOSE migration (ML-DSA) is in flight at
          IETF but not yet deployed in SD-JWT-VC profiles.
        wasDerivedFrom:
          - bibo:nist-fips204-2024
          - bibo:nist-ir8547-2024
      - id: sig-impl:assert-sdjwt-source-disclosure
        type: sig-impl:Assertion
        property: sec-prop:SourceCredentialDisclosure
        verdict: sig-impl:partial
        justification: >-
          Selective disclosure hides individual claim values inside a
          credential; the credential boundary itself is still revealed
          per-envelope (one envelope per credential).
        wasDerivedFrom:
          - bibo:ietf-sd-jwt-vc-format-2024

  # ─────────── ed25519 ───────────
  - id: sig-impl:Ed25519
    type: sig-impl:SignatureImplementation
    label: "ed25519 (RFC 8032 EdDSA over Curve25519)"
    comment: >-
      Deterministic Schnorr-like signature; widely deployed in JOSE,
      did:key, and W3C VC-DI-EdDSA.
    family: sig-impl:schnorr
    postQuantum: false
    wasDerivedFrom:
      - bibo:rfc7519
      - bibo:johnson2001elliptic
    achieves:
      - id: sig-impl:assert-ed25519-unlinkability
        type: sig-impl:Assertion
        property: sec-prop:Unlinkability
        verdict: sig-impl:no
        justification: >-
          ed25519 is deterministic; the signature value over a fixed
          message is identical across presentations and is replayed
          verbatim if the credential is re-shown. A PoK-of-signature
          lift (Schnorr-on-Schnorr) is required for unlinkability and
          is not part of stock VC-DI-EdDSA.
        wasDerivedFrom:
          - bibo:johnson2001elliptic
          - bibo:adler-2024-personhood
      - id: sig-impl:assert-ed25519-pq-forgery
        type: sig-impl:Assertion
        property: sec-prop:PostQuantumForgery
        verdict: sig-impl:no
        justification: >-
          Curve25519 is Shor-broken; a CRQC recovers the issuer secret
          key in polynomial time.
        wasDerivedFrom:
          - bibo:nist-ir8547-2024
      - id: sig-impl:assert-ed25519-sigtype-leakage
        type: sig-impl:Assertion
        property: sec-prop:SignatureTypeLeakage
        verdict: sig-impl:partial
        justification: >-
          The 64-byte fixed-size signature reveals the algorithm class
          (Schnorr / EdDSA) but not the issuer; a generic PoK adapter
          inside the circuit can blind further.
        wasDerivedFrom:
          - bibo:ietf-bbs-signatures-2025

  # ─────────── ECDSA ───────────
  - id: sig-impl:Ecdsa
    type: sig-impl:SignatureImplementation
    label: "ECDSA (NIST P-256 / secp256k1)"
    comment: >-
      Standardised elliptic-curve signature scheme; the dominant W3C
      VC-DI-ECDSA suite.
    family: sig-impl:ecdsa
    postQuantum: false
    wasDerivedFrom:
      - bibo:w3c-vc-di-ecdsa-2025
      - bibo:ecdsa2023
      - bibo:johnson2001elliptic
    achieves:
      - id: sig-impl:assert-ecdsa-unlinkability
        type: sig-impl:Assertion
        property: sec-prop:Unlinkability
        verdict: sig-impl:no
        justification: >-
          Standard ECDSA is randomised but the signature value is
          replayed across presentations; verifiers can correlate. A
          PoK-of-signature lift (Crescent's approach) is required for
          unlinkability.
        wasDerivedFrom:
          - bibo:paquin-policharla-zaverucha-2024-crescent
      - id: sig-impl:assert-ecdsa-pq-forgery
        type: sig-impl:Assertion
        property: sec-prop:PostQuantumForgery
        verdict: sig-impl:no
        justification: >-
          ECDSA is Shor-broken; a CRQC recovers issuer secrets from
          public-key material.
        wasDerivedFrom:
          - bibo:nist-ir8547-2024
      - id: sig-impl:assert-ecdsa-sigtype-leakage
        type: sig-impl:Assertion
        property: sec-prop:SignatureTypeLeakage
        verdict: sig-impl:partial
        justification: >-
          The (r, s) signature value is shape-distinguishable from
          ed25519 / BBS+; suite-blinding requires a generic PoK
          adapter.
        wasDerivedFrom:
          - bibo:w3c-vc-di-ecdsa-2025
