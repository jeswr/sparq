// [OPUS-5] sq-tag1q.5 — E2EE-Queryable Solid/RDF: encryption profiles that preserve SPARQL
// query. 🤖 SPARQ agent — written by Claude Opus 5.
//
// GATE: this draft is downstream of the survey research/e2ee-queryable-options.md (bead
// sq-tag1q.3), and specifies ONLY the two profiles that survey concludes are specifiable
// today — Profile CS (§4, client-side evaluation over locally-decrypted data) and Profile SE
// (§5, value-encrypted / structure-exposed). The survey's rejected and deferred options
// (SSE/STE, DET/ORE, FHE) appear in §7 as "considered and excluded" with their citations, NOT
// as profiles. No aspirational cryptography is specified.
//
// HONESTY (the privacy-claims + no-perf-numbers gates scan this source at build time, and
// site/scripts/build-specs.mjs re-runs both at the build boundary):
//   - The impossibility statement is in the BODY (§1.2, §6.1), not a footnote: no known
//     construction gives general server-side SPARQL over end-to-end-encrypted data without
//     leakage. The profiles are disclosed points on a trade-off curve, not workarounds.
//   - Leakage is a NORMATIVE vocabulary (§3, tiers T0–T4) and every profile MUST declare its
//     tier — the device that keeps this document honest as it evolves.
//   - The ZK/MPC composition is Annex A, explicitly INFORMATIVE and non-normative: sparq-zk is
//     internally re-audited with NO external accredited-cryptographer sign-off (the open
//     external-audit gate sq-qhy4) and sparq-mpc is honest-majority semi-honest only. Nothing
//     in this document normatively depends on either, and no soundness property is claimed.
//   - No performance numbers appear here (a spec is a design surface, not a benchmark).

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "E2EE-Queryable Solid/RDF: Encryption Profiles that Preserve SPARQL Query")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#spec-head()

#intro-section("abstract", "Abstract")[
  This document specifies how Solid / RDF resources can be end-to-end encrypted — keys held
  by the data owner and their delegates, never by the storage server — while applications
  retain a useful form of SPARQL query. It defines exactly two conformance profiles, and
  deliberately no more. #dfn[Profile CS] (mandatory for E2EE conformance) encrypts resources
  at rest under an AEAD envelope, reduces the server to an authenticated blob store, and
  evaluates #strong[full SPARQL 1.1] on the client over locally decrypted data; it specifies
  the envelope format, the data-encryption/key-encryption key hierarchy, WebID-bound recipient
  key distribution and its alignment with pod access control, encrypted-container listing
  semantics, integrity binding, and the sync protocol. #dfn[Profile SE] (optional) encrypts
  literal #emph[values] while leaving graph #emph[structure] cleartext and server-queryable,
  and is conformant only if the implementation surfaces a mandatory leakage disclosure.
  Underpinning both is a normative leakage vocabulary (tiers T0–T4) that every profile and
  every future extension must declare. The document states plainly, in its body, that no known
  construction provides general server-side SPARQL evaluation over end-to-end-encrypted data
  without leakage; searchable/structured encryption, order-revealing encryption, and fully
  homomorphic encryption are recorded as considered and excluded, with the evidence.
]

#sotd()

#intro-section("security-standing", "Security standing of this proposal")[
  Two statements govern how this document may be read.

  #strong[First: the impossibility statement.] General server-side SPARQL evaluation over
  end-to-end-encrypted data #strong[without leakage does not exist] — not in the literature,
  not in any deployed system, and not in the reference implementation. Every known scheme that
  lets an untrusted server answer queries over ciphertext buys expressiveness with a disclosed
  leakage profile, and the leakage-abuse record shows those profiles are routinely exploitable
  (#cite("IKK12"), #cite("CGPR15"), #cite("NKW15"), #cite("GSBNR17"), #cite("ZKP16")). The two
  profiles below are points on that disclosed trade-off curve. Neither is a workaround of this
  fact, and an implementation #strong[MUST NOT] present either as one.

  #strong[Second: the audit posture.] Annex A sketches a composition with the sparq zero-
  knowledge and multi-party-computation estate. That annex is #strong[informative only].
  Those crates are research-grade: they are internally re-audited but have #strong[no external
  accredited-cryptographer sign-off] — the external-audit gate (tracked in-repo as
  #strong[sq-qhy4]) is #strong[open and pending] — and the MPC layer is honest-majority
  #emph[semi-honest] only. No normative requirement in this document depends on either crate,
  and no soundness property is claimed for either anywhere in this document.
]

= Introduction

A Solid pod, or any RDF resource server, stores a user's data. Two things the user wants are
in tension. They want the server operator to be unable to read the data — #dfn[end-to-end
encryption], with keys generated and held client-side. And they want applications to keep
querying that data with SPARQL. The tension is not incidental: SPARQL's expressiveness —
basic-graph-pattern joins, property paths, the `FILTER` algebra, aggregation — is exactly the
rich computation over plaintext that encryption exists to prevent.

This document resolves the tension the only honest way available: by moving evaluation, not by
inventing cryptography. Profile CS moves the whole query to the client, where the data is
plaintext, and pays for it in sync cost. Profile SE keeps structural evaluation on the server
by leaving structure unencrypted, and pays for it in disclosed leakage. Nothing else in the
option space (§7) is specifiable today.

== What "end-to-end encrypted" means in this document

Keys are generated and held client-side; the server never possesses plaintext, nor any key
that decrypts it. Three neighbouring mechanisms are therefore #strong[out of scope], and an
implementation #strong[MUST NOT] describe any of them as satisfying this document:

- #strong[Server-side encryption at rest.] The server holds the keys and can decrypt on
  demand. This is a valuable operator-side control — it bounds media-disposal and
  compromised-backup exposure, and supports crypto-shredding — but it is not E2EE.
- #strong[Transport encryption.] TLS protects data in flight from a network observer; the
  server terminates it and sees plaintext.
- #strong[Access control.] WAC and ACP decide which requests a server answers. A server that
  #emph[could] decrypt but is configured not to is enforcing policy, not confidentiality. This
  document composes with access control (§4.4) and never replaces it.

== The impossibility statement

Stated normatively, because it is the premise of every requirement that follows: there is no
known construction that evaluates general SPARQL server-side over end-to-end-encrypted data
while leaking nothing. A conforming implementation #strong[MUST NOT] claim, in documentation,
interface copy, or marketing, that it provides server-side SPARQL over encrypted data without
leakage; and #strong[MUST], for whichever profile it implements, disclose that profile's
leakage tier (§3).

This is not a temporary limitation awaiting an engineering push. The options that trade
leakage for server-side expressiveness are surveyed in §7 and excluded on evidence, not on
effort.

== What this document does not specify

- Any scheme in which the server evaluates value-level SPARQL over ciphertext: searchable or
  structured encryption indexes, deterministic or order-revealing encryption for `FILTER` and
  ranges, and fully homomorphic evaluation. Each is recorded in §7 with the reason.
- Any server-side decryption capability — a "decrypt" extension function, a key-holding query
  service, or a trusted-execution enclave binding. Placing a decryption key server-side
  forfeits the E2EE property this document defines, whatever the enclosing mechanism.
- The access-control algorithm itself. Admission and authorisation run beside this document
  and are unchanged by it.
- Group-messaging-grade key agreement (continuous group key agreement, post-compromise
  security). §4.4 specifies a key envelope adequate for resource sharing and states its limits
  honestly; a ratcheting group protocol is future work (§9).

= Terminology and conformance

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED],
#strong[MAY], and #strong[OPTIONAL] in this document are to be interpreted as described in
#cite("RFC2119") and #cite("RFC8174") when, and only when, they appear in all capitals, as
shown here.

An #dfn[encrypted resource] is a server-stored octet stream consisting of a cleartext envelope
header and an AEAD ciphertext (§4.2). A #dfn[data-encryption key] (DEK) is the symmetric key
under which a resource's plaintext is sealed. A #dfn[recipient] is a WebID, and its
#dfn[recipient key pair] is the #emph[asymmetric] long-term key pair — the public half
published under that WebID's profile document (§4.4) — that a DEK is encapsulated to. A
#dfn[key-encryption key] (KEK) is the #emph[symmetric] key that actually encrypts a DEK; under
the construction of §4.4.1 a KEK is never stored, but is derived afresh for each wrap by the
key-encapsulation and key-derivation steps applied to the recipient's public key. The two are
distinct objects, and this document #strong[does not] use "KEK" for either half of a
recipient's key pair: writing "the recipient's KEK" would conflate asymmetric key agreement
with the symmetric key it produces. A #dfn[key envelope] is the set of wrapped DEKs published
alongside an encrypted resource (§4.4). A #dfn[leakage tier] is the classification of §3. A
#dfn[disclosure statement] is the machine- and human-readable leakage declaration a
conforming implementation publishes (§3.3).

This document defines conformance for four classes:

#table(
  columns: 2,
  align: (left, left),
  table.header[Conformance class][Artefact],
  [#emph[E2EE client]], [Software holding keys: it seals, unseals, wraps, shares, and — under
    Profile CS — evaluates SPARQL locally (§4, §5).],
  [#emph[Storage server]], [A resource server holding encrypted resources. Under Profile CS
    its obligations are deliberately near-empty (§4.8).],
  [#emph[Encrypted resource]], [A stored octet stream and its envelope (§4.2), or a graph
    carrying encrypted literals (§5.1).],
  [#emph[Disclosure statement]], [The published leakage declaration (§3.3). Every conforming
    implementation of either profile MUST publish one.],
)

