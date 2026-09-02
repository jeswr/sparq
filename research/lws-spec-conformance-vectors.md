# Wiring the `jeswr/lws-spec` conformance contract into sparq (sq-gg0qq.6) [OPUS-5]

> **Status: DESIGN RECORD — not an implementation, and not a ratified plan.**
> The bead asks for three things at once: re-home the `feat/lws*` branches into a
> `crates/sparq-lws` that does not exist yet, wire ~157 language-neutral test vectors,
> and make the N3 access-decision oracle a CI conformance gate. §1 records where the
> bead's premise does not survive contact with the sources; §2 states what the contract
> actually is; §3 and §4 settle the two questions the bead explicitly delegates
> ("vendored fixtures or pinned submodule — decide + document", and whether the oracle
> "needs node/EYE"); §7 is the phased plan.
>
> Bead: **sq-gg0qq.6** · Issue: **#2746** · Blocked-by: **#2747** (`sq-gg0qq.4`,
> [`lws-3-crate-split.md`](./lws-3-crate-split.md)) · Parent: **#2572** / `sq-gg0qq`.
>
> Author: Claude Opus 5. Every count, SHA and verdict below was taken from the sources
> themselves at sparq `8ad990e6`, not quoted from the bead or from a prior record.
> §4 reports a **measured** result and states its apparatus so it can be re-run. **No
> timings appear in this document.**

---

## 1. Premise corrections

The bead names three branches in `jeswr/solid-server-rs` to read. **Only one exists.**
`git ls-remote --heads` on that repository returns twelve heads; the only `lws` head is:

| Bead names | Exists as a head | Evidence |
|---|---|---|
| `origin/feat/lws` | **yes** — `f00c97c7b0d65f74ec2755ca259cfd25b7d2c348` | `git ls-remote --heads` |
| `origin/feat/lws-conformance-gaps` | **no** | not among the twelve heads |
| `origin/feat/lws-step8-ab` | **no** | not among the twelve heads |

This is not a loss. `feat/lws` is 20 commits ahead of that repository's `main` and
already **contains** the work the two missing names describe — `5952ead feat(lws):
step-8 alignment A+B — DPoP-SK over LWS (pop_session) + a2a-rdf AgentInteractionService`
is the step-8-A/B content, and the conformance-gap commits (`77e2163`, `4e42b3d`,
`75b6dbe`, `f00c97c`) sit above it. So the re-homing source is **one branch**, not three,
and the branch is self-consistent. Its footprint against that repository's `main` is 36
files: a net-new `src/lws/` module of eight files (`auth`, `container`, `linkset`, `mod`,
`pin`, `problem`, `rar`, `transform`), four ADRs (`decisions/0004`–`0007`), three
net-new integration-test files, and edits to `app`/`auth`/`error`/`ldp/handler`/`store`.

**`sparq-lws` does not exist, and the record that owns its creation says not to create
it yet.** [`lws-3-crate-split.md`](./lws-3-crate-split.md) §5 is explicit: `sparq-lws`
should be created "only if it has a compiling reason to exist … Prefer to create it in
the phase that gives it content (§6, phase 5)" — and that record's §6 phase 5 is *this*
bead. The circularity is real but it resolves in one direction only: phase 5 depends on
phase 3 (`sparq-solid-server` created), which depends on phases 1–2, and the whole chain
is gated on that record's §7 open question 1, **which is still open**. There is therefore
no `crates/sparq-lws` to put profile code into today, and creating one here would
front-run a maintainer decision on the crate partition.

**`sparq-lws-core` has no JLWS surface at all.** A search for `jlws`, `lws+json`, or
`w3id.org/jeswr/lws` across `crates/sparq-lws-core/src` and `tests` returns nothing. The
crate implements Solid/LDP + WAC; JLWS is a different protocol with a different container
model, a different discovery mechanism, and a different access-control profile. The
vectors are therefore **not** a measurement of the current crate that happens to be
missing a runner — the great majority of them describe a surface that has not been
written in this repository yet.

