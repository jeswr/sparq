<!-- [OPUS-5] sq-gg0qq.8: port-or-drop verdicts for the three named upstream
     housekeeping/security branches, plus the `M2-next:` marker register. -->
# LWS upstream branch triage — port-or-drop verdicts + the `M2-next:` register (sq-gg0qq.8)

> **Status: TRIAGE RECORD.** Three named branches, three written verdicts, plus a
> classified sweep of every remaining `// M2-next:` marker under
> `crates/sparq-lws-core/`. This record **corrects the premise it was written
> against** — see §1 — so read §1 before §2.
>
> Bead: **sq-gg0qq.8** · Issue: **#2744** · Blocked-by: **#2747** (`.4`) · Parent:
> **#2572** / `sq-gg0qq`. Siblings: `.1` supply-chain pre-flight, `.2` the crate
> import, `.3` the embedded-engine binding, `.4`
> [`lws-3-crate-split.md`](./lws-3-crate-split.md), `.5` the WAC-bypass fix, `.7`
> the conformance lane, `.10`
> [`lws-design-records.md`](./lws-design-records.md).
>
> Author: Claude Opus 5. Every claim below is cited to a `file:line` or a commit in
> **this** checkout and was re-derived from the code, not quoted from the brief or a
> prior record. **No timings appear in this document.**

---

## 1. Premise correction — all three "branches to port" already landed with the import

The brief asks for a port-or-drop verdict on `origin/chore/repin-async-dns-verifier`,
`origin/fix/unique-blob-keys`, and `origin/phase-existence-non-disclosure`, phrased as
though they were sparq branches awaiting triage. **They are not sparq branches.** No ref
matching any of those three names exists in this repository (688 refs; `git branch -a`
plus `.git/packed-refs` both return nothing). They are branches of the *upstream* repo
`jeswr/solid-server-rs`, which `crates/sparq-lws-core` was imported from at rev
`1e555b10` under `sq-gg0qq.2`.

And all three had **already been merged upstream before that import rev**, so their
content arrived in this repository wholesale, in one commit. The evidence is a
`git log -S` on each branch's defining symbol — every one of them is introduced by the
import commit and by nothing else:

| Branch | Defining symbol searched | Sole introducing commit |
|---|---|---|
| `fix/unique-blob-keys` | `mint_blob_key` | `47e11a5c` (#1949, `sq-gg0qq.2`) |
| `phase-existence-non-disclosure` | `guard_conditional_requires_read`, `guard_post_existence_requires_read` | `47e11a5c` (#1949, `sq-gg0qq.2`) |
| `chore/repin-async-dns-verifier` | the `solid-oidc-verifier` rev pin + its rationale block | `47e11a5c` (#1949, `sq-gg0qq.2`) |

So the correct disposition for all three is **DROP-AS-ALREADY-LANDED**, not PORT. There
is no upstream commit to cherry-pick and no code to write. What this bead can honestly
deliver instead is (a) the verification that the landed code is real and tested — §2,
§3 — (b) the one residual defect that verification found — §4 — and (c) the `M2-next:`
sweep the brief also asked for — §5.

**A caveat on the negative claim.** "Already merged upstream before `1e555b10`" is the
explanation most consistent with the evidence, but this pass could not read
`jeswr/solid-server-rs` to confirm it. What is *directly* verified is the weaker and
sufficient statement: **the functionality each branch names is present in this
checkout, arrived with the import commit, and is covered by tests.** If the upstream
branches turn out to contain work *beyond* what landed, that delta is unknown to this
record — see §7.

## 2. Per-branch verdicts

### 2.1 `origin/fix/unique-blob-keys` — DROP (already landed); bytes-integrity confirmed

**Verdict: already in-tree, keep.** The brief's guess ("likely PORT, bytes-integrity")
was directionally right about *importance* and wrong about *status*.

`CompositeStore::mint_blob_key` (`crates/sparq-lws-core/src/store/mod.rs:333`) mints a
fresh unguessable key per write rather than deriving one from the IRI, and it is used at
both write sites (`store/mod.rs:452` for `write`, `:495` for `create_in_container`). It
is **fallible** — an unavailable OS RNG fails the write closed rather than minting a
weak, possibly-colliding key (`store/mod.rs:762`).

This is load-bearing well beyond "two writes collide". The orphan reconciler's
correctness argument rests on it: because an overwrite never reuses a candidate's key,
the GC can never target live bytes, and a *versionless* backend orphan can therefore be
reclaimed by an unconditional delete instead of leaking forever
(`store/reconcile.rs:35`, `:74`). The atomic compare-and-delete
(`BlobStore::delete_if_unchanged`) is retained as defence-in-depth on top of it, not as
the primary guard.

### 2.2 `origin/phase-existence-non-disclosure` — DROP (already landed); composes with WAC

**Verdict: already in-tree, keep.** The V2/V4/V5/V6 closure family is implemented and
already has a durable design record at
[`lws-design-records.md`](./lws-design-records.md) §6 — written under sibling `.10`,
which is why no new ADR is needed here.

The brief asked to "coordinate with the WAC bead so decisions compose". **They do
compose, and the composition is explicit in the code rather than incidental.** The
`put_handler` authorizes through the WAC engine *before* any existence probe, and says
why: authorizing a create against the target's inherited ACL (not the weaker parent
`acl:Append`) is what makes create and forbidden-overwrite indistinguishable
(`ldp/handler.rs:1104-1126`). V4's conditional-request guard runs next, still before the
probe (`:1133`), and only then does the handler call `store.meta()` (`:1136`). The
ordering *is* the non-disclosure property; a future reviewer moving the existence probe
above the authorize call would silently reopen the V1 oracle.

Two things this record must repeat rather than let drift:

- The family is **not total**. A requester holding `acl:Read` through inheritance who is
  separately denied on one existing child by that child's own restrictive `.acl` can
  still distinguish it (403) from a missing one (404). The code calls this WAC-inherent
  and documents it (`ldp/handler.rs:697-700`); any summary that omits it overclaims.
- V5 is enforced by **placement**, not by a guard — the membership-derived container
  `ETag` is emitted only on the Read-gated path (`ldp/handler.rs:894-901`). It has no
  test that would fail if a future change emitted that `ETag` somewhere else. That is a
  standing structural risk, recorded as a follow-up in §6.

No observable authz behaviour is changed by this bead, so the Opus-review trigger the
brief attached to that condition does not fire.

### 2.3 `origin/chore/repin-async-dns-verifier` — DROP (already landed); one residual, fixed here

**Verdict: already in-tree — pin, lockfile, and cargo-vet delta all present.** The
verifier is pinned to an exact rev, not a floating branch
(`crates/sparq-lws-core/Cargo.toml:230`), `Cargo.lock:4801` resolves that same rev, and
the security payload the branch existed for is real: `hickory-proto` resolves at
`0.26.1` (`Cargo.lock:2119-2121`), the patched version that closes
`GHSA-q2qq-hmj6-3wpp`. The supply-chain delta is complete too — `hickory-net`,
`hickory-proto`, and `hickory-resolver` all carry `safe-to-deploy` exemptions at
`0.26.1` with the advisory cited (`supply-chain/config.toml:719-730`), and the git
source is allow-listed (`deny.toml:216`).

**The residual this verification found.** The pin-rationale comment block is an
accumulated log inherited from upstream, and its *first* line asserted a pin that is no
longer the pin: it opened "Pinned to `b146253`" while the dependency line 160 lines
later declares `89c8962`. A reader — including a supply-chain auditor, who is exactly
who that block is written for — reaches the stale sentence first and can reasonably
conclude the crate is pinned to `b146253`. This bead rewrites that opening as an
explicit log header naming the current pin; the historical entries are left intact,
because each still records something the current pin inherits.

Note also that the block's chain has an undocumented step: the later entry re-pins
`321db01 → 89c8962` and calls `321db01` "the prior rev", but nothing introduces
`321db01`. This record does **not** assert an ordering it cannot verify, so the fix
states only what the file itself proves — the current pin — and leaves the gap visible
rather than inventing the missing link.

## 3. Test coverage of the already-landed code

The acceptance criterion "ported code has tests" is satisfied by tests that arrived with
the code. Verified present:

| Area | Tests |
|---|---|
| Unique blob keys | `store/mod.rs:716` unique-per-call for the same IRI; `:737` IRI-derived prefix retained for traceability; `:762` fallible, fails closed on RNG failure; `store/blob.rs:563` concurrent writes get unique strictly-ordered generations |
| Blob-key collision, end to end | `tests/store.rs:100` each write to the same IRI mints a distinct key; `:141` concurrent writes to the same IRI do not collide; `:770` an empty-container delete does not clobber a concurrent same-IRI recreate |
| Existence non-disclosure, V6 | `ldp/handler.rs:4104` the append-dropbox POST oracle is closed; `:4186` an owner still gets a true 404; `:4337` a read-less writer's store fault folds to the denial, not a 500 |
| Existence non-disclosure, V4/V5 | `ldp/handler.rs:5078` an unauthorized caller gets the denial, not a 304; `:5712` the container `ETag` only reaches a reader |
| Existence non-disclosure, end to end | `tests/adversarial_invariants.rs:146` existing-forbidden and non-existent return the same status to a foreign reader; `tests/ldp_http.rs:838` a missing-resource DELETE is a uniform denial, not a 404; `ldp/handler.rs:3994` an authorized reader still gets a true 404; `:6029` the `solid:where` PATCH is not an existence oracle |
| Verifier pin | not unit-testable — the evidence is the lockfile resolution and the vet exemptions cited in §2.3 |

No new tests are added by this bead, because no new behaviour is added. Writing a test
for already-tested behaviour would inflate the diff without raising assurance.

## 4. What this bead changes

Comment- and documentation-only. No control flow, no public API, no dependency, no
lockfile change.

1. `crates/sparq-lws-core/Cargo.toml` — the pin-rationale block now leads with the
   current pin instead of a superseded one (§2.3).
2. Five stale `// M2-next:` markers corrected (§5.2). A marker claiming a capability is
   unimplemented, when it is implemented and enforced, is a false statement about the
   code; two of the five are about **WAC**, so the false statement is about a security
   control.

## 5. The `M2-next:` register

Sixteen markers under `crates/sparq-lws-core/`. The sweep's first job is separating the
ones that describe real remaining work from the ones that describe work already done —
filing a bead for the latter would be filing a bead for nothing.

### 5.1 Live — genuinely deferred (these are the follow-ups)

| Marker | Subject | Verified open because |
|---|---|---|
| `notifications/ws.rs:157`, `:179`, `:243`; `notifications/mod.rs:31`, `:43` | per-resource WAC authorization of a subscription | `subscribe_handler` checks authentication and mints a topic-bound receive token, then proceeds with no ACL check (`ws.rs:167-180`) |
| `store/blob.rs:95` | `BlobStore::list` via the `object_store` adapter | the only non-test `BlobStore` impl is `InMemoryBlobStore` (`blob.rs:337`) |
| `store/blob.rs:162`, `blob.rs:49`, `store/reconcile.rs:59` | backend-native versioned compare-and-delete witness | same — no backend exposes a native write version yet |
| `store/sparq.rs:109` | `usage()` / `quota()` on the `SparqClient` trait | the trait declares neither |
| `ldp/content.rs:20` | N-Triples / N-Quads / N3 read formats | `content.rs` imports only `oxttl` and `oxjsonld` |
| `authz/acl.rs:17`, `:179` (not spelled `M2-next:`, same class) | `acl:agentGroup` resolution | recognised but deliberately never matches, fail-closed |

### 5.2 Stale — the seam has since been implemented (corrected in this bead)

| Marker | Claimed | Actually |
|---|---|---|
| `ldp/mod.rs:16` | "full WAC authorization" is unimplemented, "needs the SPARQ access-control design" | the in-Rust WAC engine exists (`authz/wac.rs`, `authz/acl.rs`, `authz/mode.rs`) and `authz/mod.rs:4-6` states it *supersedes* the interim posture |
| `ldp/handler.rs:1089` | on `put_handler`: "a mutation from a public caller is a 403 (the WAC seam is M2-next)" | `put_handler` calls `state.authorize("PUT", …)` at `:1126` and the surrounding comment describes the full WAC create/overwrite mode analysis |
| `lib.rs:55`, item "the reconciler" | not yet written | `store/reconcile.rs` implements `reconcile_orphans` (`:231`) and `spawn_periodic` (`:554`), with `tests/reconcile_periodic.rs` |
| `lib.rs:55`, item "multipart Range" | not yet written | `ldp/range.rs:46` `encode_multipart`, with `tests/ldp_range_multipart.rs` |
| `store/mod.rs:489`, `:531` | orphaned bytes are GC'd by "the reconciler (M2-next)" | the reconciler exists; only the *object-store backend* part is still future |

The two remaining items in the `lib.rs:55` list — per-resource authorization of a
subscription, and `acl:agentGroup` resolution — are live and stay.

## 6. Deferred items — the follow-up set

Filed through the worker's follow-up channel rather than as `bd` records, because this
run must not write `.beads/`. Ordered by priority.

1. **Wire per-resource WAC into notification subscribe/receive.** The highest-value item
   the sweep found, and the one whose *status changed*: the code defers it to
   "`sparq#992`, the SPARQ access-control design" (`ws.rs:158-159`), but that blocker no
   longer holds — `WacAuthorizer::authorize_read` (`authz/wac.rs:253`) exists in this
   crate today. The seam is at `ws.rs:179`. Until it is wired, any authenticated WebID
   may subscribe to any topic IRI and receive its change notifications. That is a
   documented known limitation, not a silent one, and the receive bypass is separately
   closed by the token gate — but it is a live authorization gap that is now unblocked.
2. **Pin V5 with a test.** The container-`ETag` closure is enforced by placement alone
   (§2.2) and would not fail any test if a future change emitted that `ETag` off the
   Read-gated path. The lock-step note at `ldp/handler.rs:894-901` asks for exactly this.
3. **`object_store` is a declared but entirely unused dependency.** `object_store =
   "0.14"` (`Cargo.toml:253`) is a non-optional native dependency with zero non-comment
   references in the crate — the S3/GCS/Azure tree is in the build graph purely in
   anticipation of the adapter in §5.1. Either build the adapter or make the dependency
   optional behind its feature.
4. **`SparqClient::usage()` / `quota()`** (`store/sparq.rs:109`).
5. **N-Triples / N-Quads / N3 read formats** (`ldp/content.rs:20`).
6. **`acl:agentGroup` resolution** (`authz/acl.rs:17`).
7. **Re-pin the verifier to a tagged release** once upstream cuts one — the block's own
   standing `FOLLOW-UP` (`Cargo.toml:83`), which would also retire the accumulated
   rationale log that §2.3 had to repair.

## 7. What genuinely needs the maintainer

- **The unverifiable delta.** §1 confirms the *functionality* of all three branches is
  in-tree, but could not read `jeswr/solid-server-rs` to confirm the branches contain
  nothing further. Only the maintainer can say whether those three refs still exist
  upstream with unmerged commits on them. If they do not, they can be deleted upstream
  and this record closes the question; if they do, the delta needs its own bead.
- **The `321db01` gap.** §2.3 leaves the undocumented middle link in the verifier pin
  chain visible rather than guessing at it. The maintainer, who made the pins, can
  supply the missing entry or confirm it is not worth reconstructing.
- **Priority of follow-up 1.** Whether the notification WAC gap should be scheduled now
  that it is unblocked, or deliberately held until the SPARQ-authoritative access-control
  design lands so the decision moves behind the `WacAuthorizer` seam in one step
  (`authz/mod.rs:18-23`), is a sequencing call, not a technical one.
