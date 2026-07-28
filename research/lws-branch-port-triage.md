# `solid-server-rs` branch port triage — verdicts (sq-gg0qq.8) [OPUS-5]

> **Status: TRIAGE RECORD.** One written port-or-drop verdict per branch named in the
> bead, each grounded in code that is in `origin/main` today. No implementation is
> proposed here; the only code change made alongside this record is the `M2-next:`
> marker reconciliation of §3.
>
> Bead: **sq-gg0qq.8** · Issue: **#2744** · Parent: **#2572** / `sq-gg0qq`.
> Sibling records: `research/lws-3-crate-split.md` (`.4`), `research/solid-access-control-design.md`.

---

## 0. The premise correction

The bead is written as if three upstream `solid-server-rs` branches were still awaiting a
port decision:

- `origin/chore/repin-async-dns-verifier`
- `origin/fix/unique-blob-keys`
- `origin/phase-existence-non-disclosure`

**None of those refs exist on this repository's `origin`** (`git ls-remote --heads origin`
lists 542 heads; no name matches `repin|blob|existence|disclos|dns`). They were branches of
the *source* repo, `jeswr/solid-server-rs`, and they were merged into that repo's `main`
**before** the snapshot this workspace imported: `06a6428b` /`47e11a5c`
("import `jeswr/solid-server-rs@1e555b10` as `crates/sparq-lws-core`", `sq-gg0qq.2`, #1949).

So the port question does not arise for any of the three: the whole-crate import carried all
three landed changes in one commit. §1–§2 verify that claim per branch against the code
rather than asserting it from the commit message; §3 handles the remaining half of the
acceptance criterion (the `M2-next:` sweep).

Verification method used throughout: `git log -S<token> -- <path>` for the landing commit,
plus a direct read of the surviving code and its tests. Every `-S` probe below returns the
import commit and nothing later — i.e. the behaviour arrived with the import and has not
been re-litigated since.

---

## 1. Per-branch verdicts

### 1.1 `chore/repin-async-dns-verifier` — **DROP (already in main; no delta to port)**

The branch bumped the pinned `solid-oidc-verifier` rev so that the verifier's DNS-pinning
SSRF resolver moved `hickory-resolver 0.24 → 0.26.1`, which in turn lets this repo resolve
the patched `hickory-proto 0.26.1` (dependabot GHSA-q2qq-hmj6-3wpp — the DNS
name-compression DoS).

Evidence in `main`:

| Claim | Where | State |
| --- | --- | --- |
| Verifier pinned to a rev at-or-after the repin | `crates/sparq-lws-core/Cargo.toml` (`solid-oidc-verifier` git dep) | pinned to `89c8962`. The same file documents the pin chain: `b146253` is the rev that carried the hickory bump, and the later re-pins (`321db01 → 89c8962`) are the PoP Tier-1 work layered on top. The `Cargo.lock` row below is the decisive check — the current pin resolves the patched proto |
| Patched proto actually resolved | `Cargo.lock` | `hickory-proto 0.26.1`, `hickory-resolver 0.26.1` |
| MSRV consequence absorbed | `crates/sparq-lws-core/Cargo.toml` | `rust-version = "1.88"` (the MSRV `hickory-resolver 0.26.1` declares) |
| cargo-vet delta complete | `supply-chain/config.toml` | exemptions present for `hickory-proto`, `hickory-resolver`, **and** the crates the 0.26 line newly introduces — `hickory-net`, `prefix-trie`, `ipconfig` |

The cargo-vet delta is the part most likely to have been missed by a rev bump, and it is the
part that is complete: the 0.26 line split the resolver into `hickory-net` + `prefix-trie`,
and both carry exemption entries. Nothing to port.

Residual (already recorded in the `Cargo.toml` comment, not new work from this triage): the
verifier is pinned to a **branch rev**, not a tagged release. That FOLLOW-UP note stays where
it is — re-pin to a tag when upstream cuts one.

### 1.2 `fix/unique-blob-keys` — **DROP (already in main; ported code has tests)**

The branch replaced deterministic, IRI-derived blob keys with unique-per-write keys, closing
a class of collisions between a concurrent same-IRI recreate and either (a) a live overwrite
or (b) the orphan GC.

Evidence in `main`:

- `CompositeStore::mint_blob_key` — `crates/sparq-lws-core/src/store/mod.rs`. It **fails
  closed** on an unavailable OS RNG (the create errors rather than minting a weak key).
- Every write path mints through it: `CompositeStore::write` and
  `CompositeStore::create_in_container`.
- Direct unit test: `mint_blob_key_is_unique_per_call_for_the_same_iri` (`store/mod.rs`) —
  mints repeatedly for one IRI and asserts distinctness.
- The invariant is consumed, not merely asserted: `store/reconcile.rs` documents unique keys
  as the root closure of the GC clobber race (with the atomic `delete_if_unchanged` CAS kept
  as defence-in-depth), and `store/body_cache.rs` keys its LRU on `(blob_key, etag)` with a
  test — `different_blob_key_is_a_different_key` — pinning the keying.

`git log -S"mint_blob_key" -- crates/sparq-lws-core/src/store/mod.rs` returns only the import
commits. Nothing to port; the bytes-integrity concern the bead flagged is satisfied.

### 1.3 `phase-existence-non-disclosure` — **DROP (already in main; security-adjacent, verified against the WAC path)**

The branch is the 404-vs-403 existence-oracle closure. This is the one the bead singles out
as security-adjacent and asks to be composed with the WAC bead (`sq-gg0qq.5`) — so it gets
the most direct check: does authorization actually run **before** the existence probe on
every mutating verb, and are the individual disclosure channels closed?

Evidence in `main` — the vectors are named `V1`–`V6` in the code, all in
`crates/sparq-lws-core/src/ldp/handler.rs`:

| Vector | Channel | Closure in `main` (all symbols in `ldp::handler`) |
| --- | --- | --- |
| V1 | PUT create-vs-forbidden-overwrite (201 on a free name, 403 on a taken one) | `put_handler` requires `acl:Write` on the target's **effective** ACL regardless of existence, and calls `authorize` **before** the `store.meta()` probe |
| V2 | POST `Location` collision-dependence | minted child IRIs are opaque-suffixed and collision-independent (unit tests in `handler.rs`; end-to-end in `tests/ldp_http.rs`) |
| V3 | PATCH create-vs-overwrite mode split | `patch_handler` authorizes before the target read, so the under-authorized denial is timing-independent of existence |
| V4 | conditional-header channel (`If-Match`/`If-None-Match` 412-vs-2xx, and the returned ETag) | `guard_conditional_requires_read` folds to the denial code when a conditional header is present and the requester holds no Read — called before the existence probe from `put_handler`, `delete_handler`, and the `solid:where` PATCH read |
| V5 | membership-derived container ETag shift | handled on the container read path (`head_handler`/`get_handler`) |
| V6 | POST descendant-existence (404/405 branch) | `guard_post_existence_requires_read` gates the existence branch on the target's read mode — `Control` for an `.acl` (Control does not imply Read), `Read` otherwise — so an Append-only writer gets the forbidden-sibling status instead of the existence-revealing one |

Composition with WAC is not merely asserted: the `.acl` case is handled in lock-step in both
the V4 and V6 gates (reading an `.acl`'s representation/existence is itself a `Control`
operation, and `Control` does not imply `Read`), which is exactly the "decisions compose"
property the bead asked for. The doc-comment on `guard_post_existence_requires_read` records
one narrow residual — a Read-holder-via-inheritance edge — as a documented, deliberate
exception rather than a gap.

Tests: the V4 guard has dedicated coverage in `handler.rs` —
`v4_write_without_read_conditional_put_is_denied_not_412_or_2xx`,
`v4_write_without_read_conditional_delete_is_denied`, the `.acl`-target Control-mode edge
(`v4_control_only_holder_conditional_acl_write_is_not_wrongly_denied`), and two negative
controls (`v4_write_without_read_unconditional_put_still_succeeds`,
`v4_write_without_read_unconditional_delete_succeeds_proving_v4_is_the_cause`) that prove the
conditional denial comes from the V4 guard and not from the write authorization before it.
V5 has `v5_container_etag_only_reaches_a_reader`; V2 is covered end-to-end in
`tests/ldp_http.rs`. `crates/sparq-solid/tests/fail_closed_status_mapping.rs` pins the
`WacDecision` → status mapping on the sibling crate.

**One genuine gap, and it is documentation, not behaviour.** 22 code comments — 18 in
`ldp/handler.rs`, 3 in `tests/ldp_http.rs`, 1 in `tests/write_path_counters.rs` — cite
`decisions/0003` as the rationale document (`grep -rc` at this commit). That
ADR was deliberately left behind by the import ("`bench/`, `conformance/`, `docs/`,
`decisions/` stay in the source repo for their own beads" — `06a6428b`), so every one of
those cross-references currently dangles. Captured as a follow-up (§4), not fixed here: the
fix is either importing the ADR set or rewriting the citations, and both are bigger than this
bead's scope.

---

## 2. Verdict summary

| Branch | Verdict | Why |
| --- | --- | --- |
| `chore/repin-async-dns-verifier` | **DROP** | landed pre-import; pin, `Cargo.lock`, MSRV and the cargo-vet delta all present in `main` |
| `fix/unique-blob-keys` | **DROP** | landed pre-import; `mint_blob_key` fails closed, is used on every write path, and has a direct unit test |
| `phase-existence-non-disclosure` | **DROP** | landed pre-import; V1–V6 all closed and tested, composed with the WAC path. One doc-only residual (dangling `decisions/0003` citations) → follow-up |

No code was ported, because there was nothing left to port. "Ported code has tests" is
satisfied vacuously for the port and non-vacuously for what is in `main`: each of the three
concerns has a test named above.

---

## 3. The `M2-next:` sweep

The second half of the bead: sweep the remaining `M2-next:` markers and bead the deferred
items. A sweep is only useful if the inventory is true, so every marker was first checked
against the code. There are **16** `M2-next:` marker sites in the crate; **10** needed
correcting, and are corrected in the same change as this record (doc-comment text only, no
behaviour change):

- **5 are outright stale** — they defer work that has since landed: `ldp/mod.rs`, the
  `put_handler` doc-comment, the `lib.rs` seam list, and 2 sites in `store/mod.rs`.
- **5 (all in `notifications/`)** state a limitation that is still real but attribute it to a
  blocker that no longer applies.

The other 6 sites — `ldp/content.rs`, `store/blob.rs` ×3, `store/reconcile.rs`,
`store/sparq.rs` — are accurate and were left untouched.

| Marker | Was | Reality |
| --- | --- | --- |
| `ldp/mod.rs` | "full WAC authorization … not implemented" | implemented in `authz/` (per-resource `.acl`, ancestor `acl:default`, four modes, 401-vs-403, `WAC-Allow`) and called by every handler |
| `ldp/handler.rs` (PUT doc) | "the WAC seam is M2-next" | `put_handler` calls `authorize` before the existence probe |
| `lib.rs` | live SPARQ HTTP client, live JWKS deferred | `HttpSparqClient` (opt-in `http-sparq`) and `NetworkJwksProvider` are wired in `main.rs` |
| `lib.rs` | multipart `Range` deferred | `ldp::range::encode_multipart` + `tests/ldp_range_multipart.rs` |
| `lib.rs`, `store/mod.rs` ×2 | the reconciler deferred | `store/reconcile.rs` implemented; `main` spawns one periodic sweep at boot when an interval is configured (`reconcile_runtime::spawn_periodic_if_configured`, `sq-5ruwm`), covered by `tests/reconcile_periodic.rs` |
| `notifications/*` ×3 | subscription WAC "gated on `sparq#992`" | the *limitation* is real, the *blocker* is not — LDP already authorizes locally; what is missing is a store handle on `NotifyState` |

The markers that remain are genuine and are the deferred-item list in §4.

---

## 4. Deferred items (file as beads — not built here)

1. **Per-resource WAC on a notification subscription.** `NotifyState` carries only `hub` +
   `base_url`, so subscribe/receive cannot evaluate the topic's `.acl`. An authenticated
   caller may subscribe to any topic IRI. Now a wiring task against the existing local
   authorizer, not a design task. `notifications/mod.rs`, `notifications/ws.rs`.
2. **The real `object_store` `BlobStore` adapter.** There is no `object_store`-backed
   `BlobStore` impl: `InMemoryBlobStore` is the only production one (the rest are test doubles
   inside `reconcile.rs`), so the reconciler cannot sweep a real S3/GCS/Azure/local backend and
   `list()`'s grace window never sees a true object age. `delete_if_unchanged` must map the
   backend's native version/ETag/generation onto `BlobEntry::generation` — the reconciler's
   CAS witness must be a true write version, never a timestamp. `store/blob.rs`.
3. **N-Triples / N-Quads / N3 read formats.** `RdfFormat` is Turtle + JSON-LD only.
   `ldp/content.rs`.
4. **`acl:agentGroup` membership resolution.** Recognised but NEVER matches (fail-closed) —
   a member named via `agentGroup` is not granted. `authz/acl.rs`.
5. **The dangling `decisions/0003` citations** (§1.3). Import the ADR set from
   `jeswr/solid-server-rs` or rewrite the 22 citations to an in-tree record.
