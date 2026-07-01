// [OPUS-4.8] sq-rvgr2.1 — the ONE placeholder draft that exercises the whole /specs pipeline
// end-to-end (head, abstract, Status-of-This-Document box, numbered sections, an RFC 2119
// conformance section, a worked example with a term definition + note + table + code, and a
// references section) so the infra is proven. The real specification CONTENT lands in the
// sibling beads sq-rvgr2.2 / .3 / .4 (zkSPARQL, MPC-SPARQL, SPARQL vector/GenAI extensions);
// this template deliberately describes the PUBLICATION PROCESS, not any one domain.

#import "_lib/spec.typ": spec-head, sotd, intro-section, references, dfn, note, cite

#set document(title: "SPARQ Proposed Specifications — Template")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#spec-head()

#intro-section("abstract", "Abstract")[
  This document is a template and worked example for the sparq project's Proposed
  Specifications series. It defines nothing normative about sparq itself; instead it
  demonstrates the structure every sparq Unofficial Proposal Draft follows — a
  Status-of-This-Document notice, numbered sections, a conformance section keyed to the
  RFC 2119 terms, a worked example, and a references list — and proves the build pipeline that
  turns a single source into both this in-site render and a downloadable PDF.
]

#sotd()

= Introduction

The sparq project develops several capabilities whose interfaces are worth writing down in a
reviewable, familiar form: a specification. This series hosts those write-ups as W3C-style
#dfn[Unofficial Proposal Drafts]. Each draft is authored once and published as both an in-site
document and a PDF from the same source, so the two can never disagree.

A draft in this series is a proposal for discussion. It records a design so reviewers can read,
critique, and cite it — it is explicitly not a ratified specification, and adopting it confers
no guarantees.

= Terminology and conformance

The key words #strong[MUST], #strong[MUST NOT], #strong[REQUIRED], #strong[SHALL],
#strong[SHALL NOT], #strong[SHOULD], #strong[SHOULD NOT], #strong[RECOMMENDED], #strong[MAY],
and #strong[OPTIONAL] in this document are to be interpreted as described in
#cite("RFC2119") and #cite("RFC8174") when, and only when, they appear in all capitals, as
shown here.

A conforming draft in this series #strong[MUST] carry a Status-of-This-Document notice that
states it has no W3C standing, #strong[MUST] number its normative sections, and #strong[SHOULD]
provide both an in-site render and a downloadable PDF from a single source. A draft #strong[MAY]
include informative notes and worked examples; such material is non-normative.

= A worked example

This section exercises the elements a real draft uses. A #dfn[worked example] is an informative
illustration that a reader can follow without changing the meaning of the normative text.

The table below is a placeholder for the kind of structured detail a real draft carries — for
example an interface's fields, or a mapping between two vocabularies.

#table(
  columns: 2,
  align: (left, left),
  table.header[Element][Purpose in a draft],
  [Status notice], [States the draft's standing — here, that it has none.],
  [Numbered section], [Carries a normative or informative unit of the proposal.],
  [Conformance terms], [Bind the RFC 2119 keywords to their precise meaning.],
  [References], [Cite the documents a draft depends on.],
)

A draft #strong[MAY] embed short code samples, e.g. the query shape a proposal constrains:

```sparql
SELECT ?s ?p ?o WHERE { ?s ?p ?o } LIMIT 1
```

#note[
  This is an Unofficial Proposal Draft: everything in it is subject to change, and nothing in
  it should be relied upon as a settled guarantee. It exists to be reviewed and improved.
]

= References

#references((
  ("RFC2119", [Bradner, S. #emph[Key words for use in RFCs to Indicate Requirement Levels].
    RFC 2119, IETF, March 1997.]),
  ("RFC8174", [Leiba, B. #emph[Ambiguity of Uppercase vs Lowercase in RFC 2119 Key Words].
    RFC 8174, IETF, May 2017.]),
  ("ReSpec", [W3C. #emph[ReSpec — a tool for writing technical documents in the W3C's
    house style]. https://respec.org/.]),
))
