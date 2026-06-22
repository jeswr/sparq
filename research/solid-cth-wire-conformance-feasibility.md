# Solid CTH wire-conformance feasibility + phased plan

Status: **feasibility + design record** — *not* an implementation plan, *not* a commitment to
build, and explicitly **not** a change to the paper's current honest framing. <!-- [OPUS-4.8] sq-t58w -->

This record answers one question for the WAC+ACP paper (paper B4,
[`site/papers/solid-acl-conformance.typ`](../site/papers/solid-acl-conformance.typ), PR #754):
**can we honestly lift the paper from a *library-level* decision-parity signal to an HTTP-level
*wire-conformance* result, by running sparq-server's auth path against the Solid Conformance Test
Harness (CTH) corpus as a deterministic, CI-enforced ratchet?**

It is grounded in the actual code, not the brief. It does **not** restate the architecture
records it depends on:

- [`research/sparq-solid-scope.md`](./sparq-solid-scope.md) §4 (the conformance scoping — the
  source this record extends);
- [`research/solid-access-control-design.md`](./solid-access-control-design.md) (the WAC/ACP
  support matrix, the §2.4 reasoner/content boundary, the threat model);
- the paper itself and its evidence binding
  ([`site/src/data/paper-evidence.json`](../site/src/data/paper-evidence.json)
  `conformance.solid_wac_floor` / `conformance.solid_acp_floor`, both `environment=canonical`).

## Bottom line up front

**Building CTH wire conformance for *sparq-server* is not worth doing for this paper, because
sparq-server is the wrong subject.** The CTH drives a **complete Solid server** — Solid-OIDC
accounts, pod provisioning, the LDP resource lifecycle, and the `WAC-Allow` header — none of
which sparq-server has, *by deliberate architectural design* (gh-55: the HTTP-shaped auth outputs
live in the production Solid server / PSS, with sparq-solid as an authorization *oracle*, never
the HTTP gate). The gap is not "a missing header" — it is a **whole missing surface plus a
missing identity/lifecycle stack**, and closing it would mean building (most of) a Solid server
inside sparq-server, against the opt-in-lean-core house rule.

The realistic, honest, in-scope strengthening is the one the scope record already names: a
**differential oracle against a JS reference WAC/ACP evaluator** (the existing decision-parity
corpus, re-checked against Community Solid Server / a reference authorizer), which raises
confidence in the *content* the paper already claims without overclaiming a wire result. That
path is `blocked:docker` (or a pinned JS toolchain) but is a tractable second oracle.

**The paper must keep its current library-level framing until a real wire ratchet exists**, and
no CTH-derived number should be written into `paper-evidence.json` as `environment=canonical`
until it traces to a deterministic, CI-enforced run. The current paper is *already correctly
caveated* (verified below) — no edit is needed.

---

## 1. The entry-point gap — what sparq-server's auth path exposes today

I read the server auth path directly
([`crates/sparq-server/src/http.rs`](../crates/sparq-server/src/http.rs),
[`status_contract.rs`](../crates/sparq-server/src/status_contract.rs),
[`access_audit.rs`](../crates/sparq-server/src/access_audit.rs),
[`main.rs`](../crates/sparq-server/src/main.rs)). The finding is unambiguous.

### What exists: a bearer-token gate, not an authorization engine

sparq-server's *entire* HTTP authorization surface is `auth_check` / `auth_gate` in `http.rs`
(`sq-zcby` / `sq-cxk5`). It is a **single shared-secret bearer-token gate**:

- A token may be configured (`--auth-token`). Writes are gated whenever a token is set; reads are
  gated only additionally under `--auth-token-read`.
- A gated request without the exact token (constant-time compared) is refused with **`401
  Unauthorized` + `WWW-Authenticate: Bearer` + `Cache-Control: no-store`**, byte-identical for a
  missing vs a wrong token (so it leaks nothing).
- That is the *only* auth status the server emits. There is **no per-agent, per-resource,
  per-mode decision** anywhere on the HTTP path.

Crucially, **sparq-server does not depend on sparq-solid** (verified: no `sparq-solid` entry in
`crates/sparq-server/Cargo.toml`, no `use sparq_solid` in the source). `materialize_wac` /
`materialize_acp` / `AuthIndex::accessible` — the engine the paper's parity corpus exercises —
are **never called from the server**. The server's only contact with a WebID is as a *trusted
forwarded header* consumed for **audit attribution only** (`access_audit.rs`
`Actor::from_session` / `forwarded_webid`): when an operator opts into trusting an
`--audit-webid-header` set by a fronting auth layer, the audit record attributes access to that
WebID. It is never an enforcement input, and the recorded decision is derived purely from the
response status (a `401` is `Deny`, anything else is `Allow` — `audit_access_finish`).

### What the CTH corpus asserts on — and where the `403` / `WAC-Allow` are

The CTH asserts on HTTP-shaped authorization outputs that have **no entry point on this server**:

| CTH asserts on | sparq-server today |
| --- | --- |
| `401` vs **`403 Forbidden`** for authenticated-but-forbidden | only `401` (bearer gate). The one `403` the server *does* emit is the `service`-feature SERVICE-egress allowlist / SSRF refusal (`status_contract.rs` line 47) — **unrelated** to WAC/ACP; it is a policy refusal on outbound federation, not a resource-access decision. |
| **`WAC-Allow`** header (the `user="…"`/`public="…"` granted-mode split) | **absent** — grep finds the string only in doc comments describing why it is PSS's, never emitted. |
| `WWW-Authenticate` on the auth challenge | present (`Bearer`), but driven by the token gate, not a WAC challenge. |
| LDP resource lifecycle (`PUT`/`POST`/`DELETE`/`PATCH`, slugs, containment, ETags) under access control | not modeled on the SPARQL endpoint at all (these are PSS/LDP concerns, scope record §intro). |

**Verdict on the gap (question 1):** it is **not a missing header and not a missing
status-code mapping**. It is a **whole missing surface**: there is no resource-access
authorization decision on the HTTP path at all (only a shared-secret gate), no `WAC-Allow`
emission, and — underneath even that — no per-agent identity to decide *for*. Wiring sparq-solid
into the server's request path as an enforcement decision would itself be a new surface, and a
**deliberate reversal of the gh-55 design boundary** that keeps security-critical enforcement in
vetted PSS TypeScript. This is the load-bearing correction to any premise that the gap is small.

---

## 2. The CTH corpus — what it is and whether it can target a local sparq-server

The Solid Conformance Test Harness lives at `solid-contrib/conformance-test-harness` (the runner)
and `solid-contrib/specification-tests` (the test corpus). Surveyed via its README/USAGE and the
published Docker image `solidproject/conformance-test-harness`. Key facts:

- **It is a server-over-HTTP harness.** Tests are written in **KarateDSL** (a Gherkin/BDD HTTP
  DSL) + JavaScript; the harness drives a running Solid server and asserts on real HTTP
  responses, including the `WAC-Allow` header and access-control protection behaviours.
- **The subject must be (most of) a real Solid server.** The harness's `target` points at a
  test-subject description, and the subject must provide: **two registered user accounts**
  (`USERS_ALICE_WEBID`, `USERS_BOB_WEBID`) on a compatible **Solid Identity Provider**
  (`SOLID_IDENTITY_PROVIDER`), a **test container** where alice has full control
  (`TEST_CONTAINER`), and **pod hosting** with resolvable WebID documents. Authentication is via
  one of four flows (client credentials, refresh tokens, session login `LOGIN_ENDPOINT`, or local
  registration `USER_REGISTRATION_ENDPOINT`, currently CSS-only).
- **Network / runtime dependencies:** the **Docker image** is the supported way to run it; it
  expects **HTTPS with valid certs** (or `ALLOW_SELF_SIGNED_CERTS=true`) and DNS for the subject
  host. The corpus is **fetched from `solid-contrib/specification-tests`** (the Docker build bakes
  in the latest release).

**Can it target a local sparq-server?** **No — not as a meaningful conformance run.** sparq-server
has no Solid-OIDC IdP, no account registration, no pod provisioning, no LDP resource lifecycle,
and no `WAC-Allow`. The CTH would fail at the very first setup step (account/credential
acquisition), long before reaching any authorization assertion. To make it pass we would have to
stand up the entire Solid-server front *around* sparq-server — i.e. PSS. So the **only** subject
against which a CTH wire-conformance run is meaningful is **PSS (with sparq backing its
triplestore)**, conformance-tested *through PSS* — which is exactly what gh-55 says, and is
outside this repository.

**Dependency flags (question 2):**

- Any CTH run is **`blocked:docker`** (the supported runner is the published Docker image; the
  corpus is fetched at image-build time) and effectively **`blocked:ec2`** for a hosted, networked
  test-subject + IdP.
- It is **also `blocked:external-subject`**: the *only* valid subject is a full Solid server, not
  this repository's artifacts — so this is not work this repo can land end-to-end at all.

---

## 3. Deterministic-ratchet design (the part that *would* be reusable)

*Even though the CTH-against-sparq-server route is the wrong subject*, the question of **how a
pass-count becomes a deterministic, CI-enforced ratchet** is worth designing, because the answer
(a) governs the realistic JS-differential-oracle path in §5, and (b) states the bar any future
wire number must clear before it can enter `paper-evidence.json`.

### How the existing ratchets stay deterministic (the model to copy)

The Solid library-level ratchet is the right template. `conformance_wac.rs` /
`conformance_acp.rs` each pin a `const …_SCENARIO_FLOOR` and print
`<WAC|ACP> scenarios pass N / fail M (floor F)`; the `solid-conformance` CI job re-greps that line
and fails if `N < floor` (belt-and-braces, mirroring the SHACL/geo jobs). The floor is mirrored in
`crates/sparq-conformance/src/scoreboard.rs::SUITES` and held in lock-step by
`tests/scoreboard_floors.rs`. The number is `environment=canonical` in `paper-evidence.json`
**because it is a const a test asserts over a fixed in-repo scenario table** — no clock, no
network, no external corpus version.

### What would make a CTH-style pass-count deterministic vs flaky

A KarateDSL/HTTP pass-count is **inherently a flaky candidate** unless every non-determinism is
pinned. For any wire run to qualify as `environment=canonical` it must satisfy ALL of:

1. **Pinned corpus.** A specific `solid-contrib/specification-tests` git SHA, vendored or
   submoduled, never "latest" — otherwise an upstream test add/remove silently moves the floor.
2. **Pinned subject + pinned harness image** (a CTH image digest), so the same inputs always
   produce the same assertions.
3. **Deterministic subject state.** Account/pod/ACL fixtures provisioned identically each run; no
   wall-clock-dependent ACLs (the WAC/ACP engines have a time-window grant — `sq-0q7n` — that must
   be fixed-clocked).
4. **Network isolation.** No live IdP fetch / no remote WebID resolution during the assertion
   phase; everything local to the test container.
5. **A floor, never an exact count.** Report `pass N (floor F)`; gate `N >= F`; F may only rise.
   Skips/errors must count as **non-pass** (fail-closed) so a harness setup failure can never be
   laundered into a green ratchet.
6. **No fabrication into `paper-evidence.json`.** The number enters the evidence file *only* once
   it traces to a CI job that actually executed the pinned run and emitted the parsed line —
   exactly as `solid_wac_floor` traces to its `const`. Until then the paper cites nothing.

**The determinism verdict:** a CTH-over-HTTP pass-count *can* be made deterministic in principle
(1–5), but every one of those pins is **outside this repo** (Docker, external corpus, a full Solid
subject). So even the deterministic version is `blocked:docker` + `blocked:external-subject` and
cannot be a *this-repo* CI ratchet. By contrast, the **JS-differential oracle** (§5) needs only a
pinned JS toolchain + the *in-repo* corpus, so it can satisfy 1–6 inside this repo's CI (its only
blocker is the JS/Docker toolchain, not an external subject).

