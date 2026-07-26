# Adaptive re-planning in the LOCAL evaluator — scoped prior-negative record + redesign path

[FABLE-5] Design record for **sq-gafdh** / issue #2932 ("Port the federated adaptive
re-planner to the LOCAL evaluator, benchmark-gated"), which restates maintainer
feature-request **gh-903** (green-lit 2026-06-19: *"Run benchmarks to make sure this does
improve performance in the main module. Put in a shared module if it does not cause
degradations in either engine."*).

**Status: no implementation performed.** This record exists because the task as literally
specified has **already been built and measured once, on an unmerged branch, with a negative
(perf-neutral) result** — see §2 for the precise scope of that negative, which is narrower than
"the idea does not work". Re-running it blind would duplicate ~1k lines of existing work. The
durable value is in recording *what was actually measured and what was not*, so the fleet
neither re-picks the task blind nor over-reads the prior negative, and in naming the redesign
with the clearer path to a benchmark gate.

**Scope of the negative, stated up front:** the prior branch established that *its* trigger,
*its* replanning policy, *its* plan space (arm permutation under `goo_pick`), *its* executor
(`eval_bgp_binary`) and *its* two purpose-built fixtures produced no win on a non-canonical work
box. It did **not** establish that no profitable local adaptive reorder exists. §2.2 records why
the fixtures under-tested the hypothesis.

## 1. Corrections to the issue's premise

The issue makes three factual claims. Verified against `origin/main` at `a081c129`:

| Claim | Verdict | Evidence |
| --- | --- | --- |
| "No adaptive-replan in the local evaluator (no reference in sparq-engine/sparq-core)" | **TRUE** | No `adaptive`/`replan`/`RuntimeStats` symbol in `crates/sparq-engine/src`; `crates/sparq-engine/Cargo.toml` has no `sparq-fedplan` dependency and no adaptive feature. |
| "The adaptive re-planner currently lives in the federated path only: `crates/sparq-fedplan/src/adaptive.rs` **and** `crates/sparq-fedclient/src/adaptive.rs`" — implying duplication to de-duplicate | **MISLEADING** | `sparq-fedclient` is a **consumer**, not a copy. Its `adaptive.rs` drives fedplan's types (`use sparq_fedplan::{…}`, `crates/sparq-fedclient/src/adaptive.rs:92`) behind `fedclient-adaptive = ["fedclient", "sparq-fedplan/adaptive-replan"]` (`crates/sparq-fedclient/Cargo.toml:66`). The policy + executor already live in exactly **one** place. There is no federated-side duplication to factor out. |
| Implicitly, that this port has not been attempted | **FALSE** | It was built in full on a branch that was never merged, and held back because its benchmark gate was recorded UNMET. See §2. |

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

### 2.1 Why local has *less* headroom than federated (a difference of degree, not of kind)

This is the part worth carrying forward. The federated re-planner wins because its inputs are
*genuinely uncertain*: source cardinalities come from served VoID/characteristic-set
descriptors and per-source latency is unknowable ahead of time, so an observed divergence
carries real new information. The local evaluator's inputs are less uncertain — which shrinks
the opportunity, but does not eliminate it:

- Single-pattern cardinality is **index-exact**, not estimated — `PredStat { count, ndv_subj,
  ndv_obj }` (`crates/sparq-core/src/store.rs:68`) is built at load time and read directly by
  `pattern_var_ndv` (`crates/sparq-engine/src/exec.rs:7616`).
- The only genuinely-estimated quantity is the **multi-pattern join** estimate, where the
  independence assumption in `goo_pick` (`crates/sparq-engine/src/exec.rs:7596`,
  `cur_card * prepared[i].est * sel`) is wrong on correlated predicates.
- A divergence-triggered **re-order** can only permute the remaining arms — a strictly smaller
  plan space than adding an operator. Note that this bound on the *plan space* is the real
  limitation; it is **not** true that permutation is work-neutral (see below).

**A framing to retire.** The two repo records quoted below — and the commit message's own
gloss — assert that because `eval_bgp_binary` materialises every intermediate,
permuting arms changes only *when* rows are materialised, not *how many*, so there is "no work
reduction to harvest". **That claim is false as a general statement and should not be carried
forward.** For a left-deep pipeline that materialises each intermediate, total materialised rows
are the *sum of the prefix cardinalities*, and that sum is order-dependent: with a running set of
size `C` and two remaining arms of factors `a` and `b`, joining `a` first materialises
`C·a + C·a·b` rows versus `C·b + C·a·b` the other way. Only the final term is order-invariant.
When one remaining arm *prunes* (factor < 1) and another *inflates*, the gap is not a lower-order
term at all — putting the pruning arm first is exactly the classic win, and it is precisely the
correlated-predicate case the independence assumption above gets wrong. Join order changing
intermediate cardinality is the premise of the whole join-ordering literature; the prior branch's
own fixture doc-comment says so explicitly ("The avoided intermediate is the win").

### 2.2 Why the prior branch's fixtures nevertheless came out flat

Read against the branch's actual test source, the measured result is consistent with the
commit message's own parenthetical — *"the only flippable divergence reorders two multiplicative
arms"* — and that is a property of the fixtures and trigger, not of reordering:

- The mis-estimated fixture's doc-comment describes a **pruning** shape
  (`?y :big ?b . ?y :sel ?z`, where `:sel` prunes and the avoided intermediate is the win), but
  the query the benchmark actually runs is
  `?w ex:anchor ?x . ?x ex:fan ?y . ?x ex:armA ?a . ?y ex:armB ?b`. In the generated data
  `armA` contributes `ay` objects per hot `x` and `armB` contributes `by_obj` objects per
  subject — **both arms inflate; neither prunes.** The shape the comment describes is never
  instantiated. So on the headline case, reordering could only move the lower-order prefix term,
  which the shared final intermediate dominates — a flat-to-slightly-negative result is the
  expected outcome there, and it says nothing about the pruning case.
- The `collapse_graph` fixture *does* contain a pruning arm (`armZ`). Whether the policy actually
  switched on it is reported only as a printed `switches` count in stdout — not asserted, not
  recorded — so the branch carries no durable evidence either way for the one shape that could
  have tested the hypothesis.

**Conclusion, honestly scoped:** the prior run is a valid negative for that trigger + policy +
plan space + executor + those two fixtures on a non-canonical box. **Whether a local adaptive
reorder can pay for itself on correlated join shapes with a genuinely pruning remaining arm
remains OPEN** — it was not measured. That question, not impossibility, is what should be
inherited.

The two repo records below reached the stronger conclusion and recorded it as the decision of
record. **They carry the same overstated premise corrected above** and should be re-read in that
light (out of scope for this record to edit; captured as follow-up) —
`research/codebase-improvement-opportunities-2026-06-23.md:447-452`:

> **ALREADY BEADED** as `sq-6i40` (P3, OPEN), which is explicitly the redesign follow-up after
> `sq-p6p6` landed *neutral* (local estimates are index-exact, so a reorder only flips
> multiplicative arms — no work reduction). The honest path is *pruning/semi-join reduction*
> …, not arm-reordering. … the reorder framing is dropped as a known negative result.

`research/fable-work-plan.md:174` corroborates: `sq-p6p6` is **"closed, superseded"**, `sq-6i40`
is the open redesign, and `sq-0g6g` is the EC2-gated adaptive-reducer-dispatch bead.

## 3. Options

**A. Re-implement the port from scratch as specified.** Rejected — but on *duplication* grounds
only, not impossibility. `f85d0180` already implements exactly this, so re-typing it is pure
cost; the useful move is to revive that branch (option B), not to re-derive it. Note the issue's
gate ("must demonstrably IMPROVE the main module's perf") is still unmet and would still have to
be met on the fixtures of §2.2, not the flat ones.

**B. Revive `origin/sq-p6p6-adaptive-replan-local`, fix the fixture gap, and re-measure.**
Now the **preferred cheap next step**, upgraded from "optional confirmation" in light of §2.2.
The branch is complete, so the marginal cost is a fixture fix plus a benchmark run. Two distinct
things to settle, in order:
1. **The untested case** — add a mis-estimated fixture whose remaining arms include a genuinely
   *pruning* one (the shape the existing fixture's doc-comment describes but its query never
   builds), and assert the switch actually fires. This is the measurement that was missing; it
   can be run anywhere, since a large effect would not be a work-box artefact.
2. **Canonical confirmation** — only if (1) shows a signal worth gating on, re-run on a canonical
   quiet host per the benchmark protocol, recording results in `bench/`.
   If (1) is flat too, *that* is a substantially stronger negative than the one on record today.

**C. Re-scope onto the sq-6i40 redesign — checkpoint-triggered strategy switch.** The
observation that motivated the port is sound and still unexploited: because every intermediate
materialises, the *true* running cardinality is known for free at each stage boundary. Rather
than spending that signal only on re-ordering, spend it on switching to a **work-reducing
strategy** when an intermediate blows up:

- trigger the existing bitmap semi-join reducer (`semijoin-bitmap`) or the acyclic-BGP
  full-semijoin prepass (`yannakakis`) — both already implemented and proven-correct, and both
  currently dormant behind static flags and a constant threshold rather than a runtime signal
  (`research/fable-work-plan.md:175`);
- this reaches outside the permutation plan space to operators that can cut work by more than a
  reordering can, so it is the higher-ceiling bet — C and B are complementary, not exclusive.

This also dissolves a separate blocker: those reducers are gated on a default-flip that needs
EC2 sign-off, whereas a runtime adaptive gate that self-guards against the pure-overhead case
can unlock them without the flip.

**D. Attack the estimate rather than the plan.** The genuinely-wrong quantity is the correlated
multi-pattern estimate. Index-based join sampling (`research/codebase-improvement-opportunities-2026-06-23.md:130-139`)
improves it *before* execution, needing no mid-flight machinery. Complementary to C, not a
substitute.

## 4. Recommendation

These are recommendations **for maintainer decision**, not dispositions this record can make on
its own — see the verification limits in §6.

1. **Do not re-implement sq-gafdh from scratch.** The work exists on
   `origin/sq-p6p6-adaptive-replan-local`. Cross-link that branch and this record from the issue
   so the prior art is discoverable — it currently is not, which is very likely why the task was
   re-raised. **Do not close the issue as "duplicate, settled"**: per §2.2 the decisive
   measurement was never taken, so the underlying question is open even though the code exists.
2. **Prefer option B as the next action** — revive the branch, add the missing pruning-arm
   fixture, and re-measure. Cheap, and it either finds the win or produces the strong negative
   that would justify retiring the idea.
3. **Route the longer-term intent to `sq-6i40`** (option C): the runtime signal is real and free,
   and strategy-switching has the higher ceiling regardless of how B resolves.
4. **Answer gh-903 honestly, with its scope stated.** The maintainer's condition was conditional
   — *"make sure this does improve performance"*. It was tested once and did not improve
   performance *on the fixtures described in §2.2*, one of which does not exercise the shape its
   own comment claims. The honest answer is "attempted, measured perf-neutral on an incomplete
   fixture set, unmerged" — not "completed, negative". The second clause (*"put in a shared
   module"*) was already honoured by `sparq-replan-policy` in the prior branch.

## 5. Phased plan (future beads)

1. **Cross-link + verify status.** Link this record and the unmerged branch from sq-gafdh/#2932,
   and check the bead's actual status with `bd` (unavailable in this environment — see §6) before
   any close. Add a pointer from `crates/sparq-fedplan/src/adaptive.rs` module docs noting the
   local port was attempted and where the record lives, so the next reader of the federated
   re-planner finds the prior art.
2. **Preserve the evidence on `main` (do this before the branch can be lost).** The entire
   implementation, oracle and benchmark exist *only* on an unmerged branch; if it is pruned, this
   record's citations become unverifiable. Cherry-pick `adaptive_replan_bench.rs` (or its
   methodology and fixture generators) onto `main` behind the opt-in feature, or archive the
   branch under a durable tag. Methodology and fixture shapes only — no numbers in markdown.
