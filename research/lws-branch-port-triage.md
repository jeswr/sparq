<!-- [OPUS-5] sq-gg0qq.8: port-or-drop verdicts for three upstream solid-server-rs
     branches, plus the M2-next marker sweep. Evidence-based; no code was ported. -->
# LWS upstream branch triage — port-or-drop verdicts (sq-gg0qq.8)

> **Status: TRIAGE RECORD (decision, no implementation).** Three upstream branches named by the
> bead are triaged here to a written port-or-drop verdict, and the crate's remaining `M2-next`
> markers are swept to a deferred-work list.
>
> Bead: **sq-gg0qq.8** · Issue: **#2744** · Parent: **#2572** / `sq-gg0qq` · Blocked-by: **#2747**.
> Companion record: [`lws-design-records.md`](./lws-design-records.md) (the reconstructed ADR
> estate — §6 there is the normative description of the existence-non-disclosure family).
>
> **Outcome in one line:** all three branches are **DROP — subsumed by the import**. The
> `crates/sparq-lws-core` import snapshot already contains each branch's merged result, verified
> against the in-tree code and its tests. No code was ported, because there was nothing left to
> port.

---

## 0. Two corrections to the brief's premise

Both are load-bearing, and both change what this task could actually do.

**(a) The three branches are not sparq branches.** The brief reads as though
`origin/chore/repin-async-dns-verifier`, `origin/fix/unique-blob-keys`, and
`origin/phase-existence-non-disclosure` are branches on sparq's own remote. They are not — they
belong to the **upstream** repository [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs),
the repo `crates/sparq-lws-core` was imported from. All three were checked by exact name against
sparq's `origin` (608 remote refs) and are **ABSENT**:

```text
chore/repin-async-dns-verifier   -> ABSENT
fix/unique-blob-keys             -> ABSENT
phase-existence-non-disclosure   -> ABSENT
```

**(b) The crate is `sparq-lws-core`, not `sparq-solid-server`.** The brief says the verifier pin
should be folded "into the sparq-solid-server Cargo.toml pin". No such crate exists in this
workspace; the pin lives at `crates/sparq-lws-core/Cargo.toml:230`. (`crates/sparq-solid` is a
different crate — the ODRL/Solid vocabulary estate — and does not carry this dependency.)

### What follows from (a) — the honest limit on these verdicts

This triage had **no network access and no upstream remote**, so it could not fetch the branch tips
and could not compute a commit-level diff of branch-vs-import. The verdicts below are therefore
grounded in a different, weaker-but-checkable claim:

> **The capability each branch existed to deliver is present in the imported tree, with tests.**

That is *capability-level* subsumption, evidenced by `file:line` citations you can check. It is
**not** a proof that no line of those branches is missing. Where a branch could plausibly have
carried more than its headline capability, §4 says so rather than claiming completeness. If the
maintainer wants commit-level certainty, that needs a network-enabled pass that adds
`jeswr/solid-server-rs` as a remote and diffs each branch tip against import rev `1e555b10` — filed
as deferred item **D5** in §5.

## 1. Why "subsumed" is the expected answer, not a convenient one

The import commit is a single whole-tree snapshot:

