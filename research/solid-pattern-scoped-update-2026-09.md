# Pattern-scoped UPDATE enforcement — row-level write authority over a Solid pod

Status: **design record only** (spike `sq-fznmq`, epic `sq-lrtc3`). **No implementation, no
prototype, no feature flag.** This record specifies the semantics, names the leakage channels
a naive design would open, and decomposes the work into follow-up beads. Nothing described in
§4 onward exists in the tree today.

Companion records:
[`odrl-pattern-scoped-targets-2026-07.md`](odrl-pattern-scoped-targets-2026-07.md) (the READ
path this mirrors — its §2.4 defers exactly this problem and its §7 item 3 is this bead),
[`solid-access-control-design.md`](solid-access-control-design.md) (the WAC/ACP substrate, the
D1–D4 binding constraints, and the `update_as` write-gating architecture),
[`trust-graph-authorisation-2026-07.md`](trust-graph-authorisation-2026-07.md) (the
fail-closed composition posture).

<!-- [OPUS-5] 🤖 SPARQ agent. Design-first spike per the bead: options + fail-closed
semantics under INSERT/DELETE/WHERE over masked regions, produced BEFORE any implementation.
No performance numbers appear in this record. -->

> **Honest scope.** This is a *clear-path* authorisation design. No cryptographic guarantee is
> claimed anywhere in it: a `Session` is a caller-asserted claim, and every property below is a
> property of the enforcement code, not of a proof system. The design has not been implemented,
> so nothing here is verified — the acceptance obligations in §6 are what would make it so.

## 0. Premise check — what is actually true today

The bead brief says this work is "blocked conceptually on the read-path design being merged
(`sq-lrtc3.3`)". That blocker is **cleared**, but the surrounding status needs one correction
before the design can be read honestly:

| Claim | Reality on `origin/main` |
|---|---|
| The read-path design is merged | **True.** `research/odrl-pattern-scoped-targets-2026-07.md` is in the tree. |
| The read-path masking is implemented | **True, feature-gated.** `crates/sparq-solid/src/pattern_scope.rs` behind the OFF-by-default `pattern-scope` feature: `ScopePattern`, `GraphScope`, `masked_graph`, `masked_dataset`, `PodStore::scoped_dataset`, `ScopedDataset::{view,query,query_json,ask}`. Differential + fuzz batteries in `tests/pattern_scope.rs` and `tests/pattern_scope_fuzz.rs`. |
| A pattern-scoped grant can be expressed in ODRL end-to-end | **False.** The `sparq:PatternAsset` → `auth:PatternGrant` bridge (record §5, bead `sq-qnlj8`) is **not wired** — no occurrence of `PatternAsset`/`PatternGrant`/`allowPattern` exists outside the design record and `skills/usage-control-policy/SKILL.md`, which itself labels the vocabulary "Designed … not yet wired". Scopes today can only be supplied programmatically as an `FxHashMap<Term, GraphScope>`. |
| The replica cache exists | **False.** Bead `sq-nc3c6` (scoped-replica cache + write-path invalidation) is not implemented; `scoped_dataset` rebuilds every time. |

Consequence for sequencing: this design's *enforcement* work is independent of `sq-qnlj8`, but
its *vocabulary* work (§5) sits strictly downstream of it. §8 orders the beads accordingly.

## 1. The write-path leakage bar

The read record's bar was **oracle equivalence**: for every query `Q`,
`eval(masked(D), Q) = eval(D ∖ masked, Q)`. The write path needs a strictly stronger bar,
because an update is both an observation channel *and* a mutation, and because a write can
damage data the actor cannot see.

Five distinct channels, three confidentiality and two integrity:

| # | Channel | Attack shape |
|---|---|---|
| W1 | **Blind write into a masked region** (integrity) | The actor inserts triples matching a deny pattern — data it is forbidden to read — silently corrupting or contradicting what the owner sees. |
| W2 | **Masked-region overwrite / destruction** (integrity) | A `DELETE { GRAPH <g> { ?s ?p ?o } } WHERE { … }`, a `CLEAR GRAPH <g>`, or a `DROP` removes triples the actor cannot see. Worse than W1: the loss is invisible to the actor *and* to any audit keyed on what the actor could read. |
| W3 | **Delta dependence on masked data** (confidentiality) | The update's `WHERE` reads a masked triple and the resulting delta lands on a *visible* triple. `DELETE { GRAPH <g> { ?s ex:tag ?t } } WHERE { GRAPH <g> { ?s ex:diagnosis "X" . ?s ex:tag ?t } }` — the actor reads back the visible `ex:tag` triples afterwards and learns exactly which subjects carry the masked diagnosis. This is the sharpest channel: it turns the write path into an unrestricted read primitive. |
| W4 | **Verdict dependence on masked data** (confidentiality) | The allow/deny *bit* itself is a function of masked triples, giving one bit per request. See §2.2 for a concrete instance the naive composition would introduce. |
| W5 | **Metadata / error disclosure** (confidentiality) | A deny message names a graph, a triple, or a count the actor could not otherwise observe. |