An implementation claiming E2EE conformance #strong[MUST] implement Profile CS. Profile SE is
#strong[OPTIONAL]; an implementation #strong[MUST NOT] claim E2EE conformance on the basis of
Profile SE alone, because Profile SE leaves the graph's entire structure in the clear (§5.6).

= Leakage tiers: the normative vocabulary

Leakage is the load-bearing concept of this document, so it is given a vocabulary rather than
left to prose. Every profile in this document declares a tier; every future extension
#strong[MUST] declare one.

== The tiers

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Tier][What the server (or a network observer) learns][Assessment],
  [#strong[T0]], [Enough to recover plaintext under realistic auxiliary knowledge — for
    example plaintext frequency, or plaintext order, over a low-entropy value population.],
    [Plaintext-equivalent under attack. #strong[MUST NOT] be specified as a profile by this
     document or any extension to it.],
  [#strong[T1]], [The full graph structure: every subject, predicate, IRI-valued object, and
    named-graph membership; degrees, co-occurrence, and update dynamics. Values are sealed.],
    [Confidentiality of values only. Requires the mandatory disclosure of §5.6.],
  [#strong[T2]], [Search pattern (query-token repetition), access pattern (which entries
    matched), and volume, as in searchable/structured encryption.], [Not specified here; §7.1
     records the deferral and the criteria for revisiting it.],
  [#strong[T3]], [Everything in T4, plus a #emph[query-correlated] resource-level access
    pattern: which ciphertexts a client fetches, when, and which it fetches together, in
    response to querying.], [Profile CS's default tier.],
  [#strong[T4]], [The per-resource metadata that storage itself entails, with the query
    correlation removed: resource identities, container cardinality, per-resource ciphertext
    sizes bucketed by the padding ladder, and creation and update timing — but #emph[not]
    which resources any particular query touched. T4 is #strong[not] "aggregate only": the
    server still distinguishes resources and observes each one's update pattern (§4.6, §6.2).],
    [Profile CS's tier under full-replica sync with padding (§4.7).],
)

The tiers are ordered by what an adversary can reconstruct, not by implementation difficulty.
T1 is #emph[not] "one step worse" than T3 in any quantitative sense: it is a categorically
different disclosure, because structure alone is strongly identifying (§6.3).

== Declaring a tier

A conforming implementation #strong[MUST] declare, for each profile it implements and each
configuration option that changes leakage, the resulting tier. Where an option moves the tier,
the implementation #strong[MUST] declare the tier that holds when the option is #strong[on],
and #strong[MUST NOT] advertise the off-state tier as the implementation's tier.

Two options in this document move the tier and are therefore separately opt-in and separately
declared: Profile SE's deterministic equality tags (§5.5) and Profile CS's cleartext
partitioning metadata (§4.7).

== The disclosure statement

An implementation of either profile #strong[MUST] publish a disclosure statement in its
conformance documentation, containing at minimum:

+ the profile(s) implemented and, for each, the declared leakage tier;
+ a plain-language sentence naming what the storage operator can observe;
+ the state of every tier-moving option (on or off) in the shipped default configuration;
+ the key-compromise consequences of §6.4, or a reference to them.

For Profile SE the disclosure statement is not merely required documentation: it is a
#strong[conformance requirement with prescribed content] (§5.6), because a user who is not
told that their graph's structure is visible cannot meaningfully consent to Profile SE.

= Profile CS: client-side evaluation over locally decrypted data

#strong[Profile CS is MANDATORY for E2EE conformance.] Declared leakage tier: #strong[T3],
improving to #strong[T4] under full-replica sync with padding (§4.7).

== Architecture

The server stores opaque octet streams and enforces access control over them. The client
fetches the ciphertexts it is authorised for, unwraps the DEKs it holds, decrypts, parses the
RDF, indexes it locally, and evaluates SPARQL against that local index. No query, no query
plan, and no answer ever reaches the server.

```text
  owner/delegate client                          storage server
  ─────────────────────                          ──────────────
  plaintext RDF                                   (never plaintext,
    │ seal (AEAD, §4.2)                            never a key)
    ▼
  envelope ‖ ciphertext ──── PUT ──────────────▶  encrypted resource
                                                  + key envelope (§4.4)
  local index ◀── parse ◀── decrypt ◀── GET ────  ciphertext
    │
    ▼
  SPARQL 1.1 evaluated here
```

Query expressiveness under Profile CS is #strong[full SPARQL 1.1] #cite("SPARQL11"), with no
fragment carve-out, because evaluation happens over plaintext. This is the profile's
distinguishing property and the reason it is the mandatory core.

The evaluation topology is not novel: client-side SPARQL engines querying servers that only
serve documents are established practice in this ecosystem, including link-traversal query over
Solid pods #cite("TV23"). What Profile CS adds is the decryption step between fetch and parse.
The cautionary precedent is equally relevant: Mylar #cite("PSVBZ14") built applications on
client-held keys and had its #emph[server-searchable] additions broken #cite("GMNRS16") while
the plain client-side-decrypt path survived. That is the shape of the trade-off this document
encodes — the leaky part is always the part the server was asked to search.

== The encrypted-resource envelope

An encrypted resource #strong[MUST] consist of a cleartext envelope header followed by an
AEAD ciphertext, serialised exactly as §4.2.5 specifies. The header carries only framing and
key-selection metadata; every value that describes the plaintext is sealed. The cleartext
header #strong[MUST] carry:

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Cleartext header field][Requirement][Purpose],
  [Format version], [MUST], [Permits later revisions to be distinguished; an unknown version
    MUST cause the client to refuse the resource, never to guess.],
  [AEAD suite identifier], [MUST], [Names the sealing algorithm from the registry of §4.2.1.
    An unregistered identifier MUST be refused.],
  [DEK identifier], [MUST], [Selects which wrapped DEK in the key envelope (§4.4) applies.
    An opaque identifier; it MUST NOT encode the plaintext's content or type.],
  [Nonce], [MUST], [The AEAD nonce, generated as required by §4.2.2.],
  [Ciphertext length], [MUST], [Framing only: the octet length of the AEAD ciphertext
    following the header, authentication tag included. Because it measures the #emph[padded]
    plaintext, it discloses the padding bucket of §4.2.3 and nothing finer. It is carried
    explicitly so that an envelope can be embedded in a larger encoding (§5.1).],
  [Resource version counter], [SHOULD, per §4.2.4], [The rollback-detection counter. It is
    cleartext because a client needs it to reconstruct the associated data, and it is
    authenticated by being bound into that associated data.],
)

Two values that describe the plaintext are #strong[sealed inside the ciphertext] instead of
being carried in the header. They are authenticated by the AEAD and are recoverable only after
a successful decryption:

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Sealed plaintext field][Requirement][Purpose],
  [True plaintext length], [MUST be sealed, MUST NOT be in the cleartext header], [The
    #emph[unpadded] octet length of the RDF plaintext, so the padding of §4.2.3 can be
    stripped after decryption. This is a different quantity, in a different place, from the
    cleartext ciphertext length above; §4.2.5 gives each its own name and encoding.],
  [Content type of the plaintext], [MUST be sealed, MUST NOT be in the cleartext header],
    [The RDF serialisation of the plaintext is metadata about the plaintext; carrying it in
     the clear leaks the resource's nature for no benefit.],
)

The cleartext header #strong[MUST NOT] carry the resource's RDF content type, its true
(unpadded) byte count, its triple count, its vocabulary, or any label derived from its
contents.

=== Algorithm registry

Every AEAD suite has a registered two-octet identifier — the value the header's AEAD suite
identifier field carries (§4.2.5) — and fixed parameter sizes. Version 1 of this format
registers exactly two:

#table(
  columns: 5,
  align: (left, left, left, left, left),
  table.header[Suite][AEAD][Key][Nonce][Tag],
  [`0x0001`], [`XChaCha20-Poly1305` #cite("XCHACHA")], [32 octets], [24 octets], [16 octets],
  [`0x0002`], [`AES-256-GCM` #cite("GCM")], [32 octets], [12 octets], [16 octets],
)

A conforming client #strong[MUST] implement suite `0x0001` and #strong[SHOULD] implement suite
`0x0002`. Any other identifier is unregistered and #strong[MUST] be refused.

#strong[On the normative reference for suite `0x0001`.] #cite("RFC8439") specifies ChaCha20,
Poly1305, and the ChaCha20-Poly1305 AEAD with a 96-bit nonce. It does #strong[not] specify
XChaCha20-Poly1305, nor the HChaCha20 step that extends the nonce to 192 bits. The normative
definition of suite `0x0001` — the HChaCha20 subkey derivation, the 192-bit nonce
construction, and the resulting AEAD — is therefore #cite("XCHACHA"), which builds on the
primitives of #cite("RFC8439") but is a distinct construction. An implementation
#strong[MUST NOT] treat the two as interchangeable: they take different nonce sizes and derive
different keys, so a resource sealed under one cannot be opened under the other, and a client
that substitutes RFC 8439 ChaCha20-Poly1305 for suite `0x0001` is not conformant.

A client #strong[MUST NOT] seal with any algorithm that is not an AEAD: confidentiality
without integrity is inadequate here, because a storage server that can flip ciphertext bits
undetected can corrupt query answers at will (§6.5). Registry governance — how an algorithm
identifier is added or deprecated — follows the process of this specification series and is
not defined in this document; it is tracked by bead sq-tag1q.5 and revisited in §9.

=== Nonce and key generation

All keys, nonces, and salts #strong[MUST] be produced by a cryptographically strong random
number generator. A nonce #strong[MUST NOT] be reused with the same DEK: nonce reuse under
either registered algorithm is catastrophic, forfeiting confidentiality of the affected
plaintexts and, for `AES-256-GCM`, the authentication key. Because a Profile CS deployment is
inherently multi-device and multi-writer, a client #strong[MUST NOT] rely on a counter it
believes to be globally monotonic; it #strong[MUST] either generate the nonce at random under
suite `0x0001`, whose 24-octet nonce makes random generation safe at any realistic message
count, or derive a fresh DEK per sealing operation. Random nonce generation under suite
`0x0002` is #strong[NOT] safe at scale: 12 octets is too short to rely on collision
improbability across a multi-writer deployment, so a client sealing under `0x0002`
#strong[MUST] derive a fresh DEK per sealing operation.

=== Padding

Ciphertext length leaks plaintext length, and RDF resource sizes are informative (a
five-triple profile document and a ten-thousand-triple activity log are not confusable). A
client #strong[MUST] pad plaintext to a bucket drawn from a padding ladder declared in the
disclosure statement, and #strong[MUST] seal the true length inside the plaintext so padding
is removable only after decryption. A client #strong[SHOULD] use a ladder whose buckets grow
geometrically rather than a fixed block size, so that large resources do not have their sizes
revealed to within a constant.

=== Integrity binding

The AEAD's associated data #strong[MUST] bind, at minimum: the format version, the algorithm
identifier, the DEK identifier, and #strong[the resource's own identity] (its IRI, or a
stable identifier for it). Binding the identity is what prevents a #dfn[relocation attack]: a
server that serves resource A's ciphertext in response to a request for resource B cannot do
so undetected, because decryption under B's associated data fails.

