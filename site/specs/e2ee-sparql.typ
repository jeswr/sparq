// [OPUS-5] sq-tag1q.5 / issue #2548 — E2EE-SPARQL: End-to-End-Encrypted RDF with SPARQL
// Query, an Unofficial Proposal Draft.
//
// PROVENANCE. This draft is the spec artifact the E2EE program has been gating on. Its
// normative content is not invented here: it is the consolidation of four maintainer-reviewed
// design records, and every clause below traces to one of them —
//   * research/e2ee-queryable-options.md               — the option-space survey; defines the
//     T0-T4 leakage vocabulary, Profile CS, Profile SE, and the rejections (the "what is
//     specifiable at all" authority);
//   * research/e2ee-queryable-nextgraph-variant-2026-07.md — the PRIVACY authority: threat
//     model, the CS-vs-BR leakage comparison, and the BR-1..BR-9 clause set;
//   * research/e2ee-nextgraph-variant-gpt56-2026-07.md — the BINDING authority: the v0 wire
//     realization (capability / envelope / epoch / relay messages) reproduced in Annex A;
//   * research/e2ee-program-reconciliation-2026-07.md  — the canonical overlay that decides
//     which clause wins where the records disagree, and binds the CRDT to ONE artifact.
// Where the reconciliation superseded a clause, THIS document carries the surviving clause and
// says so in place. The CRDT is incorporated by reference from the SPARQL-CRDT draft; this
// document deliberately restates none of that algebra.
//
// GREENFIELD STATUS. sparq implements no conformance class of this document at this revision.
// The one shipped artifact is the opt-in `sparq-e2ee-ng` crate (Annex A's primitives layer:
// capability / envelope / epoch) plus the opt-in Profile-SE literal codec; the sync, relay,
// materialization, and key-distribution layers are unimplemented. Every normative statement is
// therefore rendered inside an explicitly labelled Proposal block.
//
// CLAIM BOUNDARY (load-bearing). No clause here is a cryptographic-soundness or
// confidentiality proof. Every confidentiality, integrity, authorization, revocation, and
// convergence property named below is DESIGNED/INTENDED, never proven; none of it has had
// external cryptographic review; production use is gated by bead sq-qhy4 (open). The
// impossibility statement in section 1 is the spine of the document: general server-side
// SPARQL evaluation over end-to-end-encrypted data without leakage does not exist, and no
// profile here claims otherwise.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(
  title: "E2EE-SPARQL: End-to-End-Encrypted RDF Datasets with SPARQL Query — Profiles and Disclosed Leakage",
)
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

// Every requirement is a proposal until an opt-in implementation and its conformance fixtures
// land. Keeping the label at each assertion site makes the greenfield status visible in both
// the HTML render and the PDF, exactly as the SPARQL-CRDT draft does.
#let proposal(id, body) = context {
  let label = "Proposal [" + id + "]"
  if target() == "html" {
    html.elem("aside", attrs: (class: "note proposal", id: "req-" + lower(id)))[
      #html.elem("span", attrs: (class: "note-title"))[#label]
      #body
    ]
  } else {
    block(
      width: 100%,
      inset: 8pt,
      radius: 3pt,
      stroke: (left: 3pt + rgb("#c80")),
      fill: rgb("#fdf8ee"),
    )[
      #strong[#label] — #body
    ]
  }
}

#spec-head()

#intro-section("abstract", "Abstract")[
  This document proposes conformance profiles for storing RDF datasets under end-to-end
  encryption — keys generated and held client-side, never possessed by the server — while
  retaining a useful form of SPARQL query. It starts from a negative result rather than hiding
  it: no known construction lets an untrusted server evaluate general SPARQL over
  end-to-end-encrypted data without disclosing a leakage profile. Every profile here is
  therefore a labelled point on a disclosed trade-off curve.

  Three profiles are defined. #strong[Profile CS] (client-side; the mandatory core) makes the
  server a blob store: the client synchronizes ciphertext, decrypts locally, and evaluates full
  SPARQL 1.1 against a local dataset, so the server observes resource-level access patterns and
  nothing else. #strong[Profile BR] (broker-relayed; optional) adds an always-on untrusted
  relay and a replicated dataset so several writers can collaborate and read while an author is
  offline; query evaluation is still #strong[always local] — the relay never evaluates a query
  — and the relay in exchange observes topic and device membership, commit timing, and block
  sizes. #strong[Profile SE] (structure-exposed; optional) is the only profile in which a
  server evaluates part of a query: literal #emph[values] are encrypted while graph
  #emph[structure] stays cleartext, so ordinary server-side basic-graph-pattern matching,
  joins, and property paths work — at the price of disclosing the full graph topology and the
  predicate vocabulary, which is a large and highly identifying disclosure.

  A normative leakage vocabulary (tiers T0–T4) is defined, and every profile and future
  extension #strong[MUST] declare its tier and publish a leakage statement. Deterministic and
  order-revealing encryption of literal values, and server-side decryption functions, are
  rejected rather than deferred, with the attack record cited. A non-normative annex records
  the v0 wire binding; a second non-normative annex scopes composition with the project's
  zero-knowledge and multi-party-computation work, which is research-grade and externally
  unaudited.
]

#sotd()

#intro-section("implementation-standing", "Implementation standing")[
  #strong[Greenfield proposal.] No sparq crate claims any conformance class in this document at
  this revision. What exists today is the opt-in `sparq-e2ee-ng` crate — the capability,
  envelope, key-schedule, signature, and epoch-transition #emph[primitives] of @sec-annex-a,
  plus the opt-in Profile-SE literal codec of @sec-profile-se — and the opt-in `sparq-crdt`
  crate for the replicated dataset. The synchronization protocol, the relay, key distribution,
  materialization, and every conformance fixture family are unimplemented. Requirements appear
  in amber #emph[Proposal] blocks and remain proposals until an implementation and its fixtures
  land. Text outside those blocks is informative.

  #strong[No proven cryptographic property is claimed anywhere in this document.] Every
  confidentiality, integrity, authorization, and revocation property is a design intent. The
  construction has had no external cryptographic review, and the project's audit gate for
  cryptographic claims (bead `sq-qhy4`) is open. A reader looking for assurance should read
  @sec-security as the honest statement of what is #emph[not] established.
]

= Introduction <sec-introduction>

A user stores RDF on a server they do not control — a Solid pod, a hosted triplestore, a
sync relay. They want two things at once: the operator must not be able to read the data, and
applications must still be able to query it with SPARQL. These pull against each other.
SPARQL's expressiveness is precisely the class of rich computation over structure and values
that encryption is designed to prevent.

== The impossibility statement <sec-impossibility>

This document begins where the survey it consolidates ended:

#note[
  #strong[General server-side SPARQL evaluation over end-to-end-encrypted data, without
  leakage, does not exist] — not in the literature, not in a deployed system, and not in sparq.
  Every known scheme that lets an untrusted server answer queries over ciphertext buys
  expressiveness with a #emph[disclosed leakage profile], and the leakage-abuse literature
  shows those profiles are routinely exploitable in practice #cite("IKK12") #cite("CGPR15")
  #cite("NKW15") #cite("GSBNR17") #cite("ZKP16").
]

The consequence for a specification is structural, not rhetorical. A profile cannot be
described by what it encrypts; it must be described by #emph[what an observer learns anyway].
So leakage is a first-class normative concept here (@sec-leakage), every profile declares a
tier, and the phrase "end-to-end encrypted" is never used in this document without the
qualification of the profile that earns it.

== Where the query runs — the choice that defines a profile <sec-where>

