// [OPUS-5] sq-tag1q.2 — Solid Vector Search: an OPTIONAL vector-store add-on to the
// Access-Controlled SPARQL Query over a Solid Pod specification. Unofficial Proposal Draft.
//
// EXTEND, DON'T FORK. This document is an add-on to the maintainer's Solid-SPARQL
// specification, not a replacement or a competing design. The base draft was read first,
// in the copy vendored in this repository at
// `crates/sparq-solid/tests/conformance/solid-sparql-query/spec.html` (pinned upstream
// commit `5ea9718…`, see that directory's PROVENANCE.md). Every authorization,
// dataset-construction, and non-disclosure rule of the base document is INHERITED
// UNCHANGED and re-applied to the vector surface. Exactly ONE departure is declared, not
// smuggled: SV-ND-4 accepts that a requester holding an eligible entry observes that some
// index is bound — and, by sweeping literal-vector dimensions, at what dimension — even when
// the descriptor is unreadable to them, so an unreadable descriptor and an absent one are
// not fully observationally equivalent. Both observables are excepted at the assertion
// itself, so no MUST NOT of this draft forbids behaviour the draft elsewhere prescribes; it
// is stated at the assertion, in the relationship section, and in the open-issues list.
// The base document mints its terms in `http://www.w3.org/ns/solid/sparql#`, NOT in the
// namespace named in the originating bead — see the editor's note in the namespace section.
//
// NO SECOND VOCABULARY. The query surface is the `vec:` magic-predicate surface already
// specified in the sibling sparq draft "SPARQL Vector & GenAI Extension" (bead sq-rvgr2.4,
// site/specs/sparql-vector-genai.typ). This document defines NO new query syntax and NO
// competing similarity vocabulary; it binds the existing one to the Solid authorized
// dataset and adds only the pod-side resource-management terms the base drafts lack.
//
// GROUNDING. Statements about the sparq estate are grounded in `crates/sparq-vectors`
// (the `.spqv` one-vector-per-term store, the exact-vs-approximate split, the
// provider-agnostic `Embedder` trait, the `IdMask` filtered path, the staleness
// fingerprint), `crates/sparq-solid` (the WAC/ACP authorized-view materialiser), and
// `research/feature-research-vector-genai.md`. The two crates do NOT depend on each other
// in this workspace, so the Solid-plus-vector seam this document specifies is GREENFIELD
// in sparq; the implementation-status section says so plainly rather than implying wiring
// that does not exist.
//
// HONESTY. No measured recall, latency, or throughput figure appears anywhere in this
// source: every measurement in the underlying estate is work-box, non-canonical, and the
// spec factory's build-boundary honesty scan enforces that over this file. Approximate
// retrieval is described as recall-below-one throughout, and the interaction between
// approximation and the base document's non-disclosure invariants is stated as a real,
// costly restriction rather than waved away.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "Solid Vector Search: An Optional Vector-Store Extension to Access-Controlled SPARQL Query over a Solid Pod")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

// A "Proposal" aside: a design offered for a FUTURE revision, outside this draft's
// normative surface. Mirrors the sibling vector/GenAI draft's helper so the two read alike.
#let proposal(body) = context {
  if target() == "html" {
    html.elem("aside", attrs: (class: "note proposal"))[
      #html.elem("span", attrs: (class: "note-title"))[Proposal]
      #body
    ]
  } else {
    block(width: 100%, inset: 8pt, radius: 3pt, stroke: (left: 3pt + rgb("#c80")), fill: rgb("#fdf8ee"))[
      #strong[Proposal] — #body
    ]
  }
}

// An "Editor's note" aside: a detail this draft deliberately does not pin.
#let ednote(body) = context {
  if target() == "html" {
    html.elem("aside", attrs: (class: "note ednote"))[
      #html.elem("span", attrs: (class: "note-title"))[Editor's note]
      #body
    ]
  } else {
    block(width: 100%, inset: 8pt, radius: 3pt, stroke: (left: 3pt + rgb("#88a")), fill: rgb("#f4f5fb"))[
      #strong[Editor's note] — #body
    ]
  }
}

// A testable-assertion identifier, rendered at the assertion site and anchored in HTML.
#let rid(id) = context {
  let label-text = "[" + id + "]"
  if target() == "html" {
    html.elem("span", attrs: (class: "req-id", id: "req-" + lower(id)))[#label-text]
  } else {
    text(size: 8pt, weight: "bold", fill: rgb("#555"))[#label-text]
  }
}

#spec-head()

#intro-section("abstract", "Abstract")[
  This document specifies an #strong[optional] vector-search add-on to
  #emph[Access-Controlled SPARQL Query over a Solid Pod] #cite("SOLID-SPARQL-QUERY"). It
  defines a vector index as an ordinary resource of a Solid storage — created, described,
  and deleted through the Solid Protocol itself, never through the query endpoint, so the
  base document's read-only query service is untouched; an ingestion contract in which
  embeddings are produced #strong[outside] the server by a provider of the storage owner's
  choosing; and a similarity query surface that reuses, without extension, the `vec:`
  magic-predicate patterns of the sibling #emph[SPARQL Vector and GenAI Extension]
  #cite("SPARQL-VEC-GENAI"). An exact scan is the #strong[mandatory] baseline; approximate
  nearest-neighbour acceleration is optional and carries a recall-below-one disclosure
  obligation. The central contribution is the authorization analysis: a top-#emph[k]
  operator is not a graph pattern, and post-filtering a globally-ranked result set both
  under-fills #emph[k] and turns result cardinality into a function of content the
  requester cannot read. This document therefore requires that candidate eligibility be
  decided in full before ranking, with readability settled by the same GET-equivalence rule the
  base document uses, that an entry be searched only while its indexed term still occurs in
  its own source resource, and that the returned set never depend on an entry the requester
  cannot read. That last requirement is shown to constrain approximate backends sharply —
  it excludes the shared proximity graph, whose very topology is built from content outside
  the requester's view — and the cost is recorded rather than hidden. Conformance is claimed
  in #strong[distinct optional classes] for the query service, the storage server, and the
  ingestion agent: a Solid SPARQL query server that implements none of this remains fully
  conformant to #cite("SOLID-SPARQL-QUERY"). Every normative assertion carries a testable
  identifier.
]

#sotd()

= Introduction <sec-intro>

A Solid storage — a #dfn[pod] — holds a person's data as ordinary web resources under that
person's control. #cite("SOLID-SPARQL-QUERY") makes such a storage queryable: it defines a
SPARQL 1.1 query service scoped to exactly one storage, maps each RDF source to a named
graph, and derives the queryable dataset per request from the storage's own access control
system, so that a requester can query exactly what that requester could read.

Semantic retrieval — "what in my pod is #emph[about] this?" — is the obvious next ask, and
it is one of the few capabilities a personal data store genuinely needs: a pod is small
enough that its owner cannot remember what is in it, and heterogeneous enough that
keyword search under-serves it. Dense-vector retrieval answers that question, but it does
not compose with an access-controlled dataset for free. Ranking is a global operator over a
candidate set; the base document's entire architecture rests on the candidate set being
fixed, before evaluation, to what the requester may read. Getting that composition wrong
does not merely return the wrong answers — it reopens exactly the disclosure oracles
#cite("SOLID-SPARQL-QUERY") closes.

This document therefore does three things and deliberately no more:

+ It makes a vector index a #strong[resource of the storage], so its lifecycle and its
  access control are the pod's own, not a new subsystem's (@sec-index).
+ It states an #strong[ingestion contract] in which the embedding model lives outside the
  server, so a pod's contents are never shipped to a third party as an invisible side
  effect of enabling search (@sec-ingest).
+ It re-derives the base document's authorization and non-disclosure invariants for a
  ranked, top-#emph[k] surface, and states plainly which vector-search implementation
  techniques those invariants exclude (@sec-authz).

== Relationship to the base specification <sec-relationship>

This document is an #strong[add-on], not a profile, a fork, or a revision. It adds
requirements; it removes and relaxes none, with one departure it declares rather than
argues away: SV-ND-4 accepts that a requester holding an eligible entry can observe that
#emph[some] index is bound — and, by sweeping literal-vector dimensions, at what dimension —
even where the index descriptor is a resource they cannot read, so an unreadable descriptor
and an absent one are not fully observationally equivalent (@sec-nondisclosure). Where a
requirement of #cite("SOLID-SPARQL-QUERY") and a requirement of this document both apply,
both must be satisfied. The following are inherited unchanged. Most are simply cited where they are
needed; a few are #emph[re-derived] for a ranked surface, where the base rule's structural
argument does not carry over unmodified and a new assertion is required to say what it
becomes. This document does not restate a base rule verbatim under a new identifier; where
a requirement is unchanged, it is cited, not copied.

#table(
  columns: 2,
  align: (left, left),
  table.header[Inherited from #cite("SOLID-SPARQL-QUERY")][Consequence for this add-on],
  [The authorized dataset and GET-equivalence], [Vector candidate eligibility is decided by
    the same rule, on the same resources (@sec-authz).],
  [The empty standing default graph], [A vector pattern in a top-level basic graph pattern
    searches nothing until the requester supplies a dataset description or opts into union
    default graph mode (@sec-scope).],
  [Existence non-disclosure and its invariants], [Restated for ranking, cardinality, and
    approximation in @sec-nondisclosure.],
  [One storage per query service], [An index is scoped to one storage; no cross-pod index
    is defined.],
  [SPARQL Update out of scope for the query service], [Index management is Solid Protocol
    traffic, not query traffic (@sec-index).],
  [Caching, content negotiation, protocol bindings], [Unchanged; vector results are ordinary
    SPARQL solutions in ordinary result formats.],
)

== Relationship to the SPARQL vector extension <sec-relationship-vec>

The similarity query surface is #strong[not defined here]. It is the `vec:nearest` and
`vec:search` magic-predicate surface of #cite("SPARQL-VEC-GENAI"), reused verbatim. This
document adds no predicate, no function, no `SERVICE` form, and no second similarity
vocabulary. Its contribution on that axis is purely to say what those patterns mean when
the dataset is a Solid authorized dataset rather than a single trusted store, and which of
their permitted execution modes survive that setting.

Two consequences are worth stating up front. First, the embedding-provenance record that
#cite("SPARQL-VEC-GENAI") requires of every vector store is #strong[reused as the index
descriptor's core content] (@sec-provenance) rather than re-invented. Second, the surface
form is inherited together with its open questions: #cite("SPARQL-VEC-GENAI") records that
a `SERVICE`-shaped and an extension-function-shaped surface were considered and not
specified, and that remains an open question owned by that document, not this one
(@sec-open).

== Non-goals <sec-nongoals>

- #strong[A new protocol.] Index management uses the Solid Protocol
  #cite("SOLID-PROTOCOL"); querying uses the SPARQL 1.1 Protocol
  #cite("SPARQL11-PROTOCOL"). No new endpoint, verb, or handshake is defined.
