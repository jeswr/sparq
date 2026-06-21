# sparq-terse

> 🤖 **SPARQ agent** [OPUS-4.8] — an opt-in, **verifiable** LLM-ergonomic SPARQL
> surface: a *pre-parse* transpiler that always emits **canonical, conformant
> SPARQL** — it never touches the vendored `spargebra` grammar. The engine only ever
> runs standard SPARQL the agent can inspect. Research prototype (epic `sq-2m6zm`,
> design `research/llm-ergonomic-sparql-surface.md`, PR #1074); NOT published.

The motivating finding (design §1.1): the dominant first-shot failure in
text-to-SPARQL is **semantic grounding** (picking the wrong IRI), *not* verbosity.
So the high-value lever is `V("phrase")` concept resolution, behind a soundness
envelope — *a convenience that shows its work, never an oracle that hides it*.

## 🚀 Quickstart

Phase 1 — the verifiable transpiler skeleton (default build, lean: only `spargebra`):

```rust
use sparq_terse::terse_to_sparql;

// Canonical SPARQL passes through BYTE-IDENTICAL, after the silent-rewrite canary
// (re-parse under spargebra) guarantees the emission is conformant.
let exp = terse_to_sparql("SELECT ?s WHERE { ?s <http://ex/p> ?o }")?;
assert_eq!(exp.canonical_sparql, "SELECT ?s WHERE { ?s <http://ex/p> ?o }");

// A V("...") construct in the default build fails LOUDLY (needs the `vectors` feature).
assert!(terse_to_sparql("SELECT ?f WHERE { ?f <http://ex/about> V(\"cats\") }").is_err());
# Ok::<(), sparq_terse::TerseError>(())
```

Phase 2 — `V("phrase")` lexical-first concept resolution (`vectors` feature):

```rust,ignore
use sparq_terse::{terse_to_sparql_with, ResolveCtx};
use sparq_core::Graph;

let graph = Graph::load_str(turtle, "turtle")?;
let ctx = ResolveCtx::lexical(&graph);            // no model, no network (the default)
// embedder is consulted ONLY for the vector fallback (a phrase lexical linking misses):
let exp = terse_to_sparql_with(
    "SELECT ?f WHERE { ?f <http://ex/about> V(\"cardinality estimation\") }",
    &ctx,
    |_phrase| None,                                // lexical-only here
)?;
// canonical_sparql now has <iri> spliced in; every bind is echoed for the agent:
for r in &exp.resolutions {
    println!("V(\"{}\") -> <{}>  score {:.3} confidence {:.3} via {}",
             r.phrase, r.iri, r.score, r.confidence, r.method.as_str());
}
```

## ✨ Features

- **Always canonical, verifiable.** `terse_to_sparql` returns `Expansion {
  canonical_sparql, resolutions, warnings }`. `canonical_sparql` is standard SPARQL,
  is what runs, and is what the agent sees — there is no hidden rewrite path.
- **Silent-rewrite canary** (design §6.7). Every emission is re-parsed under the
  unmodified `spargebra` parser; a non-parsing output is `TerseError::CanaryFailed`,
  never handed back. Canonical input is emitted byte-identical (no silent rewrite).
- **`V("phrase")` lexical-first concept resolution** (`vectors` feature, design §3.3).
  The deterministic, no-model `sparq-nlq` lexical linker is the PRIMARY path; the
  staleness-guarded `sparq-vectors` search is a FALLBACK for genuinely fuzzy phrases.
- **The §6 soundness envelope, enforced.** Every bind echoes IRI + score + runner-up +
  confidence + method; below the confidence floor or inside the ambiguity margin `V()`
  refuses to bind (`TerseError::Unresolved` with candidates) — loud-fail beats
  silent-wrong; a stale vector store is a hard `TerseError::StaleStore` (§6.5).
- **Opt-in / lean-core by construction.** The default build depends only on
  `spargebra` (to re-parse its own output). The `vectors` feature is the only thing
  that pulls `sparq-core`/`sparq-nlq`/`sparq-vectors`. **Zero edits** to `sparq-core`,
  `sparq-engine`, or the vendored parser. The embedding model is the *caller's*
  explicit dependency (this crate never embeds free text itself; design §6/§9 Q5).

## 📚 Learn more

- Design record: `research/llm-ergonomic-sparql-surface.md` (PR #1074) — the lever
  ranking (§7), the soundness envelope (§6), and the falsifiable token/quality A/B
  (§5, a future bead) that gates broad adoption.
- The reused machinery: `crates/sparq-nlq/src/link.rs` (lexical entity linking),
  `crates/sparq-vectors/src/ann.rs` (`nearest_exact`, the staleness-guarded
  `nearest_term_exact_checked`), `crates/sparq-vectors/src/store.rs`
  (`VectorStore::check_graph`).
- Epic `sq-2m6zm` (dogfood sparq as a Project Knowledge Graph): this surface is
  `sq-leg8n` (Phase 1 skeleton + Phase 2 `V()`); Phases 3–6 (keyword layer, the A/B,
  server/CLI exposure) are tracked follow-ups.

## License

MIT — see the repository-root [`LICENSE`](../../LICENSE). © 2026 Jesse Wright.
