<!-- [OPUS-5] sq-i6du2.8 (epic sq-i6du2, #1613) — 🤖 SPARQ agent. -->
# ODRE decision-agreement lane (ODRL)

Runs the AC-query benchmark's constraint-rich ODRL corpora — **U3 financial services**
and **U4 research consortium** — through both sparq's ODRL evaluation path and the pinned
**ODRE** reference implementation ([arXiv:2409.17602](https://arxiv.org/abs/2409.17602),
distributed as `pyodre`), and diffs the decisions into a classified agreement ledger.

This is the vehicle for the `odrl-policy-bridge` paper's §5.3 comparative
decision-agreement study, which that paper specifies but has never run: it cites this
lane's output rather than running its own
([`research/ac-query-benchmark.md`](../../../research/ac-query-benchmark.md) §4.2,
disposition 2).

## Run it

```bash
bash bench/ac/odre/run.sh --setup      # ONE networked step: install the pinned pyodre
bash bench/ac/odre/run.sh --smoke      # the per-commit tier — offline from here on
bash bench/ac/odre/run.sh --sf 10      # a larger nightly/EC2 corpus, same protocol
bash bench/ac/odre/run.sh --self-test  # prove the gate can fail, no corpus needed
```

Artifacts land in `out/` (git-ignored): `cases.json`, `odre-decisions.json`, and
`agreement-report.json`. The report is a **run record**, not a committed number — keeping
it out of the tree is what stops a stale figure from being cited as current.

**Without `--setup` the lane still runs.** ODRE is a gather-time dependency
(`bench/CATALOG.md` convention): if `pyodre` is not importable, every case is reported
`SKIPPED`, no agreement figure is stated, and the run exits 0. A skip with a reason, never
a fabricated decision.

## Pipeline

| Stage | What it does |
|---|---|
| `exporter/` (Rust) | Generates the U3/U4 ODRL corpora and records **sparq's own decision** per case through the real `sparq_policy` parse + evaluate path — the evaluator the `sparq-solid` `odrl-bridge` materialises from. Its own `[workspace]`, so the root workspace never links it. |
| `odre_adapter.py` | Runs every case through `pyodre` under three input encodings (`standard` / `odre-native` / `projected`). stdlib-only apart from `pyodre` itself. |
| `agreement.py` | Classifies every divergence against `known-divergences.json`, validates the report against `agreement-report.schema.json`, and gates. |

Why three encodings, and what each one costs in fidelity, is
[`MAPPING.md`](./MAPPING.md) §2 — read that before reading any figure this lane emits.

## The gate

The run **fails (exit 3) on any unclassified divergence**. Every difference between the
two systems must match a sourced entry in `known-divergences.json`, classified as one of:

- **mapping-gap** — the two systems were not asked the same question;
- **semantics-gap** — a documented, defensible difference in reading ODRL;
- **implementation-bug** — an implementation not doing what its own docs say.

It also **fails (exit 2) before computing anything** when the inputs are not the run they
would claim to be: an installed `pyodre` that is not the pinned version (the ledger's
divergence excuses are transcribed from that release specifically), or an
`odre-decisions.json` that does not cover every corpus case under every encoding exactly
once — or that carries every encoding key but a result behind one that records no verdict
(a `missing`/`skipped-no-odre`/unknown status, or an `ok` with no decision, under an ODRE
that ran). Key coverage is not verdict coverage. Those are input errors, not a batch of
skips inside a `complete` report.

`run.sh` runs `agreement.py --self-test` on every invocation: synthetic scenarios drive
the real classifier, the real gate and the real input checks, and the unexplained
divergence, the truncated decision file and the unpinned-version run **must** all come back
red. A gate that cannot fail would make the lane's green meaningless.

## Honesty contract

Restating the bead's invariant, because it is the point of this lane:

- **Agreement is claimed only from run output.** An agreement rate exists only when ODRE
  actually ran over a non-empty comparable set; otherwise it is `null` and the report says
  why. No default, no carry-forward.
- **Every divergence is classified**, or the run fails.
- **No unqualified correctness claim for either system.** A PASS means "agreed on the
  comparable cases at the pinned instant under the declared harness interventions". It is
  not a conformance, soundness or security statement about sparq or about ODRE, and the
  report repeats this in its own `claims` block.
- **Every deviation from feeding both systems identical bytes is recorded** as a
  `harness_intervention` with the reason it was necessary.
- **The report grades its own evidence.** If the comparable set never handed ODRE a
  constraint to evaluate, or ODRE returned a single decision value throughout, the report
  adds a `WEAK EVIDENCE` claim itself rather than leaving a reader to infer a stronger
  result than the run supports.
- **No timing.** This lane's contract is a decision ledger; the design record scopes ODRE
  to agreement first, timing second.

## Files

| File | Role |
|---|---|
| `run.sh` | The entry point; stages, gate, exit codes. |
| `exporter/` | Rust corpus + sparq-decision exporter (standalone cargo project). |
| `odre_adapter.py` | ODRE driver: encodings, clock pinning, per-case results. |
| `agreement.py` | Classifier, report builder, schema validation, gate, self-test. |
| `agreement-report.schema.json` | What a COMPLETE report must contain. |
| `known-divergences.json` | The sourced divergence ledger the classifier matches against. |
| `odre-capabilities.json` | ODRE's DECLARED capability table (what is comparable at all). |
| `requirements.txt` | The pinned `pyodre` version. Note: PyPI `odre` is an unrelated project. |
| `MAPPING.md` | Semantics mapping notes: sparq `Request`/`Policy` ↔ ODRE inputs. |

Sibling lanes: [`bench/ac`](../README.md) (oracle + live drivers),
[`bench/ac/overhead`](../overhead/README.md) (ODRL-gated query overhead).