The profiles differ on exactly one architectural question: #emph[where is the query
evaluated, and over what].

- #strong[Profile CS] and #strong[Profile BR] evaluate #strong[locally], over a decrypted
  dataset the client materialized. The server or relay never sees a query, a query answer, or a
  result size. These two profiles differ in #emph[distribution], not in query placement: CS has
  a passive blob store and effectively one writer; BR adds an always-on relay and multi-writer
  convergence, and pays for it in synchronization metadata.
- #strong[Profile SE] evaluates the #strong[structural fragment server-side] over cleartext
  structure, and the value-dependent remainder client-side after decryption. It is the only
  profile in this document in which a server computes over the user's data at all.

#note[
  #strong[A correction worth making explicitly, because the two asks are routinely conflated.]
  A NextGraph-shaped deployment — an always-on broker that stores and routes encrypted blocks
  for many collaborating writers #cite("NEXTGRAPH-PROTOCOL") — is #strong[not] a system in
  which the server evaluates queries. Its broker relays opaque blocks and never evaluates a
  graph pattern; querying happens in the client after decryption and merge. That deployment
  shape is @sec-profile-br here. If what is wanted is genuine #emph[server-side] query
  evaluation, that is @sec-profile-se, and it is a different and much larger disclosure: the
  server learns the whole shape of the graph. This document specifies both, and refuses to
  describe either as the other.
]

== Goals and non-goals <sec-goals>

Goals: define profiles that are implementable with reviewed primitives today; make leakage
declarable and comparable; keep full SPARQL 1.1 available in at least one profile; let a
deployment choose collaboration or minimal disclosure knowingly; and specify an interoperable
encoding for encrypted literal values, which is the interoperability gap no RDF specification
currently closes.

Non-goals, each excluded on the record rather than by omission: hiding access patterns,
volume, or timing from a server (padding and batching reduce, never eliminate, these); any
form of forward secrecy or post-compromise security (@sec-revocation); defending a compromised
client — a compromised endpoint is game-over for its own data under every profile here;
server-side decryption of any kind (@sec-core, `CORE-6`); range or order queries over encrypted
values (@sec-rejected); and access control, which is an orthogonal mechanism a server enforces
over data it #emph[can] read.

Two adjacent, frequently confused mechanisms are out of scope by definition. #strong[Server-side
at-rest encryption], where the server holds the keys, is an operator control, not
end-to-end encryption — a server that can decrypt on demand is implementing access control.
#strong[Transport encryption] protects a hop, not a store. Neither satisfies @sec-terminology's
definition of end-to-end encryption.

= Terminology, scope, and proposal status <sec-terminology>

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHOULD], #strong[SHOULD NOT], #strong[MAY], and #strong[OPTIONAL] are to be
interpreted as described in #cite("RFC2119") and #cite("RFC8174") when, and only when, they
appear in uppercase inside a Proposal block.

#dfn("End-to-end encryption (E2EE)") — in this document — means that data-encryption keys are
generated and held by the data owner and their explicit delegates, client-side, and that no
server, relay, or intermediary ever possesses plaintext or a decryption key. A deployment in
which a server can decrypt on request is out of scope regardless of how the keys are stored.

#dfn("Leakage") is everything an adversary in the threat model learns that is not the
plaintext: access patterns, sizes, timing, membership, structure, and equality relations. A
#dfn("leakage statement") is the human-readable enumeration of it that a profile requires an
implementation to publish.

#dfn("Structural fragment") of a query is the part evaluable over cleartext structure alone —
basic graph patterns over subjects, predicates, IRI- and blank-node-valued objects, named-graph
membership, property paths, and cardinality over those. The #dfn("value-dependent remainder") is
everything requiring a literal value: `FILTER` over values, `ORDER BY`, value joins, and value
aggregation.

#dfn("Materialization") is the client-side construction of a plaintext RDF dataset from
decrypted state, over which SPARQL is then evaluated by an ordinary engine.

#proposal("E2EE-STATUS")[
  Every requirement in this document is a #strong[proposal]. An implementation #strong[MUST NOT]
  claim conformance to this document until the corresponding conformance fixtures
  (@sec-classes) exist and pass. A conformance claim #strong[MUST] name the profile
  (@sec-profile-cs, @sec-profile-br, or @sec-profile-se), the declared leakage tier
  (@sec-leakage), and the concrete algorithm suite in use (`CORE-1`).
]

#proposal("E2EE-SCOPE")[
  This document specifies exactly three profiles: #strong[CS], #strong[BR], and #strong[SE].
  Profile CS is #strong[mandatory to implement] for a claim of E2EE conformance; BR and SE are
  #strong[OPTIONAL] additions. An implementation #strong[MUST NOT] describe a profile of its
  own invention as a profile of this document, and #strong[MUST NOT] present a profile's
  properties as this document's properties without naming the profile.
]

= Leakage as a normative vocabulary <sec-leakage>

Comparing encrypted-query designs is impossible without a shared scale for what escapes. This
document adopts the survey's five tiers, worst to best, and requires every profile to declare
one. The tiers describe #emph[what a server-side observer learns], not how the scheme is built.

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Tier][What the server-side observer learns][Typical source],
  [`T0`], [Plaintext-equivalent under a realistic attack: value frequencies and/or order
    suffice to recover values.], [Deterministic or order-revealing value encryption
    #cite("NKW15") #cite("GSBNR17")],
  [`T1`], [The full structure: every subject, predicate, IRI-valued object, graph membership,
    degree, and co-occurrence.], [Structure-exposed value encryption (@sec-profile-se)],
  [`T2`], [Search, access, and volume patterns over an encrypted index.], [Searchable or
    structured encryption #cite("CK10") #cite("CGPR15")],
  [`T3`], [Resource-level access pattern only: which ciphertexts were fetched together, when,
    and how large.], [Encrypted blobs on a passive store (@sec-profile-cs)],
  [`T4`], [Volume and timing only.], [Full-replica synchronization with padding and batching],
)

#proposal("LEAK-1")[
  Every profile, and every extension to a profile, #strong[MUST] declare exactly one leakage
  tier from `T0`–`T4`, and #strong[MUST] declare the strongest (numerically lowest) tier that
  applies to any part of its behaviour. An extension that raises leakage #strong[MUST] restate
  the tier of the profile as extended.
]

#proposal("LEAK-2")[
  A conforming implementation #strong[MUST] publish, in its user-facing conformance
  documentation, a #strong[leakage statement] enumerating what a server, relay, or network
  observer learns under its deployed configuration. The statement #strong[MUST] be written for
  the data owner, not for a cryptographer, and #strong[MUST] state the tier of `LEAK-1`.
]

#proposal("LEAK-3")[
  A leakage statement #strong[MUST NOT] omit access patterns, volume, or timing on the grounds
  that they are "metadata". An implementation #strong[MUST NOT] claim that padding, batching,
  or full-replica synchronization eliminates them; such mitigations #strong[MAY] be described
  only as reducing them.
]

#proposal("LEAK-4")[
  An implementation #strong[MUST NOT] use the unqualified phrase "end-to-end encrypted" for a
  profile whose declared tier is `T2` or worse without, in the same user-facing context, naming
  what is exposed. For tier `T1` the exposure named #strong[MUST] include the words "graph
  structure" or an equivalent plain-language phrase.
]

#proposal("LEAK-5")[
  Where a profile's leakage depends on a deployment choice — clear versus opaque routing
  headers, partial versus full fetch, equality tags on or off — the implementation
  #strong[MUST] declare the choice it made, and #strong[MUST] amend its leakage statement to
  match. A default configuration #strong[MUST] be the lower-leakage one where the profile
  offers a choice.
]

