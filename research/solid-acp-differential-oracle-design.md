# Solid WAC/ACP differential-oracle design

Status: **design record** — *not* an implementation, *not* a change to the paper's framing.
<!-- [OPUS-4.8] sq-t58w.2 -->

This record designs a **differential oracle** for the WAC/ACP decision-parity corpus: a
deterministic, in-repo test that diffs sparq-solid's access-control *decisions* against an
*independent* reference WAC/ACP evaluator over the same `(resource, agent, client, mode)`
scenarios. It is the realistic, ruleset-compatible alternative to the HTTP Conformance-Test-Harness
(CTH) wire conformance that the feasibility record ruled out (PSS-subject, `blocked:docker`).

It builds directly on, and does not restate, two records it depends on:

- [`research/solid-cth-wire-conformance-feasibility.md`](./solid-cth-wire-conformance-feasibility.md)
  (parent `sq-t58w`; §5 names this oracle as the "next honest strengthening");
- [`research/sparq-solid-scope.md`](./sparq-solid-scope.md) §4 (the conformance scoping; item 3
  "differential oracle against CSS — still not started" is the work this record designs).

## Bottom line up front

1. **Honest scope.** This is **DECISION parity at the library level** — the same binary
   `(agent, client, mode, resource) → allow | deny` the existing
   [`conformance_wac.rs`](../crates/sparq-solid/tests/conformance_wac.rs) /
   [`conformance_acp.rs`](../crates/sparq-solid/tests/conformance_acp.rs) corpus already asserts,
   re-checked against a *second, independent* evaluator. It is **NOT** HTTP wire conformance
   (status codes, the `WAC-Allow` header) — that stays PSS-side (gh-55), tracked as a decision in
   `sq-t58w.4`/`.5`. No `WAC-Allow` / `403` claim is in scope here.

2. **What the oracle diffs.** sparq-solid's decision = "is the resource graph in
   `AuthIndex::accessible(session, mode)`?" (the engine the corpus already exercises, reached via
   N3 rules `rules/{common,wac,acp-a,acp-b,acp-c}.n3` through `reason_n3`). The oracle re-evaluates
   the **same emitted ACL/ACR RDF + the same request tuples** through an independent evaluator and
   diffs the decision. Because sparq-solid decides WAC/ACP by **declarative N3 inference rules**,
   an independent **procedural** reference reading of the spec is exactly the kind of second oracle
   that catches rule-vs-procedure drift the hand-derived expected-decision table cannot.

3. **A clean, importable, server-free JS reference *decision* engine DOES exist** (corrected
   premise — verified against the npm registry, see §1). Two packages:
   **`@solidlab/policy-engine`** (`CommunitySolidServer/policy-engine`), a `new`-able TypeScript
   **WAC + ACP** engine whose only caller obligation is a tiny `AuthorizationManager`
   (`getParent` + `getAuthorizationData`) backed by **in-memory** ACL/ACR graphs — no HTTP server,
   IdP, or pod; and **`@solid/acl-check`** (`nodeSolidServer/acl-check`), a pure-`rdflib`
   **WAC-only** `checkAccess(...)` evaluator. CSS's *in-tree* `WebAclReader`/`AcpReader` remain
   DI-heavy (5 injected collaborators), and the `solid-contrib/specification-tests` corpus is still
   server-coupled KarateDSL (not a decision table) — but a JS oracle is **not** gated on either of
   those. The honest caveats: `@solidlab/policy-engine` is **pre-1.0** (`0.0.2`, last published
   2024-12-05, research-grade, no stated CSS-dependency guarantee), and any such engine is
   *implementation* truth, not *spec* truth.

4. **Where it lives + determinism.** Two viable, complementary oracles, both meeting the
   determinism contract:
   - **(b) an in-repo Rust reference evaluator** running the existing corpus through a second,
     independent procedural reading — **no JS toolchain, no network, no clock, no Docker**, gating
     a **`0`-divergence ratchet** in the SHACL/geo/WAC/ACP runner shape; and
   - **(a) a JS twin** running the *same emitted RDF* through `@solidlab/policy-engine` (WAC+ACP)
     and/or `@solid/acl-check` (WAC), pinned by `package-lock.json`, offline — a genuinely
     *cross-language, cross-implementation* check.

   See §4 for the recommendation (build **(b) first**, then add **(a)** as a strong second oracle),
   and §6 for the honest trade-offs.

---

## 1. What it diffs — and the reference-evaluator landscape (verified)