Binding identity does #strong[not] prevent a #dfn[rollback attack] — serving a genuine
#emph[older] version of the same resource. A client #strong[SHOULD] additionally bind a
monotonically increasing version counter into the associated data and remember the highest
version it has seen per resource, and a client that does so #strong[MUST] refuse a resource
whose bound version is lower than the remembered one. Implementations #strong[MUST] state in
the disclosure statement whether rollback detection is provided; §6.5 explains why omission
and rollback are the residual powers a storage server retains under every profile here.

An implementation #strong[MAY] additionally content-address encrypted resources (naming a
resource by a digest of its ciphertext). Content addressing gives deduplication and a
convenient integrity handle, but it #strong[MUST NOT] be treated as a substitute for the AEAD
binding above, and implementers should note that content-addressed names are stable across
readers and therefore make co-access correlation easier for the server, not harder.

=== Canonical wire format

The fields above are only interoperable if their encoding is fixed, so this section fixes it.
A conforming client #strong[MUST] serialise and parse an encrypted resource exactly as
specified here. Throughout this document: all integers are #strong[unsigned and big-endian];
`u8`, `u16`, `u32`, and `u64` denote 1-, 2-, 4-, and 8-octet integers; content types are
encoded as UTF-8 in Unicode Normalization Form C, while every IRI follows the term-identifier
rule immediately below; and `base64url` means the URL-and-filename-safe alphabet of
#cite("RFC4648") §5 #strong[without] padding — a client #strong[MUST] reject a base64url value
carrying `=` padding or a non-alphabet octet, rather than accepting it leniently.

#strong[Canonical encoding of RDF term identifiers.] Wherever this document feeds an IRI into
associated data, a MAC input, or a sealed field — `resource_id` (§4.2.5, §4.4.1), and
`graph_id`, `subject_id`, `predicate_id`, and `datatype_iri` (§5.1, §5.5.1) — that IRI is
encoded as the #strong[UTF-8 encoding of its exact Unicode code-point sequence], and a client
#strong[MUST NOT] transform it in any way beforehand. Specifically, a client #strong[MUST NOT]
apply Unicode normalisation in any form (NFC, NFD, or otherwise), #strong[MUST NOT] add,
remove, or case-change percent-encoding, and #strong[MUST NOT] apply scheme or host case
folding, default-port elision, dot-segment removal, or any other IRI-equivalence rewrite. The
reason is that in RDF an IRI's identity #emph[is] its spelling: `http://example.org/%7Ea` and
`http://example.org/~a` are different IRIs and therefore different RDF terms, so a client that
canonicalised either one would compute octets no other client reproduces, and would produce
ciphertext that is undecryptable after interchange. This rule is the single rule for all of
these fields; it overrides the UTF-8-in-NFC convention stated above, which applies to content
types only. Every such identifier is an #strong[absolute] IRI — resolved against its base
before use, never carried in relative form — and is carried behind a `u16` length prefix, so
its UTF-8 encoding #strong[MUST] be at most 65535 octets: a client whose identifier would
exceed that #strong[MUST] refuse the operation rather than truncate, and a parser
#strong[MUST] refuse any length prefix that disagrees with the octets that follow it.

```text
EncryptedResource:
  magic            8   octets  = ASCII "SPARQE2E"
  version          u8           = 0x01
  suite_id         u16          AEAD suite from the §4.2.1 registry
  flags            u8           bit 0 = version_counter present; bits 1-7 MUST be 0
  dek_id           16  octets   opaque, uniformly random (§4.2, §4.4)
  nonce            N   octets   N = the suite's nonce length (24 or 12)
  version_counter  u64          present iff flags bit 0 is set
  ciphertext_len   u64          octet length of `ciphertext`, tag included
  ciphertext       ciphertext_len octets

SealedPlaintext (the AEAD plaintext, recovered only after a successful open):
  plaintext_len    u64          TRUE, UNPADDED length of `rdf_bytes`
  content_type_len u16
  content_type     content_type_len octets   e.g. "text/turtle"
  rdf_bytes        plaintext_len octets
  padding          zero octets to the §4.2.3 ladder bucket
```

The associated data required by §4.2.4 is constructed canonically, so that two independent
implementations compute the same octets. Every variable-length component is length-prefixed,
which makes the encoding unambiguous — without the prefixes a resource IRI could be chosen to
impersonate the boundary between two fields:

```text
AAD = "sparq-e2ee/v1/envelope" || 0x00
   || version || suite_id || flags || dek_id
   || u16 length of resource_id || resource_id
   || version_counter          (u64; the value 0 when flags bit 0 is clear)
```

`resource_id` is the resource's absolute IRI, encoded by the term-identifier rule above:
percent-encoding, case, and every other aspect of its spelling preserved exactly as they appear
in that absolute IRI, with no normalisation. It is #emph[not] carried in the header: the client
already knows which resource it asked for, and binding the identity it #emph[expected] is
precisely what detects the relocation attack of §4.2.4.

#strong[Bounds and failure behaviour.] A client #strong[MUST] enforce, before allocating on a
length field, that `ciphertext_len` is at least the suite's tag length and no greater than an
implementation-declared maximum, that `content_type_len` is at most 255, and that
`plaintext_len` plus the framing fields does not exceed the recovered plaintext. A client
#strong[MUST] refuse — with no plaintext, and no partial output — any resource whose `magic`
does not match, whose `version` is unrecognised, whose `suite_id` is unregistered, whose
`flags` set a reserved bit, whose lengths are inconsistent, or whose AEAD open fails. These
conditions #strong[MUST NOT] be distinguished from one another in anything the storage server
can observe: a client that reports "bad padding" differently from "bad tag" hands the server
a decryption oracle.

== Key hierarchy

A conforming client #strong[MUST] use a two-level hierarchy:

+ A #strong[DEK] seals a resource's plaintext. A DEK #strong[MUST] be generated per resource,
  and #strong[MUST NOT] be reused across resources; sharing one DEK across a container turns
  every future grant into an all-or-nothing grant over that container's history.
+ A #strong[KEK] encrypts a DEK for one recipient. A DEK is shared by encapsulating to the
  recipient's #emph[public key], which derives a fresh KEK that wraps the DEK (§4.4.1); the
  plaintext is never re-encrypted to share it. The recipient's key pair is long-term; the KEK
  it yields is per-wrap and is never stored by either party.

A client #strong[MUST NOT] derive a DEK deterministically from the plaintext (convergent
encryption). Convergent encryption makes equal plaintexts produce equal ciphertexts across
users, which hands the server a plaintext-confirmation oracle — a T0 disclosure for any
guessable resource.

== Recipient key distribution and access-control alignment

Recipient public keys are #strong[WebID-bound]: a recipient is named by a WebID, and its
public keys are discovered by dereferencing the WebID profile document. A client
#strong[MUST] verify that a key it wraps to is published under the recipient's own WebID
document authority, and #strong[MUST NOT] accept a recipient key learned from the key envelope
itself or from any party other than the recipient's WebID document — otherwise the storage
operator can insert its own key as a "recipient" and read everything subsequently shared.

The key envelope published alongside an encrypted resource carries one wrapped DEK per
recipient. Its recipient set is the #emph[real] read-authorisation boundary.

#note[
  #strong[The two enforcement points are independent, and only one of them is load-bearing.]
  A pod's ACL decides who may #emph[fetch] a ciphertext. The key envelope decides who may
  #emph[read] it. A client MUST keep them aligned — the recipient set SHOULD equal the set of
  agents granted read access — but implementers must understand the asymmetry: an agent in the
  ACL but not the key envelope gets ciphertext it cannot read (a confusing but safe state),
  whereas an agent in the key envelope but not the ACL retains the ability to read the
  resource the moment it obtains the bytes by any means. Removing an agent from the ACL alone
  #strong[does not] revoke its ability to decrypt. Revocation is §4.5.
]

A client #strong[MUST] surface any divergence between the ACL and the key envelope to the
data owner, rather than silently reconciling it in either direction.