- #strong[A new authorization model.] Visibility is decided by the storage's existing
  access control system — WAC #cite("WAC") or ACP #cite("ACP") — exactly as in
  #cite("SOLID-SPARQL-QUERY").
- #strong[An embedding model, or any opinion about one.] The model is external, the
  provider is the storage owner's choice, and this document specifies only the contract
  the resulting vectors must satisfy.
- #strong[Cross-pod or federated similarity search.] A query service is scoped to one
  storage; searching across storages is left to clients, as in the base document.
- #strong[Answer quality.] Nothing here promises that retrieval is good, and no accuracy,
  recall, or throughput figure appears in this specification (@sec-accuracy).
- #strong[Making vector search mandatory.] A conforming Solid SPARQL query server may
  implement none of this (@sec-optionality).

= Terminology and conformance <sec-conformance>

== Requirement keywords

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED], #strong[MAY],
and #strong[OPTIONAL] in this document are to be interpreted as described in
#cite("RFC2119") and #cite("RFC8174") when, and only when, they appear in all capitals, as
shown here.

== Terms

This document uses the terms #emph[storage], #emph[RDF source], #emph[resource graph],
#emph[authorized dataset], #emph[requester], and #emph[readable] as defined in
#cite("SOLID-SPARQL-QUERY"), and #emph[vector store], #emph[answer-exact],
#emph[approximate], and #emph[embedding client] as defined in
#cite("SPARQL-VEC-GENAI"). It adds:

- A #dfn[vector index] is a persistent, named collection of index entries over the RDF
  sources of exactly one storage, described by an #dfn[index descriptor]: an RDF source of
  that storage whose IRI is the index's name.
- An #dfn[index entry] is a triple of a #strong[source resource] (the IRI of exactly one
  RDF source of the storage), an #strong[indexed term] (an IRI or a literal), and a
  #strong[vector] of the index's declared dimension. The source resource is part of the
  entry's identity, not metadata about it (@sec-entry).
- An index entry is #dfn[current] for a request if its indexed term occurs, at evaluation
  time, in its source resource's graph as that graph appears in the requester's authorized
  dataset (@sec-currency).
- The #dfn[eligible entry set] of a `vec:` pattern is the set of index entries that are
  #emph[current], whose source resource is readable by the requester, and whose source
  resource contributes a graph to the active graph of the pattern's evaluation
  (@sec-authz). Eligibility is thus a three-part test — currency, readability, active-graph
  membership — and all three are decided before ranking (SV-AUTH-3).
- The #dfn[readable entry set] of a request is the set of current entries whose source
  resource is readable by the requester, without regard to the active graph. It is a
  superset of the eligible entry set and is used only for seed lookup (@sec-seed).
- The #dfn[ranked set] of a `vec:` pattern is the intersection of its eligible entry set
  with the pattern-derived candidate mask that #cite("SPARQL-VEC-GENAI") requires when the
  neighbour variable is constrained by other triple patterns (@sec-masks). Where the
  neighbour variable is unconstrained that mask admits everything, and the ranked set is the
  eligible entry set. #strong[The ranked set, not the eligible entry set, is what a ranking
  orders and what the top-#emph[k] is taken from]; the eligible entry set is the
  authorization boundary, and the ranked set is what survives the query's own constraints
  inside it.
- A #dfn[vector-extended query service] is a query service conforming to
  #cite("SOLID-SPARQL-QUERY") that additionally satisfies every assertion of this document
  addressed to a query service.
- An #dfn[index-hosting storage server] is a Solid storage server that accepts ingestion
  writes for an index of its storage and satisfies every assertion of this document
  addressed to a storage server (@sec-ingest).
- An #dfn[ingestion agent] is any agent that writes index entries, acting under the
  storage's ordinary write authorization (@sec-ingest). It is typically a client of the
  pod owner's, not the server.

== Conformance classes and optionality <sec-optionality>

This document defines three conformance classes — #strong[vector-extended query service],
#strong[index-hosting storage server], and #strong[ingestion agent] — and #strong[every one
of them is OPTIONAL]. Conformance is claimed per class; an implementation MAY implement any
subset, but MUST satisfy every assertion addressed to each class it claims. The classes are
separable in practice: the query service reads an index, the storage server accepts writes
to it, and the ingestion agent produces its content, and these are commonly three different
pieces of software run by three different parties.

Two kinds of statement sit outside every class, and are marked as such where they appear:
obligations addressed to a #emph[deployment], a #emph[storage owner], or a #emph[consumer]
of results (SV-DESC-4, @sec-security), which no single component can be tested for, and the
client-directed advice of SV-DISC-5. They are stated normatively because they are real obligations on real parties,
not because a conformance suite can check them.

A Solid SPARQL query server that implements no part of this document remains fully
conformant to #cite("SOLID-SPARQL-QUERY"); no requirement of this document is a requirement
of that one. #rid("SV-OPT-1")

A server MUST NOT make the presence of a vector index a precondition for any behaviour
required by #cite("SOLID-SPARQL-QUERY"). In particular, binding or unbinding an index MUST
NOT change the solutions of any query that contains no `vec:` pattern, except through the
ordinary presence or absence of the index descriptor's own resource graph in the authorized
dataset — the descriptor is a resource like any other, and creating one adds a graph exactly
as creating any resource does. #rid("SV-OPT-2")

A vector-extended query service MUST satisfy every requirement of
#cite("SOLID-SPARQL-QUERY") addressed to servers, and every requirement of
#cite("SPARQL-VEC-GENAI") addressed to the vec-extended-processor and vector-store classes,
in addition to the requirements of this document. #rid("SV-OPT-3")

Where a requirement of #cite("SPARQL-VEC-GENAI") and a requirement of this document cannot
both be satisfied, this document's requirement governs, and the conflict MUST be resolved in
the direction that discloses less. Where a requirement of that document would oblige a
service to raise a query error that reveals the existence, absence, or declared parameters
of an index to a requester not otherwise entitled to learn them, the pattern MUST instead
yield zero solutions. #rid("SV-OPT-4")

#note[
  SV-OPT-4 is not a licence to ignore the sibling document; it resolves exactly one class of
  conflict, and in exactly one direction. The concrete case it settles is a literal query
  vector of the wrong dimension, which that document makes a hard query error: on a
  single-tenant store an error is the honest answer, but here the error boundary is a probe
  that reads back an index's dimension — and its existence — one guess at a time
  (@sec-nondisclosure). Zero solutions is the answer that keeps a bound index and no index
  indistinguishable to a requester with nothing to search — which is the whole of what it
  buys, and the whole of what SV-ND-4 claims.

  It does #strong[not] close the channel for anyone else, and this draft does not claim it
  does. A requester who holds at least one eligible entry learns that an index is bound the
  moment an ordinary `vec:` pattern returns a solution, with no probing at all, even where
  SV-DISC-3 withheld the descriptor. The same requester can go further and sweep dimensions,
  watching for the one that returns solutions, to recover the index's declared dimension as
  well as its existence. Closing either would mean refusing `vec:` patterns — or at least
  literal query vectors — from requesters not entitled to the descriptor, or padding the
  response; none is specified here, SV-ND-4 records the resulting boundary, and @sec-open
  records the gap.
]

#note[
  SV-OPT-2 constrains #emph[solutions] only. Whether an index's presence is observable in
  timing is SV-ND-3's SHOULD, which — like the corresponding requirement of the base
  document — this specification does not raise to a MUST.
]

== Testable assertions

Every normative assertion carries a stable identifier of the form `[SV-<group>-<n>]`,
rendered at the assertion site. Identifiers are never reused; a withdrawn assertion's
identifier is retired rather than reassigned. @sec-testable indexes the groups, states the
scenarios, and records the conformance-suite commitment.

The following are #strong[non-normative]: material marked #emph[Proposal] or
#emph[Editor's note]; every example and note; @sec-related; and @sec-impl-status.

= Related work <sec-related>

#emph[This section is informative.]

Vector retrieval inside a graph query language is established practice: GraphDB exposes
similarity indexes through special predicates, Neo4j through Cypher procedures, and
pgvector through SQL operators; #cite("SPARQL-VEC-GENAI") surveys that landscape and
explains why it adopts the pattern-level form. None of those systems is
access-control-scoped in the sense this document requires — their indexes are
store-global, and the tenancy features shipped by dedicated vector databases (per-collection
or per-tenant partitioning) attach access control to a partition, not to a per-document
authorization decision resolved per request.

The problem of ranking under a per-requester candidate mask is the #dfn[filtered ANN]
problem studied by Filtered-DiskANN #cite("FILTERED-DISKANN") and ACORN #cite("ACORN"),
and #cite("SPARQL-VEC-GENAI") already states the answer-safety contract any such backend
must meet. What this document adds is the observation that in a Solid setting the mask is
adversarially interesting: the requester knows the mask exactly (it is precisely the set of
resources they can read, which they can enumerate and fetch), so any deviation between the
service's answer and the answer the requester could compute themselves over that same set
is evidence about content outside it. That turns an accuracy question into a disclosure
question (@sec-modes).

The sensitivity of the index artifact itself is grounded in the embedding-inversion
literature: source text can be partially reconstructed from its embedding
#cite("EMB-INVERSION"), and membership inference against embedding stores is practical
#cite("EMB-LEAKAGE"). A vector derived from a private resource is therefore treated here as
derived content of that resource, not as opaque metadata about it.

= The vector index as a resource <sec-index>

== Indexes are storage resources

A vector index MUST be described by an index descriptor: an RDF source of the same storage,
whose IRI names the index. #rid("SV-IDX-1")

Making the descriptor an ordinary resource is the whole design. It gives the index, for
free and without a new mechanism: a name in the storage's URI space; a lifecycle governed
by the Solid Protocol #cite("SOLID-PROTOCOL"); access control governed by the storage's own
WAC or ACP documents; and — because the descriptor is an RDF source — a resource graph in
the authorized dataset, so a requester can query the description of the index with the same
query service that uses it.

== Create, describe, delete <sec-crud>

Index management is performed with the ordinary Solid Protocol operations on the descriptor
and on the index's entry payloads. A vector-extended query service MUST NOT expose index
creation, entry ingestion, index update, or index deletion through the query endpoint.
#rid("SV-IDX-2")

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Operation][Mechanism][Authorization],
  [Create], [`PUT`/`POST` the descriptor, per #cite("SOLID-PROTOCOL")], [Write access to the
    descriptor's location, per the storage's access control system],
  [Describe], [`GET` the descriptor, or query its resource graph], [Read access to the
    descriptor (@sec-provenance)],
  [Ingest entries], [Write the index's entry payload (@sec-ingest)], [Write access to the
    index, plus the obligations of @sec-ingest],
  [Delete], [`DELETE` the descriptor], [Write access to the descriptor],
)

