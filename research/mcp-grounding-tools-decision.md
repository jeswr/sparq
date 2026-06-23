# MCP grounding tools: `shapes` + `ask`, both opt-in (2026-06-23 design call)

**Status:** decision record, 2026-06-23. Records the choice made on the 2026-06-23 design
call (Jesse Wright, after debating Samu Lang) to add **both** grounding tools to the opt-in
`sparq-mcp` crate — the structured `shapes` tool **and** the natural-language `ask` tool —
as complementary additions. Beads: `sq-zak4f` (`shapes`), `sq-jxjgr` (`ask`). Implemented
[OPUS-4.8]. Proceed-and-document: this is a best-judgment design choice, not a credential
block; steer post-hoc if desired.

> 🤖 This record was written by a SPARQ agent.

## The two options on the table

Both tools answer the same user need — *"turn my natural-language question over this RDF
dataset into a correct SPARQL answer"* — but split the LLM call across the client/server
boundary differently.

| | `shapes` (sq-zak4f, Samu's lean) | `ask` (sq-jxjgr, Jesse's lean) |
| --- | --- | --- |
| Where the model runs | the **client's** LLM | **server-side** (configurable) |
| sparq-mcp ships a model? | no — structured JSON only | no — embeds a *configurable* call |
| Feature gate | none (default build) | opt-in `nlq` (OFF by default) |
| Output | the class's data-grounded shape (valid predicates, datatypes, cardinalities) | the executed SPARQL + real result rows (+ citations) |
| New dependency | none (reuses `sparq-introspect`) | `sparq-nlq` (under the feature only) |

## The disagreement (recorded honestly)

- **Jesse leaned server-side NL (`ask`).** Convenience: a client that already speaks MCP
  gets a one-call NL→answer round-trip without the client itself having to write SPARQL or
  understand RDF schema. The server owns the introspect→generate→validate→execute loop.
- **Samu argued the structured `shapes` tool suffices** and that `ask` is *redundant*: the
  client already has an LLM (`llm1`), so adding a second LLM server-side (`llm2`) is
  `llm1 ≈ llm2` — two models doing the same job. Hand the client the schema shape and let
  *its* model write the query; no server-side model, no `sparq-nlq` dependency, no key
  management, ships in the default build.

The `llm1 ≈ llm2` redundancy critique is real and is **not** dismissed. `shapes` is the
lean default precisely because of it: it adds capability with **no** model and **no** new
dependency. `ask` is the opt-in convenience for clients that would rather not do the
grounding themselves — it is strictly additive and OFF by default, so the redundancy is a
*choice the operator opts into*, never imposed.

## Decision: BOTH, opt-in

Add both, as complementary tools, respecting the opt-in-feature architecture:

1. **`shapes`** ships in the **default** `sparq-mcp` build. No LLM, no new dependency — it
   reuses the existing `sparq-introspect` miner to return one class's data-grounded
   SHACL-style shape (the valid predicates, their datatypes/object-kinds, observed range,
   and the cardinalities the data *proves* — `min_count`/`max_count` emitted only when
   established, never fabricated). This is Samu's lean, and it is the path we recommend
   first.
2. **`ask`** sits behind a new opt-in cargo feature `nlq` (OFF by default), wiring
   `sparq-nlq`'s loop. It degrades cleanly: with the feature ON but no LLM backend
   configured, the tool is unadvertised and a direct call returns a clear *"not
   configured"* error — never a fabricated answer, never a panic. This is Jesse's lean,
   kept off the lean default build.

## Honest framing (load-bearing)

- **No token-saving claim.** The project measured representation/token tricks as duds (the
  `V()` verdict). Neither tool is advertised as cheaper. Both are **ergonomics / grounding
  aids pending measurement**: `shapes` moves the model to the client; `ask` moves it to the
  server. Whether either improves end-to-end accuracy or cost over plain `introspect` +
  client-written SPARQL is an open empirical question — to be measured with the existing
  offline exec-accuracy harness (`sparq-nlq`'s `eval`), not asserted here.
- **`ask` embeds a configurable LLM call**, so its cost/quality depend entirely on the
  user-chosen model/endpoint (`ANTHROPIC_API_KEY`, or an OpenAI-compatible
  `SPARQ_NLQ_ENDPOINT_URL` + `_MODEL`). No model is bundled; nothing phones home.
- **`ask`'s answer is the query's real result rows**, not a free-form prose paragraph the
  model could distort — the loop validates (spargebra) and executes (the engine, under a
  `QueryBudget`) before any answer is returned.

## Follow-ups (beads, not TODOs)

- Measure whether `shapes`-grounded client SPARQL or server-side `ask` beats plain
  `introspect` on exec-accuracy / cost — capture as a bead before making any efficiency
  claim.
- An SSE/HTTP MCP transport (today only `stdio` + the embeddable `handle_message`) — already
  tracked separately; orthogonal to these tools.
