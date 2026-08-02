# Pattern-scoped UPDATE enforcement — design record

Status: **design record only** (spike `sq-fznmq`, deferred from
[`odrl-pattern-scoped-targets-2026-07.md`](odrl-pattern-scoped-targets-2026-07.md) §2.4).
No implementation ships with this record. Companions:
[`odrl-pattern-scoped-targets-2026-07.md`](odrl-pattern-scoped-targets-2026-07.md) (the
READ-path masking design this mirrors — read it first),
[`solid-access-control-design.md`](solid-access-control-design.md) §4.4 (the graph-granular
write path being extended), `crates/sparq-solid/src/update.rs` (that path's code),
`crates/sparq-solid/src/pattern_scope.rs` (the read-path prototype, feature `pattern-scope`,
OFF by default).

<!-- [OPUS-5] 🤖 SPARQ agent. Design-first spike per the bead: options + fail-closed
semantics for INSERT/DELETE/WHERE over masked regions, decided BEFORE any implementation.
§1.3 records two channels I reproduced against `origin/main` while establishing the
substrate; they are captured as follow-up work, NOT fixed here. -->

> **Honest scope.** Clear-path authorisation only — no cryptographic guarantee is claimed
> anywhere in this record. Everything below is *designed*, nothing is implemented; the only
> code that exists today is the READ-path prototype named above. No performance numbers
> appear here; §6.5 states cost as complexity, to be measured if and when this is built.

## 0. What this record decides

The read-path record left one line of future work: *"a pattern-scoped WRITE grant (row-level
update authority) is a different problem (blind writes into masked regions,
delete-visibility interactions) and is explicitly deferred"*. This record answers that,
in four parts:

1. what is actually true of the write path today (§1) — including **two channels I
   reproduced**, which are prerequisites, not consequences, of pattern-scoping;
2. the leakage bar a pattern-scoped write must clear, and the law that clears it (§2–§3);
3. the options and the recommended decision (§4), with per-construct fail-closed
   semantics (§5) and a mechanism sketch (§6);
4. a phased plan of future beads (§9) and the questions that genuinely need the
   maintainer (§10).

**The recommendation in one line:** ship *nothing* first — a pattern-scoped principal gets
**no** write authority (option W0, the current fail-closed floor) — and build toward the
**visible-delta law** (option W1): *the verdict and the applied delta must be functions of
the actor's own masked view and the update text alone*. That law is blocked on read-scoping
the update's `WHERE` clause, which is a standalone security fix at the EXISTING graph
granularity (§1.3).

## 1. Ground truth today (verified against `origin/main`, 2026-08-02)

### 1.1 The read path

Masking ships as `crates/sparq-solid/src/pattern_scope.rs` behind the OFF-by-default
`pattern-scope` feature (`crates/sparq-solid/Cargo.toml`): `ScopePattern` (a per-triple
predicate, each component a concrete term or a wildcard), `GraphScope` (`allow`/`deny`, deny
overrides allow, empty allow grants nothing), `masked_graph`, `masked_dataset`, and
`PodStore::scoped_dataset` → `ScopedDataset::{view, query, query_json, ask}`. Masking is by
**materialization**: the scoped graph is decoded, filtered and rebuilt, so the engine
evaluates a dataset in which masked triples are physically absent.

The ODRL vocabulary designed in that record's §5 (`sparq:PatternAsset`, `auth:PatternGrant`)
is **not wired** — no occurrence of `PatternAsset`/`PatternGrant`/`allowPattern` exists in
`crates/`. Pattern scopes are therefore constructible only programmatically today. This
record does not depend on that wiring, but §9 sequences against it.

### 1.2 The write path

`PodStore::update_as` → `update_inner` (`crates/sparq-solid/src/lib.rs:1262`) is
authorize-then-apply: `update::check` computes a per-graph `(graph, need)` requirement set
and denies fail-closed before `sparq_engine::update_in_place(&mut self.graph, sparql)`
mutates anything. `Need` is `Write` (delete/clear) or `WriteOrAppend` (pure insert); a
`GRAPH ?var` template slot is resolved *precisely* by evaluating the operation's `WHERE`;
anything unresolvable escalates to "must be able to write every named graph". Granularity is
the whole named graph, end to end.