= Common requirements <sec-core>

These apply to all three profiles.

#proposal("CORE-1")[
  #strong[Algorithm agility with one bound suite.] An implementation #strong[MUST] support
  algorithm agility: a suite identifier #strong[MUST] be bound into every capability,
  envelope, and session, and #strong[MUST] be covered by the authentication of the structure
  that carries it. A deployment #strong[MUST] bind exactly one reviewed concrete suite and
  #strong[MUST NOT] silently substitute an algorithm. A suite #strong[MUST] provide an
  authenticated-encryption-with-associated-data primitive, a key-derivation function, a
  signature scheme, and a recipient-wrapping mechanism.
]

#proposal("CORE-2")[
  #strong[Keys stay client-side.] Data-encryption keys, key-wrapping private keys, signing
  private keys, and capability secret fields #strong[MUST NOT] be transmitted to, derivable by,
  or stored by any server, relay, or intermediary, in any encoding, including logs, request
  paths, and routing metadata. A secret #strong[MUST NOT] appear in RDF that leaves the client
  unencrypted.
]

#proposal("CORE-3")[
  #strong[Recipient wrapping for sharing.] Sharing a key with another party #strong[MUST] use
  recipient wrapping (encryption to that party's public key) or an out-of-band protected
  channel. Where an identity system is in use, the recipient key #strong[SHOULD] be discoverable
  from that identity; for Solid deployments the recipient key #strong[SHOULD] be associated with
  the recipient's WebID. The location of that key within an identity document is out of scope
  for this revision and #strong[MUST] be declared by the deployment.

  An implementation #strong[MUST] disclose whether its wrapping construction is forward-secret.
  A wrapping construction using the recipient's long-term key #strong[MUST NOT] be described as
  forward-secret: compromise of that long-term key exposes previously recorded wraps.
]

#proposal("CORE-4")[
  #strong[Padding is mandatory, and is not a hiding claim.] Plaintext #strong[MUST] be padded
  to size classes from the suite registry before encryption, at whatever granularity the
  profile encrypts (resource, block, or literal value). An implementation #strong[MUST NOT]
  claim that padding hides sizes; it bounds the resolution of a size observation.
]

#proposal("CORE-5")[
  #strong[Fail closed.] Any failure of decryption, authentication, signature verification,
  suite check, parser limit, or context binding #strong[MUST] be treated as a rejection of the
  affected unit, and #strong[MUST NOT] be repaired by falling back to a weaker check, by
  re-encoding, or by accepting the unit unauthenticated. A rejected unit #strong[MUST NOT]
  invalidate already-accepted state.
]

#proposal("CORE-6")[
  #strong[No server-side decryption, ever.] An implementation #strong[MUST NOT] provide a
  server-side function, extension function, stored procedure, or index build step that
  decrypts a value, and #strong[MUST NOT] transmit a key to a server for such a purpose. A
  deployment that does so is outside this specification entirely: it has replaced end-to-end
  encryption with access control, and #strong[MUST NOT] claim any profile of this document.
]

#proposal("CORE-7")[
  #strong[Answers are labelled with the state they were computed over.] A query answer
  #strong[MUST] be labelled with the identity and version of the local state it was evaluated
  against — for Profile BR, the branch, epoch, and accepted frontier (`BR-6`). "Current"
  #strong[MUST] mean current at that labelled state, never globally latest. An implementation
  #strong[MUST NOT] present an answer computed over stale or partial local state as complete.
]

#proposal("CORE-8")[
  #strong[No proven-property claim.] A conformance claim under this document #strong[MUST NOT]
  assert a proven cryptographic-confidentiality, integrity, forward-secrecy, or
  post-compromise-security property. An implementation #strong[MUST] state whether its
  construction has received external cryptographic review, and #strong[MUST NOT] present an
  internal review as an external one.
]

== Revocation and the two properties nobody here provides <sec-revocation>

Revocation in an at-rest encrypted store is weaker than users expect, and the specification's
job is to prevent a comfortable description of it. Removing a member cannot erase plaintext or
keys that member already holds. Re-keying limits #emph[future] exposure only.

#proposal("CORE-9")[
  An implementation #strong[MUST] document its revocation semantics as exactly one of:
  #strong[`forward-only`] — a fresh key set is minted and future writes are encrypted under it,
  while a removed member retains the ability to read history it already holds or can re-fetch
  and decrypt with cached keys; or #strong[`history-rekeyed`] — sensitive history is
  re-encrypted under the new key set so that a cached-key holder loses access to #emph[re-fetched]
  history, noting that data already downloaded cannot be revoked at all. The declared value
  #strong[MUST] be carried in the authenticated structure that effects the transition where the
  profile has one.

  An implementation #strong[MUST NOT] describe either option as "forward secrecy" or as a
  post-compromise-security guarantee, and #strong[MUST NOT] imply that revocation retroactively
  protects data.
]

#note[
  This specification provides #strong[no forward secrecy] and #strong[no post-compromise
  security] in any profile, by design rather than by oversight. A store-at-rest model has no
  session-key ratchet, and a design goal of the collaborative profile is that a party joining
  later can read history — which is the direct opposite of forward secrecy. Any implementation
  text that suggests otherwise is a defect against `CORE-8` and `CORE-9`.
]

= Profile CS — client-side evaluation (mandatory core) <sec-profile-cs>

Profile CS is the baseline every conforming implementation provides, and the only profile with
both full query expressiveness and a confidentiality story that fits in one sentence: the
server is a blob store that holds ciphertext and answers fetches.

#proposal("CS-1")[
  #strong[Overview, informative.] A conforming Profile-CS deployment stores each RDF resource
  as an authenticated-encrypted payload on a server that implements ordinary resource
  semantics over opaque bytes. A client fetches the payloads its capabilities cover, decrypts
  them locally, ingests them into a local dataset, and evaluates #strong[full SPARQL 1.1]
  against that dataset. The server performs no RDF parsing, no pattern matching, and no query
  evaluation.
]

#proposal("CS-2")[
  #strong[Declared tier.] Profile CS declares tier #strong[`T3`] — resource-level access
  pattern, sizes, and timing. A deployment that synchronizes a full replica and pads
  #strong[MAY] declare #strong[`T4`] instead, and #strong[MUST NOT] declare `T4` while it
  performs selective fetches, because fetch selectivity is itself the access-pattern signal.
]

#proposal("CS-3")[
  #strong[Envelope.] A resource payload #strong[MUST] be sealed with the suite's
  authenticated-encryption primitive under a key the server does not hold, with associated data
  binding at least the suite identifier, a format version, and the resource's identity within
  the deployment, so that a payload cannot be relocated to another resource undetected.
  Plaintext #strong[MUST] be padded per `CORE-4`.
]

#proposal("CS-4")[
  #strong[Key hierarchy.] An implementation #strong[MUST] separate data-encryption keys from
  the key-wrapping keys used to share them, so that sharing and rotation do not require
  re-encrypting content. Sharing #strong[MUST] follow `CORE-3`; revocation #strong[MUST] follow
  `CORE-9`.
]

#proposal("CS-5")[
  #strong[Server conformance is ordinary resource semantics.] A Profile-CS server
  #strong[MUST NOT] be required to implement anything beyond authenticated read and write of
  opaque payloads with per-resource authorization. A server #strong[MUST NOT] be required to
  understand this specification. An implementation #strong[MUST NOT] make server-side SPARQL
  over plaintext a prerequisite of the profile.
]