SV-IDX-2 keeps the base document's read-only query service exactly read-only: the query
endpoint gains a retrieval capability and no mutation capability whatsoever. It also means
this add-on introduces no operation whose authorization is not already defined by the Solid
Protocol.

On deletion of an index descriptor, a vector-extended query service MUST cease to serve
that index's entries; a subsequent query MUST behave exactly as it would if the index had
never existed. #rid("SV-IDX-3")

== At most one bound index per query service <sec-binding>

A storage MAY hold any number of index descriptors — for different embedding models, for
staged rebuilds, for archived indexes. A vector-extended query service MUST, however, serve
`vec:` patterns from #strong[at most one] index at a time, the #dfn[bound index], and MUST
identify it in its service description (@sec-discovery). #rid("SV-IDX-4")

The restriction exists because the query surface has no way to select among indexes, and
scores from two indexes built by different embedding models are geometrically incomparable.
The rule is a scoping decision rather than a deduction: nothing in
#cite("SPARQL-VEC-GENAI") forbids a processor from serving two #emph[separate] stores, and
with a node-IRI seed one could argue the seed's own index is determined. But a literal query
vector determines nothing, two indexes may cover the same resource, and this document's own
non-disclosure rules make "reject the ambiguous query" an unattractive resolution, since an
error would report which indexes exist. One bound index is the conservative choice that
needs no new syntax; a selection argument would mean extending the `vec:` vocabulary, which
is that document's to extend, not this one's (@sec-open).

If no index is bound, the service MUST evaluate `vec:` patterns over an empty eligible
entry set, yielding zero solutions — not an error. #rid("SV-IDX-5") This keeps a service
that has an index today and none tomorrow indistinguishable, from the requester's side,
from a service whose pod simply contains nothing similar — for the requester whose eligible
entry set is empty either way. A requester who holds eligible entries sees solutions in the
bound case and none in the unbound one, and so can tell the two apart; SV-ND-4 states that
limit and @sec-nondisclosure explains why this draft accepts it.

== Index entries are bound to one source resource <sec-entry>

Every index entry MUST record exactly one source resource. #rid("SV-ENT-1") An ingestion
agent MUST derive an entry's vector #strong[only] from content of that source resource's
graph. #rid("SV-ENT-4")

The two are split deliberately, because they are checkable by different parties. SV-ENT-1
binds the entry's shape and a storage server can enforce it on ingestion (SV-ING-5); SV-ENT-4
binds how the vector was computed, which nobody but the ingestion agent can verify — a vector
is a list of numbers and carries no evidence of its provenance. SV-ENT-4 is therefore an
honest obligation on an agent, not a server-checkable rule, and is stated as such rather than
dressed up as enforceable.

This is the load-bearing data-model decision of the specification, and it is forced by the
authorization model. Authorization in Solid is per resource; a vector keyed by RDF term
alone is not, because the same IRI routinely occurs in several resources with different
access control. Two failures follow from a term-keyed index, and SV-ENT-1 closes both:

+ #strong[Attribution.] Asked whether a term is readable, a term-keyed index cannot answer:
  the term may occur in a readable resource and an unreadable one. Admitting it discloses
  the unreadable occurrence's existence whenever the term occurs #emph[only] there;
  excluding it withholds readable content. With a source resource on the entry, the
  question is the base document's own GET-equivalence question, and it has an answer.
+ #strong[Derivation.] A vector derived from a term's occurrences across the whole storage
  encodes unreadable context. Returning it — or its similarity score — to a requester who
  can read only some of those occurrences leaks the rest, in a form that
  #cite("EMB-INVERSION") shows is partially invertible. Restricting derivation to one
  resource's graph makes the entry no more disclosive than the resource itself.

A term occurring in several readable resources therefore yields several entries, one per
resource, each with its own vector. A service MAY return several of them for one query; it
MUST NOT merge entries across source resources into one entry. #rid("SV-ENT-2")

Because several entries may share one indexed term, an entry — not a term — is the unit a
ranking orders, and the deterministic boundary tie-break of #cite("SPARQL-VEC-GENAI") must be
extended accordingly: its key is fixed by the RDF term alone, which is no longer injective
over candidates here. The tie-break key of a candidate entry is therefore the pair
#emph[(source resource IRI, indexed term)], compared in that order, each component under the
N-Triples serialisation and ascending Unicode-codepoint order that document already
specifies. #rid("SV-ENT-5") Without this extension two conforming implementations could
return different top-#emph[k] sets at a tie, which is exactly the reproducibility the sibling
document's rule exists to guarantee.

Where an index entry is expressed in RDF, its source resource MUST be given with
`solid-sparql:sourceResource` (@sec-namespace). #rid("SV-ENT-6")

An indexed term MUST be an IRI or a literal #cite("RDF11-CONCEPTS"); a blank node MUST NOT
be an indexed term. #rid("SV-ENT-3") Blank-node labels are scoped to a single resource
representation under the base document's flattening rule and are not stable across rewrites
of that representation, so an entry keyed by one would silently re-attach to a different
node. This restriction coincides with the mainline embeddable
domain of #cite("SPARQL-VEC-GENAI"); together with SV-ENT-5 it is what keeps the tie-break
key total and stable, since a blank-node label is stable in neither component of the pair.

#note[
  The one-vector-per-term store shape of the sparq `.spqv` container is preserved by this
  model rather than displaced by it: the natural realisation is one term-keyed store per
  source resource, or a single term-keyed store over the storage with a parallel
  source-resource column consulted to build the candidate mask #emph[before] ranking — the
  `IdMask` shape the crate's filtered path already uses. What is #strong[not] admissible is
  a single term-keyed store with no source-resource attribution: the eligibility test of
  @sec-authz has no input without it.
]

== The index descriptor <sec-provenance>

An index descriptor MUST carry, expressed in RDF, the embedding-provenance record required
by #cite("SPARQL-VEC-GENAI") — the embedding model identifier, the model version or content
digest, the distance metric, the normalisation regime, and the vector dimension — and
SHOULD carry the verbalisation regime used to produce the embedded text. #rid("SV-DESC-1")

An index descriptor MUST additionally state the index's #strong[scope]: the RDF sources of
the storage the index is intended to cover, given as a set of resource or container IRIs.
#rid("SV-DESC-2") Scope is a statement of intent, not a permission: it bounds which
resources an ingestion agent may write entries for (@sec-ingest), and it lets a requester
understand why a search returned nothing. It grants no access and is never consulted to
decide eligibility, which is always the requester's own read authorization (@sec-authz).

An index descriptor is an ordinary RDF source with a single representation, not a
per-requester view: it cannot be filtered per reader, and this document does not ask it to
be. What governs it is its own access control. An index descriptor MUST NOT state any
per-resource fact — an entry count, an indexed-term count, a per-resource build timestamp —
about a resource more restrictively access-controlled than the descriptor itself.
#rid("SV-DESC-3")

Scope (SV-DESC-2) is exempt from SV-DESC-3 by construction, because a scope names resources
and containers rather than reporting facts derived from their contents; but the exemption is
not a licence to publish a descriptor more broadly than its scope. A storage owner SHOULD
give an index descriptor access control at least as restrictive as the most restrictive
resource in its scope. #rid("SV-DESC-4")

The per-requester hygiene rule this document does impose falls where per-requester filtering
is actually possible: on the #emph[service description], which the server generates per
request (@sec-discovery). The distinction matters. An entry count is a measure of pod
content; the descriptor cannot hide it from some readers and not others, so the only
available control is not to publish it beside a scope whose resources are more protected
than the descriptor.

#ednote[
  #cite("SPARQL-VEC-GENAI") requires the provenance record and requires that it be
  expressible in RDF, but explicitly leaves the concrete term names to a future revision.
  This document therefore states #emph[which facts] the descriptor must carry and
  deliberately does not mint names for them: minting them here would create precisely the
  second, competing vocabulary this draft exists to avoid. The example below uses
  placeholder `vec:` names purely illustratively; they are not normative and will be
  replaced by whatever that document registers.
]

An illustrative descriptor:

```turtle
@prefix solid-sparql: <http://www.w3.org/ns/solid/sparql#> .
@prefix vec:          <http://sparq.dev/vec#> .
@prefix ldp:          <http://www.w3.org/ns/ldp#> .

<https://alice.pod.example/indexes/notes>
    a                       solid-sparql:VectorIndex ;
    solid-sparql:indexScope <https://alice.pod.example/notes/> ,
                            <https://alice.pod.example/contacts/> ;
    vec:model               "example-embedding-model" ;
    vec:modelVersion        "2026-03" ;
    vec:dimension           384 ;
    vec:metric              vec:Cosine ;
    vec:normalisation       vec:L2 .
```

= Namespace and minted terms <sec-namespace>

This document mints the following terms in the namespace
`http://www.w3.org/ns/solid/sparql#` — the namespace #cite("SOLID-SPARQL-QUERY") already
mints in, written below with the prefix `solid-sparql:`.

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Term][Type][Definition],
  [`solid-sparql:VectorSearch`], [Instance of `sd:Feature`
    #cite("SPARQL11-SERVICE-DESCRIPTION")], [Advertises, in a service description, that the
    service implements this add-on (@sec-discovery).],
  [`solid-sparql:ApproximateVectorSearch`], [Instance of `sd:Feature`], [Advertises that the
    service may answer `vec:` patterns approximately, with recall below one (@sec-approx).
    A service that always evaluates exactly MUST NOT advertise it.],
  [`solid-sparql:VectorIndex`], [Class], [The class of index descriptor resources
    (@sec-index).],
  [`solid-sparql:indexScope`], [RDF property], [Relates an index descriptor to a resource or
    container the index is intended to cover (@sec-provenance).],
  [`solid-sparql:sourceResource`], [RDF property], [Relates an index entry to the one RDF
    source its vector was derived from (@sec-entry).],
  [`solid-sparql:vectorIndex`], [RDF property], [Relates a query service to its bound index
    descriptor (@sec-binding, @sec-discovery).],
)

Reusing the base document's namespace is deliberate: this add-on is meant to be adopted
#emph[with] that document, by the same Solid Community Group process, and a separate
namespace would signal a separate design. Like the base document's own terms, these are
proposals subject to that group's adoption; until a namespace document is published,
implementations should treat the IRIs as defined here.

#ednote[
  Two governance facts must be recorded honestly. First, minting into another
  specification's namespace requires that specification's editors' assent; this draft
  cannot grant itself that, and if it is refused the terms move to a namespace of their own
  with no change to any other requirement here. Second, the task that commissioned this
  draft described the base specification as minting in a `w3id.org` namespace under the
  maintainer's control. No such namespace appears in the vendored base specification, whose
  minted terms are in `http://www.w3.org/ns/solid/sparql#`, and it is #emph[not] the
  `w3id.org` namespace that does appear in this project's estate — that one belongs to the
  spec-companion vocabulary #cite("SPEC-COMPANION") and is a separate, already-settled
  question. This document follows the specification text rather than the task description;
  the discrepancy is flagged for the maintainer rather than silently reconciled.
]