### 1.1 The sparq-solid side of the diff (verified against the code)

The decision under test is produced by the engine the corpus already drives — confirmed in
[`crates/sparq-solid/src/wac_conformance.rs`](../crates/sparq-solid/src/wac_conformance.rs) and
[`conformance.rs`](../crates/sparq-solid/src/conformance.rs):

1. A scenario is built by `AclBuilder` / `AcrBuilder`, which **emit N-Quads** (one named graph per
   `.acl` / `.acr` document, plus placeholder document graphs) — `AclBuilder::into_nquads` /
   `AcrBuilder::into_nquads` expose the exact serialized corpus.
2. `materialize_wac` / `materialize_acp` load that dataset and run the N3 rule strata
   (`rules/common.n3` + `rules/wac.n3`; or `rules/acp-a.n3` → `acp-b.n3` → `acp-c.n3`) through
   `sparq_reason::reason_n3`, swapping the filtered closure in as the auth view.
3. The decision for one request is
   `AuthIndex::accessible(&Session { agent, client, issuer: None, now: None }, mode)` containing the
   resource graph (a boolean) — see `WacScenario::run` / `AcpScenario::run`. The request tuple is
   exactly `(agent: Option<&str>, client: Option<&str>, mode, resource)`.

So the oracle has a clean, already-RDF input (the emitted `.acl`/`.acr` N-Quads) and a clean,
already-enumerated request set (the `Expect` table's `(req_agent, req_client, req_mode,
req_resource)` tuples). **No new corpus modeling is required** — the diff reuses the same two
data structures the parity test already produces.

### 1.2 `@solidlab/policy-engine` — a server-free WAC + ACP decision engine (verified)

<!-- Corrected premise: this package is verified against the npm registry, not assumed. -->
`@solidlab/policy-engine` (GitHub `CommunitySolidServer/policy-engine`) is a plain-TypeScript,
`new`-able decision engine for **both WAC and ACP** — Components.js configs are *optional*, not
required. Its `PolicyEngine` interface exposes `getPermissions()` and `getPermissionsWithReport()`
(the latter emits an RDF justification report), with `WacPolicyEngine` and `AcpPolicyEngine`
implementations. The caller supplies an **`AuthorizationManager`** with just two methods —
`getParent(identifier)` and `getAuthorizationData(...)` — which the package deliberately does *not*
implement because it is storage-dependent. **That is the whole integration surface**: back those
two methods with the scenario's in-memory ACL/ACR graph and you have `(target, credentials,
modes) → permission report`, with **no HTTP server, no pod, no IdP**. It is far lighter to
instantiate than CSS's in-tree `WebAclReader` (which needs a `ResourceStore`/`ResourceSet`, aux +
identifier strategies, and an `AccessChecker` — five collaborators — and fetches the ACL itself
rather than accepting a pre-loaded graph).

**Verified registry facts** (queried directly): versions `0.0.1`, `0.0.2`; `dist-tags.latest =
0.0.2`; registry `modified = 2024-12-05`. **Honest caveats:** pre-1.0 and last published Dec 2024
(research-grade); SolidLab / CSS-org provenance but **no stated guarantee that CSS depends on it**;
and it is *one implementation's* reading of WAC/ACP — implementation truth, not spec truth. So it is
a *credible* second oracle, not an authoritative spec reference.

### 1.3 `@solid/acl-check` — a pure-rdflib WAC-only evaluator (verified)

`@solid/acl-check` (GitHub `nodeSolidServer/acl-check`, the evaluator the legacy Node Solid Server
used) is a **pure in-memory `rdflib`** WAC evaluator: `checkAccess(kb, resource, directory, aclDoc,
agent, modesRequired, origin, trustedOrigins)` — graph in, decision out, with nearest-ancestor ACL
resolution (matching WAC's nearest-wins). **Verified registry facts:** `dist-tags.latest = 0.4.5`;
registry `modified = 2026-05-29` (recently maintained, not abandoned); dependencies only `rdflib`
+ `solid-namespace`. **WAC-only — no ACP.** Cite the **scoped** name `@solid/acl-check` (the bare
`acl-check` is a 404 on npm). A good *WAC* cross-check; for ACP, `@solidlab/policy-engine` is the
only server-free JS option found.

### 1.4 What does NOT work as an oracle (verified)

- **Inrupt JS** (`@inrupt/solid-client`, `@inrupt/solid-client-access-grants`) — *manipulation /
  management* of ACLs/ACRs and Access-Grant VCs, with the actual ACP *evaluation* happening
  server-side in Inrupt ESS. Not a server-free decision engine.
- **CSS in-tree `WebAclReader`/`AcpReader`** — conceptually `(credentials, requested modes) →
  permission map`, but DI-heavy (5 injected collaborators; fetches the ACL via a `ResourceStore`
  rather than taking a pre-loaded graph). Usable only through heavy scaffolding — `policy-engine`
  is the lighter SolidLab-org route, so CSS-direct is **not** recommended.
- **`solid-contrib/specification-tests` / CTH corpus** — expected outcomes are **HTTP assertions in
  KarateDSL `.feature` files that presume a live server** (status codes, the `WAC-Allow` header),
  exactly as the feasibility record found. There is **no extractable server-free
  `(acl, agent, mode, resource) → allow/deny` table** to vendor; mining the `.feature` setups would
  be a derivation project, not a fixture set.

### 1.5 Conclusion on the reference (the honest answer to the brief's question)

The brief's framing ("diff vs CSS / a community-server reference implementation, *or* a hand-built
reference evaluator, *or* the expected-decision fixtures") resolves to **three usable oracles**,
which the design combines:

- **(a) a JS reference engine** — `@solidlab/policy-engine` (WAC + ACP) and/or `@solid/acl-check`
  (WAC). A genuine *cross-language, cross-implementation* check; pre-1.0 / implementation-truth
  caveats apply (§1.2). **Recommended as a strong second oracle** (`sq-t58w.2d`).
- **(b) a hand-built in-repo Rust reference evaluator** — a compact procedural reading of the spec
  in a *different paradigm* than the engine's N3 rules; **no JS toolchain**, trivially deterministic,
  in-repo today. **Recommended as the primary oracle** (`sq-t58w.2b`).
- **(c) the existing expected-decision table** — the `Expect` tables *already in-repo*; not a new
  oracle but the **third leg** of a three-way agreement check (§2). The CTH corpus is *not* a
  drop-in fixture set (§1.4); the in-repo `Expect` tables are the de-facto decision table.

The honest correction to the scope/parent records: a server-free JS WAC/ACP **decision** engine
*does* exist (the records called this "research-open / drive CSS"). The recommendation is **(b) as
the in-repo floor, (a) as a strong cross-language second oracle, (c) as the third cross-check** —
not "no JS option exists" and not "the only JS option is heavyweight CSS DI".

---

## 2. Determinism — a 0-divergence ratchet, the in-repo analogue of the parity floors

The existing parity floors are deterministic because they are a `const` a test asserts over a
fixed, in-repo scenario table — no clock, no network, no external corpus version
(feasibility record §3). The differential oracle inherits that property by construction when built
as recommended (option (b)):

1. **Fixed inputs.** The corpus is the *same* `corpus()` function the parity test uses (re-exported
   so there is one source). Inputs are the emitted `.acl`/`.acr` N-Quads + the `Expect` tuples —
   pure data, byte-stable.
2. **No network / no clock / no Docker.** The reference evaluator is in-process Rust over the same
   `Graph`; `Session.now` is `None` for both engines (WAC has no time grant; the one ACP
   time-window grant `sq-0q7n` is not in this corpus and, if added later, must be fixed-clocked —
   the determinism contract carries forward).
3. **Three-way agreement, fail-closed.** For every `(scenario, request)` the test computes three
   decisions — **engine** (`AuthIndex::accessible`), **reference evaluator** (option (b)), and the
   **declared expectation** (the hand-table) — and records a *divergence* if any two disagree. A
   request the reference evaluator cannot classify counts as a **divergence** (fail-closed), never
   as silent agreement.
4. **A `0`-divergence ratchet that may only tighten.** The test asserts
   `divergences == 0` and prints the runner line
   `WAC/ACP differential pairs N / divergences 0 (floor 0)` in the SHACL/geo/WAC/ACP shape, so the
   same belt-and-braces grep can re-check it. The floor is `0`; it can never be relaxed (a `>0`
   floor would be laundering a known disagreement into green — forbidden).
5. **One source of corpus + floor.** The scenario corpus and the `N`-pairs count derive from the
   existing corpus functions; the `0` floor is a `const` next to them. No second copy to drift.
6. **No fabrication into evidence.** A differential-parity number enters
   [`paper-evidence.json`](../site/src/data/paper-evidence.json) **only** once the ratchet is
   green-and-canonical in CI (per the feasibility record's contract) — and even then it is a
   *decision-parity* signal, never described as a wire result.

For the JS twin (option (a)), determinism requires only a **`package-lock.json`-pinned
`@solidlab/policy-engine` / `@solid/acl-check`** and an **offline `AuthorizationManager`** backed by
the in-memory scenario graph (no live IdP / WebID fetch). Notably this is **not** `blocked:docker`
— these are plain npm packages run under `node:test` in the existing `js/` workspace, no server or
container. The Rust reference evaluator (b) needs no JS toolchain at all; the JS twin (a) needs the
already-present `js/` Node toolchain plus two pinned dev-dependencies.

---

## 3. Where it lives + the corpus + how it gates

### 3.1 Location

A new sparq-solid integration test, **`crates/sparq-solid/tests/differential_oracle.rs`**, beside
the existing `conformance_wac.rs` / `conformance_acp.rs`. It:

- imports the **existing** corpus functions (refactored so `conformance_wac.rs::corpus()` /
  `conformance_acp.rs::corpus()` are reachable, or moved behind a small shared
  `tests/common` / `pub(crate)` corpus module — an implementation detail for the impl bead);
- runs each scenario's emitted RDF through both the engine and the reference evaluator;
- asserts `divergences == 0`.

The **reference evaluator** itself is best placed as a **test-only module** (not a public crate
surface): `crates/sparq-solid/tests/reference/{wac,acp}.rs` (or a `dev-dependencies`-only helper).
It must **not** reuse `materialize.rs` / the N3 rules — its whole value is being an *independent*
reading. It is a procedural walk of the emitted ACL/ACR graph implementing the spec directly
(WAC: nearest-ACL resolution, `acl:accessTo`/`acl:default`, agent/agentClass/agentGroup/origin,
mode set, Control-governs-`.acl`; ACP: matcher `allOf`/`anyOf`/`noneOf`, `allow`/`deny` with
deny-overrides, cumulative `accessControl`/`memberAccessControl` inheritance, fail-closed).

### 3.2 Corpus

The **same in-repo corpus** as the parity tests (12 WAC + 12 ACP scenarios today), driven by the
same `Expect` tables — *plus* a deliberately small set of **interaction / edge scenarios** the
reference evaluator is well-placed to stress (e.g. WAC nearest-ACL shadowing across three levels,
ACP `noneOf` combined with `deny`, mixed mode sets), added *first to the parity corpus* (raising
its floor) so the differential oracle never invents a scenario the parity test does not also own.
One corpus, two consumers.

### 3.3 How it gates

- The test runs in the standard `cargo nextest run --workspace` sweep (sparq-solid is a workspace
  member), so it gates on **every** CI run that runs the test shards — exactly like the WAC/ACP
  parity tests do today (verified: those floors are enforced by the in-test `assert!(pass >=
  FLOOR)` within the workspace nextest sweep).
- It prints `WAC differential pairs N / divergences 0 (floor 0)` and the ACP equivalent so a
  belt-and-braces grep gate can be added if desired (the `solid-conformance`-style grep the other
  floors use).
- **Scoreboard registration (optional, recommended for the impl bead):** add **two SUITES rows**
  to [`crates/sparq-conformance/src/scoreboard.rs`](../crates/sparq-conformance/src/scoreboard.rs)
  (`Runner::CrateTest { krate: "sparq-solid", target: "differential_oracle" }`, `floor_basis:
  "0 divergences"`) so the differential oracle appears in the consolidated index. **Correction to
  the parent record:** the feasibility doc states the WAC/ACP floors are "mirrored in
  `scoreboard.rs::SUITES`" — verified false. `SUITES` today carries only SPARQL / inference /
  SHACL-core / SHACL-SPARQL / OGC-GeoSPARQL; the WAC/ACP scenario floors live **only** as the
  in-test `WAC_SCENARIO_FLOOR` / `ACP_SCENARIO_FLOOR` consts (12/12) and are *not* in the
  `scoreboard_floors.rs` guard. The impl bead may register both the new differential rows **and**
  the existing WAC/ACP parity rows while it is there (a small, honest scoreboard fix).

---

## 4. Recommendation

Build **both** oracles, in order, against the same in-repo corpus and the same `0`-divergence
ratchet shape:

1. **Primary: the in-repo Rust reference evaluator (b), cross-checked against the existing
   expected-decision table (c)**, gated as a `0`-divergence ratchet in
   `tests/differential_oracle.rs`. Reasons:
   - Fully in-repo, deterministic, network/clock/Docker-free, CI-gated **today** — it meets the
     feasibility record's entire determinism contract with **zero** out-of-repo blockers (no JS
     toolchain even). It is the floor a green build always carries.
   - It delivers the confidence gain the scope record wanted from "a second oracle": catching
     **rule-vs-procedure drift**, because sparq-solid decides by **declarative N3 rules** and the
     reference reads the spec **procedurally** — genuine implementation independence within one
     language.
2. **Strong second oracle: the JS twin (a)** — `@solidlab/policy-engine` (WAC + ACP) and/or
   `@solid/acl-check` (WAC), run under `node:test` in the existing `js/` workspace over the *same
   emitted RDF*. This is now recommended (not a weak afterthought) because, contrary to the parent
   records, a **server-free JS decision engine exists** (§1.2–1.3), so the cost is two pinned
   dev-dependencies + a small `AuthorizationManager` shim — **not** Docker/DI scaffolding. It adds a
   genuinely *cross-language, cross-implementation* check that (b) cannot. Its honest limits keep it
   *second*, not the floor: `@solidlab/policy-engine` is pre-1.0 / research-grade and is
   *implementation* truth, so a divergence is investigated (not auto-trusted as sparq being wrong),
   and the JS lane only gates where the `js/` workspace gates.

Neither oracle overclaims: both check decision parity — the same binary the paper already cites —
with stronger evidence behind it. **No wire property (`403` / `WAC-Allow`) is asserted.**

---

## 5. Phased plan (ordered future beads)

Created under parent `sq-t58w` (real bead ids below; the `.2a–.2d` tags are this record's internal
labels). Ordered, with dependencies wired in `bd`:

1. **sq-t58w.6 (.2a) — Refactor the parity corpus to a single reusable source.** Expose
   `conformance_wac.rs` / `conformance_acp.rs` `corpus()` (and the `Expect` tables) so a second test
   target can consume the identical scenarios without copy-paste. Pure refactor; the parity tests
   must stay green and the floors unchanged. *(Prereq for sq-t58w.7; no behaviour change.)*
2. **sq-t58w.7 (.2b) — Build the in-repo Rust reference evaluator + the `0`-divergence differential
   test.** `tests/reference/{wac,acp}.rs` (procedural, independent of `materialize.rs`/the N3
   rules) + `tests/differential_oracle.rs` computing the three-way diff (engine / reference / hand
   table) and asserting `divergences == 0`, printing the runner line. *(The core deliverable;
   depends on sq-t58w.6; blocks sq-t58w.3/.8/.9.)*
3. **sq-t58w.8 (.2c) — Register the differential oracle (and the existing WAC/ACP parity floors) in
   the conformance scoreboard.** Add SUITES rows + extend `scoreboard_floors.rs` so the central index
   reflects all four Solid floors and the `0`-divergence ratchet. Fixes the parent record's
   mis-statement that the WAC/ACP floors are already in the scoreboard. *(Depends on sq-t58w.7.)*
4. **sq-t58w.9 (.2d) — JS reference-engine differential twin (strong second oracle).** Add a
   `js/test/solid-differential.test.mjs` (or similar) under the existing `js/` Node workspace:
   emit the same corpus RDF, evaluate through **`@solidlab/policy-engine`** (WAC + ACP) and/or
   **`@solid/acl-check`** (WAC) — pinned via `package-lock.json`, backed by an offline in-memory
   `AuthorizationManager` shim — and assert `0` divergences vs the engine's decisions. **Not
   `blocked:docker`** (plain npm packages, no server). Cross-language second oracle; do **not**
   start before sq-t58w.7 lands. Honest caveat in the bead: `@solidlab/policy-engine` is pre-1.0 /
   research-grade, so a divergence is triaged, not auto-attributed to sparq. **This also corrects
   the existing `sq-t58w.3`'s `blocked:docker` / "JS image" premise** — the JS oracle needs only the
   in-repo `js/` Node toolchain + two pinned npm dev-deps, no Docker.

`sq-t58w.3` (the existing child: "wire the differential oracle as a CI-enforced 0-divergence
ratchet") is the *gating/CI* follow-on (now `depends on sq-t58w.7`); sq-t58w.6–.9 are the *build*.
The impl beads are the atomic units; `sq-t58w.3` consumes them.

---

## 6. Honest scope + trade-offs (no overclaim)

- **This is library-level DECISION parity, not HTTP wire conformance.** The oracle never produces
  or checks a status code or the `WAC-Allow` header; those remain PSS-side (gh-55), tracked by
  `sq-t58w.4`/`.5`. The paper keeps its current honest library-level framing; this oracle
  *strengthens the existing decision-parity claim*, it does not add a new claim category.
- **A server-free JS reference engine DOES exist** (§1.2–1.3, npm-verified) — `@solidlab/policy-
  engine` (WAC+ACP) and `@solid/acl-check` (WAC). Claiming "no JS option exists" or "only
  heavyweight CSS DI" (the parent/scope records' framing) would now be wrong. But the JS engine is
  *implementation* truth, not *spec* truth, and `@solidlab/policy-engine` is **pre-1.0 /
  research-grade** — so it is a *credible second oracle*, not an authoritative reference; a
  divergence is triaged, never auto-read as "sparq is wrong".
- **Independence caveat for option (b).** A hand-built reference evaluator authored in the same
  repository carries a risk of *correlated* misreading of the spec with the engine. Three
  mitigations: (i) it is written in a *different paradigm* (procedural vs the engine's N3 rules);
  (ii) it is cross-checked against the independently-authored hand `Expect` table (a third reading);
  (iii) the JS twin (.2d) adds a genuinely external, cross-language fourth reading. The oracle's
  value scales with how independent the reference reading actually is — (a) + (b) + (c) together are
  three distinct readings (rules / procedural-Rust / hand-table) plus a fourth external one (JS).
- **Determinism is the easy part here** (unlike the CTH route): in-process Rust over fixed data is
  trivially deterministic, and the JS twin needs only lockfile-pinned npm packages run offline — no
  Docker/external subject/IdP, the blockers the parent record flagged for the CTH route, apply to
  neither oracle.

---

## 7. Open questions for the maintainer

1. **Appetite for the JS twin (.2d).** A server-free JS decision engine *does* exist
   (`@solidlab/policy-engine` WAC+ACP, `@solid/acl-check` WAC), so the twin is low-cost (two pinned
   npm dev-deps + a shim, no Docker). Build it as the recommended second oracle, or is the in-repo
   Rust reference evaluator + hand-table (b)+(c) sufficient given the JS engine is pre-1.0 /
   research-grade? (Recommendation: build it — cross-language independence is worth two npm deps.)
2. **Scoreboard scope (.2c).** Should this record's impl also backfill the existing WAC/ACP parity
   floors (12/12) into `SUITES` + the floors guard (fixing the parent record's mis-statement), or
   keep that as a separate hygiene bead?
3. **Corpus growth.** The differential oracle is most valuable on *interaction* scenarios (multi-
   level inheritance, `noneOf` + `deny`, mixed modes). How aggressively should the shared parity
   corpus grow (raising 12/12) to feed it, vs. keeping the corpus minimal-per-construct?

## Cross-references (do not duplicate)

- Parent feasibility record
  [`research/solid-cth-wire-conformance-feasibility.md`](./solid-cth-wire-conformance-feasibility.md)
  §3 (determinism contract this record inherits) and §5 (which names this oracle).
- Scope record [`research/sparq-solid-scope.md`](./sparq-solid-scope.md) §4 item 3 (the "CSS
  differential oracle — still not started" item this record designs).
- Engine + parity corpus: `crates/sparq-solid/src/{materialize,authindex,wac_conformance,
  conformance}.rs`, `crates/sparq-solid/rules/*.n3`,
  `crates/sparq-solid/tests/conformance_{wac,acp}.rs`.
- Floors + scoreboard: the in-test `WAC_SCENARIO_FLOOR` / `ACP_SCENARIO_FLOOR` (12/12);
  `crates/sparq-conformance/src/scoreboard.rs` `SUITES` (currently *without* the Solid rows — see
  §3.3 correction).
- Boundary: gh-55 (PSS owns the HTTP-shaped outputs); parent bead **sq-t58w**, child `sq-t58w.3`.
- External reference engines (npm-verified):
  [`@solidlab/policy-engine`](https://www.npmjs.com/package/@solidlab/policy-engine) (WAC+ACP,
  `CommunitySolidServer/policy-engine`, latest `0.0.2`, pre-1.0);
  [`@solid/acl-check`](https://www.npmjs.com/package/@solid/acl-check) (WAC-only,
  `nodeSolidServer/acl-check`, latest `0.4.5`). CSS in-tree `WebAclReader`/`AcpReader` (DI-heavy,
  not recommended); `solid-contrib/specification-tests` (server-coupled KarateDSL, not a decision
  table).
