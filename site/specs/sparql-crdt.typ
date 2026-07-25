// [GPT-5.6] sq-tag1q.4 — SPARQL-CRDT: Conflict-Free Replicated RDF Datasets under
// SPARQL Update, an Unofficial Proposal Draft.
//
// GREENFIELD STATUS. sparq has no SPARQL-CRDT implementation at this revision. Every
// normative statement is therefore rendered inside an explicitly labelled Proposal block.
// The proposal identifiers are stable, testable requirements for the opt-in implementation
// beads that consume this document; none describes behaviour shipped today.
//
// CLAIM BOUNDARY. The design establishes strong eventual consistency of the materialised
// RDF quad set under the assumptions in the convergence section. It does not claim serialisability,
// equivalence of distributed SPARQL Update intentions, constraint preservation, Byzantine
// tolerance, or any stronger consistency property.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "SPARQL-CRDT: Conflict-Free Replicated RDF Datasets under SPARQL Update")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

// All requirements are proposals until the opt-in sparq-crdt implementation lands. Keeping
// the label at each assertion site makes the greenfield status visible in both HTML and PDF.
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
  This document proposes a replicated RDF #emph[dataset] data type and a mapping from a
  deliberately scoped SPARQL 1.1 Update profile to joinable CRDT deltas. Each concrete quad
  addition receives a globally unique #emph[dot]; removals carry the dots observed at the
  update's origin; and replicas merge a per-quad dot store plus causal context using an
  associative, commutative, and idempotent join. Concurrent addition therefore wins over a
  removal that did not observe it. Named graphs are part of each quad key.

  `INSERT DATA` and `DELETE DATA` compile directly to add and observed-remove deltas.
  `DELETE/INSERT ... WHERE` is evaluated once against one origin snapshot and compiled to
  concrete quad-and-dot effects; receivers never re-evaluate its graph pattern. This gives
  strong eventual consistency of the materialised quad set when valid deltas are eventually
  delivered. It does #strong[not] make concurrently evaluated update intentions equivalent,
  serialisable, or constraint-preserving.

  Blank-node labels cannot identify the same node across replicas. This proposal therefore
  skolemises blank nodes at the origin boundary into globally unique, stable IRIs and never
  transports blank-node labels. Deltas and causal metadata live in a canonical out-of-band
  JSON journal/sidecar rather than in application RDF. Three independent conformance classes
  are defined: replica, delta-relay, and origin-evaluator. The entire normative surface is
  labelled #emph[Proposal] because no SPARQL-CRDT code exists in sparq at this revision.
]

#sotd()

#intro-section("implementation-standing", "Implementation standing")[
  #strong[Greenfield proposal.] This document is the normative design input to a future,
  opt-in `sparq-crdt` capability. No current sparq crate claims any conformance class in this
  document. Every requirement appears in an amber #emph[Proposal] block and remains a proposal
  until an implementation and its conformance fixtures land. Informative explanations,
  examples, prior art, and proof sketches outside those blocks are not requirements.
]

= Introduction <sec-introduction>

RDF graphs are mathematical sets, but deployed SPARQL services expose mutable graph stores:
the same triple may be inserted, removed, and reinserted, and an RDF dataset distinguishes the
default graph from named graphs. Offline-first and multi-writer deployments need those edits
to merge without electing one writer or imposing a total order on every operation.

The replicated value in this proposal is the #emph[visible set of quads], not a register
holding a serialised graph and not a replicated SPARQL program. The distinction matters.
Set membership can be made conflict-free with observed-remove semantics. A pattern update,
by contrast, first asks a query whose answer depends on the local snapshot. Running the same
source text at different replicas can select different quads, so the source operation itself
is not a commutative CRDT operation. @sec-update fixes the replication boundary at the concrete
quad effects computed at one origin.

== Goals and non-goals

The design has four goals:

- model the default graph and every named graph in one replicated RDF dataset;
- preserve causality for exact-quad additions and removals without wall-clock arbitration;
- accept duplicate, reordered delta delivery and permit state/snapshot merge; and
- give implementers concrete state, join, update-compilation, wire, journal, and conformance
  rules.

The following are outside this revision: graph-slot existence independent of graph contents;
SPARQL graph-management operations; inference closure replication; SHACL or ontology
constraint repair; access-control policy; Byzantine or malicious-replica tolerance; and a
total order or serial transaction history. Excluding them narrows the claim, not the ability
of an application to layer such mechanisms around this data type.

= Terminology, scope, and proposal status <sec-conformance>

== Requirement keywords

#proposal("CRDT-STATUS-1")[
  The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
  #strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED],
  #strong[MAY], and #strong[OPTIONAL] in Proposal blocks are to be interpreted as described
  in #cite("RFC2119") and #cite("RFC8174") when, and only when, they appear in all capitals.
  An uppercase requirement outside a Proposal block has no normative force in this revision;
  this source intentionally contains none.
]

== Terms