**`lws-spec` and `lws-ucs` are no longer unresolved.** `crates/sparq-lws-core/src/lib.rs`
currently states that both names' "Location and content unknown". Both resolve:
`github.com/jeswr/lws-spec` is the contract this bead names, and `github.com/jeswr/lws-ucs`
is a personal fork of the W3C LWS Use Cases document (its README points at
`w3c.github.io/lws-ucs/spec/`), i.e. **not** a jeswr-authored normative spec and not a
contract for this crate. That lib.rs note also sets the bar for promoting a name out of
UNRESOLVED — "a maintainer-confirmed reference **plus** an executed spec vector". Locating
a repository satisfies neither half, so this change records the location and explicitly
keeps the pinning posture unpinned. Do not read the accompanying lib.rs edit as a promotion.

**The count is exact, not approximate.** The bead says "~157". The suite's
`test-vectors/manifest.json` declares `caseCount: 157`, the ten per-suite manifests declare
17 + 14 + 7 + 8 + 36 + 14 + 24 + 12 + 21 + 4 = 157, and 157 `case.json` files exist. The
machine-readable index is internally consistent. Two prose figures in the suite's own
`README.md` are stale against it (it says "154 cases" and gives `access-grants` as 21
where the manifest says 24) — worth knowing so a runner is written against the manifest
and never against the prose, but it is an upstream matter and, per the maintainer
directive, this record raises nothing upstream.

## 2. What the contract actually is

`jeswr/lws-spec` at `ffaea0497de41cd709a742e0c4a90831a500fd97` carries four artifacts that
matter here.

**The vectors.** 157 cases in 10 suites, each a self-describing `case.json` with
`{id, title, spec, clauses, level, operation, input, expected, source}` and no dependency
on any implementation. 154 are `MUST`, 3 are `SHOULD`. Crucially they are **not** all
HTTP exchanges — they bind **16 abstract operations**:

| Operation | Cases | Operation | Cases |
|---|---|---|---|
| `http-exchange` | 74 | `verify-webhook-signature` | 5 |
| `evaluate-access` | 19 | `validate-access-document` | 5 |
| `validate-access-token` | 15 | `enforce-authorization-details` | 4 |
| `validate-storage-description` | 7 | `validate-notification-envelope` | 4 |
| `transform-representation` | 6 | `verify-realm-containment` | 3 |
| `evaluate-token-exchange` | 5 | `decode-webauthn-assertion-bundle` | 2 |
| | | `validate-as-metadata`, `discover-service`, `evaluate-pop-session-offer`, `evaluate-transform-offer` | 2 each |

That distribution is the single most important planning fact in this record. Roughly half
the suite is a **pure-function** contract — decide, validate, transform, verify — that a
library can bind with no server, no socket and no fixture container. The other half
(`http-exchange`) needs a booted server and a seeded state, which is the expensive half and
the half with no in-repo implementation to point at.

**The shapes.** Five SHACL documents in `shapes/` (`jlws-access-documents`,
`jlws-container`, `jlws-notification`, `jlws-storage-description`, `jlws-subscription`) —
the per-document-class validation the bead refers to.

**The executable semantics.** `semantics/access-decision.n3` (377 lines) is the *normative
definition* of the `evaluate-access` decision function, with `access-decision.query.n3` as
the decision-extraction query: permit iff at least one `ax:permittedBy` triple is derived,
deny iff none is (closed-world absence at decision time). It uses exactly eight builtins —
`log:collectAllIn`, `list:length`, `log:uri`, `log:notEqualTo`, `string:startsWith`,
`string:endsWith`, `string:notMatches`, `string:notLessThan`. The suite's own oracle
(`test-suite/tools/oracle-access.mjs`) executes it under EYE via the `eyereasoner` npm
package pinned at `21.1.13` (Node ≥ 20) and diffs the derivation against every
`evaluate-access` vector; that diff is part of the spec repository's own gate, so the
vectors and the rule set cannot silently disagree **upstream**. §4 is about whether we
need that toolchain here.