Two engine facts this design leans on, both confirmed by running them:

* the delta path is **set-valued and silent** — an insert of a triple already present and a
  delete of a triple that is absent both return `Ok(())` and leave the store unchanged
  (`Graph::insert` documents "re-inserting an existing triple is a no-op",
  `crates/sparq-core/src/lib.rs`; I ran both operations through
  `sparq_engine::update_in_place` and re-counted);
* `sparq_engine::update_in_place_capturing` returns a resolved `UpdateEffect` log
  (`Delta { slot, inserts, deletes }` per graph slot, plus `Clear`/`Drop`/`Create`), and
  `sparq_engine::apply_effects` replays that log onto another `Graph`
  (`crates/sparq-engine/src/update.rs`). Those two are the primitives §6 builds on — a
  capture/replay seam already exists for the durable-mirror use case.

### 1.3 Two channels I reproduced — the prerequisite, at the EXISTING granularity

While establishing the substrate I wrote two throwaway probes against the shipped WAC
fixture (`sparq_solid::wac_fixture`, in which BOB has no read grant on the `priv0` subtree
and read+write on the `team2` subtree). Both probes were deleted; neither is committed. Both
concern the graph-granular path as it stands, independently of pattern scopes.

**F1 — cross-graph copy-out through the `WHERE` clause.** `update_inner` applies the update
with `sparq_engine::update_in_place` over the **full store**, so the `WHERE` pattern reads
graphs the actor has no read grant on, and its bindings flow into triples written to a graph
the actor *can* read. Running, as BOB:

```sparql
INSERT { GRAPH <https://pod.ex/team2/c0/g0/d0.ttl> { <urn:x:copied> <urn:x:stolen> ?t } }
WHERE  { GRAPH <https://pod.ex/priv0/c0/g0/d0.ttl> { ?s <https://ex.dev/ns#title> ?t } }
```

returned `Ok(())`, after which `query_as(BOB, Read, …)` over the `team2` graph returned the
`priv0` title BOB cannot read directly (the same query against `priv0` returns zero rows for
him, before and after). This is arbitrary read-up: any triple in the pod can be copied into
any graph the actor may write, then read back.

**F2 — the deny verdict as an oracle.** `update::check`'s precise `GRAPH ?var` resolution
also evaluates the `WHERE` over the full store. Its own soundness note argues this is safe
because *"we never return rows to the actor — only an allow/deny verdict"*. That verdict is
itself a bit of information about unreadable data. Running two updates as BOB that differ
only in a guessed literal, where a match binds `?g` to a graph he can neither read nor write:

| guessed literal | outcome |
|---|---|
| the real `priv0` title | `Err("update denied: session lacks write permission on <https://pod.ex/priv0/c0/g0/d0.ttl>")` |
| a non-matching literal | `Ok(())` (no-op) |

So the verdict is a one-bit equality oracle over unreadable values, and the deny message
additionally discloses the IRI of a graph the actor cannot see.

**Why this belongs in a record about pattern scopes.** The write path's whole soundness
argument today is "the `WHERE` only reads, so it is not a write target". That argument is
false as soon as `WHERE` bindings reach data the actor can observe — which they always do
(F1) — and it is *doubly* false under masking, where "what the actor may read" is a
sub-graph predicate rather than a graph name. **Every option in §4 that permits a
pattern-scoped write requires the update's `WHERE` to be evaluated over the actor's
authorized view.** That change is worth making on its own, at today's granularity, and it is
the first bead in §9. Captured as follow-up issues, not fixed in this spike.

## 2. The leakage bar for writes

The read-path record's bar was *oracle equivalence*: `eval(masked(D), Q) = eval(D ∖ masked, Q)`.
Writes need a bar covering three distinct exposures.

