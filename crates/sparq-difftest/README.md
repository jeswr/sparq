<!-- [OPUS-4.8] sq-qcnn.4: internal-stub README for a publish=false crate. -->
# sparq-difftest

The **engine-independent value-normalisation** library for [sparq](../../README.md)'s
value-level multi-oracle differential fuzzer — node A of the DAG tracked by
bead `sq-qcnn.4`.

It normalises and compares SPARQL solution **values** across independent engines
("oracles") so a query that returns the right *number* of rows with a **wrong bound
value** can no longer slip past a cardinality-only cross-check. It covers: strict RDF
term equality (simple-literal ≡ `xsd:string`, case-insensitive language tags);
arbitrary-precision integer/decimal value equality and `xsd:double` `INF`/`NaN`;
`dateTime`/`date`/`duration` value comparison with timezone normalisation and the ±14h
indeterminate window; a SPARQL-Results-JSON reader; and the multiset + `ORDER BY`
equivalence-class comparators. Full API detail is in the rustdoc.

**Load-bearing constraint:** it depends on **no sparq crate** — only third-party exact
arithmetic (`num-bigint` / `bigdecimal`) plus explicit XSD rules — so it cannot launder
sparq's own value bugs through sparq's own comparator. Honest caveat: it is new code and
a bug surface of its own; the structural mitigation is a second independent oracle (a
later DAG node).

> **Internal tooling — not published** to crates.io (`publish = false`). It is a
> dev/test library; nothing in the shipping graph depends on it.
> Contributing: [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