**The honest inverse.** `test-vectors/GAPS.md` catalogues the normative statements
deliberately **not** pinned by a vector, with a reason class for each
(network/trust, stateful/temporal, behavioural emission, deployment-policy,
envelope/under-specified, companion-planned, covered-elsewhere, vectorable-deferred). Any
claim we make from a green suite must carry the same qualification the suite itself does:
passing is **necessary but not sufficient** for conformance. There is also `formal/tla/`
(TLC-checked models for revocation, conditional update, containment) which is design-level
model checking of the *spec text*, not of an implementation, and is out of scope here.

One more property of the suite that has to be stated because it inverts the usual
direction of trust: **no reference implementation of JLWS exists**. Except for
`evaluate-access`, every expected outcome is *derived from the normative text*, not
extracted from a running server. The suite's README says so, and draws the correct
consequence — until an implementation passes it, a vector may embody a misreading of the
spec, and a disagreement must be adjudicated against the spec text with the loser fixed.
The bead's "where crate and spec disagree, the SPEC WINS" is the right default, but it is
a default, not an axiom: the adjudication step is real work and belongs in the plan.

## 3. Decision: fetch at a pin. Not vendored, not a submodule

The bead offers "vendored fixtures or pinned submodule — decide + document". **Neither.
Fetch at a pinned commit into a gitignored path**, which is this repository's existing,
five-times-repeated answer to exactly this question.

The decisive constraint is licensing. **`jeswr/lws-spec` carries no `LICENSE` file** — a
whole-tree search for `*licen*` / `*copying*` finds nothing, and the root `README.md` says
nothing about licensing either. The only licence statement anywhere in the tree is
`"license": "MIT"` inside `test-suite/package.json`, which is a `"private": true` package
manifest scoped to the Node runner, not a grant covering `test-vectors/` or `semantics/`.
Absent an explicit grant, the default is all-rights-reserved, and **committing 3.7 MB of
that tree into a repository published as `MIT OR Apache-2.0` would be redistributing it
without one**. A git submodule is better — it records a pin without copying bytes — but it
adds a repo-wide clone-time cost and a `.gitmodules` surface for a fixture set that only
one crate's tests read, and this repository currently has no submodules at all.

The precedent is unambiguous. `scripts/fetch-conformance.sh` pulls `w3c/rdf-tests` at a
pinned commit into `/tests/w3c/`, which `.gitignore` line 29 marks "never committed", using
the shared `retry_git_clone_pinned` helper from `scripts/lib/fetch-retry.sh`. Four sibling
scripts (`fetch-inference-suites.sh`, `fetch-jsonld-tests.sh`,
`fetch-jsonld-framing-tests.sh`, `fetch-odrl-suite.sh`) do the same for other suites. A
sixth script for the JLWS vectors is the low-surprise, zero-new-mechanism choice, and it
sidesteps the licence problem entirely because nothing is redistributed.

