<!-- [GPT-5.6] sq-bu7zs — terse self-relative throughput gap record. Work-box
measurements are advisory and no timing is transcribed here. -->

# Gap record — terse expansion and compaction (2026-07)

**Axis:** `sparq-terse` keyword expansion and benchmark-local compaction.  
**Status:** deterministic self-relative harness delivered; smoke identity gate green.  
**Harness:** `crates/sparq-terse/examples/terse_bench.rs`.

## Scope and verdict

The verdict is **NOT-COMPARABLE externally**. Terse `K:` keywords are a sparq-specific
query-authoring convenience, not a standard RDF or SPARQL format, and no external peer
implements the same frozen legend. The harness therefore compares sparq's own expand and
compact directions only; throughput must not be presented as a competitor win.

This surface is a legibility convenience, **not a token-saver**. The existing adoption
record found that its only conditional token benefit depends on caching the legend card;
raw transform throughput does not change that conclusion and is not evidence of user value.

## Correctness envelope

No timing row is emitted unless every deterministic generated query passes a full identity
gate first:

- benchmark-local compaction replaces only exact IRIs from the frozen public legend;
- expanding the compact query must reproduce the canonical SPARQL byte-for-byte, which is
  stronger than parsed-query or RDF-term semantic equality;
- re-compacting that expansion must reproduce the same terse query; and
- a deliberately corrupted keyword must make the same gate fail, witnessing non-vacuity.

The timed pass is checked against the already-gated canonical corpus again before output.
The JSON envelope records the identity gate, mutation witness, NOT-COMPARABLE verdict,
input byte counts, docs/s, and MB/s. Shared-work-box output is non-canonical and must not be
copied into documentation.

## Reading the axis

Use the expand and compact rows as a self-relative regression signal only. A canonical
quiet-box run could stabilize that regression baseline, but it would not create a valid
external comparison or alter the legibility-only product verdict.