3. **Close the fixture gap and re-measure** (option B step 1, then step 2): add the pruning-arm
   mis-estimated fixture, assert the switch fires, run `adaptive_replan_bench`; escalate to a
   canonical quiet host only if a signal appears, recording results in `bench/` — not in markdown.
4. **sq-6i40 — checkpoint-triggered strategy switch (the larger prize).** At each materialised
   stage boundary in `eval_bgp_binary`, compare true running cardinality against the estimate;
   on divergence, escalate to a work-reducing reducer in addition to re-ordering arms. Opt-in
   feature, OFF by default. Reuse `sparq-replan-policy`'s divergence + hysteresis rule from the
   prior branch — that part of sq-p6p6 is sound and directly salvageable.
5. **sq-0g6g interlock.** Feed the runtime signal into the dormant `yannakakis` /
   `semijoin-bitmap` gate so it self-guards against the pure-overhead case, unlocking those
   reducers without the EC2 default-flip.
6. **Benchmark gate, defined up front.** Before building step 4, agree the acceptance
   condition: which suites (the correlated-join shapes in SP2Bench / WatDiv are the candidates),
   measured on a canonical host, no regression when the adaptive path is inert, results recorded
   in `bench/` per the no-numbers-in-markdown rule.

## 6. Uncertainties and open questions for the maintainer

