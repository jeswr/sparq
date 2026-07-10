# sparq-nlq: endpoint exec-accuracy harness — design record

**bead**: sq-2m6zm.7  
**date**: 2026-07-10  
**author**: SPARQ agent [SONNET-4.6]  
**status**: harness delivered; live measurement blocked on endpoint credential

---

## 1. Context

sq-2m6zm.6 landed `EndpointLlm` (PR #1229): a provider-agnostic OpenAI-compatible
chat-completions client (`nlq-endpoint` feature, off by default). It was stub-tested
offline; no accuracy claim was made.

sq-2m6zm.7 is the *measurement*: point `EndpointLlm` at a real cheap-model endpoint
(local Ollama/vLLM server, OpenRouter, direct OpenAI, etc.) and record exec-accuracy
on a canonical eval set.

---

## 2. Credential check

At delivery time (2026-07-10), no endpoint credential or local model server is
available on the work box:

- `OPENAI_API_KEY` — not set
- `SPARQ_NLQ_ENDPOINT_URL` / `SPARQ_NLQ_ENDPOINT_MODEL` / `SPARQ_NLQ_ENDPOINT_KEY` — not set
- `localhost:11434` (Ollama), `localhost:8080`, `localhost:1234` — no server responded

**Result: needs_credential=true.** The measurement has not been run.

What is needed to run it:

| Variable | Example | Required |
|---|---|---|
| `SPARQ_NLQ_ENDPOINT_URL` | `http://localhost:11434/v1` | yes |
| `SPARQ_NLQ_ENDPOINT_MODEL` | `llama3.1` or `gpt-4o-mini` | yes |
| `SPARQ_NLQ_ENDPOINT_KEY` | `sk-...` | no (local servers skip auth) |
| `SPARQ_OLYMPICS_NT` | `/path/to/olympics.nt` | no (defaults to bench fixture path) |

---

## 3. What the harness measures

### 3.1 Accuracy metric

**Exec-accuracy** following the QALD / Text2SPARQL convention
(`research/genai-nl-to-sparql.md` §1, §4.2): grade on the **answer set**, not
query-string equality. Both the candidate query (from the loop) and the gold query are
executed on the same sparq graph; precision / recall / F1 are computed over the resulting
bind-row sets. The gold is a query executed locally, not a checked-in answer blob — so the
gold cannot go stale (§4.3).

Additionally reported:
- **Exact match (EM)**: predicted answer set equals gold exactly.
- **Validity**: fraction of cases that produced a parseable + executable query.
- **Repairs**: repair rounds consumed (bounded by `NlqConfig::max_repair_rounds = 3`).

### 3.2 The four-cell comparison

The design doc (`research/genai-design.md` §4) requires two axes reported separately:

| | grounded | ungrounded |
|---|---|---|
| end-to-end | model writes query from schema-grounded prompt | same model, no schema deck |
| oracle-linking | gold query fed straight through validate→execute | same |

The **headline claim** (asserted in the test): grounded end-to-end macro-F1 >
ungrounded end-to-end macro-F1. "The grounding must pay for itself."

Oracle-linking isolates the engine-side loop (validate → execute → repair) from the
model's linking ability; it must be perfect (macro-F1 = 1.0) and serves as a sanity
check on the gold queries and the engine.

### 3.3 Per-question provenance

For every question the harness prints:
- The generated SPARQL (post-repair)
- Whether the query executed
- F1 and EM against the gold answer set
- Number of repair rounds

### 3.4 Eval set

8 questions over the QLever olympics dataset (1.78M triples), with gold SPARQL:

1. How many athletes are in the dataset?
2. Which team has the most athletes?
3. Are there any athletes taller than 200 centimetres? (ASK)
4. List the year and host city of every Olympic games.
5. How many medals of each type were awarded?
6. What is the average height of the athletes?
7. How many athletes are on each team?
8. List all sports with their labels.

Gold queries use only vocabulary visible in the sparq-introspect schema summary so the
grounded model has all it needs.

---

## 4. Harness deliverables

### 4.1 Test: `tests/endpoint_exec_accuracy.rs`

Feature-gated (`#[cfg(feature = "nlq-endpoint")]`) and `#[ignore]`'d. Never runs in CI.

Run command:
```sh
SPARQ_NLQ_ENDPOINT_URL=http://localhost:11434/v1 \
SPARQ_NLQ_ENDPOINT_MODEL=llama3.1 \
cargo test -p sparq-nlq --features nlq-endpoint --test endpoint_exec_accuracy \
  -- --ignored endpoint_exec_accuracy
```

Also includes one offline CI gate (`endpoint_config_missing_vars_gives_actionable_error`)
that always runs in the `nlq-endpoint` feature lane — exercises the env-var API without
a live call.

### 4.2 Example binary: `examples/endpoint_accuracy.rs`

Standalone CLI runner. Reads env vars, loads the dataset, runs the four-cell comparison,
prints the summary table + per-question provenance, optionally writes a JSON results
document (`--json <path>`), saves the recorded session pairs to
`tests/fixtures/endpoint_session_{i}.json`.

Run command:
```sh
SPARQ_NLQ_ENDPOINT_URL=http://localhost:11434/v1 \
SPARQ_NLQ_ENDPOINT_MODEL=llama3.1 \
cargo run -p sparq-nlq --example endpoint_accuracy --features nlq-endpoint --release
```

### 4.3 Session recording

Both the test and the example wrap each `EndpointLlm` in `RecordingLlm`, saving the
`(prompt, completion)` pairs to `tests/fixtures/endpoint_session_{i}.json` after a live
run. This enables:
- Replay via `ReplayLlm` without re-hitting the endpoint
- Regression testing: re-score the saved fixture offline to verify no accuracy change
  after prompt template or engine changes

---

## 5. Measurement results

**Not yet run.** No endpoint credential available on the work box at delivery time.

When a credential is available, run the example binary and append the four-cell table
here. Do NOT hardcode the numbers in source or tests — results belong in this research
note only (project hygiene: no hard-coded performance numbers in markdown, and these are
model-quality numbers that differ by endpoint/model choice).

Template for when run:

```text
endpoint: <URL>  model: <MODEL>
loaded <N> triples

=== endpoint exec-accuracy (model=<MODEL>) ===
exec-accuracy (<N> cases):
  EndToEnd   grounded=true  macroF1=X.XXX EM=X.XXX validity=X.XXX repairs=N
  EndToEnd   grounded=false macroF1=X.XXX EM=X.XXX validity=X.XXX repairs=N
  Oracle     grounded=true  macroF1=X.XXX EM=X.XXX validity=X.XXX repairs=N
  Oracle     grounded=false macroF1=X.XXX EM=X.XXX validity=X.XXX repairs=N
  grounding pays for itself: true/false
```

---

## 6. Broader measurement programme (from bead expansion 2026-07-10)

The bead was expanded to include the broader QALD-9-plus / QALD-10 programme, ablations,
and F1/cost/latency reporting. The harness delivered here is the foundation; the extended
programme is:

- QALD-9-plus slice (standard benchmark): add `endpoint_qald_cases()` to the test,
  download the dataset, add gold SPARQL for each question.
- Ablations: schema-card (drop the schema summary), few-shot count (0 vs 2 vs 5), repair
  (max_repair_rounds 0 vs 1 vs 3), dictionary constraint (`check_dictionary` on/off).
- Cost and latency: count model calls (= exchanges recorded by `RecordingLlm`) and
  measure wall-clock per question. Advisory; non-canonical on this box.
- Injection-hardening gate (sq-j1wv): run alongside this before claiming NLQ is
  production-ready.

These are follow-up work items captured separately. The current bead (sq-2m6zm.7) scope
is the harness + the eight-question olympics eval set.

---

## 7. Gates (for the code changes)

All gates verified GREEN at delivery:

- `cargo clippy --workspace -D warnings` (both feature states)
- `cargo test -p sparq-nlq` (default features)
- `cargo test -p sparq-nlq --features nlq-endpoint` (includes the offline API gate)
- `cargo doc --all-features -D warnings`
- `scripts/check-readme-template.py` (no README change)
