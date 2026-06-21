---
name: query-pkg
description: Answer a "what does the repo say about X, where was Y decided, what is the status or provenance of Z, which sources are still unexplored, what depends on bead W" question by running a SPARQL query over the ingested Project-Knowledge-Graph (PKG) instead of reading whole documents. Use when an agent working on sparq needs a sourced, answer-sized fact from AGENTS.md, the skills, or the bd backlog and would otherwise Read or Grep a large doc. The mechanism is introspect then ground then ask via the pkg-query helper in crates/sparq-kb. DEFAULT path is to DELEGATE the round-trip to a model:haiku sub-agent as a natural-language tool call (measured cheaper at equal quality, sq-zbyo7; see bench/pkg-dogfood/RESULTS.md); plain in-context pkg-query is the fallback.
---

# Query the PKG — introspect → ground → ask

[OPUS-4.8] Bead **sq-2m6zm.3** (epic sq-2m6zm). Design record:
`research/dogfooding-sparq-knowledge-graph.md` (§4.1 frontier queries, §6 worked
questions). 🤖 SPARQ agent — dogfooding sparq as a project knowledge graph.

The PKG is sparq's own knowledge stored as RDF triples: `pkg:Finding`s (sourced,
confidence-tagged discoveries from AGENTS.md), `pkg:Source`s (skills / docs with an
explored-status), `pkg:Technique`s (the surfaces), and `pkg:Task`s (the bd backlog
projected to RDF with `pkg:dependsOn` edges). Instead of `Read`-ing a whole document to
find one fact, run a SPARQL query and read the **answer-sized** result. The answer is
computed by sparq's own engine over the ingested graph — within the store the agent
cannot fabricate a fact the data does not contain (the engine bounds the result set), and
an empty result is the honest "I don't know / none outstanding" answer.

> **Honesty.** This skill is the *mechanism*. The cost saving versus reading the source
> document is now **MEASURED** (bead **sq-zbyo7**, the agent-flavour cheap-model NL-tool:
> ≈ 30× cheaper in $ than read-docs at equal quality on PKG-answerable tasks); the full
> table, method, and honest caveats live in `bench/pkg-dogfood/RESULTS.md` (the sanctioned
> measurement record) — this file only points at the headline. The win is scoped to
> **PKG-answerable** questions by construction. The PKG is a Phase-1 *head slice* (the
> AGENTS.md finding set + a mechanical bd→Task projection + the heaviest skills'
> front-matter), not the whole corpus — a miss means "not in the head slice yet", so fall
> back to Read/Grep.

## The helper