| Channel | Attack shape | Where it bites |
|---|---|---|
| `WHERE`-clause read-up | bindings from masked triples are written into a visible region and read back | F1 above, at pattern granularity: `INSERT { ?s :copy ?phone } WHERE { ?s :phone ?phone }` |
| verdict / error oracle | permit-vs-deny, or an error message, depends on masked content | F2 above; also "your insert was refused because it collides" style channels |
| effect oracle | *whether anything changed* depends on masked content — observed through counts, version/ETag bumps, a subsequent read, or timing | a `DELETE … WHERE` whose match set includes masked triples |
| blind write | the actor writes into a region they cannot read | poisoning a masked region; the actor's own view then contradicts the store |
| masked-region overwrite | a delete removes triples the actor cannot see | silent destruction of exactly the data the policy protects |

Two properties from the read path must also survive: a fully-masked graph stays
indistinguishable from an absent one (read-path §2.3), and restriction composes but never
widens (read-path §2.2).

## 3. Semantics — the visible-delta law

Fix an actor `a`, a dataset `D`, and `a`'s read scope map. Write `vis_a` for the per-triple
visibility predicate (graph-level accessible set, refined per graph by `GraphScope::visible`)
and `M_a(D)` for the masked replica — exactly what `PodStore::scoped_dataset` builds today.

> **Visible-delta law.** An update `U` submitted by `a` is permitted only if the delta it
> produces **over `a`'s own masked view** falls entirely inside `a`'s visible, writable
> region; the store then applies **that** delta. Formally, with
> `δ = delta(M_a(D), U)` (the resolved insert/delete batch the engine produces when `U` runs
> against the replica):
>
> * **W-scope**: every triple in `δ⁺ ∪ δ⁻` satisfies `vis_a` and lies in a graph `a` may
>   write in the required mode — otherwise the whole request is **denied** (never silently
>   trimmed);
> * **apply**: the store becomes `D ⊎ δ` — the *validated delta* is replayed, the update
>   text is **not** re-evaluated against `D`.

Three consequences, and they are the whole point:

1. **Determinacy (no leak).** The verdict and `δ` are functions of `(M_a(D), U)` only. `a`
   supplies `U` and can already read `M_a(D)`, so nothing observable — outcome, error text,
   resulting data, or whether anything changed — carries information about `D ∖ vis_a`. This
   subsumes all three oracle rows of §2 in one argument, instead of a construct-by-construct
   audit. It is the write-side analogue of the read path's "sound by construction".
2. **Commutation.** `M_a(D ⊎ δ) = M_a(D) ⊎ δ` — the actor's view of the result is the result
   of applying `U` to their view. The proof is one line *because* a scope pattern is a
   per-triple predicate with no join variables (read-path §2.1): masking distributes over
   `⊎`, and `δ ⊆ vis_a` means `M_a(δ) = δ`. So the actor is never lied to: no insert
   vanishes, no delete leaves a ghost.
3. **Non-interference.** For every triple outside `vis_a`, `D` and `D ⊎ δ` agree — a
   pattern-scoped writer cannot change a triple they cannot see. That kills blind write and
   masked-region overwrite together.

**The v2 trap.** Consequence 2 depends on visibility being a per-triple predicate. If scope
patterns ever gain join variables (a scope like *"the phone numbers of people I introduced"*),
visibility becomes context-dependent, `M_a` no longer distributes over `⊎`, and this law
does not hold. Any such extension re-opens this record.

### 3.1 Read scope vs write scope

