# LWS live-demo architecture — ephemeral, scale-to-zero test environment on Cloud Run (sq-cepjb)

> Design record for the maintainer-requested bead **sq-cepjb** — a live, public, ephemeral
> test environment for the sparq Solid/LWS server (`crates/sparq-lws-core`) with a bundled
> throwaway IdP. `[FABLE]`
>
> **Status: ratified, mostly designed-only.** The three judgment calls in this record (§0
> seed flag, §0/§5 no per-visitor isolation, §2 CSS + two services) were surfaced post-hoc
> via proceed-and-document issue #2329; the steering window closed with no maintainer
> redirect, so they stand as written — including the §6 bundled-IdP-over-fallback trade.
> Since this record merged, a `SOLID_SERVER_SEED_DEMO` flag has landed on main (PR #2403,
> target issue #2393) with a DIFFERENT, local-demo contract than §3.2 — see the §3.2
> divergence note; bead B is now an alignment bead. Nothing else (manifests, compat smoke,
> site section, deploy) is implemented or deployed. This is a THROWAWAY
> demo design: **its auth is throwaway, not production identity**, and the LWS server itself
> boots with an "EXPERIMENTAL parallel track" banner (`main.rs`). Every claim below is either
> verified against origin/main / upstream source (cited) or explicitly marked
> designed-only/to-be-validated.

## 0. Premise check (verified against origin/main) + one load-bearing correction

**Locked decisions from the maintainer** (taken as given): platform = Google Cloud Run,
`min-instances=0` scale-to-zero, `max-instances=1` (in-memory DPoP replay store is
per-instance); ephemerality via in-memory storage + scale-to-zero; auth via a **bundled
throwaway Solid-OIDC IdP** so visitors get ephemeral identities.

**What already exists** (further along than the bead brief assumed):

- `deploy/gcp/sparq-lws.yaml` (sq-agolp, merged) is a production-shaped Cloud Run manifest:
  `minScale: 1`, `maxScale: 1`, custom HTTP probes, instance-based billing. The demo wants a
  **different posture** (`minScale: 0`, ~$0 idle — §2.4), so the demo gets its own
  `deploy/demo/` subtree and `deploy/gcp/` stays the production template. No file overlap.
- The LWS image pipeline exists (`.github/workflows/lws-container.yml`: publish on `v*` tag
  or `workflow_dispatch`, PR #2327), **but `ghcr.io/sparq-org/sparq-lws-core:latest` is NOT
  publicly pullable today** (verified: anonymous GHCR manifest request returns 403). The demo
  is hard-gated on one publish run + package-visibility=public — a maintainer/credentialed
  action, tracked as a dependency, not a code bead.

**Corrected premise — "visitors isolated by WAC/WebID" does not hold on current main.**
The bead (and the dispatch brief) assume a bundled IdP is sufficient for visitors to *use*
the server. It is not, for two verified reasons:

1. **WAC is live and fail-closed.** With no `.acl` anywhere up to the storage root, access is
   DENIED — including reads, including *any* authenticated agent
   (`src/authz/wac.rs:16`, `src/authz/acl.rs:82-121`). A fresh in-memory boot has no ACLs.
2. **There is no runtime provisioning.** Accounts/pods/WebID docs are created only by
   boot-time seeding, gated behind the dev/conformance-only flags
   (`src/seed.rs:1-32`, `SOLID_SERVER_SEED_CONFORMANCE`); the identity gate serves id-docs
   written "by boot seeding today and by the future admin provisioning seam"
   (`src/identity.rs:1-29`). No create-account or create-pod endpoint exists.

So with a bundled IdP alone, a visitor can authenticate and then do **nothing** — every LDP
request 401/403s. The minimal sound fix (chosen; §3.2) is a new, opt-in, demo-labeled boot
seed (`SOLID_SERVER_SEED_DEMO`) that creates a shared `/playground/` container whose ACL
grants `acl:AuthenticatedAgent` Read/Write/Append (no Control) and the public Read. The
consequence is stated plainly: **v1 visitors share one playground and are NOT isolated from
each other by WAC** — they are only isolated from *anonymous* writers. True per-visitor pods
require the runtime-provisioning seam (a real LWS feature) and are explicitly out of scope
(§6). This correction, and the two other judgment calls in this record, were surfaced to the
maintainer via proceed-and-document issue #2329; the steering window closed with no
redirect, so all three decisions stand.

> On "no dev escape hatches": the demo manifests still ban `SOLID_SERVER_ALLOW_LOOPBACK`,
> `SOLID_SERVER_SEED_CONFORMANCE`, `SOLID_SERVER_SEED_BENCH`, and
> `SOLID_SERVER_ALLOW_SEED_NONMEMORY` (§4.3). `SOLID_SERVER_SEED_DEMO` is a NEW, purpose-built,
> clearly-labeled demo mode — off by default, allowed only against the `memory` backend by the
> existing startup seed-guard, and documented as never-production. It is a smaller hole than
> repurposing the conformance seed and the only alternative to shipping a demo nobody can use.

## 1. Which throwaway IdP — Community Solid Server (chosen)

### 1.1 What the verifier actually demands of an issuer

LWS delegates verification to `solid-oidc-verifier` (jeswr's crate, pinned rev `89c8962`,
`Cargo.toml:226`). Requirements verified at that rev:

| Requirement | Detail | Source |
|---|---|---|
| Token `typ` | `at+jwt` (RFC 9068) enforced | `src/verifier.rs` (enforced typ), token tests |
| Required claims | `iss`, `sub`, `aud`, `exp`, `iat`, `jti`, `client_id`, `webid`; `cnf.jkt` for DPoP binding | `verifier.rs` RFC 9068 §2.2 checks |
| `aud` semantics | configured audience must equal the string OR be contained in the array | `verifier.rs:734-740` (`audience_matches`) |
| Algorithms | ES256/ES384/PS256/PS384/PS512/RS256/RS384/RS512/EdDSA verifiable; ES512 rejected unless the `es512` feature | `src/jwk.rs:20-66` |
| DPoP | proof mandatory per request: `typ=dpop+jwt`, `htm`, `htu`, `iat`, `jti` (single-use, replay-stored), `ath`; thumbprint must match token `cnf.jkt` | `verifier.rs` DPoP path |
| Discovery | `<issuer>/.well-known/openid-configuration` → `jwks_uri` → JWKS; issuer field must match the configured trusted issuer exactly (trailing-slash sensitive) | `NetworkJwksProvider` |
| SSRF posture | HTTPS-only, no loopback/private IPs, DNS-pinned fetch (dev-only `SOLID_SERVER_ALLOW_LOOPBACK` relaxes) | `src/ssrf.rs`, `main.rs:112-114` |
| Bidirectional | fetches the `webid` doc (Turtle/JSON-LD), requires `<webid> solid:oidcIssuer <iss>`, exact issuer match; `strict` (default) 401s on mismatch/fetch failure | `NetworkWebIdResolver`, `main.rs:117-118` |

So the bundled IdP must be a **real Solid-OIDC provider**: public HTTPS discovery + JWKS,
RFC 9068 DPoP-bound access tokens with a `webid` claim, AND publicly-dereferenceable WebID
documents pointing back at itself. That is a substantial contract — it rules out "a JWKS file
on a static host" immediately.

### 1.2 Candidates

**(a) Community Solid Server (CSS) — chosen.** CSS bundles a complete Solid-OIDC IdP
(node-oidc-provider underneath) plus account registration UI plus WebID/pod hosting.
Compatibility, verified from CSS source (`src/identity/configuration/IdentityProviderFactory.ts`,
upstream main):

- `accessTokenFormat: 'jwt'` — node-oidc-provider emits RFC 9068 `typ: at+jwt` JWTs for this
  format, with `iss/sub/aud/exp/iat/jti/client_id`;
- `audience: 'solid'` — passes `audience_matches` **iff** LWS sets
  `SOLID_SERVER_AUDIENCE=solid` (the default audience = base URL would reject every CSS
  token — a mandatory demo config line, §2.3);
- `extraTokenClaims` injects `webid` (and `client_id` for client-credentials tokens);
- asymmetric signing (`jwtAlg`, ES256 by default) — inside the verifier's accepted set;
- registered accounts get a pod + WebID profile containing `solid:oidcIssuer` → CSS itself,
  publicly dereferenceable — satisfying bidirectional `strict`;
- DPoP-bound tokens are CSS's normal mode (Solid-OIDC requires DPoP);
- the official image (`docker.io/solidproject/community-server`) with the **default config
  stores everything in memory** — ephemeral accounts for free — and takes the public base URL
  via `-b`/`CSS_BASE_URL`.

**(b) Minimal node-oidc-provider + static WebID host — rejected on effort.** To satisfy §1.1
it would need: the `webid` extra claim, `at+jwt` format config, DPoP enabled, a user store,
an interaction/registration UI (visitors must self-serve throwaway identities), and a WebID
document host wired per-account. That is a hand-rolled subset of exactly what CSS ships and
maintains, with all the compatibility risk moved onto us. No advantage except image size.

**(c) Keycloak — rejected.** The verifier repo carries a `keycloak_it.rs` integration test,
so a Keycloak realm CAN be shaped to verify (webid protocol-mapper). But Keycloak hosts no
WebID documents (bidirectional would need a separate WebID host), and a JVM cold-start on a
scale-to-zero instance is the worst-case wake path. Wrong tool for a throwaway Solid demo.

**Honest status:** CSS-satisfies-the-verifier is **designed-only** until the compatibility
smoke (bead §7-C) passes against a real CSS-minted token. The known drift risks are pinned by
that smoke: `aud` value/shape, `typ`, `webid` claim presence, issuer string byte-equality
(CSS normalizes its base URL — the trailing slash must match `SOLID_SERVER_TRUSTED_ISSUER`
exactly), and bidirectional profile content. Do not claim the pairing works before that smoke
is green.

## 2. Topology under scale-to-zero — two Cloud Run services (chosen)

### 2.1 Verified platform facts

- Cloud Run multi-container services route **all ingress to exactly one container**; sidecars
  are reachable only via localhost/shared volumes (Cloud Run container contract + sidecar
  launch docs). Path-routing `/idp` to a sidecar therefore requires a reverse-proxy ingress
  container.
- Scale-to-zero (`min-instances: 0`) applies per service; idle instances are reclaimed
  **heuristically, typically within ~15 minutes, not contractually** — the maintainer's
  "~10-min wipe" is approximate. If exact idle-timeout control ever matters, Fly.io Machines
  (`auto_stop_machines` + configurable idle timeout) is the alternative platform; not needed
  for v1.
- Each service gets a deterministic public HTTPS URL
  (`https://<service>-<project-number>.<region>.run.app`) resolving to Google's public front
  end — which satisfies the verifier's SSRF gate (public IPs, real HTTPS).

### 2.2 Option comparison

| | (a) one multi-container service | (b) two services — **chosen** |
|---|---|---|
| URL shape | one URL, IdP at `/idp` | two URLs (LWS + IdP) |
| Extra artifacts | reverse-proxy ingress container (a third image to build, publish, patch) | none — stock GHCR LWS image + stock upstream CSS image |
| CSS base URL | subpath base URL (`…/idp`) — off CSS's happy path, real config risk | root base URL — CSS's default posture |
| Wipe semantics | atomic (accounts + data die together) | independent (see §3.3 — all states fail closed) |
| Cold start | one instance wakes with both containers | visitor wakes CSS first (registration/login precedes any LWS token), so the JWKS fetch almost always hits a warm IdP; worst case one extra seconds-scale wake per JWKS-TTL |
| `max-instances=1` | shared instance, shared resource caps | one per service, each sized independently |
| Self-fetch path | LWS fetches JWKS via its own public URL back into its own instance (works, but a needless re-entrancy) | plain cross-service fetch |

The single-service option's only real advantage is atomic wipes; it costs a new proxy artifact
and a CSS-subpath risk. Two services keep every component stock. **Chosen: (b).**

### 2.3 Demo configuration (the load-bearing lines)

`sparq-lws-demo` service (image `ghcr.io/sparq-org/sparq-lws-core`, digest-pinned at deploy):

```yaml
# scaling: min 0 / max 1 (in-memory replay + store are per-instance)
SOLID_SERVER_BASE_URL:        https://sparq-lws-demo-<n>.<region>.run.app
SOLID_SERVER_TRUSTED_ISSUER:  <the CSS service URL, byte-identical to its discovery `issuer`>
SOLID_SERVER_AUDIENCE:        solid          # REQUIRED: CSS tokens carry aud="solid" (§1.2)
SOLID_SERVER_TRUSTED_PROXY:   "1"            # REQUIRED: rate-limit by real client IP (§4.2)
SOLID_SERVER_SEED_DEMO:       "1"            # the new demo playground seed (§3.2)
PSS_SPARQ_BACKEND:            memory         # ephemeral by construction (or embedded w/o dir)
# rate/body/timeout tightenings per §4.2; TLS unset (Cloud Run terminates HTTPS)
# NOT SET, ever: ALLOW_LOOPBACK, SEED_CONFORMANCE, SEED_BENCH, ALLOW_SEED_NONMEMORY
```

`css-idp` service (image `docker.io/solidproject/community-server`, default in-memory
config): `-b https://css-idp-<n>.<region>.run.app`, port 3000, min 0 / max 1. If the default
config needs any override that CLI flags cannot express, the contingency is a tiny derived
image (`FROM solidproject/community-server` + one config JSON) — flagged in bead §7-A, avoided
if possible.

### 2.4 Demo posture vs the production template

`deploy/gcp/sparq-lws.yaml` keeps `minScale: 1` + custom HTTP probes (instance-based billing).
The demo intentionally differs: `minScale: 0`, default TCP startup probe only (keeps
request-based billing ⇒ ~$0 when idle and no idle-warm billing), no liveness probe (a wedged
instance is reclaimed on the next scale event; acceptable for a throwaway). This asymmetry is
documented in `deploy/demo/README.md` so nobody "fixes" the demo back to the production shape.

## 3. Ephemeral accounts, provisioning, and wipe-on-idle

### 3.1 Visitor flow

1. Visitor opens the CSS URL (linked from the site's `/deploy` demo section), registers a
   throwaway account (email is not verified — any string works), and receives a WebID +
   profile hosted by CSS.
2. Visitor uses any standard Solid app (or the copy-paste `curl`+DPoP snippet in the demo
   README, or CSS's client-credentials API for scripted use) to log in via CSS and obtain
   DPoP-bound tokens.
3. Visitor reads/writes under `https://<lws-demo>/playground/` — authenticated writes are
   accepted by the playground ACL; anonymous writes stay rejected (fail-closed LDP + WAC).

### 3.2 The demo seed (new, small LWS feature — bead §7-B)

Opt-in `SOLID_SERVER_SEED_DEMO=1`, following the existing seed architecture (`src/seed.rs`):

- creates `/playground/` with an ACL granting `acl:agentClass acl:AuthenticatedAgent`
  Read/Write/Append with `acl:default` inheritance, and `foaf:Agent` Read; **no
  `acl:Control`** — a visitor cannot lock others out or open the pod wider;
- creates a public-read `/README` (Turtle) stating the banner text: ephemeral demo, all data
  public-readable, wiped on idle, throwaway identities, no isolation between visitors;
- refuses non-`memory` backends via the existing startup seed-guard #2 (`main.rs`)
  — fail-closed against ever seeding a durable store;
- unset ⇒ byte-identical server behaviour (the feature-off-by-default invariant).

> **Divergence on current main (post-record; found while closing #2329).** PR #2403 (target
> issue #2393) shipped a `SOLID_SERVER_SEED_DEMO` flag with a different, LOCAL-demo contract:
> a public-readable `/demo/` pod plus an **anonymous** read+write open sandbox at
> `/demo/playground/` (`foaf:Agent` gets `acl:Read`+`acl:Write` via `acl:accessTo` +
> `acl:default`; owner-only `acl:Control`) so a local boot is usable with no IdP at all
> (`seed.rs` `seed_demo`, `main.rs` `ENV_SEED_DEMO`). The seed-guard #2 and off-by-default
> invariants above DO hold for it. What does not hold is the write posture: it is fine for a
> local no-IdP boot, but behind a public URL an anonymous-writable container drops the only
> write friction this design relies on (registration, §4) and contradicts §3.1's "anonymous
> writes stay rejected" (and the §5 item-2 isolation-from-anonymous-writers claim).
> **Bead B is therefore an alignment bead, not greenfield:** bring the flag's public-demo
> posture to the `acl:AuthenticatedAgent` R/W/A + public-Read, no-Control contract above
> (or split the local anonymous sandbox onto its own dev flag) BEFORE any manifest (bead A)
> sets `SOLID_SERVER_SEED_DEMO`.

### 3.3 Wipe-on-idle — what "free" actually buys

In-memory everything (LWS store + replay store; CSS accounts + keys) + scale-to-zero means an
idle reclaim wipes all state. Honest print:

- Wipe timing is **heuristic ≈15 min, not a guaranteed 10** (§2.1). Good enough here.
- The two services wipe **independently**. Every mixed state fails closed: CSS wiped → its
  signing key is gone, old tokens fail signature and WebID docs 404 the bidirectional check ⇒
  401, visitor re-registers. LWS wiped → tokens still verify but the playground is re-seeded
  empty. The demo banner says: "if anything 401s or vanishes, re-register — that is the demo
  working as designed."
- **DPoP replay caveat:** the in-memory `jti` replay store dies with the instance, so a
  captured proof could be replayed across an instance recycle within the proof-freshness
  window. Bounded, throwaway-acceptable, **not a production posture** — stated in §5.

## 4. Public-demo abuse guard

### 4.1 Platform layer

`max-instances: 1` on both services caps compute blast radius; request-based billing +
scale-to-zero caps idle cost at ~$0; a GCP budget alert on the project is recommended in the
demo README (maintainer action).

### 4.2 LWS layer — one non-obvious required setting

LWS ships a default-on pre-crypto per-IP token bucket (`src/rate_limit.rs`) — but behind
Cloud Run the direct TCP peer is Google's front end (an internal address), and internal peers
are **exempt by default**, so the limiter would be effectively inert. The demo manifest MUST
set `SOLID_SERVER_TRUSTED_PROXY=1` so the client IP is taken from `X-Forwarded-For` (Cloud
Run appends the real client IP; exactly one trusted hop). With that, the manifest tightens the
generous defaults via env (`SOLID_SERVER_RATE_LIMIT_PER_IP` / `_BURST`) to demo-appropriate
values, plus the default 2 MiB body cap and 30 s request timeout. These are policy/config
values in the manifest, not code changes and not performance claims.

CSS-side abuse (registration spam) is bounded by `max-instances: 1`, the instance's memory,
and the wipe cycle; CSS ships no CAPTCHA and we add none — accepted for a throwaway and
listed in §5.

### 4.3 No escape hatches — mechanically checked

`deploy/demo/check.sh` (bead §7-A acceptance) asserts, over both manifests: none of
`SOLID_SERVER_ALLOW_LOOPBACK`, `SOLID_SERVER_SEED_CONFORMANCE`, `SOLID_SERVER_SEED_BENCH`,
`SOLID_SERVER_ALLOW_SEED_NONMEMORY` appears; `SOLID_SERVER_AUDIENCE=solid`,
`SOLID_SERVER_TRUSTED_PROXY=1`, `minScale: 0`, `maxScale: 1` do appear; no secret literals.

### 4.4 Banner

Three placements: the seeded public `/README` resource (§3.2), the demo section on the site's
`/deploy` page (bead §7-D), and `deploy/demo/README.md`. CSS's own registration UI keeps CSS
branding — fine for a throwaway; re-theming it is explicitly out of scope.

## 5. Honest caveats (the demo's contract with visitors)

1. **Throwaway auth, not production identity.** Unverified registration, in-memory accounts,
   keys rotate on every wipe. Nothing about this demo attests the production-readiness of any
   identity flow.
2. **No per-visitor isolation in v1.** All visitors share one instance and one playground;
   any authenticated visitor can read/modify/delete any playground resource. WAC isolates
   authenticated visitors only from *anonymous* writers. (Corrects the brief's
   "isolated by WAC/WebID".)
3. **`aud: "solid"` audience posture.** Per Solid-OIDC, a CSS token is not
   audience-restricted to this RS. Accepted for the demo; noted because it is weaker than the
   verifier's default RS-specific audience.
4. **Replay-store wipe window** (§3.3): DPoP `jti` single-use is per-instance-lifetime.
5. **Image dependency.** Blocked on `ghcr.io/sparq-org/sparq-lws-core` being published +
   public (verified not pullable today); the demo pins a digest once it exists.
6. **Wipe timing is heuristic** (§2.1), and an idle instance may occasionally live longer.
7. **The LWS server is itself experimental** (its own boot banner says so); the demo is a
   test environment, not an availability or durability commitment. Cold wakes are
   seconds-scale (work-box observation class, non-canonical — do not quote a number).

## 6. Effort/scope verdict

**This is a real mini-project, not a config tweak.** Bundled-IdP demo = 4 disjoint beads:
one small Rust feature in `sparq-lws-core` (the demo seed — unavoidable, §0 correction), two
config/harness surfaces (manifests + the CSS-compat smoke that de-risks §1), one site link.
Plus the external image-publish dependency. Roughly: the seeded-read-only fallback would have
been **one** config-only bead (single Cloud Run service, public-read seed, no IdP, no writes);
the bundled-IdP choice buys real interactivity (visitors write) at ~4× the surface count and
one new server flag. The maintainer's choice stands — it is viable and now fully de-risked on
paper — but the trade is stated so it can be reversed cheaply before implementation if ~$0
interactivity is not worth four beads. Recommended order: B (seed) → C (compat smoke — the
go/no-go on CSS) → A (manifests) → D (site); if C fails against real CSS tokens, fall back to
seeded-read-only having spent only B+C.

## 7. Child beads (disjoint; each single-surface)

| # | Bead | Surface (file-area) | Tier | Invariant | Acceptance |
|---|---|---|---|---|---|
| B | feat(lws-core): align the shipped `SOLID_SERVER_SEED_DEMO` seed to the §3.2 public-demo ACL contract (see the §3.2 divergence note) | `crates/sparq-lws-core/**` (seed.rs, main.rs, README, tests) | sonnet | off-by-default byte-identical; memory-backend-only via seed-guard; ACL grants AuthenticatedAgent R/W/A + public Read, NO Control | `cargo test -p sparq-lws-core demo_seed` (authed PUT 201 / anon PUT 401 / anon GET README 200 / authed ACL write denied / flag-off ⇒ no playground) |
| A | deploy(demo): Cloud Run manifests for LWS demo + CSS IdP | `deploy/demo/` root (yamls, README.md, check.sh) — excludes `compat/` | sonnet | fail-closed demo posture: forbidden env absent, required env present, min 0 / max 1, no secret literals | `bash deploy/demo/check.sh` |
| C | test(demo): CSS↔LWS Solid-OIDC compatibility smoke | `deploy/demo/compat/**` (docker-compose, smoke.sh) | sonnet | non-vacuous proof of the §1 pairing on REAL CSS-minted DPoP tokens (goes red on aud/typ/webid/bidirectional drift); loopback allowed in this local harness ONLY | `bash deploy/demo/compat/smoke.sh` |
| D | site(deploy): demo section — link + honest banner | `site/src/app/deploy/**` | haiku | static export green; banner states §5 items 1-2, 6 plainly; no perf numbers | site lint + typecheck + static export |

Ordering (real edges only): **B → A** (the manifest sets the env B introduces — and the
published image must contain it), **B → C** (the smoke's write-path needs the playground),
**A → D** (the site section links A's README/URLs). All four are parent-child under
`sq-cepjb`. Disjointness: the four file-areas share no file; `deploy/demo/` root vs
`deploy/demo/compat/` are separate subtrees with separate READMEs.

Not beaded (external dependencies / maintainer actions): the GHCR publish + public-visibility
run for the LWS image; the actual `gcloud` deploy into the maintainer's GCP project (needs
credentials); the budget alert.

---

*Authored by a SPARQ agent 🤖 (Fable tier). Grounded against origin/main
`crates/sparq-lws-core` (main.rs / authz / seed / identity / rate_limit / Dockerfile),
`deploy/gcp/`, `.github/workflows/lws-container.yml`, the pinned `solid-oidc-verifier` rev
`89c8962` source, CSS upstream source (`IdentityProviderFactory.ts`), and Cloud Run
documentation. Designed-only; per-child verify/arm is downstream.*