- A #dfn[dataset identifier] is an absolute IRI naming one independently replicated dataset.
- A #dfn[replica identifier] is an opaque, globally unique byte string allocated to one
  durable counter lineage.
- A #dfn[quad] is `(subject, predicate, object, graph)`, where `graph = default` identifies
  the default graph and an IRI identifies a named graph.
- A #dfn[dot] is `(replica identifier, counter)`. It names one quad-addition event.
- A #dfn[causal context] is a compact set of dots that a replica has observed, including
  additions that are no longer visible.
- A #dfn[dot store] maps each quad to the non-empty set of live dots currently asserting it.
- A #dfn[data delta] is a fragment of the same dot-store/causal-context semilattice as full
  replica state. Joining it has the same visible effect as the origin mutation.
- An #dfn[origin snapshot] is the immutable visible dataset and causal context against which
  one SPARQL Update operation is evaluated.
- An #dfn[envelope] is the canonical out-of-band representation of one atomic origin delta.
- #dfn[Concurrent] means neither event is in the causal past of the other. It does not mean
  merely that wall-clock intervals overlap.
- #dfn[Strong eventual consistency] (SEC) means that replicas which have incorporated the
  same valid deltas have equivalent states and therefore the same visible quad set, and that
  quiescent replicas converge when every valid delta is eventually incorporated.

== RDF and SPARQL profile

#proposal("CRDT-SCOPE-1")[
  Version 1 of this proposal #strong[MUST] use RDF 1.1 term equality and the RDF dataset model
  #cite("RDF11-CONCEPTS"). Its SPARQL surface #strong[MUST] comprise `INSERT DATA`,
  `DELETE DATA`, and the `DELETE/INSERT ... WHERE` family, including its `INSERT ... WHERE`,
  `DELETE ... WHERE`, and `DELETE WHERE` abbreviations, as defined by
  #cite("SPARQL11-UPDATE"). A conforming origin-evaluator #strong[MUST NOT] silently accept an
  update form whose CRDT mapping is not defined in @sec-update.
]

#proposal("CRDT-SCOPE-2")[
  The replicated value #strong[MUST] be a set of quads. It #strong[MUST NOT] represent the
  existence of an empty named-graph slot. Consequently, `CREATE`, `DROP`, `CLEAR`, `LOAD`,
  `COPY`, `MOVE`, and `ADD` are outside this profile and #strong[MUST] be rejected at this
  interface. A later profile may compile some of them to concrete quad deltas, but it cannot
  change the meaning of a version-1 envelope.
]

= Prior art and design rationale <sec-prior-art>

#emph[This section is informative.]

== SU-Set and Live Linked Data

Ibáñez, Skaf-Molli, Molli, and Corby introduced SU-Set, an operation-based observed-remove
set for RDF graphs updated with SPARQL Update #cite("SUSET"). It assigns unique identifiers to
inserted triples, evaluates pattern updates at the source, and sends the concrete deleted and
inserted triple sets downstream. That is the essential semantic boundary adopted here.

SU-Set relies on causal delivery and scopes itself to RDF graphs: its published treatment does
not solve blank nodes or named graphs. This proposal retains its evaluate-at-origin insight,
extends the set element from triple to quad, takes an explicit skolemisation position, and
uses delta-state join plus causal dot contexts so duplicate and reordered delivery is safe.

== m-ld

m-ld implements an RDF graph as a shared data type based directly on SU-Set
#cite("MLD-PAPER"). Its clones use logical clocks for causal ordering and keep an operation
journal so a peer can catch up after missing messages. m-ld also separates graph convergence
from higher-level vocabulary constraints: a converged RDF graph is not automatically a
semantically valid application object.

The lesson carried forward here is the durable journal, but with a different algebraic role:
an envelope is a joinable delta, so its duplicate delivery is harmless and its order is not a
precondition for the visible dataset to converge. Constraints remain explicitly outside the
SEC claim.

== NextGraph

NextGraph documents a graph CRDT based on the SU-Set/OR-set model and transports CRDT
transactions in repository branches whose commits form a causal DAG #cite("NEXTGRAPH-CRDT")
#cite("NEXTGRAPH-PROTOCOL"). It demonstrates that RDF CRDT operations fit a local-first
repository and commit/journal architecture.

This proposal does not adopt NextGraph's repository, membership, encryption, or consensus
protocols. It borrows the clean separation between a dataset branch identity, an immutable
delta record, and a relay that moves records without re-running application operations.

== Delta-state CRDTs, causal contexts, and dot stores

Delta-state CRDTs make each mutator return a small state fragment from the same join
semilattice as full state. The fragment can be merged idempotently over an unreliable channel
while retaining state-based convergence #cite("DELTA-CRDT"). Dotted causal CRDTs separate a
payload's live dots from a causal context that remembers observed dots; version-vector
prefixes plus a small dot cloud compress that context.

This proposal instantiates that pattern per RDF quad. A remove delta contains no replacement
value: it advances the context over exactly the live dots observed for that quad. The join can
then distinguish an observed dot, which the remove suppresses, from a concurrent add dot,
which survives.

