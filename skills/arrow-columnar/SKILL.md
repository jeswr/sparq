---
name: arrow-columnar
description: Export and import sparq SPARQL SELECT QueryResult values through Apache Arrow RecordBatch, in-memory Parquet bytes, or Arrow IPC stream bytes with the opt-in sparq-arrow crate. Use when moving query results into dataframe, analytics, transport, or storage tooling while preserving RDF term kinds, datatype and language metadata, RDF 1.2 triple terms, empty literals, and unbound cells.
---

# sparq-arrow — Arrow, Parquet, and IPC interop

Use `sparq-arrow` when a Rust application needs a faithful columnar representation of a
`sparq_engine::QueryResult`. The crate maps every SELECT variable to one nullable Arrow
`Struct` column with five nullable UTF-8 children: `kind`, `value`, `datatype`,
`language`, and `direction`.

## Choose a feature

- Enable `arrow` for `to_record_batch`, `from_record_batch`, `term_schema`, and
  `term_struct_type`.
- Enable `parquet` for `to_parquet_bytes`, `from_parquet_bytes`,
  `parquet_variables_from_bytes`, and `parquet_row_count_from_bytes`; it implies
  `arrow`.
- Enable `ipc` for `to_ipc_bytes`, `from_ipc_bytes`, and
  `ipc_variables_from_bytes`; it implies `arrow`.
- Leave all features disabled to retain only the dependency-free field-name constants.

All features are default-OFF. The crate is a leaf capability crate, so no Arrow
container dependency enters `sparq-core`, `sparq-engine`, or the WebAssembly bundle.

## Use a RecordBatch

```rust,ignore
use sparq_arrow::{from_record_batch, to_record_batch};

let batch = to_record_batch(&result)?;
let restored = from_record_batch(&batch)?;
assert_eq!(restored.vars, result.vars);
assert_eq!(restored.rows, result.rows);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Add the feature with `cargo add sparq-arrow --features arrow`.

## Use Parquet bytes

```rust,ignore
use sparq_arrow::{
    from_parquet_bytes, parquet_row_count_from_bytes, parquet_variables_from_bytes,
    to_parquet_bytes,
};

let bytes: Vec<u8> = to_parquet_bytes(&result)?;
let variables = parquet_variables_from_bytes(&bytes)?;
let row_count = parquet_row_count_from_bytes(&bytes)?;
let restored = from_parquet_bytes(&bytes)?;
assert_eq!(variables, result.vars);
assert_eq!(row_count, result.rows.len());
assert_eq!(restored.vars, result.vars);
assert_eq!(restored.rows, result.rows);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Add the feature with `cargo add sparq-arrow --features parquet`. Parquet is only a
serialization of the same RecordBatch projection; it does not use a second term
encoding. `parquet_variables_from_bytes` validates and reads only the stored schema,
while `parquet_row_count_from_bytes` reads the total row count from file metadata;
neither decodes row groups. `from_parquet_bytes` additionally rejects invalid RDF
lexical components while decoding rows.

## Use Arrow IPC stream bytes

```rust,ignore
use sparq_arrow::{from_ipc_bytes, ipc_variables_from_bytes, to_ipc_bytes};

let bytes: Vec<u8> = to_ipc_bytes(&result)?;
let variables = ipc_variables_from_bytes(&bytes)?;
let restored = from_ipc_bytes(&bytes)?;
assert_eq!(variables, result.vars);
assert_eq!(restored.vars, result.vars);
assert_eq!(restored.rows, result.rows);
# Ok::<(), Box<dyn std::error::Error>>(())
```

Add the feature with `cargo add sparq-arrow --features ipc`. The IPC stream carries the
same schema as the RecordBatch and preserves variable names for an empty result.
`ipc_variables_from_bytes` validates and reads that schema without decoding batches.

## Preserve the mapping

- A null outer struct is an unbound cell. It is distinct from a bound empty-string
  literal, whose `kind` is `"literal"` and `value` is `""`.
- Plain string literals carry the explicit
  `http://www.w3.org/2001/XMLSchema#string` datatype.
- Language-tagged literals use `language`; RDF 1.2 directional literals additionally
  use `direction` with `"ltr"` or `"rtl"`.
- RDF 1.2 triple terms use `kind = "triple"` and store their N-Triples lexical form in
  `value`.
- Numeric and temporal literals remain lexical strings plus datatype IRIs. This surface
  does not narrow them into native Arrow numeric or temporal columns.

Treat the Arrow batch, Parquet bytes, or IPC stream as a transport projection, not as a
canonical RDF serialization or an RDF document.

[GPT-5.6] Verified against `sparq-arrow` for beads `sq-r3cab` and `sq-kix7x`.
