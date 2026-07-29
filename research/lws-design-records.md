<!-- [OPUS-5] sq-gg0qq.10: the durable in-repo home for the imported LWS server's
     design/decision estate. Reconstructed from in-tree code, not copied.
     [SONNET-4.6] Merged with the branch-side record of the same estate (#5076): the
     source-path map, the sync↔tokio bridge (§3), the V1/V3 closures (§6) and the
     §7 mechanism map come from that side; the file:line citation discipline,
     §8 and §9 from this one. -->
# LWS server design records — the migrated `decisions/` + `docs/design/` estate (sq-gg0qq.10)

> **Status: DESIGN RECORD (migration + reconstruction).** `crates/sparq-lws-core` was
> imported whole from [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs)
> at rev `1e555b10` under `sq-gg0qq.2`. That import deliberately left the source repo's
> `docs/design/` and `decisions/` trees behind, so ~70 doc-comments in the imported code
> now cite paths that **do not exist in this checkout**. This record is the durable
> in-repo home those citations should resolve to.
>
> Bead: **sq-gg0qq.10** · Issue: **#2742** · Parent: **#2572** / `sq-gg0qq`.
> Siblings: `.1` supply-chain pre-flight, `.2` the crate import, `.3` the in-workspace
> embedded-engine binding, `.4` the 3-crate split decision package
> ([`lws-3-crate-split.md`](./lws-3-crate-split.md)), `.5` the WAC-bypass fix,
> `.7` the conformance lane.
>
> Related in-repo records: [`lws-3-crate-split.md`](./lws-3-crate-split.md),
> [`lws-demo-architecture.md`](./lws-demo-architecture.md),
> [`lws-sparql-wac-scoped-dataset.md`](./lws-sparql-wac-scoped-dataset.md).
>
> **What this record is, precisely.** The original ADR prose is in another repository and
> was **not** available to this pass — nothing here is a verbatim copy of it. Every
> decision below is **reconstructed from the in-tree code that implements it**, and every
> claim carries a `file:line` citation you can check. Where the code does not state a
> rationale, this record says so rather than inventing one. Treat the code as normative
> and this record as the map; where the two disagree, **the code wins and this record is
> the bug**.
>
> **Reading the section numbers.** A reference written `RSS docs/design/<file>.md §N`
> names a section of **that upstream file**, not of this record. References of the form
> `research/lws-design-records.md §N` — the form the in-tree doc-comments carry — name a
> section here.
>
> **No timings appear in this document.** Three of the migrated source documents are
> throughput/latency design notes; §7 explains why their figures are pointer-only rather
> than reproduced, and the repo's no-hard-coded-perf-numbers rule is why.

---

## 1. The reference map — source-repo path → in-repo home

Every path in the left column is cited by an imported doc-comment and **does not exist in
this repository**. The right column is where the load-bearing content actually lives now.
Paths in the right column are relative to `crates/sparq-lws-core/` unless otherwise noted.

| Cited path (source repo) | Namespace | Subject | Durable in-repo home |
|---|---|---|---|
| `decisions/0001-embed-sparq-in-process.md` | RSS | in-process engine backend | §3 below · `src/store/embedded.rs`, `src/store/mod.rs`, `src/main.rs` (backend selection), the `embedded-sparq` feature in `Cargo.toml` |
| `decisions/0002` | RSS | pre-crypto public-read skip | §5 below · `src/ldp/public_read_skip.rs`, the layer order in `src/app.rs`, `tests/public_read_skip.rs` |
| `decisions/0003` | RSS | existence-non-disclosure V1–V6 | §6 below · `src/ldp/handler.rs` (the V-series guards + the byte-identical status matrix at the foot of the file) |
| `docs/design/webid-outside-pod.md` | RSS | provider WebIDs on an identity host | §4 below · `src/identity.rs`, `src/ldp/target.rs`, `src/seed.rs`, the identity env vars in `src/main.rs`, `tests/identity_http.rs` + `tests/identity_conneg.rs` |
| `docs/design/backend-read-path.md` (§3.1 read-2, §3.4 read-4, §7 read-1) | RSS | read-path round-trip budget | §7.1 · `src/store/sparq.rs` (`read_plan`), `src/store/mod.rs`, `src/authz/wac.rs` (`read_plan_candidates`), `src/store/body_cache.rs`, `src/store/counting.rs`, `tests/read_path_counters.rs`, `tests/write_path_counters.rs` |
| `docs/design/beyond-50k-throughput.md` (§4 P1.3–P1.6, §5) | RSS | transport-level throughput work | §7.2 · `src/tls.rs`, `src/nodelay.rs`, `tests/tcp_nodelay.rs`, `tests/response_write_coalescing.rs`, `tests/embedded_read_counters.rs` |
| `docs/design/high-throughput-pop-auth.md` (§4–§6 DPoP-SK, §7 bead 2) | RSS | tiered proof-of-possession | §7.3 · `src/pop/mod.rs`, `src/pop/cert_bound.rs`, `src/pop/conn.rs`, `src/pop/sk/`, the mTLS env var in `src/tls.rs` |
| `docs/design/throughput-hard-cases.md` (§5 `read-4-bodycache`) | RSS | blob body cache | §7.1 · `src/store/body_cache.rs` |
| `decisions/0020-webid-outside-pod.md` | **PSS** | upstream WebID-host convention | upstream; adapted by §4, in `src/identity.rs` |
| `decisions/0001-foundational-architecture.md` | **PSS** | upstream architecture | upstream; cited by [`concurrent-serving.md`](./concurrent-serving.md), and by `src/lib.rs` as the example that makes the RSS/PSS numbering collision concrete. No code depends on it. |
| `decisions/0003-qlever-live-update.md` | **PSS** | upstream live-update study | upstream; cited by [`concurrent-serving.md`](./concurrent-serving.md) |
| `decisions/0012` | **PSS** | upstream serving contract | upstream; cited by the `sq-kb` ingest corpus |

## 2. The two ADR namespaces — a real collision, read the number with its repo

The imported code cites **two different, independently-numbered `decisions/` trees**, and
the numbers overlap:

- **RSS** = `jeswr/solid-server-rs`, the Rust server this crate was imported from. Its
  `decisions/0001` is *embed sparq in-process*.
- **PSS** = `prod-solid-server`, the upstream TypeScript Solid server. Its `decisions/0001`
  is *foundational architecture*, and its `decisions/0020` is the WebID-host convention
  that RSS adapted.

So `decisions/0001` means two unrelated things depending on which repo the citing comment
belongs to. `crates/sparq-lws-core/src/identity.rs:4-5` is the one place the code
disambiguates explicitly — it names `docs/design/webid-outside-pod.md` as "the RSS
adaptation of prod-solid-server `decisions/0020-webid-outside-pod.md`". Everywhere else the
namespace is implicit: a citation inside `crates/sparq-lws-core/**` is RSS unless it names
prod-solid-server, and a citation inside `research/concurrent-serving.md` is PSS.

**Rule for future edits:** when you touch one of these citations, write the namespace into
it (`RSS decisions/0003`, `PSS decisions/0020`). Bare `decisions/NNNN` is ambiguous — do not
write one. The in-tree citation form follows `src/lib.rs:58`: dense implementation comments
may use the short form (`RSS decisions/0003`), while doc-comments additionally carry the
full `research/lws-design-records.md §N` pointer so the citation resolves here.

## 3. ADR — embed SPARQ in-process (RSS `decisions/0001-embed-sparq-in-process.md`)

**Decision.** The authoritative-RDF seam (`SparqClient`) is implemented by calling the
`sparq-engine` query/update entry points **directly against an in-process `Graph`**
(`sparq-core`), rather than over SPARQ's HTTP service. It is selected at boot by
`PSS_SPARQ_BACKEND=embedded`, compiled behind the `embedded-sparq` cargo feature, which is
**on by default** since `sq-gg0qq.3` (`src/store/embedded.rs:2-12`; `Cargo.toml:356-368`).
`--no-default-features` builds the engine-free profile. The remote shape lives behind the
opt-in, default-off `http-sparq` feature.

**Backends and their gates** (`src/main.rs:214-226`):

| `PSS_SPARQ_BACKEND` | Cargo feature | Default? | Durability |
|---|---|---|---|
| `memory` | none | the **boot** default | ephemeral in-memory double |
| `embedded` | `embedded-sparq` | feature on by default; **not** the boot default | see below |
| `http` | `http-sparq` | off | the external service's |

Note the deliberate split between *compiled in* and *selected*: `embedded-sparq` is a
default-**on** build feature, but runtime selection "stays EXPLICIT via this variable
(fail-safe: an unconfigured boot serves the ephemeral in-memory double, never a store an
operator did not choose)" (`src/main.rs:220-226`; `Cargo.toml:362-366`). A default build
that is never configured therefore serves the in-memory double, not the engine.

**Rationale, as stated in the code** (`src/store/embedded.rs:14-30`) — three points, none
of them a performance claim:

1. **Same queries, different transport.** Every query/update is built by the *same*
   injection-safe builders in `store::sparql` that the HTTP client uses, "VERBATIM, no new
   query strings", so conformance-equivalence to the HTTP and in-memory impls is trivial:
   identical SPARQL, identical named-graph model, only the execution path differs.
2. **Named-graph isolation is real here.** The engine's `query`/`update_in_place` support
   `GRAPH <g> { … }` over a single `Graph` holding the default graph plus named graphs
   (graph IRI == resource IRI — the WAC-design model). The code records that the live HTTP
   `sparq-server` "today folds named graphs into one default graph", which the comment
   labels the HTTP path's `DEVIATION-1`; embedding sidesteps it.
3. **No marker/follow-up-ASK atomicity dance.** Over HTTP a SPARQL UPDATE cannot return
   rows, so the outcome must be probed by a follow-up ASK against per-operation markers —
   racing a concurrent mutation. In-process the whole operation "runs as ONE indivisible
   actor job on the `Graph`'s owning thread", so `create_child`/`delete_meta_if_empty`
   return their outcome directly (check-then-act with no interleaving).

**The sync↔tokio bridge** (`src/store/embedded.rs:32-46`). The `SparqClient` trait is
`async_trait` and the server is tokio, but the engine is blocking and CPU-bound, so running
a call on a tokio worker would block the reactor. The `Graph` is instead **owned outright by
one dedicated OS thread** — the private `GraphActor` — and every engine call is a job sent
to it over an mpsc channel, awaited via a `oneshot` reply. This replaced `sq-gg0qq.3`'s
`spawn_blocking`-over-`Arc<Mutex<Graph>>` slice: no lock held across the engine call, no
blocking-pool hop, and one FIFO serialisation point that a future WAL can hang off. The
module docs are explicit that these are **structural** properties, not a measured speed-up —
the change was not benchmarked and the degree of serialisation is unchanged. Behavioural
equivalence is deliberate: jobs stay serialised and indivisible, and a panicking job
*poisons* the actor exactly as a poisoned `Mutex` failed closed.

**Durability — the honest posture.** `embedded` is durable only with a persistence
directory. `SOLID_SERVER_SPARQ_DIR` set ⇒ a directory-backed graph (durable);
unset ⇒ a fresh in-memory graph (ephemeral) (`src/main.rs:234-256`;
`src/store/embedded.rs:67-74`). Separately, the native binary's **blob** backend is
in-memory, so a durable RDF index does not by itself make resource bodies durable — the
crate README and `skills/solid-lws-server/SKILL.md` both say this, and it must stay said.

## 4. ADR — provider WebIDs outside the pod (RSS `docs/design/webid-outside-pod.md`)

**Decision.** A provider-issued WebID is served from a separate **identity host**, never as
an in-pod LDP resource. The WebID form is `https://<identity-host>/<handle>#me`, with the
document at `https://<identity-host>/<handle>`; the default host is derived as
`id.<base authority>` rather than hard-coded, so the server stays deployment-agnostic
(`src/identity.rs:11-12`, `78-102`).

**Rationale — quoted verbatim** (`src/identity.rs:5-9`):

> The WebID document is the Solid-OIDC identity trust root: every resource server on the
> web dereferences it to learn which issuers may mint tokens for that WebID
> (`solid:oidcIssuer`). Hosting it INSIDE the pod — a WAC-governed, owner-writable
> resource — leaves it one over-broad `acl:default` grant away from ecosystem-wide identity
> takeover.

**Mechanism.** Id-docs are stored under the reserved internal namespace
`<base>/.identity/<handle>`, written with no containment edge, so they appear in no
`ldp:contains` listing and are addressable only by the id-host route (`src/seed.rs:59-76`).
The LDP surface refuses `/.identity/**` outright — `404` for every method, `%`-decoded too,
**regardless of the identity feature flag** — so no `.acl` can ever exist for the namespace
and no WAC grant can ever apply to an id-doc (`src/ldp/target.rs:45-51`). That refusal is
enforced twice on purpose: the identity gate middleware refuses it first (outermost), and
`parse_target` is the belt-and-braces chokepoint covering every handler
(`src/identity.rs:238-250`). The seed comment states the consequence plainly — no `.acl` is
written for an id-doc because none *can* exist, and that is "the security property, not an
omission" (`src/seed.rs:59-76`). That impossibility, not a policy check, is the property.

**Serving and writing.** The Host-keyed identity gate is the outermost layer: `GET`/`HEAD`
only with Turtle/JSON-LD conneg, ETag/304, public cache headers, explicit CORS; no WAC
evaluation, no `.acl` Link, no auth processing; every other method → 405; anything that is
not exactly one valid non-reserved handle → 404, fail-closed. **Writes** reach id-docs
"ONLY through the `Store` seam by boot seeding today … and by the future admin provisioning
seam — never through the LDP path (which refuses the namespace)" (`src/identity.rs:27-29`).

**"LOCKED" id-doc.** The served document carries exactly the provider-controlled
statements: the Person type, the locked `solid:oidcIssuer`, `pim:storage` → the pod root,
the `<pod> solid:owner <webid>` back-link, and `rdfs:seeAlso` → the in-pod card
(`src/seed.rs:239-244`, `272-275`). The in-pod `/{u}/profile/card` is correspondingly
demoted to a user-editable extended profile carrying **neither** `solid:oidcIssuer` **nor**
`pim:storage` (`src/seed.rs:64-72`).

**Default posture.** `SOLID_SERVER_IDENTITY_ENABLE` is **default OFF**
(`src/main.rs:174-182`). Turning it on makes the identity gate serve id-docs on the id host
(`GET`/`HEAD` only, Turtle/JSON-LD, no WAC, no `.acl` Link) and makes the conformance seed
mint id-host WebIDs instead of in-pod cards. The unconditional LDP refusal of
`/.identity/**` holds either way, so that pre-seeded documents can never become
LDP-addressable — and thus `.acl`-able — when the flag is later turned on.

## 5. ADR — the pre-crypto public-read skip (RSS `decisions/0002`, "opt 3")

**Decision.** For a read whose response is fully identity-independent, a thin middleware
runs the existing anonymous WAC predicate *before* the auth layer and serves the read
directly (`src/ldp/public_read_skip.rs:2-8`). It is wired unconditionally into the router
layer stack, just inside CORS and just outside auth (`src/app.rs:288-303`) — there is no
feature flag.

**The hard scope limit, quoted verbatim** (`src/ldp/public_read_skip.rs:10-12`) — this is
the load-bearing, security-critical part:

> The skip fires **only for a GET/HEAD that carries NEITHER an `Authorization` NOR a `DPoP`
> header.** A request that carries credentials (or a `DPoP` proof header) is NEVER
> short-circuited — it falls through to the full auth path.

Two independent reasons are recorded, both pinned by the WAC-Allow conformance suite
(`src/ldp/public_read_skip.rs:18-28`): `WAC-Allow user=` is identity-**dependent**, so
serving an authenticated owner's public read as anonymous would under-report their access —
a wrong, observable response; and a forged proof is indistinguishable from a legitimate
owner's without verifying it, so the only way to serve forged proofs as harmless-anonymous
would be to serve *every* proof-carrying public read as anonymous, which is exactly the
first error. `tests/public_read_skip.rs:5-6` restates the limit as the property under test.

**Fall-through cases that keep the full path's behaviour.** Mutations always fall through.
The `DPoP` header is *also* a fall-through trigger, so a no-`Authorization` GET carrying a
malformed `DPoP` header keeps the auth path's canonical `400` instead of being served as
anonymous — a deliberate divergence-closure (`src/ldp/public_read_skip.rs:33-34`,
`110-113`).

**Dispatch — one WAC pass.** The middleware constructs a public token and delegates
**straight to the same `serve_read` the handler uses**, rather than pre-checking and then
serving; `serve_read` does exactly ONE effective-ACL resolution, so the response is
byte-identical to the full anonymous path — it is the same call. An earlier
pre-check-then-`serve_read` version performed a second WAC pass and was a review-flagged
regression (`src/ldp/public_read_skip.rs:42-51`).

**Do not oversell this one.** The module docstring is explicit that because there is no
proof to verify on such a request, the win is "a marginal one (one fewer public-token
construction + an earlier WAC pass), **NOT a crypto saving**"
(`src/ldp/public_read_skip.rs:6-8`). The nickname "skip-crypto" overstates it; the code
already corrects itself, and any doc that repeats the nickname should carry the correction.

