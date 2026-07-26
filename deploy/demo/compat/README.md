# CSS ↔ sparq-lws-core Solid-OIDC compatibility harness

A **local development harness**, not a deployment manifest. It exists to answer one
question with real bytes instead of a design argument:

> Will a Community Solid Server (CSS) — proposed in
> [`research/lws-demo-architecture.md`](../../../research/lws-demo-architecture.md) §1 as the
> throwaway IdP for the public LWS demo — actually mint access tokens that `sparq-lws-core`'s
> `solid-oidc-verifier` will accept?

§1.2 of that record calls the answer **designed-only** and says in as many words: *do not
claim the pairing works before that smoke is green*. This directory is that smoke, and it is
the go/no-go for the rest of the demo work (bead C in the record's §7 table — if it fails,
the fallback is a seeded read-only demo, having spent only beads B and C).

```bash
bash deploy/demo/compat/smoke.sh
```

## ⚠️ Why this directory is allowed to set `SOLID_SERVER_ALLOW_LOOPBACK`

`docker-compose.yml` sets `SOLID_SERVER_ALLOW_LOOPBACK=1` on both LWS services — the dev/IT-only
escape hatch that lets the SSRF gate accept an `http:`/loopback IdP and WebID host. §4.3 of the
design record **bans** that variable (along with `SOLID_SERVER_SEED_CONFORMANCE`,
`SOLID_SERVER_SEED_BENCH` and `SOLID_SERVER_ALLOW_SEED_NONMEMORY`) from the public demo
manifests. There is no contradiction: those manifests live in the `deploy/demo/` **root**, a
separate subtree with its own README and its own `check.sh`. Nothing under `compat/` is ever
deployed, published, or referenced by a manifest. It runs on loopback, with no TLS and no
public DNS, and that is the entire point.

If you are adding the `deploy/demo/` root manifests, make sure their forbidden-env check scopes
itself to that directory and does **not** recurse into `compat/`.

## Topology

Three processes in a single network namespace, so `localhost:<port>` means the same thing to
the host, to CSS and to both LWS instances:

| Service | Port | Role |
| --- | --- | --- |
| `css` | 3000 | The throwaway Solid-OIDC IdP: issuer, JWKS, account API, pod + WebID host |
| `lws-strict` | 3001 | `sparq-lws-core` with `SOLID_SERVER_AUDIENCE=solid` — must **accept** |
| `lws-defaultaud` | 3002 | The same image with `SOLID_SERVER_AUDIENCE` unset — must **reject** |

The shared namespace matters more than it looks. The verifier compares a token's `iss` to
`SOLID_SERVER_TRUSTED_ISSUER` by exact string equality, CSS binds each DPoP proof to a literal
`htu`, and the bidirectional check dereferences the `webid` claim exactly as written. Any
host or port rewriting between the three parties would turn a genuine incompatibility into an
artefact of the harness — or, worse, hide one.

## What the smoke asserts

Every assertion runs against a **real** access token that CSS minted through its real
client-credentials + DPoP flow. Nothing is stubbed or replayed from a fixture, so the token
carries whatever `typ`, `aud`, `webid`, `cnf.jkt` and signing algorithm CSS emits *today*, and
it is checked by whatever the pinned `solid-oidc-verifier` rev enforces *today*.

| | Assertion | Goes red when |
| --- | --- | --- |
| 0 | CSS's discovery `issuer` is byte-identical to the configured trusted issuer | CSS normalises its base URL differently (e.g. a trailing-slash change) |
| A | Anonymous `PUT` under `/playground/` → 401 | LWS stops rejecting anonymous writes — the demo's only write friction |
| B | Authenticated `PUT` with the CSS token → 201 | `typ`, `aud`, `webid`, `cnf.jkt`, the signing alg, or the strict bidirectional check drifts |
| C | The written resource reads back with its content | B is passing on a write path that does not actually store |
| D | Replaying one DPoP proof (same `jti`) → 401 | Single-use replay protection regresses (a replay would become an ordinary update) |
| E | The CSS WebID document carries `solid:oidcIssuer` → the issuer | CSS's profile template stops closing the bidirectional loop |
| F | The **same** token → 401 against `lws-defaultaud` | `SOLID_SERVER_AUDIENCE=solid` stops being load-bearing |

Assertion **F** is the one that is easy to omit. A smoke that only checked the happy path
would stay green even if the audience check were deleted from the verifier outright, which
would quietly make the demo manifest's `SOLID_SERVER_AUDIENCE` line decorative. So the harness
runs a second LWS that differs in exactly that one environment variable and requires the very
same token to be refused. `smoke.sh` also refuses to run if `docker-compose.yml` ever sets
that variable on more than one service, because the one-variable delta *is* the experiment.

## The JOSE self-test

DPoP proofs are generated in pure bash + `openssl` so the harness has no npm/pip dependency
and no lockfile to rot. That code is load-bearing enough to check on its own:

```bash
bash deploy/demo/compat/smoke.sh --self-test   # no Docker required
```

It pins base64url encoding, the RFC 7638 JWK thumbprint (against RFC 9449 §6.1's published
example), ES256 DER→raw R‖S conversion in both the zero-strip and zero-pad directions, the fact
that the signed bytes are exactly `header.payload`, and the DPoP claim set. It runs
automatically before Docker is touched, so a broken helper is reported in about a second
rather than masquerading as a compatibility failure ten minutes into a build.

Its limit, stated plainly: it verifies signatures with the same `openssl` that produced them,
so it proves internal consistency and wire-format correctness — not that a third party accepts
the result. That is what CSS and assertion B are for.

## Notes

- **The first run is slow.** `lws-strict` builds `sparq-lws-core` from source inside a
  container. Later runs reuse the Docker layer cache.
- **CSS runs in memory.** The published CSS image defaults to `config/file.json` with
  `CSS_ROOT_FILE_PATH=/data`, which is *file* storage. The compose file overrides
  `CSS_CONFIG=config/default.json` — the config that imports `storage/backend/memory.json` —
  so accounts, signing keys and pods all die with the container and every run starts clean.
- **`--keep`** leaves the stack up for poking at; tear it down with
  `docker compose -f deploy/demo/compat/docker-compose.yml down -v`.
- **Requires** Docker with Compose v2, plus `curl`, `openssl`, `python3`, `sed` and `od` on
  the host. All are preflight-checked.
- A green run de-risks the CSS pairing for the demo design. It is a local-harness result about
  a *throwaway* identity setup, and says nothing about the production-readiness of any
  identity flow — see §5 of the design record for the demo's full caveat list.
