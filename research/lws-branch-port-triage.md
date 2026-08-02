<!-- [OPUS-5] sq-gg0qq.8: port-or-drop verdicts for three upstream solid-server-rs
     branches, plus the M2-next marker sweep. Evidence-based; no code was ported. -->
# LWS upstream branch triage — port-or-drop verdicts (sq-gg0qq.8)

> **Status: TRIAGE RECORD (decision, no implementation).** Three upstream branches named by the
> bead are triaged here to a written **port-or-drop** verdict — both halves answered, see the
> outcome line — and the crate's remaining `M2-next` markers are swept to a deferred-work list.
>
> Bead: **sq-gg0qq.8** · Issue: **#2744** · Parent: **#2572** / `sq-gg0qq` · Blocked-by: **#2747**.
> Companion record: [`lws-design-records.md`](./lws-design-records.md) (the reconstructed ADR
> estate — §6 there is the normative description of the existence-non-disclosure family).
>
> **Outcome in one line:** all three branches are **DROP — nothing to port**. Each branch tip is a
> git **ancestor of the import rev** `1e555b10`, so every commit on every one of them is already in
> the snapshot (§0.1); the headline capability of each is additionally citable in-tree (§3–§5).
> Both halves of the port/drop question are now answered from evidence, not inference — the earlier
> revision of this record deferred the branch-level half, and that deferral is **closed**.

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

## 0.1 The decisive check: every branch tip is an ancestor of the import

An earlier revision of this record was written without network access and could only argue
*capability-level* subsumption from in-tree `file:line` citations, explicitly deferring the
branch-level verdict. That limit no longer applies: the upstream remotes were reachable on this
pass, so the commit-level comparison it deferred (item **D5**) was simply **run**.

Cloning `jeswr/solid-server-rs` and asking, for each branch, which of its commits are *not* already
in the import rev:

```text
git rev-list --count origin/<branch> --not 1e555b10
chore/repin-async-dns-verifier   -> 0     git merge-base --is-ancestor -> ANCESTOR
fix/unique-blob-keys             -> 0     git merge-base --is-ancestor -> ANCESTOR
phase-existence-non-disclosure   -> 0     git merge-base --is-ancestor -> ANCESTOR
```

Every branch tip is an **ancestor** of `1e555b10`. Each branch was therefore merged upstream *before*
the snapshot was taken, and — this is the part the capability argument could never reach — **carries
no commit that the import lacks**. That answers §8 Q1 directly and upgrades all three rows from
"capability subsumed" to a genuine branch-level **DROP**.

Each branch tip is far *behind* the import (the tip-vs-import diff is dominated by tens of thousands
of deleted lines — work added after the branch point), which is the expected shape for a merged,
stale branch and the reason a raw two-dot diff is the wrong tool here; ancestry is the right one.

**The one residual.** Ancestry is a fact about upstream's commit graph. It shows the branches hold
nothing the import's *history* lacks; it does not by itself certify that sparq's import commit
`47e11a5c` transcribed that tree faithfully. The capability-level citations in §3–§5 are what check
the transcription, which is why both halves are kept rather than the ancestry check replacing them.

## 1. Why capability-level subsumption is the expected answer, not a convenient one

The import commit is a single whole-tree snapshot:

| Fact | Evidence |
|---|---|
| The crate was imported whole from `jeswr/solid-server-rs` at rev `1e555b10` | `research/lws-design-records.md:6-8` (bead `sq-gg0qq.2`) |
| That import is one commit | `47e11a5c` — *feat(lws): import jeswr/solid-server-rs as crates/sparq-lws-core (sq-gg0qq.2)* (#1949) |
| Each of the three subjects' defining symbols was **introduced by that commit** | `git log -S` on `mint_blob_key`, `guard_post_existence_requires_read`, and the verifier rev string each return exactly `47e11a5c` |

So each subject's defining text was **already in upstream's tree when the snapshot was taken**, and
work that is already inside the snapshot you imported cannot be "ported" — it is already here. That
is the mechanism behind the capability half of each verdict, and it is why the three agree.

**The mechanism is only as strong as what the symbol denotes**, and that differs by subject. For
`fix/unique-blob-keys` and `phase-existence-non-disclosure` the symbol *is* the implementation:
`mint_blob_key` and `guard_post_existence_requires_read` are in-tree functions whose behaviour §4
and §5 read directly and whose tests are in-tree. For `chore/repin-async-dns-verifier` the symbol is
merely a **40-hex rev string** naming an out-of-tree dependency: that the string was in the snapshot
shows only *which revision was pinned*, not *what that revision does*. Lockfile membership of
`hickory-resolver` does not close that gap either — a dependency can go unused on the relevant path,
or be used without the SSRF policy. §3 therefore does not infer the capability; it reads the pinned
revision's own source and cites the call path.

**What `git log -S` alone does not establish.** Run in *this* repository it shows only what the
imported snapshot contains. It cannot distinguish "the branch was merged upstream before the
snapshot" from "the capability reached upstream `main` by some other route while the branch carried
further, unmerged work" — so on its own it licenses a *capability-level* verdict at best. That is
precisely the gap **§0.1's ancestry check** closes, and why the branch-level verdicts rest on that
check rather than on this table.

## 2. The verdict table

Each verdict now rests on **two independent legs**: the branch-level ancestry check (§0.1 — the
branch holds no commit the import lacks) and the capability-level in-tree evidence (§3–§5 — the
thing the branch existed to deliver is here, and works).

| Upstream branch | Subject | Verdict | Basis |
|---|---|---|---|
| `chore/repin-async-dns-verifier` | verifier git-pin bump to the async-DNS resolver | **DROP — nothing to port** | ancestor of the import (§0.1); the import pins a *descendant* of the rev the branch repinned to, and that pinned source resolves via async hickory behind the SSRF gate (§3) |
| `fix/unique-blob-keys` | blob-key collision fix (bytes integrity) | **DROP — nothing to port** | ancestor of the import (§0.1); unique-per-write minting implemented, fail-closed, 3 unit tests incl. a mutation check (§4) |
| `phase-existence-non-disclosure` | 404-vs-403 existence non-disclosure | **DROP — nothing to port** | ancestor of the import (§0.1); the V2/V4/V5/V6 family + the exhaustive byte-identical matrix are in-tree and pinned (§5 of this doc, §6 of the design record) |

No branch is a **PORT**, and no code changed under this bead. All three are safe to delete on the
strength of §0.1 — with the single caveat recorded there (ancestry certifies upstream's history, not
the fidelity of sparq's transcription of it; §3–§5 are what check that).

## 3. `chore/repin-async-dns-verifier` — DROP (nothing to port)

**What the branch was for.** Bumping the `solid-oidc-verifier` git pin to a revision whose
WebID/issuer resolution uses an **asynchronous, SSRF-guarded** DNS resolver rather than a blocking
one — a housekeeping/hardening bump, with a cargo-vet delta for the new transitive crates.

**The branch's own repin is superseded, not merely matched.** The branch tip pins the verifier at
rev `836899d`, which still used `hickory-resolver` **0.24**; the imported tree pins `89c8962`, a
**descendant** of `836899d` on the verifier's own history, using **0.26.1**. So the in-tree pin is at
or beyond the branch's target:

| Where | Verifier rev | `hickory-resolver` |
|---|---|---|
| branch tip `origin/chore/repin-async-dns-verifier`, `Cargo.toml:50` | `836899d` | 0.24 |
| import rev `1e555b10`, `Cargo.toml:94` = in-tree `crates/sparq-lws-core/Cargo.toml:230` | `89c8962` | 0.26.1 |

The async, SSRF-guarded resolver design the branch existed to adopt is present at **both** revs —
`836899d`'s `src/net.rs:127` already documents the dedicated-thread + `TokioAsyncResolver` fix — and
the in-tree rev additionally carries the 0.24 → 0.26.1 migration
(`TokioAsyncResolver::tokio_from_system_conf` → `Resolver::builder_tokio()`) that picks up the
GHSA-q2qq-hmj6-3wpp `hickory-proto` fix. The import is therefore strictly ahead of the branch here.

**The capability is verified at the source, not inferred from the lockfile.** Dependency membership
is not use, so the pinned revision `89c8962` was checked out and read. The WebID/issuer resolution
call path (all line numbers in `jeswr/solid-oidc-verifier` @ `89c8962`):

1. **Both fetching seams share one guarded fetcher.** `NetworkWebIdResolver::new` builds
   `SafeFetcher::system` (`src/webid.rs:214`) for the WebID-profile fetch; the issuer side — OIDC
   discovery + JWKS — builds the same thing at `src/config.rs:261`. Both are `#[cfg(feature =
   "network")]`, and `network` is the crate's **default** feature and the one our pin selects
   explicitly (`Cargo.toml:230`), so this is the path this build takes.
2. **The resolver really is async hickory.** `SafeFetcher::system` (`src/net.rs:304`) wires
   `SystemResolver`, whose background thread builds `hickory_resolver::Resolver::builder_tokio()`
   (`src/net.rs:199`) — falling back to an explicit `udp_and_tcp(&GOOGLE)` config, and **failing
   closed** if even that cannot be built (`src/net.rs:212-218`) — then awaits `resolver.lookup_ip(…)`
   per host (`src/net.rs:228-231`).
3. **The SSRF policy is applied by the caller, on every record.** `SafeFetcher`'s fetch resolves the
   host and runs **each** returned address through `classify_resolved_address_with_nat64`
   (`src/net.rs:354`, the classifier at `src/webid.rs:166`), and *any* non-public record fails the
   whole request — the documented DNS-rebinding mitigation, "the attacker cannot mix a public +
   private record" (`src/net.rs:351-353`). The connection is then **pinned** to the validated
   addresses, and redirects are followed manually with the whole gate re-applied per hop, under a
   bounded hop count (`src/net.rs:319`, `:326`, bound documented at `:49-50`).

**One correction to the branch's framing.** "Async rather than blocking" is true of the *resolver*
but not of the fetch seam. `HostResolver::resolve_host` is deliberately a **synchronous** trait
method (`src/net.rs:109`) and the HTTP hop uses `reqwest::blocking` on a dedicated OS thread
(`src/net.rs:428-439`). The async hickory resolver runs on its own background current-thread runtime
precisely *because* hickory's synchronous `lookup_ip` internally calls `Runtime::block_on` and would
panic with "Cannot start a runtime from within a runtime" when a Tokio/axum handler calls
`Verifier::verify` directly (`src/net.rs:127-140`) — which is exactly how `solid-server-rs`, and
therefore `sparq-lws-core`, calls it. The repin fixed a **nested-runtime panic**, not a latency
problem; anyone reading "async DNS" as "the verifier's fetch path is now async" would be misled.

**Upstream has tests for the guard**, so this is a pinned property rather than a read of the source:
`tests/webid_ssrf.rs` covers private-IP literals (`:165`), the per-record rebinding loop where one
public + one private record must fail (`:211`), and redirect-to-private (`:220`); `src/net.rs:566`
and `:577` cover the same through the fetcher with an injected adversarial resolver.

**A cargo-vet delta is also recorded**, under this bead family's own pre-flight. This is
supply-chain bookkeeping about *which* crates entered the graph — it is not what establishes the
call path above (points 1–3 are), and its wording is a pre-existing in-repo assertion this pass did
not re-verify against the advisory database (residual 2 below):

| Crate | Version | Criteria | Note site |
|---|---|---|---|
| `hickory-resolver` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:722-725` |
| `hickory-proto` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:718-720` |
| `hickory-net` | 0.26.1 | `safe-to-deploy` | `supply-chain/config.toml:714-716` |
| `jsonwebtoken` | 10.4.0 | `safe-to-deploy` | `supply-chain/config.toml:907` |

The `hickory-resolver` note reads: *"solid-server-rs import pre-flight (sq-gg0qq.1): transitive of
solid-oidc-verifier — its DNS-pinned SSRF-guarded async resolver (0.26.1 carries the
GHSA-q2qq-hmj6-3wpp hickory-proto fix)."* So the sibling bead `sq-gg0qq.1` did the supply-chain half
of this branch's work, deliberately. The note's *"DNS-pinned SSRF-guarded"* phrasing is corroborated
by the call path above rather than relied upon for it.

**Stated honestly — two residuals.**

1. These are **exemptions**, not audits: `supply-chain/audits.toml` and `imports.lock` contain no
   `hickory` entry. An exemption is an accepted-risk marker, not a review. Converting the three
   hickory rows to real audits (or importing a trusted third-party audit) is genuine remaining
   work — deferred item **D4**.
2. The GHSA-q2qq-hmj6-3wpp claim was **not** checked against the advisory database by this pass. It
   now has two independent in-source corroborations — the in-repo cargo-vet note, and the pinned
   verifier's own `Cargo.toml`, which documents the `>=0.26.1` pin as fixing exactly that advisory
   (*"the 0.24 line's `hickory-proto` is vulnerable to GHSA-q2qq-hmj6-3wpp … the fix is only on
   hickory-proto 0.26.1, reachable ONLY via hickory-resolver 0.26"*) — but two authors agreeing is
   not an advisory-database check, and D4 still owns that.

The residual that used to head this list — *"whether the pinned revision actually performs
asynchronous, SSRF-guarded resolution is unverified"* — is **closed** by the call path above. So is
the *"the branch may have carried unrelated commits"* residual, by §0.1's ancestry check.

## 4. `fix/unique-blob-keys` — NO PORT (capability subsumed)

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

## 5. `phase-existence-non-disclosure` — NO PORT (capability subsumed)

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
5. **D5 — Commit-level confirmation — ✅ DONE, not deferred.** This item asked for a network-enabled
   pass comparing each branch tip against import rev `1e555b10`. It was run; all three tips are
   ancestors of the import (§0.1), which is what upgraded §2 to branch-level DROP. Retained here as
   a record of how the verdict was reached, not as outstanding work.
6. **D6 — `object_store` blob backend.** The genuinely large open M2 item: the S3/Local adapter
   behind `BlobStore`, including `list`, the backend-native CAS witness for
   `delete_if_unchanged`, and the `stat` override. Sizeable; listed for completeness, not proposed
   for the next slice.
7. **D7 — N-Triples/N-Quads/N3 read formats.** Extend `content.rs::classify` and the negotiation
   path. Small and self-contained.
8. **D8 — Verify the pinned verifier's DNS resolution path — ✅ DONE, not deferred.** Raised because
   §3 originally inferred the capability from `Cargo.lock` membership. Rev `89c8962` was checked out
   and the WebID/issuer call path cited in §3; the async hickory resolver and the per-record SSRF
   gate are both real and upstream-tested. Retained as provenance for §3's evidence.

Ordering: **D2** first (doc-only, removes the misinformation that hid D1), then **D1** and **D3**
together as the security-composition slice alongside the WAC bead, then **D4** (the only remaining
supply-chain item, and the owner of the un-re-verified GHSA claim). **D6** and **D7** are independent
feature work. **D5** and **D8** are done.

## 8. Open questions for the maintainer

1. ~~**Are the three branches merged upstream and safe to delete?**~~ **ANSWERED — yes, by §0.1.**
   Each tip is an ancestor of the import rev, so each was merged upstream before the snapshot and
   carries no commit the import lacks. This no longer needs the maintainer's recollection. The one
   thing worth a maintainer's eye is the residual §0.1 records: ancestry certifies upstream's
   history, not the fidelity of sparq's transcription of that tree.
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
- **It asserts containment, not tree-equality.** §0.1 shows each branch tip is an *ancestor* of the
  import rev — so no branch holds a commit the import lacks — which is what licenses the DROP. It is
  not a claim that sparq's imported tree is byte-identical to upstream's at `1e555b10`; §3–§5's
  in-tree citations are the check on the transcription.
- **It does not audit the `solid-oidc-verifier` dependency.** §3 traces the resolution call path at
  the pinned rev far enough to establish that the async hickory resolver *is* reached and that every
  resolved record *is* run through the SSRF classifier, and it names upstream's tests for that. That
  is a call-path verification, **not** a security audit of the classifier's completeness (whether its
  private-range/NAT64 policy covers every case is upstream's to prove, not this record's).
- **It claims no capability for the crate.** `sparq-lws-core` remains EXPERIMENTAL and
  `publish = false`.