== Why a plain last-writer-wins value is insufficient

A last-writer-wins register for the whole dataset discards unrelated concurrent edits. A
last-writer-wins bit per quad can converge, but it requires a total timestamp/tie-break policy,
turns clock or identifier order into user-visible conflict resolution, and cannot distinguish
two independent concurrent assertions from a delete that observed only one of them. It also
retains delete timestamps indefinitely or risks resurrection after delayed delivery.

The required semantics is causal rather than chronological: a removal suppresses the add
events it actually observed, and a concurrent add survives. Dots express that relation
directly. They do not make pattern-update intentions commutative; they make the concrete quad
effects mergeable.

= Replicated RDF dataset model <sec-data-model>

== Quad identity and named graphs

#proposal("CRDT-DATA-1")[
  A replica #strong[MUST] key membership by the complete quad `(s, p, o, g)`. The graph
  component #strong[MUST] be either the distinguished `default` value or an absolute IRI.
  Equal triples in the default graph and in a named graph, or in two different named graphs,
  #strong[MUST] be distinct CRDT elements. RDF-term and graph-name comparison #strong[MUST]
  use term equality, not value equality or serialised-string equality.
]

The visible RDF dataset is obtained by dropping the dot sets: every dot-store key whose value
is non-empty contributes its quad. Thus concurrent duplicate assertions remain one RDF quad
while their causal identities remain independently removable.

== Blank-node identity: mandatory skolemisation

Blank-node labels are syntax-local and are neither persistent nor portable RDF identifiers
#cite("RDF12-CONCEPTS"). Reusing `_:` labels between replicas would sometimes merge unrelated
existential nodes; renaming every delivery would sometimes split one origin node used by
several quads. Both choices corrupt RDF identity.

#proposal("CRDT-BNODE-1")[
  Every blank node entering the replicated dataset #strong[MUST] be replaced at the origin
  boundary by one globally unique Skolem IRI before query evaluation, quad comparison, delta
  creation, or journal persistence. The origin #strong[MUST] preserve one-to-one identity
  within the blank-node scope: repeated use of one label in one input scope maps to one IRI;
  distinct blank nodes map to distinct IRIs. A version-1 dot store or envelope #strong[MUST
  NOT] contain a blank-node term in any quad position.
]

#proposal("CRDT-BNODE-2")[
  Each dataset #strong[MUST] configure a controlled absolute `skolemBase` whose namespace is
  reserved from client-supplied IRIs. A minted IRI #strong[MUST] append an encoding of the
  dataset identifier, origin replica identifier, origin journal sequence, and a request-local
  blank-node index to that base. A Web-visible deployment #strong[SHOULD] use the
  `/.well-known/genid/` convention recommended by RDF Concepts. Replicas #strong[MUST NOT]
  de-skolemise these IRIs while the dataset remains replicated, because doing so would discard
  the portable identity on which quad equality depends.
]

#proposal("CRDT-BNODE-3")[
  `INSERT DATA` blank nodes #strong[MUST] receive a fresh mapping for each origin operation, as
  required by SPARQL Update. Blank nodes instantiated by an `INSERT` template #strong[MUST]
  receive a fresh mapping per solution and template label. Variables in a delete template
  #strong[MAY] bind existing Skolem IRIs; this is ordinary IRI equality, not blank-node label
  matching. An origin-evaluator #strong[MUST] reject a client IRI under `skolemBase` unless it
  is already a known minted term.
]

Skolemisation changes an existential blank node into a named witness, as RDF Concepts notes.
That semantic trade-off is explicit and is preferred here to ambiguous cross-replica
identity. Applications that cannot accept it are outside this profile.

== Dots and causal contexts

#proposal("CRDT-DOT-1")[
  A dot #strong[MUST] be a pair `(replica, counter)` with a non-zero unsigned counter. A
  replica #strong[MUST] durably advance its local dot counter before publishing the associated
  addition and #strong[MUST NOT] reuse a dot. If durable replica state is lost or cloned, the
  process #strong[MUST] allocate a new replica identifier rather than resume an uncertain
  counter lineage.
]

A causal context `C` denotes a set of dots. Its required compact representation is:

- `clock[r] = n`: every dot `(r, 1)` through `(r, n)` is in `C`; and
- `cloud`: the finite set of dots above, or in gaps outside, those contiguous prefixes.

Normalisation repeatedly moves `(r, clock[r] + 1)` from the cloud into the clock. Context
union is set union followed by normalisation. `contains(C, d)` tests membership in the denoted
set, not merely in the cloud.

#proposal("CRDT-CTX-1")[
  Implementations #strong[MUST] preserve the exact denotation of a causal context across
  normalisation, serialisation, restart, snapshot, and merge. They #strong[MAY] use another
  lossless internal encoding, but the interchange encoding #strong[MUST] expose the
  `clock`-plus-`cloud` form in @sec-interchange. A context entry #strong[MUST NOT] be discarded merely
  because its associated quad is no longer visible.
]

