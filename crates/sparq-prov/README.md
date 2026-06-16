# sparq-prov

W3C **[PROV-O]** data-lineage records for data **derived** by a sparq operation —
a small, **opt-in** public API over sparq's term model.

When a sparq operation *derives* new triples from existing data, this crate
optionally records *who/what/when* produced them, as standard PROV-O RDF:
a `prov:Activity` (the operation, time-stamped), a `prov:Entity` (the result),
and the lineage edges `prov:wasGeneratedBy`, `prov:used`, `prov:wasDerivedFrom`.

[PROV-O]: https://www.w3.org/TR/prov-o/

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).
> Bead sq-ntcg · CDMC capability gap **CD-1** (first-class data lineage).

## 🚀 Quickstart

```rust
use sparq_core::Graph;
use sparq_prov::{derive_construct, ProvConfig};
use oxrdf::NamedNode;

let g = Graph::load_str(
    "@prefix ex: <http://ex/> . ex:alice ex:age 30 .", "turtle",
).unwrap();

// Run a CONSTRUCT and capture its PROV-O lineage, naming the input source.
let config = ProvConfig::with_inputs([NamedNode::new_unchecked("http://ex/src")]);
let d = derive_construct(
    &g,
    "PREFIX ex: <http://ex/> CONSTRUCT { ?s ex:years ?a } WHERE { ?s ex:age ?a }",
    config,
).unwrap();

let derived = d.triples();          // the data the operation produced
let lineage = d.prov_graph();       // its PROV-O record (Vec<Triple>)
let turtle  = d.prov_ntriples();    // …serialised (N-Triples ⊂ Turtle)
```

## ✨ Features

- **CONSTRUCT/DESCRIBE lineage** — [`derive_construct`] wraps the engine's
  `CONSTRUCT`/`DESCRIBE` evaluation, times it, and returns a [`Derivation`]
  carrying both the derived triples and a PROV-O lineage graph.
- **Standard PROV-O shape** — for result entity `E`, activity `A`, inputs `Iᵢ`:
  `A a prov:Activity` · `E a prov:Entity` ·
  `A prov:startedAtTime/endedAtTime "…"^^xsd:dateTime` ·
  `E prov:wasGeneratedBy A` · `A prov:used Iᵢ` · `E prov:wasDerivedFrom Iᵢ` ·
  (optional) `A prov:wasAssociatedWith <agent>`. Every IRI is absolute, so the
  graph is valid PROV-O that round-trips through any RDF parser.
- **Opt-in, zero core overhead** — a standalone member (like `sparq-canon`):
  nothing in sparq's default build or the wasm artifact depends on it, so the
  capability is **off by default** and the lean core is byte-identical without
  it. Pull it in explicitly only where you need lineage.
- **Configurable identity** — IRIs default to stable, content-addressed
  `urn:sparq:prov:` nodes (same derivation ⇒ same IRIs); set `ProvConfig`
  `activity`/`entity`/`used`/`agent` to integrate with an external provenance
  store or named-graph scheme. The clock is injectable for deterministic tests.
- **Dependency-light** — `xsd:dateTime` is formatted in-crate (no `chrono`/
  `time` dep); the formatter is the inverse of `sparq-core`'s dateTime parser,
  so a recorded timestamp parses back to the same instant (tested).

## Scope — covered vs deferred

| Derivation path | Status |
|---|---|
| `CONSTRUCT` / `DESCRIBE` (query → new graph) | ✅ covered here |
| SPARQL UPDATE (`INSERT … WHERE`, `INSERT DATA`) | ⏳ deferred (follow-up bead) |
| Reasoner materialization (RDFS / OWL-RL / N3) | ⏳ deferred — would reuse `sparq-reason`'s per-triple `why()` proof trees, a *finer-grained* derivation provenance than PROV-O alone (follow-up bead) |

The CONSTRUCT path is the cleanest, best-tested derivation in the engine and the
natural first PROV-O target; the deferred paths are filed as beads.

## 📚 Learn more

- W3C PROV-O: <https://www.w3.org/TR/prov-o/>
- Skill: `skills/prov-lineage/SKILL.md`
- Hartig provenance research: `research/feature-research-hartig.md` (§6)
- CDMC CD-1 gap: `compliance/cdmc/gap-register.md`

## License

MIT — `publish = false` workspace member. [OPUS-4.8]