The similarity query vocabulary is #strong[not] minted here: it is the `vec:` namespace of
#cite("SPARQL-VEC-GENAI"), used unchanged. That namespace is a vendor namespace, and this
document inherits both the liability and the migration commitment that document records for
it (@sec-open).

= Ingestion <sec-ingest>

== Embeddings are produced outside the server

A vector-extended query service MUST NOT compute embeddings of storage content. Index
entries are produced by an ingestion agent and written to the storage. #rid("SV-ING-1")

This is not an efficiency choice; it is the point. If the server embeds, then enabling
search means the server acquires a reason to read every resource in the storage and send
a rendering of it to whatever model it is configured with — a data flow the pod's owner did
not authorize resource by resource and cannot inspect. Keeping the model outside makes the
disclosure explicit and attributable: some agent, holding some credential, read some
resources and sent them somewhere, and the storage's own access logs say which. It also
matches the estate this draft is grounded in, where embeddings are produced out of process
behind a provider-agnostic trait and imported.

A vector-extended query service MUST NOT transmit any content of the storage to a
third-party service as a side effect of evaluating a query. #rid("SV-ING-2")

== Ingestion authorization

An ingestion agent MUST hold write access to the index, per the storage's access control
system, to write entries. #rid("SV-ING-3")

An ingestion agent MUST hold read access to a source resource to write an entry whose
source resource it is. #rid("SV-ING-4") An agent that cannot read a resource cannot
legitimately have embedded it; accepting such an entry would let an agent inject vectors
attributed to resources it has never seen, and — because entries are served subject to the
#emph[reader's] authorization, not the writer's — those vectors would be served to readers
as if derived from that resource (@sec-security).

== Validation obligations

An index-hosting storage server MUST reject an ingestion request, without partial
application, if any of the following holds: #rid("SV-ING-5")

+ the ingesting agent does not hold write access to the index (SV-ING-3);
+ the ingesting agent does not hold read access to an entry's declared source resource
  (SV-ING-4);
+ an entry's source resource is outside the index's declared scope (@sec-provenance);
+ a vector's dimension differs from the index's declared dimension;
+ a vector contains a value that is not finite;
+ the declared provenance of the ingested vectors is not identical to the index
  descriptor's provenance record (@sec-provenance);
+ an entry's indexed term is a blank node (SV-ENT-3);
+ an entry's source resource is not an RDF source of this storage;
+ the request contains two entries with the same source resource and indexed term;
+ the request is not fully parseable within its declared bounds.

The first two conditions are what make SV-ING-3 and SV-ING-4 conformance requirements rather
than advice: they bind the ingestion agent, but the storage server is the party that can
check them, and this list is where it MUST.

Fail-closed validation is the same obligation #cite("SPARQL-VEC-GENAI") places on bulk
import and on embedding-provider responses, and for the same reason: a silently padded,
truncated, or re-ordered vector batch attaches vectors to the wrong terms, and the result
is not an error but a plausible wrong answer.

Ingestion MUST be atomic with respect to concurrent queries: a query MUST NOT observe a
partially applied ingestion. #rid("SV-ING-6")

#ednote[
  This document specifies the ingestion #emph[contract] but does not pin a media type or
  byte-level payload format for entry batches. A dense vector payload is a poor fit for an
  RDF serialisation, and choosing a binary format is a registry-shaped decision that should
  be made once, across #cite("SPARQL-VEC-GENAI") and this document, rather than twice.
  Interoperable ingestion is therefore not achieved by this revision; @sec-open records it
  as the most consequential open item.
]

== Verbalisation and what leaves the pod

The text an ingestion agent embeds is a #emph[verbalisation] of pod content — labels,
literals, types — and it is exactly what leaves the storage when a remote provider is used.
An ingestion agent SHOULD NOT transmit verbalised content of a storage to a third-party
embedding provider without the storage owner's explicit, informed configuration for that
provider. #rid("SV-ING-7")

The verbalisation regime changes what a vector means, so it belongs in the provenance
record (SV-DESC-1) — two indexes over the same resources with different verbalisation
regimes are not interchangeable even under an identical model.

= The similarity query surface <sec-query>

== Reuse, without extension

A vector-extended query service MUST recognise and evaluate the `vec:nearest` and
`vec:search` graph patterns of #cite("SPARQL-VEC-GENAI"), and MUST NOT define any
additional similarity predicate, function, or `SERVICE` form for this add-on.
#rid("SV-QRY-1")

Every grammar, typing, error, degenerate-case, ordering, and boundary-tie requirement of
that document applies unchanged. In particular the solutions of a `vec:` pattern enter the
surrounding query as an unordered solution multiset, so a requester who wants best-first
order writes `ORDER BY DESC(?score)` over a `vec:search` score variable, and every
SPARQL 1.1 #cite("SPARQL11-QUERY") construct — joins, `FILTER`, aggregation, `ORDER BY` —
applies to vector results exactly as to any other solutions. No new grammar production is
introduced by this document.

== Scope: what a vector pattern searches <sec-scope>

A `vec:` pattern MUST be evaluated relative to the #strong[active graph] of the basic graph
pattern in which it occurs: no entry outside its eligible entry set (@sec-conformance) —
current, readable, and sourced from a resource contributing to the active graph — may be a
candidate. #rid("SV-QRY-2") Within that boundary the candidates are the pattern's ranked set
(@sec-masks).

This single rule inherits the base document's dataset discipline wholesale, and it has one
consequence that will surprise implementers, stated here explicitly so it is not
discovered as a bug: with no dataset description the standing default graph is empty, so a
`vec:` pattern in a top-level basic graph pattern has an #strong[empty] eligible entry set
and yields zero solutions. Two idioms follow from the same rule rather than from any new
one — inside `GRAPH ?g { … }` the pattern is evaluated once per named graph of the query's
dataset, so each readable resource contributes its own top-#emph[k] and `?g` binds to the
source resource (which is how a requester learns where a neighbour came from, with no new
vocabulary); and under the base document's opt-in union default graph mode the default graph
is the merge of all readable resource graphs, so the eligible entry set is every readable
current entry and the pattern returns a single storage-wide top-#emph[k].

A vector-extended query service MUST NOT evaluate a `vec:` pattern over a candidate set
larger than the active graph's — in particular, it MUST NOT treat the union of readable
graphs as an ambient scope for vector search in a query that did not request union default
graph mode. #rid("SV-QRY-3")

SV-QRY-3 is the base document's "never a standing default" rule, which forbids exactly this
ambient union, applied to the one surface where an implementer is most tempted to
reintroduce it: a storage-wide index is a single physical object, and searching all of it
is the easy implementation. The rule says the easy implementation is non-conforming.

== Composition with the sibling document's filtered-search mask <sec-masks>

Where a `vec:` pattern's neighbour variable is constrained by ordinary triple patterns,
#cite("SPARQL-VEC-GENAI") requires the candidate mask to be derived from the neighbour
variable's join-connected component, and requires every returned identifier to be in that
mask. That mask and this document's eligible entry set are different sets, and both bind.

The ranked set MUST be the #strong[intersection] of the eligible entry set and the sibling
document's pattern-derived mask, and the authorization component MUST be applied first — the
pattern mask narrows an already-authorized set, never the reverse. #rid("SV-QRY-4") An empty
intersection yields zero solutions, exactly as an empty mask does in that document.

Ordering matters even though intersection commutes, because the sibling document permits a
processor to derive and #emph[cache] a pattern mask. A mask computed over unauthorized
content and then intersected has already read what it must not read, and the cache it
populates is keyed by a sub-pattern rather than by a requester — a cross-requester leak
waiting to be served. Authorization first means the cached artifact is per-authorization,
or it is not built.

== The query seed <sec-seed>

Where the `query` argument of a `vec:` pattern is a node IRI, its vector MUST be drawn from
the #strong[readable] entry set (@sec-conformance) — the seed lookup is #emph[not] confined
to the active graph. #rid("SV-QRY-5") If the seed has no entry in the readable entry set,
the pattern MUST yield zero solutions — not an error. #rid("SV-QRY-6")

Confining the seed lookup to the active graph would make the `GRAPH ?g` idiom useless: the
seed has an entry in its own resource and in no other, so every other binding of `?g` would
find no seed and the query would collapse to the seed's own resource. Widening the lookup to
the readable entry set costs nothing in disclosure — every entry in it comes from a resource
the requester may read in full — while the #emph[candidate] set stays confined by SV-QRY-2,
which is where the disclosure question actually lives. Reading a seed vector from an
#emph[unreadable] entry, by contrast, would be a clean read of protected content, since the
returned neighbours describe where that vector sits in the space; SV-QRY-5 forbids it.
Yielding zero solutions rather than an error preserves the base document's
unreadable-equals-non-existent equivalence: an unreadable seed, a seed that was never
indexed, and a seed that does not exist are mutually indistinguishable.

Where several readable entries share the seed term — which SV-ENT-2 makes routine — the
service MUST use the one whose tie-break key (SV-ENT-5) is least. #rid("SV-QRY-7") Any other
choice would make the result depend on an implementation's internal ordering, breaking both
the invariance requirement of @sec-invariance and cross-implementation reproducibility.

Where the `query` argument is a literal vector, the requester supplies the vector directly
and no seed lookup occurs. A service MAY require an explicit configuration before accepting
literal query vectors at all, as #cite("SPARQL-VEC-GENAI") permits. That document supplies
no provenance for a literal vector and requires it to be treated as being under the index's
declared metric and normalisation regime; only its dimension is checked — and under SV-OPT-4
a dimension mismatch here yields zero solutions rather than that document's query error.

#note[
  A literal query vector is how a client asks a natural-language question without the
  server ever calling an embedding provider: the client embeds its own query text, with its
  own credentials, and sends the numbers. The query text is the requester's, not the pod's,
  so this discloses nothing about the storage. Nothing forbids a service from offering to
  embed query text on the requester's behalf — SV-ING-1 forbids it from embedding
  #emph[storage content], which is a different flow — but the literal-vector path is what
  makes text search available to a service that offers no such thing.
]

== Worked example

A requester searching their own notes for material similar to a seed document, across the
whole storage, and recovering best-first order explicitly:

```sparql
PREFIX vec: <http://sparq.dev/vec#>

SELECT ?note ?score
FROM <http://www.w3.org/ns/solid/sparql#union-default-graph>
WHERE {
  ( ?note ?score ) vec:search ( <https://alice.pod.example/notes/seed#it> 10 ) .
}
ORDER BY DESC(?score)
```

The same search, per resource, without opting into union default graph mode — note that
`?g` reports the source resource of each neighbour:

```sparql
PREFIX vec: <http://sparq.dev/vec#>

SELECT ?g ?note ?score WHERE {
  GRAPH ?g {
    ( ?note ?score ) vec:search ( <https://alice.pod.example/notes/seed#it> 10 ) .
  }
}
ORDER BY DESC(?score)
```