One command, in `crates/sparq-kb`, behind the default-OFF `query` feature (keeps the lean
default build a pure data crate):

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- <args>
```

It loads `pkg.ttl` (the ontology) + `pkg-instances.ttl` (the ingested data) into a sparq
store, runs a **named canned query** or a **raw SPARQL string**, prints the executed
SPARQL (always — for verification) and the result rows. Add `--features close` and
`--close owl-rl` to materialise the RDFS/OWL-RL closure first (entails the `pkg:dependsOn
owl:inverseOf pkg:blockedBy` pair and `pkg:couldBeMergedWith` symmetry — the design's
"always close first" step).

## Cheap-model NL-tool — the DEFAULT path (delegate to Haiku)

[OPUS-4.8] **For a PKG-answerable question, do NOT run the round-trip yourself in the
expensive orchestrator. DELEGATE it as a natural-language tool call to a `model:haiku`
sub-agent.** The cheap model does the whole `introspect → ground → SPARQL → run → NL`
loop; the orchestrator (Opus) only emits the question and reads the answer back. It
never sees the schema card, the SPARQL, or the raw rows.

**Why this is the default.** The verbose NL→SPARQL→run→NL middle is exactly the part a
~15×-cheaper model handles fine, and pushing it off the expensive orchestrator is a
large **model-price-weighted** saving. The agent-flavour cheap-model NL-tool was
measured (bead **sq-zbyo7**, N=30 frozen PKG-answerable tasks) at **≈ 30× cheaper in $
than Opus reading the docs, and ≈ 16× cheaper than Opus running `pkg-query` itself, at
equal answer quality.** Full table + method + caveats: `bench/pkg-dogfood/RESULTS.md`.
(The decision metric is $, not raw tokens — that is why the cheap-model arm wins even
though its raw-token count is similar.)

**The brief to give the Haiku sub-agent** (it must return a self-checkable answer):

> Use the `query-pkg` skill. Answer this question over the PKG by `introspect → ground →
> ask` via `pkg-query`. Question: «…». Return (1) a concise NL answer, (2) the **exact
> executed SPARQL**, and (3) the **provenance + confidence** of each fact (the
> `dcterms:source` section anchor and `pkg:confidence` each row carries). If the query
> returns 0 rows, say so — an empty result is the honest "not in the head slice / none
> outstanding" answer; do not invent rows.

**The verification echo (the guardrail).** Because the sub-agent returns the executed
SPARQL + resolved IRIs + the per-row provenance/confidence, the caller can **verify**
the answer was computed from a real query over the data, not guessed — the soundness
echo. If the returned SPARQL does not match the question, re-ask or fall back; never
accept a bare NL answer with no query behind it.

**Fallback — plain Opus `pkg-query`.** When sub-agent delegation is not available (e.g.
you are already inside a leaf sub-agent, or the dispatch path is unavailable), run
`pkg-query` yourself in-context as described below. That is arm **B** of the measurement
— still cheaper than reading the docs, just not as cheap as the Haiku NL-tool.

## (a) INTROSPECT — what can I ask about?

Before grounding a question, learn the dataset's effective schema so you know which
classes and predicates exist (cheaper than scanning sample data). The `schema-classes`
query lists every `pkg:` class actually instantiated, with a count:

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- --query schema-classes
```

```text
class  |  instances
Task  |  1255
Finding  |  11
Source  |  6
Document  |  6
Technique  |  5
```

`--query schema-properties` lists the predicates in use (label, type, title, identifier,
priority, status, issueType, isPartOf, **dependsOn**, subject, source, …). List every
canned query with:

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- --list
```

## (b) GROUND — translate the question to SPARQL over those terms

The canned queries are the §4.1-class, verified-expressible SPARQL frontier queries
(plain SELECT / `FILTER NOT EXISTS` / `GROUP BY`; the §4.2/§4.3 N3 rules are explicitly
Phase-2/3 and NOT used here). Pick the template that matches the question shape; some take
an argument via `--arg`.

| Question shape | canned `--query` | `--arg` |
|---|---|---|
| "What does the repo say about TOPIC?" | `findings-about` | a topic IRI |
| "Where was this decided / what is its provenance + confidence?" | `finding-provenance` | — |
| "Which sources are still UNEXPLORED (target follow-up)?" | `unexplored-sources` | — |
| "What is the explored-status of the source catalog?" | `source-status` | — |
| "What does bead W depend on, and is each dependency done?" | `task-depends-on` | a bd id |
| "What is blocked by bead W (downstream impact)?" | `task-blocks` | a bd id |
| "Which sources should I read next (priority order)?" | `high-followup-priority` | — |
| "How many beads are ready (§4.1 dependency frontier)?" | `ready-frontier` | — |

The topic IRIs in the current head slice are
`https://sparq.dev/ns/pkg/kb#topic-merge-discipline`,
`…#topic-subagent-rules`, and `…#topic-zk-discipline`.

To go beyond the templates, write a raw SPARQL SELECT/ASK against the `pkg:` terms and
pass it with `--sparql '<query>'`. Use `--sparql-only` to print the SPARQL without running
it (verify before executing).

## (c) ASK — run the helper and read the answer

Run the chosen query; read the small result table instead of the source doc. Each finding
row carries its `dcterms:source` section anchor and `pkg:confidence`, so the answer is
**sourced and confidence-tagged**, not bare prose.

---

## End-to-end examples (real questions, real returned answers)

### Example A — "What is the merge discipline?" (a findings-about GROUND query)