**Not reconstructable.** The source ADR also recorded options 1 and 2, and why applying
option 3 to *credentialed* reads is unsafe in full. The code names them
(`src/ldp/public_read_skip.rs:30-31`) but does not restate them, and they are **not stated
in the code** — so they are not reconstructed here.

## 6. ADR — existence-non-disclosure (RSS `decisions/0003`, variants V1–V6)

A family of closures over one invariant: **a requester who lacks the read-mode on a target
must not be able to distinguish "exists but forbidden" from "does not exist"** — the read-
mode being `acl:Control` for an `.acl` target and `acl:Read` otherwise. The exhaustive
byte-identical matrix is the top-level pin: `404` is served only to a requester who holds
the operation's required mode; every other requester gets their own denial code (`401`
anonymous / `403` authenticated) for **both** the forbidden-existing and the not-found case,
byte-identically (`src/ldp/handler.rs:3780`).

| Variant | Channel closed | Mechanism | Guard / pin |
|---|---|---|---|
| **V1** | create-vs-forbidden-overwrite on PUT | `acl:Write` is required on the **target**'s effective ACL (inherited via `acl:default` when the target does not yet exist) rather than the weaker parent-`acl:Append`, so create and overwrite are indistinguishable to an under-authorized requester; authorizing *before* any target-dependent `meta()`/existence probe closes the timing variant too | `handler.rs:1103-1123`; test section `handler.rs:4377` |
| **V2** | the minted `Location` header on POST | the mint **always** appends the opaque suffix, so the shape is collision-independent — it never varies on whether the slug was free — while still *containing* the slug so `post-uri-assignment-slug` conformance holds; a slug-less POST falls back to an identically-shaped default stem | `mint_child_iri` (`handler.rs:2200-2216`) |
| **V3** | the PATCH create-vs-modify split | the required mode is derived purely from the already-parsed patch content (insert-only ⇒ `acl:Append`, any delete ⇒ `acl:Write`, `.acl` ⇒ `acl:Control`) and authorized against the same inherited target ACL for both cases, so create and forbidden-modify return byte-identical denials; authorizing before the target read closes the timing channel | `handler.rs:1569-1592`; test section `handler.rs:4856` |
| **V4** | conditional requests carrying a **concrete** entity-tag validator (`If-Match: "x"` / `If-None-Match: "x"`) | such a validator is content- or membership-derived, so a non-reader carrying one gets their denial code **instead of** the precondition being evaluated — decided *before* the existence probe, so neither the `412`-vs-`2xx` outcome nor any `ETag` is observable. A bare **`*`** (`If-None-Match: *` safe-create / `If-Match: *` lost-update) is deliberately **EXEMPT**: it carries no ETag fingerprint and tests only existence, which a holder of the operation's required mode already learns from the unconditional `201`-vs-`204` write split — so Read-gating it broke the standard `PUT … If-None-Match: *` create pattern for a Write-without-Read holder at zero non-disclosure gain. The same gate covers a patch carrying a `solid:where` clause, which reads the target graph | `guard_conditional_requires_read` (`handler.rs:602-657`), wildcard split via `conditional_carries_etag`, rationale at `handler.rs:631-640`; called from PUT (`:1125`), DELETE (`:1443`), PATCH (`:1615`); `solid:where` gate at `handler.rs:1593-1600` |
| **V5** | the container `ETag` | the membership-derived container `ETag` shifts on every child add/remove, so it is a listing oracle; it is emitted only on the `acl:Read`-gated GET/HEAD path, and its conditional-request sibling is closed by V4 | structural — no separate guard (`handler.rs:894-901`) |
| **V6** | the POST `404`-vs-`405` existence branch | an agent holding `acl:default acl:Append` but no `acl:Read` could otherwise probe descendant existence; a non-reader gets their denial code instead of the branch, again *before* the existence probe | `guard_post_existence_requires_read` (`handler.rs:659-679`, `701-720`), called at POST (`:1261`, `:1285`, `:1291`) |

