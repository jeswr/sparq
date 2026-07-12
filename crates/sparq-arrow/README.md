# sparq-arrow

> 🤖 **SPARQ agent** [OPUS-4.8] — opt-in **Apache Arrow** columnar export for sparq.
> Projects a SPARQL `SELECT` result (`sparq_engine::QueryResult`) into an Arrow
> `RecordBatch` so query results flow into the dataframe / analytics / ML ecosystem
> (Polars, DuckDB, pandas) without a CSV round-trip. Issue #910, bead `sq-v78l4`.
>
> [GPT-5.6] Bead `sq-lsp7k.16` adds the checked inverse, `from_record_batch`.

**Opt-in / lean-core by construction.** This is a separate leaf crate, and Arrow
import/export sits behind the `arrow` feature (OFF by default). The `arrow-*` dependency
closure NEVER enters `sparq-core` / `sparq-engine` / the wasm bundle; nothing in the
workspace default build depends on it. The default build of *this* crate pulls no Arrow
code at all — only the dependency-free field-name constants and the schema docs.

## 🚀 Quickstart

```rust,ignore
use sparq_arrow::{from_record_batch, to_record_batch};
use sparq_engine::{query, QueryResult};
use sparq_core::Graph;

let graph = Graph::load_str("<http://ex/a> <http://ex/p> \"42\" .", "turtle")?;
let result: QueryResult = query(&graph, "SELECT ?s ?o WHERE { ?s ?p ?o }")?;

// One Arrow struct column per SELECT variable; one row per solution.
let batch = to_record_batch(&result)?;       // needs --features arrow
assert_eq!(batch.num_columns(), 2);          // ?s, ?o
assert_eq!(batch.schema().field(0).name(), "s");
let restored = from_record_batch(&batch)?;
assert_eq!(restored.vars, result.vars);
assert_eq!(restored.rows, result.rows);
# Ok::<(), Box<dyn std::error::Error>>(())
```

`cargo add sparq-arrow --features arrow` (or, in this workspace,
`cargo build -p sparq-arrow --features arrow`).

## ✨ The RDF-term → Arrow mapping

Arrow has no native RDF-term type, so the export widens each term into a **struct** —
faithful and round-trippable, nothing is thrown away. Every `SELECT` variable becomes
one column of `Struct<kind, value, datatype, language, direction>` (all nullable
`Utf8`); the field names are the public `FIELD_*` / `RDF_TERM_FIELDS` constants.

| field        | meaning                                                                          |
|--------------|----------------------------------------------------------------------------------|
| `kind`       | `"uri"` / `"bnode"` / `"literal"` / `"triple"` (mirrors SPARQL-JSON `type`)       |
| `value`      | IRI string / blank-node label / literal lexical value / N-Triples-encoded triple |
| `datatype`   | datatype IRI of a **typed** literal (null otherwise)                             |
| `language`   | BCP-47 language tag of a **language-tagged** literal (null otherwise)            |
| `direction`  | RDF 1.2 base direction `"ltr"` / `"rtl"` (null otherwise)                        |

An **unbound** binding is a `null` struct slot — distinct from a bound empty-string
literal. `term_schema(&vars)` builds the schema without materialising rows.
`from_record_batch(&batch)` validates that exact schema and reconstructs the variables,
row order, unbound cells, and RDF terms. Invalid variable names, schemas, kinds, IRIs,
blank-node labels, language tags, directions, triple terms, or incompatible literal
metadata return `ArrowError`; they never panic or silently select a fallback term kind.

## 📚 Honest boundary / caveats

- **`xsd:string` is written explicitly**, not elided (SPARQL-JSON elides it); `kind`
  already separates literals from IRIs, so this is purely additive and removes a
  consumer special-case.
- **No numeric narrowing yet.** `42^^xsd:integer` is the *string* `"42"` plus a datatype
  field, not an Arrow `Int64`. A typed-column projection (numbers/dates → native Arrow
  types, term-struct fallback) is a deliberate follow-up — this v1 is the lossless
  baseline a typed view can build on.
- **Triple terms are stringified** to N-Triples in `value` (`kind = "triple"`), not
  exploded into nested struct fields.
- This is a **projection for transport**, not a canonical RDF serialisation: the Arrow
  batch is not itself an RDF document. `from_record_batch` is its checked inverse.
- **Python binding.** Issue #910 frames a `sparq-py` `Graph.query_arrow() ->
  pyarrow.Table` over this export; that PyO3 binding lives in the opt-in `arrow` feature
  of `sparq-py` (bead `sq-lt1ml`) — it reuses `to_record_batch` here and bridges the
  batch to pyarrow through the Arrow C Data Interface. No performance numbers are claimed.

## License

MIT — see the repository-root [`LICENSE`](../../LICENSE). © 2026 Jesse Wright.