#note[
  The two queries differ in more than convenience. The first returns the ten best
  neighbours in the storage; the second returns up to ten per readable resource, because
  each `GRAPH` binding is a separate evaluation over that resource's entries alone. Neither
  is a filtered version of the other, and a service MUST NOT implement the second by taking
  the first and grouping. Both work because the seed lookup is not confined to the active
  graph (SV-QRY-5): the seed's vector comes from its own resource in every one of the second
  query's per-graph evaluations.
]

= Execution modes <sec-modes>

== Exact scan is the mandatory baseline

Unless the conditions of @sec-approx are met, a vector-extended query service MUST evaluate
a `vec:` pattern exactly: the returned result MUST equal the true top-#emph[k] of the
pattern's #strong[ranked set] under the index's declared metric, with boundary ties broken
per SV-ENT-5. #rid("SV-MODE-1")

An exact scan over the eligible entry set is always available: it needs no index structure
beyond the entries themselves, and its result depends on nothing outside that set. That
second property, not its accuracy, is why it is the mandatory baseline.

== Invariance <sec-invariance>

Two requirements are stated separately here because they are justified differently. A single
combined invariant would condemn an implementation for a scoping mistake in the same breath
as for a leak, and the two deserve different urgency.

#strong[Non-disclosure invariance.] The result set of a `vec:` pattern MUST NOT depend on
any index entry whose source resource is unreadable by the requester. #rid("SV-MODE-2")
Equivalently: two storages that present a requester with the same readable entry set MUST
return the same results for every query that requester can issue, however much they differ
in content that requester cannot read. This is the base document's resource-limit invariant
generalised, and it is the requirement that carries the security argument of @sec-approx.

#strong[Scope determinacy.] The result set MUST be a function of the query, the index's
declared parameters, the pattern's ranked set, and — where the `query` argument is a node
IRI — the seed vector resolved from the readable entry set per SV-QRY-5 and SV-QRY-7. It
MUST NOT depend on anything else. #rid("SV-MODE-3")
This is a semantic requirement, not a disclosure one: it is what makes `GRAPH ?g` mean per
resource and union mode mean storage-wide, rather than both meaning "whatever the index
structure happened to reach". A readable-but-out-of-scope entry influencing a result is a
correctness defect, not a leak.

SV-MODE-2 follows from SV-MODE-3 whenever the eligible set is a subset of the readable set,
which SV-QRY-2 guarantees. They are stated separately anyway, because an implementation that
violates only SV-MODE-3 is wrong, while one that violates SV-MODE-2 is unsafe, and a reader
deciding what to prioritise is entitled to know which is which.

== Post-filtering is non-conforming

A service MUST NOT compute a ranking over entries outside the eligible entry set and then
remove ineligible results. #rid("SV-MODE-4")

The base document already forbids post-filtering for ordinary patterns, on the ground that
intermediate cardinalities and limit behaviour become functions of unreadable content even
when the final solutions coincide. For top-#emph[k] the final solutions do #emph[not] even
coincide, and the failure is louder:

+ #strong[Under-fill.] A global top-#emph[k] with ineligible results removed returns fewer
  than #emph[k] results whenever an ineligible entry falls within the #emph[global]
  top-#emph[k] — so the requester sees a short answer where a full one exists in content
  they are entitled to.
+ #strong[Disclosure.] The number removed is a count of ineligible entries ranking above the
  requester's #emph[k]-th best match: a bounded statistic, between zero and #emph[k], but one
  the requester can evaluate at any point of the embedding space they care to name, and
  accumulate across probes. A short answer is not a degraded answer; it is a measurement of
  the part of the pod the requester was refused.

The conforming behaviour is pre-filtering: restrict to the eligible entry set, then rank
within it, so that #emph[k] is filled whenever #emph[k] eligible entries exist. A service
MAY choose freely among implementation strategies — including over-fetching — provided
candidate generation is itself confined to the eligible entry set and the choice is not
observable in the result set. #rid("SV-MODE-5")

== Approximate retrieval <sec-approx>

A service MAY use an approximate nearest-neighbour backend #cite("HNSW")
#cite("DiskANN") #cite("ACORN") #cite("FILTERED-DISKANN") only if #strong[both] of the
following hold: #rid("SV-MODE-6")

+ its returned result set satisfies SV-MODE-2 — that is, the entries the backend misses do
  not depend on any entry whose source resource the requester cannot read; and
+ the service advertises `solid-sparql:ApproximateVectorSearch` in its service description
  (@sec-namespace, @sec-discovery) and states in its documentation that results are
  approximate with recall below one.

If either condition fails for a request, the service MUST evaluate that request exactly.
#rid("SV-MODE-7")

A service MUST NOT represent approximate results as exact to any consumer, and MUST NOT
substitute approximate evaluation for exact evaluation silently. #rid("SV-MODE-8")

=== Why the first condition bites

This is a real restriction, and it excludes the most natural implementation, so the
argument is set out rather than asserted.

Consider a single proximity-graph index built over every entry in the storage, searched
with a mask that admits only readable entries. Two distinct dependencies on unreadable
content survive the mask. The traversal moves #emph[through] unreadable nodes, so which
readable entries the search reaches depends on where the unreadable ones sit; and, more
fundamentally, the graph's #strong[topology] was built by pruning candidate edges against
the full vector set, so even a traversal restricted to readable nodes walks edges whose
existence was decided by unreadable vectors. Either dependency violates SV-MODE-2, and
unlike a timing side channel neither is marginal: both are visible in the result set itself.

How exploitable that is depends on the adversary. In the strongest case — a requester who
holds the index's provenance record #emph[and] can run the same model and verbalisation
pipeline — the requester can reproduce the readable vectors, compute the exact top-#emph[k]
themselves, and read the difference from the service's answer as a signal about the
unreadable neighbourhood, steering the probe by choosing query vectors. That case is not the
typical one: SV-DESC-1 makes the verbalisation regime only a SHOULD, and the model may be a
service the requester cannot call. But the structural violation of SV-MODE-2 does not depend
on the strong adversary being realised, and the base document's invariants are stated
structurally for exactly that reason — a channel is closed because it exists, not because
someone has been observed using it.

Constructions that #emph[do] satisfy the condition exist, and the topology argument narrows
them sharply. It is not enough to confine #emph[traversal] to readable entries; the
structure traversed must itself have been built over readable entries only. Building index
structures per authorization-equivalence class does that by construction. Over-fetching from
a shared structure and re-ranking exactly does #emph[not], because the candidate pool was
shaped — in both reachability and topology — by content outside the requester's view. The
honest summary is that approximation buys far less here than in a single-tenant store, and
@sec-open records the search for a construction that is both shared and invariant as an open
problem rather than pretending it is solved.

#note[
  The restriction is a #emph[multi-reader] restriction, and it lifts where there is nothing
  to hide: on an index whose scope is entirely public, or a storage with a single reader,
  every entry is readable by every requester who can use the index at all, so SV-MODE-2 is
  satisfied by any backend whatsoever. Note that this does not exempt such a service from
  SV-MODE-3: a shared structure over the whole storage still has to respect active-graph
  scoping, which is a correctness obligation and holds regardless of who can read what.
]

= Authorization <sec-authz>

== Eligibility is GET-equivalence

An index entry's source resource is #dfn[readable] for a request exactly when
#cite("SOLID-SPARQL-QUERY") deems that resource readable — that is, when the storage's
access control system would authorize a `GET` on it for that requester. An entry is eligible
if and only if it is current (@sec-currency), its source resource is readable, and that
resource contributes to the active graph (@sec-scope). #rid("SV-AUTH-1")

No access mode other than the one that makes the source resource #emph[readable] in the
base document's sense grants or denies eligibility. #rid("SV-AUTH-2") The base document
folds the auxiliary-resource case into that definition, and this document inherits it
wholesale rather than restating it: for a WAC access control resource, readability is
governed by WAC's own rule for reading access control resources, so an agent that may read
`/x` but not `/x.acl` has no entry sourced from `/x.acl` in its eligible set — even though
the same agent's read access to `/x` is unaffected. An index over access control documents
is an unusual thing to build, but it is not forbidden, and this is the rule that governs it.

#note[
  SV-AUTH-2 is worded by reference rather than by enumeration for a reason. The intuitive
  phrasing — "write, append, and control access are irrelevant" — reads well and is wrong:
  for an access control resource, control access is exactly what makes it readable. Deferring
  to the base document's `readable` predicate keeps that case correct without a special case.
]

== Eligibility is decided before ranking

A service MUST determine the eligible entry set — all three of its components — before
ranking, and MUST evaluate the ranking over exactly that set. #rid("SV-AUTH-3")

The authorization decisions used MUST reflect the storage's access control state as
evaluated for this request; a service MAY use caches or indexes to compute the eligible set,
provided the observable result equals a fresh per-request evaluation, and MUST invalidate
such a cache when the access control state it was derived from changes. #rid("SV-AUTH-4")
Serving a stale eligibility mask is not a performance defect but an authorization failure:
it serves a revoked reader content the pod owner has already withdrawn.

== The currency rule <sec-currency>

An index entry is #strong[current] for a request if and only if its indexed term occurs, at
evaluation time, in its own source resource's graph as that graph appears in the requester's
authorized dataset. A stale entry is not eligible, and therefore never enters a ranking.
#rid("SV-AUTH-5")

An index is a derived artifact, and derived artifacts go stale. Ordinarily staleness is an
accuracy problem; here it is a disclosure problem, because a `vec:` pattern binds
#strong[terms], and a term may be a literal — that is, content. An index built before a
resource was edited holds entries for literals the owner has since deleted; serving them
republishes deleted content from a resource whose current state does not contain it.

Two details of SV-AUTH-5 are load-bearing, and both are easy to get wrong.
Currency is checked against the entry's #emph[own] source resource, not against the
authorized dataset at large: a literal deleted from `/notes/a` but still present in
`/notes/b` must not resurrect `/notes/a`'s entry, which is exactly what a dataset-wide
membership test would allow. And currency is a component of #emph[eligibility], evaluated
before ranking — not a filter applied to results afterwards. Applied afterwards it would be
a post-filter, forbidden by SV-MODE-4 and productive of the same under-fill SV-ND-1 forbids;
applied before, #emph[k] is filled from the remaining current entries and nothing leaks. The
check is cheap either way: membership of the indexed term in one graph is a triple lookup
the service can already perform.

The party responsible for an index — its ingestion agent, not the query service, which
SV-IDX-2 bars from mutating it — SHOULD keep the index current with the storage.
#rid("SV-AUTH-6") A stale index
returns fewer neighbours than a fresh one would — that is a recall cost, and it is the
honest failure mode this design chooses over serving content that no longer exists.

= Existence non-disclosure for vector search <sec-nondisclosure>

