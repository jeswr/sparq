# Pattern-scoped UPDATE enforcement — row-level write authority over a masked graph

Status: **design record only** (spike `sq-fznmq`, epic `sq-lrtc3`). No implementation
accompanies this record. Parent record:
[`odrl-pattern-scoped-targets-2026-07.md`](odrl-pattern-scoped-targets-2026-07.md) §2.4,
which deferred the UPDATE surface with the words *"blind writes into masked regions,
delete-visibility interactions"*. Companion records:
[`solid-access-control-design.md`](solid-access-control-design.md) (D1–D4, the §4.4 write
path this must extend), [`trust-graph-authorisation-2026-07.md`](trust-graph-authorisation-2026-07.md)
(the fail-closed enforcement law). Implementation surfaces surveyed:
`crates/sparq-solid/src/update.rs`, `crates/sparq-solid/src/pattern_scope.rs`,
`crates/sparq-solid/src/lib.rs` (`PodStore::update_as`).

<!-- [OPUS-5] 🤖 SPARQ agent. Design-first spike, per the bead: options + fail-closed
semantics BEFORE any implementation. §1 records a MEASURED correction to the premise. -->

> **Honest scope.** Clear-path authorisation only — no cryptographic guarantee is claimed
> anywhere in this record. Nothing here is implemented; every "would" is a proposal for
> maintainer review, not a description of shipped behaviour. No performance numbers appear
> in this record.

## 0. What the bead asks for

`sq-fznmq` asks for a design record covering **options + fail-closed semantics under
INSERT / DELETE / WHERE over masked regions**, before any implementation. The three
questions §2.4 named:

1. **Blind write** — may a principal write into a region of a graph it cannot *see*?
2. **Delete-visibility** — may a `DELETE` remove triples the principal cannot see?
3. **Masked-region overwrite** — what happens when an `INSERT` lands on (or recreates) a
   triple that the principal's read scope masks?

This record answers all three, and adds three the parent record did not name: the WHERE
clause as a read channel (§1, and it is the load-bearing one), the accept/reject verdict
as a channel (§3, L7), and the interaction with the store's set semantics that turns
insertion into a membership oracle (§3, L4).

## 1. Correction to the premise, and a MEASURED precondition

### 1.1 The stated blocker is cleared; a different one is not