**Trade-off (recorded, not incidental).** The V1 closure means an `acl:Append`-only agent
can no longer PUT-create; it must POST, which mints a server-opaque collision-free name.
This is CTH-safe — no conformance row expects an Append-only PUT-create to succeed
(`handler.rs:1113-1117`).

**The residual disclosure is documented, not closed — do not describe this family as
total.** The code records an explicit WAC-inherent exception: a requester holding `acl:Read`
through inheritance who is separately denied on one existing child by *that child's own*
restrictive `.acl` can still distinguish that child (`403`) from a missing one (`404`),
"this is WAC-inherent (a per-child `.acl` legitimately overrides inheritance) and documented
in decisions/0003" (`handler.rs:697-700`). Any doc summarising the V-series must carry that
caveat rather than claim existence is never disclosed.

**The lock-step note is load-bearing.** V6 computes the read-mode "EXACTLY [as]
`guard_conditional_requires_read` computes [it], kept in lock-step so the two
existence-disclosure gates cannot drift" (`handler.rs:677-678`). V5 carries the matching
forward-looking caveat: it is enforced by *placement* rather than by a guard, so "if a
future change emits a container's representation ETag outside a Read-gated path, that gate
must be re-established there too" (`handler.rs:894-901`). Both are the kind of invariant
that survives only if it is written down — which is the reason this section exists.