| Fact | Evidence |
|---|---|
| The crate was imported whole from `jeswr/solid-server-rs` at rev `1e555b10` | `research/lws-design-records.md:6-8` (bead `sq-gg0qq.2`) |
| That import is one commit | `47e11a5c` — *feat(lws): import jeswr/solid-server-rs as crates/sparq-lws-core (sq-gg0qq.2)* (#1949) |
| Each of the three subjects' defining symbols was **introduced by that commit** | `git log -S` on `mint_blob_key`, `guard_post_existence_requires_read`, and the verifier rev string each return exactly `47e11a5c` |

So the three branches were merged **upstream, before** the snapshot was taken. A branch whose work
is already inside the snapshot you imported cannot be "ported" — it is already here. That is the
single mechanism behind all three verdicts, and it is why the verdicts agree.

## 2. The verdict table

| Upstream branch | Subject | Verdict | Basis |
|---|---|---|---|
| `chore/repin-async-dns-verifier` | verifier git-pin bump to the async-DNS resolver | **DROP — subsumed** | pinned rev already resolves the async DNS resolver; cargo-vet delta already recorded (§3) |
| `fix/unique-blob-keys` | blob-key collision fix (bytes integrity) | **DROP — subsumed** | unique-per-write minting implemented, fail-closed, 3 unit tests incl. a mutation check (§4) |
| `phase-existence-non-disclosure` | 404-vs-403 existence non-disclosure | **DROP — subsumed** | the V2/V4/V5/V6 family + the exhaustive byte-identical matrix are in-tree and pinned (§5 of this doc, §6 of the design record) |

No branch is a **PORT**. No code changed under this bead.

## 3. `chore/repin-async-dns-verifier` — DROP (subsumed)

**What the branch was for.** Bumping the `solid-oidc-verifier` git pin to a revision whose
WebID/issuer resolution uses an **asynchronous, SSRF-guarded** DNS resolver rather than a blocking
one — a housekeeping/hardening bump, with a cargo-vet delta for the new transitive crates.

**Why it is subsumed.** The in-tree pin is an immutable `rev` pin whose resolved dependency graph
**already contains** the async resolver:

- `crates/sparq-lws-core/Cargo.toml:230` — `solid-oidc-verifier = { git = "…", rev = "89c896249a726398b78302fd2f65eef0a82af681", features = ["network"] }`
- `Cargo.lock:4799-4812` — that package's dependency list includes `hickory-resolver`
- `Cargo.lock:2119-2141` — `hickory-proto` and `hickory-resolver` both resolve to `0.26.1`

`hickory-resolver` **is** the async DNS resolver (the Tokio-native successor to trust-dns); its
presence in the pinned verifier's dependency set is exactly the property the repin existed to
produce.

**The cargo-vet delta is already recorded**, and recorded under this bead family's own pre-flight:

| Crate | Version | Criteria | Note site |
|---|---|---|---|
| `hickory-resolver` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:722-725` |
| `hickory-proto` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:718-720` |
| `hickory-net` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:714-716` |
| `jsonwebtoken` | 10.4.0 | `safe-to-deploy` | `supply-chain/config.toml:907` |

The `hickory-resolver` note reads: *"solid-server-rs import pre-flight (sq-gg0qq.1): transitive of
solid-oidc-verifier — its DNS-pinned SSRF-guarded async resolver (0.26.1 carries the
GHSA-q2qq-hmj6-3wpp hickory-proto fix)."* So the sibling bead `sq-gg0qq.1` already did the
supply-chain half of this branch's work, deliberately.

**Stated honestly — three residuals.**

1. These are **exemptions**, not audits: `supply-chain/audits.toml` and `imports.lock` contain no
   `hickory` entry. An exemption is an accepted-risk marker, not a review. Converting the three
   hickory rows to real audits (or importing a trusted third-party audit) is genuine remaining
   work — deferred item **D4**.
2. The GHSA-q2qq-hmj6-3wpp claim is **quoted from the existing in-repo note**, not independently
   re-verified by this pass against the advisory database.
3. Because the branch tip was unreachable, I cannot rule out that it also carried unrelated
   housekeeping (lint fixes, CI tweaks) beyond the repin. The *repin* is subsumed; "the branch
   contained nothing else" is not asserted.

## 4. `fix/unique-blob-keys` — DROP (subsumed)

**What the branch was for.** Blob keys derived deterministically from the resource IRI mean two
writes to the same IRI reuse the same storage key. That is a bytes-integrity hazard: the orphan
GC can target a key that a concurrent recreate has since re-populated, and clobber live bytes.

**Why it is subsumed.** The composite store mints a **unique-per-write** key:

`crates/sparq-lws-core/src/store/mod.rs:333-355` — `mint_blob_key` builds
`{iri-derived-prefix}-{32 hex chars}`, where the 16-byte suffix comes from the OS CSPRNG
(`getrandom`). The prefix is explicitly cosmetic (operator traceability); **uniqueness comes
entirely from the random suffix**.

It is wired into every write path that creates bytes:

- `src/store/mod.rs:452` — `write`
- `src/store/mod.rs:495` — `create_in_container`

And it **fails closed**. `mint_blob_key` returns `ServerResult<String>`: if the OS CSPRNG is
unavailable the write errors rather than falling back to a timestamp-derived suffix. The in-code
rationale is precisely the collision class the branch existed to remove — *"the wall clock is NOT
unique per call: two concurrent mints in the same clock tick would derive the SAME suffix and thus
the SAME key"* (`src/store/mod.rs:325-332`).

**Ported code has tests** — the acceptance bar, met by three unit tests in
`src/store/mod.rs`:

| Test | Line | Property pinned |
|---|---|---|
| `mint_blob_key_is_unique_per_call_for_the_same_iri` | `:716` | 32 mints for one IRI are all distinct |
| `mint_blob_key_keeps_an_iri_derived_prefix_for_traceability` | `:737` | prefix retained; suffix is exactly 32 lowercase hex chars |
| `mint_blob_key_is_fallible_and_succeeds_on_a_working_os_rng` | `:762` | the fallible signature, plus an end-to-end write/read-back through the minted key |

The first carries an explicit **mutation check** in its comment — *"revert to the old deterministic
`iri.replace(...)` and every minted key is identical ⇒ this fails"* — which is the non-vacuity
evidence this repo asks for.

**Downstream, the fix is load-bearing and the code knows it.** The reconciler's module doc records
that unique keys close the snapshot-staleness reuse race *at its root*, with the re-stat and the
atomic compare-and-delete retained as defence-in-depth
(`src/store/reconcile.rs:35-37`, `:74-77`); the same reasoning appears at
`src/store/mod.rs:176`, `:229`, `:443`, `:491`, `:567`, `src/store/blob.rs:108`, `:143`, `:168`,
and `src/store/body_cache.rs:11`. The `getrandom` dependency carries the rationale at
`crates/sparq-lws-core/Cargo.toml:154`.

## 5. `phase-existence-non-disclosure` — DROP (subsumed)

**What the branch was for.** Closing the 404-vs-403 existence oracle: a requester who cannot read a
target must not be able to tell "exists but forbidden" from "does not exist".

**Why it is subsumed.** The whole V2/V4/V5/V6 family is in-tree. `lws-design-records.md` §6 is the
normative description and is not restated here; the enforcement sites are:

| Variant | Guard | Call sites |
|---|---|---|
| V2 — `Location` mint on POST | `mint_child_iri` (`handler.rs:2230`) | `handler.rs:1361` |
| V4 — concrete-ETag conditional requests | `guard_conditional_requires_read` (`handler.rs:627`) | PUT `:1133`, DELETE `:1453`, PATCH `:1628` |
| V5 — container `ETag` | structural (Read-gated placement only) | `handler.rs:904` |
| V6 — POST 404-vs-405 branch | `guard_post_existence_requires_read` (`handler.rs:706`) | `:1270`, `:1294`, `:1300` |

The two guards are kept in **lock-step** on how the required read-mode is computed —
`acl:Control` for an `.acl` target, `acl:Read` otherwise — so the pair cannot drift
(`handler.rs:681`, `:713`).

**It is pinned by tests at two levels:**

- *Unit, exhaustive.* `handler.rs:3795` opens the byte-identical matrix, whose top-level case
  `matrix_missing_equals_forbidden_byte_identical_for_every_verb` (`handler.rs:3949`) asserts the
  rule across every verb, with `authorized_reader_gets_true_404_on_genuinely_missing`
  (`handler.rs:3994`) as the matching positive control — without that second test the first would
  pass trivially on a server that returned 403 for everything.
- *Integration, end-to-end over real DPoP.*
  `tests/adversarial_invariants.rs:146` — `existence_is_not_disclosed_to_a_foreign_reader` mints a
  real token for a foreign WebID and asserts the existing-forbidden and non-existent statuses are
  **equal** and neither is 200.

Adjacent oracles closed by the same invariant are pinned in `tests/ldp_http.rs:1147` (PATCH-with-
deletes on a missing target) and `:1180` (uniform 401, not a 401-vs-409 oracle), and
`tests/public_read_skip.rs:294`, `:314`, `:323` assert the fast path is byte-identical to the full
anonymous path — status, **all** headers, and body.

### The caveat that must travel with this verdict

The brief flags this branch as SECURITY-adjacent and asks that it compose with the WAC bead. Two
things must be said plainly, and neither is a reason to re-open the port question:

1. **This family is not total, by design, and the code says so.** A requester who holds `acl:Read`
   through inheritance but is denied on one existing child by *that child's own* restrictive `.acl`
   can still distinguish that child (403) from a missing one (404). The code records this as
   WAC-inherent — a per-child `.acl` legitimately overrides inheritance
   (`handler.rs:697-700`). Any summary of V2/V4/V5/V6 must carry this rather than claim existence
   is never disclosed.
2. **V5 is enforced by placement, not by a guard.** The container `ETag` is non-disclosing only
   because it is emitted solely on the Read-gated GET/HEAD path. The code carries the
   forward-looking warning that *"if a future change emits a container's representation ETag
   outside a Read-gated path, that gate must be re-established there too"* (`handler.rs:894-901`).
   This is the concrete composition hazard for the WAC bead: a WAC change that adds a new
   representation path is the way V5 silently regresses, and no test would necessarily catch it.
   Deferred item **D3** proposes the structural pin.

**No Opus review is triggered by this bead**, on its own terms: the brief conditions that on the
work changing authz-observable behaviour, and this bead changes no code at all.

## 6. The M2-next marker sweep

Every `M2-next` / `M2:` marker under `crates/sparq-lws-core` was triaged by reading the code it
annotates rather than trusting the marker. **Five are stale** — the marker says "not implemented"
about something that now is. That is honesty drift in a security-relevant crate and is worth a bead
in its own right.

| Marker site | Verdict | Evidence |
|---|---|---|
| `src/ldp/mod.rs:16` — "full WAC authorization" | **STALE** | `authz/wac.rs` + `wac_allow.rs` implement it; enforced on GET/HEAD, PUT, POST, DELETE, PATCH via `WacAuthorizer` |
| `src/ldp/handler.rs:1089` — "the WAC seam is M2-next" | **STALE** | PUT authorizes at `handler.rs:1126` before the write |
| `src/store/mod.rs:489`, `:531` — reconciler/GC | **STALE** | `store/reconcile.rs` implements it; `reconcile_runtime.rs` + `tests/reconcile_periodic.rs` wire and pin the periodic sweep (landed by `sq-5ruwm`, `91fc6e45`) |
| `src/store/sparq.rs:109` — usage/quota | **PARTIAL** | `usage()`/`quota()` exist; the marker's "future access-evaluation step" clause is stale — WAC already reads ACL graphs through the `Store` |
| `src/ldp/content.rs:20` — N-Triples/N-Quads/N3 read formats | **OPEN** | `classify` (`content.rs:50-64`) accepts only `text/turtle` and `application/ld+json`; every other type is a 415 |
| `src/notifications/mod.rs:31`, `:43`; `src/notifications/ws.rs:157`, `:179`, `:243` — per-resource WAC on subscribe | **OPEN** | `subscribe_handler` (`ws.rs:162`) rejects anonymous callers but performs no topic ACL check; receive is token-gated (`ws.rs:276`) |
| `src/store/blob.rs:95`, `:162` — `object_store` adapter (`list` / `delete_if_unchanged`) | **OPEN** | no `object_store` backend in the crate; the in-memory double is the only impl |
| `src/store/reconcile.rs:59` — native version/ETag CAS witness | **OPEN** | the in-memory monotonic `generation` exists; the backend-native mapping does not (it is the same seam as the row above) |

**One nuance worth the maintainer's attention.** The notifications markers say the WAC check is
*"gated on `sparq#992` … same blocker as LDP read authorization"* (`ws.rs:157-160`). That blocker
has **lifted**: LDP read authorization shipped, and `WacAuthorizer::authorize_read`
(`authz/wac.rs:253`) is public and callable from the notifications path today. So the notification
subscribe gap is no longer *blocked* — it is simply *open*, and actionable now. The stale blocker
reference is why it has stayed quiet, which is exactly the cost of stale markers.

The security shape of that gap, stated precisely: an authenticated caller may subscribe to **any**
topic IRI. Receive is token-gated to the authenticated subscriber and topic (`ws.rs:276`), so this
is not an open firehose; but subscription is not ACL-checked per resource, so notification
*delivery* is not currently constrained by the same WAC decision that constrains a read of the same
resource. The crate is `publish = false` and EXPERIMENTAL, which is why this is a bead rather than
an advisory — but it should not stay open silently.

## 7. Deferred work — the beads this sweep produces

Per the bead's instruction these are **beaded, not built**. They are filed as follow-up issues
rather than implemented here.

1. **D1 — Notification subscribe: enforce per-resource WAC.** Call
   `WacAuthorizer::authorize_read` in `subscribe_handler` at the documented seam
   (`ws.rs:179`); reject a subscription to a topic the WebID cannot read. Must not become a new
   existence oracle: the denial has to match §5's rule (a non-reader learns nothing about whether
   the topic exists). *Security-relevant; deserves Opus review because it changes authz-observable
   behaviour.*
