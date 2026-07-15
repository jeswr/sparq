<!-- [SONNET-4.6] sq-dwame — PaaS one-click deploy configs for sparq-server + sparq-lws-core. -->

# PaaS one-click deploy

One-click deploy configs for the sparq SPARQL server (`sparq-server`) and the Solid/LWS
server (`sparq-lws-core`) on three fast PaaS providers: **Fly.io**, **Render**, and
**Railway**.

## Secure-defaults notice (R1/R2/R4/R9)

> **sparq-server is open-by-default at the image layer** (it bakes `SPARQ_ALLOW_REMOTE=1`
> and ships with no auth). Anyone who can reach the published port can read AND write the
> dataset. **These templates enforce auth ON** at the template layer via `SPARQ_AUTH_TOKEN`
> — do not remove that wiring.

Key rules applied in every config here:

- **R1 (auth ON):** `SPARQ_AUTH_TOKEN` is a required secret for sparq-server. The token is
  stored only in the PaaS secret store — never in the config files.
- **R2 (no anonymous write):** Unauthenticated write requests return 401/403. The
  sparq-lws-core image is fail-closed by design; anonymous mutation is rejected by default.
- **R4 (secrets in the secret store):** No literal token, password, or key appears in any
  committed config file. Secrets are set via `fly secrets set`, Render's dashboard prompt
  (`sync: false`), or `railway variables set`.
- **Auto-HTTPS:** All three providers terminate TLS automatically on their managed domains.
  Plaintext HTTP is redirected to HTTPS at the edge (`force_https = true` on Fly; automatic
  on Render and Railway). **Do not expose either server on a plaintext public endpoint.**

## 🚀 sparq-server (SPARQL 1.1 Protocol server)

Image: `ghcr.io/sparq-org/sparq-server:latest`
Port: 3030 | Health: `GET /health`

### Fly.io

[![Deploy on Fly.io](https://img.shields.io/badge/Deploy%20on-Fly.io-7B5EA7?logo=fly.io)](https://fly.io/launch?source=github&template=https://github.com/sparq-org/sparq/tree/main/deploy/paas/sparq-server)

```bash
# 1. Install flyctl: https://fly.io/docs/getting-started/installing-flyctl/
cd deploy/paas/sparq-server
fly launch --copy-config --no-deploy
fly secrets set SPARQ_AUTH_TOKEN="$(openssl rand -hex 32)"
fly deploy
```

Config: [`deploy/paas/sparq-server/fly.toml`](sparq-server/fly.toml)

### Render

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/sparq-org/sparq&branch=main&blueprint=deploy/paas/sparq-server/render.yaml)

Render will prompt you to set `SPARQ_AUTH_TOKEN` during deployment — set a strong random
value (e.g. `openssl rand -hex 32`).

Config: [`deploy/paas/sparq-server/render.yaml`](sparq-server/render.yaml)

### Railway

[![Deploy on Railway](https://railway.app/button.svg)](https://railway.app/new/template?template=https://github.com/sparq-org/sparq&envs=SPARQ_AUTH_TOKEN,IMAGE&SPARQ_AUTH_TOKENDesc=Bearer+auth+token+for+sparq-server+(required)&IMAGEDesc=GHCR+image+ref&IMAGEDefault=ghcr.io%2Fsparq-org%2Fsparq-server%3Alatest)

```bash
# Via Railway CLI:
railway link   # link to your Railway project
railway variables set SPARQ_AUTH_TOKEN="$(openssl rand -hex 32)"
railway up
```

Config: [`deploy/paas/sparq-server/railway.toml`](sparq-server/railway.toml)

---

## 🚀 sparq-lws-core (Solid/LWS server)

Image: `ghcr.io/sparq-org/sparq-lws-core:latest`
Port: 3000 | Health: `GET /readyz` (readiness), `GET /livez` (liveness)

> **Dependency:** The `sparq-lws-core` image is built by bead `sq-lmz40` and will be
> published to `ghcr.io/sparq-org/sparq-lws-core`. The configs below are correct and
> ready; the image activates once `sq-lmz40` ships. Link the deploy buttons from the
> `/deploy` site surface once that bead lands.

> **Required inputs:** Unlike sparq-server, the LWS server requires two parameters at boot:
> `SOLID_SERVER_BASE_URL` (your public HTTPS URL) and `SOLID_SERVER_TRUSTED_ISSUER`
> (your OIDC identity provider). The server cannot self-issue an IdP — a Solid OIDC
> provider must be named. These are supplied as secrets, never inlined.

> **Multi-instance:** LWS uses an in-memory DPoP jti replay store by default. Running more
> than one replica requires `SOLID_SERVER_REPLAY_REDIS_URL` (the `redis-replay` feature).
> Single-instance is correct and safe without Redis.

### Fly.io

```bash
cd deploy/paas/lws
fly launch --copy-config --no-deploy
fly secrets set SOLID_SERVER_BASE_URL="https://sparq-lws.fly.dev"
fly secrets set SOLID_SERVER_TRUSTED_ISSUER="https://your-oidc-provider.example.com"
fly deploy
```

Config: [`deploy/paas/lws/fly.toml`](lws/fly.toml)

### Render

Render will prompt you to set `SOLID_SERVER_BASE_URL` and `SOLID_SERVER_TRUSTED_ISSUER`
during deployment.

Config: [`deploy/paas/lws/render.yaml`](lws/render.yaml)

### Railway

```bash
railway link
railway variables set SOLID_SERVER_BASE_URL="https://sparq-lws.railway.app"
railway variables set SOLID_SERVER_TRUSTED_ISSUER="https://your-oidc-provider.example.com"
railway up
```

Config: [`deploy/paas/lws/railway.toml`](lws/railway.toml)

---

## File layout

```
deploy/paas/
  sparq-server/
    fly.toml       Fly.io app config (port 3030, /health, auth required)
    render.yaml    Render Blueprint  (port 3030, /health, auth required)
    railway.toml   Railway config    (port 3030, /health, auth required)
  lws/
    fly.toml       Fly.io app config (port 3000, /readyz, fail-closed)
    render.yaml    Render Blueprint  (port 3000, /readyz, fail-closed)
    railway.toml   Railway config    (port 3000, /readyz, fail-closed)
  README.md        This file (deploy buttons + secure-defaults guidance)
```

## License

MIT — see [LICENSE](../../LICENSE).
