# yaml-ld source for the bibliographic provenance extension. Each entry
# corresponds to a BibTeX key in `notes/research/08-bibliography.md` of the
# parent workspace. We materialise these as `bibo:Document` instances so
# the `prov:wasDerivedFrom` references in `sec-prop`, `sig-impl`, and
# `sec-req` resolve under SHACL closed-shape validation.
#
# Where a Zenodo or DOI URL is known, `dcterms:identifier` and
# `bibo:doi` carry the dereferenceable identifier so a downstream BibTeX
# generator can pull a canonical reference. URLs that are not DOIs
# (W3C drafts, GitHub repos, IETF datatracker pages) go in `dcterms:source`
# / `prov:atLocation`.
#
# Custom terms in this file are kept to the minimum: we only define
# `prov-ext:bibtexKey` because no FAIR-friendly term in `bibo:` /
# `dcterms:` / `prov:` carries the BibTeX-key role unambiguously.
"@context":
  "@vocab": "https://w3id.org/zkp-sparql/prov-ext#"
  prov-ext: "https://w3id.org/zkp-sparql/prov-ext#"
  bibo:     "http://purl.org/ontology/bibo/"
  dcterms:  "http://purl.org/dc/terms/"
  rdf:      "http://www.w3.org/1999/02/22-rdf-syntax-ns#"
  rdfs:     "http://www.w3.org/2000/01/rdf-schema#"
  owl:      "http://www.w3.org/2002/07/owl#"
  prov:     "http://www.w3.org/ns/prov#"
  xsd:      "http://www.w3.org/2001/XMLSchema#"
  label:        { "@id": "rdfs:label",   "@language": "en-GB" }
  comment:      { "@id": "rdfs:comment", "@language": "en-GB" }
  title:        { "@id": "dcterms:title", "@language": "en-GB" }
  bibtexKey:    { "@id": "prov-ext:bibtexKey" }
  doi:          { "@id": "bibo:doi" }
  url:          { "@id": "dcterms:source", "@type": "@id" }
  identifier:   { "@id": "dcterms:identifier" }
  date:         { "@id": "dcterms:issued", "@type": "xsd:gYear" }
  # Round-2 additions: aliases for existing bibo / dcterms terms used to
  # carry canonical bibliographic metadata. No new terms are coined here
  # — every property below already has a stable IRI in bibo / dcterms.
  creator:      { "@id": "dcterms:creator", "@language": "en-GB" }
  venue:        { "@id": "dcterms:bibliographicCitation", "@language": "en-GB" }
  arxivId:      { "@id": "bibo:identifier" }
  pages:        { "@id": "bibo:pages" }
  volume:       { "@id": "bibo:volume" }
  abstract:     { "@id": "bibo:abstract", "@language": "en-GB" }
  publisher:    { "@id": "dcterms:publisher", "@language": "en-GB" }
  notes:        { "@id": "rdfs:comment", "@language": "en-GB" }
  type:         "@type"
  id:           "@id"