## 7. The four `docs/design/*.md` performance records — figures pointer-only, on purpose

`backend-read-path.md`, `beyond-50k-throughput.md`, `high-throughput-pop-auth.md`, and
`throughput-hard-cases.md` are throughput/latency design notes. Their **figures and their
argued-from measurements are not reproduced here**, for two reasons, and the omission is
deliberate rather than an oversight:

1. **The repo forbids hard-coded performance numbers in markdown** (`AGENTS.md`;
   `scripts/check-no-perf-numbers.py`). The canonical homes for figures are the bench
   registry and the generated dashboard. A reconstruction of a throughput design note that
   *omitted* its numbers would be a shell; one that included them would be a gate
   violation, and — worse — an unreproducible number quoted from a document this pass
   never read.
2. **Their load-bearing output is already in-tree as executable pins, not prose.** The
   round-trip budgets are asserted by counting decorators and deterministic counter tests
   (`src/store/counting.rs`, `tests/read_path_counters.rs`, `tests/write_path_counters.rs`,
   `tests/embedded_read_counters.rs`); the transport knobs and their opt-in/default-off
   posture are documented on the items themselves (`src/tls.rs`); the tiered
   proof-of-possession design is documented across the `src/pop/` module docs; the blob body
   cache is documented on `src/store/body_cache.rs`. A test that fails when the count
   changes is a stronger record than a paragraph asserting the count.