---

## 4. Phased plan (ordered, atomic future beads)

The plan is split so the honest, in-repo-feasible work (the JS differential oracle) is separable
from the out-of-repo wire-conformance work (which is recorded as a decision/spike, not a build
commitment). **Server-touching beads are flagged: `sparq-server` is serialized — only one branch
may touch its hot `http.rs` auth path at a time, so these must not run concurrently with each
other or with any other open server-auth branch.**

The beads created by this record (all parented under `sq-t58w`, ordered):

1. **sq-t58w.1** *(decision/spike — `blocked:docker` + `blocked:external-subject`)* — Record the
   **CTH-subject decision**: the only valid CTH wire-conformance subject is a full Solid server
   (PSS), not sparq-server; capture the test-subject/IdP/Docker dependency matrix so it is never
   re-litigated. No code. This is the cheap, correct first step (closes the "can sparq-server pass
   the CTH?" question with a documented *no*).
2. **sq-t58w.2** *(research — feasible in-repo, the realistic strengthening)* — Design the **JS
   reference-evaluator differential oracle**: run the existing `conformance_wac.rs` /
   `conformance_acp.rs` scenario corpus through Community Solid Server's WAC/ACP authorizer (or a
   pinned reference) and diff the `(agent, client, mode, resource) -> allow/deny` decisions. Sizes
   the JS-toolchain cost and pins the determinism contract from §3. Supersedes/absorbs the
   scope-record §4 "CSS differential oracle (still not started)" item — link it.