#proposal("CS-6")[
  #strong[Query placement.] Evaluation #strong[MUST] be local. A client #strong[MUST NOT]
  delegate any part of query evaluation, including index construction, to a party that does not
  already hold the decryption keys for the data involved.
]

#note[
  Profile CS is where sparq's existing engine does the work: the full query surface already
  runs client-side, including in the browser, and ingests from an in-memory or streaming source,
  so a decrypt-then-ingest pipeline needs no engine change. The net-new work is the envelope and
  key management — not the query engine. The cost of the profile is equally plain: the client
  downloads all ciphertext relevant to what it is authorized to see, so work scales with the
  authorized corpus rather than with the answer.
]

= Profile BR — broker-relayed collaboration (optional) <sec-profile-br>

Profile CS has two structural weaknesses: collaboration is manual, and a collaborator cannot
receive an update while the author is offline. Profile BR addresses both with an always-on
untrusted relay and a replicated dataset. It is the NextGraph-shaped profile
#cite("NEXTGRAPH-PROTOCOL") #cite("NEXTGRAPH-CRDT"), and its clauses `BR-1`–`BR-9` are carried
from the privacy-authority record, amended where the program reconciliation superseded them.

Query evaluation in Profile BR is #strong[still local]. The relay stores and routes encrypted
blocks; it does not evaluate, and is not permitted to.

#proposal("BR-1")[
  #strong[Overview, informative.] A conforming Profile-BR deployment stores an RDF dataset as a
  causal graph of end-to-end-encrypted commits over a per-branch replicated dataset, relayed by
  an untrusted relay that stores and routes encrypted blocks without decrypting them. Clients
  holding the relevant read capability synchronize the encrypted commits, decrypt and merge them
  locally into a materialized RDF dataset, and evaluate SPARQL 1.1 over that local dataset.
  Query evaluation is #strong[ALWAYS] local; the relay #strong[MUST NOT] evaluate queries.
]

#proposal("BR-2")[
  #strong[Encryption granularity and header visibility.] Content #strong[MUST] be encrypted per
  object as a set of authenticated-encrypted blocks, using a suite from the registry, padded
  per `CORE-4`. Commit #strong[headers] #strong[MUST] be encrypted in a default-conforming
  deployment, so the plaintext commit graph is not exposed to the relay; a deployment that
  enables clear routing headers #strong[MUST] declare that choice under `LEAK-5` and
  #strong[MUST] amend its `BR-8` statement to disclose the exposed commit graph.
]

#proposal("BR-3")[
  #strong[Object keys and identifiers are randomized in this revision.] A read capability
  #strong[MUST] convey the ability to derive the decryption keys for the branch it covers, and
  those keys #strong[MUST NOT] be transmitted to or stored by the relay. Per-object keys
  #strong[MUST] be derived from the branch read secret by a domain-separated key-derivation
  function, bound to the repository, branch, epoch, and object identity. Object and block
  identifiers #strong[MUST] be random and #strong[MUST NOT] be a function of plaintext or
  ciphertext.

  Convergent keying and content-addressed identifiers — which would enable deduplication — are
  #strong[NOT] part of this revision. They are a future opt-in extension that #strong[MUST]
  declare, under `LEAK-5`, the equal-plaintext-within-a-store linkability they introduce.
]

#proposal("BR-4")[
  #strong[Three separated authorities.] Authority #strong[MUST] be separated into
  #strong[read], #strong[publish], and #strong[admin]. Every commit #strong[MUST] be signed by
  a publisher key, and every membership or key transition #strong[MUST] be signed by an admin
  key. The publish and admin private keys #strong[MUST NOT] be the same key and
  #strong[MUST NOT] be derived from the read secret. A relay #strong[MUST] verify that a
  publisher is admitted for a topic before relaying, without access to plaintext, and
  #strong[MUST NOT] be able to forge or alter commit content. Delegation #strong[MUST] only
  narrow: a delegated capability #strong[MUST NOT] widen the authority set, validity window, or
  epoch bound of its parent.
]

#proposal("BR-5")[
  #strong[One replicated dataset, defined elsewhere.] RDF content #strong[MUST] be represented
  as the convergent replicated dataset defined by the SPARQL-CRDT draft #cite("SPARQL-CRDT"),
  incorporated here by reference. Profile BR #strong[MUST NOT] define, name, or version a
  replicated-dataset algebra of its own, and #strong[MUST NOT] re-encode, reorder, or
  re-canonicalize a delta: the delta's canonical bytes are opaque payload to this profile and
  are what get padded, sealed, and bound in associated data.

  One branch is one replication domain, and a branch replicates a whole dataset — the default
  graph and all named graphs — because graph membership is part of quad identity in the
  referenced draft. The encryption epoch of this profile and the membership epoch of the
  replicated dataset #strong[MUST] be the same value for a branch. Replica identifiers and
  causal metadata #strong[MUST NOT] be derived from any key material, so that a key rotation
  does not rewrite causal history.
]

#proposal("BR-6")[
  #strong[Sync, materialize, query.] A client #strong[MUST] subscribe only to the topics its
  read capabilities cover; #strong[MUST] verify envelopes, signatures, epochs, and causal
  parents locally before accepting a commit; #strong[MUST] apply merge in causal order and
  materialize the result into a local dataset over which it evaluates SPARQL 1.1 locally; and
  #strong[MUST] label every answer with the branch, epoch, and accepted frontier per `CORE-7`.
  A receiving replica #strong[MUST NOT] re-evaluate the graph pattern of an update; it applies
  concrete effects. Incremental application of new commits #strong[SHOULD] be supported.
]

#proposal("BR-7")[
  #strong[Revocation is an epoch transition.] An epoch transition #strong[MUST] mint a fresh
  topic, read secret, and publisher key set, distribute new capabilities only to remaining
  members, and encrypt all subsequent commits under the new epoch. The transition
  #strong[MUST] be authenticated by an admin key and #strong[MUST] bind the old and new epoch,
  the old and new topic, the new verification-key set, and the declared history policy of
  `CORE-9` (`forward-only` or `history-rekeyed`). Relay-side refusal of a removed publisher
  #strong[MAY] be implemented as an online enforcement layer, and #strong[MUST NOT] be
  described as retrospective secrecy.
]

#proposal("BR-8")[
  #strong[Mandatory leakage statement.] Profile BR declares tier #strong[`T3`] for content and
  query, with an explicitly larger metadata surface than Profile CS. A Profile-BR
  implementation #strong[MUST] surface, in its conformance documentation, a leakage statement
  to the effect that a serving relay observes: topic and overlay membership; the set of a
  user's devices; per-commit synchronization timing, ordering, and block sizes; which blocks a
  client fetches; and client network metadata — while it does #strong[not] observe plaintext
  content, query text, query answers, or, in a default-conforming opaque-header deployment, the
  plaintext commit graph. The statement #strong[MUST] say that this metadata surface is
  strictly larger than Profile CS's.

  The statement #strong[MUST] additionally disclose that causal metadata carries a persistent
  per-replica identifier which is visible to #strong[every] read-capability holder of the
  branch — including a party who joins later and reads history, since this profile provides no
  forward secrecy — so authorship and activity ordering over the branch's history are
  reconstructable by an authorized reader. This is a consequence of two accepted design choices
  (no forward secrecy, and causal metadata not derived from keys), not a defect, and it
  #strong[MUST NOT] be omitted on the grounds that the observer is authorized.
]