W3 and W4 are the reason this is not a small extension of `update::check`. They are also the
reason the only defensible definition is not "oracle equivalence" but **non-interference**:

> **The pattern-scoped write law.** Let `S` be the actor's scope and `M = masked(D, S)` the
> sub-dataset it may observe. For any update text `U`, both the **verdict** (permit/deny, and
> the deny message) and the **delta** `(Δ⁻, Δ⁺)` must be functions of `(M, S, U)` alone —
> never of `D ∖ M`. Equivalently: for any two datasets `D₁`, `D₂` with
> `masked(D₁, S) = masked(D₂, S)`, running `U` must yield the same verdict, the same delta, and
> post-states whose masked views are equal.

This law subsumes W3, W4 and W5 (a message derived only from `(M, S, U)` cannot disclose masked
data) and it is **mechanically testable** by a two-dataset differential — see §6. W1 and W2 are
integrity obligations and need a separate predicate (§4.2); non-interference alone would happily
permit an actor to delete data it cannot see.

Prior art the law is calibrated against — both are relational row-level security, so the analogy
is by row ≈ triple, not exact:

* **PostgreSQL row-level security** splits the predicate in two: `USING` "determines which
  records the `UPDATE` command will see to operate against", `WITH CHECK` "defines which modified
  rows are allowed to be stored back into the relation"; a row failing `USING` is *silently
  filtered* while a row failing `WITH CHECK` *raises an error*. The docs also warn that policies
  "are not applied when the system is performing internal referential integrity checks or
  validating constraints", so "there are indirect ways to determine that a given value exists" —
  i.e. Postgres explicitly does **not** claim W4-freedom.
* **SQL Server row-level security** splits the same way — *filter* predicates "silently filter
  the rows available to read operations (`SELECT`, `UPDATE`, and `DELETE`)" while *block*
  predicates "explicitly block write operations (`AFTER INSERT`, `AFTER UPDATE`, `BEFORE UPDATE`,
  `BEFORE DELETE`) that violate the predicate" — and documents a "carefully crafted queries"
  side channel where a divide-by-zero error exfiltrates a filtered value.

Both estates therefore land on the **USING/WITH CHECK shape** (a read predicate governing what
an operation may touch, a write predicate governing what may be stored), and both concede
residual side channels. sparq is in a materially better position to close W3/W4 than either,
because the read path already chose *materialization* over *filtering* — a physically-reduced
dataset cannot leak through a channel nobody enumerated. §4 exploits exactly that.

## 2. What exists today, and where a naive composition breaks

### 2.1 The current write path (graph-granular, no scopes)

`crates/sparq-solid/src/update.rs` (`check`, called from `PodStore::update_inner`,
`lib.rs:1336`) authorizes an update *before* any mutation:

* `analyze` walks the parsed `Update` and collects per-graph `Need`s — `Write` for
  delete/clear/drop/create, `WriteOrAppend` for pure inserts (`update.rs:254`).
* Default-graph targets are denied outright (`update.rs:537`).
* A `GRAPH ?var` template slot is resolved **precisely** (`resolve_var_graphs`,
  `update.rs:407`): the operation's `WHERE` is evaluated to enumerate the concrete graphs the
  apply will write, with a `USING`/`WITH` re-scope re-expressed as an explicit `FROM`/`FROM NAMED`
  clause (`rescope_dataset`, `update.rs:328`) so the binding SELECT sees the *same active dataset*
  the engine's `build_using` assembles.
* Anything unresolvable escalates to the conservative all-graphs wildcard
  (`raise_wildcard`, `update.rs:221`); a budget exhaustion is a hard deny (`Bail::Budget`).
* The apply is `sparq_engine::update_in_place_with_budget`, which returns `Result<(), String>` —
  **no counts, no effect log** (`crates/sparq-engine/src/update.rs:654`). The capturing variant
  `update_in_place_capturing` returns `Vec<UpdateEffect>` where `Delta { slot, inserts, deletes }`
  carries the actual triples (`update.rs:599`). Insert is idempotent and delete-of-absent is a
  silent no-op (`crates/sparq-core/src/store.rs:899`), so today's `update_as` surface already
  has no cardinality side channel.

