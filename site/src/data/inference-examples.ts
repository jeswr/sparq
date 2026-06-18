// [OPUS-4.8] sq-0po6 — built-in examples for the live /surface/inference page.
// The default is the classic Socrates syllogism under RDFS (a subClassOf axiom +
// an instance entailing `Socrates a Mortal`), the example crates/sparq-reason-wasm
// itself documents. Two modes:
//   - "profile": run RDFS / OWL 2 RL forward-chaining over a Turtle document.
//   - "n3":      run a Notation3 rule document ({ … } => { … } + facts).

import type { ReasoningProfile } from "@/lib/reason-wasm";

export type InferenceMode = "profile" | "n3";

export interface InferenceExample {
  id: string;
  label: string;
  description: string;
  mode: InferenceMode;
  /** The reasoning profile (profile mode only; ignored for N3). */
  profile: ReasoningProfile;
  /** The RDF / N3 document. */
  data: string;
}

const SOCRATES_RDFS = `@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <http://ex/> .

# Ontology: every Human is a Mortal.
ex:Human rdfs:subClassOf ex:Mortal .

# Fact: Socrates is a Human.
ex:Socrates a ex:Human .
`;

const SOCRATES_N3 = `@prefix ex: <http://ex/> .

# Rule: anything that is a Human is a Mortal.
{ ?x a ex:Human } => { ?x a ex:Mortal } .

# Fact: Socrates is a Human.
ex:Socrates a ex:Human .
`;

const RDFS_DOMAIN_RANGE = `@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .
@prefix ex:   <http://ex/> .

# Ontology: knows has domain Person and range Person; Friend is a sub-property of knows.
ex:knows  rdfs:domain ex:Person ;
          rdfs:range  ex:Person .
ex:friend rdfs:subPropertyOf ex:knows .

# Facts: Alice friend Bob.
ex:Alice ex:friend ex:Bob .
`;

const OWL_RL_INVERSE = `@prefix rdf:  <http://www.w3.org/1999/02/22-rdf-syntax-ns#> .
@prefix owl:  <http://www.w3.org/2002/07/owl#> .
@prefix ex:   <http://ex/> .

# Ontology: parentOf and childOf are inverse properties; siblingOf is symmetric.
ex:parentOf  owl:inverseOf ex:childOf .
ex:siblingOf a owl:SymmetricProperty .

# Facts.
ex:Alice ex:parentOf  ex:Bob .
ex:Bob   ex:siblingOf ex:Carol .
`;

export const INFERENCE_EXAMPLES: InferenceExample[] = [
  {
    id: "socrates-rdfs",
    label: "Socrates (RDFS)",
    description:
      "The classic syllogism under RDFS: ex:Human rdfs:subClassOf ex:Mortal + ex:Socrates a ex:Human entails ex:Socrates a ex:Mortal.",
    mode: "profile",
    profile: "rdfs",
    data: SOCRATES_RDFS,
  },
  {
    id: "socrates-n3",
    label: "Socrates (N3 rule)",
    description:
      "The same syllogism as a Notation3 rule: { ?x a ex:Human } => { ?x a ex:Mortal } over ex:Socrates a ex:Human.",
    mode: "n3",
    profile: "rdfs",
    data: SOCRATES_N3,
  },
  {
    id: "rdfs-domain-range",
    label: "Domain / range (RDFS)",
    description:
      "rdfs:domain, rdfs:range and rdfs:subPropertyOf entailments: ex:Alice ex:friend ex:Bob types both as ex:Person and infers ex:Alice ex:knows ex:Bob.",
    mode: "profile",
    profile: "rdfs",
    data: RDFS_DOMAIN_RANGE,
  },
  {
    id: "owl-rl-inverse",
    label: "Inverse / symmetric (OWL 2 RL)",
    description:
      "OWL 2 RL property axioms: owl:inverseOf infers ex:Bob ex:childOf ex:Alice, and owl:SymmetricProperty infers ex:Carol ex:siblingOf ex:Bob.",
    mode: "profile",
    profile: "owl-rl",
    data: OWL_RL_INVERSE,
  },
];