If a figure from one of those documents is ever needed, it must be **re-measured in this
repo** through the bench harness — not quoted from the source repo.

What *is* recorded below is the **mechanism map**: which in-tree item implements which
numbered item of which source document, so a citation resolves without the source repo to
hand. The shared discipline across all four is the repo's perf-gate rule — **deterministic
metrics are hard-pinned, wall-clock is advisory and never asserted.**

### 7.1 Backend round-trips — RSS `docs/design/backend-read-path.md` (+ `throughput-hard-cases.md`)

- **read-1 (§7) — measure first.** `CountingSparqClient` / `CountingBlobStore`
  (`src/store/counting.rs`) are transparent decorators counting calls per trait method at
  the `SparqClient`/`BlobStore` seams; zero-cost when not wired in.
  `tests/read_path_counters.rs` pins **exact integer** per-operation counts end-to-end
  through the assembled router, with `max_in_flight == 1` as the await-depth witness that
  the pinned totals equal the operation's sequential round-trip depth.
  `tests/write_path_counters.rs` is the write-verb sibling and the measure-first baseline
  for the write-path walk-collapse.
- **read-2 (§3.1) — collapse the ACL walk.** `Store::read_plan` / `SparqClient::read_plan`
  (`src/store/mod.rs:219`, `src/store/embedded.rs:535`) answer the target's authoritative
  metadata **and** the presence/etag of every ACL candidate on its resolution chain in
  **one** index round-trip, replacing the sequential child→root probes. Candidates are
  derived up front as pure string work by `WacAuthorizer::read_plan_candidates`
  (`src/authz/wac.rs:307`; nearest-first — element 0 is the protected resource's own ACL,
  scope `AccessTo`; the rest are ancestors', scope `Default`). Two IRI roles are
  load-bearing for `.acl` targets: a GET of `foo.acl` is governed by Control on `foo`.
  Fail-closed on any backend error. **write-2** applies the same plan to
  PUT/POST/DELETE/PATCH via `WacAuthorizer::authorize_planned` (`src/authz/wac.rs:198`),
  whose in-memory walk plus live found-ACL re-confirm is differentially tested bit-for-bit
  against the sequential walk for every `AccessMode` (`src/authz/wac.rs:1837`).
- **read-4 (§3.4, and `throughput-hard-cases.md` §5 `read-4-bodycache`) — the blob-body
  LRU.** A byte-budgeted cache of resource **bodies** keyed by `(blob_key, etag)`, in front
  of `BlobStore::get` inside `CompositeStore`. It needs **no invalidation protocol**:
  `mint_blob_key` mints a fresh 128-bit-random key on every write, so a blob object never
  changes after creation and every lookup is keyed by *this* request's authoritative index
  metadata — a rewrite is a guaranteed miss, and the etag in the key is defence-in-depth
  (`src/store/body_cache.rs:11`; `src/store/mod.rs:179`). It also cannot bypass
  authorization: it sits inside the store, strictly below the WAC gate, and `serve_read`
  runs `authorize_read` before ever asking for bytes. Budgets are tunable via
  `SOLID_SERVER_BODY_CACHE_BYTES` / `SOLID_SERVER_BODY_CACHE_MAX_ENTRY_BYTES`; `0` disables
  (`src/store/body_cache.rs:45-49`, `:64`, `:128`).

### 7.2 Connection + transport — RSS `docs/design/beyond-50k-throughput.md`

- **P1.3 — connection amortization (`src/tls.rs`).** The rustls session-resumption cache is
  sized by `SOLID_SERVER_TLS_SESSION_CACHE_SIZE` (`src/tls.rs:123`; `0` disables
  resumption), and an opt-in, default-off stateless `Ticketer` (random per-process keys,
  rotated, forward-secret by key erasure) is the stateless half (`src/tls.rs:61-73`,
  `:140-146`). Default-off is deliberate: a per-process ticket key is not shared across a
  scaled fleet, so cross-node resumption falls back to a full handshake — a perf, never a
  correctness, effect. **0-RTT early data is never enabled:** `max_early_data_size` is
  forced to `0` on every path including with the ticketer, because 0-RTT is replayable by
  design and incoherent under this server's anti-replay DPoP `jti` model (§5 of the source
  doc) (`src/tls.rs:56-59`, `:73`, `:121-123`, `:146`). A `debug_assert` pins the value so
  a future rustls default change surfaces in tests. Ticketer construction is fail-safe — an
  RNG error logs and falls back to the stateful cache.
