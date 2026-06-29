# sparq-reason

<p>
  <a href="https://crates.io/crates/sparq-reason"><img src="https://img.shields.io/crates/v/sparq-reason.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-reason"><img src="https://docs.rs/sparq-reason/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

**Opt-in RDFS / OWL-RL / Notation3 reasoning** for the [sparq](../../README.md) RDF engine.

It forward-chains the deductive closure (RDFS, the OWL 2 RL property/class axioms, or
user-supplied N3 rules) over dictionary-encoded triples and **materializes** the entailed
facts, so querying stays exactly as fast as before. Reasoning runs over integer ids (joins
on fixed-width keys); the closure can be maintained incrementally under inserts/deletes, and
the non-default `explain` feature answers `why(triple)` with a proof tree. This crate is
**isolated** — depend on it to get reasoning; the core engine and wasm build carry zero cost.

## 🚀 Quickstart

```rust
# fn main() -> Result<(), Box<dyn std::error::Error>> {
use sparq_core::Graph;
use sparq_reason::{materialize, Profile};

// Parse to (Dict, triples), expand in place with every entailed triple, then index.
let (mut dict, mut triples) = Graph::parse_to_triples(turtle, "turtle")?;
let _added = materialize(Profile::Rdfs, &mut dict, &mut triples); // OwlRl includes RDFS
let g = Graph::from_parts(dict, triples);
# let _ = g;
# Ok(()) }
# const turtle: &str = "<http://ex/a> <http://www.w3.org/2000/01/rdf-schema#subClassOf> <http://ex/b> .";
```

## ✨ Features

- **RDFS entailment** — the non-explosive subset (rdfs2/3/5/7/9/11), materialized in one pass.
- **OWL 2 RL** — property/class axioms (`sameAs`, `inverseOf`, Transitive / Symmetric,
  `equivalentClass` / `equivalentProperty`, …) over the same fixpoint engine. Complete for the
  assertion-style RL/RDF rules (W3C OWL-RL conformance row is at the profile ceiling — the
  documented divergences are conclusions provably outside RL, not missing rules); use
  `sparq-reason-el` for complete class classification.
- **D-entailment** (opt-in `d-entail`) — `Profile::D`: the rdfD1 datatype-typing rule under
  a recognized datatype map, with **correct typed value-space equality**
  (`"1"^^xsd:integer` is the same value as `"1.0"^^xsd:decimal`, never an f64 fast path).
- **Notation3** — `{ … } => { … }` rules with EYE-validated builtins (a separate subsystem).
- **Incremental maintenance** — `MaterializedGraph` keeps the closure current under
  inserts/deletes by exact derivation counting; cost scales with the change, not a re-run.
- **Proof trees** (`explain` feature) — `why(triple)` returns which rule fired from which
  premises, recursively down to asserted facts (a flat, ZK-witness-friendly shape).
- **Shared join kernels** (opt-in `substrate-join`) — the RDFS predicate join (rdfs2/3/7)
  drives the *same* `sparq-substrate::join` hash-join body the SPARQL engine drives, supplying
  the reasoner's own key projection + budget monomorphically. Behaviour-neutral: the same
  closure, only the join machinery is shared. Off by default; byte/bundle ratchets unchanged.

## 📚 Learn more

- **How-to** — [`skills/inference/SKILL.md`](../../skills/inference/SKILL.md) (profiles,
  incremental maintenance, proof trees, CLI seam).
- **API reference** — [docs.rs/sparq-reason](https://docs.rs/sparq-reason).
- **Design** — the inference verdicts in [`research/`](../../research) and
  [`research/ARCHITECTURE.md`](../../research/ARCHITECTURE.md).
- **Performance** — see the [benchmarks dashboard](https://jeswr.github.io/sparq/dev/bench).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md) and [`CONTRIBUTING.md`](../../CONTRIBUTING.md).

## License

[MIT](../../LICENSE).