The load-bearing soundness invariant of that module, stated in its own docs: the checked write
set must equal the engine's actual write set — *"no MORE (an over-approximation only costs a
false denial) and crucially no LESS"* (`update.rs:631`), guarded by the differential tests in
`update.rs::differential_writeset_tests`. The mechanism that makes it hold is that **the check
dataset and the apply dataset are the same dataset**.

### 2.2 Why layering a scope on top is unsound (a concrete W4)

Suppose we keep `check` exactly as it is and merely add a post-hoc pattern test. `check`
evaluates the binding SELECT over the **full store**, justified today by a note that says this
leaks nothing because *"we never return rows to the actor — only an allow/deny verdict"*
(`update.rs:348`). That justification is correct for graph-granular authorization, where the
verdict depends only on graph names the actor already named. It **fails** the moment a scope
exists, because the verdict then depends on masked triples:

```sparql
DELETE { GRAPH ?g { ?s <https://ex.dev/ns#tag> ?t } }
WHERE  { GRAPH <https://pod.ex/g1> { <https://pod.ex/p1#it> <https://ex.dev/ns#diagnosis> "X" }
         GRAPH ?g { ?s <https://ex.dev/ns#tag> ?t } }
```

Give the actor graph-level write on `g1` only, read on `g1` and `g2`, and a scope masking
`ex:diagnosis` inside `g1`. Then:

* if the masked triple **exists**, the `WHERE` yields solutions, `?g` binds to `g2` among
  others, and `check` denies with `"update denied: session lacks write permission on
  <https://pod.ex/g2>"`;
* if it **does not exist**, the `WHERE` yields nothing, `resolve_var_graphs` returns the empty
  precise set, and the update is permitted as a no-op.

The actor distinguishes `Err` from `Ok` and learns one bit about a triple it is forbidden to
read; iterate over candidate objects and the masked region is fully recovered. Note this is
**not a bug in today's code** — there is no scoped write path today, so no such session can
exist. It is a hazard that the obvious composition would introduce, and it is why §4 moves *both*
the check dataset and the apply dataset onto the masked replica rather than bolting a filter on.

The same shape drives W3: because the apply's `WHERE` is evaluated over `graph` directly
(`crates/sparq-engine/src/update.rs:831`) with no view hook — the update entry points accept no
`DatasetView` at all — every scoped update would instantiate its templates from unmasked data.

## 3. Options

### A. Deny every UPDATE on a scoped graph — the status quo, formalized

Keep `update_as` graph-granular; deny whenever the request would take the scoped path — i.e. deny
on gate 0's `SCOPED` verdict (§4.1), which is syntactic, rather than on a set of targeted graphs
that a `GRAPH ?var` request cannot name without evaluating its `WHERE`.

* **Sound and free.** Trivially satisfies the law (the verdict depends only on `(S, U)`).
* **Useless for the invariant.** The maintainer's motivating cases — *"the researcher may
  correct the trial results but not the participant identifiers"* — are precisely row-level
  write authority. This option says the answer is "no".
* **Kept as the v1 fallback rule**, not as the design: §4 makes it the behaviour whenever the
  chosen mechanism cannot prove safety (unresolvable updates, wildcard operations, control
  documents).

### B. Static template checking in `update::check` — REJECTED

Extend `analyze` to test each `INSERT`/`DELETE` template quad against the scope, alongside the
existing per-graph `Need` check.

* Works for `INSERT DATA` / `DELETE DATA`, whose quads are ground.
* **Fails for `DELETE/INSERT … WHERE`**, which is the shape that matters: template quads carry
  variables, so scope membership is unknown until the `WHERE` is evaluated — and evaluating it
  over the full store is exactly W3/W4. Adding a scope test *after* a full-store evaluation
  converts a leak into a leak with extra steps.
* Also re-taxes every future SPARQL Update construct with a soundness obligation, the same
  cost model the read record rejected in its §1-A.

### C. Rewrite the update (inject scope guards into the templates and the WHERE) — REJECTED

Rewrite `DELETE { … } INSERT { … } WHERE { … }` so the `WHERE` carries a `FILTER NOT EXISTS`
exclusion per deny pattern and the templates carry the same.

* Inherits every objection the read record's §1-B raised — a missed construct is a silent leak,
  guards do not compose with negation, and the optimizer rewrites underneath.
* Adds a write-specific failure the read side does not have: guards constrain *which solutions
  survive*, not *which quads are written*, so a template quad whose bindings all come from
  guarded patterns can still instantiate to a triple outside the scope (any constant in the
  template is unguarded by construction). There is no formulation that makes the guard cover a
  ground term.
* Has no safety net: on the read path a missed guard yields a wrong answer that a differential
  oracle catches; here it yields a *write*, which is not undone by a failing test.