"@graph":

  # ─────────── Property definition ───────────
  - id: prov-ext:bibtexKey
    type: owl:DatatypeProperty
    label: "BibTeX citation key"
    comment: >-
      The BibTeX key under which this document appears in the paper's
      bibliography (`notes/research/08-bibliography.md`). Stable across
      paper revisions; the canonical reference for cross-referencing.
    rdfs:domain: { id: bibo:Document }
    rdfs:range:  { id: xsd:string }

  # ─────────── Bibliographic instances ───────────

  # SPARQL / RDF foundations
  - id: bibo:w3c-sparql11-2013
    type: bibo:Document
    bibtexKey: "w3c-sparql11-2013"
    title: "SPARQL 1.1 Query Language"
    url: "https://www.w3.org/TR/sparql11-query/"
    date: "2013"
  - id: bibo:perez-arenas-gutierrez-2009-semantics
    type: bibo:Document
    bibtexKey: "perez-arenas-gutierrez-2009-semantics"
    title: "Semantics and Complexity of SPARQL"
    doi: "10.1145/1567274.1567278"
    date: "2009"
  - id: bibo:w3c-rdfc10-2024
    type: bibo:Document
    bibtexKey: "w3c-rdfc10-2024"
    title: "RDF Dataset Canonicalization 1.0 (RDFC-1.0)"
    url: "https://www.w3.org/TR/rdf-canon/"
    date: "2024"

  # Verifiable Credentials & data-integrity
  - id: bibo:w3c-vcdm20-2025
    type: bibo:Document
    bibtexKey: "w3c-vcdm20-2025"
    title: "Verifiable Credentials Data Model 2.0"
    url: "https://www.w3.org/TR/vc-data-model-2.0/"
    date: "2025"
  - id: bibo:w3c-vc-di-bbs-2025
    type: bibo:Document
    bibtexKey: "w3c-vc-di-bbs-2025"
    title: "Verifiable Credentials Data Integrity BBS Cryptosuites 1.0"
    url: "https://www.w3.org/TR/vc-di-bbs/"
    date: "2025"
  - id: bibo:w3c-vc-di-ecdsa-2025
    type: bibo:Document
    bibtexKey: "w3c-vc-di-ecdsa-2025"
    title: "Verifiable Credentials Data Integrity ECDSA Cryptosuites 1.0"
    url: "https://www.w3.org/TR/vc-di-ecdsa/"
    date: "2025"
  - id: bibo:ietf-sd-jwt-vc-2025
    type: bibo:Document
    bibtexKey: "ietf-sd-jwt-vc-2025"
    title: "Selective Disclosure for JWTs (SD-JWT)"
    url: "https://datatracker.ietf.org/doc/draft-ietf-oauth-selective-disclosure-jwt/"
    date: "2025"
  - id: bibo:ietf-sd-jwt-vc-format-2024
    type: bibo:Document
    bibtexKey: "ietf-sd-jwt-vc-format-2024"
    title: "SD-JWT-based Verifiable Credentials (SD-JWT VC)"
    url: "https://datatracker.ietf.org/doc/draft-ietf-oauth-sd-jwt-vc/"
    date: "2024"
  - id: bibo:ietf-bbs-signatures-2025
    type: bibo:Document
    bibtexKey: "ietf-bbs-signatures-2025"
    title: "The BBS Signature Scheme"
    url: "https://identity.foundation/bbs-signature/draft-irtf-cfrg-bbs-signatures.html"
    date: "2025"
  - id: bibo:anoncreds-spec
    type: bibo:Document
    bibtexKey: "anoncreds-spec"
    title: "AnonCreds Specification"
    url: "https://hyperledger.github.io/anoncreds-spec/"
    date: "2024"
  - id: bibo:rfc7519
    type: bibo:Document
    bibtexKey: "rfc7519"
    title: "JSON Web Token (JWT)"
    url: "https://www.rfc-editor.org/rfc/rfc7519"
    date: "2015"

  # ZK over credentials (signature-level)
  - id: bibo:boneh-boyen-shacham-2004
    type: bibo:Document
    bibtexKey: "boneh-boyen-shacham-2004"
    title: "Short Group Signatures"
    doi: "10.1007/978-3-540-28628-8_3"
    date: "2004"
  - id: bibo:paquin-policharla-zaverucha-2024-crescent
    type: bibo:Document
    bibtexKey: "paquin-policharla-zaverucha-2024-crescent"
    title: "Crescent: Stronger Privacy for Existing Credentials"
    url: "https://eprint.iacr.org/2024/2013"
    date: "2024"
  - id: bibo:tessaro-zhu-2023
    type: bibo:Document
    bibtexKey: "tessaro-zhu-2023"
    title: "Revisiting BBS Signatures"
    creator: "Stefano Tessaro; Chenzhi Zhu"
    doi: "10.1007/978-3-031-30589-4_24"
    url: "https://doi.org/10.1007/978-3-031-30589-4_24"
    venue: "Advances in Cryptology — EUROCRYPT 2023, Part V, pp. 691–721"
    pages: "691-721"
    publisher: "Springer"
    identifier: "eprint.iacr.org/2023/275"
    date: "2023"
    notes: >-
      DBLP `conf/eurocrypt/TessaroZ23`. IACR ePrint preprint at
      https://eprint.iacr.org/2023/275 carries the same title and authors.
  - id: bibo:adler-2024-personhood
    type: bibo:Document
    bibtexKey: "adler-2024-personhood"
    title: >-
      Personhood Credentials: Artificial Intelligence and the Value of
      Privacy-Preserving Tools to Distinguish Who Is Real Online
    creator: >-
      Steven Adler; Zoë Hitzig; Shrey Jain; Catherine Brewer; Wayne Chang;
      Renée DiResta; Eddy Lazzarin; Sean McGregor; Wendy Seltzer;
      Divya Siddarth; Nouran Soliman; Tobin South; Connor Spelliscy;
      Manu Sporny; Varya Srivastava; John Bailey; Brian Christian;
      Andrew Critch; Ronnie Falcon; Heather Flanagan; Kim Hamilton Duffy;
      Eric Ho; Claire R. Leibowicz; Srikanth Nadhamuni;
      Alan Z. Rozenshtein; David Schnurr; Evan Shapiro; Lacey Strahm;
      Andrew Trask; Zoe Weinberg; Cedric Whitney; Tom Zick
    doi: "10.48550/arXiv.2408.07892"
    url: "https://arxiv.org/abs/2408.07892"
    arxivId: "arXiv:2408.07892"
    venue: "arXiv preprint, cs.CY"
    date: "2024"
    abstract: >-
      Anonymity is an important principle online; however, malicious
      actors have long used misleading identities to conduct fraud,
      spread disinformation, and carry out other deceptive schemes.
  - id: bibo:firstperson-network-2024
    type: bibo:Document
    bibtexKey: "firstperson-network-2024"
    title: "The First Person Project White Paper, v1.2"
    creator: >-
      Linux Foundation Decentralized Trust (LFDT);
      Ayra Association;
      Trust Over IP Foundation;
      Decentralized Identity Foundation;
      OpenWallet Foundation
    url: "https://www.firstperson.network/white-paper"
    venue: "First Person Network white paper, v1.2"
    date: "2024"
    notes: >-
      No DOI / ISBN; multi-organisation industry whitepaper. Citing the
      v1.2 PDF available from the project landing page; bibliography
      preserves 2024 to match the citation key, while the publisher
      copyright was updated to 2026 after v1.2 release.

  # ZK proof systems and verification tooling
  - id: bibo:gabizon-2019-plonk
    type: bibo:Document
    bibtexKey: "gabizon-2019-plonk"
    title: "PLONK: Permutations over Lagrange-bases for Oecumenical Noninteractive arguments of Knowledge"
    url: "https://eprint.iacr.org/2019/953"
    date: "2019"
  - id: bibo:lampe-2025
    type: bibo:Document
    bibtexKey: "lampe-2025"
    title: >-
      Lampe: Extracting the Semantics of Noir to Lean for Formal
      Verification
    creator: "Reilabs"
    url: "https://github.com/reilabs/lampe"
    venue: "GitHub repository (Apache-2.0); no tagged releases"
    date: "2025"
    notes: >-
      Source-code-only release; no peer-reviewed paper, no Zenodo
      DOI, no tagged release at the time of citation. Lean 4 + Noir +
      Rust codebase under reilabs/lampe.
  - id: bibo:nave-2025
    type: bibo:Document
    bibtexKey: "nave-2025"
    title: "Formally Verifying Noir Zero Knowledge Programs with NAVe"
    creator: "Pedro Antonino; Namrata Jain"
    doi: "10.48550/arXiv.2601.09372"
    url: "https://arxiv.org/abs/2601.09372"
    arxivId: "arXiv:2601.09372"
    venue: "arXiv preprint, cs.CR"
    date: "2026"
    notes: >-
      Bibtex key carries `2025` for citation continuity with the round-1
      vocab; the canonical preprint posted in January 2026
      (arXiv:2601.09372) is the citable artefact. Companion code at
      github.com/pedrotbtl/nave (dual Apache-2.0 / MIT, Rust).

  # PQ regulation
  - id: bibo:nist-fips204-2024
    type: bibo:Document
    bibtexKey: "nist-fips204-2024"
    title: "FIPS 204: Module-Lattice-Based Digital Signature Standard (ML-DSA)"
    url: "https://csrc.nist.gov/pubs/fips/204/final"
    date: "2024"
  - id: bibo:nist-ir8547-2024
    type: bibo:Document
    bibtexKey: "nist-ir8547-2024"
    title: "NIST IR 8547: Transition to Post-Quantum Cryptography Standards"
    url: "https://csrc.nist.gov/pubs/ir/8547/ipd"
    date: "2024"
  - id: bibo:eu-2024-1183
    type: bibo:Document
    bibtexKey: "eu-2024-1183"
    title: "Regulation (EU) 2024/1183 — European Digital Identity Framework"
    url: "https://eur-lex.europa.eu/eli/reg/2024/1183/oj"
    date: "2024"
  - id: bibo:eu-arf
    type: bibo:Document
    bibtexKey: "eu-arf"
    title: "EU Digital Identity Wallet — Architecture and Reference Framework"
    url: "https://eu-digital-identity-wallet.github.io/eudi-doc-architecture-and-reference-framework/"
    date: "2024"
  - id: bibo:european_digital_identity
    type: bibo:Document
    bibtexKey: "european_digital_identity"
    title: "European Digital Identity"
    creator: "European Commission"
    url: "https://commission.europa.eu/strategy-and-policy/priorities-2019-2024/europe-fit-digital-age/european-digital-identity_en"
    venue: "European Commission, Priorities 2019–2024 (Europe fit for the digital age)"
    date: "2025"
    notes: >-
      Living Commission landing page; no DOI. Mirrors the bibliography
      `@misc{european_digital_identity}` entry at year=2025.
  - id: bibo:cloudflare-pq-anoncreds-2024
    type: bibo:Document
    bibtexKey: "cloudflare-pq-anoncreds-2024"
    title: >-
      Policy, privacy and post-quantum: anonymous credentials for everyone
    creator: "Lena Heimberger; Christopher Patton"
    url: "https://blog.cloudflare.com/policy-privacy-and-post-quantum-anonymous-credentials-for-everyone/"
    venue: "Cloudflare Research blog"
    date: "2025"
    notes: >-
      Bibtex key retains `2024` for citation continuity with the round-1
      vocab; canonical post is dated 30 October 2025 at the slug
      `policy-privacy-and-post-quantum-anonymous-credentials-for-everyone`.
      Bibliography updated to match.

  # UK DVS
  - id: bibo:ukgov-data-bill-2024
    type: bibo:Document
    bibtexKey: "ukgov-data-bill-2024"
    title: "Data (Use and Access) Bill 2024 (UK)"
    url: "https://bills.parliament.uk/bills/3825"
    date: "2024"
  - id: bibo:ukgov-diatf-2024
    type: bibo:Document
    bibtexKey: "ukgov-diatf-2024"
    title: "UK Digital Identity and Attributes Trust Framework (DIATF) 2024"
    url: "https://www.gov.uk/government/publications/uk-digital-identity-and-attributes-trust-framework"
    date: "2024"
  - id: bibo:govuk_digital_driving_licence_2025
    type: bibo:Document
    bibtexKey: "govuk_digital_driving_licence_2025"
    title: "Digital Driving Licence Coming This Year"
    creator: >-
      Department for Science, Innovation and Technology;
      Government Digital Service;
      Department for Transport;
      Ministry of Defence;
      Office for Veterans' Affairs
    url: "https://www.gov.uk/government/news/digital-driving-licence-coming-this-year"
    venue: "GOV.UK news, 21 January 2025"
    date: "2025"
    notes: >-
      UK government press release; no DOI. Cited as evidence of UK DVS
      rollout commitment for round-1 sec-req.

  # ECDSA / curve foundations
  - id: bibo:ecdsa2023
    type: bibo:Document
    bibtexKey: "ecdsa2023"
    title: "Data Integrity ECDSA Cryptosuites v1.0"
    creator: "Manu Sporny; Dave Longley; Greg Bernstein"
    url: "https://www.w3.org/TR/vc-di-ecdsa/"
    venue: "W3C Recommendation, 15 May 2025"
    date: "2025"
    notes: >-
      CEUR-side citation alias for the W3C VC-DI ECDSA Cryptosuites
      Recommendation; identical content to `bibo:w3c-vc-di-ecdsa-2025`
      but kept as a separate bibtex key for citation continuity with
      Wright's CEUR Vol-4085 paper-19 / Oxford-Berkeley-KIT skeleton.
      Despite the `2023` suffix the canonical Recommendation date is
      2025; bibliography preserves the suffix per §I notes.
  - id: bibo:johnson2001elliptic
    type: bibo:Document
    bibtexKey: "johnson2001elliptic"
    title: "The Elliptic Curve Digital Signature Algorithm (ECDSA)"
    creator: "Don Johnson; Alfred Menezes; Scott Vanstone"
    doi: "10.1007/s102070100002"
    url: "https://doi.org/10.1007/s102070100002"
    venue: "International Journal of Information Security, 1(1), pp. 36–63"
    pages: "36-63"
    volume: "1"
    publisher: "Springer"
    date: "2001"
