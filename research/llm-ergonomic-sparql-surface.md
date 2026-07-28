# An LLM-ergonomic SPARQL surface for the project knowledge graph [OPUS-4.8]

> 🤖 SPARQ agent — design-for-maintainer-review (bead **sq-9cxo0**, under the dogfooding
> epic **sq-2m6zm**). No implementation here; this surveys prior art + the actual codebase
> and maps each candidate lever to an **opt-in** design with a scientific eval plan and
> explicit soundness guardrails. Every phase below is a future bead. Empirical claims are
> tagged `[established]` (peer-reviewed / first-party) / `[claimed]` (vendor / self-report)
> / `[measured-here]` (this repo) / `[uncertain]`. No performance numbers are frozen into
> this doc (`check-no-perf-numbers.py`).

**Status: design-only / proposed. Nothing here is adopted.**

---

## 0. The brief, and a correction to its premise

The brief asks: is there a SPARQL representation that is cheaper (fewer tokens) and
lower-error (fewer first-shot mistakes) for an **LLM agent** to write than raw SPARQL,
and can we design one as an **opt-in** surface for querying the PKG? Three candidate
levers are named to *evaluate, not assume*:

1. a curated **well-known-vocab keyword layer** beyond Turtle's `a` (RDFS/OWL/PROV-O/QUDT/
   xsd terms: `label`, `type`, `subClassOf`, `wasDerivedFrom`, datatypes…);
2. an opt-in **LENIENT parse mode** with typo/alias + shorthand tolerance
   (`FLTR`→`FILTER`, special-character operators);
3. **vector-backed concept resolution** `V("Cat")` that binds the nearest-concept IRI via
   `sparq-vectors` at parse/bind time.

### Note: `crates/sparq-kb` now exists — the Phase-0 dependency is satisfied