### D. Shadow-apply on the masked replica, authorize the delta, replay — **CHOSEN**

Reuse the read path's chosen mechanism verbatim. Evaluate the whole update against the actor's
**materialized masked replica**, capture the resulting delta, authorize the delta triple-by-triple,
and only then replay it onto the real store.

```text
M   := masked replica of the session's accessible dataset under scope S   (pattern_scope.rs)
Δ   := (Δ⁻, Δ⁺) = capture( apply(clone(M), U) )                            (update_in_place_capturing)
gate: every t ∈ Δ⁻ must be present in M and write-authorized under S
      every t ∈ Δ⁺ must be write-authorized under S
replay: apply Δ to D                                                        (only if the gate passes)
```

* **Non-interference is an identity, not an audit.** `Δ` is computed from `M`; `M` is a function
  of `(D, S)` that by construction contains no masked triple. Therefore `Δ`, the gate verdict and
  every deny message derived from them depend on `(M, S, U)` only. W3 and W4 close *by
  construction*, exactly as OPTIONAL/EXISTS/MINUS/COUNT equivalence closed by construction on the
  read side. No engine change, no new fast-path obligation, no per-construct audit.
* **W2 closes for free.** `Δ⁻ ⊆ M` because the engine can only delete what the dataset it is
  applied to contains. A masked triple is not in `M`, so it cannot appear in `Δ⁻`, so it cannot
  be removed by the replay. "You can only delete what you can see" is not a rule that has to be
  enforced — it is a consequence of where the apply runs.
* **W1 needs the explicit gate** (`Δ⁺` against a write predicate), which is the `WITH CHECK`
  half of the RLS shape. Nothing about materialization prevents an insert into the hole.
* **Preserves the module's load-bearing invariant.** `update.rs`'s soundness rests on
  *check dataset ≡ apply dataset*; this design keeps that identity and simply moves both to `M`.
  It also makes `resolve_var_graphs` redundant on the scoped path — the delta enumerates the
  written graphs exactly, so there is nothing left to approximate.
* **Better atomicity as a side effect.** `update_as` today inherits the engine's non-atomicity
  across `;`-separated operations (`lib.rs:1296`). Under shadow-apply the whole request's delta
  is computed and authorized before anything touches `D`, so a scoped update is all-or-nothing.
* **Honest costs.** (i) A replica materialization per scoped update — the read path can amortize
  a `ScopedDataset` across many queries, a write cannot, because it invalidates its own replica.
  Mitigation in §4.5 (materialize only the graphs the update can reach). (ii) The `WHERE` is
  evaluated twice on the *unscoped* path if the existing graph-level `check` is kept in front —
  §4.1 shows why it is kept and why the scoped path skips the redundant resolution. (iii) The
  replay is a set-semantics delta application, so an insert already present in `D` but masked out
  of `M` replays as a no-op (`store.rs:906`) — correct, and observationally identical to the
  actor.

### E. In-line per-triple write filtering in the engine — deferred to v2, if ever

The write mirror of the read record's rejected §1-A: teach the engine's apply path to consult a
per-graph pattern predicate. Same objection, amplified — the read side had ~15 scan entry points
to defend and a differential oracle to catch misses; the write side would have to defend the
delta construction of every update form with no equivalent safety net. Only revisit if measured
replica cost ever dominates, and only with the fuzzed differential harness of §6 in place first.

## 4. The chosen design in detail

### 4.1 Layering — the routing decision must come before any evaluation

Composition here has an ordering trap that must be resolved before the gates can be listed at
all. The natural applicability test — *"does any targeted graph carry a scope entry for this
session?"* — is **not answerable for a `GRAPH ?var` target without evaluating the `WHERE`**, and
evaluating the `WHERE` over `D` is precisely the §2.2 leak. A routing test that must perform the
dangerous operation in order to decide whether the operation is dangerous is circular, and the
circularity is load-bearing: it sits on the W4 boundary, not below it. So routing is decided
**first**, and from syntax alone.

**Gate 0 — the static routing classifier.** A pure function of the session's scope map `S` and
the update text `U`. It reads neither `D` nor `M`, and it runs before `update::check`:

```text
scoped_path(S, U) :=
  if S carries no scope entry for this session                            -> UNSCOPED
  if U contains a wildcard operation (CLEAR/DROP ALL|NAMED|DEFAULT)       -> SCOPED
  if any GRAPH slot in U — template or WHERE — is a variable              -> SCOPED
  if U contains a bare (non-GRAPH) pattern                                -> SCOPED
  if any ground GRAPH IRI in U carries a scope entry in S                 -> SCOPED
  otherwise                                                               -> UNSCOPED
```