=== Key wrapping and the key-envelope format

"Wrapped to a public key" is not an algorithm, so this section names one. A conforming client
#strong[MUST] wrap DEKs using #dfn[HPKE] #cite("RFC9180") in base mode (`mode_base`),
single-shot `Seal`. HPKE is referenced rather than reinvented because it is exactly the
authenticated key-encapsulation, key-derivation, and AEAD composition this step needs, with
the domain separation already specified. Version 1 registers one HPKE suite:

#table(
  columns: 5,
  align: (left, left, left, left, left),
  table.header[Wrap suite][KEM][KDF][AEAD][Encapsulated key],
  [`0x0001`], [DHKEM(X25519, HKDF-SHA256) (`0x0020`)], [HKDF-SHA256 (`0x0001`)],
    [ChaCha20-Poly1305 (`0x0003`)], [32 octets],
)

The KEM encapsulation derives the shared secret; HPKE's key schedule derives from it the
per-wrap #strong[KEK] that seals the DEK. The recipient's key pair is the asymmetric input;
the KEK is the symmetric output. Neither party stores the KEK.

```text
KeyEnvelope:
  magic            8   octets  = ASCII "SPARQE2K"
  version          u8           = 0x01
  dek_id           16  octets   the resource header's dek_id (§4.2.5)
  recipient_count  u16          MUST be >= 1
  recipients       recipient_count × RecipientEntry, sorted ascending by recipient_kid

RecipientEntry:
  recipient_kid    32  octets   SHA-256 over the recipient's HPKE public key encoding
  wrap_suite_id    u16          from the registry above
  enc_len          u16          length of `enc`
  enc              enc_len octets    the HPKE encapsulated key
  wrapped_len      u16
  wrapped_dek      wrapped_len octets   HPKE ciphertext of the DEK, tag included
```

`recipient_kid` is a #emph[key] identifier, not a WebID: it identifies which key pair an entry
was encapsulated to, so a recipient holding several keys — or rotating one (§4.5) — can select
its entry without trial decryption. Entries are sorted by `recipient_kid` so the serialisation
is canonical and the entry order cannot itself become a side channel. Deriving the identifier
by digest, rather than carrying the WebID, keeps the recipient #emph[set] from being readable
off the envelope by the storage operator; implementers should note this hides identities only
against an operator that cannot guess the candidate WebIDs, and §6.2 does not claim otherwise.

The HPKE `info` and `aad` parameters #strong[MUST] be:

```text
info = "sparq-e2ee/v1/wrap" || 0x00
    || u8 wrap format version (0x01) || wrap_suite_id || dek_id
    || u16 length of resource_id || resource_id
aad  = recipient_kid
```

Binding `dek_id` and `resource_id` into `info` is what stops a storage operator from moving a
recipient entry between key envelopes: an entry wrapped for resource A does not open under
resource B's `info`. A client #strong[MUST] treat any HPKE `Open` failure, unknown
`wrap_suite_id`, unknown `version`, or malformed length as a refusal of that entry, and
#strong[MUST NOT] fall back to another entry's key material or to an unauthenticated path.

== Sharing, rotation, and revocation

#strong[Sharing] a resource with a recipient means wrapping its DEK to that recipient's
verified public key and adding the wrapped DEK to the key envelope. Sharing #strong[MUST NOT]
require re-encrypting the resource.

#strong[Rotation.] A client #strong[SHOULD] support rotating a resource's DEK (re-sealing the
plaintext under a fresh DEK and re-wrapping to the current recipient set) and #strong[MUST]
support rotating a recipient's #emph[key pair] (re-wrapping that recipient's DEKs to a fresh
public key, and publishing the new key under its WebID per §4.4).

#strong[Revocation] is the honest hard case, and this document specifies it plainly rather
than optimistically. Removing a recipient from a key envelope prevents that recipient from
decrypting #emph[future] versions only. It cannot retract what the recipient already holds,
and it cannot help if the recipient retained a copy of the ciphertext and its DEK. Therefore:

- A client that revokes a recipient #strong[MUST] re-seal the affected resources under fresh
  DEKs and re-wrap only to the remaining recipients (#dfn[re-encrypt on membership change]).
- A client #strong[MAY] instead defer re-sealing until the resource next changes
  (#dfn[lazy revocation]). Lazy revocation is a documented weakening: between revocation and
  the next write, the revoked recipient can still decrypt any copy of the ciphertext it can
  obtain. An implementation offering lazy revocation #strong[MUST] disclose it and
  #strong[MUST NOT] describe the revocation as immediate.
- Neither approach provides post-compromise security against a recipient that archived
  plaintext. No cryptographic mechanism does. Implementations #strong[MUST NOT] imply
  otherwise.

== Encrypted-container listing semantics

Containers are where a naive E2EE deployment leaks most, because container listings are
structure the server serves in the clear by construction.

- Resource names inside an encrypted container #strong[MUST] be opaque: an identifier that is
  independent of the resource's contents, type, and human-meaningful label. A container
  holding `medical-records.ttl` has already disclosed the interesting part.
- The human-meaningful names, and any client-side hierarchy, #strong[MUST] be carried in an
  #dfn[encrypted manifest] — itself an encrypted resource under §4.2 — and never in container
  membership.
- A container listing nonetheless discloses #strong[cardinality] (how many resources) and
  #strong[creation and update timing]. This is unavoidable when the server serves containers,
  and #strong[MUST] be stated in the disclosure statement.
- A client #strong[SHOULD NOT] mirror the plaintext hierarchy in the container hierarchy: a
  flat container of opaquely named resources, with structure held only in the encrypted
  manifest, discloses strictly less.

== Synchronisation, and the cost this profile actually pays

The client's local index must be populated before it can answer anything. A conforming client
#strong[MUST] be able to determine which encrypted resources it is authorised for and which
have changed, and #strong[SHOULD] do so from an encrypted manifest listing resource
identifiers and version indicators, refreshed by conditional requests or change notification.

The honest cost — stated because it is the profile's real trade-off, not a footnote — is that
there is #strong[no server-side selectivity]. The client must fetch and decrypt every resource
that could contribute to an answer; work scales with the authorised, potentially relevant
corpus rather than with the answer size, and the first query after a cold start pays for
decrypting, parsing, and indexing all of it. Implementers should also note that a browser
client runs against the `wasm32` linear-memory ceiling and a single thread, which bounds the
practical in-browser corpus; a native client does not have that ceiling.

Three mitigations exist, in #strong[increasing] leakage order. An implementation
#strong[MUST] treat them as tier-moving options (§3.2):

+ #strong[Full-replica sync] — fetch everything the client is authorised for, on a schedule
  independent of queries. This is the #strong[T4] mode: because fetching is uncorrelated with
  querying, the resource-level access pattern stops carrying a query signal. It does not stop
  existing — the server still sees which resources exist, how many, their bucketed ciphertext
  sizes, and when each one changes (§4.6, §6.2) — so this mitigation removes the correlation,
  not the metadata. This is the #strong[RECOMMENDED] default where corpus size permits.
+ #strong[Selective fetch] — fetch only the resources a query might need. This is the
  #strong[T3] default: the server observes which ciphertexts are fetched together and when.
+ #strong[Cleartext partitioning metadata] — attach coarse cleartext tags (a topic, a graph
  category) to resources so the client can fetch a subset. This discloses the category of every
  resource to the server and #strong[MUST] be separately opt-in and separately declared. It
  #strong[MUST NOT] be enabled by default.

A client-maintained encrypted index sidecar (the client uploads its own encrypted index shards
and fetches only the shards it needs) is a fourth point on this ladder, with shard-access-
pattern leakage that approaches the structured-encryption tier of §7.1. It is not specified in
this document; see §9.

== Server conformance

Deliberately minimal, and that is the point. A conforming storage server:

- #strong[MUST] store and return encrypted resources as opaque octet streams, unmodified;
- #strong[MUST NOT] parse, index, transform, or content-negotiate the ciphertext, and
  #strong[MUST NOT] require the plaintext's RDF content type to be declared;
- #strong[MUST] continue to enforce its ordinary access control over those resources;
- #strong[MUST NOT] hold any DEK, KEK, or recipient private key, and #strong[MUST NOT] offer
  any interface that decrypts on a client's behalf.

An ordinary Solid server #cite("SOLID") already satisfies the first three. That near-zero
server surface is the deliberate design outcome: Profile CS is deployable against unmodified
storage.

= Profile SE: value-encrypted, structure-exposed

#strong[Profile SE is OPTIONAL.] Declared leakage tier: #strong[T1], and #strong[T1] with an
additional value-equality pattern when the equality tags of §5.5 are enabled.

Profile SE exists because there is a real and useful intermediate point between "the server
sees everything" and "the server sees nothing and does nothing": hiding the #emph[values] in a
graph — measurements, names, diagnoses, amounts — from the storage operator while structural
application queries keep working server-side. It is specifiable because it requires no novel
cryptography: an encrypted literal is just a literal. It is also the profile with genuine
RDF-specific prior art, in partial encryption of RDF graphs #cite("Gie05"), policy-scoped
encryption of graph fragments #cite("FKPS17"), and compressed-and-encrypted RDF datasets
#cite("FKPS20"); what none of that prior art settled, and what this profile contributes, is an
interoperable encoding and a mandatory disclosure discipline.

It is #strong[not] a substitute for Profile CS, and §5.6 is the conformance requirement that
keeps that honest.

== The encrypted-literal encoding

Under Profile SE, literal #strong[objects] are sealed and everything else stays cleartext.
Concretely, a conforming client:

- #strong[MUST] encrypt literal objects it is configured to protect, replacing each with a
  literal whose datatype IRI is the #dfn[encrypted-literal datatype]
  `https://sparq.dev/ns/e2ee#encryptedLiteral` — registered by this document, and the only
  datatype IRI that carries the semantics of §5.3 — and whose lexical form is the base64url
  encoding (#cite("RFC4648") §5, no padding, per §4.2.5) of a #strong[literal envelope];
- #strong[MUST NOT] encrypt subjects, predicates, graph names, or IRI- and blank-node-valued
  objects — under this profile those are cleartext by definition, and an implementation that
  encrypted some of them would be claiming a protection this profile does not provide;
- #strong[MUST] bind, in the AEAD associated data, the triple's subject, predicate, and graph
  name, so an encrypted value cannot be moved to a different predicate or subject undetected.
  Without this binding a server can swap two patients' sealed diagnoses, and every client
  decrypts the swap without noticing.

Because the original datatype and language tag are metadata about the value, a client
#strong[MUST] seal them inside the envelope rather than preserving them on the encrypted
literal: leaving `xsd:date` visible on an encrypted object narrows the plaintext considerably.

The literal envelope follows the conventions of §4.2.5 — big-endian unsigned integers, UTF-8
text under that section's term-identifier rule for IRIs, length-prefixed variable fields — and
differs from the resource envelope in that it selects a key family (§5.2) rather than a
per-resource DEK, and seals the term's own metadata:

```text
LiteralEnvelope:
  magic            8   octets  = ASCII "SPARQE2L"
  version          u8           = 0x01
  suite_id         u16          from the §4.2.1 registry
  key_family_id    16  octets   selects the key family of §5.2
  nonce            N   octets   N = the suite's nonce length
  ciphertext_len   u32
  ciphertext       ciphertext_len octets

SealedValue (the AEAD plaintext):
  term_kind        u8           0x01 = literal; other values reserved
  datatype_len     u16
  datatype_iri     datatype_len octets   the ORIGINAL datatype IRI
  lang_len         u8
  lang             lang_len octets       BCP 47, lowercased; empty when absent
  lexical_len      u32
  lexical_form     lexical_len octets

AAD = "sparq-e2ee/v1/literal" || 0x00
   || version || suite_id || key_family_id
   || u16 length of graph_id     || graph_id      (empty for the default graph)
   || u16 length of subject_id   || subject_id
   || u16 length of predicate_id || predicate_id
```

`graph_id`, `subject_id`, and `predicate_id` are the absolute IRIs naming the triple's graph,
subject, and predicate, each encoded by the term-identifier rule of §4.2.5 — the UTF-8 octets
of the IRI's exact code-point sequence, with no Unicode normalisation, no percent-encoding
change, no IRI-equivalence rewrite, and at most 65535 octets behind its `u16` prefix. Length
prefixes remove the boundary ambiguity between the three; this rule is what makes each one's
octets reproducible, and both are required for two independent clients to compute the same
associated data. The #strong[default graph] is represented by a `graph_id` of
#strong[zero length], which is unambiguous because an absolute IRI is never empty; a client
#strong[MUST NOT] substitute a sentinel IRI for it, and #strong[MUST] refuse a triple whose
`subject_id` or `predicate_id` encodes to zero octets. Inside `SealedValue`, `datatype_iri`
and `lexical_form` are likewise the term's exact octets, so that decrypting returns the same
RDF term that was sealed rather than a normalised neighbour of it.

A protected triple's subject #strong[MUST] be an IRI, or a blank node the client has
skolemised to a stable IRI before sealing. This is a real constraint, not a formality:
blank-node labels are not preserved across serialisations, so binding an unskolemised label
would make a value undecryptable after any round trip through a server that relabels.

A conforming implementation #strong[MUST] publish cross-implementation test vectors for the
literal envelope covering, at minimum: a default-graph triple and a named-graph triple that are
otherwise identical (distinct associated data, and neither openable under the other's); two
subject IRIs differing only in percent-encoding spelling, such as `http://example.org/%7Ea` and
`http://example.org/~a` (distinct associated data); two predicate IRIs whose NFC and NFD
spellings differ (distinct associated data); and a graph IRI containing the octets of a length
prefix.

== Key families

A client #strong[MUST] support per-predicate DEK families, so that disclosure can be
selective: sharing the key family for `schema:name` need not share the family for a health
predicate. A client #strong[MAY] additionally scope key families per resource. Key families
are wrapped to recipients exactly as in §4.4, and revocation behaves exactly as in §4.5 — with
the same honest limits.

== Query semantics: what survives, and what must fail

Server-side evaluation over a Profile SE graph is evaluation over the cleartext skeleton.
The following work unchanged, because they never inspect a value: basic-graph-pattern matching
and joins on subjects, predicates, and IRI-valued objects; property paths; `OPTIONAL`,
`UNION`, and `MINUS` over structure; and counting over structure.

The following #strong[MUST NOT] be evaluated over encrypted literals server-side: value
comparison and ordering, arithmetic, string functions, regular-expression matching,
aggregation over values, and value-based joins.

This is the profile's most dangerous corner, and it needs a hard normative rule, because the
failure mode is #emph[silent wrong answers] rather than an error. An encrypted literal is
syntactically an ordinary typed literal; a naive engine will happily compare two of them. But
sealing is randomised, so two encrypted literals with #emph[equal plaintexts] have unequal
lexical forms — a plaintext-equality test evaluates to `false`, and an ordering test imposes
ciphertext order, which is meaningless. Therefore:

- An engine #strong[MUST] treat the encrypted-literal datatype as opaque, with an empty value
  space and no ordering.
- Relational and ordering operators over an encrypted literal #strong[MUST] raise a type
  error, which eliminates the solution under SPARQL's `FILTER` semantics #cite("SPARQL11").
  They #strong[MUST NOT] return `false`, and #strong[MUST NOT] fall back to lexical
  comparison.
- This requirement deliberately overrides the RDF-term-equality fallback that SPARQL applies
  to literals of unrecognised datatypes. Under that fallback, two #emph[identical] encrypted
  literals would compare equal — which would report ciphertext identity as though it were
  plaintext equality, and so leak, through query results, exactly the equality relation that
  §5.5 makes a separately opt-in disclosure. An engine #strong[MUST] therefore raise the type
  error even when the two encrypted literals are the same RDF term.
- An engine #strong[MUST NOT] implement any function that decrypts an encrypted literal
  server-side. Such a function requires a key on the server and forfeits the E2EE property
  (§1.3).

Answers therefore return #strong[with ciphertext literals in them]. The client decrypts the
bindings it receives and #strong[MAY] apply the value-level `FILTER`, ordering, or aggregation
locally — a hybrid evaluation whose server half is structural and whose client half is
value-level. A client #strong[SHOULD] make this split visible to the application rather than
silently returning a partially filtered result set.

== Query rewriting and its limits

A client-side rewriter #strong[MAY] transform an application query into a structural
server-side query plus a local post-processing step. Where a query's value-level operations
cannot be moved to the client without changing the answer — for example a `LIMIT` applied
after a value-level `ORDER BY`, where the server cannot know which solutions the client will
keep — the rewriter #strong[MUST] either fetch the unlimited structural result and complete
the operation locally, or refuse the query. It #strong[MUST NOT] push a cardinality-reducing
operator past a value-level operator it cannot evaluate.

== Deterministic equality tags: separately opt-in, separately disclosed

Server-side value equality can be restored by attaching a #dfn[deterministic equality tag] to
each encrypted value: a keyed pseudorandom function (an HMAC under a client-held per-predicate
tag key) over the plaintext term, canonically encoded as §5.5.1 specifies, published as a
cleartext companion. Equal plaintexts then produce equal tags, so the server can perform
equality joins on values and evaluate `FILTER(?x = constant)` after the client rewrites the
constant to its tag — subject to the exact equality relation §5.5.1 defines, which is
narrower than SPARQL value equality.

This is a #strong[distinct leakage increment], and this document treats it as such:

- Equality tags #strong[MUST] be separately opt-in, per predicate, and #strong[MUST NOT] be
  enabled by default.
- Enabling them #strong[MUST] be declared as a tier-moving option (§3.2), and the disclosure
  statement #strong[MUST] name the predicates on which they are enabled.
- The tag key #strong[MUST] be distinct from the value DEK family, so that granting a
  recipient the ability to decrypt values and granting a party the ability to match them are
  separable.
- Tags #strong[MUST NOT] be derived without a secret key (a bare digest of the value is a
  brute-forceable commitment to a low-entropy plaintext, which is a T0 disclosure).

#note[
  #strong[Normative warning.] Equality tags disclose the #emph[frequency distribution] of
  values under each tagged predicate. Frequency analysis against a public prior is precisely
  the attack that recovered property-preserving-encrypted database columns wholesale in
  #cite("NKW15"). Personal-data predicates are the worst case for it: names, places, dates,
  and diagnoses are low-entropy populations with published distributions. An implementation
  #strong[SHOULD NOT] offer equality tags on predicates whose value populations are small or
  skewed, and #strong[MUST NOT] describe a tagged predicate's values as hidden from the
  storage operator.
]

Order-revealing tags — anything that would restore range `FILTER` or `ORDER BY` server-side —
are #strong[MUST NOT] under this document. §7.2 records the evidence.

=== The equality relation a tag decides

"Equal plaintexts produce equal tags" is only well defined once "equal" is. RDF and SPARQL
offer several candidate relations, and they disagree: `"1"^^xsd:integer` and
`"01"^^xsd:integer` are the #emph[same value] but #emph[different terms]; `"1"^^xsd:integer`
and `"1.0"^^xsd:decimal` are equal under SPARQL value equality and unequal as terms;
`"NaN"^^xsd:double` is not value-equal even to itself; two `xsd:dateTime` literals with
different timezone offsets may denote the same instant.

A tag is computed over a term, not a value. This document therefore defines the relation
narrowly and says so plainly: two tags computed under the same key are equal #strong[if and
only if] the two plaintext literals are the #strong[same RDF term] — identical lexical form,
identical datatype IRI, and language tags equal under RDF's case-insensitive comparison.
Concretely:

- Lexical forms and datatype IRIs are fed to the MAC as the #strong[UTF-8 octets of their exact
  code-point sequences], under the term-identifier rule of §4.2.5, with #strong[no Unicode
  normalisation of any kind]. This is not a stylistic preference. An RDF literal's lexical form
  is a Unicode string whose code points participate in term identity, so a composed and a
  decomposed spelling of the same character are #emph[distinct] RDF terms, as are two datatype
  IRIs that differ only in normalisation or percent-encoding. Normalising before the MAC would
  give two distinct terms one tag, and a server-side same-term rewrite would then return a
  #strong[false-positive join] — the silent-wrong-answer failure of §5.3 in a third form, and
  the reason this document specifies no normalisation here rather than NFC.
- The #emph[only] transformation applied is that language tags are lowercased before the MAC,
  because RDF itself compares language tags case-insensitively; that rule is RDF's, not this
  document's. Nothing else is case-folded, and nothing else is transformed.
- #strong[No value canonicalisation is performed]: numeric lexical variants, `xsd:dateTime`
  timezone offsets, and `xsd:boolean` `"1"` versus `"true"` are #emph[not] reconciled.
  Ill-typed and invalid lexical forms are tagged as the literals they are, with no validation.
- Consequently a tag join is #strong[strictly weaker] than SPARQL value equality: it can miss
  solutions that `FILTER(?x = ?y)` would return. A rewriter #strong[MUST NOT] present a
  server-side tag join or tag-rewritten `FILTER` as value equality, and #strong[MUST] either
  restrict the rewrite to same-term equality or complete the comparison on the client (§5.4).
  This is the silent-wrong-answer hazard of §5.3 in its second form, and the same rule
  applies: it is better to refuse the rewrite than to return a quietly incomplete answer.

The tag is `HMAC-SHA-256` #cite("RFC2104") under the per-family tag key, over a canonical,
length-delimited input, published as the base64url encoding (§4.2.5) of the full 32-octet
output. Tags #strong[MUST NOT] be truncated: a truncated tag raises the collision probability,
and a tag collision is a #emph[wrong join result], not merely a lost one.

```text
TagInput = "sparq-e2ee/v1/eqtag" || 0x00
        || u8 tag format version (0x01)
        || u8 term_kind (0x01 = literal)
        || u16 length of key_family_id || key_family_id
        || u16 length of predicate_id  || predicate_id
        || u16 length of datatype_iri  || datatype_iri
        || u8  length of lang          || lang
        || u32 length of lexical_form  || lexical_form
```

The leading label, format version, `key_family_id`, and `predicate_id` provide #strong[domain
separation]. They are not decoration: without the predicate and family in the MAC input, one
tag key reused across predicates would let the operator test whether a value under
`schema:name` equals a value under a health predicate, correlating across contexts that the
key-family design (§5.2) exists to keep separate. Length prefixes prevent the corresponding
encoding attack, in which a crafted lexical form imitates the boundary between two fields so
that two distinct terms produce one MAC input. `predicate_id` is encoded by the
term-identifier rule of §4.2.5, identically to §5.1's, so that a value's tag and its associated
data agree on the predicate's octets.

A conforming implementation #strong[MUST] publish conformance test vectors for its tag
derivation covering, at minimum: two term-equal literals (equal tags); two value-equal but
term-distinct literals such as `"1"^^xsd:integer` and `"01"^^xsd:integer` (#emph[unequal]
tags); two lexical forms that are composed (NFC) and decomposed (NFD) spellings of the same
character, which are distinct RDF terms (#emph[unequal] tags); the same lexical form under two
datatype IRIs differing only by Unicode normalisation or percent-encoding spelling
(#emph[unequal] tags); the same lexical form under two different datatypes; the same lexical
form under two different language tags, and the same lexical form under two language tags
differing only in case (#emph[equal] tags); the same term under two different predicates in one
key family (unequal tags); and a lexical form containing the octets of the field delimiters.

== Mandatory leakage disclosure

This section is a #strong[conformance requirement], not advisory text. An implementation of
Profile SE #strong[MUST] present, in its conformance documentation #strong[and] to the data
owner before Profile SE is enabled for any resource, a statement that conveys at least the
following:

#note[
  Under this profile the storage operator can see the complete structure of your graph: every
  subject, every predicate, every link between resources, how many statements there are, and
  when they change. Only the values are encrypted. Because predicates name what each value
  #emph[is], the operator can see that you have, for example, a diagnosis, a location history,
  or a salary — it cannot read them. If equality tags are enabled, the operator can also see
  which values are equal to which, and how often each value occurs.
]

A conforming implementation:

- #strong[MUST NOT] present Profile SE as end-to-end encryption without this disclosure;
- #strong[MUST NOT] relegate the disclosure to a footnote, a linked appendix, or a
  default-collapsed interface element;
- #strong[MUST] state the disclosure in terms of what the operator #emph[can see], not in
  terms of which cryptographic primitive is used.

The reason this is normative rather than advisory is in §6.3: graph structure is identifying
on its own. Profile SE protects the values in a life, not the shape of it.

= Security considerations

This section is the heart of this document. A reader who reads only one section should read
this one.

== The impossibility statement, restated

There is no known construction for general server-side SPARQL evaluation over
end-to-end-encrypted data without leakage. §1.2 states it normatively; §7 records the option
space and why each excluded option is excluded. Profile CS achieves full expressiveness by
moving evaluation to the client and pays in sync cost. Profile SE achieves server-side
structural evaluation by not encrypting structure and pays in T1 disclosure. There is no third
thing in this document, because there is no third thing that is specifiable today.

== Per-profile leakage

#table(
  columns: 4,
  align: (left, left, left, left),
  table.header[][Profile CS (selective fetch)][Profile CS (full-replica sync)][Profile SE],
  [Declared tier], [T3], [T4], [T1],
  [Triple contents], [Not disclosed], [Not disclosed], [Values sealed; #strong[structure
    fully disclosed]],
  [Query text and plan], [Never leaves the client], [Never leaves the client],
    [Structural part is #strong[sent to the server]],
  [Answers], [Never leave the client], [Never leave the client], [Structural bindings
    disclosed; sealed values returned as ciphertext],
  [Resource-level access pattern], [Disclosed, and correlated with queries (which ciphertexts,
    when, together)], [Disclosed but #strong[not] query-correlated: resource identities,
    cardinality, bucketed sizes, and per-resource update timing all remain visible],
    [Not the relevant channel — the server sees the query directly],
  [Sizes], [Bucketed by the padding ladder (§4.2.3)], [Bucketed], [Value-ciphertext lengths
    disclosed unless padded],
  [Container structure], [Cardinality and timing disclosed (§4.6)], [Cardinality and timing
    disclosed], [Disclosed],
  [Value-equality pattern], [Not disclosed], [Not disclosed], [Disclosed #strong[only] if
    equality tags are enabled (§5.5)],
)

== Structure is identifying

The single most under-appreciated point in this document. Under Profile SE the operator holds
a labelled, ontology-typed graph of a person's life with the leaves blanked out. Three
consequences deserve stating:

- #strong[Predicates announce what the ciphertext is.] A sealed object of
  `schema:birthDate` is a date; of a health-vocabulary predicate, a diagnosis. The value is
  sealed; its nature is not.
- #strong[Topology alone re-identifies.] De-anonymisation from graph structure without any
  attribute data is a classical result #cite("NS09"), and RDF hands the adversary more than
  the social graphs in that work did: typed, labelled edges drawn from published vocabularies.
- #strong[Vocabularies are a strong prior.] Predicates and classes come from public
  ontologies, so an adversary's uncertainty about the #emph[schema] is near zero. This is also
  why the structured-encryption family (§7.1) is a worse fit for RDF than for text corpora:
  token-frequency analysis starts from a published distribution.

Profile CS does not have this exposure: under it, structure is inside the ciphertext.

== Key-compromise impact

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Compromised key][Immediate exposure][Containment],
  [One resource DEK], [That resource's plaintext, in every version sealed under that DEK.],
    [Per-resource DEKs (§4.3) bound this to one resource. Rotation (§4.5) protects subsequent
     versions only.],
  [A recipient's private key], [Every DEK ever wrapped to that recipient, hence every resource
    ever shared with them, including versions from before the compromise if the ciphertexts
    are retained.], [None retroactively. This is the dominant risk in the profile; rotating
     the recipient's key pair (§4.5) protects future wraps only.],
  [A Profile SE predicate key family], [Every value under those predicates, across every
    resource in the family's scope.], [Narrower families reduce blast radius at the cost of
     more keys.],
  [A Profile SE equality-tag key], [The ability to test any guessed value for equality against
    every tagged value — a brute-force oracle over the plaintext population, without
    decrypting anything.], [Tag keys MUST be distinct from value keys (§5.5) so this does not
     imply decryption; but for a low-entropy predicate, an equality oracle is close to
     decryption.],
  [The storage operator's own credentials], [Ciphertexts and all cleartext structure —
    everything in the relevant tier row of §6.2, and nothing more.], [This is the threat the
     profiles exist to bound, and the bound holds.],
)

Two properties this document does #strong[not] provide, stated so no one assumes them:
#strong[forward secrecy] (a compromised recipient private key exposes past shares) and
#strong[post-compromise
security] (recovery after compromise without re-keying out of band). Both would require a
ratcheting group key agreement; see §9.