== Replica state and invariants

A full state is `State = (store, context)`:

- `store[q]` is a finite, non-empty set of live dots for quad `q`; and
- `context` is the causal context of all add dots observed by the replica, whether live or
  removed.

#proposal("CRDT-STATE-1")[
  A well-formed state #strong[MUST] satisfy all of the following invariants: every dot in
  `store` is contained in `context`; one dot occurs under at most one quad; no store entry has
  an empty dot set; every quad is blank-node-free and valid under `CRDT-DATA-1`; and every
  locally minted dot obeys `CRDT-DOT-1`. A replica #strong[MUST] fail closed on a state or
  delta that violates an invariant.
]

#proposal("CRDT-STATE-2")[
  The materialised dataset view #strong[MUST] contain exactly the keys `q` for which
  `store[q]` is non-empty. Causal metadata and journal records #strong[MUST NOT] appear as
  application RDF quads or SPARQL query results.
]

== State and delta join

For input states `A = (SA, CA)` and `B = (SB, CB)`, compute each quad's retained dots and the
union context as follows. A missing store entry denotes the empty set.

```text
join(A, B):
  C = normalize(union(CA, CB))
  for q in union(keys(SA), keys(SB)):
    both       = intersection(SA[q], SB[q])
    only_a_new = {d in SA[q] where not contains(CB, d)}
    only_b_new = {d in SB[q] where not contains(CA, d)}
    live       = union(both, only_a_new, only_b_new)
    if live is not empty: S[q] = live
  return (S, C)
```

The first term preserves a dot present on both sides. The second and third preserve dots that
the other side has not observed. A dot absent from one store but present in that side's
context was observed and removed, so it is not retained.

#proposal("CRDT-JOIN-1")[
  Full states and data deltas #strong[MUST] use the join above. Equality of contexts
  #strong[MUST] compare their denoted dot sets after normalisation. Implementations
  #strong[MAY] optimise indexes or iteration but #strong[MUST] produce the same store and
  context denotation. Merge #strong[MUST] be atomic with respect to readers.
]

== Primitive delta mutators

`add(q)` reserves a fresh dot `d`, and returns the delta state `({q: {d}}, {d})`.
`remove(q)` reads `D = store[q]` from one origin snapshot and returns `(empty, D)`, where `D`
is encoded as a causal context. Removing an absent quad therefore returns the bottom/empty
delta. Batch add/remove composes these fragments with `join`.

#proposal("CRDT-MUT-1")[
  One concrete insertion intent #strong[MUST] mint exactly one fresh dot for each distinct
  inserted quad, even if that quad is already visible at the origin. Multiple identical
  instantiations within one SPARQL operation #strong[MUST] be deduplicated before dot minting.
  This metadata-only assertion is what makes that insertion win over a concurrent remove;
  it does not create RDF multiplicity.
]

#proposal("CRDT-MUT-2")[
  One concrete removal #strong[MUST] place exactly the dots visible under its quad in the
  origin snapshot into the removal context. It #strong[MUST NOT] invent a wildcard, timestamp,
  or future-dot tombstone. Therefore, a causally later remove suppresses observed additions,
  while an unobserved concurrent addition survives.
]

#proposal("CRDT-MUT-3")[
  When one origin operation both deletes and inserts the same quad, its delta #strong[MUST]
  remove the old observed dots and carry a fresh live addition dot. The new dot #strong[MUST
  NOT] appear in that operation's removal context. Joining the delta therefore leaves the
  quad visible with the new dot.
]

= SPARQL Update compilation at the origin <sec-update>

== Common evaluation transaction

#proposal("CRDT-UPD-1")[
  An origin-evaluator #strong[MUST] execute each accepted SPARQL Update operation as one local
  transaction: (1) acquire an immutable origin snapshot `(view, context)`; (2) perform all
  SPARQL dataset selection, `WHERE` evaluation, template substitution, validation, and
  skolemisation against that snapshot; (3) compile the result to concrete add/remove delta
  components; (4) reserve a durable origin journal sequence and any add-dot counters; and
  (5) atomically persist the envelope, join it into local replica state, and record the journal
  sequence before acknowledging success. A failure before commit #strong[MUST] leave all
  three durable records unchanged.
]

The `basis` field in @sec-interchange records the data causal context of the immutable snapshot. It
is audit metadata and an evaluation provenance boundary. It is not joined as the delta's
removal context, because doing that would incorrectly remove every observed dot omitted from
the delta payload.

== INSERT DATA

#proposal("CRDT-UPD-INSERT-DATA-1")[
  For `INSERT DATA`, the origin-evaluator #strong[MUST] resolve the request's target graph for
  every triple, skolemise request blank nodes per `CRDT-BNODE-3`, deduplicate the resulting
  quads, and invoke `add(q)` once per distinct quad. Receivers #strong[MUST] apply only the
  concrete envelope; they #strong[MUST NOT] parse or re-execute the source request.
]