**What is verified vs. asserted.** Verified directly: branch `sq-p6p6-adaptive-replan-local`
exists at commit `f85d0180` and is not an ancestor of `main`; its diffstat, commit message, and
test source are as quoted; `research/fable-work-plan.md:174` does record `sq-p6p6` as "closed,
superseded". **Not verified:** the bead's own status field — `bd` is not on `PATH` in this
environment, so every statement here about sq-p6p6 being *deliberately* closed rests on a
secondary research record, not the task database.

- **Was sq-p6p6's non-merge a deliberate close, or did the branch simply stall?** Unresolved, and
  this record deliberately does not assume the answer. The commit message reads as a considered
  "gate unmet" hold, but a hold is not a close, and no merged artefact records the decision. **A
  maintainer should confirm via `bd` before anything is closed on the strength of this record.**
- **Should the evidence be brought onto `main` first?** Recommended (§5 step 2). Every load-
  bearing citation here points at an unmerged branch; if it is pruned, this record degrades into
  unverifiable commit-message hearsay.
- **Is any real query shape known where a local re-order alone reduces materialised work?**
  Open, and per §2.1 the prefix-sum argument says such shapes *should* exist whenever a remaining
  arm prunes. The prior benchmark did not rule them out — its mis-estimated fixture contains no
  pruning arm at all (§2.2). This is the single highest-value question to settle, and option B
  step 1 is the cheap experiment that settles it.

## 7. Scope note

The routed area scope for this task was `sparq-fedplan`. Even had the port been the right call,
its target is `sparq-engine` — outside that scope. The one in-scope refactor the issue gestures
at ("put in a shared module rather than duplicating") has **no duplication to remove**: per §1,
`sparq-fedclient` consumes fedplan's re-planner rather than copying it. Rewiring `sparq-fedplan`
onto an extracted policy crate with no consumer on the other side would put the federated path
(and its adaptive test suite) at risk for zero gain — the prior branch deliberately declined to
do exactly that, keeping the fed path provably unaffected. No source changes were made.