Every case that cannot be settled syntactically resolves to `SCOPED`, so the classifier is
fail-closed by construction and monotone in `S` (adding a scope entry can only move a request
*onto* the scoped path, never off it). The degenerate rule **"any scope entry ⇒ `SCOPED`"** is
always admissible and strictly safer; the clauses above are the cost refinement, and an
implementation may start at the degenerate rule. The bare-pattern clause is deliberately blunt:
a bare pattern under the legacy union-default semantics can read any accessible graph, and while
a `USING`/`FROM`-pinned active dataset of unscoped ground graphs could be admitted later, v1
does not attempt it.

Why the `UNSCOPED` branch is sound even when the session *does* carry scopes: its conditions
force every graph the request can read or write to be a ground IRI with **no** scope entry, and
an accessible graph without a scope entry contributes to the replica *whole* — the read path
already builds `M` that way, filling in a permissive entry for every accessible unscoped graph
(`scoped_dataset`, `pattern_scope.rs:205`). So `M` and `D` agree on exactly the graphs such a
request can reach, and the verdict and delta computed over `D` are the ones that would have been
computed over `M`: on that branch non-interference is an identity, not a promise. (`M` also drops
graph-level *inaccessible* graphs, which is the graph-level gate's business and is unchanged by
this design — the claim here is only that no *masked* triple can influence an `UNSCOPED` request.)
That branch also never calls `resolve_var_graphs`, since it admits no variable graph slot:
**every request that needs precise var resolution and carries any scope is on the scoped path**,
which is what makes the §2.2 leak unreachable rather than merely avoided.

The routing bit is itself part of the verdict, so the law of §1 applies to it: a classifier that
is a function of `(S, U)` is a fortiori a function of `(M, S, U)`.

**Gate ordering, reconciled.** With gate 0 fixed, the remaining gates split by path.

*Unscoped path* — `update::check` runs exactly as today, byte-for-byte, `resolve_var_graphs`
included; every existing invariant, test and deny message is preserved untouched.

*Scoped path* — in order:

1. **Static graph-level check.** The graph-level authorization of `update::check` restricted to
   what is statically known: ground `GRAPH <iri>` targets, the default-graph rejection
   (`update.rs:537`), the wildcard rejection (§4.3). No evaluation, no `resolve_var_graphs`.
2. **Shadow-apply.** Materialize `M` (§4.5), `update_in_place_capturing` on a clone, capture `Δ`.
3. **Delta-derived graph-level check.** Apply the same per-graph `Need` test to the graph slots
   the delta actually names. This *replaces* the precise var resolution rather than deferring
   it — the shadow-apply enumerates the written graphs exactly, so there is nothing left to
   approximate. (Admissible alternative, same guarantee: run the binding SELECT over `M` instead
   of `D`, which restores the check-dataset ≡ apply-dataset identity at the new dataset.)
4. **Delta gate.** Authorize `Δ` triple-by-triple against `S_r`/`S_w` (§4.2, §4.3).
5. **Replay** onto `D`, only if every gate above passed.

The rule that governs all five: **no evaluation on a scoped request may read `D ∖ M`.**

One honesty note on "restriction composes, never widens": steps 1+3 are not pointwise stricter
than today's unscoped graph-level check on the same `D`. `resolve_var_graphs` over `D` can
enumerate graphs that the shadow-apply over `M` does not, so a request that today's check would
deny can be permitted on the scoped path. That difference *is* the fix — those extra graphs are
reachable only through masked bindings, and denying on them is the §2.2 one-bit oracle. The
invariant that must survive is the module's own (checked write set ⊇ actual write set), and it
does: the actual write set is the replayed `Δ`, whose slots are exactly what step 3 checked.
What never widens is authority relative to the actor's view: the scoped path can only turn a
graph-level permit into a per-triple deny (§4.3).

### 4.2 The two predicates (USING / WITH CHECK, in RDF terms)

| Predicate | Governs | v1 value |
|---|---|---|
| **Read scope** `S_r` | which triples the shadow-apply's `WHERE` and `Δ⁻` can reach | the existing `GraphScope` from the read path — unchanged type, unchanged algebra |
| **Write scope** `S_w` | which triples may appear in `Δ⁺` | **v1: `S_w := S_r`.** A separate write scope is designed (§5) but not enabled. |