The pin should be the **vectors' repository commit**, recorded in the script and bumped
deliberately — the same discipline the existing scripts state ("pass-rates are only
comparable across runs when the suite revision is fixed"). Note that this is a different
pin from the one inside the data: `manifest.json` records `specSource: lws-spec@59da847`,
the spec-text commit the vectors were last reconciled against, which today is three
commits behind the repository head; only one of those three touched `semantics/` or
`test-vectors/`, and it was the reconciliation bump itself. Both pins should be recorded,
and the runner should assert `manifest.schemaVersion == 1` and fail loudly on drift rather
than silently skipping cases it does not recognise.

## 4. Measured: the N3 oracle does **not** need node or EYE

The bead hedges that "oracle re-derivation can be an opt-in CI lane if it needs node/EYE".
It does not. **`sparq-reason` reproduces all 19 `evaluate-access` decisions in-tree** —
but only when the rule set is driven correctly, and the incorrect drive fails **open**.

### 4.1 What was run

The suite's own encoder was reused so the N3 input is byte-identical to what the upstream
EYE oracle consumes: `encodeCase` is an exported function of `oracle-access.mjs`, and the
CLI half of that file is guarded by an `isMain` check, so it can be imported as a library.
With a stub standing in for the unavailable `eyereasoner` package (the encoder never calls
it), all 19 `evaluate-access` cases were encoded to N3. Each was then concatenated with
`semantics/access-decision.n3` and projected through `semantics/access-decision.query.n3`
by an out-of-tree probe crate depending on `crates/sparq-reason` by path, built with
Rust 1.88.0 (the workspace pins 1.97.1 in `rust-toolchain.toml`; that toolchain is not
installed on this box, and 1.88.0 is the crate's declared `rust-version`).

### 4.2 Result

| Driver | Agrees with vector | Disagrees | Direction of the disagreement |
|---|---|---|---|
| `reason_n3_query_terms` — one document, single pass | 17 / 19 | 2 | **permit where the spec says deny — fail open** |
| `reason_n3_stratified` — four strata | **19 / 19** | 0 | — |

The two single-pass failures are `access-grants/prohibition-denies-despite-permission` and
`access-grants/unmet-obligation-fail-closed`, i.e. precisely the two cases governed by the
rule set's decision-time composition rules N and O.

The cause is neither a builtin gap nor an encoding difference, and it is worth stating
exactly because it is the load-bearing detail for anyone wiring this. All eight builtins
the rule set uses are implemented by name in `crates/sparq-reason/src/n3/mod.rs`, and
probing the intermediate predicates on those two cases shows `ax:prohibitedIn` and
`ax:obligationUnmetIn` **are** derived — one answer each. The failure is that rule D's
negation-as-failure guard, spelled `( true { ?req ax:prohibitedIn ?g } ?LP )
log:collectAllIn _:sdp . ?LP list:length 0 .`, still sees an empty list and so derives
`ax:permittedBy` anyway. This is documented engine behaviour, not an engine defect:
`reason_n3_stratified`'s rustdoc states that the non-monotonic premise operators
"are only reliable over predicates FULLY PRESENT before their stratum starts", and directs
callers to "derive such predicates to a fixpoint in an earlier stratum and negate/aggregate
over them in a later one". A single-document run of `access-decision.n3` negates over
predicates derived by the same document, which is outside that envelope.

Splitting the rule set at its own section banners into four strata — `{P, A, B, C, K}`
(profile facts, action satisfaction, assignee, target coverage, constraint evaluation) →
`{M}` (rule matching) → `{N, O}` (prohibition/obligation composition) → `{D}` (the permit
derivation) — and running them through `reason_n3_stratified` yields 19/19. The strata
boundaries are not invented: they follow the rule set's own dependency order, which its
header comments already spell out.

### 4.3 What follows

1. **The oracle lane is native and cheap.** No Node, no npm, no `eyereasoner`, no separate
   opt-in CI lane with a JS toolchain — a Rust test in the workspace that reads the fetched
   `semantics/*.n3` and the 19 vectors is enough. This also realises, on our side, exactly
   what the spec's `semantics/README.md` anticipates when it observes that the definitional
   evaluator is an N3 rule set because that is "the same formalism sparq executes".
2. **The stratification is a correctness requirement, not a tuning knob**, and the wrong
   choice is silent and fails open on an access-control decision. Whatever code lands must
   pin the strata split explicitly, and should carry a red-on-wrong-answer test — the
   naive single-pass drive on those two cases is a ready-made mutation check.
3. **This is an agreement measurement, not a soundness proof.** 19 cases is the whole
   `evaluate-access` vector set, not the whole decision function; the vectors are
   point-wise samples of it, as the spec's own README says. Agreement on 19 sampled points
   is evidence that the rule set is executable here with the same verdicts, and nothing
   more. It is emphatically not a claim that sparq implements JLWS access control.
4. **The encoder is unbuilt work.** The measurement borrowed the suite's JavaScript
   `encodeCase`. A Rust JSON-document → N3 encoder reproducing that mapping (documented in
   `semantics/README.md`, fail-loud on anything outside the profile's tables) is a real,
   separately-testable deliverable — and it is the piece most likely to disagree with the
   spec in a way the vectors will catch.

## 5. Where the crate and the spec already disagree

`crates/sparq-lws-core/src/authz/odrl.rs` is not a partial implementation of JLWS
`evaluate-access` that needs finishing; it is a **different function**. It is an opt-in
read-path gate over `sparq-policy` whose only action vocabulary is `odrl:read` (a single
`ODRL_READ` constant), whose verdict is `Permit | Deny | NotApplicable` composed with a WAC
decision by the caller, and whose fail-closed posture is "a grant carrying a prohibition
denies". JLWS `evaluate-access` is default-deny permit-derivation over *recorded* grants,
with five actions (`read`/`modify`/`delete` from ODRL plus `create`/`append` from the JLWS
namespace), one-directional action inclusion, trailing-slash-guarded prefix target
coverage, conjunctive fail-closed constraints, and per-grant — never global — prohibition
and obligation composition, with revocation composing structurally by the grant record's
absence rather than by a deny rule.

So the bead's adjudication rule ("the SPEC WINS, or propose a spec change") does not bite
here, because there is no conflict to adjudicate: these are two functions with different
domains that happen to share the letters ODRL. The JLWS decision function is **new code**,
and the existing gate must not be repurposed for it. Nothing in this record proposes
changing `authz::odrl`, whose own contract is pinned by
`crates/sparq-lws-core/tests/odrl_query_enforcement.rs`.

## 6. Recommendation

**Do not attempt this bead as one change, and do not create `crates/sparq-lws` yet.**

Split it at the seam §2 exposes. The pure-function half of the suite — 83 of 157 cases
across 15 operations — needs no server, no `sparq-lws` crate, and no resolution of the
crate-partition question that blocks #2747. It can be landed as a library-level,
data-driven conformance runner starting with the one operation family that already has an
executable normative definition and a demonstrated in-tree evaluator: `evaluate-access`.
The `http-exchange` half is genuinely blocked — on `crates/sparq-lws` existing, which is
blocked on the `sparq-solid-server` split, which is blocked on a maintainer decision.

The re-homing itself should be treated as what it is: a **semantic port of one branch**,
not a merge and not three. `sparq-lws-core` has already diverged forward from
`solid-server-rs` `main` (61 source files against 53, having gained `authz/odrl.rs`,
`authz/trust_admit.rs`, `store/limits.rs`, `store/sparql.rs`, `sparql_endpoint.rs`,
`clock.rs`, `reconcile_runtime.rs` and the wasm modules), so no branch merges cleanly and
none should be attempted. `feat/lws`'s four ADRs (`decisions/0004`–`0007`) should be
re-homed as `research/` records under the namespace rule
[`lws-design-records.md`](./lws-design-records.md) §2 already established, not carried as
`decisions/` paths.

**The gate should be a monotone floor, not all-green.** A suite where most cases describe
unwritten surface cannot be a green/red gate without either blocking every merge or being
declared advisory and ignored. `crates/sparq-conformance-floors` is the existing mechanism
for exactly this: a shared `pub const` floor that may only rise, enforced in the runner and
reported centrally, with the cargo edges ensuring change-based test selection cannot skip
the lane. A `JLWS_EVALUATE_ACCESS_FLOOR` starting at the measured 19 is honest, is a real
gate from day one, and ratchets as surface lands. Where a lane genuinely cannot gate — the
`http-exchange` half before there is a server — it must be **declared** in
`.github/advisory-registry.json` with `owner_bead` and `promotion_criteria`, because
`scripts/ci_summary_gate.py` treats every undeclared check-run as gating; `lws-cth.yml`
already documents that trap for this crate's sibling harness.

## 7. Phased plan (each phase = one future bead)

1. **Fetch script + pin.** `scripts/fetch-lws-spec.sh` using `retry_git_clone_pinned` into
   a new gitignored path, pinned to a `jeswr/lws-spec` commit, with both that pin and the
   manifest's `specSource` recorded in the script header. *Acceptance:* the script is
   idempotent, the destination is gitignored, and a manifest `schemaVersion` other than 1
   fails loudly. No test consumes it yet.
2. **Manifest reader + skip-when-absent harness.** Parse `manifest.json` and the ten suite
   manifests; assert the declared counts against the files found; expose cases grouped by
   `operation`. Tests skip cleanly (not fail) when the fetched tree is absent, matching the
   existing conformance crates' posture. *Acceptance:* counts reconcile at 157/10, and a
   deliberately corrupted manifest reds the test.
3. **The N3 encoder.** Rust JSON → N3 encoding of the strict-ODRL profile document shape,
   reproducing `semantics/README.md`'s mapping table, fail-loud on any action, operand,
   operator or context key outside the profile's tables. *Acceptance:* encoder output for
   all 19 `evaluate-access` inputs matches the suite encoder's output.
4. **The oracle lane** (`evaluate-access`, 19 cases). Drive `semantics/access-decision.n3`
   through `reason_n3_stratified` with the four-stratum split of §4.2; introduce
   `JLWS_EVALUATE_ACCESS_FLOOR` in `sparq-conformance-floors`. *Acceptance:* 19/19, a
   red-on-wrong-answer mutation check, **and** an explicit regression test that the naive
   single-document drive is not used — the fail-open direction is the whole risk.
5. **The remaining pure-function operations** (64 further cases, 14 operations), landed as
   separate beads per operation family and each raising its own floor: document validation
   against the five SHACL shapes, `validate-access-token`, the token-exchange and
   authorization-details family, storage-description and discovery, notification envelope
   and webhook signature, `transform-representation`. Each of these is a bead only once
   the surface it validates exists; several do not exist in this repository today, and the
   bead should say so rather than budget for a pass.
6. **Adjudication log.** A section of this record, or a sibling, recording every
   crate-vs-vector disagreement and its resolution against the spec text — since no
   reference implementation exists, a vector may be wrong, and the resolution must be
   written down either way. Per the maintainer directive, nothing is raised upstream.
7. **`http-exchange` (74 cases)** — blocked. Depends on `crates/sparq-lws` existing, which
   depends on [`lws-3-crate-split.md`](./lws-3-crate-split.md) §6 phases 1–3, which depend
   on that record's §7 open question 1 being answered.
8. **Re-home `feat/lws`** as profile code over the split crates, semantically ported one
   `src/lws/` module at a time against the now-green suite, with the four ADRs re-homed as
   `research/` records. Depends on 7.

Phases 1–4 are unblocked today and are the whole of the near-term value. Phases 5–6 follow
1–4. Phases 7–8 are gated on #2747's open question.

## 8. Open questions for the maintainer

1. **Licensing.** `jeswr/lws-spec` has no `LICENSE`. §3 avoids the problem by never
   copying bytes, but if the intent is for sparq to vendor or redistribute any part of the
   suite, the spec repository needs an explicit grant first. This record does not assume one.
2. **Is JLWS a target for sparq at all, or is the suite a contract for a future
   `sparq-lws` only?** §5 shows the existing ODRL gate is a different function; §1 shows
   the crate has no JLWS surface. If the answer is "future crate only", phases 1–4 are
   still worth landing (they cost little and give the future crate a working oracle), but
   phase 5's per-operation beads should not be opened.
3. **Which floor does the gate ratchet on** — case count, or per-normative-statement
   coverage keyed to the suite's `clauseIndex`? The latter is closer to what the suite
   measures and is what `test-suite/` reports upstream, but it is a larger runner.
4. **`sparq-lws` naming and placement** remains open per #2747 §7 Q1, and phases 7–8 cannot
   start without it.

## 9. What this record does not do

- **It does not implement anything.** No fetch script, no runner, no encoder, no floor.
  The one accompanying source change is a doc-comment correction in
  `crates/sparq-lws-core/src/lib.rs` replacing a now-false "location and content unknown"
  with the located repository, while explicitly **keeping** both names unpinned — that
  file's own bar for promotion (maintainer confirmation plus an executed spec vector) is
  not met by this record and is not claimed to be.
- **It claims no conformance.** sparq does not implement JLWS. §4's 19/19 is agreement
  between `sparq-reason` and the spec's own rule set on the 19 sampled decision points of
  one operation, measured on a work box; it is not a pass-rate, not a benchmark, and not a
  statement about any other operation.
- **It does not read the `feat/lws` diff line by line.** §1's characterisation comes from
  the commit log and `--name-status`; the semantic port in phase 8 will need the actual
  reading, module by module.
- **It raises nothing upstream**, per the maintainer directive — including the two stale
  prose counts in the suite's own README noted in §1.