The base document's overarching requirement — that a query service disclose nothing, by any
observable behaviour, about resources outside the requester's authorized dataset, and that
unreadable and non-existent resources be observationally equivalent — applies to this add-on
in full. Most of it is carried by assertions already stated — bindings and scores by
SV-AUTH-1 and SV-AUTH-3, result-set shape under approximation by SV-MODE-2, ancillary
surfaces by SV-DESC-3 and @sec-discovery — and this document does not restate those with
fresh identifiers. Three obligations, however, take a shape that ranking gives them and
nothing else states, and a fourth assertion records what this section does #strong[not]
conceal:

+ #strong[Cardinality.] The number of solutions a `vec:` pattern returns MUST NOT be a
  function of any entry outside the pattern's ranked set. Under exact evaluation this is
  exact: writing #emph[n] for the number of ranked-set entries whose indexed term is not the
  seed term, the pattern MUST return exactly `min(k, n)` solutions — never fewer because an
  entry outside the ranked set outranked one inside it. Under an approximate backend admitted
  by SV-MODE-6 the count MAY fall short of `min(k, n)`, since recall is below one, but the
  shortfall MUST NOT depend on any entry the requester cannot read (SV-MODE-2).
  #rid("SV-ND-1")
+ #strong[Errors and status.] A request whose seed, bound index, or referenced graph is
  unreadable or absent MUST be indistinguishable — in status code, headers, error document,
  and message — from the same request against a non-existent one. There is no
  forbidden-versus-not-found distinction anywhere in this surface, which is why SV-QRY-6,
  SV-IDX-5, and SV-OPT-4 all resolve to zero solutions rather than to an error.
  #rid("SV-ND-2")
+ #strong[Timing.] A service SHOULD ensure that observable differences in processing time do
  not disclose the existence or contents of resources outside the authorized dataset.
  #rid("SV-ND-3")
+ #strong[Neither the bound index nor its declared dimension is concealed.] A requester
  holding at least one eligible entry observes solutions where a service with no bound index
  yields none, and MAY therefore conclude that #emph[some] index is bound; that same
  requester MAY, by issuing literal query vectors of differing dimension and observing which
  of them return solutions, conclude what the bound index's declared dimension is. This
  document does not require either inference to be prevented, and a service is not required
  to withhold or pad results in order to prevent them. What a service MUST NOT disclose to a
  requester who cannot read the index descriptor is anything further about the descriptor
  #emph[as a resource] — its IRI, its scope, its entry counts, the remainder of its
  provenance record (SV-DESC-1) beyond the declared dimension, or any other fact it carries
  — through the service description (SV-DISC-3, SV-DISC-4) or any other surface, and it MUST
  NOT disclose any entry sourced from a resource that requester cannot read (SV-AUTH-1,
  SV-SEC-1). #rid("SV-ND-4")

The seed exclusion in SV-ND-1 is not an afterthought: #cite("SPARQL-VEC-GENAI") excludes a
node-IRI seed from its own results, so with #emph[k] at or above the number of eligible
entries the attainable maximum is short by the seed's own entries. A cardinality rule stated
without that term would be unsatisfiable exactly when the pod is small — the case a personal
data store makes common.

SV-ND-4 is where this document draws a boundary rather than asserting a stronger invariant
it cannot keep, so the boundary is stated plainly. Eligibility (SV-AUTH-1) turns on the
#emph[source resource], never on the descriptor: a requester who can read `/notes/a` gets
`/notes/a`'s neighbours whether or not they may read the descriptor of the index those
entries live in. The consequence is that the bound index's #emph[existence] is observable to
any requester who has something to search, by the ordinary result set alone and without any
of the probing @sec-optionality discusses. The #emph[declared dimension] follows for the same
requester at the cost of a sweep: SV-OPT-4 converts the wrong-dimension query error into zero
solutions, which removes the error as an oracle but leaves the difference between a literal
query vector that returns solutions and one that does not — and that difference is the
dimension. SV-DESC-1 places the dimension in the descriptor's provenance record, so a rule
prohibiting disclosure of that record entire would prohibit behaviour this document
elsewhere prescribes; SV-ND-4 therefore excepts the dimension alongside the existence bit
rather than asserting a prohibition the specified behaviour breaks.

This draft accepts both channels and scopes the claim to match them: what SV-IDX-5 and
SV-OPT-4 make indistinguishable is a bound index and no index #strong[to a requester whose
eligible entry set is empty] — the requester with nothing to search, for whom every such
pattern yields zero solutions whatever its dimension. For every other requester the binding
and the dimension are observable, and no assertion of this document says otherwise.

#note[
  The alternative was considered and rejected. Making descriptor readability a precondition
  for eligibility — evaluate every `vec:` pattern over an empty eligible set when the
  requester cannot read the descriptor — would close both channels, and would also disable the
  add-on for precisely the readers it exists to serve: SV-DESC-4 asks an owner to keep the
  descriptor at least as restrictive as the most restrictive resource in its scope, so on any
  index spanning a shared resource and a private one, the sharee is exactly the requester who
  can read some scoped resource and not the descriptor. That rule would make the shared case
  return nothing, and it would leave SV-MODE-7's descriptor-unreadable branch (@sec-open)
  describing a request that can no longer return a solution. Concealing existence and dimension
  is not worth that, and the honest move is to say which requester the concealment holds for.

  It is a narrow departure from the base document's rule that an unreadable resource and an
  absent one be observationally equivalent, and it is recorded as one in @sec-relationship
  rather than argued away: the unreadable resource is the descriptor, and what leaks about it
  is that some index is bound and, to a requester willing to sweep, at what dimension. Its
  IRI, its scope, the remainder of its provenance record, and every entry under it remain
  governed by SV-AUTH-1, SV-DESC-3, SV-DISC-3, and SV-SEC-1.
]

#note[
  SV-ND-3 is weaker than the rest, and deliberately so — it is a SHOULD in the base document
  too. It is worth noting that exact scan over the eligible set makes evaluation time
  roughly proportional to the eligible set, which is the requester's own readable content;
  a shared approximate structure does not, since traversal cost depends on the whole
  storage. The mode restriction of @sec-approx therefore improves the timing story as a side
  effect, though it does not settle it.
]

= Discovery and service description <sec-discovery>

This document defines no new link relation type #cite("RFC8288") and no new discovery
mechanism: a vector-extended query service is discovered exactly as any query service is,
through the endpoint link relation of #cite("SOLID-SPARQL-QUERY"), and the add-on is
discovered from the service description of that endpoint.

A vector-extended query service SHOULD advertise the add-on in its SPARQL 1.1 Service
Description #cite("SPARQL11-SERVICE-DESCRIPTION") with
`sd:feature solid-sparql:VectorSearch`, and a service not implementing this document MUST
NOT advertise that feature. #rid("SV-DISC-1")

A service that may answer any `vec:` pattern approximately MUST additionally advertise
`solid-sparql:ApproximateVectorSearch`, and a service that always evaluates exactly MUST NOT
advertise it. #rid("SV-DISC-2") This is where SV-MODE-6's disclosure condition is discharged:
without a term for it, the requirement that approximation be surfaced would have no vocabulary
to be surfaced in, and would reduce to an obligation nobody could satisfy in the machine-readable
description clients actually read.

A service description MAY identify the bound index with `solid-sparql:vectorIndex`, and
MUST omit it — as it MUST omit the features themselves — from descriptions served to a
requester who cannot read the index descriptor. #rid("SV-DISC-3")

What SV-DISC-3 keeps from such a requester is the descriptor's IRI and everything reachable
through it. It does not keep the binding's existence or the declared dimension, both of
which SV-ND-4 declines to conceal and both of which that requester can observe in the
results of `vec:` patterns once they hold an eligible entry. The rule is worth stating even
so: the descriptor IRI is a resource identifier the requester may not dereference, and
advertising a feature naming it invites a request that must then be refused.

A service description MUST NOT enumerate index scopes, indexed resources, entry counts, or
any other index-derived inventory that is not filtered to the requester's readable subset.
#rid("SV-DISC-4")

A service description example, extending the base document's:

```turtle
@prefix sd:           <http://www.w3.org/ns/sparql-service-description#> .
@prefix solid-sparql: <http://www.w3.org/ns/solid/sparql#> .

<https://alice.pod.example/sparql>
    a                          sd:Service ;
    sd:endpoint                <https://alice.pod.example/sparql> ;
    sd:supportedLanguage       sd:SPARQL11Query ;
    sd:feature                 solid-sparql:UnionDefaultGraphOptIn ,
                               solid-sparql:VectorSearch ;
    solid-sparql:vectorIndex   <https://alice.pod.example/indexes/notes> .
# No graph inventory; no index inventory; no entry counts.
```

A client that depends on vector search SHOULD confirm the feature is advertised before
relying on it. #rid("SV-DISC-5") The reason is that the failure is silent: a service that
does not implement this document is not a vec-extended processor at all, so
#cite("SPARQL-VEC-GENAI")'s hard-error rule for unrecognised `vec:` terms does not apply to
it — and `vec:nearest` is in any case one of that document's #emph[recognised] terms. To a
plain query service the pattern is an ordinary triple pattern matching no data, so the
request succeeds with zero solutions, indistinguishable from a pod with nothing similar in
it. That is the same shape as the base document's warning about union default graph mode,
and it has the same remedy: check the service description first.

= Security and privacy considerations <sec-security>

#strong[The index is derived content, not metadata.] A vector produced from a resource
carries information about that resource's text; the embedding-inversion and
membership-inference literature #cite("EMB-INVERSION") #cite("EMB-LEAKAGE") is clear that
this is partially recoverable. A server MUST NOT expose an entry's vector — through a
result, a descriptor, or a downloadable index payload — to a requester who cannot read that
entry's source resource. #rid("SV-SEC-1") The descriptor's own access control is governed
by SV-DESC-4, which asks the storage owner to make it at least as restrictive as the most
restrictive resource in its scope.

#strong[Disclosure to embedding providers.] Under any remote embedding arrangement, the
verbalised content of the covered resources is disclosed to the provider. SV-ING-1 and
SV-ING-2 keep the server out of that flow entirely, so the disclosure is the ingestion
agent's, made with the owner's credentials and visible in the storage's own access logs.
Deployments MUST treat everything embedded through a remote provider as disclosed to that
provider. #rid("SV-SEC-2")

#strong[Index poisoning.] An agent with write access to an index can write
entries that rank chosen terms highly for chosen queries; an agent with write access to a
covered #emph[resource] can influence what a future honest ingestion embeds. Neither breaks
the authorization rules of @sec-authz — a poisoned entry is still served only to requesters
who can read its source resource, and SV-AUTH-5 still requires the bound term to occur in
the live graph — but both bias retrieval. Consumers MUST NOT treat vector rank as evidence
of anything beyond similarity in a space whose construction they are trusting.
#rid("SV-SEC-3") The write-access story for an index is the write-access story of the pod;
this document adds no new privilege and no new escalation path, and the derived-content
rule of SV-SEC-1 means an index cannot be used to read around a resource's access control.