Postgres row-level security splits the two: the `USING` expression selects which existing
rows are visible (and hence updatable/deletable), the `WITH CHECK` expression constrains
rows produced by `INSERT`/`UPDATE`; a `USING` failure filters silently, a `WITH CHECK`
failure raises an error ([PostgreSQL `CREATE POLICY`](https://www.postgresql.org/docs/current/sql-createpolicy.html)).
That is a good vocabulary and a bad default here:

* the analogy is exact — read scope ≈ `USING`, write scope ≈ `WITH CHECK`;
* Postgres deliberately permits `WITH CHECK` ⊄ `USING` (write rows you cannot read), and its
  own documentation then has to warn that constraint machinery leaks: *"attempting to insert
  a duplicate value into a column that is a primary key or has a unique constraint. If the
  insert fails then the user can infer that the value already exists."* — a determinacy
  violation of exactly the kind consequence 1 forbids;
* RDF removes that particular hazard: datasets are sets, there are no unique or
  referential constraints, and sparq's delta path is silent on both re-insert and
  absent-delete (§1.2, measured). So sparq *can* be strictly better than the SQL baseline
  here — provided it does not reintroduce the hole by allowing write-up.

**Decision:** v1 requires `write_scope ⊆ read_scope`. A pattern-scoped write grant with no
corresponding read visibility is **denied at policy-materialization time**, not silently
narrowed. Whether to relax this is open question Q2.

## 4. Options

| | Option | Verdict |
|---|---|---|
| W0 | **No write authority under a pattern scope** — masking stays read-only; writes remain graph-granular, and a principal whose only grant on `g` is pattern-scoped may not write `g` at all | **Recommended for v1 (ship first)** |
| W1 | **Visible-delta (materialize → capture → validate → replay)** — §3's law, mechanised in §6 | **Recommended target; gated, opt-in** |
| W2 | Split scopes, Postgres-style (`USING`/`WITH CHECK` with write-up allowed) | Rejected for v1 |
| W3 | In-line per-triple write filter inside the engine's apply | Rejected |
| W4 | Rewrite the update text with guards | Rejected |

**W0** is the honest floor and is *already* the semantics — nothing in `update.rs` consults a
scope, so a scoped grant confers no write today. Making that explicit (and refusing to
materialize a pattern-scoped grant into a write mode) costs nothing and is the fail-closed
default the estate's enforcement law demands. Its cost is expressiveness only: *"let the
clinic update the trial results but never touch participant identifiers"* stays unexpressible
until W1 lands.

**W1** is the mirror of the read path's chosen option D, and inherits its two virtues: the
soundness argument is an identity rather than an audit obligation (§3, consequence 1), and it
needs no new enforcement machinery in the engine's scan paths. Its costs are real and stated
in §6.5: an O(read-accessible dataset) replica build on the write path, plus the delta-replay
seam and its two hazards (structural effects, blank nodes).

**W2** is rejected because it breaks commutation: an insert the actor cannot see afterwards
either vanishes from their view (their client's model diverges from the store) or must be
reported through a channel that itself leaks. It also hands a scoped principal a
write-only pipe into the exact region the policy is protecting. Postgres accepts this
trade-off because SQL applications routinely need append-into-a-hidden-audit-table; no Solid
use case in the maintainer's invariant needs it. Revisit under Q2 if one appears.

**W3** is rejected for the same reason the read path rejected its option A, one level worse:
the read path had roughly fifteen scan entry points to defend and count/estimate fast paths
that iterate no rows at all; the write path additionally has template instantiation,
per-slot delta application, WAL/durability paths and the capture sink. Missing one is a
silent leak or a silent unauthorized write, not a test failure.

**W4** is rejected because a guard cannot constrain a `WHERE` clause — the very construct F1
shows to be the leak — and because the estate's rewrite discipline ("the rewrite stays
trivial because policy ran at materialization time") is load-bearing in
`solid-access-control-design.md`.

## 5. Per-construct fail-closed semantics (under W1)

Throughout: **deny the whole request** rather than trim it. Silent trimming would make the
outcome depend on masked content, violating determinacy, and SPARQL's own silent-drop rules
(an uninstantiable template quad simply produces no write) are already load-bearing
elsewhere — overloading them with a security meaning would be unreadable.

| Construct | Semantics under a pattern scope on the target graph |
|---|---|
| `INSERT DATA` | each quad must satisfy `vis_a` for its graph's scope and the graph must be writable in `WriteOrAppend`; otherwise deny. A quad already present in `D` but masked for `a` cannot arise — a triple cannot be both visible (to be insertable) and masked |
| `DELETE DATA` | only quads present in `M_a(D)` are removed. A quad the actor names that is masked is **not** deleted and the request is **not** an error — indistinguishable from naming a quad that does not exist, which §1.2 confirms is already a silent no-op. This is the delete-visibility answer: **a pattern-scoped delete cannot reach a masked triple** |
| `DELETE WHERE` | equivalent to `DELETE { P } WHERE { P }` over `M_a(D)`: matches, and therefore deletes, only visible triples. A `DELETE WHERE { GRAPH <g> { ?s ?p ?o } }` leaves every masked triple of `g` in place |
| `DELETE … INSERT … WHERE` | `WHERE` is evaluated over `M_a(D)` (this is the F1 fix); delete quads are constrained as `DELETE DATA`, insert quads as `INSERT DATA`; a template that instantiates to a triple outside `vis_a` denies the request |
| `GRAPH ?var` template slot | resolved against `M_a(D)`, not the full store — the precise resolution of `update.rs` moves onto the replica, which also closes F2. A binding to a graph absent from the replica cannot occur; the existing blank-node-graph-name bail stays |
| `USING` / `WITH` | re-scope the replica exactly as `rescope_dataset` re-scopes the store today; the re-scoped set is intersected with the replica's graphs (restriction, never widening) |
| `LOAD` | the fetched document's triples are validated exactly as `INSERT DATA` quads; any triple outside `vis_a` denies. The captured delta (not a re-fetch) is what reaches the store, so replica and store cannot diverge on a non-deterministic remote |
| `CLEAR <g>` | **lowered** to "delete every visible triple of `g`" — a per-triple delta, never the structural effect. Replaying the structural `Clear` onto the store would destroy masked triples (a non-interference violation) |
| `DROP <g>` | **lowered** identically. This is sound *and* indistinguishable: by the read path's omit-empty rule (§2.3 there), a graph with no visible triples is already absent from `a`'s view, so "graph removed" and "all visible triples removed" are the same observation for `a` |
| `CREATE GRAPH <g>` | outside a scope's domain (a scope refines an existing graph). Requires **unscoped** graph-level write on `g`; denied for a purely pattern-scoped principal |
| `CLEAR`/`DROP` `ALL` / `NAMED` | the existing conservative rule stands (write on every graph), now additionally requiring that no graph in scope carries a mask — i.e. denied for any pattern-scoped principal |
| default graph | denied, unchanged (pod data never lives there) |
| `<urn:sparq:auth>` | never writable, unchanged; the replica must keep the reserved view out of the actor's visible set exactly as `scoped_dataset` does |

## 6. Mechanism sketch

### 6.1 Shape

```text
update_scoped_as(session, scopes, sparql):
  1. replica  ← scoped_dataset(session, Mode::Read, scopes)      # M_a(D), existing code
  2. effects  ← update_in_place_capturing(replica_graph, sparql) # δ, over the replica ONLY
  3. lower    ← structural effects (Clear/Drop) → explicit per-triple deltas over the replica
  4. validate ← every quad of δ: vis_a(quad) ∧ graph writable in the required mode
                (reuse update::check's Need/allowed logic, per quad instead of per graph)
  5. deny     ← on any violation: return Err, store untouched
  6. apply    ← apply_effects(store_graph, δ)   # replay the VALIDATED delta, not the text
  7. remat    ← the existing .acl/.acr/group-document re-materialization rule
```

Steps 1, 2 and 6 are existing public surfaces (`PodStore::scoped_dataset`,
`sparq_engine::update_in_place_capturing`, `sparq_engine::apply_effects`). Steps 3–5 are new
and live entirely in `sparq-solid` behind the `pattern-scope` feature — **no engine change**,
matching the read path's zero-engine-change commitment.

### 6.2 Why replay, not re-execute

Re-running the update text against the store after validating against the replica would
re-evaluate the `WHERE` over unmasked data: a different δ, computed from data the actor
cannot see. That is both a time-of-check/time-of-use gap and a direct reintroduction of F1.
Replaying the captured delta is what makes determinacy (§3, consequence 1) mechanically true
rather than argued.

### 6.3 Hazard — structural effects must be lowered

`UpdateEffect::{Clear, Drop, Create}` are recorded as *operations*, not deltas, precisely
because they are pure functions of the text — which is what makes them wrong to replay here:
`Clear(<g>)` over the replica clears only visible triples, but replayed against the store it
clears everything. Step 3 must convert them into explicit per-triple deletes computed over
the replica, or refuse the operation. Getting this wrong is a silent destruction bug, so the
acceptance test must include a masked graph subjected to `CLEAR` and `DROP` with an assertion
that the masked triples survive in the store.

### 6.4 Hazard — blank-node identity across the replica boundary

`masked_graph` rebuilds with a fresh dictionary, interning the same `Term`s, so an existing
blank node keeps its label and a delta mentioning it maps back onto the store. But a template
that mints **fresh** blank nodes mints them in the replica's label space; replaying such a
delta into the store could collide with an existing store blank node of the same label and
silently merge two distinct nodes. Mitigation options: skolemize minted blank nodes before
replay, or verify freshness against the store's dictionary and reject on collision. Must be
decided before implementation and covered by a test.

### 6.5 Cost, stated as complexity

A replica build is O(read-accessible dataset) per update — the read path pays this once per
(session × scope) and amortizes it across queries; the write path would pay it per request
unless it shares the replica cache designed in the read record's §6 (bead `sq-nc3c6`). Two
mitigations to evaluate, both fail-closed:

* build the replica only when the session actually carries a scope entry for a graph the
  update touches — otherwise fall through to today's graph-granular path unchanged;
* restrict the replica to the graphs the update statically names, falling back to the full
  read-accessible set whenever a `GRAPH ?var`, `USING`/`WITH` or unresolvable pattern makes
  the touched set unknown.

No numbers are asserted; measurement belongs with the implementation bead, on the
benchmark discipline's terms.

### 6.6 Atomicity and concurrency

`update_as` takes `&mut self`, so replica-build and apply are already exclusive within a
process; step 5's deny leaves the store untouched exactly as `check` does today. If the
replay is ever split across a durable mirror, it must go through the request-atomic path
(`update_in_place_atomic`) so a mid-request denial cannot leave a prefix applied.

## 7. Worked example

Policy: *"the clinic may correct trial results, but participant identifiers are none of its
business."* Read scope on `<https://pod.ex/trial>`: `deny_within([(None, ex:participantId, None)])`.

```sparql
# (a) permitted — every delta quad is visible
DELETE { GRAPH <https://pod.ex/trial> { ?s <https://ex.dev/ns#outcome> ?old } }
INSERT { GRAPH <https://pod.ex/trial> { ?s <https://ex.dev/ns#outcome> "revised" } }
WHERE  { GRAPH <https://pod.ex/trial> { ?s <https://ex.dev/ns#outcome> ?old } }

# (b) denied — the insert template instantiates to a masked triple
INSERT DATA { GRAPH <https://pod.ex/trial> { <urn:p1> <https://ex.dev/ns#participantId> "X" } }

# (c) permitted, and a no-op — the WHERE matches nothing in the replica, so no identifier
#     is copied out and the verdict reveals nothing about whether any exists
INSERT { GRAPH <https://pod.ex/notes> { <urn:x> <urn:copied> ?id } }
WHERE  { GRAPH <https://pod.ex/trial> { ?s <https://ex.dev/ns#participantId> ?id } }

# (d) permitted, and deliberately partial — the outcome triples go, the identifiers stay
DELETE WHERE { GRAPH <https://pod.ex/trial> { ?s ?p ?o } }
```

Case (c) is F1 under masking, closed by evaluating the `WHERE` over the replica. Case (d) is
the delete-visibility answer: the clinic's "delete everything" removes only what it could
see, and — because a fully-masked graph is omitted from its view — it cannot tell the
difference between that and having emptied the graph.

## 8. Residual channels and non-goals

* **Out-of-band metadata.** Version counters, ETags, `Last-Modified`, materialization epoch
  bumps and reindex timing are *not* covered by the law: they are properties of `D`, not of
  `M_a(D)`. If a scoped actor can observe a resource version that changes when a masked
  triple changes, that is a channel. Mitigation is to derive any actor-visible version from
  the actor's visible sub-graph, or to keep it coarse. Design that with the HTTP surface, not
  here.
* **Timing.** Replica build time is O(accessible dataset) and therefore leaks the size of the
  region, not its content. Same posture as the read path.
* **Concurrent writers.** Another principal's write to a masked region is invisible to the
  scoped actor by construction, but changes the store between two of the actor's requests.
  No lost-update protection is claimed; that is the existing write path's story.
* **Not a confidentiality guarantee against a compromised process.** Everything here is
  in-process, clear-path enforcement.

## 9. Phased plan (future beads — ids to be assigned by `bd create`)

1. **`fix(solid)`: read-scope the update `WHERE` at graph granularity.** Evaluate the apply's
   `WHERE` and the `GRAPH ?var` resolution over the actor's authorized read view rather than
   the full store; assert F1 and F2 closed with regression tests. **Security fix, independent
   of pattern scopes, and the hard blocker for everything below.** Note the behaviour change:
   a principal holding `acl:Write`/`acl:Append` without `acl:Read` gets an empty `WHERE`, so
   `DELETE/INSERT … WHERE` becomes a no-op for them while `INSERT DATA` still works — that is
   correct under WAC (Write does not imply Read) but it is a change (Q3).
2. **`feat(engine or solid)`: the delta-replay seam.** Structural-effect lowering (§6.3) and
   the blank-node freshness rule (§6.4), with tests that a `CLEAR`/`DROP` over a masked
   replica cannot destroy masked triples in the store.
3. **`feat(solid)`: `WriteScope` + the visible-delta validator** — `PodStore::update_scoped_as`
   behind `pattern-scope`, implementing §5's table and §6.1's pipeline. Fail-closed unit
   tests per row of that table.
4. **`test(solid)`: the differential oracle.** For a battery of updates × scopes assert
   `M_a(apply(D, U)) == apply(M_a(D), U)` (commutation), `D ∖ vis_a` unchanged
   (non-interference), and verdict-equality between a store where the masked triples are
   physically deleted and one where they are merely masked (determinacy). Plus a non-vacuity
   mutation: a no-op mask must flip the test red — mirroring `tests/pattern_scope.rs`.
5. **`test(solid)`: fuzz** random scopes × random updates against the oracle, mirroring
   `tests/pattern_scope_fuzz.rs`.
6. **`feat(solid)`: ODRL write vocabulary** — pattern-scoped `auth:write`/`auth:append`
   grants, with the `write_scope ⊆ read_scope` check enforced at materialization time.
   Depends on the read-path bridge wiring (`sq-qnlj8`) landing first.
7. **`perf(solid)`: replica reuse on the write path** — share the read path's scoped-replica
   cache (`sq-nc3c6`) and implement §6.5's two narrowing mitigations; measure then.

Beads 2–7 are gated on bead 1. Bead 1 is worth doing whether or not pattern-scoped writes
are ever built.

## 10. Open questions for the maintainer

1. **Ship W0 explicitly?** Should a pattern-scoped grant be *refused* at materialization time
   when it names a write mode (loud), or silently confer no write authority (quiet, and what
   happens today)? I recommend loud — a policy author who writes a pattern-scoped write
   permission should learn it is unsupported, not discover it by observing no effect.
2. **Is write-up ever wanted?** §3.1 forbids `write_scope ⊄ read_scope`. Is there a real Solid
   use case (append-to-an-unreadable-audit-graph) that needs it? If yes, it needs its own
   record — the determinacy argument does not survive it.
3. **Is bead 1 a fix or a breaking change?** Read-scoping the `WHERE` will make some updates
   that succeed today become no-ops (a Write-without-Read principal). I read it as a security
   fix that should land regardless, but it touches `sparq-server`'s update endpoint behaviour
   and deserves an explicit call.
4. **`CLEAR`/`DROP` under a scope — lower or deny?** §5 lowers them to per-triple deletes on
   the argument that the result is indistinguishable to the actor. Denying is simpler and
   even more conservative. Which reads better to a policy author?
5. **Do pattern-scoped writes earn their cost at all?** W1 is a genuine amount of machinery
   for one line of the maintainer invariant. If the near-term need is only *"share X except
   Y"* (read), W0 plus bead 1 may be the whole answer for a long time.
