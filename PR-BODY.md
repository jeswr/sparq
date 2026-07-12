# reason: opt-in quoted-triple (RDF 1.2 reifier) inference for the OWL-RL profile — destructure/construct bridge rules with a finite-Herbrand-base guarantee

## What

A new **opt-in `quoted-triples` feature** on `sparq-reason` (default OFF, zero new deps,
no new `Profile` variant, nothing outside the crate changes except the feature-matrix
leg): two monotone RL-profile rules over the loaders' one canonical quoting shape
(`R rdf:reifies <<( s p o )>>`, the triple term one structurally-interned dictionary id):

| rule | premise ⊢ conclusion |
|---|---|
| **reif-dtr** (destructure) | `(r rdf:reifies tt)`, `tt = <<( s p o )>>` ⊢ `(r rdf:type rdf:Statement)`, `(r rdf:subject s)`, `(r rdf:predicate p)`, `(r rdf:object o)` |
| **reif-ctr** (construct) | `(r rdf:subject s)`, `(r rdf:predicate p)`, `(r rdf:object o)`, `(s p o)` in the closure, `o` not a triple term ⊢ `(r rdf:reifies <<( s p o )>>)` |

reif-dtr is the "unstar" projection into the classic reification vocabulary, so ordinary
RL rules (domain/range, subPropertyOf, sameAs, …) reason over reifier **annotations** and
the recovered components; reif-ctr is its converse, surfacing classic-reification data as
native RDF 1.2 reifiers. Wired into `materialize_owl_rl` as an alternation — one reify
round, one RL closure — to the joint fixpoint.

**Decidability / termination.** Forward chaining with a term *constructor* is not
obviously terminating: unrestricted, `rdf:reifies rdfs:subPropertyOf rdf:object` lets the
constructor quote its own conclusions forever. reif-ctr therefore carries two
restrictions (full argument in the `reify` module docs, adversarial cascade pinned by
`construct_never_quotes_a_triple_term`): (1) **only EXISTING triples are reified** — the
referent triple must already be in the closure; (2) **no quoting of triple terms** — every
constructed term has leaf components (subject IRI/blank + predicate IRI kind guards;
object checked explicitly). Together: no rule invents nodes, the mintable term set is
bounded by triples over a fixed finite leaf universe, so the Herbrand base is finite,
all rules are monotone, and the alternation reaches a least fixpoint. The same
structural (not history-dependent) form of the guards keeps `materialize` idempotent.

**Opacity** (coordinated with the sibling opacity-semantics PR; no dependency): quotation
never asserts — `(r rdf:reifies <<( s p o )>>)` does NOT entail `(s p o)` (pinned by
leak tests); a triple term is one atomic id, so nothing — including `owl:sameAs` entity
rewriting — ever substitutes inside a quotation; only the outer quotation of a nested
term is destructured.

**Default-off safety, three layers:** (1) feature OFF ⇒ the module is cfg'd out
entirely — plain `Profile::OwlRl` closures are byte-identical to before (the bridge is a
deliberate, NON-normative entailment extension, so the committed inference-conformance
ratchet and existing consumers cannot shift); (2) feature ON ⇒ an occurrence gate (four
read-only lookups, no interning) keeps reification-free closures byte-identical and
~zero-cost; (3) `MaterializedOwlGraph` routes any base mentioning the trigger vocabulary
to its documented Fallback mode (any-position check, mirroring the gate), preserving the
incremental == from-scratch parity invariant — with the documented Fallback `why()`
limitation applying to the new rules.

**Tests:** in-module unit suite (gate, destructure, guarded construct, ill-kinded
skip, nested-term opacity, the divergence-cascade termination pin, idempotency) +
`tests/owl_reify.rs` end-to-end fixtures (Turtle 1.2 reifiers / annotation blocks /
classic-reification data through the real loader, expected-answer closures, opacity
non-entailments, incremental parity). CI: a `sparq-reason (quoted-triples)` feature-matrix
leg (build + test + clippy, satisfying the C1 feature-gated-test-execution guard) and the
golden leg-name line.

## First increment; follow-ups

Deliberately small first increment. Named follow-ups (to be beaded on acceptance):

- **Fuse into the semi-naive core** — the alternation re-enters the batch closure; a
  delta-driven form of both rules inside the fixpoint's fused emitter removes the
  re-scan (profile first; reify rounds are rare by construction).
- **Counting-mode incremental support** — teach `MaterializedOwlGraph`'s counting modes
  the two rules instead of Fallback re-materialization.
- **`explain` proof-tree coverage** for reif-dtr/reif-ctr derivations.
- **Annotation-guarded unquoting** (a trust/provenance-gated `(s p o)` assertion rule)
  — explicitly deferred until the sibling opacity-semantics work lands, so the two
  don't define divergent opacity postures.
- **N3/datalog surface** — expose the same bridge to the N3 chainer's quoted-triple
  terms if a consumer needs it there.

## Context: why

This comes out of the **Kern (kernel-of-truth) research programme** — an agent-driven
effort evaluating whether statements-about-statements held *without assertion* (RDF 1.2
quotation) earn measurable lift when the reasoner can natively destructure, construct,
and annotate them. Today sparq's loaders and engine fully support RDF 1.2 quoting, but
the reasoner treats a reified triple as an inert constant — the gap this PR closes. The
capability is useful to sparq independently of that programme (any RDF 1.2 + reasoning
consumer meets it immediately), which is why it is proposed upstream rather than kept on
a fork. Authored by an AI agent (Fable) under the programme's
review-everything-before-commit workflow; treat with the usual reviewer skepticism and
feel free to request splits or renames.

---

*@jeswr will review; expect a delay — active review Wed–Fri.*