SPARQL permits an implementation to process an already-present triple with no visible action.
This profile chooses to mint a new invisible add dot, as `CRDT-MUT-1` requires, so an explicit
concurrent insertion has add-wins meaning.

== DELETE DATA

#proposal("CRDT-UPD-DELETE-DATA-1")[
  For `DELETE DATA`, the origin-evaluator #strong[MUST] resolve every ground triple to a quad,
  deduplicate those quads, and invoke `remove(q)` against the single origin snapshot. It
  #strong[MUST] reject blank nodes or variables as SPARQL 1.1 requires. A requested quad absent
  from the snapshot #strong[MUST] contribute no removal dots.
]

== DELETE/INSERT ... WHERE

The query pattern in a SPARQL `DELETE/INSERT` is not a commutative replicated operation. Two
offline replicas can evaluate the same `WHERE` text over different datasets and obtain
different solution multisets. Broadcasting the source and re-evaluating it would therefore
make replicas diverge.

#proposal("CRDT-UPD-WHERE-1")[
  The `WHERE` group graph pattern #strong[MUST] be evaluated exactly once at the origin using
  SPARQL 1.1 query semantics, including `WITH`, `USING`, and `USING NAMED` dataset selection.
  All delete and insert templates #strong[MUST] be instantiated from that one solution
  multiset using SPARQL 1.1 Update semantics. The evaluator #strong[MUST] then reduce the
  instantiated results to two concrete sets of quads, `D` and `I`; no solution binding or
  uninstantiated pattern is a replication operation.
]

#proposal("CRDT-UPD-WHERE-2")[
  For each quad in `D`, the origin-evaluator #strong[MUST] collect the live dots present in the
  origin snapshot. For each distinct quad in `I`, it #strong[MUST] mint one fresh add dot after
  applying `CRDT-BNODE-3`. It #strong[MUST] form one atomic delta whose store contains the new
  add dots and whose removal context contains the collected old dots. If a quad is in both
  sets, `CRDT-MUT-3` applies.
]

#proposal("CRDT-UPD-WHERE-3")[
  A replica or delta-relay receiving the resulting envelope #strong[MUST NOT] evaluate the
  original `WHERE` clause, recover update text, substitute bindings, or broaden a removal to
  newly matching quads. The only replicated intent is the envelope's concrete add dots and
  observed remove dots.
]

For example, origin A may evaluate `DELETE WHERE { ?s :status "stale" }` and observe one
matching quad. A concurrent origin B may add a second matching quad. A's envelope removes the
dots for the first quad only; B's unobserved add survives. Every replica converges on that
same outcome after both deltas arrive, but the result is not equivalent to interpreting A's
English-language intention as “remove all stale things globally.”

== Requests containing multiple operations and retries

#proposal("CRDT-UPD-BATCH-1")[
  Operations in one SPARQL Update request #strong[MUST] be evaluated in source order against
  the local result of preceding operations, as SPARQL requires. An implementation
  #strong[MAY] join their data-delta fragments into one envelope only if the joined fragment
  yields the same local state as that ordered evaluation and the entire request commits
  atomically. A later delete in the request may therefore remove dots added earlier in the
  request; a later add uses a fresh surviving dot.
]

#proposal("CRDT-UPD-RETRY-1")[
  The pair `(origin, sequence)` #strong[MUST] identify one envelope. An origin #strong[MUST]
  return the previously committed result when retrying a request under an already committed
  application idempotency key, and #strong[MUST NOT] mint replacement dots. Receivers
  #strong[MUST] treat byte-identical duplicate envelope identities idempotently and
  #strong[MUST] reject the same identity with different canonical bytes.
]

= Delta interchange and durable sidecar <sec-interchange>

== Out-of-band metadata decision

RDF 1.2 reifiers can describe statements related to triple terms
#cite("RDF12-CONCEPTS"). They are not selected for CRDT metadata here. A dot identifies one
add event for one #emph[quad], including its graph position, rather than merely the proposition
denoted by an RDF triple. Encoding dots, removals, clocks, and journal identities as ordinary
RDF would make replication bookkeeping visible to application queries, require reserved
graphs and filtering rules, and make the metadata itself subject to recursive update and
replication semantics.

#proposal("CRDT-WIRE-1")[
  Version-1 CRDT metadata #strong[MUST] be stored and exchanged out of band from application
  RDF as an append-only envelope journal plus an atomically checkpointed state sidecar. It
  #strong[MUST NOT] be encoded as asserted RDF, RDF 1.2 triple annotations, or reifier quads in
  the replicated dataset. A storage engine #strong[MUST] commit application indexes, CRDT
  state, and the journal record in one crash-consistent transaction or reconstruct all three
  from the committed journal before serving reads.
]

This is a semantic separation, not a claim that RDF cannot represent provenance. An
application may publish an informative RDF projection of journal events elsewhere, but that
projection is not the merge input and cannot replace the sidecar.