2. **D2 — Correct the five stale `M2-next` markers.** Doc-only. Re-point
   `src/ldp/mod.rs:16`, `handler.rs:1089`, `store/mod.rs:489`, `:531`, and `store/sparq.rs:109` at
   what is now true, and drop the dead `sparq#992` blocker reference from the notifications markers
   (the seam stays; the "blocked" claim goes).
3. **D3 — Pin V5 structurally.** The container-`ETag` non-disclosure is enforced by placement, so
   add a test (or a single chokepoint) that fails if a container representation `ETag` is emitted
   outside a Read-gated path. Closes the composition hazard §5 names against the WAC bead.
4. **D4 — Promote the hickory rows from exemption to audit.** `hickory-resolver`, `hickory-proto`,
   `hickory-net` @ 0.26.1 are `safe-to-deploy` **exemptions** with no audit entry; either audit them
   or import a trusted third-party audit, and re-verify the GHSA-q2qq-hmj6-3wpp claim against the
   advisory database rather than against the in-repo note.
5. **D5 — Optional: commit-level confirmation of this triage.** Add `jeswr/solid-server-rs` as a
   remote in a network-enabled pass and diff each of the three branch tips against import rev
   `1e555b10`, to upgrade §2's capability-level verdicts to commit-level ones. Low value if the
   maintainer already knows the branches merged upstream pre-snapshot — which is the point of §8's
   first question.
