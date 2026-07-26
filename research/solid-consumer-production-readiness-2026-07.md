<!-- [OPUS-5] Maintainer-review response to #1346 — the Solid-server track's consumer readiness checklist, reconciled against origin/main. -->
# Solid-server consumer production-readiness — reconciliation and sequencing verdict

**Responds to:** [#1346](https://github.com/jeswr/sparq/issues/1346) (PSS agent, consumer-side
requirements for the Solid-server track).
**Status:** design record for maintainer review. No crate code changes.
**Scope:** the served (`sparq-server`) surface and the consumer-facing contracts around it.

Issue #1346 is a consolidated consumer requirements view: it asks the SPARQ side to (a) **confirm the
P0/P1 sequencing** and (b) **steer whether the HTTP `/authz` surface is on the critical path**. It
explicitly implies no code change.

This record answers both questions. Before answering, it does the thing the issue could not do from
outside: **re-verifies every claimed gap against `origin/main`**. #1346 was written against a
substantially older tree, and most of its P1 block has since landed. Sequencing advice built on the
issue's own snapshot would be wrong, so the reconciliation comes first.

> **Honesty note.** Every verdict below cites a file, a test, or a config line in this repository.
> Where a capability is implemented but its *result* is unpublished, or implemented but *not
> ratified*, that distinction is kept — "built" and "blessed" are different states, and the whole
> point of #1346 is the second one.

## 0 — The verdict, up front

**The issue's P0/P1 sequencing is only half right, and the half that is wrong is the expensive
half.**

- **P0 is confirmed, but its two items are not equal.** The *distribution* item (P0-2) is the real
  blocker and is entirely unstarted in the sense that matters to a consumer. The *frozen API* item
  (P0-1) is ~80% done as engineering — the policy document, the tier-1 surface definition and the
  in-code markers all exist — and what remains is a governance signature plus one CI gate.
- **P1 should be struck as a milestone.** All three P1 items have substantially landed.
  DEVIATION-1 is closed and regression-tested over real HTTP; the Access-Controlled SPARQL Query
  endpoint is built, feature-gated and conformance-tested; the `/authz` dependency-direction call
  was made by implementation (`sparq-server → sparq-solid`, feature-gated). What is left of P1 is
  *publication and blessing*, not construction.
- **The next production milestone is therefore not "frozen API + DEVIATION-1/AC-SPARQL". It is
  "frozen API + shippable artifact".** Concretely: ratify the tier-1 freeze, make it enforceable,
  and fix the parser-fidelity-vs-distribution problem so the blessed surface is reachable by
  something other than a git rev.
- **On the `/authz` steer: it is built, so the question is no longer whether to build it — it is
  whether to *support* it.** Recommendation: keep it feature-gated and explicitly *unblessed*
  (tier-2) for now; do not put it on the critical path. It is the right surface for the future
  TS `prod-solid-server` QLever→SPARQ migration, and blessing it should be sequenced *with* that
  migration, not ahead of it.

The rest of this document is the evidence for those four claims, then a phased plan.

## 1 — Reconciliation against `origin/main`

