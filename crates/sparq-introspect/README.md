# sparq-introspect

**Ontology / schema introspection** for the sparq RDF engine — an opt-in crate
(GenAI phase 2, see [`research/genai-design.md`](../../research/genai-design.md) and
[`research/genai-ontology-introspection.md`](../../research/genai-ontology-introspection.md))
that mines the *effective schema* a knowledge graph actually uses, straight from the
store's sorted permutation indexes. No models, no network, no extra state — and
**no GROUP BY**: every statistic is a sorted scan over indexes the store already keeps.

The output grounds NL→SPARQL prompts (compact, budgeted text summaries; full JSON for
machine consumption) and doubles as planner-grade statistics (the characteristic-set
table is the SOTA star-join cardinality estimator — see `TODO.md` for the wiring).

## What it computes

- **Characteristic sets** (Neumann & Moerkotte, ICDE 2011): one SPO scan groups
  subjects by their *exact* predicate set — subjects are contiguous and predicates
  sorted within each subject run, so the per-subject predicate list falls out of run
  boundaries. Each distinct set carries its subject count, per-predicate triple counts
  (avg multiplicity = `predicate_triples[i] / subjects`), and the `rdf:type` histogram
  of its subjects (the declared classes behind the emergent entity type).
- **Schema summary**:
  - classes (`rdf:type` objects) with instance counts;
  - per-class predicate usage: which predicates appear on instances of each class,
    with subject/triple counts and **coverage ratios**;
  - per-predicate global stats: triples, distinct subjects/objects, literal-vs-IRI
    object split, datatype distribution, deterministic sample values (the first sample
    is the predicate's *minimum* object value — sorted indexes give min/max for free);
  - **observed domain/range**: the most-common subject/object classes per predicate,
    inferred from usage, alongside any **declared** `rdfs:domain`/`rdfs:range` present
    (real KGs are under-declared and mis-typed; usage wins, both are reported).
- **Vocabulary detection**: namespaces in use (IRIs split at the last `#`/`/` — the
  split the dictionary itself stores) with distinct-term counts, recognised against a
  bundled offline table of well-known vocabularies (rdf/rdfs/owl/xsd/foaf/skos/
  schema.org/dcterms/wd/dbo/…).

## API

```rust
use sparq_introspect::{Introspection, BuildOptions};

let ix = Introspection::build(&graph);              // or build_with(&graph, &BuildOptions{..})

ix.classes        // Vec<ClassProfile>      — by instance count, with per-class usage
ix.predicates     // Vec<PredicateProfile>  — global stats + observed/declared domain/range
ix.characteristic_sets  // CharacteristicSets — top sets + exact tail aggregates
ix.vocabularies   // Vocabularies           — namespaces + well-known recognition

ix.to_json()                  // pretty JSON — the machine surface for LLM grounding
ix.to_text_summary(2500)      // prompt-ready digest, most-important-first, ≤ 2500 chars
```

`to_text_summary(budget_chars)` renders: header totals → prefix glossary (exactly the
namespaces the body uses; well-known prefixes, `nsN` otherwise, assigned in first-use
order) → classes with per-class coverage + range hints + samples → characteristic-set
patterns → global predicate stats. Lines are dropped greedily at the budget; a final
`…` marks elision. Range/sample hints in the class section come from the predicate's
*global* profile (the per-class numbers are exact; hints are hints).

## Cost model

One full SPO scan (characteristic sets, per-class usage, observed domains), one pass
over every predicate's POS block (object kinds, datatypes, samples, observed ranges —
type lookups once per *distinct* object), two range scans for declared domain/range,
one dictionary pass (vocabularies). `O(|G| + |dict|)` time, output-sized memory plus
the subject→types map.

## Measured results — Apple M1, 16 GB, rustc 1.89, `--release`

Reproduce: `cargo run -p sparq-introspect --example olympics_introspect --release`
(fixture paths via `SPARQ_OLYMPICS_NT` / `SPARQ_SYNTHETIC_NT`).

| Dataset | Load | `Introspection::build` | `to_json` | `to_text_summary(2500)` |
|---|---|---|---|---|
| olympics, 1.78M triples (407k subjects, 8 classes, 16 predicates, 15 char. sets, 6243 namespaces) | 0.98 s | **0.10 s** | <1 ms (65 KB) | 0.1 ms |
| qlever-synthetic, 10.0M triples (1.25M subjects, 1 class, 5 predicates, 1 char. set) | 5.8 s | **0.99 s** | <1 ms (5 KB) | 0.1 ms |

Gates (design doc §4): full introspection scan ≤ dataset load time — **6–10× under** on
both datasets; summary generation < 100 ms at olympics scale — **0.1 ms**; the olympics
text summary names the dataset's actual classes (foaf:Person, dbo:SportsEvent,
dbo:SportsTeam, dbo:Olympics, …) — asserted by `tests/olympics.rs`.

Olympics summary excerpt (budget 2500):

```text
# Schema summary — 1781625 triples, 406700 subjects, 8 classes, 16 predicates, 15 entity patterns
## Prefixes
foaf: http://xmlns.com/foaf/0.1/ — FOAF (people & agents) (3 terms)
dbo: http://dbpedia.org/ontology/ — DBpedia ontology (10 terms)
ns1: http://wallscope.co.uk/resource/olympics/team/ (1184 terms)
…
## Classes (by instance count)
### foaf:Person — 134730 instances
- dbo:team — 134730/134730 subjects (100%) → dbo:SportsTeam, e.g. ns1:Malaysia
- rdfs:label — 134730/134730 subjects (100%) → rdf:langString, e.g. "A. Aanantha Sambu Mayavo"@en
- foaf:age — 128429/134730 subjects (95%, avg 1.4/subj) → xsd:int, e.g. "26"
…
### dbo:SportsEvent — 765 instances
- rdfs:subClassOf — 765/765 subjects (100%) → dbo:Sport, e.g. ns3:Aeronautics
…
```

## Tests

- 9 unit tests (`src/lib.rs`): characteristic-set exactness (counts, multiplicities,
  partition of subjects, cap aggregation), class instance counts + coverage, object
  kinds/datatypes/samples (incl. inline integers and language tags), observed-vs-
  declared domain/range, vocabulary counts + well-known recognition, JSON validity,
  text-summary budget/truncation/content, empty + typeless graphs.
- 1 integration test (`tests/olympics.rs`): the design-doc gate at 1.78M-triple scale —
  skips (passes with a note) when the fixture is absent; override the path with
  `SPARQ_OLYMPICS_NT`.

## Zero impact

Separate opt-in crate: no core crate depends on it, the default build does not compile
it, and it is read-only over `sparq-core`'s public API (`Graph::id_of`, `store.scan`/
`scan_sorted`, `dict.term_parts`). Works against every storage mode (raw, mmap'd,
compressed) because it only uses the public scan surface.