== Version-1 JSON envelope

The media type proposed for a single envelope is
`application/vnd.sparq-crdt-delta+json;version=1`. Its abstract JSON shape is:

```json
{
  "format": "sparq-crdt-delta/1",
  "dataset": "https://example.test/datasets/team",
  "epoch": "3",
  "origin": "cGVlci1h",
  "sequence": "42",
  "basis": {
    "clock": { "cGVlci1h": "87", "cGVlci1i": "19" },
    "cloud": [["cGVlci1i", "21"]]
  },
  "adds": [
    {
      "quad": "<https://ex/s> <https://ex/p> \"new\" <https://ex/g> .",
      "dot": ["cGVlci1h", "88"]
    }
  ],
  "removes": [
    {
      "quad": "<https://ex/s> <https://ex/p> \"old\" <https://ex/g> .",
      "dots": [["cGVlci1b", "19"]]
    }
  ]
}
```

`epoch` is the membership epoch in which the origin committed the operation. `origin` and
replica components use unpadded base64url. Epochs are unsigned decimal integers; sequence and
dot counters are non-zero unsigned decimal integers. All are encoded as JSON strings, avoiding
host-language JSON number limits. `quad` is one blank-node-free canonical N-Quads line. `basis`
is the complete data causal context observed before evaluation. The data delta joined at a
receiver is derived only as follows:

```text
delta.store   = each adds[i].quad -> { adds[i].dot }
delta.context = union(all adds[i].dot, all removes[i].dots)
```

#proposal("CRDT-WIRE-2")[
  An envelope #strong[MUST] contain exactly the fields shown above; `adds` and `removes`
  #strong[MAY] be empty but not both empty unless the origin operation committed a semantic
  no-op that still requires an idempotency record. Replica bytes #strong[MUST] use unpadded
  base64url per #cite("RFC4648"); `epoch` #strong[MUST] use its shortest decimal string;
  `sequence` and dot counters #strong[MUST] use their shortest non-zero decimal string; each
  quad #strong[MUST] use canonical N-Quads form per #cite("NQUADS") and #cite("RDFC10"); and
  the complete document #strong[MUST] use the JSON Canonicalization Scheme #cite("RFC8785")
  for hashing, equality, and transport.
]

#proposal("CRDT-WIRE-3")[
  `adds` #strong[MUST] be sorted by the UTF-8 bytes of `quad`, then replica bytes, then numeric
  counter. `removes` #strong[MUST] be sorted by `quad`; each `dots` array and `basis.cloud`
  #strong[MUST] be sorted by replica bytes then numeric counter. Arrays #strong[MUST] contain
  no duplicates. `basis.clock` keys are canonicalised by JCS. A decoder #strong[MUST] reject
  non-canonical input rather than normalise and accept a second byte representation of one
  envelope identity.
]

#proposal("CRDT-WIRE-4")[
  Before journal commit, a receiver #strong[MUST] validate the format version, dataset
  identifier, membership epoch, origin/sequence identity, canonical encoding, quad grammar,
  blank-node absence, dot uniqueness, and all state invariants. An envelope whose `epoch` is
  not the receiver's current membership epoch #strong[MUST] be rejected. An add dot
  #strong[MUST NOT] also occur in that envelope's remove set. If a received dot is already
  known under another quad, the envelope #strong[MUST] be rejected. Authentication and replica
  authorisation are deployment responsibilities outside the CRDT algebra; unauthorised
  envelopes #strong[MUST] be rejected before this validation path.
]

== Journal frontier, relay, and catch-up

Envelope identities have their own journal context, separate from the data-dot context:
`(origin replica, origin sequence)`. It uses the same `clock`-plus-`cloud` representation.
This separate namespace records delete-only and no-op envelopes, which mint no data dot.

#proposal("CRDT-JOURNAL-1")[
  Every replica and delta-relay #strong[MUST] durably record the canonical bytes associated
  with each accepted envelope identity and maintain an exact journal frontier. Peers
  #strong[MUST] exchange frontiers and transfer envelopes missing from the receiver. Delivery
  #strong[MAY] be duplicated or reordered; an implementation #strong[MUST NOT] require
  exactly-once transport or wall-clock order for convergence.
]

#proposal("CRDT-JOURNAL-2")[
  A full snapshot #strong[MUST] carry the dataset identifier, complete dot store, complete
  data causal context, and journal frontier under one format version. Joining snapshot state
  #strong[MUST] use `CRDT-JOIN-1`. A journal envelope #strong[MAY] be pruned only after a
  durable snapshot incorporates it and the relay's retention policy guarantees that a peer
  missing it can obtain an equal-or-newer snapshot. Pruning #strong[MUST NOT] discard the
  causal context needed to suppress removed dots.
]

The envelope epoch is an admission discriminator; no protocol for establishing or advancing
membership epochs, or for causal stability, is standardised here. A deployment that cannot
prove a stronger safe-compaction condition retains the compressed context. Replica identifiers
are never recycled, so a retired replica cannot resurrect an old counter lineage.

