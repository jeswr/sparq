---
name: sparql-formal-semantics
description: Cheat-sheet for the formal semantics of the SPARQL fragment used in the ZKP-SPARQL paper — Pérez–Arenas–Gutiérrez algebra, RDF graph model, blank-node canonicalisation choices, fragment-scoping conventions. Use when writing operator specs, scoping which SPARQL constructs the paper supports, drafting semantics sections, or reconciling differences between the W3C SPARQL 1.1 rec and the paper's formalisation.
---

# SPARQL formal semantics cheat-sheet

A working reference for the formal semantics layer Jesse's paper
sits on. Citations matter — every claim here should be traceable to
either the W3C SPARQL 1.1 rec or to the Pérez–Arenas–Gutiérrez (PAG)
paper.

## Algebra at a glance

PAG models a SPARQL graph pattern as expressions built from:

- **Triple patterns** `(s, p, o)` where each position is a term or
  variable.
- **Basic graph patterns (BGPs)** as sets of triple patterns.
- **AndP1, P2** — join (compatibility on shared variables).
- **OptP1, P2** — left outer join (`OPTIONAL`).
- **UnionP1, P2** — set union of solution mappings.
- **FilterP, R** — selection by a built-in expression.

Solutions are partial functions `μ : V → Term` (called solution
mappings). The semantics `eval(G, P)` of pattern `P` over graph `G`
is a multiset of solution mappings.

## Compatibility

Two solution mappings `μ1`, `μ2` are *compatible* iff they agree on
every variable in `dom(μ1) ∩ dom(μ2)`. The join `μ1 ⨝ μ2` is
defined when they're compatible and equals `μ1 ∪ μ2`.

This single notion unifies BGP joining, `AND`, and the inner part of
`OPTIONAL`.

## Standard equivalences

Useful when re-shaping queries before circuit compilation:

- `(P1 AND P2) AND P3 ≡ P1 AND (P2 AND P3)` (associativity).
- `P1 AND P2 ≡ P2 AND P1` (commutativity, multiset).
- `(P1 UNION P2) AND P3 ≡ (P1 AND P3) UNION (P2 AND P3)`
  (distributivity).
- `OPTIONAL` is **not** associative or commutative — be careful.

## RDF graph model

For the paper's purposes, an RDF graph is a finite multiset of
triples. Triples have:

- **IRIs**, **literals** (with optional language tag, optional
  datatype IRI), and **blank nodes**.
- Blank nodes are *existential* — they have local scope.

## Blank-node canonicalisation

Two living standards:

- **URDNA2015** — the original RDF Dataset Canonicalisation
  algorithm; widely deployed.
- **URDNA2024** — the W3C-track successor; addresses some hash
  collisions and edge cases.

For the paper, **pick one and document it**. The choice flows
through to circuit-side encoding, signature schemes that sign
canonical N-Quads, and the Lean model.

## Fragment scoping — start small

Suggested opening fragment for the paper:

| Construct | In scope (v1)? | Notes |
| --- | --- | --- |
| BGP | yes | Foundation. |
| AND | yes | Joins. |
| FILTER (built-in expressions over literals) | yes (subset) | Equality, numeric `<`, `>`; defer regex. |
| UNION | maybe v1.1 | Multiset union — cheap if hash-committed. |
| OPTIONAL | v2 | Left-outer-join is genuinely tricky in-circuit. |
| Property paths | out of scope | Recursion needs a different proof technique. |
| Aggregates (COUNT, SUM, ...) | v2 | Sort or multi-set hash gadget. |
| ORDER BY / LIMIT | v2 | Relevant only if the verifier needs ordered output. |

This is a starting point — `sparql-semantics` owns the live version
in the paper.

## Where the paper deviates from the standard

When the paper's algebra deviates from W3C, document each deviation
in a **single** comparison table in the paper. Acceptable reasons:

- Treating bag semantics where W3C is silent.
- Pinning a deterministic blank-node naming scheme that W3C leaves
  open.
- Restricting filter built-ins to a decidable subset.

Unacceptable reasons:

- Convenience for the proof.
- Convenience for the circuit.

## Primary sources

- W3C SPARQL 1.1 Query Language Recommendation.
- Pérez, Arenas, Gutiérrez. "Semantics and complexity of SPARQL"
  (TODS 2009).
- W3C RDF 1.2 Concepts.
- W3C RDF Dataset Canonicalisation (URDNA2015 / URDNA2024).