#proposal("BR-9")[
  #strong[No soundness claim, and one honest limit on the relay.] No conformance claim under
  this profile asserts a proven cryptographic-confidentiality, forward-secrecy, or
  post-compromise-security property (`CORE-8`). A malicious relay #strong[can] withhold,
  delay, reorder, replay, and selectively deliver blocks, and #strong[can] equivocate — present
  different clients with different views of history — while being unable to forge a signed
  commit or decrypt content. Resistance to equivocation is a #strong[design requirement, not an
  established property]: an implementation that wants it #strong[MUST] specify how clients
  cross-check heads out of band, and #strong[MUST NOT] claim it otherwise. Availability is not
  protected against a relay that refuses service.
]

== What Profile BR buys, and what it costs <sec-br-tradeoff>

#table(
  columns: 4,
  align: (left, left, left, left),
  table.header[Property][Profile CS][Profile BR][Profile SE],
  [Plaintext triples], [Hidden], [Hidden], [Literal values hidden; #strong[structure
    cleartext]],
  [Query text and answers], [Hidden — local evaluation], [Hidden — local evaluation],
    [#strong[Structural query exposed] to the server; values post-filtered locally],
  [Where SPARQL runs], [Client, full SPARQL 1.1], [Client, full SPARQL 1.1],
    [Server (structure) + client (values)],
  [Graph topology], [Hidden], [Hidden with opaque headers; a coarse activity and rate signal
    remains], [#strong[Fully disclosed]],
  [Multi-writer collaboration], [Weak — manual merge], [#strong[Strong] — convergent merge],
    [Depends on the store's own concurrency],
  [Updates while author offline], [Poor — passive server], [#strong[Good] — always-on relay],
    [Good — ordinary server],
  [Membership and device sets], [Weak co-access signal], [#strong[Disclosed] to the serving
    relay], [As the server's own access logs],
  [Declared tier], [`T3` (`T4` with full replica + padding)], [`T3` for content, larger
    metadata surface], [#strong[`T1`]],
)

The honest one-sentence verdict: Profile BR leaks #emph[strictly more metadata] than Profile
CS — a bounded, characterized channel of membership, device sets, and live commit
timing and sizes to a serving relay — and in exchange delivers collaboration and availability
that CS structurally cannot. It leaks no more plaintext, query, or answer than CS does.

= Profile SE — structure-exposed, server-side structural query (optional) <sec-profile-se>

Profile SE is the profile to reach for when the requirement is genuinely #emph[server-side
evaluation]: an ordinary SPARQL server, holding no keys, answers the structural fragment of a
query, and the client decrypts the values in the answer. It is the only profile here in which a
server computes over the user's data, and it carries the largest disclosure in this document.

#note[
  #strong[Read this before choosing Profile SE.] Profile SE protects the #emph[values], not
  the #emph[shape of the user's life]. The server learns every subject, every predicate, every
  IRI-valued relationship, named-graph membership, degrees, co-occurrence, and update dynamics.
  Predicates announce the #emph[kind] of every hidden value: a `foaf:name` or a
  `dbo:diagnosis` edge tells the server what the ciphertext beside it is. Structure alone is
  highly identifying — de-anonymization from graph topology alone is classical
  #cite("NS09") — and RDF hands the adversary labelled, ontology-typed edges. Profile SE is
  appropriate when the values are the secret and the shape is not; it is inappropriate as a
  general substitute for Profile CS.
]

#proposal("SE-1")[
  #strong[Overview, informative.] A conforming Profile-SE deployment stores RDF in which graph
  structure — subjects, predicates, IRI- and blank-node-valued objects, and named-graph
  membership — is cleartext, and literal #emph[values] are individually
  authenticated-encrypted and carried as typed literals. An ordinary SPARQL server, holding no
  keys, evaluates the structural fragment. Encrypted values are opaque to it. The client
  decrypts the values in the answer and evaluates the value-dependent remainder locally.
]

#proposal("SE-2")[
  #strong[Declared tier, and the mandatory leakage statement.] Profile SE declares tier
  #strong[`T1`]. A Profile-SE implementation #strong[MUST] surface, in its conformance
  documentation and in any user-facing description, a leakage statement to the effect that the
  #strong[full graph structure and predicate vocabulary are visible to the server], that
  predicates disclose the kind of each encrypted value, that ciphertext lengths disclose an
  approximate value size within a padding class, and — if equality tags are enabled (`SE-6`) —
  that per-predicate value-equality patterns and frequencies are also visible. Describing
  Profile SE as "end-to-end encrypted" without naming the structural exposure violates
  `LEAK-4`.
]

#proposal("SE-3")[
  #strong[Encrypted-literal encoding.] An encrypted value #strong[MUST] be encoded as a literal
  with datatype IRI `urn:jeswr:w3id:e2ee-sparql:draft:2026-07#enc` whose lexical form is a
  deterministic, canonical encoding of an envelope carrying at least: a format version, the
  suite identifier, the nonce, and the ciphertext with its authentication tag. The lexical form
  #strong[MUST] be canonical — a parser #strong[MUST] reject a non-canonical encoding, a wrong
  length, and trailing bytes rather than normalizing it (`CORE-5`).

  The plaintext sealed inside the envelope #strong[MUST] carry both the original lexical form
  and the original datatype IRI, so that decryption restores the exact original literal,
  including its datatype. The original datatype #strong[MUST NOT] be left in cleartext outside
  the envelope: the datatype of a value is often as identifying as the value.

  The datatype IRI above is a non-dereferenceable placeholder for this draft. A published
  revision #strong[MUST] replace it with a dereferenceable IRI in a namespace the editors
  control, and #strong[MUST] treat the change as a wire-format break.
]

#proposal("SE-4")[
  #strong[Position binding.] The associated data of a value envelope #strong[MUST] bind the
  format version, the suite identifier, a domain separator distinguishing a value envelope from
  every other envelope kind in this specification, the predicate IRI, and the named graph. It
  #strong[SHOULD] additionally bind the subject IRI.

  An implementation that does not bind the subject #strong[MUST] disclose, in its leakage
  statement, that a server can relocate a ciphertext from one subject to another with the same
  predicate without detection — a plausible-looking wrong answer rather than a decryption
  failure. An implementation #strong[MUST NOT] describe unbound-subject mode as
  integrity-protecting the position of a value.
]

#proposal("SE-5")[
  #strong[Randomized values and per-predicate keys.] Value encryption #strong[MUST] be
  randomized: two occurrences of the same value #strong[MUST NOT] produce the same ciphertext
  by default. Value keys #strong[MUST] be derived by a domain-separated key-derivation function
  from a key family scoped no more broadly than per predicate, so that disclosing one
  predicate's key family does not disclose another's. Value plaintext #strong[MUST] be padded
  per `CORE-4`.
]

#proposal("SE-6")[
  #strong[Equality tags are a separate, separately-disclosed opt-in.] An implementation
  #strong[MAY] additionally emit a per-value #strong[equality tag] — a deterministic,
  key-derived tag over the value and its datatype — carried as a literal with datatype IRI
  `urn:jeswr:w3id:e2ee-sparql:draft:2026-07#eqtag`, which restores server-side value-equality
  joins and equality filters against a client-computed tag.

  Equality tags #strong[MUST NOT] be enabled by default (`LEAK-5`), #strong[MUST] be a distinct
  configuration decision from using Profile SE, and #strong[MUST] be declared in the leakage
  statement of `SE-2`, because equal values produce equal tags and therefore disclose
  per-predicate value frequency — the first rung of the ladder @sec-rejected rejects. Tags
  #strong[MUST] be derived under the same per-predicate scoping as `SE-5` so that a tag is not
  comparable across predicates. An implementation #strong[MUST NOT] emit tags for a predicate
  whose value distribution is low-entropy or public-prior — names, dates, places, coded
  diagnoses — without disclosing that frequency analysis against public auxiliary data is a
  known, practical attack on exactly that shape of data #cite("NKW15").
]