= Convergence and intention-preservation boundary <sec-convergence>

== Assumptions

#proposal("CRDT-SEC-1")[
  The SEC claim in `CRDT-SEC-2` applies only when: (a) replicas operate on the same dataset
  identifier and format version; (b) replica identifiers and dots are globally unique and
  durably monotonic; (c) all accepted states and deltas satisfy the invariants; (d) every
  generated valid delta is eventually incorporated directly or through a state/snapshot join;
  (e) all replicas use the specified RDF term equality and skolemisation boundary; and
  (f) replicas are non-Byzantine with respect to dot minting and delta construction.
]

== Precise convergence claim

#proposal("CRDT-SEC-2")[
  Under `CRDT-SEC-1`, the replicated RDF dataset #strong[MUST] provide strong eventual
  consistency: any two replicas that have incorporated the same set of valid deltas have
  equivalent causal-context denotations, equivalent live dot stores, and exactly the same
  materialised quad set, independent of delivery order, grouping, or duplication. If updates
  cease and every valid delta is eventually incorporated, every replica's materialised quad
  set converges.
]

The proof obligation is the standard dotted-set argument. Context union is associative,
commutative, and idempotent. For each `(quad, dot)`, the join's membership decision depends
only on whether each operand carries the live dot or has observed it in its context; that
decision is likewise associative, commutative, and idempotent. The product over all quads is a
join-semilattice. Delta mutators return elements of that same semilattice, so grouping and
reordering cannot alter the least upper bound.

== Add-wins scope

#proposal("CRDT-INTENT-1")[
  For one exact quad, an addition causally observed by a later removal #strong[MUST] be absent
  after both effects merge. An addition concurrent with a removal #strong[MUST] remain live
  after both effects merge. Concurrent removals are idempotent. These are the complete
  intention-preservation claims of this proposal.
]

#proposal("CRDT-INTENT-2")[
  An implementation #strong[MUST NOT] describe dataset SEC as preservation of the source
  intention of a pattern-based update. It #strong[MUST NOT] claim linearizability,
  serialisability, global snapshot isolation, equivalence to evaluating every source update
  against one shared graph store, preservation of SHACL/OWL/application invariants, or
  convergence of instantaneous query answers before all deltas arrive. Such properties
  require coordination or a separately specified higher-level CRDT.
]

In particular, the system converges on the union/join of #emph[concrete effects]. Two origins
may validly compile identical update text into different effects. The effects converge; the
intents are not retroactively reconciled.

= Conformance classes <sec-classes>

Conformance is claimed independently for the following three roles. One product may implement
more than one role, but a claim for one does not imply another.

== Replica

#proposal("CRDT-CLASS-REPLICA-1")[
  A conforming #dfn[replica] #strong[MUST] implement the quad/skolem boundary for all imported
  state, the state invariants, dot/context representation, state and delta join, atomic local
  apply, canonical envelope validation, durable journal identity, snapshot import/export, and
  the visible-dataset projection. It #strong[MUST] pass the replica fixtures in @sec-tests.
  A replica need not parse or evaluate SPARQL Update.
]

== Delta-relay

#proposal("CRDT-CLASS-RELAY-1")[
  A conforming #dfn[delta-relay] #strong[MUST] validate envelope identity and canonical bytes,
  durably map each identity to exactly one byte string, exchange exact journal frontiers,
  deliver missing envelopes without rewriting them, and provide a snapshot recovery path when
  retention has pruned an envelope. It #strong[MUST] tolerate duplicate and reordered network
  delivery. A relay need not parse N-Quads beyond validation and need not materialise RDF.
]

== Origin-evaluator

#proposal("CRDT-CLASS-ORIGIN-1")[
  A conforming #dfn[origin-evaluator] #strong[MUST] implement the SPARQL profile and every
  origin transaction, blank-node, concrete-delta, batching, and retry rule in @sec-data-model,
  @sec-update, and @sec-interchange. It #strong[MUST] be connected to a conforming replica transaction so the snapshot,
  envelope, local join, and acknowledgement satisfy `CRDT-UPD-1`. It #strong[MUST NOT]
  delegate pattern evaluation to receiving replicas.
]

= Conformance fixtures and assertion index <sec-tests>

== Minimum fixture families

#proposal("CRDT-TEST-1")[
  A conformance suite #strong[MUST] test at least: add round-trip; observed removal; concurrent
  add versus remove in both delivery orders; duplicate delivery; all permutations of a small
  delta set; default-versus-named graph separation; two named graphs containing the same
  triple; state/snapshot join; context clock/cloud normalisation; dot reuse rejection;
  canonical-envelope rejection; and crash/retry idempotency. Every convergence fixture
  #strong[MUST] compare both the visible quad set and normalised causal metadata.
]