== What a malicious server can still do

Encryption bounds what the server can #emph[read]. It does not make the server trustworthy:

- #strong[Omission.] The server can withhold resources, return an empty container, or serve a
  subset. The client then computes a correct answer over an incomplete corpus, and cannot
  distinguish that from a genuinely smaller dataset. #strong[No profile in this document
  provides answer completeness against a malicious server.] An encrypted manifest (§4.6) turns
  omission of a #emph[listed] resource into a detectable fetch failure, which is why §4.6
  requires the manifest — but a client cannot detect the omission of a resource it was never
  told exists.
- #strong[Rollback.] The server can serve a genuine older version. §4.2.4's version binding
  plus client-remembered state detects this; without it, it is undetectable.
- #strong[Tampering.] Detected: AEAD authentication fails, and the associated-data binding
  additionally defeats relocation (§4.2.4) and, under Profile SE, cross-triple value swapping
  (§5.1).
- #strong[Structure forgery under Profile SE.] Structure is cleartext and therefore
  #strong[unauthenticated by this document]: a Profile SE server can add, remove, or rewire
  triples whose objects are IRIs, and the client has no way to tell. Deployments needing
  structural integrity must sign the graph by some means outside this document; that is not
  defined in this document and is tracked in §9 alongside bead sq-tag1q.5.

== Traffic analysis and metadata

Beyond the tier tables: request timing correlates with user activity; a distinctive fetch
sequence after a login is a behavioural fingerprint; and co-access of the same resources by
two WebIDs discloses a relationship between them even when nothing about the content is
disclosed. Full-replica sync on a schedule (§4.7) addresses the first two, by decoupling fetch
timing and fetch order from user activity, which is why it is RECOMMENDED. It does #strong[not]
address the third: two WebIDs authorised for the same resource both replicate it, so co-access
— and the relationship it discloses — survives full-replica sync. Padding (§4.2.3) addresses
sizes only, and only to the granularity of the ladder's buckets.

== No claim of server-side SPARQL over encrypted data

To close where this section opened: nothing in this document should be read as providing
server-side SPARQL evaluation over end-to-end-encrypted data. Profile CS evaluates on the
client. Profile SE evaluates the structural fragment over #emph[cleartext structure] — the
values it evaluates around are sealed, but the query the server runs is a query over
plaintext structure. An implementation #strong[MUST NOT] describe Profile SE as querying
encrypted data.

== Audit posture of the referenced estate

Annex A is informative. The sparq zero-knowledge and multi-party-computation crates it
references are research-grade and internally re-audited, with #strong[no external
accredited-cryptographer sign-off]; the external-audit gate #strong[sq-qhy4] is #strong[open
and pending], and the MPC layer is honest-majority #emph[semi-honest] only. No conformance
requirement in this document depends on them, and this document makes no soundness claim about
them. Deployments #strong[MUST NOT] represent anything in Annex A as an audited guarantee.

= Considered and excluded

Recorded with the evidence, so that re-litigation starts from it. This section is informative
as to reasoning and normative in its exclusions.

== Searchable and structured encryption — deferred, with criteria

Structured encryption #cite("CK10") encrypts a data structure together with per-query tokens;
graph-shaped instantiations exist for neighbour, shortest-distance #cite("MKNK15"), and
shortest-path #cite("GKT21") queries, and relational cousins cover selection and equality-join
fragments #cite("FVYSHGKMC17"). It is real cryptography of the wrong shape for this document:
it answers lookup- and equality-class queries over structures pre-built per query family, so
each new query shape is a new bespoke index with its own leakage, and nothing resembling
general BGP joins, the `FILTER` algebra, paths, or aggregation is on offer. Its leakage is T2
(search, access, and volume patterns), which the leakage-abuse literature shows to be
exploitable under realistic auxiliary knowledge #cite("IKK12"), #cite("CGPR15"),
#cite("ZKP16") — and RDF's low-entropy, publicly enumerable vocabularies make the prior
unusually strong. The field is also still moving: the shortest-path scheme of #cite("GKT21")
drew a query-recovery attack #cite("FP22") and a repaired successor #cite("FPSO24") within
three years, which is a caution against freezing any particular scheme into a specification.

#strong[Re-entry criteria.] This document should be revisited when an encrypted-graph scheme
exists with (i) a stable query API that maps onto a recognisable SPARQL fragment, (ii) a
peer-reviewed leakage analysis, and (iii) an interoperable wire format. Until then, the
client-maintained encrypted index sidecar of §4.7 delivers much of the practical benefit with
client-controlled leakage.

== Deterministic and order-revealing encryption — rejected

Deterministic encryption makes equal plaintexts produce equal ciphertexts; order-revealing
encryption #cite("BCLO09"), #cite("BLRSZZ15") additionally exposes order, which would restore
range `FILTER` and `ORDER BY` server-side. The CryptDB lineage #cite("PRZB11") made this
pattern briefly plausible. The attack record closed it: frequency analysis against public
auxiliary data recovered property-preserving-encrypted columns wholesale #cite("NKW15"),
leakage-abuse extends to even idealised order-revealing encryption #cite("GSBNR17"), and
reconstruction attacks recover plaintexts from range-query access patterns alone
#cite("KKNO16"). Deployed systems built on these primitives were broken along the same seams
#cite("GMNRS16"), #cite("GRS17").

Personal RDF is the worst case for these attacks: names, dates, places, and diagnoses are
exactly the low-entropy, public-prior populations they feast on. Order leakage on a date
predicate is a timeline; equality leakage on a location predicate is a movement profile. This
document therefore specifies #strong[no] order-revealing construction, and admits only the
narrowly scoped, separately disclosed equality tag of §5.5, which carries its own normative
warning. A client needing range `FILTER` over protected values uses Profile CS.

== Fully homomorphic encryption — rejected as impractical

Fully homomorphic encryption would let a server evaluate over ciphertext with no leakage
beyond volume — asymptotically the "right" answer. It is not a profile for three reasons.
#strong[Cost]: after a decade of engineering, general computation under FHE remains orders of
magnitude slower than plaintext #cite("VJH21"), which composes badly with SPARQL's join
complexity. #strong[Shape]: FHE circuits are data-oblivious by construction, so evaluation
must touch the whole dataset — no indexes, no selectivity — which is why private-database-query
research retreats to private-information-retrieval-class functionality rather than a general
query algebra. #strong[Scope]: it addresses confidentiality in computation and leaves key
management, sharing, update, and multi-writer semantics exactly as Profile CS must solve them
anyway.

== Server-side decryption in any form — rejected

A "decrypt" extension function, a key-holding query service, and a trusted-execution-environment
binding all place a decryption capability on the server. Whatever their engineering merits,
they forfeit the property this document defines (§1.1), and are excluded from it by
construction rather than by evidence.

= Annex A (informative): verifiable answers over committed data

#note[
  #strong[This annex is informative.] It contains no conformance requirement. Nothing in this
  document normatively depends on the mechanisms sketched here, and no soundness or
  confidentiality property is claimed for them. The crates referenced are research-grade,
  internally re-audited, with #strong[no external accredited-cryptographer sign-off]: the
  external-audit gate #strong[sq-qhy4] is #strong[open and pending], and the MPC layer is
  honest-majority #emph[semi-honest] only.
]

Zero-knowledge proofs over committed graphs and secure multi-party computation are frequently
proposed in the same breath as E2EE, so it is worth stating precisely what they are and are
not, in the vocabulary of this document.

#strong[Neither is E2EE storage.] A commitment is not encryption: it hides nothing from
whoever holds the data, because the prover #emph[is] the data holder. MPC's confidentiality is
in-computation: each participant's inputs stay with their owner, in the clear, and are
protected only during a joint evaluation. In neither case does the server-holds-only-ciphertext
property of §1.1 appear.

What they could add to a deployment of this document is orthogonal and genuinely useful:

- #strong[Answer verifiability.] A Profile CS client — or a third party who cannot see the
  data at all — could check that an answer is consistent with a published commitment to the
  graph, an idea explored for SPARQL in #cite("BWK26") and #cite("WRIGHT-DC25") and for SQL in
  #cite("ZKSQL23"). This addresses the completeness gap of §6.5, which encryption alone does
  not close. It is not specified here, and the fragment such proofs cover is small.
- #strong[Cross-pod joins without disclosure.] Two E2EE pods could answer a joint query via
  MPC after each owner decrypts locally, without either re-uploading plaintext to the other;
  collaborative proving over distributed secrets #cite("OB22") is the corresponding multi-prover
  direction.

Both are recorded here as directions, not as mechanisms this document defines.

= Open issues and future work

- #strong[Envelope algorithm-registry governance] (§4.2.1) — how identifiers are added,
  deprecated, and retired. Tracked under bead sq-tag1q.5 and the specification-series process
  bead sq-rvgr2.
- #strong[The normative reference for `XChaCha20-Poly1305`] (§4.2.1) — the construction is
  specified only by an #emph[expired] Internet-Draft #cite("XCHACHA"), because no stable RFC
  defines it. This is a known weakness in the reference chain of a MUST-implement suite: the
  reference should be re-pointed if the CFRG publishes a stable document, and until then
  implementers should treat #cite("XCHACHA") as the sole interoperability authority for suite
  `0x0001`. Tracked under bead sq-tag1q.5.
