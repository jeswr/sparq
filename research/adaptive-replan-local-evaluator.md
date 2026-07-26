# Adaptive re-planning in the LOCAL evaluator — prior-negative record + redesign path

[FABLE-5] Design record for **sq-gafdh** / issue #2932 ("Port the federated adaptive
re-planner to the LOCAL evaluator, benchmark-gated"), which restates maintainer
feature-request **gh-903** (green-lit 2026-06-19: *"Run benchmarks to make sure this does
improve performance in the main module. Put in a shared module if it does not cause
degradations in either engine."*).

**Status: no implementation performed.** This record exists because the task as literally
specified has **already been built, measured, and closed as a negative result**. Re-running it
would duplicate ~1k lines of existing (unmerged) work and would fail the same benchmark gate
that stopped it the first time. The durable value is in recording *why*, so the fleet stops
re-picking it, and in naming the redesign that can actually pass a benchmark gate.

## 1. Corrections to the issue's premise

The issue makes three factual claims. Verified against `origin/main` at `a081c129`:

| Claim | Verdict | Evidence |
| --- | --- | --- |
| "No adaptive-replan in the local evaluator (no reference in sparq-engine/sparq-core)" | **TRUE** | No `adaptive`/`replan`/`RuntimeStats` symbol in `crates/sparq-engine/src`; `crates/sparq-engine/Cargo.toml` has no `sparq-fedplan` dependency and no adaptive feature. |
| "The adaptive re-planner currently lives in the federated path only: `crates/sparq-fedplan/src/adaptive.rs` **and** `crates/sparq-fedclient/src/adaptive.rs`" — implying duplication to de-duplicate | **MISLEADING** | `sparq-fedclient` is a **consumer**, not a copy. Its `adaptive.rs` drives fedplan's types (`use sparq_fedplan::{…}`, `crates/sparq-fedclient/src/adaptive.rs:92`) behind `fedclient-adaptive = ["fedclient", "sparq-fedplan/adaptive-replan"]` (`crates/sparq-fedclient/Cargo.toml:66`). The policy + executor already live in exactly **one** place. There is no federated-side duplication to factor out. |
| Implicitly, that this port has not been attempted | **FALSE** | It was built in full and rejected on the benchmark gate. See §2. |

## 2. The prior attempt: sq-p6p6 (the load-bearing finding)

Commit `f85d0180` — *"feat(sparq-engine): port adaptive divergence-triggered re-planner to the
LOCAL evaluator (sq-p6p6)"*, 2026-06-20 — exists on `origin/sq-p6p6-adaptive-replan-local` and
is **not an ancestor of `origin/main`** (`git merge-base --is-ancestor` returns non-zero). It
implements precisely what sq-gafdh asks for, including the shared-module half:

```text
crates/sparq-engine/src/adaptive.rs                | 389 +++
crates/sparq-engine/src/exec.rs                    | 151 +-
crates/sparq-engine/tests/adaptive_replan_bench.rs | 187 +++
crates/sparq-replan-policy/{Cargo.toml,README.md,src/lib.rs} | 219 +++
crates/sparq-engine/{Cargo.toml,lib.rs,README.md}, skills/sparql-query/SKILL.md
12 files changed, 1035 insertions(+), 23 deletions(-)
```

It is not a sketch. It ships an opt-in `adaptive-replan-local` feature (OFF by default), a pure
dependency-free `sparq-replan-policy` crate holding the divergence + hysteresis rule so the two
re-planners cannot drift, a non-vacuous `replan_result_equals_static` oracle (a fixture that
fires a *real* plan switch and asserts multiset equality against the static plan), and an
apples-to-apples in-process micro-benchmark. Its commit message records gates green in both
feature states.

**Why it was held back** — quoted from the commit message, which is explicit that the numbers
are work-box-indicative and therefore **NON-CANONICAL**:

> the engine micro-benchmark shows NO perf win — a slight regression when the re-plan fires
> (the only flippable divergence reorders two multiplicative arms without reducing total
> materialised work) and […] no degradation when inert. The local planner's EXACT per-pattern
> estimates leave little for a divergence-triggered reorder to gain. The benchmark gate is
> therefore UNMET pending a canonical-host result or a redesign […] the feature must NOT be
> armed on a perf claim and is documented as correctness-complete-but-perf-neutral.

### 2.1 Root cause — why local is structurally different from federated

This is the part worth carrying forward. The federated re-planner wins because its inputs are
*genuinely uncertain*: source cardinalities come from served VoID/characteristic-set
descriptors and per-source latency is unknowable ahead of time, so an observed divergence
carries real new information. The local evaluator's inputs are not uncertain in the same way:

- Single-pattern cardinality is **index-exact**, not estimated — `PredStat { count, ndv_subj,
  ndv_obj }` (`crates/sparq-core/src/store.rs:68`) is built at load time and read directly by
  `pattern_var_ndv` (`crates/sparq-engine/src/exec.rs:7616`).
- The only genuinely-estimated quantity is the **multi-pattern join** estimate, where the
  independence assumption in `goo_pick` (`crates/sparq-engine/src/exec.rs:7596`,
  `cur_card * prepared[i].est * sel`) is wrong on correlated predicates.
- But a divergence-triggered **re-order** can only permute the remaining arms. Because
  `eval_bgp_binary` materialises every intermediate anyway, permuting multiplicative arms
  changes *when* rows are materialised, not *how many*. There is no work reduction to harvest,
  while the re-plan itself costs a planning pass.

The repo already reached this conclusion and recorded it as the decision of record —
`research/codebase-improvement-opportunities-2026-06-23.md:447-452`:

> **ALREADY BEADED** as `sq-6i40` (P3, OPEN), which is explicitly the redesign follow-up after
> `sq-p6p6` landed *neutral* (local estimates are index-exact, so a reorder only flips
> multiplicative arms — no work reduction). The honest path is *pruning/semi-join reduction*
> …, not arm-reordering. … the reorder framing is dropped as a known negative result.

`research/fable-work-plan.md:174` corroborates: `sq-p6p6` is **"closed, superseded"**, `sq-6i40`
is the open redesign, and `sq-0g6g` is the EC2-gated adaptive-reducer-dispatch bead.

## 3. Options

**A. Re-implement the port as specified.** Rejected. It duplicates `f85d0180`, contradicts the
repo's own recorded decision, and cannot pass its own benchmark gate — the gate is a hard
condition in the issue text ("must demonstrably IMPROVE the main module's perf"), and the
mechanism has no work-reducing effect to demonstrate. Building it again to re-derive the same
negative is pure cost.

**B. Revive `origin/sq-p6p6-adaptive-replan-local` and re-measure on a canonical host.** Cheap
(the branch is complete) and it would convert a work-box-indicative negative into a canonical
one. But the negative is *structural*, not measurement noise: a reorder that does not reduce
materialised work will not become a win on a quieter box. Worth doing only as a cheap
confirmation if the maintainer wants canonical closure before the issue is retired.

**C. Re-scope onto the sq-6i40 redesign — checkpoint-triggered strategy switch.** The
observation that motivated the port is sound and still unexploited: because every intermediate
materialises, the *true* running cardinality is known for free at each stage boundary. The
error in sq-p6p6 was spending that signal on re-ordering. Spend it instead on switching to a
**work-reducing strategy** when an intermediate blows up:

- trigger the existing bitmap semi-join reducer (`semijoin-bitmap`) or the acyclic-BGP
  full-semijoin prepass (`yannakakis`) — both already implemented and proven-correct, and both
  currently dormant behind static flags and a constant threshold rather than a runtime signal
  (`research/fable-work-plan.md:175`);
- this is *work reduction*, so unlike a reorder it has a mechanism by which a benchmark gate
  can actually be met.

This also dissolves a separate blocker: those reducers are gated on a default-flip that needs
EC2 sign-off, whereas a runtime adaptive gate that self-guards against the pure-overhead case
can unlock them without the flip.

**D. Attack the estimate rather than the plan.** The genuinely-wrong quantity is the correlated
multi-pattern estimate. Index-based join sampling (`research/codebase-improvement-opportunities-2026-06-23.md:130-139`)
improves it *before* execution, needing no mid-flight machinery. Complementary to C, not a
substitute.

## 4. Recommendation

1. **Do not implement sq-gafdh as written.** Retire it as a duplicate of the closed `sq-p6p6`,
   cross-linking `origin/sq-p6p6-adaptive-replan-local` and this record so the prior art is
   discoverable — it currently is not, which is very likely why the task was re-raised.
2. **Route the underlying intent to `sq-6i40`** (option C): the runtime signal is real and
   free; only its consumer was wrong.
3. **Answer gh-903 honestly.** The maintainer's condition was conditional — *"make sure this
   does improve performance"*. It was tested and it does not. That is a completed, negative
   answer to the request, not an unstarted task. The second clause (*"put in a shared module"*)
   was also already honoured by `sparq-replan-policy` in the prior branch.

## 5. Phased plan (future beads)

1. **Retire + cross-link.** Close sq-gafdh/#2932 as duplicate-of-sq-p6p6, linking this record
   and the unmerged branch. Add a pointer from `crates/sparq-fedplan/src/adaptive.rs` module
   docs noting the local port was attempted and why it is federated-only, so the next reader of
   the federated re-planner finds the negative in place.
2. *(Optional, cheap)* **Canonical confirmation.** Revive the branch, run
   `adaptive_replan_bench` on a canonical quiet host, and record the result in `bench/` — not in
   markdown. Converts a work-box-indicative negative into a canonical one. Skip if the
   structural argument in §2.1 is accepted.
3. **sq-6i40 — checkpoint-triggered strategy switch (the real work).** At each materialised
   stage boundary in `eval_bgp_binary`, compare true running cardinality against the estimate;
   on divergence, escalate to a work-reducing reducer rather than re-ordering arms. Opt-in
   feature, OFF by default. Reuse `sparq-replan-policy`'s divergence + hysteresis rule from the
   prior branch — that part of sq-p6p6 is sound and directly salvageable.
4. **sq-0g6g interlock.** Feed the runtime signal into the dormant `yannakakis` /
   `semijoin-bitmap` gate so it self-guards against the pure-overhead case, unlocking those
   reducers without the EC2 default-flip.
5. **Benchmark gate, defined up front.** Before building step 3, agree the acceptance
   condition: which suites (the correlated-join shapes in SP2Bench / WatDiv are the candidates),
   measured on a canonical host, no regression when the adaptive path is inert, results recorded
   in `bench/` per the no-numbers-in-markdown rule.

## 6. Uncertainties and open questions for the maintainer

- **Was sq-p6p6's non-merge a deliberate close, or did the branch simply stall?** The commit
  message and both research records read as a deliberate "gate unmet, superseded" decision, and
  `research/fable-work-plan.md:174` records it as closed — but `bd` was unavailable in this
  environment, so the bead's own status field is unverified. If it in fact stalled for an
  unrelated reason, option B becomes more attractive.
- **Does the maintainer want canonical closure (step 2) before retiring gh-903?** The structural
  argument says the answer will not change; this is a question of how much evidence the record
  should carry.
- **Is any real query shape known where a local re-order alone reduces materialised work?** I
  did not find one, and the prior benchmark's purpose-built divergent fixture failed to produce
  one. A counter-example from the maintainer would reopen option A.

## 7. Scope note

The routed area scope for this task was `sparq-fedplan`. Even had the port been the right call,
its target is `sparq-engine` — outside that scope. The one in-scope refactor the issue gestures
at ("put in a shared module rather than duplicating") has **no duplication to remove**: per §1,
`sparq-fedclient` consumes fedplan's re-planner rather than copying it. Rewiring `sparq-fedplan`
onto an extracted policy crate with no consumer on the other side would put the federated path
(and its adaptive test suite) at risk for zero gain — the prior branch deliberately declined to
do exactly that, keeping the fed path provably unaffected. No source changes were made.