#proposal("CRDT-TEST-2")[
  Origin-evaluator fixtures #strong[MUST] include: `INSERT DATA` with repeated and fresh blank
  nodes; `DELETE DATA` of present and absent quads; `DELETE/INSERT ... WHERE` with overlap
  between delete and insert sets; `WITH` and `USING NAMED`; multiple operations in one request;
  and two divergent origin snapshots where the same `WHERE` source matches different quads.
  The last fixture #strong[MUST] demonstrate convergence after exchanging only the two
  concrete envelopes and #strong[MUST] reject receiver-side re-evaluation.
]

The fixture corpus is not part of this greenfield document. These requirements pin what the
implementation beads must supply before any sparq build claims conformance.

== Assertion groups

#table(
  columns: 3,
  align: (left, left, left),
  table.header[Group][Sections][Subject],
  [`CRDT-STATUS`, `CRDT-SCOPE`], [@sec-conformance], [proposal status and supported profile],
  [`CRDT-DATA`, `CRDT-BNODE`, `CRDT-DOT`, `CRDT-CTX`, `CRDT-STATE`, `CRDT-JOIN`, `CRDT-MUT`],
    [@sec-data-model], [dataset state, identity, merge, and primitive deltas],
  [`CRDT-UPD`], [@sec-update], [SPARQL Update evaluate-at-origin compilation],
  [`CRDT-WIRE`, `CRDT-JOURNAL`], [@sec-interchange], [canonical sidecar, relay, and recovery],
  [`CRDT-SEC`, `CRDT-INTENT`], [@sec-convergence], [precise guarantees and exclusions],
  [`CRDT-CLASS`], [@sec-classes], [replica, delta-relay, and origin-evaluator classes],
  [`CRDT-TEST`], [@sec-tests], [minimum conformance fixture families],
)

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997. https://www.rfc-editor.org/rfc/rfc2119.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017. https://www.rfc-editor.org/rfc/rfc8174.]),
  ("RFC4648", [Josefsson, S. #emph[The Base16, Base32, and Base64 Data Encodings]. RFC 4648,
    IETF, October 2006. https://www.rfc-editor.org/rfc/rfc4648.]),
  ("RFC8785", [Rundgren, A.; Jordan, B.; Erdtman, S. #emph[JSON Canonicalization Scheme
    (JCS)]. RFC 8785, IETF, June 2020. https://www.rfc-editor.org/rfc/rfc8785.]),
  ("RDF11-CONCEPTS", [Cyganiak, R.; Wood, D.; Lanthaler, M. (eds.) #emph[RDF 1.1 Concepts
    and Abstract Syntax]. W3C Recommendation, 25 February 2014.
    https://www.w3.org/TR/rdf11-concepts/.]),
  ("RDF12-CONCEPTS", [Kellogg, G.; Champin, P.-A.; Longley, D. (eds.) #emph[RDF 1.2 Concepts
    and Abstract Data Model]. W3C Recommendation, 2026.
    https://www.w3.org/TR/rdf12-concepts/.]),
  ("NQUADS", [Carothers, G. (ed.) #emph[RDF 1.1 N-Quads: A line-based syntax for RDF
    datasets]. W3C Recommendation, 25 February 2014. https://www.w3.org/TR/n-quads/.]),
  ("RDFC10", [Longley, D.; Kellogg, G. (eds.) #emph[RDF Dataset Canonicalization (RDFC-1.0)].
    W3C Recommendation. https://www.w3.org/TR/rdf-canon/.]),
  ("SPARQL11-UPDATE", [Gearon, P.; Passant, A.; Polleres, A. (eds.) #emph[SPARQL 1.1
    Update]. W3C Recommendation, 21 March 2013.
    https://www.w3.org/TR/sparql11-update/.]),
  ("SUSET", [Ibáñez, L. D.; Skaf-Molli, H.; Molli, P.; Corby, O. #emph[Live Linked Data:
    Synchronising Semantic Stores with Commutative Replicated Data Types]. International
    Journal of Metadata, Semantics and Ontologies 8(2), 119–133, 2013.
    https://doi.org/10.1504/IJMSO.2013.056605.]),
  ("MLD-PAPER", [Svarovsky, G. #emph[m-ld: Realtime Information Sharing with RDF].
    SEMANTiCS 2021, CEUR Workshop Proceedings 2941. https://ceur-ws.org/Vol-2941/paper1.pdf.]),
  ("NEXTGRAPH-CRDT", [NextGraph. #emph[Conflict-Free Replicated Data Types (CRDT)].
    https://docs.nextgraph.org/en/framework/crdts/.]),
  ("NEXTGRAPH-PROTOCOL", [NextGraph. #emph[Sync Protocol].
    https://docs.nextgraph.org/en/protocol/.]),
  ("DELTA-CRDT", [Almeida, P. S.; Shoker, A.; Baquero, C. #emph[Delta State Replicated Data
    Types]. Journal of Parallel and Distributed Computing 111, 162–173, 2018.
    https://doi.org/10.1016/j.jpdc.2017.08.003.]),
))