The bead says this work is *"blocked conceptually on the read-path design being merged
(sq-lrtc3.3)"*. That blocker **is cleared** — the read-path design record and its
prototype merged in `ac66102a` (PR #1951) and `crates/sparq-solid/src/pattern_scope.rs`
is on `main` behind the OFF-by-default `pattern-scope` feature.

But the premise that a *pattern-scoped write grant* has somewhere to attach is **not yet
true**. Verified on this checkout: `PatternAsset`, `PatternGrant` and `allowPattern` do
not appear anywhere under `crates/` — the ODRL bridge wiring designed in the parent
record §5 is still the unstarted follow-up `sq-qnlj8`. Today the ONLY way a `GraphScope`
reaches enforcement is a caller passing an `FxHashMap<Term, GraphScope>` by hand to
`PodStore::scoped_dataset`. So a pattern-scoped *write* has, at present, no policy source
at all. This does not block the design; it does mean the phased plan (§7) must order the
read-path bridge wiring **before** any write-path implementation, and it means no write
bead should be launched as "just extend the existing grant kind" — the grant kind does not
exist yet.

### 1.2 P0 — the WHERE clause is an unauthorized read channel (measured, live on `main`)

Any pattern-scoped write design rests on the write path already being sound at *graph*
granularity for the data an update can observe. It is not.

`update::check` authorizes only **write targets**. Its own comment states the position
explicitly: *"The WHERE pattern only READS, so it is not a write target"*
(`crates/sparq-solid/src/update.rs:246-247`). Consequently `Mode::Read` never appears in
`update.rs`, and `PodStore::update_inner` hands the **full store** to
`sparq_engine::update_in_place` (`crates/sparq-solid/src/lib.rs:1267`). An update's WHERE
therefore evaluates over every graph in the store, including graphs the actor has no
`acl:Read` grant on, and the resulting bindings can be materialized into a graph the actor
*can* write and then read back.

**Measured**, not inferred. A temporary integration test over the crate's own
`wac_fixture()` (written, run, and deleted — no diff is proposed by this record):

```text
PROBE writable    = https://pod.ex/team2/                 (CAROL: group Read+Write)
PROBE secret      = https://pod.ex/friends3/c0/g0/d0.ttl  (CAROL: no grant at all)
PROBE direct-read = {"head":{"vars":["o"]},"results":{"bindings":[]}}      <- cannot read it
PROBE update      = INSERT { GRAPH <writable> { ?s ex:leaked ?o } }
                    WHERE  { GRAPH <secret>   { ?s ex:title  ?o } }
PROBE verdict     = Ok(())                                                <- PERMITTED
PROBE read-back   = {... "value":"doc 3-0-0-0"}                           <- exfiltrated
```

The literal `"doc 3-0-0-0"` lives only in a graph CAROL provably cannot read, and after the
permitted update she reads it out of a graph she can. Nothing in `research/` documents this
as an accepted trade-off (searched: no statement about read authorization on an update's
WHERE anywhere in `research/` or `skills/access-control/SKILL.md`).

This is filed as an out-of-scope follow-up rather than fixed here (design-only bead). It is
a **precondition**: masking a *region* of a graph is meaningless while an update's WHERE can
read *whole graphs* unauthorized — a pattern-scoped principal would simply write
`INSERT { GRAPH <mine> { ?s ?p ?o } } WHERE { GRAPH <scoped> { ?s ?p ?o } }` and recover the
masked triples verbatim. §5's algorithm closes it as a side effect (it evaluates the WHERE
over the actor's scoped read replica), but it should be fixed at graph granularity FIRST,
on its own, so the fix is not gated behind an OFF-by-default feature.

## 2. Options

Throughout: `R(g)` is the principal's read scope on graph `g` (the existing `GraphScope`),
and a write scope is what this record is deciding the shape of.

### W-A. Refuse pattern-scoped write grants — the fail-closed status quo

A `PatternGrant` carrying a write-family mode materializes **nothing**; row-level write
authority is simply not expressible. `update_as` keeps today's graph-granular enforcement
unchanged.

Costs nothing, adds no attack surface, and is what the code does today. It is the correct
**default** and the correct behaviour of any build without the new feature — §5's rule
"an unparseable or write-moded pattern grant grants nothing" is exactly D4 (absence = deny).
Rejected as the *whole* answer only because the maintainer invariant ("an ODRL policy over
(requester, action, target=graph|pattern) gates the query") names `odrl:modify` and
`odrl:append` alongside `odrl:read`, and `action_to_mode`
(`crates/sparq-solid/src/odrl_bridge.rs:197`) already maps them to `Mode::Write` /
`Mode::Append`. A pattern-targeted `odrl:modify` permission is expressible in ODRL today
and would silently do nothing.

### W-B. Query rewrite — inject guards into the update's templates and WHERE — REJECTED

Rewrite the incoming update so each template quad carries a scope guard. Rejected for the
parent record §1-B reason (a rewrite must cover every construct and survive the optimizer),
plus one that is specific to updates and strictly worse: templates are **instantiated
per-solution**, so the concrete quads written are not known until the WHERE has evaluated.
A static rewrite cannot bound them; it can only approximate, and an approximation on the
write side is a silent over-grant.

### W-C. Post-hoc audit — apply, then delete what was out of scope — REJECTED (unsound)

Apply the update, then remove any quad that fell outside the write scope. Unsound by
construction, for the parent record §1-C reason and one more: a multi-operation body
(`DELETE … ; INSERT …`) reads the store between operations, so an out-of-scope quad written
by operation 1 is *observable to operation 2* before the audit runs. Listed only because it
is the tempting-cheap option.

### W-D. In-engine per-quad veto hook — REJECTED for v1, kept as the v2 performance path

Add a callback the engine's apply consults per candidate quad. This is the write-side twin
of the in-line scan filter the parent record §1-A rejected, and it carries the same
liability: the apply path has multiple write entry points (`update_in_place`,
`update_in_place_capturing`, `update_in_place_atomic`, `apply_effects`, plus the GSP
`PUT`/`POST`/`DELETE`/`PATCH` translations in `sparq-server`), and missing one is a silent
over-grant rather than a test failure. Revisit only if §5's cost is ever measured to
dominate, and only behind a fuzzed differential harness across every write entry point.

### W-E. Evaluate-on-masked-replica, authorize the concrete delta, replay — **CHOSEN**

Fork the principal's **masked read replica**, apply the update to the fork, capture the
concrete `UpdateEffect::Delta` the engine produced, authorize **those literal quads**
against the write scope, and replay the authorized delta onto the real store. This is the
write-side analogue of the read side's "materialize, don't filter" decision and inherits
its soundness argument in both directions:

* the update's WHERE sees the **oracle dataset** (`D ∖ masked-triples`), so masked triples
  cannot influence a binding, an `EXISTS`, a `MINUS`, an `OPTIONAL` or an aggregate inside
  an update any more than inside a query — an identity, not a per-construct audit;
* the authorization check sees **concrete ground quads**, not a pattern approximation of
  them, so there is no gap between "what was checked" and "what is written" — the same
  reason the existing `resolve_var_graphs` differential guard
  (`crates/sparq-solid/src/update.rs:604`) asserts precise-set == engine-write-set.

Every primitive it needs already exists and is public: `Graph::fork()` (O(pending delta),
not O(triples)), `sparq_engine::update_in_place_capturing` → `Vec<UpdateEffect>`,
`UpdateEffect::Delta { slot, inserts, deletes }`, and `sparq_engine::apply_effects`. **No
engine or core change is required** — the same "zero engine changes" property the read-path
prototype has. Its cost and its honest semantic caveats are §5.4 and §6.

## 3. Fail-closed semantics — the write-side laws

The read-side leakage bar asks: can a masked triple be *observed*? The write side has to ask
two questions, and conflating them is the classic error:

* **visibility** — which existing triples may this operation *touch*?
* **validity** — which triples may this operation *leave behind*?

The prior art that gets this right is PostgreSQL row-level security, which splits a policy
into a `USING` expression ("determines which existing rows are visible to the user") and a
`WITH CHECK` expression ("determines which new or modified rows are allowed to be written
back") — `SELECT`/`DELETE` consult `USING` only, `INSERT` consults `WITH CHECK` only, and
`UPDATE` consults both. Crucially the two differ in *failure mode*: a row failing `USING`
is **silently filtered**, a row failing `WITH CHECK` **raises an error**. That asymmetry is
not an accident and §3's laws reproduce it.

So a write scope is a **pair**, never a single predicate:

| Symbol | ODRL source (proposed) | Role |
|---|---|---|
| `R(g)` | `odrl:read` on a `PatternAsset` | read scope — the existing `GraphScope` (`USING`) |
| `Wdel(g)` | `odrl:modify` / `odrl:delete` on a `PatternAsset` | which existing triples may be REMOVED |
| `Wins(g)` | `odrl:modify` / `odrl:append` on a `PatternAsset` | which triples may be ADDED (`WITH CHECK`) |

### L1 — Refinement, never widening

A pattern write scope may only **shrink** the graph-level write grant. `update::check` runs
FIRST and unchanged; a scope entry on a graph the graph level denies contributes nothing,
exactly as `scoped_dataset` refuses to widen on the read side (parent §2.2). Restriction
composes per layer.

### L2 — Delete implies read: no blind delete

A quad may be deleted only if it is **both** in `Wdel(g)` **and** visible under `R(g)`.

This is Postgres's `USING` law and it is the direct answer to §2.4's *delete-visibility*
question. Deleting a triple you cannot see is a probe: the delete either changes the store
or does not, and any later observation (a read-back, a re-insert, a subsequent conditional
update) distinguishes the two. Under §5's algorithm the law is **automatic** — the fork is
the masked replica, so a masked triple is not there to be matched by a `DELETE … WHERE` and
a `DELETE DATA` naming it is a no-op on the fork and therefore absent from the captured
delta. Corollary, and it is a good one: a principal with write-but-no-read on a graph has
`R(g) = ∅`, so it can delete **nothing** — fail-closed with no special case.

Note this makes `Mode::Append` structurally safe: append cannot delete at all, so L2 is
vacuous for it.

### L3 — Insert must be in scope, and an out-of-scope insert DENIES

Every concrete quad the apply would insert must be in `Wins(g)`. A quad outside it
**denies the whole update** with an error — it is not silently dropped.

Silently dropping is the tempting choice (it mirrors L2's silent filtering) and it is
wrong: the principal would believe it had written data that is not there, and a subsequent
read-back returning nothing becomes an oracle for the *scope boundary* rather than for the
data. An error is safe here precisely because the inserted quad is the principal's **own
input** — telling it "this quad is outside your write scope" reveals nothing it did not
already supply. This is exactly the Postgres `WITH CHECK` failure mode, for the same reason.

### L4 — You may only write where you can read: `Wins(g) ⊆ visible(R(g))` and `Wdel(g) ⊆ visible(R(g))`

This is the direct answer to §2.4's *blind write* and *masked-region overwrite* questions,
and it is the law this record most wants the maintainer to rule on (§8).

Why `Wins ⊆ R` rather than an independent `Wins`: an RDF graph is a **set**. Inserting a
triple that is already present is a no-op; inserting one that is absent changes the store.
If a principal may insert into a region it cannot read, then insert-then-delete (or
insert-then-observe-any-downstream-effect: a SHACL validation outcome, a re-materialization,
a `CREATE`-conditional) is a **membership oracle** over the masked region — it recovers
exactly the bit that masking exists to hide. The parent record's leakage bar names the
counting side-channel for reads; this is its write-side twin, and it is worse because the
attacker chooses the probe.

The clean, cheap, fail-closed rule is therefore: **derive the write scopes from the read
scope by intersection, never independently.** A pattern-targeted `odrl:modify` on graph `g`
grants `Wdel(g) = Wins(g) = patterns ∩ visible(R(g))`; a principal with no read scope on `g`
gets no pattern write authority on `g` at all.

The cost of this rule, stated honestly: it forbids the **blind drop-box** — an append-only
audit log or inbox where the writer must NOT be able to read back what it (or anyone) wrote.
That is a real Solid use case (`ldp:inbox`). It is deliberately out of scope here: making it
safe needs a write-only scope kind whose leak analysis has to defeat the set-semantics
oracle above (plausibly by making the insert unconditionally succeed regardless of prior
membership, which the current store cannot express). §7 beads it separately rather than
smuggling it in.

### L5 — The WHERE is a read, and it must be scoped — but the write-set enumeration must not be

The two evaluations `update.rs` currently conflates must be separated, and they must
approximate in **opposite directions**:

* the **authorization enumeration** (`resolve_var_graphs`: which graphs could a `GRAPH ?var`
  slot bind?) must keep evaluating over the **full store**. Its existing soundness note is
  correct — narrowing it could miss a graph the apply would write, which is a hole. Over-
  approximating here can only deny. **Unchanged.**
* the **apply's WHERE** must evaluate over the principal's **scoped read replica**.
  Under-approximating the bindings here can only write *less*, which is safe.

Today both are the same evaluation because `update_in_place` receives the raw store, which
is P0 (§1.2) at graph granularity and would be a straight masked-region leak at pattern
granularity. §5 keeps them separate by construction.

### L6 — No new counting channel

`PodStore::update_as` returns `Result<(), String>` — it reports no affected-triple count.
That is a *property*, and this design must preserve it. Any future "n triples changed"
return value is a counting side-channel over the masked region (delete a wildcard pattern,
read the count, learn how many masked triples matched) and must not be added without a
`Wdel ⊆ R` argument. Recorded here so a later ergonomics PR does not reintroduce it
accidentally.

### L7 — Uniform denial: the verdict must not describe the data

A deny must name only the **scope that was violated**, never the offending quad's content:
two denials raised by the same law over *different* quads must produce a **byte-identical**
message. Otherwise the error string is itself the oracle L2 and L4 exist to close.

Note carefully what this law does *not* say. Under §5.1 a `DELETE` naming a triple that is
absent — or masked, which is the same thing on the fork — produces no delete effect and so
no deny at all; it succeeds as a no-op (§4). "The triple you named does not exist" therefore
never reaches an error string in the first place, and there is no message to make
indistinguishable from anything. The indistinguishability that matters is *within* the deny
path, and it is an INSERT property: an out-of-scope insert must deny identically whether or
not the quad it names is already present behind the mask (§5.3 item 4).

The cautionary prior art is Postgres's own documented residual channel: *"they are
not applied when the system is performing internal referential integrity checks or
validating constraints … attempting to insert a duplicate value into a column that … has a
unique constraint. If the insert fails then the user can infer that the value already
exists."* The estate already has this discipline elsewhere — `N3PatchError` deliberately
withholds parse detail from the client so it never echoes attacker-controlled term text
(`crates/sparq-server/src/n3_patch.rs`) — and it should be the same discipline here.

### L8 — Graph-granular operations are refused under a pattern scope

`CLEAR`, `DROP` and `CREATE` are graph-granular by nature — they cannot be expressed as a
row predicate, and "clear the part of the graph you can see" is a different operation from
`CLEAR` with different observable consequences (the graph continues to exist). Under a
pattern write scope they are **denied**, not reinterpreted. `LOAD` likewise: its content is
not known at check time, so its quads must go through the same delta check as any insert or
be denied.

### L9 — Re-materialization and cache invalidation still apply

A pattern-scoped write that lands in an `.acl`/`.acr`/group document must re-materialize
exactly as today (`update.rs` `Permit::rematerialize`). Additionally, any write to `g`
invalidates every cached masked replica of `g` — so the `sq-nc3c6` replica cache key must
include the store generation, not just the scope fingerprint. Cheapest correct reading: the
write path bumps the generation and stale entries are dropped, which the `session_cache`
epoch precedent already does for the auth view.

## 4. The semantics this yields, stated as an equivalence

The read side's defensible definition was oracle equivalence:
`eval(masked(D), Q) = eval(D ∖ masked, Q)`. The write side's analogue, and the property §5
is built to make true:

> For an update `U` permitted under §3, the store transition is exactly the one the
> principal would have caused **if the masked triples genuinely did not exist** — i.e.
> `apply(D, U)` restricted to the authorized delta equals `D ⊎ delta(D ∖ masked, U)`.

This has a consequence worth stating plainly rather than hiding, because a reviewer will
otherwise find it and call it a bug: **an actor's own update cannot see, and therefore
cannot touch, the masked triples of a graph it is otherwise writing.** A
`DELETE { GRAPH <g> { ?s ex:phone ?o } } WHERE { GRAPH <g> { ?s ex:phone ?o } }` issued by a
principal whose scope masks `ex:phone` deletes nothing and reports success. That is not a
defect — it is precisely the "behaves exactly as if the masked triples were physically
absent" contract, extended to writes, and any other answer reintroduces L2's probe. It does
mean the property only holds for deltas whose quads are all in scope, which is what L3's
deny (rather than a silent drop) enforces.

## 5. Proposed mechanism (W-E), for review — NOT implemented

A new OFF-by-default feature `pattern-scope-write` (implying `pattern-scope`), adding ONE
method. `PodStore::update_as` / `update_as_acp` are **untouched**, so a build without the
feature is byte-identical and the semantic change of §4 is opt-in.

### 5.1 The algorithm

```text
update_scoped_as(session, sparql, scopes) -> Result<(), String>:
  1. permit = update::check(full_store, auth, session, sparql, group_docs)?   # unchanged, first, L1
  2. if no graph the update targets carries a scope entry:
        fall through to today's update_as   # the common case; zero added cost (§6)
  3. S = self.scoped_dataset(session, Mode::Read, scopes)   # masked read replica       L5
  4. F = S.graph.fork()                                     # O(pending delta)
  5. effects = sparq_engine::update_in_place_capturing(&mut F, sparql, budget)?
  6. for each effect:
       Delta { slot: Some(g), inserts, deletes } =>
           every d in deletes must satisfy  Wdel(g).visible(d)   else DENY   # L2 (masked
                                                                            # triples are
                                                                            # already absent
                                                                            # from F)
           every i in inserts must satisfy  Wins(g).visible(i)   else DENY   # L3, L4
       Delta { slot: None, .. }                => DENY   # default graph, as today
       Clear | Drop | Create                   => DENY   # L8
  7. drop F                                                # nothing was written anywhere real
  8. sparq_engine::apply_effects(&mut self.graph, &effects)?   # replay onto the REAL store
  9. if permit.rematerialize { materialize_wac()/materialize_acp() }
     invalidate cached replicas for every touched graph                       # L9
```

Steps 4–7 mutate only a fork, so the deny path leaves the real store canonically identical —
the property `denied_single_op_leaves_store_canonically_identical`
(`crates/sparq-solid/tests/update.rs:468`) already pins for the current path, and the new
path should be added to that same test.

### 5.2 Where the write scopes come from

Triples-native (D1), structurally parallel to the parent record §5's read-side
`auth:PatternGrant`, and consumed at materialization time:

```turtle
<urn:sparq:auth> {
  _:pg a auth:PatternGrant ;
       auth:agent   <https://alice.ex/card#me> ;
       auth:mode    auth:write ;          # or auth:append  (read grants use auth:read)
       auth:graph   <https://pod.ex/contacts> ;
       auth:allowPattern [ auth:predicate <https://ex.dev/ns#nickname> ] .
}
```

`AuthIndex` gains one parsed node kind producing a `GraphScope` per `(principal, mode,
graph)` — the same shape the read side needs, keyed additionally by mode. Proposed ODRL
mapping, consistent with the existing `action_to_mode` table:

| ODRL action on a `PatternAsset` | Yields |
|---|---|
| `odrl:read` | `R(g)` |
| `odrl:append` | `Wins(g)` only (no delete authority) |
| `odrl:modify`, `odrl:write` | `Wins(g)` and `Wdel(g)` |
| `odrl:delete` | `Wdel(g)` only |
| anything else | nothing (fail-closed, as today) |

Fail-closed parse rules, inherited verbatim from the parent record §5: a `PatternAsset`
with zero parseable patterns, an ambiguous `sourceGraph`, or a non-concrete component
materializes **nothing** — never a whole-graph fallback. Added here: a write-moded pattern
grant on a graph carrying **no** read scope for the same principal materializes nothing
(L4), and every materialized write scope is intersected with the read scope at index-build
time so the L4 invariant is established once rather than re-checked per update.

### 5.3 What must be tested before this could be called sound

Not "would be nice" — the acceptance bar, mirroring what the read path already carries
(`tests/pattern_scope.rs` battery + `tests/pattern_scope_fuzz.rs` randomized differential):

1. **Write-side differential oracle.** For a random (dataset, scope, update) triple, the
   store after `update_scoped_as` must equal the store after applying the same update to an
   ORACLE `PodStore` whose masked triples were physically deleted, then re-merging the
   masked triples untouched. Non-vacuity: a no-op mask must flip it red. This is the write
   twin of `pattern_scope_fuzz.rs` and should reuse its SplitMix64 harness.
2. **L2 probe battery.** Every shape that could reveal a masked triple through a delete:
   `DELETE DATA` naming it, `DELETE … WHERE` matching it, `DELETE/INSERT` with the masked
   triple in an `OPTIONAL`/`MINUS`/`EXISTS` inside the WHERE.
3. **L4 membership-oracle battery.** Insert a triple that is masked-and-present vs
   masked-and-absent; the observable outcome (verdict, store, subsequent read) must be
   identical in both cases.
4. **L7 uniform-denial battery.** Compare messages only across pairs that *both* actually
   deny. Under §5.1 a `DELETE` of an absent or masked quad does not deny — it is a silent
   no-op (§4, L7) — so it has no message to compare, and the delete-side obligation is
   discharged as *successful* indistinguishability by item 2, not here. The two denying
   pairs, both under L3 (an insert outside `Wins`):
   * **Content independence.** Two out-of-scope inserts whose offending quads carry
     *different* attacker-supplied term text must produce byte-identical messages — the
     message names the violated scope and nothing about the quad.
   * **Scope-boundary indistinguishability.** Item 3's pair — a quad masked-and-present vs
     masked-and-absent, both outside `Wins` by L4 and so both denied — must produce
     byte-identical messages in addition to item 3's identical store and subsequent read.
5. **Delta-parity guard.** The quads authorized in step 6 must equal the quads
   `apply_effects` writes in step 8 — the write twin of the existing
   `differential_writeset_tests` module.
6. **Feature-off byte-identity.** `update_as` behaviour unchanged; both feature states green
   under `clippy -D warnings`.

### 5.4 Known gaps in this mechanism — open, not hand-waved

* **Blank-node minting across the replay boundary.** An `INSERT` that mints fresh blank
  nodes does so in the *fork's* dictionary. Fork-local *ids* are safe by construction
  (`Graph::fork` interns new terms above the shared base's high-water mark), but the
  captured `UpdateEffect::Delta` carries **terms**, and `apply_effects` re-interns those
  blank-node LABELS into a different graph — the real store — where a label of the same
  name may already denote a different node. Whether that can collide has NOT been verified.
  It must be, before implementation; if it can, the design needs an explicit skolemization
  step or must deny blank-node-minting inserts under a scope.
* **Multi-operation bodies read the replica between operations.** `DELETE … ; INSERT …`
  evaluates operation 2 against the post-operation-1 *replica*. Per §4 that is the intended
  semantics (the actor's world is the masked world), but it means the replayed delta is the
  delta of the masked world, and it is the reason the equivalence in §4 is stated over the
  authorized delta rather than over the raw store.
* **`update_in_place_capturing` on error leaves the fork partially applied.** Harmless (the
  fork is discarded) but the error must not be forwarded verbatim if it can quote masked
  term text — L7 applies to engine errors too.
* **Cost.** Step 3 is O(accessible dataset) per update, which is the wrong shape for a
  write-heavy path. Mitigations: step 2's fast path (no scope on any targeted graph ⇒ zero
  added cost, and this is the overwhelmingly common case), and the `sq-nc3c6` replica cache
  once L9's generation-keyed invalidation exists. No numbers are claimed; a measured
  envelope, in `bench/` and non-canonical, is part of the implementation bead.

## 6. Recommendation

1. **Fix P0 first, at graph granularity, as its own bead** (§1.2). Require the actor to hold
   `Mode::Read` on every graph an update's WHERE can range over — resolvable with the
   machinery `resolve_var_graphs` already has — or evaluate the apply's WHERE over the
   actor's authorized view. This is independent of pattern scopes, is a live read-
   authorization bypass, and must not be gated behind an OFF-by-default feature.
2. **Ship W-A as the standing default**, permanently: a write-moded pattern grant grants
   nothing unless the opt-in feature is built AND enabled. This is D4 and costs nothing.
3. **Wire the read-path bridge (`sq-qnlj8`) before any write work** — the grant kind a write
   scope extends does not exist yet (§1.1).
4. **Then implement W-E** behind `pattern-scope-write`, with §3's laws and §5.3's acceptance
   bar, and with L4 (`Wins, Wdel ⊆ R`) as a hard structural invariant established at
   index-build time rather than a per-update check.
5. **Do not build the blind drop-box** in this program. Bead it separately with its own leak
   analysis.

## 7. Phased plan (future beads, ordered; each is a dependency of the next)

1. `spike→feat(solid)`: **P0 fix** — require read authorization for an update's WHERE at
   graph granularity, with the exfiltration probe of §1.2 as the failing acceptance test.
   Independent of everything else here; highest priority.
2. `feat(solid)`: the parent record's `sq-qnlj8` — `sparq:PatternAsset` parsing +
   `auth:PatternGrant` materialization + `AuthIndex` read-scope extraction. Unchanged from
   the parent plan; listed because everything below depends on it.
3. `feat(solid)`: extend `auth:PatternGrant` to carry `auth:mode` write/append and produce
   `Wins`/`Wdel` scopes, **intersected with `R` at index-build time** (L4). Index/vocabulary
   only — no enforcement, no new public update method. Ships with the ODRL action→scope
   mapping table of §5.2 and its fail-closed parse tests.
4. `feat(solid)`: `PodStore::update_scoped_as` behind `pattern-scope-write` — the §5.1
   algorithm, with L1/L2/L3/L5/L8 and the step-2 fast path. Acceptance: §5.3 items 1, 5, 6.
5. `test(solid)`: the leakage batteries — §5.3 items 2, 3, 4 (L2 probes, L4 membership
   oracle, L7 uniform denials), as a randomized differential twin of
   `tests/pattern_scope_fuzz.rs`.
6. `feat(solid)`: generation-keyed replica-cache invalidation on the write path (L9),
   folding into the parent record's `sq-nc3c6`.
7. `spike(solid)`: write-only ("blind drop-box" / `ldp:inbox`) scopes — separate leak
   analysis, explicitly NOT covered by this record.

## 8. Open questions for the maintainer

1. **Is L4 (`Wins, Wdel ⊆ R` — "you may only write where you can read") acceptable?** It is
   the cheapest fail-closed rule and it makes the whole design fall out; but it forbids the
   blind drop-box, which is a real Solid pattern (`ldp:inbox`). If the drop-box is required,
   the design changes materially and phase 7 becomes a blocker rather than a follow-up.
2. **Is §4's consequence acceptable?** A principal's own `DELETE` silently affects nothing in
   its masked region and reports success. This record argues it is the only answer
   consistent with the read-path contract; it is nevertheless surprising, and a maintainer
   who wants an error instead should say so — an error is a (small, bounded) L2 probe.
3. **Should the P0 fix require `Read` on WHERE graphs, or evaluate the WHERE over the
   authorized view?** The first denies more updates and is a visible behaviour change; the
   second silently changes results. This record leans to the first at graph granularity (a
   loud, auditable change) and the second at pattern granularity (§5, where masking already
   means "as if absent"), but the inconsistency is deliberate and worth a ruling.
4. **Does `odrl:delete` on a pattern asset mean `Wdel` only, as proposed in §5.2?** The
   existing `action_to_mode` collapses `modify`/`delete`/`write` all to `Mode::Write`, so
   the finer split proposed here is new vocabulary semantics, not just a refinement.

## 9. Uncertainties

* The blank-node replay question (§5.4) is genuinely unverified — this record does not claim
  `apply_effects` handles it either way.
* The `USING`/`WITH CHECK` prior art is cited from the PostgreSQL documentation, which was
  read directly. The Solid Protocol's own N3-Patch mode requirements were **not** verified
  against the specification for this record (no network access); the append-vs-write mode
  split described here is taken from this repository's own implementation
  (`crates/sparq-solid/src/update.rs` `Need::WriteOrAppend` / `Need::Write`), not quoted
  from the spec.
* No cost figure of any kind is asserted. §5.4's cost discussion is a shape argument only.
* Nothing here is implemented; §5.3's acceptance bar is the standard this design must be
  held to before any claim of soundness is made about it.
