<!-- [OPUS-4.8] sq-leg8n: README brought to template. -->
# sparq-terse

<p>
  <a href="https://crates.io/crates/sparq-terse"><img src="https://img.shields.io/crates/v/sparq-terse.svg" alt="crates.io"></a>
  <a href="https://docs.rs/sparq-terse"><img src="https://docs.rs/sparq-terse/badge.svg" alt="docs.rs"></a>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

An **opt-in, verifiable** LLM-ergonomic SPARQL surface for the project knowledge graph
(design: `research/llm-ergonomic-sparql-surface.md`). The single contract is a
**pre-parse transpiler**: a terse / LLM-friendly query is expanded — as text — into
**canonical SPARQL** that the unmodified vendored `spargebra` parser accepts. It never
touches the grammar (which carries the W3C-conformance patches), so the engine only ever
sees standard SPARQL. Nothing in the workspace depends on this crate; the default build
does not compile it, and `sparq-core` / `sparq-engine` stay lean.

The governing principle is **opt-in, verifiable, loud-failing**: it is a convenience that
*shows its work*, never an oracle that hides it.

## 🚀 Quickstart

```rust
use sparq_terse::terse_to_sparql;

// Phase 1: an already-canonical query is a byte-for-byte identity pass-through
// (the "silent-rewrite canary" — no directive fires, nothing is rewritten).
let q = "SELECT * WHERE { ?s ?p ?o }";
let exp = terse_to_sparql(q).unwrap();
assert!(exp.is_identity(q));
assert_eq!(exp.canonical_sparql, q);
```

```rust,ignore
// Phase 2 (the `link` feature): V("phrase") resolves to the nearest concept IRI,
// LEXICAL-FIRST, and echoes the bind — IRI + score + runner-up + confidence + method.
use sparq_terse::Resolver;
let resolver = Resolver::new(&graph);
let exp = resolver
    .transpile(r#"SELECT ?f WHERE { ?f pkg:about V("cardinality estimation") }"#)?;
// exp.canonical_sparql now contains the resolved <IRI>; exp.resolutions documents it.
// An unknown or ambiguous phrase REFUSES (a loud TerseError::Unresolved with the
// candidate list) — it never silently binds a low-confidence guess.
```

## ✨ Features

- **Verifiable transpiler skeleton** (Phase 1, default build, `spargebra`-only) — always
  emits canonical SPARQL, re-parsed under `spargebra` before it is returned. A query with
  no terse directive is a pure identity pass-through; the **silent-rewrite canary** test
  asserts an already-canonical query comes back unchanged.
- **`V("phrase")` concept resolution** (`link` feature) — attacks *grounding* (the real
  first-shot bottleneck), not verbosity: the agent writes the concept it means instead of
  guessing an opaque IRI. **Lexical-first** — reuses `sparq-nlq`'s deterministic, no-model
  label match; only `link` pulls `sparq-nlq` (and the engine) into the graph.
- **The soundness envelope** (mandatory) — every resolution **echoes** its IRI + score +
  runner-up + confidence + method; resolution is **confidence-gated** (below the floor, or
  within the ambiguity margin, `V()` refuses and surfaces the candidates); it **never
  auto-accepts the uncertain**. Loud-fail beats silent-wrong.
- **Vector fallback** (`vectors` feature) — used **only** when lexical returns nothing.
  The caller injects any `sparq_vectors::Embedder` (this crate never opens a socket); the
  search runs through the **staleness-guarded** `check_graph` path, so a store built
  against a different graph generation is a hard error, never stale neighbours.
- **No lenient parsing** — by design (the research recommends against it): a malformed
  terse query fails *loudly* with the parser's own error, preserving the agent's
  recoverable feedback loop. There is no typo/alias tolerance and no silent rewrite.

## Honest status — what is and is not here

- **Phase 1 (skeleton) and Phase 2 (`V()` lexical-first + the envelope) are implemented**
  and tested (round-trip to canonical SPARQL; a known concept resolves; an ambiguous one
  refuses; the canary). The vector fallback wiring is in place behind `vectors`.
- **NOT implemented (tracked follow-ups):** the fixed keyword layer (Phase 3), the
  did-you-mean diagnostic (Phase 4), the scientific A/B + verdict object (Phase 5), and
  `sparq-server` / CLI exposure (Phase 6). The live phrase-embedder is the *caller's*
  dependency — this crate ships the seam, not a model.
- **No accuracy claim.** Whether the surface pays off in agent tokens / first-shot success
  is the Phase-5 measurement, not asserted here.

## 📚 Learn more

- **How-to** — [`skills/genai-retrieval/SKILL.md`](../../skills/genai-retrieval/SKILL.md).
- **API reference** — [docs.rs/sparq-terse](https://docs.rs/sparq-terse).
- **Design** — [`research/llm-ergonomic-sparql-surface.md`](../../research/llm-ergonomic-sparql-surface.md)
  (§3, §6 soundness envelope, §8 phases).
- **Lexical linking it reuses** — [`sparq-nlq`](../sparq-nlq); the vector path —
  [`sparq-vectors`](../sparq-vectors).
- **Contribute** — [`AGENTS.md`](../../AGENTS.md).

## License

[MIT](../../LICENSE).
