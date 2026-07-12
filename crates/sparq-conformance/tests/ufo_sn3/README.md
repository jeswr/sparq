# UFO-SN3 reference implementation

UFO-SN3 is an executable, finite-world projection of representative Unified Foundational Ontology concepts onto SPARQ’s N3 forward reasoner.

The artifact covers:

- UFO-A: Kind, Role, Phase, Category, identity criteria, rigidity, Quality, Mode, Disposition, Relator, inherence, and existential dependence.
- UFO-B: Event and reified Participation records.
- UFO-C: Agent, Goal, Commitment, Claim, Norm, and supplied Obligation records.
- Explicit Situation and World resources, accessibility, existence, propositions, and closed validation scopes.

## Layout and execution

The directory is organized as:

```text
ufo_sn3/
├── vocab/ufo-sn3.ttl
├── rules/ufo-sn3.n3
├── cases/<name>/input.n3
├── cases/<name>/answer.n3
└── README.md
```

Each conformance case follows the existing `eye_cases` execution model:

1. Concatenate `cases/<name>/input.n3` with `rules/ufo-sn3.n3`.
2. Run the concatenated document through `sparq_reason::reason_n3`.
3. Parse `cases/<name>/answer.n3`.
4. Require every answer triple to occur in the materialized closure.

The comparison is deliberately superset entailment. Input facts and additional valid consequences may occur in the closure.

## Reification-node projection

SPARQ’s N3 `Term` model has variables, lists, and quoted N3 formulae, but no RDF 1.2 triple-term variant. Rules therefore cannot bind a pattern such as `<<( ?s ?p ?o )>>`.

UFO-SN3 v1 uses an explicit proposition node instead:

```n3
ex:p1 a ufo:Proposition;
    ufo:subj ex:alice;
    ufo:pred ex:employedBy;
    ufo:obj ex:acme;
    ufo:holdsIn ex:w1.
```

The proposition node may hold, fail to hold, or be necessary in multiple represented situations without asserting the encoded subject-predicate-object triple in the enclosing graph.

The N3 engine lacks RDF 1.2 triple terms, so v1 uses this reification-node projection. Native RDF 1.2 quoted-triple parsing, matching, normalization, and proof serialization are a separate engine gap and are not claimed by this artifact.

## Decidable profile

The shipped rules are:

- function-free;
- range-restricted, with every conclusion variable bound in the premise;
- finite and situation-scoped;
- monotone;
- free of conclusion blank nodes and other fresh-term generation;
- independent of network access, aggregation, and nondeterministic built-ins;
- restricted to supplied worlds, propositions, events, relators, manifestations, commitments, claims, and obligations.

Positive materialization therefore reaches a finite fixpoint on finite input. With the ruleset fixed, data complexity is polynomial. General function-free Datalog has EXPTIME-complete combined complexity when the rules are also part of the input; UFO-SN3 does not claim a lower unrestricted combined bound.

Identity rules derive `ufo:sameContinuant` over existing resources. They do not emit `owl:sameAs` and do not collapse situation records or other representations.

## Closed validation

Open-world absence is never interpreted as falsity.

The runnable v1 validation rule requires all of the following:

- a represented situation;
- an explicit `ufo:closedFor` declaration;
- a finite validation scope;
- a supplied `ufo:requiredProposition`;
- an explicit `ufo:notHoldsIn` refutation.

Only then does it derive `ufo:refutedIn` and a scope finding. Merely omitting `ufo:holdsIn` derives nothing.

This explicit-refutation design keeps the concatenated `reason_n3` execution monotone. Missing-witness detection by negation-as-failure would require a separately ordered, stratified validation pass after positive closure; it is not simulated inside the single forward closure.

## Boundary and non-claims

UFO-SN3 is a finite-world reference profile, not a full UFO decision procedure.

It reasons only over explicitly represented situations and accessibility edges. It does not quantify over unrepresented possible worlds. Rigidity and necessity results therefore concern the supplied finite frame, not every metaphysically possible world.

The profile does not provide:

- unrestricted quantified modal or deontic validity;
- arbitrary executable identity formulae;
- automatic existential witnesses;
- unrestricted event or individual fusion;
- general counterfactual causation;
- defeasible normative conflict resolution;
- anti-rigidity validation from missing counter-situations;
- RDF 1.2 triple-term matching.

Required events, propositions, worlds, manifestations, obligations, commitments, and validation records must be supplied and finitely named. The rules may classify or relate those resources but never invent them.

## UFO and gUFO attribution

This reference profile is an independent executable projection informed by the Unified Foundational Ontology (UFO) and the lightweight gUFO implementation.

gUFO is by Victorio A. Carvalho, João Paulo A. Almeida, Claudenir M. Fonseca, and Giancarlo Guizzardi and is distributed under the Creative Commons Attribution 4.0 International license:

- https://nemo-ufes.github.io/gufo/
- https://creativecommons.org/licenses/by/4.0/

UFO and gUFO names are used with attribution. This artifact does not claim to reproduce the complete reference-UFO semantics or to be an official UFO/gUFO release.
