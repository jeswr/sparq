# Pod-scoped (incremental) materialization of the Solid auth view

Status: **design-for-review** (research record). Nothing described below is implemented;
`crates/sparq-solid` still re-derives the whole `<urn:sparq:auth>` view on every ACL write.
This record is the architect-level deliverable of bead `sq-nmx4l` (issue
[#2852](https://github.com/jeswr/sparq/issues/2852), ask 1 of issue #1571), the follow-up to
`sq-b7k7u` (PR #1585). It specifies the confinement analysis, the scoped-input assembly, the
splice algorithm, the mandated property test, and the disjoint implementation children — and
it records the six hazards found while doing the analysis (§4) that make the *naive*
pod-scoped splice **incorrect**, so an implementer does not build it.

Companion records: [`solid-access-control-design.md`](solid-access-control-design.md) (the
shipped architecture; §2.4 security boundary, §3.4–3.5 rule sets and stratification) and
[`solid-acp-differential-oracle-design.md`](solid-acp-differential-oracle-design.md).

Code read for this record, at `ccae3aec`: `crates/sparq-solid/src/{loader,materialize,
authindex,write_through,session_cache}.rs` and `crates/sparq-solid/rules/{common,wac,
acp-a,acp-b,acp-c}.n3`.

---

## 1. The problem, and what `sq-b7k7u` already fixed

`sq-b7k7u` split the **read/decide** half by origin: `AuthIndex` buckets allow/deny/conditional
grants by the target graph's origin (`authindex.rs`), `SessionCache::invalidate_origin` does
per-origin surgery on each cached session, and `PodStore::reindex_with(ReindexScope::Origin)`
diffs the old and new `AuthIndex` per origin and invalidates exactly the origins whose buckets
changed. That invalidation is sound *without* any confinement argument, because it compares
the two indexes directly — a cross-origin effect shows up as a bucket difference.

The **write** half is untouched. `put_acl_inner` / `delete_acl_inner`
(`write_through.rs`) swap one control graph and then call `rematerialize_scoped`, which calls
`materialize_wac` / `materialize_acp_with`. Those:

1. `assemble_input_ids` over **every** named graph in the store — resource facts for every
   content graph plus its whole container chain, every `.acl`/`.acr` graph's triples, every
   referenced group document;
2. run the compiled N3 evaluator to fixpoint over that whole fact set (three chained strata
   for ACP);
3. rebuild `<urn:sparq:auth>` from scratch (`install_auth_view`);
4. and `reindex_with` then calls `AuthIndex::from_graph` over the whole rebuilt view.

So a single-`.acl` write is O(all ACLs + all resources) in the reasoner and O(whole view) in
the index. That is the ACL-write-churn hard case from `solid-server-rs`.

**Ask.** Re-derive only the affected slice of the view and splice it into the retained
slices, and patch only the affected `AuthIndex` buckets.

---

## 2. Setup and notation

Let `S` be the store's named-graph set. Write `org(x)` for `loader::iri_origin(x)`
(`scheme://authority`, the whole IRI when there is no `://`). Partition the non-reserved
graphs of `S`:

- `Ctl(S)` — control graphs, IRI ending `.acl` (WAC) or `.acr` (ACP);
- `Res(S)` — every other named graph, plus every structural container prefix reachable by
  `loader::parent_iri` (containers exist as inheritance anchors without their own graph);
- `Grp(S) ⊆ Res(S)` — the group documents named by some `acl:agentGroup` object with its
  fragment stripped (WAC only).

`parent_iri` never crosses the authority, so **the container chain of a resource at origin `O`
lies entirely in `O`** (`loader.rs`, and `loader::tests::parent_walks_to_root_and_stops`).
That is the one structural fact the whole analysis leans on.

Write `V` for the installed `<urn:sparq:auth>` triple set and `V|O` for its origin-`O`
slice — defined precisely in §5.1, because the obvious "triples whose object is at `O`" is
*not* a partition of `V` (conditional-grant nodes and matcher accept-set facts do not have a
resource object at all).

---

## 3. Output confinement

**Theorem (grant *anchoring* confinement).** Every simple grant/deny triple
`⟨p, auth:MODE, r⟩ ∈ V` is **anchored** by a control document at `org(r)`: the
`solidx:appliesTo` / `solidx:appliesToResource` premise that every grant rule carries can be
satisfied only through `?r`'s own or inherited `.acl`/`.acr`, and those lie at `org(r)`.

Read the scope of that claim precisely. It says the *anchor* is at `org(r)`, hence that the
grant's object — and so its slice attribution (§5.1) — is at `org(r)`. It does **not** say the
grant is derived *only* from documents at `org(r)`: the remaining premises of the same rule
bodies are matched against the merged fact set and may be satisfied by triples stored anywhere
in `Ctl(S)` (§4.1).

*Proof sketch, WAC (`rules/wac.n3`).* Every grant rule requires `?auth solidx:appliesTo ?r`,
which is derivable in only two ways:

- own ACL: `?r solidx:ownAcl ?acl . ?auth solidx:inDoc ?acl . ?auth acl:accessTo ?r`. The
  loader emits `ownAcl` *only* by naming convention — `<R>` is the target IRI with the
  `.acl` suffix stripped (`loader.rs`, step 2) — so `org(acl) = org(r)`; and `inDoc` pins
  `?auth` to that document.
- inherited ACL: `?r solidx:inheritedAcl ?acl` walks `solidx:parent`, which is within-origin,
  to an ancestor with its own ACL; `?auth solidx:inDoc ?acl` again pins the document, and
  `?auth acl:default ?c` pins it to the ancestor it belongs to.

The Control rule (`?p auth:control ?r . ?r solidx:ownAcl ?acl ⇒ ?p auth:read/write ?acl`)
emits an object `?acl` with `org(acl) = org(r)`. Origin-restricted grants mint a pair
*principal*; the object is unchanged. ∎

*Proof sketch, ACP (`rules/acp-a.n3`, `acp-c.n3`).* Every grant requires
`?pol solidx:appliesToResource ?r`, derivable only from `?r solidx:ownAcr ?acr` (naming
convention, same origin) or `?r solidx:ancestor ?anc . ?anc solidx:ownAcr ?acr`
(within-origin ancestry). The resource-scoped provenance grants use
`?k solidx:provForResource ?r`, minted in `acp-a.n3` only under
`?pol solidx:appliesToResource ?r` — same premise. ∎

This is the same confinement `sq-b7k7u` relied on for the *output* graph object, restated for
the whole grant class. **It is not sufficient**, for two reasons developed next: the
*derivation* of an `O`-slice may read documents outside `O` (§4), and two classes of view
triple are not grant triples at all (§4.3).

---

## 4. What breaks the naive splice

### 4.1 Cross-document indirection: the reasoner input is one merged fact set

`assemble_facts` (`loader.rs`, step 3) walks **every** control graph and pushes its triples —
skolemized, and minus the `solidx:`-predicate triples the derivation-vocabulary guard drops —
into a single flat fact set. The only provenance it retains is a separate
`⟨s, solidx:inDoc, D⟩` fact per subject `s` of document `D` — added *alongside* the triples,
never used to filter them. So for any subject `?s`, what the reasoner sees about `?s` is the
**union of the triples about `?s` over every control graph in `Ctl(S)` that mentions it**. A
control document's IRI names *where a triple is stored*; it never constrains *which subjects
that document may describe*, because RDF subjects are not confined to their IRI-named
document. Two distinct hazards follow.

**(a) Reference indirection — the referenced IRI is at another origin.**
`acp-a.n3` derives effective policies as `?acr acp:accessControl ?c . ?c acp:apply ?pol`. The
IRI `?pol` is not constrained to `?acr`'s document:

```turtle
<https://p.ex/.acr>     acp:accessControl <https://p.ex/.acr#c> .
<https://p.ex/.acr#c>   acp:apply         <https://o.ex/.acr#pol> .   # ← in P's document
<https://o.ex/.acr#pol> acp:allow         acl:Read .                  # ← in O's document
```

is a grant at origin `P` whose *body* lives at origin `O`. Consequences:

- **Scoped input for `O` must include the transitive closure of referenced control
  documents**, not just `O`'s own.
- **A `put_acl` to `O` can change `P`'s slice.** A write-side reverse dependency index is
  mandatory; re-deriving only `org(written .acr)` is unsound (fail-open: deleting
  `<https://o.ex/.acr#pol>`'s `acp:deny` would leave `P`'s stale deny in the retained slice —
  here it fails *closed*, but the allow-side dual fails open).
- Policy bodies can only live in `.acr` documents (content graphs are never fed to the
  reasoner — design record §2.4), so the closure is finite and bounded by `Ctl(S)`.

**(b) Split subjects — the containing document is not computable from the subject IRI.**
Hazard (a) is *not* discharged by resolving `<https://o.ex/.acr#pol>` to the graph
`<https://o.ex/.acr>`, which is the obvious walk and the wrong one. Nothing requires that
subject's triples to be stored there; a third, differently named control document supplies
them just as effectively:

```turtle
# graph <https://p.ex/.acr>
<https://p.ex/.acr>        acp:accessControl <https://p.ex/.acr#c> .
<https://p.ex/.acr#c>      acp:apply         <https://o.ex/.acr#pol> .
# graph <https://o.ex/.acr>  — the "obvious" home of #pol, and it may say nothing about it
# graph <https://q.ex/other.acr>  — a THIRD document whose name resembles neither
<https://o.ex/.acr#pol>    acp:allow  acl:Read .
<https://o.ex/.acr#pol>    acp:allOf  <https://q.ex/other.acr#m> .
<https://q.ex/other.acr#m> acp:agent  <https://alice.ex/#me> .
```

The full materializer merges `Q` and derives the allow. A dependency walk that fragment-strips
the object IRI and pulls the like-named graph reaches `O` and never reaches `Q`, so `Q` is
outside `P`'s scoped closure: a later write to `Q` re-derives only `org(Q)`, leaving `P`'s
retained slice stale — **including a stale allow** when `Q`'s `acp:allow` or matcher facts are
narrowed or deleted. That is the fail-open direction, and it is not covered by a regression
that stores the body in `O`'s correspondingly named graph.

The fix is to stop inferring provenance from names: define dependencies from **actual triple
occurrence** (§5.2), or enforce a document-locality invariant at write admission so that the
name really does determine the document (§5.2, and it is a behaviour change, not an
assumption the scoped path may help itself to).

**WAC narrows this but does not escape it.** `?auth solidx:inDoc ?acl` is a premise of both
`appliesTo` rules, so an Authorization can only *anchor* to a resource through the own or
inherited ACL it is declared in — a foreign document cannot introduce a wholly new
Authorization for `?r`. But `inDoc` pins the anchor only: the rest of the same rule bodies —
`?auth a acl:Authorization`, `?auth acl:accessTo ?r`, `?auth acl:default ?c`, `?auth acl:mode`,
and the `acl:agent`/`acl:agentGroup`/`acl:origin` triples behind `solidx:grantsAgent` — are
matched against the merged fact set like any others. So once `?auth` appears in `O`'s `.acl`
at all, a `.acl` at `Q` carrying `⟨?auth, acl:agent, mallory⟩` widens `O`'s grant. **WAC's
`rdeps` is therefore empty only under the document-locality invariant, not provably.** WAC is
still the easier half — no reference indirection, so its dependency set is exactly "the
control graphs mentioning subjects already declared in `O`'s own/inherited ACLs" — and should
still ship first, but on the same occurrence index, not on the naming convention.

**(c) Open question this record does not resolve.** The same merged-fact-set property means a
control document at `Q` can also carry triples whose subject is *another origin's* `.acr` —
e.g. `⟨<https://p.ex/.acr>, acp:accessControl, <https://q.ex/other.acr#c>⟩` — and the shipped
full materializer will honour it. Whether that is intended (one dataset, one trust domain) or
a cross-tenant injection surface is a question about the **existing** materializer, not about
scoping, and it is out of scope here; the document-locality invariant below would close it as
a side effect. Recorded so it is not mistaken for something the scoped path introduces.

### 4.2 The global principal lattice (`isCandAgent` / `isCandClient` / `isCandIssuer`)

`acp-a.n3` closes accept-sets downward over candidates:

```n3
{ ?m solidx:acceptsAgentP auth:Authenticated . ?a solidx:isCandAgent true }
=> { ?m solidx:acceptsAgentP ?a } .
```

and `?a solidx:isCandAgent true` is derived from **any** policy's `candAgent` anywhere in the
store. So a WebID introduced by origin `P`'s ACR enlarges the accept-set of origin `O`'s
matchers. Two questions follow, with different answers.

**(a) Does it change `O`'s grants?** No. `?pol solidx:candTriple ?k` is minted only from
`?pol solidx:candAgent/candClient/candIssuer`, which come from `?pol`'s own matchers plus the
three dimension tops. So the candidate triples of `O`'s policies are fixed by `O`'s reachable
documents. Stratum B's rejections (`acp-b.n3`) negate `?m solidx:acceptsAgentP ?pa` only for
`?pa` ranging over those candidates; and any such `?pa` that is a concrete WebID is *already*
`isCandAgent` via `O`'s own policy (`{ ?pol candAgent ?a . ?a isWebId true } => { ?a isCandAgent true }`),
while the non-concrete ones are the tops, handled by the top rules. **Foreign candidates
therefore only ever add `acceptsAgentP` facts for agents that are not `O`'s candidates, and
cannot flip any stratum-B outcome for `O`.** The same argument transfers verbatim to the
client and issuer dimensions.

**(b) Does it change the view?** **Yes** — and this is the hazard. `install_auth_view`
copies `MATCHER_FACTS` (`solidx:acceptsAgentP` / `acceptsClientP` / `acceptsIssuerP`) into
`<urn:sparq:auth>` for every matcher referenced by an `auth:exceptMatcher`
(`materialize.rs`). So `O`'s `noneOf` matcher carries accept-set triples whose presence
depends on `P`'s inputs. A pod-scoped run for `O` that omits `P`'s candidates produces a
**smaller** accept-set — i.e. `scoped ≠ full` at the triple level.

Is the difference semantically inert? At the session layer, yes, *as the code stands*:
`AuthIndex::matcher_accepts` short-circuits on `set.contains(PUBLIC)` /
`set.contains(AUTHENTICATED)` (and `ANY_CLIENT` / `ANY_ISSUER`), and every widened fact is
derived *downstream of* one of those tops already being in the set. So the extra concrete
entries are redundant for the decision. But that inertness is an accident of the current
`matcher_accepts` implementation, it is a *fail-open* difference if it ever stops holding
(a missing accept-set entry means the `noneOf` exception does not fire, so the conditional
grant is **not** suppressed), and it defeats the cheap exact oracle that makes this whole
bead reviewable.

**Design decision.** Do not rely on the inertness. Maintain the three candidate sets, plus
the `solidx:isWebId` set, as an explicit **global principal-lattice summary** (§5.2), seed
every scoped run with it, and **fall back to a full re-materialization whenever a write
changes the summary**. A `.acl`/`.acr` write changes it only by introducing a WebID / client
/ issuer that was not previously mentioned anywhere, or by removing its last occurrence —
rare on the churn workload this bead targets, and the fallback keeps the exact-equality
property (§7) as the acceptance oracle rather than a weakened decision-equivalence one.

### 4.3 The view is not partitioned by object origin

Two classes of `V` triple have no resource object:

- **Conditional-grant nodes.** `?g a auth:ConditionalGrant`, `?g auth:effect/agent/client/
  issuer/mode/graph/exceptMatcher …`. The node IRI is `urn:sparq:grant?cand=…&mode=…&graph=…`
  with percent-encoded components, so `iri_origin` of the *subject* is the whole IRI — not the
  target origin. `AuthIndex::from_graph` already handles this correctly by bucketing on the
  `auth:graph` object (`authindex.rs`), and the slice definition must do the same: attribute
  **all** triples of a grant node to `org(auth:graph object)`.
- **Matcher accept-set facts.** Subject is a matcher IRI, which under §4.1 may live at a
  *different* origin than the grant that references it. Attribution must therefore be by the
  referencing grant, not by the matcher IRI's origin — and a matcher shared by two origins'
  grants appears in **both** slices. Slices overlap; the splice is a union, not a disjoint
  concatenation (§5.3).

### 4.4 The ODRL bridge writes into the same view

With the `odrl-bridge` feature on, `<urn:sparq:auth>` also carries bridged grants
(`odrl_bridge`), and every static re-materialization calls
`PodStore::reconcile_bridged_after_static` to re-capture the static view as the ledger's
baseline and replay still-valid bridged grants on top. Bridged grants are **not** derived
from any control document, so they belong to no origin slice under §5.1 and would be dropped
by a naive splice.

**Design decision.** The splice produces the *static* view only; `reconcile_bridged_after_static`
runs afterwards exactly as it does on the full path, so the baseline it captures is the
spliced view and the replay is unchanged. The `#[cfg(not(feature = "odrl-bridge"))]` stub keeps
the lean default build unaffected. The §7 property test must run in **both** feature states,
with at least one generated sequence interleaving a bridged grant with ACL writes — otherwise
the default-features suite proves nothing about the feature-on path.

### 4.5 Positional skolemization is not stable across writes

`loader::skolemize` mints `urn:skolem:g{gix}:{blank}` where `gix` is the **index of the graph
in `graph.named`**. `write_through::take_named_slot` uses `swap_remove`, so an ACL write
permutes those indices. Under whole-store re-materialization this is invisible (every skolem
is re-minted in the same run). Under splicing it is fatal: retained slices carry skolem IRIs
minted against the *old* indexing, and a re-derived slice mints different ones for the same
blank nodes.

Skolem IRIs do reach the view, by three routes, so this is not hypothetical:

- WAC `{ ?auth a acl:Authorization . ?auth acl:agent ?a } ⇒ { ?auth solidx:grantsAgent ?a }`
  has no WebID guard, so `acl:agent _:b` produces `⟨skolem, auth:read, r⟩` — a skolem in
  *principal* position.
- ACP `⟨g, auth:exceptMatcher, ?nm⟩` copies the matcher term verbatim, and a matcher written
  as a blank node (`acp:allOf [ acp:agent … ]`) is a skolem there and in the subject of its
  `MATCHER_FACTS` accept-set triples.
- The ACP conditional-grant node IRI is minted by `string:concatenation` over `?k log:uri ?ks`,
  and `?k` embeds `?pol` — a blank-node policy makes the *grant node's own IRI* contain a
  skolem.

(`acp:agent _:b` does **not** reach the view: `collect_agents` records only `NamedNode`
objects as WebIDs, so no `solidx:isWebId` fact is emitted and `agentValP` never fires.)

**Prerequisite.** Replace the positional `gix` with a **deterministic per-document key**:
percent-encode the control document's IRI (the `.acl`/`.acr` graph name — group documents key
on their own graph IRI), e.g. `urn:skolem:<encoded-doc-iri>:<blank-label>`. This is
position-independent, origin-local, and idempotent, and it is a strict improvement
independent of scoping. It must land **before** any splicing work.

### 4.6 Principal validation becomes origin-local

`assemble_facts` runs `validate_principal_iri` over the principals of **every** control and
group document and fails the whole materialization on a reserved-encoding collision. A scoped
run only validates its own input closure, so a store already containing a poisoned ACL at
another origin would make `full` error and `scoped` succeed — a legitimate divergence that
would falsify the §7 property on an adversarially-constructed store.

**Design decision.** Validation is a *write-admission* check, not a materialization check: a
poisoned document can only enter through `put_acl` (which re-validates its own closure) or
through `PodStore::new` (which materializes fully). State the §7 property over stores whose
every control document passes validation, and add a separate regression test asserting
`put_acl` of a poisoned document still errors and rolls back.

---

## 5. The design

### 5.1 Slice definition

For an origin `O`, define `V|O ⊆ V` as:

1. every `⟨p, auth:{read,write,append,control,denyRead,denyWrite,denyAppend,denyControl}, r⟩`
   with `org(r) = O`;
2. every triple of a conditional-grant node `g` with `⟨g, auth:graph, r⟩` and `org(r) = O`
   (including `g`'s `rdf:type`, `auth:effect/agent/client/issuer/mode/graph/exceptMatcher`,
   and — for completeness of the node's triple set — any `auth:notBefore`/`auth:notAfter`
   window, though today only the ODRL bridge emits those and bridged grants are outside the
   splice entirely, §4.4);
3. every `⟨m, solidx:accepts{Agent,Client,Issuer}P, x⟩` for each matcher `m` appearing as
   `⟨g, auth:exceptMatcher, m⟩` for some `g` in class 2.

By §3, classes 1–2 partition `V`'s grant triples exactly. Class 3 **overlaps** across origins.
`V = ⋃_O V|O`, and for any `O ≠ O'`, `V|O ∩ V|O'` contains only class-3 triples.

The overlap is well-defined *only if* class-3 triples are origin-independent — which is
exactly what §4.2's pinned lattice summary buys. **This is the load-bearing invariant of the
splice, and the property test in §7 must be able to red on it.**

### 5.2 Store-level derived state

Add to `PodStore` (all reconstructible from the graph; never authoritative):

| State | Content | Invalidated by |
| --- | --- | --- |
| `resource_facts` | the `solidx:isResource` fact set + the `solidx:parent`/`ancestor` closure `common.n3` derives from it | a content-graph add/remove — **never** by `put_acl`/`delete_acl` (§5.5) |
| `lattice` | the `isWebId`, `isCandAgent`, `isCandClient`, `isCandIssuer` sets | any control-document write that changes it → **full fallback** (§4.2) |
| `subjects` | subject term → the set of control graphs containing ≥1 triple with that subject (§4.1b) | the written document's entries only |
| `deps` | control document → the control documents its policies actually read (§4.1 closure, over `subjects`) | recomputed for the written document only |
| `rdeps` | the reverse of `deps`: control document → origins whose slice reads it | derived from `deps` |
| `slices` | `org → V\|org`, the installed view partitioned per §5.1 | replaced per re-derived origin |

`deps` is computed from **actual triple occurrence**, never by fragment-stripping an IRI to
guess its document (§4.1b). Build `subjects` in the same pass `assemble_facts` already makes
over `Ctl(S)`, then take `deps*(D)` as the least fixed point of:

1. seed the frontier with the subjects appearing in `D`'s own triples;
2. for each frontier subject `s`, add **every** control graph in `subjects[s]` to the
   dependency set — this is the step the naming-convention walk gets wrong;
3. from those graphs' triples about `s`, follow the object IRIs of the linkage predicates
   `acp:{accessControl, memberAccessControl, apply, allOf, anyOf, noneOf}` and
   `acl:agentGroup`, and push the resulting terms onto the frontier;
4. iterate until the dependency set stops growing (finite: bounded by `Ctl(S)`).

Group documents are the one place naming *is* the mechanism and stay as they are: `acl:agentGroup`
selects a **content** graph by fragment-stripped IRI, which is how `loader::collect_agents`
already resolves it, so that lookup is faithful to what the full materializer does.

`rdeps` is the reverse of that relation, keyed by the graphs step 2 actually yielded. So a
write to any control graph that *contributes triples* about a reachable subject invalidates
the origins reading it, whether or not the graph's name resembles the subject's.

A conservative alternative — follow *every* object IRI, and treat every control graph
mentioning any reachable subject as a dependency — is strictly safer and cheaper to argue;
prefer it unless measurement (on the bench harness, not the work box) shows the precise walk
is needed. Either way the predicate list must be guarded by an **exhaustiveness test** that
re-reads `rules/*.n3` and fails when a linkage predicate appears in a rule body but not in the
list — otherwise a future rule silently un-sounds the scoping.

**The alternative is to make the naming convention true.** Enforce a **document-locality
invariant** at write admission — reject a control document carrying a triple whose subject's
fragment-stripped IRI is not that document — and validate it on load. That collapses every
`subjects[s]` to a singleton, makes the cheap fragment-stripping walk correct, and makes WAC's
`rdeps` genuinely empty (§4.1). The cost is that it rejects documents the shipped full
materializer accepts today, so it is a **behaviour change with a migration story**, owed its
own bead and its own conformance review — not something a scoped-materialization child may
assume. Pick one of the two explicitly; the fast path is unsound under neither-of-the-above.

### 5.3 The write path

On `put_acl(D)` / `delete_acl(D)` with `O = org(D)`:

1. Swap the document as today (parse-first, capture the prior slot for rollback).
2. Recompute `lattice`. **If it changed, fall back to the existing full re-materialization**
   and return — the fast path is not attempted.
3. `affected = {O} ∪ { org' : D ∈ deps(org') }` — i.e. `O` plus `rdeps(D)`, with `rdeps`
   built on triple occurrence (§5.2). For WAC `rdeps(D)` is *narrow* but **not** empty; it
   collapses to `{O}` only under the document-locality invariant (§4.1, §5.2).
4. For each `org ∈ affected`, assemble the **scoped input**: `resource_facts` restricted to
   `org` (memoized, §5.5), plus the triples of every control document in `deps*(org)` (the
   closure), plus every group document those reference, plus the pinned `lattice` seed facts,
   plus (ACP) the `AccessProvenance` entries whose resource is at `org`. Run the same compiled
   rule set(s) over it — one stratum for WAC, three chained for ACP — and filter through the
   existing `install_auth_view` predicate filter to get the new `V|org`.
5. **Splice**: `V' = ⋃ { new V|org : org ∈ affected } ∪ ⋃ { retained V|org : org ∉ affected }`,
   built into a fresh sub-graph dictionary exactly as `install_auth_view` does today. Because
   class-3 triples are pinned by the lattice, the union is consistent on the overlap; assert
   this (debug-only) rather than assuming it.
6. **Patch the index**: replace only the `allow`/`deny`/`cond` buckets keyed by
   `affected` in a clone of the current `AuthIndex`, leaving the rest — instead of
   `AuthIndex::from_graph` over the whole view. The matcher maps are rebuilt from the
   spliced class-3 triples (they are pinned, so this is cheap and stable).
7. Invalidate the session cache exactly as `sq-b7k7u` does: `reindex_with` already diffs old
   vs new index per origin. **Keep the diff.** It is a cheap, independent safety net that
   catches any over-narrow `affected` set as a stale-cache bug rather than a wrong decision —
   do not "optimize" it away into `affected`.
8. Any error at any step ⇒ discard everything and fall back to the full path, preserving the
   existing all-or-nothing contract (`materialize_*` mutates only after its last fallible
   step, so the previous view survives).

### 5.4 Group-document changes

Group documents are content graphs, written through `update_as`, which today re-materializes
fully (`ReindexScope::Full`). That stays correct and is the v1 answer to the issue's third
open question. To scope it later: `rdeps` already records which control documents reference a
group document, so `affected = { org(D) : D references the changed group doc }`; the same
lattice-fallback rule applies, because a group document contributes `vcard:hasMember` WebIDs
to `isWebId`. Ship it as a separate child, after the ACL-write path is proven.

### 5.5 Free win: ancestry memoization

`put_acl`/`delete_acl` change **only** a control graph. Control graphs are excluded from
`Res(S)` by `assemble_facts` (step 1 skips anything ending in the control suffix), and the
`.acl` document's target `R` is added to `Res(S)` only if `R` independently exists as a
content graph or container prefix. Therefore **`Res(S)` — and hence `common.n3`'s entire
`parent`/`ancestor` closure — is invariant under an ACL write.**

That closure is the part `common.n3` warns is quadratic if seeded wrong ("split into
candidate + filter ON PURPOSE"). Memoizing it keyed by a fingerprint of `Res(S)` is a
correctness-trivial, system-independent win that lands *before* any scoping work and is
independently measurable. It may well be the larger share of the write-path cost; measure it
on the bench harness before assuming the scoping is where the time goes.

---

## 6. What this does not establish

- **No claim that the scoped path is faster.** Every number belongs in `bench/` measured on
  the canonical box; the fallback branches (§4.2, §5.3 step 8) mean a pathological workload
  can be *slower* than today. Each child bead must carry its own measurement, and a child
  that does not beat the full path on the ACL-churn workload should not land.
- **No claim of a security improvement.** This is a latency change with a soundness
  obligation, not a hardening. The §2.4 reasoner-input boundary, the reserved-principal
  validation and the `solidx:` derivation-vocabulary guard are all preserved unchanged; §4.5
  narrows *when* validation runs and is compensated at write admission.
- **The confinement proofs in §3 are proofs about the rule files as they are at
  `ccae3aec`.** They are premise-level arguments, not machine-checked. A rule change can
  invalidate them silently, which is why §5.2's exhaustiveness test and §7's property test
  are mandatory parts of the deliverable rather than nice-to-haves.
- **No document-locality invariant is claimed or enforced.** The store today accepts a
  control document describing any subject, including another document's (§4.1b), and this
  record does not change that — it only requires the dependency index to be computed from
  where triples actually are. Whether the merged-fact-set behaviour §4.1c describes is the
  intended trust model is an open question about the shipped materializer, left to its own
  bead.

---

## 7. Mandated property test

Every implementation child ships against this harness; it is the acceptance oracle.

**Property.** For any store whose control documents all pass `validate_principal_iri`, and
any finite sequence of operations `op₁ … opₙ` drawn from {`put_acl`, `delete_acl`,
`put_acl_acp`, `delete_acl_acp`, group-document edit, content-graph add/remove}, after **every
prefix** the store's installed `<urn:sparq:auth>` triple set and its `AuthIndex` buckets are
**identical** to those of a store built by replaying the same prefix with the full
materializer.

Concretely, as a `proptest`/hand-rolled generator over a small alphabet:

```text
for each generated sequence:
    scoped = PodStore::new(seed)   // scoped write path enabled
    full   = PodStore::new(seed)   // forced full re-materialization
    for op in sequence:
        apply(op, scoped); apply(op, full)
        assert_eq!(auth_view(scoped), auth_view(full))          // triple-set equality
        assert!(index_buckets_eq(scoped, full))                 // per-origin, all origins
        for s in probe_sessions, m in all modes:                // decision equivalence
            assert_eq!(scoped.accessible(s, m), full.accessible(s, m))
```

The generator must be able to produce, and the suite must contain a named regression for,
each of the hazards above:

| # | Scenario | Reds if |
| --- | --- | --- |
| 1 | ACP policy body at `O`, applied by an ACR at `P`; write to `O` | `affected` omits `rdeps` (§4.1) |
| 2 | `.acr` write introducing a WebID unseen elsewhere, with a `noneOf` matcher present at another origin | the lattice fallback is missing (§4.2) |
| 3 | Two origins' conditional grants sharing one `noneOf` matcher | class-3 slice overlap mishandled (§4.3) |
| 4 | `.acl` containing a blank node, followed by a write that permutes `graph.named` | positional skolemization (§4.5) |
| 5 | `acl:agentGroup` pointing at a group document at a third origin, group doc then edited | group-doc dependency tracking (§5.4) |
| 6 | Container-chain inheritance: `.acl` written at `/a/`, resources under `/a/b/c` | scoped input drops the ancestor chain |
| 7 | `delete_acl` of the only ACL granting a foreign-subject principal | stale retained slice (fail-open) |
| 8 | ACP split subject: `P`'s ACR applies `<o#pol>`, but `<o#pol>`'s `acp:allow` + `acp:allOf` matcher facts live in a *differently named* `<https://q.ex/other.acr>`; then narrow, then delete, `Q` | `deps` located the body by fragment-stripped IRI instead of by subject occurrence, so `Q ∉ deps*(P)` (§4.1b) |
| 9 | WAC split subject: `<o.acl#auth>` is declared in `O`'s `.acl` (so `inDoc` holds) but its `acl:agent` is supplied by a `.acl` at `Q`; edit `Q` | WAC's `rdeps` was assumed empty rather than computed from occurrence (§4.1) |

**Non-vacuity obligation.** Before any child PR opens, delete or invert its headline guard —
the `affected`-set computation, the lattice-fallback branch, the splice overlap check — and
run the suite. If nothing reds, the test is vacuous and the PR is not ready. Name the test
that died in the PR body. A suite that asserts only `accessible()` equivalence will **not**
red on hazard 2 (§4.2 shows the divergence is decision-inert today), which is precisely why
triple-set equality is mandated and not optional.

Hazards 8 and 9 carry a **named** mutation obligation on top of that, because they are the two
the naming-convention shortcut passes by accident: drop the occurrence lookup (§5.2 step 2) so
`deps*` is computed by fragment-stripping alone, and re-run. Hazard 8 must red — `Q` is now
missing from `P`'s closure — and hazard 9 with it. A dependency index that stays green under
that mutation is discovering `Q` some other way, or the fixture is not actually split, and the
regression proves nothing.

---

## 8. Disjoint implementation children

Ordered by dependency; each is single-crate (`sparq-solid`) and separately mergeable.

1. **Deterministic per-document skolemization** (§4.5). Replace positional `gix` with a
   percent-encoded document-IRI key in `loader::skolemize`. Prerequisite for everything
   below; a strict improvement on its own. Acceptance: skolem IRIs are stable across a
   `graph.named` permutation; existing differential tests in `materialize.rs` stay green.
2. **Ancestry/resource-set memoization** (§5.5). Cache `Res(S)` + the `parent`/`ancestor`
   closure; prove and test invariance under `put_acl`/`delete_acl`. Independent of scoping;
   measurable on its own.
3. **Global principal-lattice summary + fallback** (§4.2, §5.2). Maintain the four sets;
   detect change on a control-document write; wire the full-path fallback. No scoping yet —
   the fallback fires every time, so this is a pure no-op refactor with the summary under test.
4. **Subject-occurrence + dependency/reverse-dependency index** (§5.2, §4.1). Built on
   `subjects` — the subject → containing-control-graphs map — **not** on fragment-stripped
   IRIs; ships the linkage-predicate exhaustiveness test against `rules/*.n3` and the
   split-subject discovery test (hazards 8–9). Still no scoping.
5. **The property-test harness** (§7) with all nine regressions, run against the *unscoped*
   store first (it must pass trivially) so the harness is proven non-broken before it is the
   oracle for anything.
6. **WAC scoped materialize + splice** (§5.1, §5.3). The easier half — no reference
   indirection, so `deps*` is just the control graphs mentioning subjects already declared in
   `O`'s own/inherited ACLs — but `rdeps` is empty only under the document-locality invariant,
   so it still consumes child 4's occurrence index (§4.1). Gated by 4 and 5.
7. **ACP scoped materialize + splice** (§5.3 with the full `affected` set). Gated by 4 and 6.
8. **`AuthIndex` per-origin patch** (§5.3 step 6), replacing the whole-view `from_graph` on
   the scoped path. Keep the `reindex_with` diff as the safety net.
9. **Scoped group-document invalidation** (§5.4). Last, optional.

Children 1, 2, 3, 4 and 5 touch disjoint files/functions and can run in parallel. 6 and 7
both touch `materialize.rs` and must serialize.