6. **D6 — `object_store` blob backend.** The genuinely large open M2 item: the S3/Local adapter
   behind `BlobStore`, including `list`, the backend-native CAS witness for
   `delete_if_unchanged`, and the `stat` override. Sizeable; listed for completeness, not proposed
   for the next slice.
7. **D7 — N-Triples/N-Quads/N3 read formats.** Extend `content.rs::classify` and the negotiation
   path. Small and self-contained.

Ordering: **D2** first (doc-only, removes the misinformation that hid D1), then **D1** and **D3**
together as the security-composition slice alongside the WAC bead, then **D4**. **D5** is
opportunistic; **D6** and **D7** are independent feature work.

## 8. Open questions for the maintainer

1. **Are the three branches merged upstream and safe to delete?** This triage's verdict is that
   their content is in the snapshot. Confirming they were merged (not abandoned mid-flight) turns
   "subsumed" into a closed question and makes D5 unnecessary.
2. **Was `phase-existence-non-disclosure` complete at the snapshot?** The branch name suggests a
   *phase* of a larger effort. §6 of the design record documents V2/V4/V5/V6 and the numbering skips
   V1 and V3 — if those are further variants that landed elsewhere (or never landed), that is a real
   gap this pass cannot see from in-tree evidence alone.
3. **Should D1 land before or with the WAC bead?** The brief asks that this bead's decisions compose
   with WAC. D1 is the only place the two genuinely interact, and doing it inside the WAC bead's
   review may be cheaper than a separate authz-observable change.

## 9. What this record does not do

- **It ports no code**, because it found nothing to port. The bead's acceptance clause "ported code
  has tests" is satisfied vacuously — and §4 and §5 record the tests that already pin each subject,
  so the clause is checkable rather than merely unmet.
- **It does not re-audit the security properties it cites.** §5 reports what the in-tree guards and
  tests enforce and carries the code's own documented residual disclosure; it does not certify the
  existence-non-disclosure family as complete.
- **It asserts no commit-level equivalence** between the upstream branch tips and the import. See
  §0's stated limit — the claim is capability-level, and D5 is how it would be upgraded.
- **It claims no capability for the crate.** `sparq-lws-core` remains EXPERIMENTAL and
  `publish = false`.