The brief (and the bead title) say the PKG lives in `crates/sparq-kb`. **It does** — this
record's earlier draft was written off a stale local `main` and wrongly claimed the crate
did not exist; corrected here. As of origin/main, `crates/sparq-kb` is built: the PKG
ontology + SHACL guardrail shapes and the ingestion PoC are **merged** (sq-2m6zm.1 /
sq-2m6zm.2, landed via PR #1069), shipping `ontology/pkg/pkg.ttl`, `shapes/pkg.shapes.ttl`,
the `ingest/ingest_pkg.py` pipeline, and an ingested `ingest/pkg-instances.ttl` graph. A
"query-the-PKG" skill (sq-2m6zm.3, PR #1075) is in flight on top of it. So the PKG is no
longer a design-only artefact: the ontology, the ingestion, and a real ingested graph all
exist as code. This matters for scoping — **this surface is downstream of an *existing*
PKG**, not a hypothetical one. The honest dependency order still holds: **PKG ingestion
(the data, now done) → this ergonomic surface (the query convenience) → the §5 A/B that
measures whether the convenience pays** — but its first link is satisfied, so the surface
can be prototyped now against the real `sparq-kb` graph. The surface itself remains a
*new* opt-in crate (call it `sparq-terse`) layered over the existing query stack; it does
not modify `sparq-kb`.

### The deeper reframe the brief invites — and the honest tension

Stepping back: the brief's frame is "make the *query language* terser." But the dominant
first-shot failure mode in the text-to-SPARQL literature is **not** verbosity — it is
**semantic grounding**: picking the wrong IRI for a predicate/class/entity `[established]`
(Text2SPARQL'25; §1.2). Terser syntax does nothing for grounding; in some readings it
*hurts* (a novel terse dialect is under-represented in pre-training, §1.4). So the levers
split cleanly: **lever 1 and lever 2 attack token-cost and syntax-error rate; lever 3
attacks grounding.** They are not the same problem and must be ranked separately. The
single most important finding of this record is that **the codebase already ships the
grounding machinery** (`sparq-nlq::link`, `constrain`, `sparq-vectors::vec:`), so the
highest-value move is to *expose what exists* as a token-cheap, verifiable surface — not to
invent a new dialect.

---

## 1. Prior art — surveyed, cited, with the honest verdicts

### 1.1 LLM text-to-SPARQL and its failure modes

The NL→SPARQL loop (embed schema → retrieve → generate → validate → execute → verbalise)
is mature and well-studied. The recurring, **named** failure modes `[established]`:

- **Out-of-schema predicate / class hallucination** — the LLM emits a plausible IRI
  (`dbo:director`) that does not exist in the target KG; the query parses, executes, and
  returns a silent empty or mis-grounded result. The single most-cited fix is **barring
  out-of-schema predicates with an automated correction message** ("predicate X invalid;
  valid predicates for class C are …") — Emonet et al., *LLM-based SPARQL Query Generation
  over Federated KGs* (arXiv:2410.06062, >90% F1 via schema-validation + decomposition);
  and the post-generation-memory-retrieval line (arXiv:2502.13369) which retrieves
  ground-truth URIs after generation to repair hallucinated ones.
- **URI hallucination proper** — inventing IRIs absent from the KG (the same mechanism,
  for entities not just predicates).
- **Relation-inversion / reversal curse** — the model knows `A directs B` but emits the
  edge backwards; an under-recognised first-shot error.
- **Schema-grounding is the bottleneck** — the Text2SPARQL'25 challenge (ESWC 2025) named
  "accurately mapping ambiguous NL phrases to the precise canonical IRIs of classes and
  properties" as *the* critical challenge; the winning systems (mKGQAgent, ARUQULA —
  arXiv:2510.02200) are **agentic ReAct loops with explicit entity-linking + KG-exploration
  tools + a verify step**, not terser query syntax.

**Verdict for this brief:** the literature's lever of choice for first-shot success is
**schema-constrained generation + repair** (our lever-3 family + the existing
`constrain.rs`), *not* a terser dialect. The bioinformatics result (Oxford *Bioinformatics*
2026, btag174) is blunt: "most improvement stems from barring out-of-schema predicates and
automated correction messaging." That is already what `sparq-nlq::constrain` does.

### 1.2 Controlled-natural-language query (SQUALL, Sparklis, GF)

A 20-year line predates LLMs:

- **SQUALL** (Ferré) — a controlled NL that exposes the *full* expressiveness of SPARQL 1.1
  (superlatives → solution modifiers, coordinations → relational algebra, comparatives →
  filters), defined as a Montague grammar and translated to SPARQL by a web service. It is
  *more verbose* than SPARQL, not less — it optimises for human readability/authorability,
  not token count.
- **Sparklis** (Ferré, *Semantic Web* 8(3), 2017) — an *interactive faceted query builder*
  that guides the user through valid choices so **no invalid query is constructible** and
  no SPARQL knowledge is needed. The query is verbalised in English/French.

**The load-bearing lesson from Sparklis:** its power is **guidance by construction** — at
every step the tool offers only the *schema-valid* next moves, so grounding errors are
structurally impossible. That is the inverse of lenient parsing (which accepts more) — it
is *constrained* offering (which accepts only valid). For an LLM agent the analogue is
**schema-grounded generation + a validator that rejects out-of-schema terms** — again the
`constrain.rs` pattern, and again the opposite of lever 2. CNLs are a strong *human*
ergonomics result with weak read-across to *token-cost-for-an-LLM* (they trade more tokens
for zero training-data risk).

### 1.3 Terse RDF dialects & default-vocab keyword schemes

The brief's lever 1 has direct precedents:

- **Turtle `a`** — the one universally-known keyword shorthand for `rdf:type`. It works
  *because every LLM has seen it millions of times*. That is the bar a new keyword must
  clear.
- **JSON-LD `@vocab`** — sets a default vocabulary IRI so bare terms expand against it; the
  closest standard analogue to "a default well-known-vocab layer." `@vocab` is *not* a
  keyword set — it is a single base IRI for un-prefixed terms.
- **`PREFIX`/`@prefix`** — the standard, model-familiar IRI-abbreviation mechanism. An LLM
  writing `prov:wasDerivedFrom` after a `PREFIX prov:` line is already terse *and*
  unambiguous *and* in-distribution.

**The crucial nuance:** SPARQL *already has* a terse, low-ambiguity, in-distribution way to
avoid writing full IRIs — prefixed names. A bespoke keyword layer (`derivedFrom` →
`prov:wasDerivedFrom`) saves a handful of characters over `prov:wasDerivedFrom` but trades
**zero training-data familiarity** for it. The win is real only if (a) the agent would
otherwise have to *emit and get-right* the PREFIX declarations (a token + error cost), and
(b) the keyword set is small, fixed, and supplied in-context. See §1.4 for why this is a
knife-edge.

### 1.4 Does a novel terse dialect help or hurt an LLM? (mixed evidence — read carefully)

This is the pivotal question for levers 1 and 2, and the evidence is **genuinely mixed**,
not one-sided:

- **The "hurts" side** `[established]`: Microsoft DevBlogs (*AI Coding Agents and DSLs*) —
  models lack syntax blueprints for under-represented languages and **"guess and synthesize
  non-existent constructs"**; custom assignment operators and non-standard syntax "violate
  assumptions embedded in mainstream patterns." The survey on low-resource/DSL code-gen
  (arXiv:2410.03981) confirms higher hallucination + syntax-error rates for DSLs with many
  custom names. A terse SPARQL dialect with novel keywords and special-char operators is
  exactly such a DSL.
- **The "helps" side** `[established]`: the *Anka* DSL paper (arXiv:2512.23214) reports a
  model achieving **99.9% parse success on a DSL it had zero training exposure to**, from
  the prompt alone; *DSL or Code?* (arXiv:2601.00469) found a constrained DSL **beat Python
  by +4.6pp overall accuracy (and +40pp on multi-step pipelines)** *despite* Python's
  training-data advantage — because constraint removes degrees of freedom to err.

**Reconciling them — the design-deciding rule:** a new dialect helps **iff** it is (i)
*small and constrained* (removes error degrees of freedom rather than adding novel ones),
(ii) *anchored by 3–5 in-context worked examples* (the Microsoft + Anka precondition), and
(iii) *validated with errors fed back* (compiler-in-the-loop). It hurts when it is a grab-bag
of novel keywords/operators with no anchoring. **This directly demotes lever 2** (lenient
typo/alias tolerance *adds* degrees of freedom and novelty — the anti-pattern) and
*conditionally* supports lever 1 (a *small, fixed, example-anchored* keyword set is the
"constrained DSL" shape that can help). The keyword set must be tiny and stable, never a
sprawling alias dictionary.

### 1.5 Lenient parsing / error recovery — the compiler literature is a warning

The compiler-design literature on lenient parsing and error recovery is consistent and
**cautionary** `[established]`: error recovery "can immediately print out spurious syntax
errors that aren't any fault of the programmer," **mask real errors**, and cause
**cascading errors** so "experienced programmers scroll back to the first error, ignoring
the repairs." The modern resilient-parsing work (Diekmann & Tratt, *Don't Panic!*,
arXiv:1804.07133; Kladov's resilient-LL tutorial) is about producing a *usable AST for IDE
tooling despite* errors — explicitly **not** about silently *accepting* a wrong program as
correct. For a *query* (vs an editor buffer) the danger is sharper: a lenient parse that
"helpfully" rewrites `FLTR` → `FILTER` or guesses an operator can produce a **different,
valid, silently-wrong query** the agent never sees. This is the soundness crux of §6.

### 1.6 What other LLM↔KG and coding-agent systems actually do

- **Coding agents abandoned magic and embraced tool-driven exploration + validation.** The
  cross-industry pattern (mirrored in this repo's own `agent-effectiveness-program.md`): the
  reliable levers are **structured context + curated examples + compiler/validator-in-the-loop
  + tool-search/deferred tool loading**, not bespoke terse syntaxes. DSL support improves via
  "supply knowledge and impose structure," not via inventing terseness.
- **Token efficiency is overwhelmingly a *caching* problem, not a *syntax* problem.** The
  cited AGENTS.md-consolidation study (−16.58% output tokens / −28.64% runtime,
  arXiv:2601.20404 `[independent]`) and this repo's whole `agent-efficiency-tooling.md`
  thesis: the big lever is **prompt-cache hygiene** (reads at ~0.1×, writes at 1.25–2×).
  A schema card / keyword legend / few-shot examples that sit *behind the cache breakpoint*
  cost ~0.1× on warm turns; the same content re-billed per turn erases any query-body
  saving. **Any token claim for these levers is dominated by where the legend lives in the
  cache, not by how terse the query body is.** This is the §5 measurement's central trap.

---

## 2. What the codebase ALREADY ships (verified in-tree) — the load-bearing finding

I read the actual source. Two of the three levers are **already substantially implemented**
for the NL→SPARQL path; the surface this bead proposes is largely *re-exposing* them.

| Capability | Where (verified) | Maturity | Relation to a lever |
|---|---|---|---|
| **SPARQL parser** | `vendor/spargebra` (spargebra 0.4.6 + surgical W3C patches, `SPARQ-PATCHES.md`) | production | levers 1+2 would touch the *grammar* if done in-parser — **do not** |
| **Entity/relation linking (lexical, no model)** | `sparq-nlq/src/link.rs` (548 ln) | built | a **non-vector** form of lever 3: maps mentions → IRIs via label index + IDF ranking + `sparq-sim` sibling expansion |
| **Dictionary-grounded constraint + did-you-mean** | `sparq-nlq/src/constrain.rs` (771 ln) | built | a form of lever-2's *value*: post-parse, walks algebra, flags out-of-dict predicates/classes with nearest-term repair hints — **at the IRI level, soundly** |
| **`vec:` magic predicate (k-NN in SPARQL, zero engine change)** | `sparq-vectors/src/rewrite.rs` (`query_vec`, `vec:nearest`/`vec:search`) | built | the engine half of lever 3's `V()` — already exists |
| **Entity embedding + verbalisation** | `sparq-vectors/src/{labels,verbalize}.rs` (`embed_labels`, `verbalize`) | built | turns entities into embeddable text for `V()` |
| **Staleness-checked nearest-term** | `sparq-vectors/src/ann.rs` (`nearest_term_exact_checked`, sq-32i5) | built | the *soundness* primitive for `V()`: `Err` if store built against a different graph generation |
| **Token-budgeted schema card** | `sparq-introspect/src/lib.rs` (`to_text_summary`) | built | the in-context legend that anchors any terse surface |

**Two consequences:**

1. **Levers 1 and 2 must NOT touch the vendored parser.** `vendor/spargebra` carries
   surgical W3C-conformance fixes prepared as upstream PRs; forking its grammar for a terse
   dialect would corrupt that conformance story and couple an ergonomic experiment to the
   engine's correctness substrate. Both levers are realisable as a **pure pre-parse textual
   expansion** (terse-source → canonical SPARQL string → unmodified spargebra) in a new
   opt-in crate. This keeps `sparq-core`/`sparq-engine` and the vendored parser lean and
   untouched.
2. **Lever 3 (`V()`) is ~70% built.** `vec:nearest` already does string→vector→k-NN inside
   SPARQL; `embed_labels`/`verbalize` build the index; `nearest_term_exact_checked`
   provides the staleness guard. The genuinely-new parts are (a) a parse-time `V("…")`
   sugar that expands to a `vec:`-style bind, (b) the **live embedder dependency** to embed
   the query phrase at bind time (today `vec:nearest` takes a pre-computed vector string;
   embedding `"Cat"` needs a model), and (c) the **echo-the-resolved-IRI-and-confidence**
   verification envelope (§6). The hard part of `V()` is not the search — it is honesty.

---

## 3. The three levers, each mapped to a concrete opt-in design

All three live in **one new opt-in crate, `sparq-terse`** (working name; `publish` per the
repo's opt-in-feature discipline), feature-gated, depending only on `spargebra`
(pre-parse) + optionally `sparq-nlq`/`sparq-vectors`. **Zero changes to `sparq-core`,
`sparq-engine`, or `vendor/spargebra`.** The crate's single public contract is a
*transpiler*: `terse_to_sparql(src, ctx) -> Result<Expansion, TerseError>` where
`Expansion { canonical_sparql, resolutions: Vec<Resolution>, warnings }` — it **always
returns the canonical SPARQL** so the agent (and the engine) only ever execute standard,
conformant SPARQL. Nothing is ever executed that the agent cannot inspect.

### 3.1 Lever 1 — well-known-vocab keyword layer (CONDITIONAL ADOPT, narrow)

**Design.** A *small, fixed, curated* keyword→IRI table for the highest-frequency PKG terms
only, expanded pre-parse. Scope it to the PKG ontology's actual hot predicates (from
`research/dogfooding-sparq-knowledge-graph.md` §2): `type`(`a`), `label`, `subClassOf`,
`derivedFrom`(`prov:wasDerivedFrom`), `generatedBy`, `about`, `confidence`, `assurance`,
`dependsOn`, `status`, the xsd datatypes. Mechanism: a default `@vocab`-style base plus an
explicit, in-context-published legend; tokens that match the legend expand, everything else
passes through as ordinary SPARQL (so prefixed names still work).

**Why it might help (per §1.4):** it is the *constrained-DSL* shape — tiny, fixed, removes
the need to emit + get-right PREFIX lines and long property IRIs, and is trivially
example-anchored. It composes with caching: the legend lives behind the cache breakpoint.

**Why it might not (be honest):** prefixed names already deliver most of the terseness *in
distribution*; the marginal saving over `prov:wasDerivedFrom` is small; and the keyword set
must be frozen and published or it becomes a novel-DSL liability. **This is a knife-edge
lever — adopt only if §5 measures a real, cache-discounted win on a realistic task mix.**
Hard guardrails: (i) the keyword table is **fixed and versioned** (no per-user aliases);
(ii) the expansion is **always echoed** in `canonical_sparql`; (iii) a keyword that
*collides* with a real prefixed name is a hard parse error, never a silent guess.

### 3.2 Lever 2 — lenient parse mode (RECOMMEND AGAINST as designed; one safe sliver)

**The honest verdict: lever 2 as described (typo/alias tolerance, `FLTR`→`FILTER`,
special-char operators) is likely NET-NEGATIVE and should not ship.** Reasons, traced to
evidence:

- **It is the anti-pattern from §1.4** — it *adds* novelty and degrees of freedom (the
  thing that *raises* DSL hallucination), the opposite of the constrained-DSL shape that
  helps.
- **It is the cautioned pattern from §1.5** — lenient acceptance silently rewrites the
  agent's intent; a `FLTR` that becomes `FILTER`, or a guessed operator, can yield a
  *different, valid, wrong* query. For a query (not an editor buffer) this is a soundness
  hazard, not a convenience.
- **It masks the agent's real errors.** A typo in a SPARQL keyword is a *loud, recoverable*
  signal: the parse fails, the agent sees the error, and fixes it on the next turn — cheaply
  and correctly. Silently "fixing" it removes the agent's feedback loop and can entrench a
  misunderstanding. (This is the compiler-literature "scroll back to the first error"
  problem applied to an agent.)
- **The token saving is negligible** — `FLTR` vs `FILTER` is one token at most, and an LLM
  rarely *mistypes* a keyword it has seen billions of times; keyword typos are not a real
  first-shot failure mode (grounding is — §1.1).

**The one safe sliver, if anything:** a **strict, non-rewriting "did-you-mean" diagnostic**
on *parse failure only* — i.e. when standard SPARQL fails to parse, return a *suggestion*
("unknown keyword `FLTR` — did you mean `FILTER`?") **without** auto-applying it. This is
the loud-fail-with-a-hint pattern (exactly `constrain.rs`'s existing did-you-mean, lifted to
the keyword level), and it preserves the agent's feedback loop. It is *diagnostics*, not
*lenient parsing*. Even this is low-value (keyword typos are rare) and should be a small
afterthought, never a mode that accepts wrong input.

**Do not** implement special-character operator aliases or alias dictionaries: maximal
novelty, maximal silent-rewrite risk, minimal saving.

### 3.3 Lever 3 — vector-backed concept resolution `V("Cat")` (HIGHEST VALUE — but the highest soundness burden)

**Design.** `V("phrase")` is parse-time sugar that, given a built label/entity vector index
over the PKG, resolves `phrase` to the nearest concept IRI and binds it. It expands to the
existing `vec:`-style machinery so the engine stays unaware:

```sparql
# terse:
SELECT ?finding WHERE { ?finding pkg:about V("cardinality estimation") }
# expands (illustrative) to canonical SPARQL the agent sees and the engine runs:
SELECT ?finding WHERE {
  ?finding pkg:about ?c0 .
  # resolved at bind time; echoed back with IRI + score:
  VALUES ?c0 { <topic:cardinality-estimation> }   # V#0 -> score s, runner-up <…> score s'
}
```

**Reuse, not reinvent (§2):** binding uses `embed_labels`/`verbalize` (build) +
`nearest_term_exact_checked` (the staleness-guarded search) + the `vec:nearest` rewrite
path. The new parts: the `V("…")` parse-sugar, the **live embedder** to vectorise the
phrase (today `vec:nearest` consumes a pre-computed vector — embedding a free-text phrase at
bind time requires a model, an explicit opt-in network/dep cost), and the verification
envelope (§6).

**Why it is the highest-value lever:** it attacks **grounding** — the *actual* first-shot
bottleneck (§1.1) — not syntax. It lets the agent write the *concept it means* instead of
guessing an opaque content-addressed PKG IRI it cannot know a priori. This is the lever the
text-to-SPARQL SOTA actually invests in (entity linking).

**Why it is also the highest-risk lever:** a `V()` that resolves to the **wrong** IRI
produces a **plausible-but-wrong answer** with no syntactic signal. This is the silent-drift
failure mode named throughout this repo's own vector research. It is *only* acceptable
behind the §6 envelope: opt-in, always echo the resolved IRI + score + runner-up + a
confidence, never auto-accept a low-confidence bind, and prefer the existing **lexical**
`link.rs` (exact-label match, no model, deterministic) when it fires — falling back to
vectors only for genuinely fuzzy phrases. **Lexical-first, vector-fallback** is the sound
default: it keeps the no-model, no-network, deterministic path primary and pays the model +
silent-drift risk only when lexical linking returns nothing.

---

## 4. Opt-in / lean-core architecture (how it stays additive)

Per the repo's opt-in-feature discipline (`MEMORY: feedback-opt-in-feature-architecture`):

- **New crate `sparq-terse`**, default-feature-empty. Pre-parse expansion (levers 1 + the
  safe sliver of 2) depends only on `spargebra` for *re-parsing the canonical output*
  (validation), not for grammar changes.
- **Lever 3 behind a `vectors` feature** that pulls `sparq-vectors` (+ the linking from
  `sparq-nlq`); the live embedder behind a further `live`/`embed` feature mirroring
  `sparq-nlq`'s existing `live` gate — OFF by default, fixtures/record-replay in CI.
- **Zero edits** to `sparq-core`, `sparq-engine`, `vendor/spargebra`. The engine only ever
  sees standard SPARQL. This is verifiable by a CI check that the default build of the core
  crates is byte-identical with and without `sparq-terse` present.
- **Exposure** rides the existing `sparq-server`/CLI wiring gap (the same gap
  `dogfooding-…md` §3.2 flags for nlq/introspect): the transpiler is the natural thing to
  expose, returning `{canonical_sparql, resolutions, warnings}` so the *contract is the
  verifiable expansion*, not an opaque answer.

---

## 5. Scientific evaluation plan (per dogfooding §5; pre-registered, falsifiable)

The metrics the brief names — **query-writing token cost** AND **first-shot query success /
error rate** — both must be measured against **plain SPARQL** on the *same* PKG and task
mix, with pre-registered kill-criteria. Build on the existing instruments; do not reinvent.

### 5.1 Instruments (already in-tree — reuse)

- `scripts/agent-telemetry/agent_telemetry.py` — cache-aware token accounting (the §5
  engine of the dogfooding doc and `agent-effectiveness-program.md`). Diff two JSON reports.
- `sparq-nlq`'s record/replay trait — pin model completions so both arms see identical model
  behaviour (defeats the model-drift confound).
- The dogfooding §5 effective-token formula:
  `effective_input = 1.0*fresh + 0.1*cache_read + 1.25*cache_write`.

### 5.2 Arms (counterbalanced, interleaved within-task A/B over a FROZEN corpus)

For each task `t` in a stratified frozen set, three arms over the *same* ingested PKG:

- **A (plain SPARQL):** the agent writes raw SPARQL (with PREFIX lines) against the schema
  card.
- **B (terse, levers 1 + 2-sliver):** the agent writes terse-keyword SPARQL; the legend
  sits **behind the cache breakpoint**.
- **C (terse + `V()`):** as B plus `V("…")` concept resolution.

Counterbalance arm order and warm/cold cache state. **Charge each arm all its costs**: arm
B/C pay the legend tokens (at their *true* cache multiplier — the load-bearing honesty
check), every repair round-trip, and for C the embedder call + per-`V()` resolution tokens
**plus** an amortised slice of one-time index build `(embed)/N`.

### 5.3 The two primary metrics + their measurement

1. **Query-writing token cost** — the agent's *output* tokens to author the query, **plus**
   the cache-discounted *input* delta from the legend/examples it must carry. Report
   nominal AND effective, and the components (a "win" that is purely the cache discount, or
   that ignores the legend tax, is flagged invalid). Churn-normalise (tokens per
   correct-query) so "fewer tokens by writing a wrong query" cannot win.
2. **First-shot query success / error rate** — a *composite*, reported sub-flag-by-sub-flag
   (mirroring `agent-effectiveness-program.md` §1.3), each derived independently, never
   self-reported:
   - `parses` — the (expanded, for B/C) query parses under spargebra;
   - `grounded` — no out-of-schema predicate/class (reuse `constrain.rs::unknown_terms`);
   - `answer_correct` — answer-set F1 vs held-out gold on a pinned corpus snapshot;
   - `first_try` — all of the above on the **first** generated query, no repair round.

   For C specifically add a **resolution-correctness** sub-flag: did each `V()` bind the
   *gold* IRI? (the silent-drift detector — a C arm that "succeeds" by binding the wrong
   concept to a tolerant gold must be caught).

### 5.4 Pre-registered significance bar + kill-criteria (declare BEFORE running)

A lever is **adopted** only if, on ≥30 stratified tasks (strata: point-lookup, multi-hop,
synthesis, negative/out-of-KG existence):

- **token win:** paired-median effective-token reduction **≥20%** (below the noise floor of
  the one independent 15–28% memory benchmark) **AND** Wilcoxon `p<0.05` **AND** bootstrap
  95% CI on the median delta excludes 0 **AND** the win is not *solely* the cache-discount
  component; **AND**
- **quality non-regression:** first-shot composite (esp. `answer_correct` and, for C,
  `resolution_correctness`) lower-CI-bound **≥** arm A — a token saving bought with worse or
  silently-wrong answers fails.

**KILL criteria (mechanical):**
- **KILL-token:** median effective reduction `<20%`, OR `p≥0.05`, OR CI includes 0, OR the
  saving is entirely cache-discount, OR (for B) it inverts once the legend tax is charged at
  its production cache multiplier (the prefix-tax trap) → do not adopt for token reasons.
- **KILL-quality:** any arm raises hallucination/out-of-schema rate, OR drops
  `answer_correct` lower-CI below arm A, OR (C) `resolution_correctness` < a pre-registered
  floor → reject regardless of token saving.
- **KILL-lever-2:** if the lenient mode produces **any** silently-different-but-valid query
  vs the agent's intent on the canary set (§6) → automatic reject (soundness is not
  tradeable). This is why §3.2 recommends against it before measuring: the kill condition is
  near-certain by construction.

All numbers are **work-box / non-canonical** (`MEMORY: project-ec2-execution-env`) — they
feed the verdict object at runtime, never frozen into committed markdown. Committed
artifacts: the harness, fixtures, the pre-registration, the verdict schema.

### 5.5 The verdict object (decide on the object, not a gut read)

```json
{ "lever": "keyword|lenient|V",
  "token_delta_median_pct": float, "token_delta_ci": [lo, hi], "token_win": bool,
  "legend_tax_charged": bool, "win_survives_cache_discount": bool,
  "first_shot": { "parses": float, "grounded": float, "answer_correct_f1": {...},
                  "resolution_correctness": float|null },
  "silent_rewrite_count": int,   // lever-2 canary; must be 0
  "honest": bool, "recommend_adopt": bool }
```

`recommend_adopt = true` requires `token_win AND win_survives_cache_discount AND
legend_tax_charged AND quality non-regression AND silent_rewrite_count==0`.

---

## 6. Soundness / guardrails — the non-negotiable core

Lenient parsing and concept-guessing can **silently produce wrong queries**. A `V()`
resolving to the wrong IRI yields plausible-but-wrong answers with no syntactic signal. The
design is sound **only** under all of the following, all of which are opt-in and none of
which is ever silent:

1. **Echo the canonical expanded query, always.** The transpiler's contract returns
   `canonical_sparql` — standard, conformant SPARQL — which is what executes and what the
   agent must be shown. There is no path where the engine runs something the agent cannot
   read. (This is *stronger* than lenient parsers, which hide the rewrite.)
2. **Echo every resolution with IRI + score + runner-up + confidence.** Each `V("phrase")`
   returns `Resolution { phrase, iri, score, runner_up, runner_up_score, method:
   Lexical|Vector }`. The agent can verify the bind, and a close runner-up is a visible
   ambiguity flag.
3. **Confidence-gated, never auto-accept the uncertain.** Below a pre-registered confidence
   floor, or when score and runner-up are within a margin (ambiguous), `V()` **does not
   bind** — it returns the candidate list as an error/`needs-disambiguation` signal, not a
   silent guess. Loud-fail beats silent-wrong.
4. **Lexical-first, vector-fallback.** Prefer `link.rs`'s deterministic exact-label match
   (no model, no drift); use vectors only when lexical returns nothing; mark `method` so the
   agent knows which fired. This minimises the silent-drift surface.
5. **Staleness guard mandatory.** Use `nearest_term_exact_checked` — a `V()` over a vector
   store built against a different graph generation must `Err`, never return stale neighbours
   (`sq-32i5` already provides this).
6. **Lever 2 never rewrites.** Per §3.2: no lenient acceptance of wrong input; at most a
   *diagnostic suggestion on parse failure*, never an auto-applied repair.
7. **A "silent-rewrite canary" in CI.** A fixture set of terse queries with *known intent*;
   the test asserts the canonical expansion matches the intended SPARQL exactly and that no
   input ever produces a *different-but-valid* query without surfacing it. This is the
   mechanical KILL-lever-2 gate (§5.4) and the regression guard for levers 1+3.

The governing principle: **opt-in, verifiable, loud-failing.** The surface is a
*convenience that shows its work*, never an oracle that hides it.

---

## 7. Ranking the levers by value / risk (non-sycophantic)

| Rank | Lever | Value | Risk | Verdict |
|---|---|---|---|---|
| **1** | **`V()` concept resolution (lever 3), lexical-first** | **High** — attacks grounding, the *real* first-shot bottleneck; ~70% built; reuses `link`/`vec:`/staleness-guard | High (silent drift) — *contained* by §6 | **Build behind §6 + measure.** The one lever the SOTA actually invests in. |
| **2** | **Keyword layer (lever 1), tiny+fixed+anchored** | Low–Medium — modest terseness over prefixed names; constrained-DSL shape *can* help | Low–Medium (novel-DSL risk if it grows) | **Conditional — measure first.** Adopt only on a real cache-discounted §5 win; freeze the keyword set. |
| **3** | **Lenient parse / typo-alias (lever 2)** | Very low — keyword typos aren't a real failure mode; ~1-token saving | **High** — adds novelty *and* silently rewrites intent; masks the agent's loud, recoverable errors | **Recommend against.** Ship at most a *non-rewriting did-you-mean diagnostic on parse failure*. The lenient mode itself is likely net-negative; the canary KILL is near-certain. |

**Cross-cutting honest caveat:** the *biggest* token lever for all of this is **cache
hygiene + a token-budgeted schema card**, not query terseness (§1.6) — and these are
*already* the levers `agent-effectiveness-program.md` / `agent-efficiency-tooling.md`
identify. If the §5 A/B shows the terse surface's whole "win" is really the schema card +
caching (which arm A also gets), the honest conclusion is "expose the schema card and
linking; skip the dialect." Be prepared for that outcome — it is the most likely one for
levers 1 and 2.

**Top recommendation:** invest in **lever 3 (`V()`), lexical-first with the §6 envelope**,
because it is the only lever that targets the measured first-shot bottleneck (grounding),
it overwhelmingly reuses already-shipped, already-soundness-guarded machinery, and its
risk is fully containable by "echo the resolution + confidence-gate + never auto-accept the
uncertain." Treat lever 1 as a measure-first conditional and lever 2 as recommend-against.

---

## 8. Phased plan (each phase = a future bead)

Ordered; each gated on its predecessor.

1. **Phase 0 — dependency: PKG ingestion (SATISFIED).** This surface is downstream of the
   `sparq-kb` ingestion PoC + a real PKG graph — which now exist on `main` (sq-2m6zm.2,
   PR #1069: `crates/sparq-kb` with `ontology/pkg/pkg.ttl` and an ingested
   `ingest/pkg-instances.ttl`). There is a real graph to query terser, so this dependency
   is no longer a blocker and Phase 1 can start immediately.
2. **Phase 1 — the verifiable transpiler skeleton + soundness harness (do this first,
   cheaply).** New `sparq-terse` crate: `terse_to_sparql -> Expansion {canonical_sparql,
   resolutions, warnings}`; identity pass-through (canonical SPARQL in → out unchanged); the
   §6.7 silent-rewrite canary CI fixture. No keyword table, no `V()` yet — just the
   *contract that everything echoes its expansion*. Markdownlint/clippy-clean, opt-in,
   zero core edits. *(Future bead.)*
3. **Phase 2 — `V()` lexical-first concept resolution + the §6 envelope.** Wire
   `link.rs` (lexical) → `nearest_term_exact_checked` (vector fallback, behind `vectors`
   feature; embedder behind `live`/`embed`, OFF by default). Emit `Resolution{iri, score,
   runner_up, confidence, method}`; confidence-gate; staleness-guard. Record/replay
   fixtures. *(Future bead — the top-recommendation deliverable.)*
4. **Phase 3 — the small fixed keyword table (lever 1).** Scope to the PKG hot predicates;
   freeze + version it; publish the legend as an in-context card behind the cache
   breakpoint; collision-with-prefix is a hard error. *(Future bead — conditional on Phase 5
   measuring a win.)*
5. **Phase 4 — the did-you-mean *diagnostic* (the only sliver of lever 2).** On parse
   failure only, suggest the nearest keyword without applying it. *(LANDED — `sq-h7zlx`:
   `TerseError::CanaryFailed` carries `KeywordSuggestion` hints, computed only on the failed
   parse and never applied. Shipped on the "cannot change a query, costs nothing on the
   success path" argument, NOT on evidence: the Phase-5 A/B measured levers 1 and 3 only, so
   the §3.2 "low value" reading stands unmeasured.)*
6. **Phase 5 — the scientific A/B (§5) + verdict object.** Stratified frozen task set over
   the real PKG; counterbalanced A/B/C; cache-discounted effective tokens + first-shot
   composite + resolution-correctness; pre-registered thresholds; emit the §5.5 verdict per
   lever. **Adopt each lever only on its verdict.** *(Future bead — the gate on broad
   adoption.)*
7. **Phase 6 — expose the transpiler over `sparq-server`/CLI** (closes the same wiring gap
   the dogfooding doc flags), returning `{canonical_sparql, resolutions}` so the network
   contract is the verifiable expansion. *(Future bead — gated on Phase 5.)*

**Discipline throughout:** opt-in/feature-gated; zero core/parser edits; synthetic fixtures
+ committed harness + a verdict object as outputs; no hard-coded perf numbers; `[OPUS-4.8]`
markers; discovered work captured as beads.

---

## 9. Open questions that genuinely need the maintainer

1. **Is the PKG real enough yet to measure against?** This surface is downstream of
   dogfooding Phase 1. Worth building the *transpiler skeleton* now (cheap, useful
   regardless), but Phases 3–5 need an ingested PKG. Sequence accordingly?
2. **`V()` syntax & expansion target.** `V("phrase")` as `VALUES`-bind vs reusing the
   `vec:nearest` predicate form directly — preference? And the confidence-floor + ambiguity
   margin values are pre-registration knobs you should set.
3. **Keyword table ownership.** If lever 1 proceeds, the frozen keyword set should be a
   maintainer decision (it is effectively a mini-vocabulary contract); align it with the
   PKG ontology terms in `dogfooding-…md` §2.
4. **Is lever 2 dead?** I recommend against the lenient mode and for only a non-rewriting
   diagnostic. Confirm — if you want the lenient mode explored despite the soundness
   argument, it must be gated behind the §6.7 canary with the KILL-lever-2 condition armed.
5. **Live embedder choice for `V()`.** The phrase-embedding model is a new dependency (the
   honest cost of vector concept resolution). Reuse `sparq-nlq`'s `live` Anthropic path, a
   local SentenceTransformer, or stay lexical-only until a model is justified by measurement?

---

## Citations

**Codebase (verified in-tree):**
- `vendor/spargebra/SPARQ-PATCHES.md`, `vendor/spargebra/src/` — the vendored SPARQL parser
  (do not fork for a dialect).
- `crates/sparq-nlq/src/link.rs` — lexical entity/relation linking (the no-model form of
  lever 3); `crates/sparq-nlq/src/constrain.rs` — dictionary-grounded constraint + did-you-mean
  (the sound form of lever 2's diagnostic); `crates/sparq-nlq/Cargo.toml` (`live` feature gate).
- `crates/sparq-vectors/src/rewrite.rs` (`query_vec`, `vec:nearest`/`vec:search`),
  `src/labels.rs`/`src/verbalize.rs` (`embed_labels`/`verbalize`), `src/ann.rs`
  (`nearest_term_exact_checked`, sq-32i5 staleness guard).
- `crates/sparq-introspect/src/lib.rs` (`to_text_summary` schema card).
- `research/dogfooding-sparq-knowledge-graph.md` (§2 ontology, §5 eval protocol, §6 phases) —
  now realised as the merged `crates/sparq-kb` (PKG ontology + ingested graph, PR #1069).
- `research/agent-effectiveness-program.md` / `agent-efficiency-tooling.md` — cache hygiene
  as the dominant token lever; the measure-first / kill-criteria discipline.
- `skills/{genai-retrieval,sparql-query,vector-search}/SKILL.md`.

**External `[established]`:**
- Emonet et al., *LLM-based SPARQL Query Generation over Federated KGs* (arXiv:2410.06062) —
  schema validation + decomposition, out-of-schema-predicate barring.
- *Reducing Hallucinations … Post-Generation Memory Retrieval* (arXiv:2502.13369).
- Text2SPARQL'25 (ESWC 2025); mKGQAgent / ARUQULA (arXiv:2510.02200) — agentic ReAct +
  entity-linking, schema grounding as the named bottleneck.
- *Accurate SPARQL generation via in-context learning + schema-based construction*
  (Oxford *Bioinformatics* 2026, btag174).
- Ferré, *SQUALL* (controlled NL, Montague grammar); *Sparklis* (*Semantic Web* 8(3), 2017)
  — guidance-by-construction CNL query building.
- JSON-LD 1.1 `@vocab` (w3c json-ld-syntax); RDF 1.1/1.2 Turtle (`a`, prefixed names).
- Microsoft DevBlogs, *AI Coding Agents and DSLs* — under-represented-language hallucination,
  "supply knowledge + impose structure," 3–5 in-context examples.
- *Anka: A DSL for Reliable LLM Code Generation* (arXiv:2512.23214) — 99.9% parse success on
  a zero-exposure DSL from prompt alone; *DSL or Code?* (arXiv:2601.00469) — constrained DSL
  +4.6pp over Python; survey on low-resource/DSL code-gen (arXiv:2410.03981).
- Diekmann & Tratt, *Don't Panic! Better, Fewer, Syntax Errors for LR Parsers*
  (arXiv:1804.07133); compiler error-recovery literature — lenient recovery masks/cascades
  real errors.
- AGENTS.md consolidation study (arXiv:2601.20404) — cache-hygiene token/runtime win
  `[independent]`.

**Uncertainties:** the helps-vs-hurts evidence for a novel terse dialect (§1.4) is *mixed*;
the deciding factor (small+constrained+anchored vs grab-bag) is a *design choice*, not a
settled empirical fact — hence §5 measures it rather than asserting it. No A/B has been RUN;
§5's thresholds are pre-registered, not results.