Setting `S_w := S_r` by default is the Postgres rule ("`ALL` … if no `WITH CHECK` defined,
`USING` applies to both cases") and it gives the semantics a one-line statement an operator can
actually reason about: **an actor may write exactly the region it can read.** Divergent scopes
(write-only regions, append-only holes) are a real requirement — a form that may deposit an
answer into a field it cannot read back — but they reintroduce the write-then-not-read asymmetry
and need their own analysis; §7 puts the question to the maintainer.

Every deny must be a function of `(M, S, U)`. Note this permits data-dependent denials on
*visible* data — an instantiated `Δ⁺` triple that falls outside `S_w` may raise an error naming
that triple, because the actor supplied the template and can read every binding that produced it.
That is the `WITH CHECK` error and it discloses nothing.

### 4.3 Per-operation semantics (fail-closed)

Applies when the request is on the scoped path (gate 0, §4.1). "Deny" means the whole request is
rejected and `D` is untouched.

| Update form | Semantics under a scope |
|---|---|
| `INSERT DATA` | Every quad must be in `S_w` for its graph. Any quad outside ⇒ deny (the `WITH CHECK` error). Re-insert of a triple already in `M` stays a no-op. |
| `DELETE DATA` | Every quad must be in `S_r` for its graph, i.e. deletable-if-visible. A quad *in* `S_r` that does not exist stays a silent no-op — the denial is scope-dependent, never existence-dependent, so no oracle. A quad outside `S_r` ⇒ deny. |
| `DELETE/INSERT … WHERE` | Shadow-apply on `M`. `Δ⁻ ⊆ M` holds by construction; every `t ∈ Δ⁺` must be in `S_w`. Any violation ⇒ deny. |
| `DELETE WHERE` | As above with an empty insert template. |
| `CLEAR GRAPH <g>` where `g` carries a scope | **Deny.** A "clear the visible sub-graph" semantics is defensible but is a foot-gun: the actor reasonably believes `g` is empty afterwards while the masked region survives, and the owner sees a partially-emptied graph with no record of who or why. Revisit only with an explicit, named operation. |
| `DROP GRAPH <g>` where `g` carries a scope | **Deny.** Dropping the graph destroys the masked region (W2) and changes `GRAPH ?g` enumeration for every other principal. |
| `CREATE GRAPH <g>` | Permitted iff the graph-level gate permits. A fresh graph has no scope entry; it inherits whole-graph semantics. |
| `LOAD … INTO <g>` where `g` carries a scope | **Deny** in v1. `LOAD`'s content is not bounded by the request text and is already refused unless the embedder installs an allowlisted base directory (`lib.rs:1285`); pattern-checking loaded triples is possible but buys little for the risk. |
| `CLEAR`/`DROP` `ALL`/`NAMED`, or any wildcard escalation | **Deny** if the session carries any scope entry. Today these already demand write on every store graph; under scoping that demand cannot be satisfied coherently — a scope means the session does *not* hold whole-graph write. |
| Default-graph target | Denied already, unchanged (`update.rs:537`). |

Two absolute rules that sit above the table:

* **No scope may apply to an access-control document.** A pattern scope naming an `.acl`/`.acr`
  graph, a referenced group document (`loader::referenced_group_docs`), or the reserved
  `<urn:sparq:auth>` view is **rejected at scope-construction time**, and any write to such a
  graph on the scoped path is denied. A partial view of an authorization document is a rules-
  integrity hazard: an actor could insert a permission while blind to the prohibition that
  would have conflicted with it. This also keeps the re-materialization trigger
  (`affects_auth_view`, `update.rs:163`) reasoning about whole graphs only.
* **The scoped path never widens.** It can only convert a graph-level *permit* into a deny, per
  triple. A session with no graph-level write on `g` gains nothing from a scope on `g` — the
  mirror of the read path's `scoped_dataset` refinement rule.

### 4.4 What the actor can observe

`update_as` returns `Result<(), String>`, and the engine's non-capturing apply returns no counts,
so the scoped path adds **no cardinality channel** provided the delta gate's error messages are
derived only from `(M, S, U)`. Two disciplines follow:

* A deny message may name a triple from `Δ⁺` (actor-supplied or derived from visible bindings)
  and may name a scope pattern. It must **not** name a triple from `D ∖ M`, and it must not
  report how many triples were skipped, filtered, or masked.
* If a future API surfaces an effect log or an affected-triple count for scoped updates, that
  value must be computed over `M`, which makes it the oracle-equivalent value by construction.

Related, and **out of scope for this bead**: today's wildcard deny message names the first store
graph the session cannot write (`update.rs:601`), which discloses a graph name the session may
not be able to read. Pre-existing, unrelated to scoping, and captured as a follow-up rather than
fixed here.

### 4.5 Cost

The replica is `O(|reachable accessible dataset|)` per scoped update, paid before the apply.
Two honest reductions, neither of which changes the semantics:

* **Reach-limited materialization.** Materialize only the graphs the request can touch: the
  static target set ∪ the graphs the `WHERE` can read. Both are exact when every `GRAPH` slot is
  a constant; a `GRAPH ?var` slot or a bare pattern under the legacy union-default semantics
  forces the whole accessible set. Fail-closed: when in doubt, materialize more.
* **Replica cache reuse.** Bead `sq-nc3c6` (read-path replica cache keyed by graph name × scope
  fingerprint) is directly reusable — a scoped update invalidates only the replicas of the graphs
  its delta touched, which the delta names exactly.

No numbers appear here; the measurement obligation is bead 6 in §8, and any figures produced on a
work box are non-canonical.

## 5. Vocabulary (designed, downstream of `sq-qnlj8`)

The read record's §5 mints `sparq:PatternAsset` (a `sparq:sourceGraph` plus ≥1 `sparq:pattern`)
and materializes an `auth:PatternGrant` carrying `auth:agent`, `auth:mode`, `auth:graph` and
`auth:allowPattern`/`auth:denyPattern`. Write authority needs **no new node kind** — only the
mode axis already present:

* `odrl:target <PatternAsset>` on a permission whose action maps to a write mode
  (`odrl:modify`/`odrl:delete`-family → `auth:write`, an append-only action → `auth:append`)
  materializes an `auth:PatternGrant` with `auth:mode auth:write` (resp. `auth:append`). The
  `AuthIndex` extraction already keys scopes per `(principal, mode, graph)`, so a write scope is
  the same product at a different mode.
* A prohibition targeting a `PatternAsset` at a write mode contributes deny patterns, resolved by
  the same deny-overrides walk.
* **Fail-closed parse rules, extending the read record's:** a pattern grant at a write mode whose
  `sourceGraph` is an `.acl`/`.acr`/group document or the auth view materializes **nothing**; a
  write-mode `PatternAsset` with zero parseable patterns or any non-concrete component
  materializes **nothing** (absence of grant, never a whole-graph fallback).
* If `S_w` is ever allowed to diverge from `S_r` (§7), it is simply the `auth:write`-mode scope
  where `S_r` is the `auth:read`-mode scope — no vocabulary change, only a policy-authoring
  hazard (an actor granted write on a region it cannot read).

`Rule::target` in `sparq-policy` stays a bare IRI, exactly as the read record commits.

## 6. Acceptance obligations (what would make this real)

An implementation of §4 is only credible with these, and they are the acceptance tests the
follow-up beads must carry:

1. **Non-interference differential (the law of §1).** Build `D₁` and `D₂` that agree on
   `masked(·, S)` but differ arbitrarily in the masked region. For a battery of updates — every
   form in §4.3, plus the §2.2 var-graph shape — assert: identical verdict, identical deny
   message, and `masked(apply(D₁,U), S) = masked(apply(D₂,U), S)`. This is the test that fails
   red on the §2.2 leak, and it is the one test that cannot be replaced by an oracle comparison.
   Two routing-specific cases are mandatory, because gate 0 (§4.1) is where the leak is actually
   closed:
   * A **`GRAPH ?var` update whose bindings differ only in `D ∖ M`** — `D₁` and `D₂` therefore
     resolve different concrete target sets — must still produce the identical verdict, message
     and masked post-state. This is the §2.2 shape stated as a test.
   * **The classifier is store-independent.** Assert `scoped_path(S, U)` is identical for `D₁`
     and `D₂` for every case in the battery, and make it non-vacuous by mutation: replacing gate 0
     with the naive *"does any **resolved** target carry a scope entry?"* test must turn the
     battery red. A test suite that passes under that mutation is not testing the routing.
2. **Write-oracle differential.** `apply_scoped(D, U)` must equal `D` updated by the delta a
   *physically-reduced* store would produce — the write mirror of `tests/pattern_scope.rs`'s
   `scoped == oracle` battery, with the same non-vacuity guard (a no-op mask must flip it red).
3. **Integrity assertions.** No triple of `D ∖ M` is removed or altered by any permitted scoped
   update (W2); no triple outside `S_w` appears in `D` after any permitted scoped update (W1).
4. **Randomized fuzz.** Extend the existing deterministic SplitMix64 harness in
   `tests/pattern_scope_fuzz.rs` (64 seeded cases, `case=<i> seed=<hex>` in every message) to
   update batteries — the cheapest path to covering update forms nobody enumerated.
5. **Non-regression of the graph level.** `update.rs::differential_writeset_tests` must stay
   green untouched, and an unscoped session's behaviour must be byte-identical (feature OFF ⇒ no
   code change; feature ON but no scope entry ⇒ same path).
