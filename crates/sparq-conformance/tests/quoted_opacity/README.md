# RDF 1.2 quoted-triple OPACITY fixtures

Fixtures for `tests/quoted_triple_opacity.rs` — the conformance lane pinning the
SEMANTIC OPACITY of RDF 1.2 quoted/reified triples under the reasoning profiles
(`sparq-reason` RDFS + OWL 2 RL; `sparq-reason-el` behind the `el-suite` feature).

## What "opacity" means here (RDF 1.2 Semantics)

A quoted/reified triple is a TERM, not a statement. Asserting

```turtle
:r rdf:reifies <<( :s :p :o )>> .
```

asserts exactly ONE triple — that `:r` reifies the triple TERM `<<( :s :p :o )>>`
— and NOTHING about `:s :p :o` itself. RDF 1.2 (CR, "RDF 1.2 Semantics" §
triple terms) makes triple terms REFERENTIALLY OPAQUE: quoting never asserts.
The reasoner must therefore

1. **never assert the reified triple's content** — `<< :s :p :o >>` in a graph
   does not entail `:s :p :o` (nor anything, e.g. domain/range typings, that
   would follow from it), unless `:s :p :o` is SEPARATELY asserted;
2. **not let triple terms interfere with closure** — the RL/EL closure of a
   base graph is byte-identical whether or not reified triples referring to it
   (true, false, or entailed-but-unasserted ones) are present;
3. **reason over the reifier node normally** — a reifier's own annotations
   (type, certainty, provenance) participate in closure like any other triples,
   WITHOUT leaking the quoted content into the base closure.

Note the asymmetry the fixtures also pin: the ANNOTATION syntax
`:s :p :o {| … |}` DOES assert `:s :p :o` (and additionally reifies it) — that
is the positive control proving the harness' negative guards are meaningful.

## Files

| file | role |
|---|---|
| `never_asserts.ttl` | reified triples in all three surface forms (`rdf:reifies` + triple term, `<< … ~ :r >>` as subject, `<< … >>` as object) plus the asserted annotation-syntax control |
| `base.ttl` | the base graph (NO triple terms): RDFS schema + OWL RL (transitive, inverse) axioms + facts |
| `quoted_overlay.ttl` | reified triples REFERRING to `base.ttl` (an asserted, a false, an entailed-but-unasserted, and a reversed one) + an inert reifier annotation |
| `annotated_reifier.ttl` | a reifier carrying annotations that the schema reasons over normally |
| `el_base.ttl` / `el_overlay.ttl` | the EL-classifier variant: a subclass chain, and a quoted (never asserted) `rdfs:subClassOf` axiom |
| `expected/base_rdfs_closure.nt` | committed expected answer: the FULL RDFS closure of `base.ttl`, sorted N-Triples, byte-pinned |
| `expected/base_owlrl_closure.nt` | committed expected answer: the FULL OWL 2 RL closure of `base.ttl`, sorted N-Triples, byte-pinned |

The expected-answer files are hand-verified (every line is either a `base.ttl`
triple or annotated with the rule that derives it — see the comments in
`base.ttl`). They pin the base closure BYTE-EXACTLY, so the non-interference
test (removing the overlay-only closure delta from `closure(base ∪ overlay)`
yields `closure(base)`) is anchored to a committed artifact, not to a same-run
self-comparison.