#proposal("SE-7")[
  #strong[Server conformance, and what the server must not do.] A Profile-SE server is an
  #strong[ordinary] SPARQL server: encrypted values are typed literals, and structural
  evaluation over them requires no new server behaviour. A conforming server #strong[MUST NOT]
  be required to understand the encrypted-literal datatype. A server #strong[MUST NOT] be given
  a key or a decryption function for these literals (`CORE-6`), #strong[MUST NOT] attempt to
  order, compare, or arithmetically evaluate an encrypted literal, and #strong[MUST NOT] treat
  an encrypted literal's lexical form as a value for `ORDER BY` or a range filter. A client
  #strong[MUST NOT] rely on server-side ordering of encrypted values.
]

#proposal("SE-8")[
  #strong[Client obligations on the answer.] A client #strong[MUST] treat a returned encrypted
  literal as opaque until it has verified its authentication tag against the associated data of
  `SE-4`, #strong[MUST] evaluate the value-dependent remainder of the query locally after
  decryption, and #strong[MUST NOT] present a server-computed answer as complete if the query
  had a value-dependent part the server could not evaluate. An implementation #strong[SHOULD]
  make the split explicit to the application, so that a partially-server-evaluated answer
  cannot be mistaken for a fully evaluated one.
]

#proposal("SE-9")[
  #strong[Composition with the other profiles.] Profile SE #strong[MAY] be composed with
  Profile CS: a client may hold SE-encoded data and evaluate everything locally, which strictly
  reduces disclosure and is the reasonable migration path.

  Composition of Profile SE with Profile BR — structure-exposed commits relayed so that a relay
  can route on structure — is #strong[NOT specified by this revision] and is
  #strong[discouraged]: it raises Profile BR's disclosure to `T1` while keeping BR's metadata
  surface, i.e. it collects both profiles' costs. An implementation #strong[MUST NOT] claim
  both profiles for one dataset without declaring the composed tier `T1` and restating both
  leakage statements.
]

#note[
  #strong[Implementation standing for Profile SE.] The value codec of `SE-3`–`SE-6` — the
  canonical envelope, the domain-separated per-predicate key derivation, the padding, the
  position binding, and the opt-in equality tag — is implemented in sparq as an opt-in feature
  of the `sparq-e2ee-ng` crate. The rest of the profile (a key-distribution scheme for
  predicate key families, an authoring pipeline that encrypts on write, and the client-side
  answer-decryption and residual-filter step) is not implemented, and no conformance claim is
  made. As everywhere in this document, the construction is research-grade and externally
  unaudited (`sq-qhy4`).
]

= Conformance classes <sec-classes>

#proposal("CLASS-1")[
  #strong[E2EE client.] Implements Profile CS in full (`CS-1`–`CS-6`), the common requirements
  of @sec-core, and the leakage obligations of @sec-leakage. This class is
  #strong[mandatory] for any claim of conformance to this document. It #strong[MAY] additionally
  implement Profile BR, Profile SE, or both, declaring each.
]

#proposal("CLASS-2")[
  #strong[Blob-store server.] Implements authenticated read and write of opaque payloads with
  per-resource authorization (`CS-5`). It #strong[MUST NOT] be required to parse RDF, evaluate
  SPARQL, or hold a key. An existing Solid pod or HTTP resource server satisfies this class
  without change; that is the intent.
]

#proposal("CLASS-3")[
  #strong[Relay.] Implements Profile BR's transport: block storage, topic subscription,
  publisher admission (`BR-4`), and fan-out, without decrypting. A relay #strong[MUST NOT]
  implement query evaluation, #strong[MUST NOT] link a query engine or the replicated-dataset
  algebra, and #strong[MUST] publish the disclosure ledger of `BR-8` describing what it
  observes.
]

#proposal("CLASS-4")[
  #strong[Structure-exposed server.] Implements ordinary SPARQL 1.1 query over a dataset in
  which some literals carry the encrypted-literal datatype (`SE-7`). No new behaviour is
  required; the class exists so that a deployment can state which server it is relying on and
  so that `SE-7`'s prohibitions are attributable.
]

#proposal("CLASS-5")[
  #strong[Profile declaration is part of conformance.] An implementation of any class
  #strong[MUST] publish: the classes it implements, the profiles it implements, the declared
  leakage tier of each (`LEAK-1`), its leakage statement (`LEAK-2`), the concrete suite bound
  (`CORE-1`), its revocation semantics (`CORE-9`), and whether the construction has had
  external cryptographic review (`CORE-8`). A conformance claim missing any of these is not a
  conformance claim under this document.
]

== Conformance fixture families <sec-tests>

#proposal("TEST-1")[
  Before an implementation claims a class, fixtures #strong[MUST] exist for at least: envelope
  round-trip; rejection under every wrong associated-data field, individually; rejection of a
  non-canonical encoding, an over-long input, and trailing bytes; cross-domain rejection (an
  envelope of one kind #strong[MUST NOT] open as another kind); padding — inputs of different
  lengths within one class producing equal ciphertext lengths; delegation narrowing-only;
  epoch-transition monotonicity and signature verification; and, for Profile SE, restoration of
  the exact original lexical form and datatype, plus tag equality behaviour under `SE-6` when
  tags are enabled. Each fixture #strong[MUST] fail if the property it pins is broken;
  a fixture that passes against a deliberately wrong implementation does not count.
]

= Rejected and deferred options <sec-rejected>

A specification that only says what it includes invites re-litigation. These are excluded on
the record.

#proposal("REJ-1")[
  #strong[Deterministic and order-revealing value encryption are REJECTED], not deferred. An
  implementation #strong[MUST NOT] encrypt literal values with a scheme that preserves order,
  and #strong[MUST NOT] offer server-side range filters or `ORDER BY` over encrypted values.
  Frequency analysis against public auxiliary data recovered protected columns wholesale in the
  published attack record #cite("NKW15"), leakage-abuse extends to even idealized
  order-revealing encryption #cite("GSBNR17"), reconstruction from range queries succeeds from
  access patterns alone #cite("KKNO16"), and deployed systems built on these primitives were
  broken along the same seams #cite("GMNRS16") #cite("GRS17") #cite("PRZB11"). Personal RDF
  literals — names, dates, places, diagnoses — are exactly the low-entropy, public-prior data
  those attacks consume: order leakage on a date predicate is a timeline; equality leakage on a
  location predicate is a movement profile. The only surviving deterministic construction is
  `SE-6`'s narrowly-scoped, separately-disclosed equality tag.
]

#proposal("REJ-2")[
  #strong[Fully homomorphic encryption is REJECTED] as a profile of this revision, on cost and
  on shape: general query evaluation under it is impractical today #cite("VJH21"), and it does
  not solve the stated problem — an oblivious server either returns everything or evaluates the
  whole query circuit, so selectivity, the reason for server-side evaluation, is what it
  removes. It is recorded here so that a future revision starts from this reasoning rather than
  from enthusiasm.
]

#proposal("REJ-3")[
  #strong[Searchable and structured encryption are DEFERRED with criteria], not rejected: the
  cryptography is real, but the shape is wrong for SPARQL today — lookup- and
  equality-class queries, bespoke per-query-family indexes, well-studied search- and
  access-pattern leakage #cite("IKK12") #cite("CGPR15") #cite("ZKP16"), and no interoperable
  standard to profile against #cite("CK10") #cite("FVYSHGKMC17"). A future revision
  #strong[MAY] add a `T2` profile when an encrypted-graph scheme with a SPARQL-shaped query
  interface and a stated leakage profile exists; such a profile #strong[MUST] declare `T2` or
  worse and #strong[MUST] carry the leakage-abuse citations.
]

