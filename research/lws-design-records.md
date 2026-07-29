# LWS design records — the in-repo home for the imported `docs/design/` + `decisions/` citations [SONNET-4.6]

> **Status: DURABLE POINTER RECORD.** `crates/sparq-lws-core` was imported whole from
> [jeswr/solid-server-rs](https://github.com/jeswr/solid-server-rs) at rev `1e555b10`
> (bead `sq-gg0qq.2`; see the crate `README.md` header and `research/lws-3-crate-split.md`).
> That import deliberately left the source repo's `docs/design/` and `decisions/` trees
> behind — **those paths do not exist in this checkout.** Dozens of doc-comments across the
> crate still cite them, so this record is the durable in-repo target those citations
> resolve through: §1 is the source-path → in-repo map, §§3–7 record each cited decision **as
> it is actually implemented here**, and name the module docs that hold the full argument.
>
> Related in-repo records: `research/lws-3-crate-split.md` (the crate-split decision package),
> `research/lws-demo-architecture.md`, `research/lws-sparql-wac-scoped-dataset.md`.

---

## 1. Source-path → in-repo map

### 1.1 The two `decisions/` namespaces

Two independently-numbered `decisions/` trees are cited by this crate and **their numbers
collide**, so every in-tree citation names its namespace:

| Prefix | Repository | What it is |
| --- | --- | --- |
| **RSS** | `jeswr/solid-server-rs` | The Rust server this crate was imported from. |
| **PSS** | `prod-solid-server` | The upstream TypeScript server whose conventions RSS adapted. |

A bare `decisions/NNNN` with no namespace prefix is ambiguous — do not write one. Dense
implementation comments may use the short form (`RSS decisions/0003`); doc-comments carry the
full `research/lws-design-records.md §N` pointer as well.

### 1.2 The map

Every source path cited in `crates/sparq-lws-core`, the section of this record that covers it,
and the in-repo code whose module docs are now the durable record of the decision:

| Cited source path | § here | Durable in-repo home |
| --- | --- | --- |
| RSS `decisions/0001-embed-sparq-in-process.md` | [§3](#3-embedded-sparq-in-process--rss-decisions0001) | `src/store/embedded.rs`, `src/store/mod.rs`, `src/main.rs` (backend selection), the `embedded-sparq` feature in `Cargo.toml` |
| RSS `decisions/0002` | [§5](#5-the-pre-crypto-public-read-skip--rss-decisions0002) | `src/ldp/public_read_skip.rs`, the layer order in `src/app.rs`, `tests/public_read_skip.rs` |
| RSS `decisions/0003` | [§6](#6-existence-non-disclosure--rss-decisions0003) | `src/ldp/handler.rs` (the V-series guards + the existence-non-disclosure test matrix at the foot of the file) |
| RSS `docs/design/webid-outside-pod.md` | [§4](#4-provider-webids-outside-the-pod--the-identity-host) | `src/identity.rs`, `src/ldp/target.rs`, `src/seed.rs`, the identity env vars in `src/main.rs`, `tests/identity_http.rs` + `tests/identity_conneg.rs` |
| RSS `docs/design/backend-read-path.md` (§3.1 read-2, §3.4 read-4, §7 read-1) | [§7](#7-the-throughput-program) | `src/store/sparq.rs` (`read_plan`), `src/store/mod.rs`, `src/authz/wac.rs` (`read_plan_candidates`), `src/store/body_cache.rs`, `src/store/counting.rs`, `tests/read_path_counters.rs`, `tests/write_path_counters.rs` |
| RSS `docs/design/beyond-50k-throughput.md` (§4 P1.3–P1.6, §5) | [§7](#7-the-throughput-program) | `src/tls.rs`, `src/nodelay.rs`, `tests/tcp_nodelay.rs`, `tests/response_write_coalescing.rs`, `tests/embedded_read_counters.rs` |
| RSS `docs/design/high-throughput-pop-auth.md` (§4–§6 DPoP-SK, §7 bead 2) | [§7](#7-the-throughput-program) | `src/pop/mod.rs`, `src/pop/cert_bound.rs`, `src/pop/conn.rs`, `src/pop/sk/`, the mTLS env var in `src/tls.rs` |
| RSS `docs/design/throughput-hard-cases.md` (§5 `read-4-bodycache`) | [§7](#7-the-throughput-program) | `src/store/body_cache.rs` |
| PSS `decisions/0020` | [§4](#4-provider-webids-outside-the-pod--the-identity-host) | The convention RSS adapted; the adaptation lives in `src/identity.rs` |
| PSS `decisions/0001` | — | Referenced only in `src/lib.rs`, as the example that makes the RSS/PSS numbering collision concrete. No code depends on it. |

Paths in the right-hand column are relative to `crates/sparq-lws-core/`.

---

## 2. What this record is — and what it is not

**It is** the resolution target for the in-tree citations, plus a summary of each decision as
implemented, sufficient to read the code without the source repo to hand.

**It is not** a copy of the source documents. The RSS/PSS prose — the alternatives weighed, the
measurements taken there, the rejected options — was not imported and is **not** reproduced or
paraphrased here from memory. Where the full argument survives in this repo it survives in the
module doc-comments, which several modules explicitly designate as the durable record
(`src/pop/mod.rs`, `src/pop/conn.rs`, `src/pop/sk/mod.rs`). Where it does not, the source
repository remains the only copy. Section numbers written as `RSS docs/design/<file>.md §N` are
sections of **that upstream file**, not of this record.

Bringing the RSS `docs/` + `decisions/` trees into this repo is tracked under the `sq-gg0qq`
epic; until such a bead lands, this record is the in-repo index and nothing here should be read
as a transcription of the originals.

---

## 3. Embedded SPARQ, in-process — RSS `decisions/0001`

The `SparqClient` seam (authoritative RDF + WAC metadata) is satisfied by calling the
`sparq-engine` query/update entry points **directly against an in-process `Graph`**, rather than
over SPARQ's HTTP service.

- **Feature/selection.** `embedded-sparq` (default-on since `sq-gg0qq.3`, because the crate now
  lives in the sparq workspace) pulls in `sparq-core` + `sparq-engine`; `--no-default-features`
  builds the engine-free profile. Runtime selection stays **explicit** via
  `PSS_SPARQ_BACKEND=embedded` so an unconfigured boot serves the in-memory double rather than a
  store the operator did not choose. The remote shape lives behind the opt-in `http-sparq`
  feature.
- **Why it simplifies.** The same injection-safe query builders in `src/store/sparql.rs` are used
  verbatim, so conformance-equivalence across the impls is a transport difference only; named-graph
  isolation is real in-process; and the HTTP impl's marker + follow-up-ASK atomicity dance
  disappears because the operation runs as one indivisible job.
- **The sync↔tokio bridge.** The engine is blocking and CPU-bound, so the `Graph` is owned by a
  dedicated OS thread and every call is a job sent to it over a channel, awaited via `oneshot` —
  the reactor is never blocked.

Full record: the module docs of `src/store/embedded.rs`.

---

## 4. Provider WebIDs OUTSIDE the pod — the identity host

RSS `docs/design/webid-outside-pod.md`, the RSS adaptation of PSS `decisions/0020`.

The WebID document is the Solid-OIDC identity trust root — every resource server dereferences it
to learn which issuers may mint tokens for that WebID. Hosting it inside the pod, as a
WAC-governed owner-writable resource, leaves it one over-broad `acl:default` grant away from
ecosystem-wide identity takeover. The separation is therefore structural:

- **Form.** `https://<identity-host>/<handle>#me`; default host `id.<base authority>`.
- **Storage.** Id-docs live under the reserved `<base>/.identity/<handle>` namespace, which is
  outside the LDP-resource→storage mapping: no containment edge, no `ldp:contains` listing, and
  the LDP surface refuses the whole subtree (`is_reserved_identity_path` — 404 for every method,
  every origin, %-decoded, **regardless of the identity feature flag**). No `.acl` can exist for
  it, so no WAC grant can apply. That impossibility, not a policy check, is the property.
- **Serving.** The Host-keyed `identity_gate_middleware` is the outermost layer: `GET`/`HEAD` only
  with Turtle/JSON-LD conneg, ETag/304, public cache headers, explicit CORS; no WAC evaluation, no
  `.acl` Link, no auth processing; every other method → 405; anything not exactly one valid
  non-reserved handle → 404, fail-closed.
- **Writes.** Only through the `Store` seam (boot seeding today, a future admin provisioning
  seam) — never through the LDP path.
- **Flag posture.** `SOLID_SERVER_IDENTITY_ENABLE` is default OFF and gates only the *serving* +
  the seed's choice of WebID; the reserved-namespace refusal is unconditional either way, so
  pre-seeded documents can never become LDP-addressable when the flag later turns on.

Full record: the module docs of `src/identity.rs`; the chokepoint rationale in
`src/ldp/target.rs`; the seeded document shape in `src/seed.rs`.

---

## 5. The pre-crypto PUBLIC-READ skip — RSS `decisions/0002`

Opt 3 of the skip-crypto options. For a read whose response is **fully identity-independent**, a
thin middleware sitting just inside CORS and just outside auth constructs
`VerifiedToken::public()` and delegates straight to the same `serve_read` the handler uses.

**The hard scope limit is load-bearing and security-critical:** the skip fires only for a
`GET`/`HEAD` carrying **neither an `Authorization` nor a `DPoP` header**. A credentialed request
is never short-circuited, for two independent reasons — both proven by the WAC-Allow conformance
suite, which an earlier "serve any proof-carrying public read as anonymous" attempt failed:

1. **`WAC-Allow user=` is identity-dependent.** An authenticated owner of a public resource holds
   `read/write/control`, not the public `read`; serving them as anonymous under-reports their
   access. Computing the correct `user=` needs the verified WebID — i.e. the crypto.
2. **A forged proof is indistinguishable from an owner's without the crypto.** Serving forged
   proofs as anonymous would mean serving *every* proof-carrying public read as anonymous,
   including the owner's — which is exactly the wrong behaviour in (1).

So the credentialed variant cannot be both correct and safe; opt 3 is scoped to the genuinely
anonymous case. The `DPoP` header is also a fall-through trigger, so a malformed `DPoP` on an
uncredentialed GET keeps the auth path's canonical 400. Mutations always fall through.

Delegating to `serve_read` (rather than pre-checking, then serving) keeps it to **one** effective-ACL
resolution and makes the response byte-identical to the full anonymous path — it is the same call.

Full record: the module docs of `src/ldp/public_read_skip.rs`, whose stated invariants
(anonymous-equivalence, identity-independence) are pinned by `tests/public_read_skip.rs`; the
layer placement is argued in `src/app.rs`.

---

## 6. Existence non-disclosure — RSS `decisions/0003`

**The rule:** a `404` is served only to a requester who holds the operation's required mode.
Every other requester — anonymous → 401, authenticated-but-unauthorized → 403 — gets their
denial code for **both** the missing and the existing target, so the status carries no existence
signal. The closures are numbered V1–V6 in the code:

| Vector | Channel closed |
| --- | --- |
| **V1** | Create-vs-forbidden-overwrite on PUT: `acl:Write` is required on the **target**'s effective ACL regardless of existence, so create and overwrite are indistinguishable to an under-authorized requester. Authorization runs *before* any target-dependent probe, which also closes the timing variant. |
| **V2** | The POST `Location` header: the minted child IRI is always an opaque-suffixed, collision-**independent** name, never the verbatim slug — the old free-vs-taken shape difference leaked which child names exist. |
| **V3** | The PATCH timing variant: the required mode is derived purely from the already-parsed patch content and authorized before any target read. |
| **V4** | The conditional channel: `If-Match`/`If-None-Match` evaluate against a content- or membership-derived ETag, so their 412-vs-2xx outcome is an existence/content oracle. Folded to the denial when the requester lacks `acl:Read` and sent a conditional header — before the existence probe. Same gate applies to a patch carrying a `solid:where` clause, which reads the target graph. |
| **V5** | The container ETag: membership-derived, so it shifts on every child add/remove and is a listing oracle. Exposed only on the GET/HEAD read path, which `authorize_read` already gates on `acl:Read` for the container. |
| **V6** | The POST descendant-existence branch (405 when a resource is present, 404 when absent). Read-gated by `guard_post_existence_requires_read`, before the existence probe, so the deny path performs no target-dependent lookup. |

**Trade-off (recorded, not incidental):** an `acl:Append`-only agent can no longer PUT-create; it
must POST, which mints a server-opaque collision-free name. This is CTH-safe — no conformance row
expects an Append-only PUT-create to succeed.

**Residual (known, WAC-inherent):** a requester holding Read via inheritance who is separately
denied on a specific existing child by that child's own restrictive `.acl` can still distinguish
that child (403) from a missing one (404). A per-child `.acl` legitimately overriding inheritance
is WAC's own semantics.

Full record: the guards and the exhaustive byte-identical status matrix in `src/ldp/handler.rs`.

---

## 7. The throughput program

Four upstream documents feed one program; the in-repo discipline they share is the repo's
perf-gate rule — **deterministic metrics are hard-pinned, wall-clock is advisory and never
asserted.**

### 7.1 Backend round-trips — RSS `docs/design/backend-read-path.md`

- **read-1 (§7) — measure first.** `CountingSparqClient` / `CountingBlobStore` are transparent
  decorators counting calls per trait method at the `SparqClient`/`BlobStore` seams; zero-cost
  when not wired in. `tests/read_path_counters.rs` pins **exact integer** per-operation counts
  end-to-end through the assembled router, with `max_in_flight == 1` as the await-depth witness
  that the pinned totals equal the operation's sequential RTT depth.
  `tests/write_path_counters.rs` is the write-verb sibling and the measure-first baseline for the
  write-path walk-collapse.
- **read-2 (§3.1) — collapse the ACL walk.** `Store::read_plan` / `SparqClient::read_plan` answer
  the target's authoritative metadata **and** the presence/etag of every ACL candidate on its
  resolution chain in **one** index round-trip, replacing the sequential child→root probes.
  Candidates are derived up front as pure string work by `WacAuthorizer::read_plan_candidates`
  (nearest-first: element 0 is the protected resource's own ACL, scope `AccessTo`; the rest are
  ancestors', scope `Default`). Two IRI roles are load-bearing for `.acl` targets — a GET of
  `foo.acl` is governed by Control on `foo`. Fail-closed on any backend error.
  **write-2** applies the same plan to PUT/POST/DELETE/PATCH via `authorize_planned`, whose
  in-memory walk plus live found-ACL re-confirm is differentially tested bit-for-bit against the
  sequential walk for every `AccessMode`.
- **read-4 (§3.4, and RSS `throughput-hard-cases.md` §5 `read-4-bodycache`) — the blob-body LRU.**
  A byte-budgeted cache of resource **bodies** keyed by `(blob_key, etag)`, in front of
  `BlobStore::get` inside `CompositeStore`. It needs **no invalidation protocol**: `mint_blob_key`
  mints a fresh 128-bit-random key on every write, so a blob object never changes after creation
  and every lookup is keyed by *this* request's authoritative index metadata — a rewrite is a
  guaranteed miss, and the etag in the key is defence-in-depth. It also cannot bypass
  authorization: it sits inside the store, strictly below the WAC gate, and `serve_read` runs
  `authorize_read` before ever asking for bytes. Budgets are tunable via
  `SOLID_SERVER_BODY_CACHE_BYTES` / `..._MAX_ENTRY_BYTES`; `0` disables.

### 7.2 Connection + transport — RSS `docs/design/beyond-50k-throughput.md`

- **P1.3 — connection amortization (`src/tls.rs`).** The rustls session-resumption cache is
  sized by `SOLID_SERVER_TLS_SESSION_CACHE_SIZE` (`0` disables resumption), and an opt-in,
  default-off stateless `Ticketer` (random per-process keys, rotated, forward-secret by key
  erasure) is the stateless half. Default-off is deliberate: a per-process ticket key is not
  shared across a scaled fleet, so cross-node resumption falls back to a full handshake — a perf,
  never a correctness, effect. **0-RTT early data is never enabled:** `max_early_data_size` is
  forced to `0` on every path including with the ticketer, because 0-RTT is replayable by design
  and incoherent under this server's anti-replay DPoP `jti` model (§5 of the source doc). A
  `debug_assert` pins the value so a future rustls default change surfaces in tests. Ticketer
  construction is fail-safe — an RNG error logs and falls back to the stateful cache.
- **P1.4 — response-write coalescing.** `tests/response_write_coalescing.rs` pins the number of
  write-family syscalls the HTTP/1.1 response path emits per response, measured at the
  hyper→transport seam while driving the real assembled router over the same `axum::serve` path
  production uses, on both the anonymous (skip) and DPoP-authenticated routes.
- **P1.5 — `TCP_NODELAY` on both serve paths.** The deterministic metric is the socket-option
  *state* read back via `TcpStream::nodelay()`, not any latency. The plain path taps accepted
  streams with `nodelay::tap_nodelay`; the TLS path composes `nodelay::NoDelayAcceptor` inside the
  `RustlsAcceptor` so the option is set on the raw stream before the handshake.
  Pinned by `tests/tcp_nodelay.rs`.
- **P1.6 — counters on the real engine.** `tests/embedded_read_counters.rs` re-proves the
  seam-level round-trip counts against the in-process `EmbeddedSparqClient`, so a regression that
  adds a round-trip on the embedded path fails. Gated on `embedded-sparq`; a no-op binary when off.

### 7.3 Tiered proof-of-possession — RSS `docs/design/high-throughput-pop-auth.md`

DPoP (RFC 9449) stays the **mandatory** Solid-OIDC baseline. The tiers are negotiated, opt-in fast
paths, individually refusable, and never a silent downgrade.

- **T1a** — the confirmation dispatch plus the RFC 8705 `cnf.x5t#S256` cert-binding verification
  core (`src/pop/mod.rs`, `src/pop/cert_bound.rs`), transport-agnostic and unit-testable in
  isolation, fail-closed (a cert-bound token with no visible cert is rejected).
- **T1b** — the transport half (`src/pop/conn.rs`): read the peer's client certificate **once per
  connection**, compute its thumbprint, and expose it to every request on that connection.
  Enabled by `SOLID_SERVER_MTLS_BOUND_TOKENS`, **default OFF** — when unset the serve paths are
  byte-identical to the pre-T1b behaviour.
- **T2 — DPoP-SK** (`src/pop/sk/`): negotiated symmetric session keys for DPoP-bound requests,
  against the [DPoP-SK draft](https://jeswr.github.io/dpop-sk-spec/) whose Appendix-A worked
  example the implementation reproduces byte-for-byte in tests.

These three module doc-comments are explicitly the durable record of the tiering design; the
source document's §4–§7 were not carried over by the import.