Instead of reading the whole `AGENTS.md` contribution-workflow + gate-matrix sections:

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- \
  --query findings-about --arg https://sparq.dev/ns/pkg/kb#topic-merge-discipline
```

Returned answer (sourced + confidence-tagged):

```text
label  |  section  |  conf
A PR merges only when ci-summary is green and every review thread is resolved
    |  AGENTS.md#contribution-workflow--prs-reviews-resolved-the-ci-summary-gate  |  0.98
A verified-clean non-perf PR auto-arms; never arm auto-merge on a stacked PR whose base is not main
    |  AGENTS.md#contribution-workflow--prs-reviews-resolved-the-ci-summary-gate  |  0.93
2 row(s).
```

(Swap the topic to `…#topic-zk-discipline` for "what's the merge discipline for a ZK PR?"
— it adds the forge_gates + gate-count snapshot row and the HARD privacy-claims gate row.)

### Example B — "Which sources are still unexplored?" (an honest empty answer)

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- --query unexplored-sources
```

Returned answer:

```text
source  |  title  |  status  |  prio
0 row(s).
```

Over the Phase-1 ingest every source is `pkg:Explored`, so the targeted-follow-up list is
**empty** — the honest "none outstanding" answer, computed by the engine rather than
guessed. (As `Unexplored`/`Exploring` sources are ingested in later phases they appear
here automatically.)

### Example C — "What is blocked by bead sq-8thu?" (a task-blocks GROUND query)

Instead of running `bd dep tree` or grepping `.beads/issues.jsonl`, ask the read-model:

```bash
cargo run -p sparq-kb --features query --bin pkg-query -- --query task-blocks --arg sq-8thu
```

Returned answer (downstream dependents + each one's status; truncated):

```text
downstreamId  |  downstreamStatus  |  downstreamTitle
sq-0po6  |  Closed  |  page: /surface/inference (tier-b live)
sq-11zy  |  Closed  |  page: /surface/streaming-rsp (tier-b live)
sq-13rg  |  Closed  |  ZK: bb.js in-browser UltraHonk proving wiring (WZK)
…
22 row(s).
```

The inverse direction (`?b pkg:blockedBy sq-8thu`) is never asserted in the data; under
OWL-RL closure it is entailed, so this is equivalent — verify with:

```bash
cargo run -p sparq-kb --features close --bin pkg-query -- --close owl-rl \
  --sparql 'PREFIX pkg: <https://sparq.dev/ns/pkg#> PREFIX dcterms: <http://purl.org/dc/terms/>
            SELECT (COUNT(*) AS ?n) WHERE { ?b dcterms:identifier "sq-8thu" . ?b pkg:blockedBy ?d }'
```

## When NOT to use this

- The fact is **not in the head slice** (PKG is a Phase-1 subset) → an empty/partial
  result means fall back to `Read`/`Grep`. An empty result is honest, not a failure.
- The document is **small** and you would read it once → loading it directly is fewer
  tokens and fewer round-trips (the design's §1.1 honest boundary).
- For the **live** ready-frontier (git/gh/nproc-aware) use `scripts/push-frontier.sh`;
  `ready-frontier` here is only the §4.1 dependency half over the projected backlog.
- bd remains the **source-of-record** for tasks; the PKG `pkg:Task`s are a read-model
  mirror, not a place to write task state.

## Where this lives

- Helper binary: `crates/sparq-kb/src/bin/pkg_query.rs` (`--bin pkg-query`).
- Canned queries (the library the binary + the test share):
  `crates/sparq-kb/src/query/canned.rs`.
- Loader + optional closure: `crates/sparq-kb/src/lib.rs` (`query` / `query::close` modules).
- Rot-guard test: `crates/sparq-kb/tests/query_canned.rs`
  (`cargo test -p sparq-kb --features query --test query_canned`; add `--features close`
  for the closure test).
- Ontology + data: `crates/sparq-kb/ontology/pkg/pkg.ttl`,
  `crates/sparq-kb/ingest/pkg-instances.ttl`.