= Security and privacy considerations <sec-security>

== Threat model <sec-threat>

Two adversary strengths are in scope for a server, relay, or pod host. An
#strong[honest-but-curious] operator follows the protocol and analyzes everything it
legitimately sees; this is the realistic operator threat and it is what the leakage statements
of @sec-leakage describe. A #strong[malicious] operator may additionally drop, delay, reorder,
replay, or selectively deliver ciphertext, and may lie about what it stores; it cannot forge
signed content and cannot decrypt, but it can attack availability and freshness and can
equivocate (`BR-9`). A #strong[network observer] sees ciphertext, timing, and volume.

What each party holds: the data owner holds all keys for its data; a read delegate holds the
keys for what was shared with it; the server or relay holds ciphertext plus the metadata its
role requires; a removed member holds whatever it cached before removal, which `CORE-9` makes
explicit. A #strong[compromised client] is out of scope: no profile here defends the endpoint
that holds the keys.

The assumptions the whole design rests on, none of them established here: the
authenticated-encryption primitive and the signature scheme behave as specified; keys are
generated and kept client-side and never reach the server; the replicated-dataset merge is
deterministic and convergent; and the client device and its key storage are not compromised.

== What is not established <sec-not-established>

Stated plainly, because this is the section a reader should trust most:

- #strong[No external cryptographic review has been performed] on any construction in this
  document or in the sparq crates that partially implement it. The project's audit gate
  (`sq-qhy4`) is open, and the concrete suite identifiers of Annex A are placeholders pending
  it.
- #strong[No forward secrecy and no post-compromise security] are provided, in any profile, by
  design (@sec-revocation).
- #strong[Access patterns, volume, and timing are not hidden] in any profile (`LEAK-3`).
- #strong[Profile SE does not hide structure] — it discloses it (`SE-2`).
- #strong[Equivocation resistance in Profile BR is a design requirement, not an established
  property] (`BR-9`).
- #strong[Metadata that is hidden from the server is not hidden from authorized readers]: the
  per-replica identifier of `BR-8` is visible to every read-capability holder, including future
  joiners reading history.

== Considerations a deployment must decide <sec-deployment>

Where the recipient key lives in an identity document (`CORE-3`); whether the relay is a
dedicated service or an existing pod acting as one (@sec-annex-a); whether a revocation default
is declared at all or every transition declares its own (`CORE-9`); whether Profile SE binds
the subject (`SE-4`); and whether equality tags are enabled for any predicate (`SE-6`). Each is
a leakage-relevant choice and each must appear in the leakage statement.

= Annex A — the v0 wire binding (informative) <sec-annex-a>

This annex is #strong[informative]. It records the concrete v0 realization that the sparq
project implemented for the Profile-BR primitives, so that implementers have a worked reference
and so that the normative clauses above can be read against something concrete. It is not a
second specification, and where it differs from @sec-profile-br the normative clauses win.

#strong[Encoding.] Protocol structures use a deterministic binary object encoding (RFC 8949
core deterministic encoding: shortest-form integers, definite lengths, ascending map keys), with
explicit parser limits on string, array, and map size and on nesting depth. Unknown mandatory
fields are rejected; extension keys are negative integers and are ignored unless negotiated.
Non-canonical encodings and trailing bytes are rejected rather than normalized.

#strong[Suite.] Algorithm agility is mandatory (`CORE-1`) and the v0 suite names are
#emph[placeholders] pending review: an authenticated-encryption primitive, a key-derivation
function, a signature scheme, and a recipient-wrapping mechanism, each named in the suite
identifier that every capability, envelope, and session binds.

#strong[Identifiers.] A repository identifier, a branch identifier, an epoch counter, an
epoch-specific topic identifier for routing, random object and block identifiers, a commit
identifier that is a hash over the canonical encrypted commit envelope, and a frontier — the
set of causally maximal commit identifiers. Identifiers are not RDF IRIs unless a deployment
maps them locally, and none is derived from plaintext.

#strong[Capabilities.] A read capability carries the repository, branch, epoch, and topic
identifiers, the branch read secret, relay locators, validity bounds, and the suite identifier.
A write capability adds a publishing private key; an admin capability carries a distinct admin
private key and never reuses the read secret or the publishing key. Capabilities are bearer
secrets: they are recipient-wrapped or moved over a separately protected channel, and never
appear in RDF, relay requests, logs, or URLs. A public grant — what a relay may see — carries
no secret field, and a parser rejects a grant that contains one. Delegation narrows only
(`BR-4`).

#strong[Envelopes.] A block envelope carries a version, random block and object identifiers,
the chunk index and count, the suite identifier, the nonce, the ciphertext with tag, and the
padding class. The per-object key is derived from the branch read secret bound to repository,
branch, epoch, and object; a per-block key is derived from it with domain separation. Associated
data binds the version, suite, repository, branch, epoch, object and block identity, chunk
position, and object kind — the "opaque header": those fields are authenticated but not
serialized, so a wrong context fails to decrypt rather than being detectable from the wire.

#strong[Commits and epochs.] A commit names its parents, branch, epoch, author key identity,
logical clock, and the operation object carrying the replicated-dataset delta as opaque bytes,
and is signed by a publisher key. An epoch transition additionally binds old and new epoch, old
and new topic, the new verification-key set, and the declared history policy, signed by an admin
key (`BR-7`).

#strong[Relay messages.] The relay exposes block existence, fetch, and store operations; topic
subscription and synchronization request with a frontier and a compact known-identifier summary;
and publish with admission checks. Bloom-style summaries are a bandwidth hint only, repaired by
parent-closure fetching, never a correctness mechanism. Every request carries a correlation
identifier and every response returns a typed error or success.

#strong[Relay disclosure ledger.] What a relay necessarily observes: transport facts (network
endpoint, session identity and duration); topic identifiers, subscriptions, publisher
registrations, and which peer publishes to or fetches from which topic; message types, timing,
ordering, cursors, retries, and sizes; ciphertext bytes, opaque block identifiers, the
requested, present, and missing identifier sets, retention state, and storage volume; and
registered publisher verification keys. With clear routing headers enabled it additionally
observes parent commit identifiers and block counts — i.e. the commit graph — which is why
`BR-2` makes opaque headers the default. What a conforming relay does not observe: RDF terms,
quads, graph names, deltas, plaintext commits, read secrets and private keys, SPARQL text,
plans, intermediate bindings, answers, and the materialized dataset.

#strong[Relay binding is an open decision.] Whether the relay is a dedicated service or an
existing pod acting as one is deliberately unresolved; both are bindings of the same abstract
block-plus-subscription contract, and nothing in the primitives forecloses either.

#strong[Crate shape.] Encryption and key material live only behind the opt-in crate boundary:
the core engine crates never link a cipher and do not depend on the E2EE crate, so the default
build and the browser artifact are unchanged by its existence. The relay links neither the query
engine nor the replicated-dataset algebra (`CLASS-3`).

= Annex B — verifiable answers over committed data (informative) <sec-annex-b>

This annex is #strong[informative] and is #strong[not] an encryption profile.

A different question from confidentiality-at-rest is whether a recipient can check that an
answer was computed correctly, or whether several parties can compute over data none of them
will reveal to the others. The sparq project has research-grade work in both directions:
single-prover zero-knowledge proofs of query answers over committed graphs, and secure
multi-party evaluation of federated patterns across mutually distrusting holders.