Verdicts: **OPEN** (as #1346 describes it) · **PARTLY** · **CLOSED** (no longer true as stated).

| # | #1346 item | Verdict | One-line reason |
|---|---|---|---|
| P0-1 | Frozen, semver-stable public API | **PARTLY** | Policy + tier-1 surface + in-code markers exist; ratification and an enforcement gate do not. |
| P0-2 | Consumable, conformance-complete releases | **OPEN** | `[patch.crates-io]` does not propagate to published crates; a git pin is still mandatory for a conformant parser. |
| P1-1 | Named-graph isolation on the served surface (DEVIATION-1) | **CLOSED** | The SPARQL and GSP paths are per-graph isolated and regression-tested over HTTP. |
| P1-2 | AC-SPARQL Query service + conformance scenarios | **PARTLY** | Endpoint built and gated; 15 executable query-semantics cases pass, covering 10 of the draft's 16 scenarios; result unpublished, the other 6 scenarios delegated out of class. |
| P1-3 | The `sparq-server → sparq-solid` `/authz` architecture call | **CLOSED (built)** | All three endpoints ship behind the `solid-authz` feature; the dependency direction is settled. |
| P2-1 | Backup-delta / PITR design + DR runbook | **PARTLY** | Design is *not* open — delta/PITR is implemented and tested. The runbook is genuinely missing. |
| P2-2 | Shared durable backend / replication | **OPEN** | ADR is `PROPOSED — awaiting sign-off`; no replication, log-shipping or object-store code exists. |
| P2-3 | First-party endpoint authn/authz | **OPEN (by decision)** | Still one shared secret with no per-user identity — but delegate-to-gateway is a *recorded* decision, not an oversight. |
| P2-4 | Published served-surface Protocol conformance | **PARTLY** | A CI-gated HTTP Protocol lane exists; nothing publishes its result. |
| P2-5 | Operations doc for the authoritative-store profile | **OPEN** | Content exists, scattered across README / SKILL / `deploy/`; no consolidated runbook. |

### 1.1 — P1-1 DEVIATION-1 is closed (correcting the issue's premise)

The issue states the served GSP/read path "folds named graphs into one default graph" and that "the
client-facing endpoint must not ship on the fold path". **This is no longer true.**

- GSP named-graph reads go through a graph-scoped query, not a default-graph dump —
  `crates/sparq-server/src/http.rs:7770` builds `SELECT ?s ?p ?o WHERE { GRAPH <iri> { ?s ?p ?o } }`
  for `GraphRef::Named`.
- GSP named-graph writes are wrapped in `GRAPH <iri> { … }` (`graph_data_block`,
  `crates/sparq-server/src/http.rs:7312`), and `PUT` lowers to
  `DROP SILENT GRAPH <iri> ; INSERT DATA { GRAPH <iri> { … } }`.
- Isolation is asserted over real HTTP by
  `crates/sparq-server/tests/named_graphs.rs::named_graph_data_is_isolated_from_default_graph`: a
  default-graph query over named-graph-only data must return zero rows, while a `GRAPH ?g` wildcard
  sees all of it. `crates/sparq-server/tests/protocol.rs::gsp_put_then_get_roundtrip_named` asserts
  a named-graph `PUT` does not leak into the default graph.
- Cross-graph joins, `FROM`/`FROM NAMED` overrides, `VALUES` membership and graph-scoped
  `DELETE/INSERT … WHERE` are all covered in `crates/sparq-server/tests/named_graphs.rs`, whose
  module header records that it exists specifically to close this server-level gap.

**The residual fold is real but is correct GSP behaviour, not a deviation.** `Graph::load_str`
(`crates/sparq-core/src/lib.rs:927`) folds a quad syntax into the default graph, and the GSP write
path reuses it deliberately: under the Graph Store Protocol the *URL* names the graph and the body
is a triple payload, so graph names inside a TriG/N-Quads body are not authoritative. The
`load_dataset` path preserves them where dataset semantics are wanted.

**Action for the consumer, not for sparq:** the deviation note in
`crates/sparq-lws-core/src/store/http.rs:22-34` is now stale on two counts (it says HTTP does not
isolate named graphs, and that GSP write verbs return `501`). Both are false on this tree. That
comment lives outside this issue's routed scope, and **it has no task record yet** — this record is
the only place it is currently written down (§5, item 12).

### 1.2 — P1-2 AC-SPARQL is built; what is missing is publication

`POST /authz/query` is routed at `crates/sparq-server/src/http.rs:3464` to
`crate::solid_authz::query_endpoint`, alongside `/authz/decide` (`:3459`) and `/authz/wac-allow`
(`:3461`). It is **double-opt-in**: the `solid-authz` cargo feature *and* the `--solid-authz` /
`SPARQ_SOLID_AUTHZ=1` runtime flag; without the runtime flag all three routes are `404`.

The spec semantics live at the library seam in `sparq-solid` and are real:

- Per-request WAC-restricted graph-set construction — `PodStore::accessible(&session, mode)`.
- **Empty standing default graph** with explicit union opt-in — `wrap_for_view_opt_in`, keyed on
  `UNION_DEFAULT_GRAPH_IRI` (`crates/sparq-solid/src/rewrite.rs:132`), exact-match so a near-miss
  IRI fails closed.
- Read-only — the query entry points take `Read` mode and never reach the update path;
  `update_as` is the separate write-gated path.

Conformance: `crates/sparq-solid/tests/conformance/solid-sparql-query/manifest.json` vendors the
editor's draft as **15 executable query-semantics cases**, all asserted passing behind a hard floor
in `crates/sparq-solid/tests/conformance_solid_sparql_query.rs`. **Cases are not scenarios**: the
draft numbers **16 scenarios**, the 15 cases cover **10 of them (2–8 and 11–13)** — several
scenarios carry more than one case, and one case discharges three at once — and the manifest records
the remaining **6 (1, 9, 10, 14, 15, 16) as explicitly out of class**: protocol-binding equivalence,
JSON-LD flattening, no-raw-preservation, service description, Update refusal and caching are
HTTP-server or parser properties, not query-engine ones, and each carries a written reason.
So the in-class scenario coverage is complete *for the query-semantics conformance class*, which is
a narrower claim than conformance to the draft as a whole.

Two honest caveats worth carrying into any consumer-facing claim:

1. **Non-disclosure is enforced behaviourally, not against timing.** The tests assert that an
   unreadable graph yields zero bindings, `false` for `ASK`, and no contribution to aggregates. **No
   test asserts timing-channel equivalence** between an unreadable graph and a nonexistent one. If
   the consumer's threat model includes a timing oracle, that is an open item, and it should not be
   described as covered.
2. **The out-of-class scenarios are the consumer's obligation.** Scenarios 1/14/15/16 land on
   whatever HTTP surface fronts this. If `solid-server-rs` embeds in-process, it owns them; if it
   uses `/authz/query`, `sparq-server` owns them and they are currently untested.

### 1.3 — P0-2 is the real P0

The distribution problem is structural, not a matter of tagging:

- `Cargo.toml:20-21` wires the conformance-fixed parser as `[patch.crates-io] spargebra = { path =
  "vendor/spargebra" }`. **`[patch.crates-io]` is a workspace-root mechanism and is not carried into
  a published `.crate`.** Every publishable crate declares `spargebra.workspace = true`, resolving
  to the registry version. A crates.io consumer therefore gets upstream `spargebra`, without the
  fixes catalogued in `vendor/spargebra/SPARQ-PATCHES.md`.
- This is already acknowledged honestly in `CHANGELOG.md` (v0.1.0 "Crates.io build caveat").
- Tags are `v0.1.0` and `v0.1.0-dev.3` only. No release carries the fixes — **and none could**,
  under the current mechanism, for a registry consumer.
- `release-plz.toml:103` sets `publish = false` with the reason recorded inline: the
  `CARGO_REGISTRY_TOKEN` / trusted-publisher bootstrap has not happened. The 17-crate
  `version_group` is otherwise configured and ready.
- npm: the name **is** settled — `@jeswr/sparq` in `js/package.json`. What is outstanding is the
  one-time registry-side bootstrap publish that npm Trusted Publishing requires.

So the consumer's "must git-pin an exact rev" is correct and will remain correct until the parser
fidelity problem is resolved. There are three ways out, and the choice is a maintainer call
(§4, Q2): land the patches upstream; publish the vendored parser under a sparq-owned crate name and
depend on that; or accept the divergence and document the git-pin as the supported channel.

**Why this outranks the API freeze:** freezing an API that can only be consumed by git rev freezes
the *shape* of the dependency without fixing its *delivery*. A consumer pinned to a rev already
absorbs breakage on every bump — but it also, today, silently gets a *different parser* depending on
how it depends on sparq. Parser divergence is a correctness hazard; API churn is a maintenance
hazard. Correctness first.

### 1.4 — P0-1 is mostly built; the remainder is governance plus one gate

`docs/api-stability.md` exists and is substantive: a two-tier model, a defined tier-1 surface
covering both the embed data path and the WAC-decision surface, written tier-1 guarantees, and a
deprecate-then-remove window (announce → one full minor → removal at major). Its status section is
explicit and correct: **"The tier-1 semver freeze is NOT active"** — it is a proposal that exists so
ratification is a signature rather than a design exercise. The in-code markers in
`crates/sparq-serve/src/embed.rs` and the `sparq-solid` decision surface say the same.

The named surfaces all exist: `sparq_serve::embed` re-exports `GenerationRing`, `Writer`,
`Generation`, `RingConfig` and friends; `PodStore` implements `decide`, `decide_batch`,
`resolve_acl` and `wac_allow`, with `put_acl` / `delete_acl` in
`crates/sparq-solid/src/write_through.rs`.

**The one engineering gap is enforcement.** `cargo-semver-checks` runs only inside `release-plz
release-pr`; there is no per-PR gate. A freeze without a gate is a promise the CI cannot keep — the
first accidental breaking change lands and is discovered by the consumer. Ratification should not
precede the gate.

### 1.5 — P2, briefly

- **Backup/PITR (P2-1): the issue's premise is wrong.** The design is not open. `POST
  /admin/backup`, `POST /admin/backup/delta?from=N` and `POST /admin/restore` are routed at
  `crates/sparq-server/src/http.rs:3358-3363` behind the `backup` feature, with `--restore` /
  repeatable `--restore-delta` for base+chain replay on start, single-flight restore (`409` on
  concurrent), and `?persist=true` write-through. `crates/sparq-server/tests/backup.rs` covers the
  round trip, startup restore, a full HTTP PITR recovery, and fail-closed rejection of a corrupt
  delta in the chain. **What is genuinely missing is the runbook** — an ordered, validated
  snapshot → delta cadence → simulated failure → restore-to-point → verify procedure. The reference
  material in `skills/http-server/SKILL.md` is API documentation, not a procedure.
- **Replication (P2-2): confirmed open.** `research/adr-horizontal-scaling.md` is `PROPOSED —
  awaiting sign-off` with open questions, and no replication / consensus / log-shipping /
  object-store code exists. `deploy/terraform` confirms instance count is forced to 1. The single
  sequenced writer remains a deliberate choice, as `crates/sparq-server/README.md` states.
- **First-party authn (P2-3): confirmed open, but it is a decision, not an omission.** The only
  gate is `--auth-token` / `--auth-token-read`, one shared secret with no per-user identity. The
  delegate-to-gateway posture is recorded in `research/threat-model.md` (which names the residual
  gap plainly) and in `compliance/asvs/gap-register.md` as "N/A by architecture". No document
  *proposes* OIDC/JWT for `sparq-server` itself. The honest framing for #907 is therefore: this is
  answered for embedded consumers and unanswered for HTTP-exposed ones.
- **Protocol conformance (P2-4): built, unpublished.** `crates/sparq-conformance/src/http_protocol.rs`
  drives raw HTTP against a real in-process server (GET / urlencoded POST / direct POST, the `QUERY`
  method, Update, dataset overrides, conneg, status codes), floored at `HTTP_PROTOCOL_FLOOR = 21` in
  `crates/sparq-conformance/tests/http_protocol_suite.rs:59` and gated in `.github/workflows/ci.yml`.
  The result is a CI log line and nothing else — no report file, no site page. The consumer's actual
  need ("so a consumer can trust the wire contract") is unmet even though the testing exists.
- **Operations doc (P2-5): confirmed open.** The pieces exist — writer ceiling and `--persist`
  semantics in the server README, the auth × bind matrix and backup API in the SKILL, infrastructure
  in `deploy/` — but sizing is undocumented and nothing binds them into one authoritative-store
  operator guide.

## 2 — Recommendation: the next production milestone

Replace #1346's "P0 + P1" milestone with a tighter one. Call it **"a blessed, reachable surface"**:

1. **Make the freeze enforceable, then ratify it.** Add a per-PR `cargo-semver-checks` gate scoped
   to the tier-1 surface in `docs/api-stability.md`. Then ratify — flip the markers from *proposed*
   to *in force*. Gate first: ratifying an unenforced promise is worse than not ratifying.
2. **Resolve parser fidelity vs distribution.** Decide among upstream / sparq-owned parser crate /
   documented-git-pin (§4 Q2), then execute. Until this lands, "production-ready for a consumer" is
   not a claim sparq can make, because two consumers depending on sparq two different ways get two
   different parsers.
3. **Publish the served-surface Protocol conformance result.** Cheap — the lane already runs. Emit a
   report artifact the way the inference report is emitted. This converts existing work into the
   consumer-facing trust signal that was actually asked for.
4. **Write the two missing operator documents** — the DR runbook (P2-1) and the authoritative-store
   operations guide (P2-5). Both are assembly of verified existing behaviour, not new engineering,
   and both are on the path to any real deployment.

Explicitly **not** in this milestone: replication (P2-2), first-party OIDC (P2-3), and blessing
`/authz` (§3). Each is a genuine gap, and each is gated on a decision that the current consumer —
which embeds in-process — does not need.

## 3 — The `/authz` steer

**The question in #1346 has been overtaken: the surface is built.** All three endpoints ship behind
`solid-authz`, dependency direction `sparq-server → sparq-solid`, fail-closed, double-opt-in.

So the live question is whether to put `/authz` on the critical path *now*. **Recommendation: no.**

- The current consumer does not use it. `solid-server-rs` consumes the decision in-process per its
  ADR 0001, which is the path that also gets the strongest isolation guarantees.
- The consumer that *will* need it is the TS `prod-solid-server` QLever→SPARQ migration, which
  #1346 itself describes as a future scoped initiative.
- Blessing it early means freezing a wire contract — request/response JSON, session shape, error
  taxonomy — against a consumer whose requirements are not yet written. That is exactly the mistake
  P0-1 exists to prevent, repeated on the HTTP surface.
- The out-of-class AC-SPARQL scenarios (1, 14, 15, 16) become `sparq-server`'s obligation the moment
  `/authz/query` is a supported client-facing endpoint. Blessing it without them would be
  overclaiming spec conformance.

**Concrete disposition:** keep `/authz/*` **tier-2 (experimental)** in `docs/api-stability.md`,
keep it feature-gated and runtime-opt-in, and state in the server README that the in-process
`sparq-serve::embed` path is the supported embedding channel today. Sequence the `/authz` freeze
with the TS migration, and make the out-of-class scenarios a precondition of that freeze rather
than a follow-up to it.

## 4 — Open questions that genuinely need the maintainer

1. **Ratify the tier-1 freeze?** `docs/api-stability.md` is a complete proposal awaiting a
   signature. Recommendation: yes, immediately after the per-PR semver gate lands.
2. **Which route out of the parser-fidelity problem?** (a) land `vendor/spargebra/SPARQ-PATCHES.md`
   upstream and wait; (b) publish the vendored parser under a sparq-owned name and depend on it
   directly, dropping `[patch.crates-io]`; (c) accept divergence and document the git pin as the
   only supported channel. This is a governance and maintenance-burden call, not a technical one.
   Path (b) unblocks fastest and is reversible if (a) later lands.
3. **Do the crates.io / npm bootstrap publishes happen now?** Both are blocked on a one-time
   registry-side maintainer action that cannot live in the repository.
4. **Sign off `research/adr-horizontal-scaling.md`, or defer it explicitly?** It has been `PROPOSED`
   long enough that its status is itself ambiguous. Either answer is fine; the ambiguity is not.
5. **Is "front it with a gateway" the standing answer to #907?** It is already the de facto recorded
   decision in the threat model and the ASVS gap register. If yes, say so in the server README as a
   posture rather than a gap, and close #907 as answered-by-architecture. If no, #907 needs a design
   record it does not currently have.
6. **Does the consumer's threat model include a timing oracle on AC-SPARQL?** If yes, timing-channel
   equivalence needs a test and is a new work item. If no, record that so the behavioural-only
   coverage is not later mistaken for more than it is.

## 5 — Phased plan (proposed sequencing — not yet tracked work)

Ordered. Each is a separate unit of work. **None of these has a task record**: this is a *proposed*
decomposition for the maintainer to accept or reject, not a backlog that exists somewhere. Nothing
here is started, and nothing here should be cited as captured work until the corresponding tracking
records are created — which is a deliberate follow-on to accepting this record, not part of it.

1. **Per-PR `cargo-semver-checks` gate over the tier-1 surface** defined in `docs/api-stability.md`.
   Blocks (2). *(ci-infra)*
2. **Ratify the tier-1 freeze** — flip the in-code markers and `docs/api-stability.md` from
   *proposed* to *in force*. Maintainer action; depends on (1) and Q1. *(docs/governance)*
3. **Resolve parser fidelity for published artifacts** per the Q2 decision, so a non-git consumer
   gets the conformant parser. Depends on Q2. *(build/supply-chain)*
4. **Crates.io + npm bootstrap publish**, then flip `release-plz.toml` `publish = true`. Depends on
   (3) and Q3. *(release)*
5. **Publish the served-surface SPARQL 1.1 Protocol conformance report** as an artifact from the
   existing `http-protocol` lane, alongside the inference report. Independent; can run in parallel.
   *(conformance)*
6. **Backup → restore → PITR disaster-recovery runbook**, validated end to end against the existing
   `backup` feature. Independent. *(docs + sparq-server)*
7. **Authoritative-store operations guide** — sizing, `--persist` guarantees, writer-ceiling
   implications, auth × bind matrix, backup/restore pointers, high-tenancy many-small-named-graphs
   profile. Depends on (6) for its DR section. *(docs)*
8. **AC-SPARQL out-of-class scenarios (1, 14, 15, 16) on the `/authz/query` surface** — service
   description, Update refusal, protocol-binding equivalence, caching. Precondition for ever
   blessing `/authz`. *(sparq-server)*
9. **Timing-channel non-disclosure assessment** for the access-controlled query path. Gated on Q6.
   *(sparq-solid)*
10. **`/authz` wire-contract freeze**, sequenced with the TS `prod-solid-server` migration, not
    before. Depends on (8). *(governance)*
11. **Horizontal-scaling ADR sign-off or explicit deferral.** Gated on Q4. *(research/governance)*
12. **Correct the stale DEVIATIONS note** in `crates/sparq-lws-core/src/store/http.rs:22-34` (§1.1):
    it still says HTTP does not isolate named graphs and that GSP write verbs return `501`, both of
    which are false on this tree. Independent and cheap. *(sparq-lws-core)*

Items 1–7 and 12 are the recommended milestone. Items 8–11 are gated on maintainer decisions or on
the downstream migration and should not block it.

## 6 — Corrections to #1346's premise, collected

For the record, so the consumer-side checklist can be updated rather than re-litigated:

- **DEVIATION-1 is closed.** The served SPARQL and GSP paths isolate named graphs and are
  regression-tested over HTTP. The remaining fold is GSP-correct body handling.
- **AC-SPARQL is not "unbuilt as a served endpoint".** It is built, feature-gated, and passes all
  15 executable query-semantics conformance cases, which cover the 10 in-class scenarios (2–8,
  11–13); the draft's other 6 scenarios (1, 9, 10, 14, 15, 16) are recorded as out of class with
  reasons.
- **The `/authz` architecture call is not pending.** It was made by implementation.
- **Backup-delta / PITR design decisions are not open.** They are implemented and tested. Only the
  runbook is missing.
- **A written API stability and deprecation policy exists** (`docs/api-stability.md`). What is
  missing is ratification and enforcement, not the document.
- **A served-surface HTTP Protocol conformance lane exists and gates CI.** What is missing is
  publication.
- Conversely, **#1346 understates P0-2**: the parser divergence is not a tagging oversight but a
  structural consequence of `[patch.crates-io]`, and no tag can fix it without a decision.