- #strong[Recipient key discovery in Solid] — precisely where a WebID's encryption keys live
  and how key discovery interacts with pod session handling; to be coordinated with the
  external Solid specification programme (bead sq-tag1q.8).
- #strong[Forward secrecy and post-compromise security] (§6.4) — a ratcheting group key
  agreement in place of the static key envelope of §4.4. This is the most significant
  cryptographic gap in Profile CS.
- #strong[Structural integrity under Profile SE] (§6.5) — cleartext structure is
  unauthenticated; a signed-graph binding is not defined in this document and needs its own
  design record before specification. Tracked under bead sq-tag1q.5.
- #strong[Encrypted index sidecars] (§4.7) — the intermediate point between full-replica sync
  and structured encryption, with client-controlled shard-access leakage.
- #strong[Per-recipient equality tags] (§5.5) — unlinkable across audiences, at the cost of
  tag-set growth.
- #strong[Interaction with replicated datasets] — an E2EE pod that is also a conflict-free
  replicated replica multiplies both designs' constraints; the interaction is flagged now and
  designed later, against the companion SPARQL-CRDT draft (bead sq-tag1q.4).
- #strong[Access-pattern hardening beyond padding] — bucketised fetch, or private information
  retrieval for index shards. Strictly future work; the profiles are not gated on it.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("RFC8439", [Nir, Y., Langley, A. #emph[ChaCha20 and Poly1305 for IETF Protocols].
    RFC 8439, IETF, June 2018. Defines ChaCha20-Poly1305 with a 96-bit nonce; it does
    #strong[not] define XChaCha20-Poly1305 — see #cite("XCHACHA").]),
  ("XCHACHA", [Arciszewski, S. #emph[XChaCha: eXtended-nonce ChaCha and]
    `AEAD_XChaCha20_Poly1305`. draft-irtf-cfrg-xchacha-03, IRTF CFRG, January 2020. The
    normative definition of the
    HChaCha20 subkey derivation and the 192-bit-nonce AEAD registered as suite `0x0001` in
    §4.2.1. An expired Internet-Draft, cited because no stable RFC defines this
    construction; §9 tracks re-pointing this reference if one is published.]),
  ("RFC9180", [Barnes, R., Bhargavan, K., Lipp, B., Wood, C. A. #emph[Hybrid Public Key
    Encryption]. RFC 9180, IRTF CFRG, February 2022.]),
  ("RFC4648", [Josefsson, S. #emph[The Base16, Base32, and Base64 Data Encodings]. RFC 4648,
    IETF, October 2006.]),
  ("RFC2104", [Krawczyk, H., Bellare, M., Canetti, R. #emph[HMAC: Keyed-Hashing for Message
    Authentication]. RFC 2104, IETF, February 1997.]),
  ("GCM", [Dworkin, M. #emph[Recommendation for Block Cipher Modes of Operation:
    Galois/Counter Mode (GCM) and GMAC]. NIST Special Publication 800-38D, 2007.]),
  ("SPARQL11", [Harris, S., Seaborne, A. #emph[SPARQL 1.1 Query Language]. W3C
    Recommendation, 21 March 2013.]),
  ("SOLID", [Capadisli, S., et al. #emph[Solid Protocol]. W3C Solid Community Group,
    https://solidproject.org/TR/protocol.]),
  ("BCLO09", [Boldyreva, A., Chenette, N., Lee, Y., O'Neill, A. #emph[Order-Preserving
    Symmetric Encryption]. EUROCRYPT 2009, LNCS 5479.]),
  ("BLRSZZ15", [Boneh, D., Lewi, K., Raykova, M., Sahai, A., Zhandry, M., Zimmerman, J.
    #emph[Semantically Secure Order-Revealing Encryption]. EUROCRYPT 2015, Part II,
    pp. 563–594.]),
  ("BWK26", [Braun, C., Wright, J., Käfer, T. #emph[Proving Soundness of SPARQL Query Results
    Using Selective Disclosure of RDF Datasets and Zero-Knowledge Proofs]. The Semantic Web
    (ESWC 2026), Springer, pp. 297–318.]),
  ("CGPR15", [Cash, D., Grubbs, P., Perry, J., Ristenpart, T. #emph[Leakage-Abuse Attacks
    Against Searchable Encryption]. ACM CCS 2015, pp. 668–679.]),
  ("CK10", [Chase, M., Kamara, S. #emph[Structured Encryption and Controlled Disclosure].
    ASIACRYPT 2010, LNCS 6477, pp. 577–594.]),
  ("FKPS17", [Fernández, J. D., Kirrane, S., Polleres, A., Steyskal, S. #emph[Self-Enforcing
    Access Control for Encrypted RDF]. ESWC 2017, LNCS 10249, pp. 607–622.]),
  ("FKPS20", [Fernández, J. D., Kirrane, S., Polleres, A., Steyskal, S. #emph[HDTcrypt:
    Compression and encryption of RDF datasets]. Semantic Web 11(2):337–359, 2020.]),
  ("FP22", [Falzon, F., Paterson, K. G. #emph[An Efficient Query Recovery Attack Against a
    Graph Encryption Scheme]. ESORICS 2022.]),
  ("FPSO24", [Falzon, F., Paterson, K. G., et al. #emph[PathGES: An Efficient and Secure Graph
    Encryption Scheme for Shortest Path Queries]. ACM CCS 2024.]),
  ("FVYSHGKMC17", [Fuller, B., Varia, M., Yerukhimovich, A., et al. #emph[SoK:
    Cryptographically Protected Database Search]. IEEE S&P 2017, pp. 172–191.]),
  ("Gie05", [Giereth, M. #emph[On Partial Encryption of RDF-Graphs]. ISWC 2005.]),
  ("GKT21", [Ghosh, E., Kamara, S., Tamassia, R. #emph[Efficient Graph Encryption Scheme for
    Shortest Path Queries]. ACM ASIA CCS 2021.]),
  ("GMNRS16", [Grubbs, P., McPherson, R., Naveed, M., Ristenpart, T., Shmatikov, V.
    #emph[Breaking Web Applications Built On Top of Encrypted Data]. ACM CCS 2016.]),
  ("GRS17", [Grubbs, P., Ristenpart, T., Shmatikov, V. #emph[Why Your Encrypted Database Is
    Not Secure]. HotOS 2017.]),
  ("GSBNR17", [Grubbs, P., Sekniqi, K., Bindschaedler, V., Naveed, M., Ristenpart, T.
    #emph[Leakage-Abuse Attacks against Order-Revealing Encryption]. IEEE S&P 2017.]),
  ("IKK12", [Islam, M. S., Kuzu, M., Kantarcioglu, M. #emph[Access Pattern Disclosure on
    Searchable Encryption: Ramification, Attack and Mitigation]. NDSS 2012.]),
  ("KKNO16", [Kellaris, G., Kollios, G., Nissim, K., O'Neill, A. #emph[Generic Attacks on
    Secure Outsourced Databases]. ACM CCS 2016.]),
  ("MKNK15", [Meng, X., Kamara, S., Nissim, K., Kollios, G. #emph[GRECS: Graph Encryption for
    Approximate Shortest Distance Queries]. ACM CCS 2015, pp. 504–517.]),
  ("NKW15", [Naveed, M., Kamara, S., Wright, C. V. #emph[Inference Attacks on
    Property-Preserving Encrypted Databases]. ACM CCS 2015, pp. 644–655.]),
  ("NS09", [Narayanan, A., Shmatikov, V. #emph[De-anonymizing Social Networks]. IEEE S&P
    2009.]),
  ("OB22", [Ozdemir, A., Boneh, D. #emph[Experimenting with Collaborative zk-SNARKs:
    Zero-Knowledge Proofs for Distributed Secrets]. USENIX Security 2022, pp. 4291–4308.]),
  ("PRZB11", [Popa, R. A., Redfield, C. M. S., Zeldovich, N., Balakrishnan, H. #emph[CryptDB:
    Protecting Confidentiality with Encrypted Query Processing]. SOSP 2011, pp. 85–100.]),
  ("PSVBZ14", [Popa, R. A., Stark, E., Valdez, S., et al. #emph[Building Web Applications on
    Top of Encrypted Data Using Mylar]. NSDI 2014.]),
  ("TV23", [Taelman, R., Verborgh, R. #emph[Link Traversal Query Processing over Decentralized
    Environments with Structural Assumptions]. ISWC 2023, LNCS 14265.]),
  ("VJH21", [Viand, A., Jattke, P., Hithnawi, A. #emph[SoK: Fully Homomorphic Encryption
    Compilers]. IEEE S&P 2021.]),
  ("WRIGHT-DC25", [Wright, J. #emph[Towards Provable Provenance and Privacy-Preserving Queries  // privacy-claims-allow: prior-work reference title (Wright, ISWC 2025 DC), not a sparq claim
    in Decentralised Data Architectures]. ISWC 2025 Companion (Doctoral Consortium), CEUR-WS
    Vol-4085, paper 19.]),
  ("ZKP16", [Zhang, Y., Katz, J., Papamanthou, C. #emph[All Your Queries Are Belong to Us: The
    Power of File-Injection Attacks on Searchable Encryption]. USENIX Security 2016,
    pp. 707–720.]),
  ("ZKSQL23", [Li, X., et al. #emph[ZKSQL: Verifiable and Efficient Query Evaluation with
    Zero-Knowledge Proofs]. PVLDB 2023.]),
))
