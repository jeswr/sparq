# FO-KM benchmark — Metric 1 (agent KM-task accuracy + cost over the PKG)

> 🤖 **SPARQ agent** [OPUS-4.8]. The runnable harness for **Metric 1** of the design
> record `research/foundational-ontology-km-benchmark.md` (epic **sq-mztg8**; PR #1106).
> NOT a perf claim — the actual A/B numbers live here (the `bench/` tree), never frozen
> into markdown.

## What this measures

An **apples-to-apples A/B**: the same NL-tool (`crates/sparq-kb` `pkg-query`, the
introspect→ground→ask helper / `nl_tool` envelope) answers **FO-exercising
knowledge-management questions** over the **same** Project-Knowledge-Graph (PKG), typed
under different **foundational-ontology (FO) overlays**. The question under test (epic
sq-mztg8): *does any FO beat the no-FO incumbent (and gUFO) on agent KM-task accuracy +
cost?* The pre-registered prior is **NEUTRAL** (design §7) — this harness is built to be
able to return that null honestly.

## The arms (overlays/)

Each arm = the shipped PKG (`pkg.ttl` + `pkg-instances.ttl`) **plus** one overlay TTL
loaded via `pkg-query --extra-graph`, optionally closed with `--close owl-rl` so the
overlay's `rdfs:subClassOf` axioms entail the FO-typed facts (rdfs9 type propagation +
rdfs11 transitive subclass).

| Arm | Overlay | PKG-class → FO top category | FO source |
|---|---|---|---|
| **no-FO** (incumbent) | `overlays/no-fo.ttl` | (none — the shipped reuse-first PKG) | — |
| **gUFO** (named baseline) | `overlays/gufo.ttl` | Task→`gufo:Event`; Finding→`gufo:AbstractIndividual`; Source/Technique→`gufo:Object` | nemo-ufes.github.io/gufo |
| **DOLCE-DUL** | `overlays/dolce-dul.ttl` | Task→`dul:Action`; Finding→`dul:Description`; Source→`dul:InformationObject`; Technique→`dul:Method` | ontologydesignpatterns.org DUL |
| **schema.org-as-top** | `overlays/schema-org.ttl` | Task→`schema:Action`; Finding→`schema:Claim`; Source→`schema:DigitalDocument`; Technique→`schema:HowTo` | schema.org |

Each overlay inlines only the **minimal** FO taxonomy fragment needed for closure (it
does not import the whole FO) and cites its source in the file header.

## The per-arm command

```bash
# FO arm (e.g. gUFO): load the overlay, close, ask the FO-typed query
cargo run -p sparq-kb --features close --bin pkg-query -- \
  --extra-graph bench/fo-km/overlays/gufo.ttl --close owl-rl \
  --sparql 'PREFIX gufo: <http://purl.org/nemo/gufo#> SELECT (COUNT(DISTINCT ?x) AS ?n) WHERE { ?x a gufo:Event }'

# no-FO arm: the same question over the incumbent (no FO category → 0 rows / can't answer)
cargo run -p sparq-kb --features close --bin pkg-query -- \
  --extra-graph bench/fo-km/overlays/no-fo.ttl --close owl-rl \
  --sparql 'PREFIX gufo: <http://purl.org/nemo/gufo#> SELECT (COUNT(DISTINCT ?x) AS ?n) WHERE { ?x a gufo:Event }'
```

Swap `gufo.ttl` for `dolce-dul.ttl` / `schema-org.ttl` (and the matching FO query from
`tasks.jsonl`'s `select` map) for the other arms. `--json` emits the verifiable NL-tool
envelope (executed SPARQL + resolved IRIs + grounding confidence).

## The tasks (tasks.jsonl)

16 **FO-exercising** KM tasks, stratified (design §5): **TH** type-hierarchy, **ER**
entailment-dependent, **CC** cross-category. Each line:
`{id, kind, question, gold_keys, gold_count, select, no_fo, discriminates}` —
`select` is the per-arm FO query (`gufo` / `dolce-dul` / `schema-org`; `null` where an
FO honestly cannot draw that distinction); `no_fo` is what the incumbent would attempt;
`discriminates` records why the no-FO arm genuinely cannot answer.

These tasks **discriminate**: an FO arm under closure returns the gold answer; the no-FO
arm returns 0 / can only hand-enumerate (the FO-win construction). Tasks answerable by
plain `pkg:` terms are deliberately excluded — they would not differentiate the arms.

## Authoring + validation

- `build_tasks.py` regenerates `tasks.jsonl`.
- `validate_tasks.py` proves every task discriminates (each FO arm answers with the
  expected count; the no-FO arm returns 0) — run from the repo root:
  `python3 bench/fo-km/validate_tasks.py` (needs the `close` feature).

## Honest scope

- This is **Metric 1** (runnable on a work box at char-/result-fidelity). Metric 2 (the
  KGE closure-prior MRR) needs a canonical/EC2 box and is a separate phase (design §5.1).
- The **full A/B run** (scoring accuracy + token cost across all arms, with the
  pre-registered kill-criteria) is run by the orchestrator next — this PR ships the
  **harness**, not the verdict. The closure-build CPU/wall cost is **non-canonical** and
  is never charged as a token cost (design §5.1).
