# sparq-introspect

**Ontology / schema introspection** for the sparq RDF engine — an **opt-in** crate
(GenAI phase 2) that mines the *effective schema* a knowledge graph actually uses,
straight from the store's sorted permutation indexes. No models, no network, no extra
state, and **no GROUP BY**: every statistic is a sorted scan over indexes the store
already keeps. The output grounds NL→SPARQL prompts (budgeted text summaries; full JSON
for machines) and doubles as planner-grade statistics (the characteristic-set table is
a SOTA star-join cardinality estimator).

## 🚀 Quickstart

```rust
use sparq_introspect::{Introspection, BuildOptions};

let ix = Introspection::build(&graph);              // or build_with(&graph, &BuildOptions{..})

ix.classes              // Vec<ClassProfile>      — by instance count, with per-class usage
ix.predicates           // Vec<PredicateProfile>  — global stats + observed/declared domain/range
ix.characteristic_sets  // CharacteristicSets     — top sets + exact tail aggregates
ix.join_hints           // JoinHints              — cross-class (C, p, D) edges + triple counts
ix.entities             // u64                    — distinct typed subjects (void:entities)
ix.vocabularies         // Vocabularies           — namespaces + well-known recognition

ix.to_json()                                  // pretty JSON — the machine surface for LLMs
ix.to_text_summary(2500)                      // prompt-ready digest, most-important-first, ≤ N chars
ix.to_void("http://ex.org/dataset")           // W3C VoID description, as N-Triples (valid Turtle)
ix.to_void_with_cs("http://ex.org/dataset")   // VoID + characteristic-set source stats (scs: ext)
ix.schema_summary_for(&seeds, 2500)           // retrieval-mode: schema scoped to seed IRIs
```

## ✨ Features

- **Characteristic sets** (Neumann & Moerkotte, ICDE 2011) — one SPO scan groups
  subjects by their *exact* predicate set; each distinct set carries its subject count,
  per-predicate triple counts (avg multiplicity = `predicate_triples[i] / subjects`),
  and the `rdf:type` histogram of its subjects. Top sets retained, exact tail aggregates.
- **Schema summary** — classes with instance counts; per-class predicate usage with
  subject/triple counts and **coverage ratios**; per-predicate global stats (triples,
  distinct subjects/objects, literal-vs-IRI split, datatype distribution, deterministic
  sample values); and **observed domain/range** (most-common subject/object classes,
  inferred from usage) alongside any **declared** `rdfs:domain`/`rdfs:range` — real KGs
  are under-declared and mis-typed, so usage wins and both are reported.
- **Cross-class join hints** — the `(subject_class) --predicate--> (object_class)` edge
  table with per-edge triple counts, mined in the *same* SPO scan as the characteristic
  sets; top edges by triple count, capped with exact tail aggregates.
- **Text summary** (`to_text_summary(budget_chars)`) — header totals → prefix glossary
  (only the namespaces the body uses; well-known prefixes, `nsN` otherwise) → classes
  with coverage + range hints + samples → characteristic-set patterns → predicate stats.
  Lines drop greedily at the budget; a final `…` marks elision.
- **VoID export** (`to_void`) — a [W3C VoID](https://www.w3.org/TR/void/) description as
  N-Triples (parses as Turtle too — no serializer dep), with exact `void:triples`,
  `void:entities`, `void:distinctSubjects`, `void:classes`, `void:properties`, plus a
  `void:classPartition` and `void:propertyPartition` each. `void:distinctObjects` is
  **not** emitted — omitted rather than misleading (no global de-duplicated count kept).
- **VoID + characteristic-set source stats** (`to_void_with_cs`, federation A3/Z2) — a
  strict superset of `to_void`, then the mined sets under a documented sparq extension
  vocab `scs:` (`<http://sparq.dev/ns/cs#>`); the served federation-descriptor surface
  that primes a remote CostFed/Odyssey-class source-selector with star/multi-join
  cardinalities. Served at `GET /.well-known/void` behind the opt-in
  `federation-descriptors` feature.
- **Vocabulary detection** — namespaces in use (split at the last `#`/`/`, the split the
  dictionary stores) with distinct-term counts, recognised against a bundled offline
  table (rdf/rdfs/owl/xsd/foaf/skos/schema.org/dcterms/wd/dbo/…).
- **Retrieval-mode summary** (`schema_summary_for(seeds, budget)`) — a seed-scoped digest
  for KGs whose full schema overflows a prompt; filters the already-mined profiles by IRI
  (no re-scan).
- **Graph scoping** — `Introspection::build(&graph)` introspects the store of whatever
  `Graph` it is handed: the **default graph** when you pass the top-level `&graph`, and a
  **single named graph** when you pass that graph's sub-`Graph`. Each named graph of a
  quad dataset (loaded via `Graph::load_dataset` from N-Quads / TriG) is a self-contained
  `Graph`, fetched by name with [`Graph::named_graph(&name)`][named-graph] (sq-quuu):
  ```rust
  let g1 = graph.named_graph(&ex_g1).expect("graph exists");
  let card = sparq_introspect::Introspection::build(g1); // schema card for ex:g1 alone
  ```
  There is **no cross-graph or union-of-all-graphs** build: a VoID / schema card is always
  scoped to exactly one graph, so on a multi-graph dataset run it per graph (or over the
  default graph) rather than expecting it to merge the quads. Default-graph-only crates
  silently mixing graphs was sq-quuu; this is the documented + supported scope.

  [named-graph]: https://docs.rs/sparq-core/latest/sparq_core/struct.Graph.html#method.named_graph
- **Cost & zero impact** — `O(|G| + |dict|)` time, output-sized memory plus the
  subject→types map. Separate opt-in crate: no core crate depends on it, the default
  build does not compile it, and it is read-only over `sparq-core`'s public scan surface
  (works against raw, mmap'd, and compressed storage).

## Measured results

The `olympics_introspect` example reports load / build / `to_json` / `to_text_summary`
time over the olympics (1.78M triples) and qlever-synthetic (10M) fixtures (paths via
`SPARQ_OLYMPICS_NT` / `SPARQ_SYNTHETIC_NT`). The design-doc §4 gates: the introspection
scan stays within dataset load time; summary generation is quick relative to that load;
and the olympics summary names the dataset's actual classes (asserted by
`tests/olympics.rs`). Tracked figures live on the perf dashboard.

```sh
cargo run -p sparq-introspect --example olympics_introspect --release
```

Tests: 14 unit (`src/lib.rs`: characteristic-set exactness, coverage, object
kinds/datatypes/samples, observed-vs-declared domain/range, vocabularies, JSON validity,
text-summary budget, join hints, VoID export, retrieval mode) plus 1 integration
(`tests/olympics.rs`, skip-if-absent at 1.78M scale).

## 📚 Learn more

- Design records: [`research/genai-design.md`](../../research/genai-design.md),
  [`research/genai-ontology-introspection.md`](../../research/genai-ontology-introspection.md)
- W3C VoID: <https://www.w3.org/TR/void/>
- Perf dashboard: <https://jeswr.github.io/sparq/dev/bench>
- Open work for this crate: `bd list -l area:sparq-introspect`

## License

MIT. [OPUS-4.8] sq-lsxd
