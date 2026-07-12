# trust-graph-findings.yaml.ld — the WRITE-PATH authoring form for the trust-graph
# admission-program provenance tier of the PKG (bead sq-7utko; epic sq-pfae context).
# 🤖 SPARQ agent [FABLE-5].
#
# WHY THIS FILE EXISTS. The PKG ingests AGENTS.md / skills / beads, but did not surface
# the trust-graph admission program's provenance: which record decided the
# certification-closure model, and what its honest constraints are. An agent asking
# "what is the status of trust-graph unlinkability" should get a SOURCED, caveat-carrying
# answer from a query — not read the whole design record. This file authors that tier;
# the deterministic compiler crates/sparq-kb/ingest/yamlld_compile.py expands it to
# SHACL-conformant Turtle (trust-graph-findings.ttl), exactly like the AGENTS.md tier
# (agents-findings.yaml.ld). Recompile after editing:
#
#   python3 crates/sparq-kb/ingest/yamlld_compile.py \
#       crates/sparq-kb/ingest/trust-graph-findings.yaml.ld \
#       --out crates/sparq-kb/ingest/trust-graph-findings.ttl
#
# HONESTY (load-bearing, per the source record's own framing): every statement below
# CARRIES its caveat — the sparq ZK verifier has NO external accredited-cryptographer
# sign-off (open gate sq-qhy4), sparq-mpc is honest-majority semi-honest only, and the
# trust-graph layer is a clear-path mechanism that makes NO anonymity, unlinkability,
# or ZK-soundness claim. Nothing here upgrades a hedged claim.
#
# NOTE on the Source IRI: kb:doc-research-trust-graph-authorisation-2026-07 is exactly
# the IRI ingest_pkg.py's research_doc_iri() mints for bd spec-links to this record, and
# the exploredStatus / followUpPriority / confidence / identifier values byte-match its
# emission — so if a future pkg-instances.ttl regeneration also mints the Source from a
# bead reference, the shared triples deduplicate instead of conflicting.

"@context":
  "@vocab": "https://sparq.dev/ns/pkg#"
  pkg:      "https://sparq.dev/ns/pkg#"
  kb:       "https://sparq.dev/ns/pkg/kb#"
  prov:     "http://www.w3.org/ns/prov#"
  skos:     "http://www.w3.org/2004/02/skos/core#"
  dcterms:  "http://purl.org/dc/terms/"
  fabio:    "http://purl.org/spar/fabio/"
  cito:     "http://purl.org/spar/cito/"
  sigimpl:  "https://w3id.org/zkp-sparql/sig-impl#"
  secx:     "https://w3id.org/zkp-sparql/sec-prop#"
  rdfs:     "http://www.w3.org/2000/01/rdf-schema#"
  xsd:      "http://www.w3.org/2001/XMLSchema#"
  # Author-facing fluent keys -> FO predicates (no IRIs/xsd in the @graph below).
  finding:        { "@id": "rdfs:label",            "@language": "en-GB" }
  prefLabel:      { "@id": "skos:prefLabel",        "@language": "en-GB" }
  definition:     { "@id": "skos:definition",       "@language": "en-GB" }
  inScheme:       { "@id": "skos:inScheme",         "@type": "@id" }
  about:          { "@id": "pkg:about",             "@type": "@token" }   # concept TOKEN
  subject:        { "@id": "dcterms:subject",       "@type": "@token" }   # concept TOKEN
  source:         { "@id": "prov:wasDerivedFrom",   "@type": "@id" }
  evidence:       { "@id": "cito:citesAsEvidence",  "@type": "@id" }
  fromSection:    { "@id": "dcterms:source" }
  verdict:        { "@id": "pkg:verdict",           "@type": "@verdict" }
  justification:  { "@id": "sigimpl:justification", "@language": "en-GB" }
  confidence:     { "@id": "pkg:confidence",        "@type": "xsd:decimal" }
  assurance:      { "@id": "pkg:assurance",         "@type": "@assurance" }
  exploredStatus: { "@id": "pkg:exploredStatus",    "@type": "@id" }
  followUpPriority: { "@id": "pkg:followUpPriority", "@type": "xsd:integer" }
  identifier:     { "@id": "dcterms:identifier" }
  format:         { "@id": "dcterms:format" }
  title:          { "@id": "dcterms:title",         "@language": "en-GB" }

# The concept catalog the guarded V() resolver grounds TOKENS against.
concepts:
  - "@id": kb:topic-trust-graph-authz
    "@type": skos:Concept
    inScheme: pkg:Topics
    prefLabel: "Trust-graph authorisation"
    definition: "The sq-pfae trust-graph admission program for Solid/LWS — certification-edge closure into effective trust rules, its clear-path honesty constraints, and the sq-qhy4 external-audit gate."