- **P1.4 — response-write coalescing.** `tests/response_write_coalescing.rs` pins the number
  of write-family syscalls the HTTP/1.1 response path emits per response, measured at the
  hyper→transport seam while driving the real assembled router over the same `axum::serve`
  path production uses, on both the anonymous (skip) and DPoP-authenticated routes.
- **P1.5 — `TCP_NODELAY` on both serve paths.** The deterministic metric is the
  socket-option *state* read back via `TcpStream::nodelay()`, not any latency. The plain
  path taps accepted streams with `nodelay::tap_nodelay` (`src/nodelay.rs:53`); the TLS path
  composes `nodelay::NoDelayAcceptor` inside the `RustlsAcceptor` so the option is set on
  the raw stream before the handshake (`src/nodelay.rs:20`, `:31-34`). Pinned by
  `tests/tcp_nodelay.rs`.
- **P1.6 — counters on the real engine.** `tests/embedded_read_counters.rs` re-proves the
  seam-level round-trip counts against the in-process `EmbeddedSparqClient`, so a regression
  that adds a round-trip on the embedded path fails. Gated on `embedded-sparq`; a no-op
  binary when off.

### 7.3 Tiered proof-of-possession — RSS `docs/design/high-throughput-pop-auth.md`

DPoP (RFC 9449) stays the **mandatory** Solid-OIDC baseline. The tiers are negotiated,
opt-in fast paths, individually refusable, and never a silent downgrade.