3. **sq-t58w.3** *(impl — `blocked:docker`; depends on sq-t58w.2)* — Wire the differential oracle
   as a **deterministic, gated, fixtured ratchet**: pinned JS image/digest + in-repo corpus + a
   `divergences <= 0` floor (a `0`-divergence ratchet that may only tighten), reported in the
   `solid-conformance` job's shape and (if it lands) registered as a scoreboard row. Only after
   this is green-and-canonical may a "differential-parity" number enter `paper-evidence.json`.
4. **sq-t58w.4** *(decision — `area:sparq-server`, SERIALIZED server-touch; do NOT auto-run)* —
   **Decision record only**: whether sparq-server should *ever* expose a WAC/ACP authorization
   decision over HTTP (the `403` + `WAC-Allow` surface), or whether that stays in PSS forever per
   gh-55. Default recommendation: **stays in PSS** (the opt-in-lean-core + vetted-TS-enforcement
   house rules both point that way). This bead exists so the boundary is an explicit, reviewable
   decision rather than an implicit one — it must NOT be started as an implementation without the
   maintainer first accepting the boundary change.
5. **sq-t58w.5** *(spike — `area:sparq-server`, SERIALIZED server-touch; `blocked:docker` +
   `blocked:external-subject`; depends on sq-t58w.4 accepting the boundary change)* — *Only if*
   sq-t58w.4 decides the boundary should move: a minimal spike to expose a `WAC-Allow` header +
   `403` mapping behind an **opt-in feature**, driven by sparq-solid's `AuthIndex::accessible`,
   plus the CTH harness wiring needed to test it. Explicitly gated behind the decision; not a
   default-on change to core; the actual CTH run remains out-of-repo (a PSS concern). This is the
   "smallest server-side work to expose the missing HTTP outputs" the brief asked to scope —
   honestly sized as *large, opt-in, and decision-gated*, not a near-term task.