Neither is end-to-end encryption of storage, and this annex exists partly to prevent that
confusion. In the zero-knowledge case the prover holds cleartext and a commitment is not
a ciphertext; in the multi-party case each holder evaluates over its own cleartext and the
secret sharing is transient. They compose with Profile CS as an answer-verification layer, not
as a storage mechanism.

#note[
  #strong[Audit status, stated verbatim as the project's own gate requires.] The
  zero-knowledge and multi-party estate is #strong[research-grade and has NOT received external
  cryptographic review]; the multi-party protocols are honest-majority semi-honest only, and
  security against a malicious participant is designed, not built. No claim of a proven
  property is made for either, and production reliance is gated by the open audit bead
  `sq-qhy4`. A future revision #strong[MUST NOT] make a normative dependency of any profile in
  this document on that estate while the gate is open.
]

= Assertion groups <sec-groups>

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Group][Sections][Subject],
  [`E2EE-STATUS`, `E2EE-SCOPE`], [@sec-terminology], [proposal status and the three profiles],
  [`LEAK-1`–`LEAK-5`], [@sec-leakage], [the T0–T4 vocabulary and mandatory disclosure],
  [`CORE-1`–`CORE-9`], [@sec-core], [suite agility, client-side keys, padding, fail-closed,
    no server-side decryption, answer labelling, revocation],
  [`CS-1`–`CS-6`], [@sec-profile-cs], [the mandatory client-side profile],
  [`BR-1`–`BR-9`], [@sec-profile-br], [the broker-relayed collaboration profile],
  [`SE-1`–`SE-9`], [@sec-profile-se], [the structure-exposed, server-side-structural profile],
  [`CLASS-1`–`CLASS-5`, `TEST-1`], [@sec-classes], [conformance classes and fixture families],
  [`REJ-1`–`REJ-3`], [@sec-rejected], [rejections and one criteria-bound deferral],
)

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997. https://www.rfc-editor.org/rfc/rfc2119.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017. https://www.rfc-editor.org/rfc/rfc8174.]),
  ("RFC8949", [Bormann, C.; Hoffman, P. #emph[Concise Binary Object Representation (CBOR)].
    RFC 8949, IETF, December 2020. https://www.rfc-editor.org/rfc/rfc8949.]),
  ("RFC5869", [Krawczyk, H.; Eronen, P. #emph[HMAC-based Extract-and-Expand Key Derivation
    Function (HKDF)]. RFC 5869, IETF, May 2010. https://www.rfc-editor.org/rfc/rfc5869.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) #emph[RDF 1.1 Concepts and
    Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("SPARQL11-QUERY", [Harris, S.; Seaborne, A. (eds.) #emph[SPARQL 1.1 Query Language]. W3C
    Recommendation, 21 March 2013. https://www.w3.org/TR/sparql11-query/.]),
  ("SPARQL-CRDT", [#emph[SPARQL-CRDT: Conflict-Free Replicated RDF Datasets under SPARQL
    Update]. sparq project, Unofficial Proposal Draft. The replicated dataset incorporated by
    reference in `BR-5`.]),
  ("SOLID-PROTOCOL", [Capadisli, S.; Berners-Lee, T.; Verborgh, R.; Kjernsmo, K. (eds.)
    #emph[Solid Protocol]. Solid Community Group. https://solidproject.org/TR/protocol.]),
  ("NEXTGRAPH-PROTOCOL", [NextGraph. #emph[Sync Protocol]. https://docs.nextgraph.org/en/protocol/.
    Documented design of an unaudited alpha system; a design reference, not evidence.]),
  ("NEXTGRAPH-CRDT", [NextGraph. #emph[Conflict-Free Replicated Data Types (CRDT)].
    https://docs.nextgraph.org/en/framework/crdts/.]),
  ("Gie05", [Giereth, M. #emph[On Partial Encryption of RDF-Graphs]. ISWC 2005.]),
  ("FKPS17", [Fernández, J. D.; Kirrane, S.; Polleres, A.; Steyskal, S. #emph[Self-Enforcing
    Access Control for Encrypted RDF]. ESWC 2017, LNCS 10249, pp. 607–622.]),
  ("FKPS20", [Fernández, J. D.; Kirrane, S.; Polleres, A.; Steyskal, S. #emph[HDTcrypt:
    Compression and encryption of RDF datasets]. Semantic Web 11(2):337–359, 2020.]),
  ("CK10", [Chase, M.; Kamara, S. #emph[Structured Encryption and Controlled Disclosure].
    ASIACRYPT 2010, LNCS 6477, pp. 577–594. https://eprint.iacr.org/2011/010.]),
  ("FVYSHGKMC17", [Fuller, B.; Varia, M.; Yerukhimovich, A.; et al. #emph[SoK:
    Cryptographically Protected Database Search]. IEEE Symposium on Security and Privacy 2017,
    pp. 172–191. https://arxiv.org/abs/1703.02014.]),
  ("IKK12", [Islam, M. S.; Kuzu, M.; Kantarcioglu, M. #emph[Access Pattern Disclosure on
    Searchable Encryption: Ramification, Attack and Mitigation]. NDSS 2012.]),
  ("CGPR15", [Cash, D.; Grubbs, P.; Perry, J.; Ristenpart, T. #emph[Leakage-Abuse Attacks
    Against Searchable Encryption]. ACM CCS 2015, pp. 668–679.]),
  ("ZKP16", [Zhang, Y.; Katz, J.; Papamanthou, C. #emph[All Your Queries Are Belong to Us: The
    Power of File-Injection Attacks on Searchable Encryption]. USENIX Security 2016,
    pp. 707–720.]),
  ("NKW15", [Naveed, M.; Kamara, S.; Wright, C. V. #emph[Inference Attacks on
    Property-Preserving Encrypted Databases]. ACM CCS 2015, pp. 644–655.]),
  ("GSBNR17", [Grubbs, P.; Sekniqi, K.; Bindschaedler, V.; Naveed, M.; Ristenpart, T.
    #emph[Leakage-Abuse Attacks against Order-Revealing Encryption]. IEEE Symposium on Security
    and Privacy 2017. https://eprint.iacr.org/2016/895.]),
  ("KKNO16", [Kellaris, G.; Kollios, G.; Nissim, K.; O'Neill, A. #emph[Generic Attacks on
    Secure Outsourced Databases]. ACM CCS 2016.]),
  ("GMNRS16", [Grubbs, P.; McPherson, R.; Naveed, M.; Ristenpart, T.; Shmatikov, V.
    #emph[Breaking Web Applications Built On Top of Encrypted Data]. ACM CCS 2016.]),
  ("GRS17", [Grubbs, P.; Ristenpart, T.; Shmatikov, V. #emph[Why Your Encrypted Database Is Not
    Secure]. HotOS 2017.]),
  ("PRZB11", [Popa, R. A.; Redfield, C. M. S.; Zeldovich, N.; Balakrishnan, H. #emph[CryptDB:
    Protecting Confidentiality with Encrypted Query Processing]. SOSP 2011, pp. 85–100.]),
  ("NS09", [Narayanan, A.; Shmatikov, V. #emph[De-anonymizing Social Networks]. IEEE Symposium
    on Security and Privacy 2009.]),
  ("VJH21", [Viand, A.; Jattke, P.; Hithnawi, A. #emph[SoK: Fully Homomorphic Encryption
    Compilers]. IEEE Symposium on Security and Privacy 2021.]),
))