- **T1a** — the confirmation dispatch plus the RFC 8705 `cnf.x5t#S256` cert-binding
  verification core (`src/pop/mod.rs`, `src/pop/cert_bound.rs`), transport-agnostic and
  unit-testable in isolation, fail-closed (a cert-bound token with no visible cert is
  rejected).
- **T1b** — the transport half (`src/pop/conn.rs`): read the peer's client certificate
  **once per connection**, compute its thumbprint, and expose it to every request on that
  connection. Enabled by `SOLID_SERVER_MTLS_BOUND_TOKENS` (`src/tls.rs:105`), **default
  OFF** — when unset the serve paths are byte-identical to the pre-T1b behaviour.
- **T2 — DPoP-SK** (`src/pop/sk/`): negotiated symmetric session keys for DPoP-bound
  requests, against the [DPoP-SK draft](https://jeswr.github.io/dpop-sk-spec/) whose
  Appendix-A worked example the implementation reproduces byte-for-byte in tests
  (`src/pop/sk/derive.rs:162`); flag-off builds run the pre-Tier-2 path
  (`src/pop/sk/mod.rs:25`, `:66`).

These three module doc-comments are explicitly the durable record of the tiering design; the
source document's §4–§7 were not carried over by the import.

## 8. The maintainer spec estate — the specs are the contract

Several behaviours of this crate are not sparq's to define: they implement an external
normative specification, and the spec — not this implementation — is the contract. When the
two disagree, **the spec is right and the code is the bug**; when the spec moves, the pinned
reference moves with it in the same change.

Verified in-tree members of that estate:

| Spec / dependency | Pinned reference | What depends on it | Evidence |
|---|---|---|---|
| **DPoP-SK** — the sender-key DPoP profile (<https://jeswr.github.io/dpop-sk-spec/>) | cited by section; the spec's Appendix-A worked example is executed as a test vector | `src/pop/sk/**` (HKDF/HMAC DPoP-SK attestation) | `src/pop/sk/mod.rs:4-5`; `src/pop/sk/derive.rs:5`, `:162` |
| **solid-oidc-verifier** (<https://github.com/jeswr/solid-oidc-verifier>) | git dependency at rev `89c896249a726398b78302fd2f65eef0a82af681`, `network` feature | baseline (cache-miss) Solid-OIDC access-token + DPoP proof verification; `JwksProvider` / `ReplayStore` | `Cargo.toml:230`; `src/auth.rs:5`; `src/lib.rs:15` |
| **solid-oidc-verifier**, cache-hit path | same pin, used through the verifier's *public* primitives (`verify_proof_with_embedded_jwk`, `Jwk::thumbprint_sha256`, `proof_has_ath`, `peek_claims`) + the SHARED `ReplayStore` | on a verified-token-cache hit the cached token is reused, and `src/auth_cache.rs` re-verifies the fresh proof itself: signature, `htm`/`htu`/`iat`, `ath == H(token)`, the `jti` replay mark, and the `cnf.jkt` binding | `src/auth_cache.rs:13-31`, `:55-57` |
| **lws-acp** (<https://github.com/jeswr/lws-acp/tree/main/docs>) | consulted as prior art | the ACP/trust-graph authorization design | [`solid-trust-graph-authz-design.md`](./solid-trust-graph-authz-design.md) §; [`security-properties-ontology-design.md`](./security-properties-ontology-design.md) |

The DPoP-SK row is the pattern worth copying: `src/pop/sk/derive.rs:162` runs the *spec's
own* execution-verified Appendix-A worked example as a unit-test vector, so a drift between
the spec's arithmetic and this implementation's fails a test rather than sitting silent.
That is what "the specs are the contract" should mean mechanically — a pinned reference plus
an executed vector, not a prose citation.

**Unresolved, stated honestly.** The `sq-gg0qq.10` bead also names **`lws-spec`** and
**`lws-ucs`** as members of this estate. Neither name appears anywhere in this
repository — not in code, docs, `research/`, or `.beads/issues.jsonl` — so this pass has
**no in-tree evidence of their location or content** and does not link them rather than
guess a URL. Resolving them (and, if they are normative for shipped behaviour, pinning them
the way DPoP-SK is pinned) is follow-up work, not something this record can assert.

## 9. What this record does not do

- **It is not a verbatim migration.** The source ADRs were not read. Everything above is
  reconstructed from code, and §5 flags the one place where the source ADR provably holds
  content (options 1 and 2) that the code does not restate. Where the full argument survives
  in this repo it survives in the module doc-comments, several of which explicitly designate
  themselves as the durable record (`src/pop/mod.rs`, `src/pop/conn.rs`,
  `src/pop/sk/mod.rs`). Where it does not, the source repository remains the only copy.
- **It does not bring the upstream trees in — the rewritten citations resolve *here*, not to
  the originals.** The imported citations inside `crates/sparq-lws-core/**` now carry their
  namespace (`RSS`/`PSS`) per §2's rule, with the module- and item-level doc-comments
  additionally carrying the `research/lws-design-records.md §N` pointer and dense
  implementation comments using the short form §1 resolves. So they resolve — but they
  resolve to **this reconstruction**. The RSS `docs/design/` + `decisions/` trees themselves
  are still not in this repository and no upstream prose is copied verbatim, so whatever
  those documents hold that the in-tree code does not state remains readable only in the
  source repo (§5's options 1 and 2 are the one instance this pass could prove). Bringing the
  trees in is tracked under the `sq-gg0qq` epic; until such a bead lands, nothing here should
  be read as a transcription of the originals.
- **It claims no capability.** Every behaviour above is stated with its default posture
  (`embedded-sparq` on, `SOLID_SERVER_IDENTITY_ENABLE` off, `http-sparq` off) and its
  durability caveat. `sparq-lws-core` remains **EXPERIMENTAL**, is `publish = false`, and
  does not replace the TypeScript prod-solid-server.
