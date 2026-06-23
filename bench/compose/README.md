<!-- [OPUS-4.8] sq-mztg8.1 (FO-LLM-bridge Phase 3, epic sq-mztg8). 🤖 SPARQ agent. -->

# bench/compose — read-path URI-hiding A/B (FO-LLM-bridge Phase 3)

The closed **NL→NL** A/B for the read-path **URI-hiding "compose"** step: present an
answer-row binding to the agent as a **label/grounded view** (`UriHidden`) instead of a raw
IRI (`UriVisible`), behind the #1074 echo/confidence envelope, and measure whether hiding
helps (open question **K4**, design `research/fo-llm-bridge.md` §2.3/§3.3/§6).

## Run it

```sh
# model-free fidelity + cost A/B over the in-tree PKG
cargo run -p sparq-vectors --features compose --example compose_ab

# or over your own PKG sources
cargo run -p sparq-vectors --features compose --example compose_ab -- path/to/pkg.ttl ...
```

It prints a per-class + whole-PKG table (`class  n_iri  lexical  local_name  fell_open
coverage  fidelity_ok  collisions  char_ratio`) and exits **non-zero** if the hidden view
loses any answer identity (a label collision).

## What it measures — and what it does NOT

- **MEASURED (model-free):** round-trip **fidelity** (does the hidden label re-identify to
  exactly the IRI it hid?) and **presentation cost/coverage** over the real PKG.
- **UNMEASURED:** the headline K4 **accuracy** question (does hiding help a real small model
  answer?) — it needs a real-model NL→NL fan-out and there is no API key on the work box. The
  accuracy verdict is registered UNMEASURED with a pre-registered NEUTRAL null.

See [`RESULTS.md`](RESULTS.md) for the measured figures + the non-sycophantic verdict (the
short version: **hiding is sound here but ~2.4× LARGER to read** — not the token-saver the
naive assumption predicts).

## Library surface

The harness drives `sparq_vectors::compose` (the `compose` feature, off by default):
`compose()` (per-view rendering + echo), `ab_report()` (the fidelity/cost report), and
`AbReport::fidelity_preserved()` (the soundness gate).