#strong[Probing the embedding space.] A literal query vector lets a requester probe the
index at any point of the space. Under the rules of @sec-authz that probe is answered only
from entries the requester may read, so it grants nothing they could not obtain by reading
those resources and embedding them. Under a non-conforming post-filtering implementation it
grants a great deal (@sec-modes) — which is the concrete reason SV-MODE-4 is a MUST NOT and
not a SHOULD NOT.

#strong[Cost and denial of service.] Exact scan cost grows with the eligible entry set, and
a requester may issue many `vec:` patterns in one query. The base document's query-cost
considerations apply unchanged, with one addition: any limit a service imposes MUST itself
satisfy SV-MODE-2, so a limit that triggers as a function of total index size rather than of
the requester's own readable content is a disclosure channel as well as a fairness problem.
#rid("SV-SEC-4")

#strong[Update remains out of scope.] Because index management is Solid Protocol traffic
(SV-IDX-2), this add-on introduces no write path through the query endpoint and therefore
no new surface for the update-refusal requirements of the base document to cover.

= Accuracy and honesty considerations <sec-accuracy>

This section states plainly what this document does not promise.

- #strong[No accuracy, recall, or throughput figure is part of this specification.] None
  appears in it. Conformance is a property of semantics — eligibility, exactness,
  cardinality, error behaviour — never of a benchmark score.
- #strong[Approximate retrieval is honestly approximate.] Its recall is below one by
  construction, and @sec-approx restricts #emph[which] approximate constructions are
  admissible on disclosure grounds, not on accuracy grounds. Meeting SV-MODE-6 does not make
  an approximate answer exact.
- #strong[Retrieval quality is a property of the embedding model, not of this document.]
  A conforming service with a poor model returns poor neighbours, correctly and
  non-disclosively.
- #strong[Reproducibility depends on the provenance record.] Two indexes built with
  different models, versions, normalisation regimes, or verbalisation regimes are not
  comparable; SV-DESC-1 exists so that incomparability is detectable rather than silent.
- #strong[Nothing here is a cryptographic protection.] Access control is enforced by the
  storage's authorization system and by the structural rules of @sec-authz. This document
  defines no encryption, no oblivious retrieval, and no protection against a compromised or
  dishonest server, which sees all entries by construction. A deployment whose threat model
  includes the server itself is not addressed by this design.

= sparq implementation status <sec-impl-status>

#emph[This section is informative.]

The sparq estate contains both halves of this design and #strong[no wiring between them].
The opt-in `sparq-vectors` crate provides the term-keyed store, the exact scan, two
approximate backends, the provider-agnostic embedding trait, the fail-closed bulk import,
the graph-fingerprint staleness guard, the mask-filtered search path, and the `vec:`
magic-predicate rewrite. The `sparq-solid` crate
provides the WAC/ACP materialiser and the per-session authorized view that
#cite("SOLID-SPARQL-QUERY") requires, and passes that document's vendored query-semantics
conformance suite. Neither crate depends on the other in the workspace.

Consequently #strong[no assertion of this document is implemented today]. The table records
what each group would need, so a reader can judge the distance rather than infer it.

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Assertions][sparq status][Notes],
  [`SV-OPT-1..4`], [Not implemented], [there is no vector-extended query service; the add-on
    being optional is, trivially, satisfied],
  [`SV-IDX-1..5`], [Not implemented], [no index-as-resource model exists; `.spqv` stores are
    files, not storage resources],
  [`SV-ENT-1..6`], [Not implemented], [the `.spqv` store is keyed by term identifier with no
    source-resource attribution — the single largest data-model gap, and the reason a naive
    integration would be non-conforming rather than merely incomplete. The crate's
    boundary-tie rule is keyed on the term alone, so SV-ENT-5's extended key is not
    implemented either],
  [`SV-DESC-1..4`], [Partly available], [the provenance record of SV-DESC-1 exists as an
    opt-in `.spqv` version-3 header and is expressible in RDF; scope and the descriptor
    hygiene rules have no counterpart],
  [`SV-ING-1..7`], [Partly available], [embeddings are already produced outside the engine
    behind a provider-agnostic trait, and bulk import is already fail-closed on dtype,
    shape, length, and non-finite values; the Solid-side authorization obligations
    (SV-ING-3..5) have no counterpart],
  [`SV-QRY-1..7`], [Partly available], [the `vec:` patterns and their rewrite exist; the
    active-graph scoping of SV-QRY-2..3 does not, because the crate has no notion of an
    authorized dataset. The crate's own filtered path is the sibling document's BGP mask,
    not this document's authorization mask, so SV-QRY-4's composition rule is unaddressed],
  [`SV-MODE-1..8`], [Partly available], [exact scan is available and is what the crate's
    answer-exact entry points call; the #emph[default build] nonetheless compiles an
    approximate backend (the on-disk Vamana graph — only the in-RAM HNSW backend is behind
    an opt-in feature), so "exact by default" is a property of which entry point a caller
    invokes, not of the build. Both invariance requirements and the admissibility test of
    SV-MODE-6 are unanalysed for the crate's shared index structures],
  [`SV-AUTH-1..6`], [Not implemented], [the authorized view exists in `sparq-solid` and the
    mask-filtered search path exists in `sparq-vectors`; nothing connects them, and the
    crate's staleness guard is a whole-store fingerprint rather than SV-AUTH-5's per-entry
    currency test],
  [`SV-ND-1..3`], [Not implemented], [follows from the above],
  [`SV-DISC-1..4`], [Not implemented], [service-description emission is a server concern; no
    vector feature is advertised anywhere],
  [`SV-SEC-1..4`], [Deployment obligations], [not machine-checkable in the crate],
)

The shortest honest path to a first conforming implementation is: attribute index entries to
a source resource (SV-ENT-1) — plausibly reachable through the crate's existing opt-in
per-vector metadata sidecar rather than a new container version, which is worth checking
before assuming a format change; derive the candidate mask from `sparq-solid`'s authorized
view and intersect it with the existing BGP mask in that order (SV-AUTH-1, SV-AUTH-3,
SV-QRY-4); scope the rewrite to the active graph (SV-QRY-2); and serve exactly, since the
crate's shared approximate indexes are not obviously admissible under SV-MODE-6.

= Open design questions <sec-open>

Each of these is a real gap, recorded rather than papered over.

+ #strong[An interoperable ingestion format.] This revision specifies the ingestion
  contract but not its payload encoding (@sec-ingest), so two implementations cannot
  exchange index entries. The decision is shared with #cite("SPARQL-VEC-GENAI"), whose
  container binding for the provenance record is likewise unpinned, and should be made once
  for both. This is the most consequential open item.
+ #strong[Admissible approximation for multiple readers.] @sec-approx admits per-view index
  structures and confined candidate generation, and excludes the shared proximity graph that
  makes approximation attractive. Whether a construction exists that is both shared and
  invariant under the eligible-entry mask — perhaps by bounding the influence of masked
  nodes on traversal — is open, and is the difference between this add-on scaling and not.
+ #strong[Multi-index selection.] SV-IDX-4 permits one bound index because the query surface
  cannot name one. Adding a selection argument means extending the `vec:` vocabulary, which
  belongs to #cite("SPARQL-VEC-GENAI"); until then a storage with two models must run two
  query services or rebind.
+ #strong[Surface form.] Inherited unresolved from #cite("SPARQL-VEC-GENAI"): whether the
  normative surface should be magic predicates, a `SERVICE` form, or extension functions.
  This document takes no position beyond declining to invent a second one.
+ #strong[The observable binding, and the residual dimension oracle.] SV-ND-4 accepts that a
  requester holding any eligible entry can see that an index is bound — solutions appear
  where an unbound service would return none — and SV-OPT-4's zero-solutions rule converts
  the wrong-dimension probe but still lets that requester recover the bound index's declared
  dimension by sweeping; SV-ND-4 excepts both observables, so neither is a prohibition this
  document then breaks. Both channels have the same shape and the same candidate remedies:
  evaluate `vec:` patterns over an empty eligible set for requesters not entitled to the
  descriptor, which costs the shared-index case (@sec-nondisclosure); or pad responses, which
  is unspecified here. This revision accepts both channels and says so; a future one must
  decide whether that is the right trade for a personal data store.
+ #strong[Advertising approximation without disclosing the index.] SV-DISC-2 requires an
  approximate service to advertise `solid-sparql:ApproximateVectorSearch`, while SV-DISC-3
  requires the features to be omitted from descriptions served to a requester who cannot read
  the index descriptor. For such a requester SV-MODE-6's disclosure condition cannot be met,
  so SV-MODE-7 forces exact evaluation. That is a safe resolution — it fails toward exactness
  — but it means approximation is unavailable precisely for the requesters a shared index
  would most want it for, and no better arrangement is specified.
+ #strong[Namespace governance.] Whether these terms may be minted in the base document's
  namespace, and which namespace the base document's own terms ultimately live in
  (@sec-namespace).
+ #strong[Index freshness as a first-class contract.] SV-AUTH-5 makes staleness safe but
  not visible: a requester cannot tell a pod with nothing similar from a pod whose index has
  not caught up. A freshness statement in the descriptor would help, but per-resource
  freshness is itself disclosive (SV-DESC-3), so the useful form is not obvious.
+ #strong[Deletion and the right to be forgotten.] Deleting a resource makes its entries
  unservable under SV-AUTH-5, but this document does not require the entries to be
  #emph[erased]. For a personal data store that gap is likely unacceptable, and a future
  revision should require erasure with a bounded deadline.

#proposal[
  Two capabilities are deliberately deferred rather than specified. A #strong[hybrid]
  surface fusing vector rank with a lexical rank inside the query would ride on whatever
  in-query fusion #cite("SPARQL-VEC-GENAI") eventually specifies, and inherits every rule of
  @sec-authz unchanged. #strong[Per-language or multi-modality named vectors] would need
  more than one vector per (source resource, term) and therefore a data-model change to
  SV-ENT-1 rather than an addition to it. Neither carries conformance language in this
  draft.
]

= Testable assertions and conformance testing <sec-testable>

== Assertion groups