Ordering rationale: 1 (close the question) → 2 (design the in-repo realistic oracle) → 3 (land
its deterministic ratchet) are the honest, recommended path. 4 → 5 are the *only-if-the-boundary-
moves* branch, deliberately last and decision-gated so no one builds a server auth surface that
contradicts gh-55 without explicit maintainer sign-off.

---

## 5. Honest worth-it verdict + the smallest real wire floor

**Is CTH wire conformance worth doing for the paper? No, not against sparq-server — and not in
this repo at all for the foreseeable future.** Three honest reasons:

1. **Wrong subject.** The CTH grades a full Solid server; sparq-server is a SPARQL endpoint with a
   bearer gate. Passing it means building PSS, which is out of repo and against the lean-core /
   vetted-TS-enforcement house rules.
2. **Out-of-repo dependencies.** Every meaningful CTH run is `blocked:docker` +
   `blocked:external-subject` (a real IdP + accounts + pod provisioning). A this-repo CI ratchet
   over it cannot exist.
3. **The paper does not need it to be honest.** The paper already states the limitation up front
   (verified: `solid-acl-conformance.typ` lines 30–32, 105–109 — "explicitly *not* HTTP /
   Conformance-Test-Harness wire conformance, which has no library entry point here"), and binds
   only `environment=canonical` library-floor numbers. It is *already* correctly framed; nothing
   needs to change, and nothing should be added that implies a wire result.

**The smallest real wire-conformance floor we could honestly claim:** **none, today** — there is
no wire-conformance artifact, so the only honest claim is the current library-level decision
parity. The *next* honest, achievable strengthening is **not** a wire floor at all but a
**zero-divergence differential-oracle floor** (sq-t58w.2/.3): "the library decision matches a JS
reference WAC/ACP evaluator on the pinned corpus, with `0` divergences, CI-enforced." That
raises confidence in the *content* the paper already claims without ever asserting an HTTP-shaped
property the engine does not produce. A genuine wire floor would require PSS-side CTH runs and
would be PSS's result to publish, not sparq's.

**Do not** present any future CTH or differential number as a wire-conformance result until a
deterministic, CI-enforced run (per §3) traces it into `paper-evidence.json`. Until then the
paper keeps its current, honest library-level framing.

---

## Open questions for the maintainer

1. **Is the gh-55 boundary fixed?** Should sparq-server *ever* expose a WAC/ACP HTTP decision
   (`403` + `WAC-Allow`), or is enforcement permanently PSS's (the current design)? sq-t58w.4
   exists to capture this as an explicit decision; the recommendation is "stays in PSS".
2. **JS-differential oracle appetite.** Is the JS-toolchain / Docker cost of sq-t58w.2/.3 worth
   it for the confidence gain, given the paper is already honestly framed without it?
3. **PSS-side CTH ownership.** If a real wire-conformance result is wanted, it is a PSS deliverable
   (sparq backing the triplestore). Is that in scope for any sparq paper, or purely PSS's to
   publish and merely *cite*?

## Cross-references (do not duplicate)

- Scope record [`research/sparq-solid-scope.md`](./sparq-solid-scope.md) §4 (this record extends
  it; the "CSS differential oracle / still not started" item is absorbed into sq-t58w.2).
- Paper B4 [`site/papers/solid-acl-conformance.typ`](../site/papers/solid-acl-conformance.typ) and
  its `environment=canonical` evidence in
  [`site/src/data/paper-evidence.json`](../site/src/data/paper-evidence.json).
- Library harness + ratchet: `crates/sparq-solid/src/wac_conformance.rs`,
  `crates/sparq-solid/tests/conformance_{wac,acp}.rs`,
  `crates/sparq-conformance/src/scoreboard.rs` (`SUITES`), the `solid-conformance` CI job.
- Server auth path: `crates/sparq-server/src/http.rs` (`auth_check`/`auth_gate`/`unauthorized`),
  `status_contract.rs`, `access_audit.rs`.
- Parent: gh-55 (the PSS/oracle boundary); parent bead **sq-t58w**.
