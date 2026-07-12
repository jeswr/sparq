# Facet-count benchmark gap record — 2026-07

<!-- [GPT-5.6] sq-ywe8p -->

## Evidence now available

`crates/sparq-introspect/examples/facet_bench.rs` provides the sparq-only,
self-relative facet row. It exercises unfiltered, class-filtered, value-filtered,
combined-filter, selected-predicate, and bounded-distribution scenarios. An independent
full-scan grouping loop is the oracle; exact `FacetResponse` equality gates all timing
output.

The generated dataset and scenarios are deterministic. Elapsed work-box measurements
remain non-canonical and are intentionally not copied into this record.

## Remaining comparison gap

This slice does not claim competitor parity. The external Virtuoso faceted-browse
column and its quiet-box execution remain owned by `sq-hmd7l.49`. That run must use
equivalent filters and distributions, preserve result equality as a precondition for
timing, and record its environment in the comparative envelope.