# The Source node every Finding below is derived from: the 2026-07 estate-audit +
# certification-closure design record. IRI + shared field values byte-match
# ingest_pkg.py's research-doc Source emission (see the NOTE above).
sources:
  - "@id": kb:doc-research-trust-graph-authorisation-2026-07
    "@type": [pkg:Source, pkg:Document, fabio:Expression]
    title: "research/trust-graph-authorisation-2026-07.md — trust-graph estate audit + certification-closure design record"
    identifier: "research/trust-graph-authorisation-2026-07.md"
    format: "text/markdown"
    exploredStatus: pkg:Explored
    followUpPriority: 2
    confidence: 0.9
    subject: trust-graph-authz

# The Findings. Each is sourced + confident + assured + non-filler (pkg:FindingShape),
# and each STATES its own caveat — the honesty invariant of sq-7utko.
"@graph":

  # Q — "what is the status of trust-graph unlinkability?"
  - "@id": kb:find-trust-graph-unlinkability-not-v1
    "@type": pkg:Finding
    finding: "Trust-graph unlinkability is NOT a v1 guarantee — ZK/private certification-edge admission is hard-gated on sq-qhy4"
    about: trust-graph-authz
    verdict: yes
    justification: "Unlinkable or hidden-issuer admission of certification edges is sq-pfae.8 + sq-6syab.5 territory, hard-gated on the open sq-qhy4 gate (external accredited-cryptographer sign-off pending; sparq-mpc is honest-majority semi-honest only); the clear-path trust-graph layer makes no anonymity, unlinkability, or ZK-soundness claim."
    confidence: 0.97
    assurance: claimed
    source: kb:doc-research-trust-graph-authorisation-2026-07
    evidence: kb:doc-research-trust-graph-authorisation-2026-07
    fromSection: "research/trust-graph-authorisation-2026-07.md#2 (explicit non-goals: anything ZK/private)"
    subject: trust-graph-authz

  # Q — "what kind of mechanism is the trust-graph layer?"
  - "@id": kb:find-trust-graph-clear-path
    "@type": pkg:Finding
    finding: "The trust-graph layer is a clear-path authorisation mechanism — operator-anchored keys and signed attestations, not cryptographic anonymity"
    about: trust-graph-authz
    verdict: yes
    justification: "The record's load-bearing honesty framing: the trust-graph layer bottoms out in operator-anchored keys and signed attestations, not cryptographic anonymity; signature verification rides the internally re-audited but externally unaudited sparq-zk estate (sq-qhy4 pending), and the whole layer is a research prototype behind default-OFF features."
    confidence: 0.97
    assurance: claimed
    source: kb:doc-research-trust-graph-authorisation-2026-07
    evidence: kb:doc-research-trust-graph-authorisation-2026-07
    fromSection: "research/trust-graph-authorisation-2026-07.md#honesty-framing + #3 soundness posture"
    subject: trust-graph-authz

  # Q — "which record decided the certification-closure model?"
  - "@id": kb:find-trust-graph-closure-decision-record
    "@type": pkg:Finding
    finding: "The certification-edge closure model — a depth-bounded closure into effective trust rules ahead of the UNCHANGED admission gate — was decided in research/trust-graph-authorisation-2026-07.md"
    about: trust-graph-authz
    verdict: yes
    justification: "The 2026-07 estate-audit record designs the one net-new increment: evaluating certification edges into the admission decision via a depth-bounded closure with a visited set (a cycle or depth exhaustion denies), the derived effective rules then feeding the unchanged admission gate; the model substrate remains research/solid-trust-graph-authz-design.md (sq-pfae.1, merged) and the prototype ships depth 1."
    confidence: 0.95
    assurance: claimed
    source: kb:doc-research-trust-graph-authorisation-2026-07
    evidence: kb:doc-research-trust-graph-authorisation-2026-07
    fromSection: "research/trust-graph-authorisation-2026-07.md#2 design + #0 corrected premise"
    subject: trust-graph-authz

  # Q — "what may the trust-graph program claim about MPC / cryptographic soundness?"
  - "@id": kb:find-trust-graph-mpc-semi-honest
    "@type": pkg:Finding
    finding: "sparq-mpc remains honest-majority semi-honest only; the trust-graph program asserts no settled cryptographic soundness, privacy, or unlinkability guarantee"
    about: trust-graph-authz
    verdict: yes
    justification: "Standing discipline inherited by every child bead of the record: the sparq ZK verifier is remediated and internally re-audited but has no external accredited-cryptographer sign-off (open gate sq-qhy4), and sparq-mpc is honest-majority semi-honest only, so nothing in the record asserts a settled cryptographic soundness, privacy, or unlinkability guarantee."
    confidence: 0.97
    assurance: claimed
    source: kb:doc-research-trust-graph-authorisation-2026-07
    evidence: kb:doc-research-trust-graph-authorisation-2026-07
    fromSection: "research/trust-graph-authorisation-2026-07.md#honesty-framing"
    subject: trust-graph-authz