6. **Mutation check.** Flipping an expected value in each new test must turn it red.

## 7. Open questions for the maintainer

1. **May write authority diverge from read authority?** v1 proposes `S_w := S_r` ("write exactly
   what you can read"). Divergent scopes enable a real pattern — a drop-box region an actor may
   append to but not read — at the cost of an asymmetry that is much harder to explain and audit.
   Enable, or defer?
2. **`CLEAR`/`DROP` on a scoped graph: deny, or define "clear the visible sub-graph"?** §4.3
   chooses deny on foot-gun grounds. The alternative is coherent under the law but surprising.
3. **Term-identity matching on the write path.** The read path matches scope patterns by term
   identity, not value equality (`pattern_scope.rs:32`), so `"01"^^xsd:integer` does not match a
   pattern naming `"1"^^xsd:integer`. On the read path that under-hides. On the **write** path it
   is an *evasion* vector: an actor can write a value-equal but syntactically different term into
   a denied region. Options: accept and document; canonicalize literals before matching; or
   restrict write-mode deny patterns to the subject/predicate positions. This needs a decision
   before any implementation.
4. **Ordering vs `sq-qnlj8`.** Should enforcement land ahead of the ODRL bridge wiring (scopes
   supplied programmatically only, as today), or should the bridge land first so the feature is
   reachable end-to-end from a policy? §8 assumes enforcement-first, since the bridge's write-mode
   extension (§5) is small once the semantics are fixed.
5. **Is a scoped update allowed to trigger re-materialization at all?** §4.3 forbids scoping
   control documents, which makes the question moot for `.acl`/`.acr`. Confirm that is the
   intended posture rather than "scoped writes to an `.acl` with a scope covering only the
   actor's own authorizations".

## 8. Proposed follow-up beads

Ordered; each is single-crate (`sparq-solid`) and gated on the OFF-by-default `pattern-scope`
feature unless noted. Ids to be minted by the orchestrator — this record creates none.

1. **`spec+test`: the non-interference harness.** Land §6 obligation 1 as a *failing-by-absence*
   test scaffold over the existing read-path masking plus today's unscoped `update_as`: two
   datasets agreeing on the masked view, asserting equal verdicts. Cheap, and it is the acceptance
   gate every later bead is measured against. No production code.
2. **`feat`: `WriteScope` + the delta gate.** The pure predicate layer — authorize a
   `(Δ⁻, Δ⁺)` pair against a scope map, with the §4.3 per-form rules and the control-document
   rejection at scope construction. No store plumbing; unit-testable in isolation.
3. **`feat`: shadow-apply plumbing — `PodStore::scoped_update_as`.** The gate 0 classifier (§4.1)
   as a pure `(S, U)` function — it must land here, since nothing can be routed without it, and
   may ship as the degenerate "any scope entry ⇒ scoped" rule. Then materialize `M` (reach-limited
   per §4.5), `update_in_place_capturing` on a clone, gate the delta, replay. Carries §6
   obligations 2 and 3.
4. **`fix`: re-scope the graph-level resolution on the scoped path.** The §2.2 leak fix — the
   delta-derived graph check of §4.1 step 3 (or the binding SELECT over `M`), plus the syntactic
   refinement of gate 0 past the degenerate rule. Guarded by bead 1's harness, which must go red
   without it — including the mutation to the naive resolved-target routing test.
5. **`feat`: fuzz extension.** §6 obligation 4 — update batteries in
   `tests/pattern_scope_fuzz.rs`.
6. **`perf`: replica reuse + measurement.** Reach-limited materialization tuning and cache reuse
   from `sq-nc3c6`; record the envelope under `bench/pattern-scope/` (work-box, non-canonical).
7. **`feat`: ODRL write-mode pattern grants.** §5 — strictly downstream of `sq-qnlj8`.
8. **`docs`: surface sync.** `crates/sparq-solid/README.md` +
   `skills/usage-control-policy/SKILL.md` §pattern targets, which currently states the write path
   is out of scope and must be corrected in lock-step with bead 3 — not before.

## 9. Sources

* PostgreSQL `CREATE POLICY` (`USING` vs `WITH CHECK`, silent-filter vs error, the
  referential-integrity disclosure note) — <https://www.postgresql.org/docs/current/sql-createpolicy.html>
* Microsoft SQL Server *Row-Level Security* (filter vs block predicates, the four block-predicate
  operations, the "carefully crafted queries" side-channel note) —
  <https://learn.microsoft.com/en-us/sql/relational-databases/security/row-level-security>
* In-repo: `crates/sparq-solid/src/update.rs`, `crates/sparq-solid/src/pattern_scope.rs`,
  `crates/sparq-solid/src/lib.rs`, `crates/sparq-engine/src/update.rs`,
  `crates/sparq-core/src/store.rs`, `crates/sparq-solid/tests/pattern_scope_fuzz.rs`.