#table(
  columns: 4,
  align: (left, left, left, left),
  table.header[Group][Section][Class addressed][Scope],
  [`SV-OPT`], [@sec-optionality], [Query service], [optionality, class composition, conflict
    resolution],
  [`SV-IDX`], [@sec-index], [Query service], [index-as-resource, CRUD placement, index
    binding],
  [`SV-ENT`], [@sec-entry], [Storage server (SV-ENT-1..3, 6); query service (SV-ENT-5);
    ingestion agent (SV-ENT-4)], [entry identity, source-resource attribution, tie-break
    key],
  [`SV-DESC`], [@sec-provenance], [Storage server (SV-DESC-1..3); storage owner
    (SV-DESC-4)], [descriptor content, provenance record, descriptor hygiene],
  [`SV-ING`], [@sec-ingest], [Query service (SV-ING-1, 2); ingestion agent (SV-ING-3, 4,
    7); storage server (SV-ING-5, 6)], [ingestion authorization and fail-closed
    validation],
  [`SV-QRY`], [@sec-query], [Query service], [query-surface reuse, active-graph scoping,
    mask composition, seed handling],
  [`SV-MODE`], [@sec-modes], [Query service], [exact baseline, invariance, approximation
    admissibility],
  [`SV-AUTH`], [@sec-authz], [Query service], [eligibility, pre-ranking enforcement,
    currency],
  [`SV-ND`], [@sec-nondisclosure], [Query service], [cardinality, error indistinguishability,
    timing, the limit of index-existence and declared-dimension concealment],
  [`SV-DISC`], [@sec-discovery], [Query service (SV-DISC-1..4); client (SV-DISC-5)],
    [service description and advertisement],
  [`SV-SEC`], [@sec-security], [Query service (SV-SEC-1, 4); deployment (SV-SEC-2);
    consumer (SV-SEC-3)], [derived-content exposure, provider disclosure, poisoning, limits],
)

The class column matters for testing: an assertion addressed to an ingestion agent cannot be
tested by probing a query service, and the two are commonly different software. Where an
agent-directed obligation has a server-side counterpart that #emph[can] be probed — as
SV-ING-3 and SV-ING-4 do, through SV-ING-5's rejection list — the test targets the server.

== Conformance test scenarios

The scenarios below restate normative requirements in testable form; the referenced
sections are authoritative. They assume a storage in which requester #emph[R] can read
`/notes/a` and `/notes/b` but not `/private/journal`, `/no-such-resource` does not exist,
and a bound index covers all three of the first-named containers' resources.

+ #strong[Optionality.] A server implementing none of this document passes the
  #cite("SOLID-SPARQL-QUERY") conformance suite unchanged. (@sec-optionality)
+ #strong[Neutrality.] Every query containing no `vec:` pattern returns identical solutions
  with the index bound and unbound. (@sec-optionality)
+ #strong[Empty standing default.] A `vec:` pattern in a top-level basic graph pattern with
  no dataset description returns zero solutions. (@sec-scope)
+ #strong[Union mode.] The same pattern under
  `FROM <http://www.w3.org/ns/solid/sparql#union-default-graph>` returns neighbours from
  `/notes/a` and `/notes/b`, and never a term occurring only in `/private/journal`.
  (@sec-scope, @sec-authz)
+ #strong[Per-graph scoping.] Inside `GRAPH ?g`, `?g` binds only to readable resources, and
  each binding's solutions are the top-#emph[k] within that resource alone. (@sec-scope)
+ #strong[No under-fill.] With #emph[k] set below the number of eligible entries, and with a
  `/private/journal` entry that outranks every readable entry, the pattern still returns
  exactly #emph[k] solutions. (@sec-modes)
+ #strong[Unreadable seed equals absent seed.] A `vec:` pattern seeded on a term occurring
  only in `/private/journal`, and the same pattern seeded on a term occurring nowhere, both
  return zero solutions with indistinguishable responses — no error, no distinct status.
  (@sec-seed, @sec-nondisclosure)
+ #strong[Unreadable descriptor.] Where #emph[R] can read `/notes/a` but not the bound
  index's descriptor, a `vec:` pattern still returns `/notes/a`'s neighbours, and the service
  description served to #emph[R] names neither the descriptor nor the vector features. Where
  #emph[R] can read no resource in the index's scope, the same pattern returns zero solutions,
  indistinguishable from the same request against a service with no bound index. That #emph[R]
  can, from this behaviour, tell that an index is bound — and, by varying a literal query
  vector's dimension, at what dimension — is within SV-ND-4's stated exception and is not a
  conformance failure; disclosing any other descriptor fact is. (@sec-nondisclosure,
  @sec-discovery)
+ #strong[Wrong-dimension literal vector.] A literal query vector whose dimension differs
  from the bound index's returns zero solutions with the same response as one issued against
  a service with no bound index — not the sibling document's query error. (@sec-optionality)
+ #strong[Invariance.] Two storages presenting #emph[R] with the same #emph[readable entry
  set] but differing arbitrarily in `/private/` return identical results for every query
  #emph[R] can issue. Stating the scenario over readable #emph[resources] instead would be
  wrong: two storages with identical readable resources may have been ingested differently,
  and may then differ legitimately. (@sec-invariance)
+ #strong[Currency.] After a literal is removed from `/notes/a` and before the index is
  rebuilt, no `vec:` pattern binds that literal from `/notes/a` — even if the same literal
  is still present in `/notes/b`, in which case `/notes/b`'s entry still binds it, and the
  result count is filled from the remaining current entries rather than short. (@sec-currency)
+ #strong[Revocation.] After #emph[R]'s read access to `/notes/b` is revoked, the next
  request returns no term sourced from `/notes/b`. (@sec-authz)
+ #strong[Ingestion authorization.] A storage server rejects an entry batch from an agent
  without read access to `/notes/a` whose declared source resource is `/notes/a`.
  (@sec-ingest)
+ #strong[Ingestion validation.] An entry batch containing a wrong-dimension vector, a
  non-finite value, a blank-node term, a duplicate (source, term) pair, or a provenance
  mismatch is rejected whole, with no partial application. (@sec-ingest)
+ #strong[No query-endpoint mutation.] Every index-management operation attempted through
  the query endpoint is refused, with no side effects. (@sec-crud)
+ #strong[Description hygiene.] The service description exposes no index inventory, no entry
  counts, and no index descriptor #emph[R] cannot read. (@sec-discovery)
+ #strong[Deleted index.] After the descriptor is deleted, `vec:` patterns return zero
  solutions rather than errors. (@sec-binding)
+ #strong[Mask composition.] A `vec:` pattern whose neighbour variable is also constrained by
  an ordinary triple pattern returns the top-#emph[k] within the intersection of the eligible
  entry set and that constraint — filled to #emph[k] whenever #emph[k] such entries exist,
  and never containing an entry failing either. (@sec-masks)
+ #strong[Seed determinism.] Where the seed term has readable entries in both `/notes/a` and
  `/notes/b`, two independent conforming implementations return the same result set.
  (@sec-seed)
+ #strong[Tie determinism.] Where two entries with different source resources tie on score at
  the #emph[k]-th position, the one admitted is the one whose (source resource, term) key is
  least, on every implementation. (@sec-entry)
+ #strong[Approximation advertised.] A service that may answer approximately advertises
  `solid-sparql:ApproximateVectorSearch`; one that always evaluates exactly does not.
  (@sec-approx, @sec-discovery)

== Conformance suite commitment

#strong[No conformance suite accompanies this draft.] The commitment, and it is the
project's standing practice for this specification family rather than a new promise, is:
before this document advances beyond Unofficial Proposal Draft, the scenarios above are to
be published as a runnable manifest in the same shape as the base specification's
query-semantics suite, together with a machine-readable normative-statement companion
#cite("SPEC-COMPANION") pairing every `SV-` assertion with a test case or an honest
recorded gap. Any implementation this project builds contributes its spec tests
#strong[upstream], to the repository that owns the base specification, rather than keeping
them in-tree — the same rule under which the base document's own suite is vendored here and
run against the engine. An assertion that cannot be paired with a test will be reworded
until it can be, or demoted to informative.

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("RFC8288", [Nottingham, M. #emph[Web Linking]. RFC 8288, IETF, October 2017.]),
  ("SOLID-SPARQL-QUERY", [Wright, J. (ed.) #emph[Access-Controlled SPARQL Query over a Solid
    Pod]. Editor's Draft. https://github.com/jeswr/solid-sparql-query. (Read for this draft
    in the copy vendored at
    `crates/sparq-solid/tests/conformance/solid-sparql-query/spec.html`.)]),
  ("SPARQL-VEC-GENAI", [The sparq project. #emph[SPARQL Vector & GenAI Extension].
    Unofficial Proposal Draft. https://sparq.jeswr.org/specs/sparql-vector-genai.]),
  ("SOLID-PROTOCOL", [Capadisli, S.; Berners-Lee, T.; Verborgh, R.; Kjernsmo, K. (eds.)
    #emph[Solid Protocol]. Solid Community Group.
    https://solidproject.org/TR/protocol.]),
  ("WAC", [Solid Community Group. #emph[Web Access Control].
    https://solidproject.org/TR/wac.]),
  ("ACP", [Solid Community Group. #emph[Access Control Policy].
    https://solidproject.org/TR/acp.]),
  ("SPARQL11-QUERY", [Harris, S.; Seaborne, A. (eds.) #emph[SPARQL 1.1 Query Language].
    W3C Recommendation, 21 March 2013. https://www.w3.org/TR/sparql11-query/.]),
  ("SPARQL11-PROTOCOL", [Feigenbaum, L.; Williams, G. T.; Clark, K. G.; Torres, E. (eds.)
    #emph[SPARQL 1.1 Protocol]. W3C Recommendation, 21 March 2013.
    https://www.w3.org/TR/sparql11-protocol/.]),
  ("SPARQL11-SERVICE-DESCRIPTION", [Williams, G. T. (ed.) #emph[SPARQL 1.1 Service
    Description]. W3C Recommendation, 21 March 2013.
    https://www.w3.org/TR/sparql11-service-description/.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) #emph[RDF 1.1 Concepts
    and Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("HNSW", [Malkov, Y.; Yashunin, D. #emph[Efficient and Robust Approximate Nearest Neighbor
    Search Using Hierarchical Navigable Small World Graphs]. IEEE TPAMI, 2020.]),
  ("DiskANN", [Subramanya, S. J. et al. #emph[DiskANN: Fast Accurate Billion-point Nearest
    Neighbor Search on a Single Node]. NeurIPS, 2019.]),
  ("ACORN", [Patel, L.; Kraft, P.; Guestrin, C.; Zaharia, M. #emph[ACORN: Performant and
    Predicate-Agnostic Search Over Vector Embeddings and Structured Data]. SIGMOD, 2024.]),
  ("FILTERED-DISKANN", [Gollapudi, S. et al. #emph[Filtered-DiskANN: Graph Algorithms for
    Approximate Nearest Neighbor Search with Filters]. WWW, 2023.]),
  ("EMB-INVERSION", [Morris, J. X.; Kuleshov, V.; Shmatikov, V.; Rush, A. M. #emph[Text
    Embeddings Reveal (Almost) As Much As Text]. EMNLP, 2023.]),
  ("EMB-LEAKAGE", [Song, C.; Raghunathan, A. #emph[Information Leakage in Embedding Models].
    ACM CCS, 2020.]),
  ("SPEC-COMPANION", [Wright, J. #emph[spec-companion: a vocabulary, SHACL shapes, and
    validator for machine-readable normative-statement companions].
    https://github.com/jeswr/spec-companion.]),
))
