<!-- [SONNET-4.6] sq-dwame — PaaS one-click deploy configs for sparq-server + sparq-lws-core. -->
<!-- [GPT-5.6] Post-hoc correctness/security review and hardening after PR #2314. -->
<!-- [GPT-5.6] Restored fail-closed deploy buttons alongside the controlled CLI flows. -->

# PaaS deployment

Deployment configs for the SPARQL server (`sparq-server`) and Solid/LWS server
(`sparq-lws-core`) on Fly.io, Render, and Railway.

Each provider includes a deploy button alongside an explicit controlled/secure setup path. The
buttons retain the same fail-closed auth, public-bind, health-check, and single-instance wiring as
the checked-in configs; use the explicit path when you want to install secrets before first deploy.

## Secure defaults

> **sparq-server is open by default at the image layer.** It bakes
> `SPARQ_ALLOW_REMOTE=1` and has no token unless the deployment supplies one. These templates
> inject `SPARQ_AUTH_TOKEN` from the provider's secret store and set
> `SPARQ_AUTH_TOKEN_READ=1`, so both reads and writes require the token. Do not remove that
> wiring. They also override the baked `SPARQ_ALLOW_REMOTE=1` to `0`, so a missing token makes
> the public bind fail closed instead of starting an unauthenticated server.

- **Secrets:** no token, password, key, or Redis URL is committed. Use Fly secrets, Render's
  generated environment-group secret, or Railway variables.
- **HTTPS:** Fly sets `force_https = true`; Render and Railway terminate TLS and redirect HTTP
  at their managed edges. Do not expose the internal container ports directly.
- **Health:** `sparq-server` uses port 3030 and `/health`. LWS uses port 3000,
  `/livez` for liveness, and `/readyz` for readiness.
- **One instance only:** `sparq-server` owns an uncoordinated in-memory dataset, so writable
  replicas silently diverge. LWS has an in-memory DPoP replay store unless a shared Redis replay
  backend is wired. Every flow below pins one instance.

The configs consume the canonical images `ghcr.io/sparq-org/sparq-server:latest` and
`ghcr.io/sparq-org/sparq-lws-core:latest`. A deployment cannot succeed until its selected tag is
published. The LWS image remains dependent on `sq-lmz40`.

## sparq-server

### Fly.io

[![Deploy on Fly.io](https://img.shields.io/badge/Deploy%20on-Fly.io-7B5EA7?logo=fly.io)](https://fly.io/launch?source=github&template=https://github.com/sparq-org/sparq/tree/main/deploy/paas/sparq-server)

Fly's launch flow does not prompt for secrets. After launch, run
`fly secrets set SPARQ_AUTH_TOKEN=...` and `fly scale count 1`; until the token is set, the service
fails closed. The controlled/secure CLI path below installs the secret before first deploy.

Fly creates two Machines by default for a new HTTP service. Both `--ha=false` and the explicit
scale command are load-bearing single-writer safeguards.

```bash
cd deploy/paas/sparq-server
fly launch --copy-config --generate-name --no-deploy --ha=false
fly secrets set SPARQ_AUTH_TOKEN="$(openssl rand -hex 32)"
fly deploy --ha=false
fly scale count 1
fly scale show
```

The app is not deployed until after the token is in Fly's encrypted secret vault. The config
enforces HTTPS and probes `GET /health`.

Config: [`sparq-server/fly.toml`](sparq-server/fly.toml)

### Render

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/sparq-org/sparq&branch=main&blueprint=deploy/paas/sparq-server/render.yaml)

The button targets this Blueprint, which generates `SPARQ_AUTH_TOKEN` in Render's secret store and
works without a pre-created token. For the controlled/secure path:

1. In the Render dashboard, choose **New → Blueprint** and connect this repository.
2. Select branch `main` and Blueprint Path
   `deploy/paas/sparq-server/render.yaml`.
3. Review and deploy the Blueprint.

The Blueprint generates `SPARQ_AUTH_TOKEN` into the `sparq-server-secrets` environment group,
injects it into the service, enables read auth, and pins `numInstances: 1`. Copy the generated
token from the environment group for authenticated clients.

Config: [`sparq-server/render.yaml`](sparq-server/render.yaml)

### Railway

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/new/template?template=https://github.com/sparq-org/sparq/tree/main/deploy/paas/sparq-server&envs=SPARQ_AUTH_TOKEN%2CSPARQ_AUTH_TOKEN_READ%2CPORT&SPARQ_AUTH_TOKENDesc=Required+bearer+token+for+reads+and+writes&SPARQ_AUTH_TOKEN_READDesc=Require+the+token+for+reads+and+writes&SPARQ_AUTH_TOKEN_READDefault=1&PORTDesc=Fixed+sparq-server+listen+and+health-check+port&PORTDefault=3030)

The button prompts for `SPARQ_AUTH_TOKEN` and includes the required read-auth and port variables.
For the controlled/secure CLI path, Railway config-as-code controls build/deploy settings but
cannot select an image or create variables. The adjacent `Dockerfile` consumes and fail-closes the
canonical image; the commands below create an empty service, install variables before its first
deployment, and upload only this config directory.

```bash
railway init
railway add --service sparq-server
openssl rand -hex 32 | railway variables set SPARQ_AUTH_TOKEN --stdin --skip-deploys
railway variables set SPARQ_AUTH_TOKEN_READ=1 PORT=3030 --skip-deploys
railway up deploy/paas/sparq-server --path-as-root
railway domain --port 3030
```

`PORT=3030` is required because Railway uses `PORT` for deployment health checks while the
image binds a fixed port. The TOML pins `numReplicas = 1`, probes `/health`, and leaves the
image entrypoint intact. Railway provisions TLS and redirects HTTP when the domain is created.

Configs: [`sparq-server/railway.toml`](sparq-server/railway.toml) and
[`sparq-server/Dockerfile`](sparq-server/Dockerfile)

## sparq-lws-core

The LWS server requires `SOLID_SERVER_BASE_URL` (the public HTTPS origin) and
`SOLID_SERVER_TRUSTED_ISSUER` (an external OIDC issuer). It rejects anonymous mutation by
default. Do not set its loopback or seed escape hatches in production.

LWS uses an in-memory DPoP `jti` replay store by default. Keep one instance unless the image was
built with `redis-replay` and every instance uses the same secret
`SOLID_SERVER_REPLAY_REDIS_URL`.

### Fly.io

[![Deploy on Fly.io](https://img.shields.io/badge/Deploy%20on-Fly.io-7B5EA7?logo=fly.io)](https://fly.io/launch?source=github&template=https://github.com/sparq-org/sparq/tree/main/deploy/paas/lws)

Fly's launch flow does not prompt for the required LWS settings. After launch, set
`SOLID_SERVER_BASE_URL` and `SOLID_SERVER_TRUSTED_ISSUER` with `fly secrets set`, then run
`fly scale count 1`; until both settings exist, LWS cannot boot. The controlled/secure CLI path
below installs them before first deploy.

Choose a globally unique app name first so the configured base URL exactly matches Fly's public
HTTPS hostname.

```bash
cd deploy/paas/lws
APP="sparq-lws-$(openssl rand -hex 4)"
fly launch --copy-config --no-deploy --ha=false --name "$APP"
fly secrets set SOLID_SERVER_BASE_URL="https://${APP}.fly.dev"
fly secrets set SOLID_SERVER_TRUSTED_ISSUER="https://your-oidc-provider.example.com"
fly deploy --ha=false
fly scale count 1
fly scale show
```

The config binds `0.0.0.0:3000`, enforces HTTPS, and checks both `/livez` and `/readyz`.

Config: [`lws/fly.toml`](lws/fly.toml)

### Render

[![Deploy to Render](https://render.com/images/deploy-to-render-button.svg)](https://render.com/deploy?repo=https://github.com/sparq-org/sparq&branch=main&blueprint=deploy/paas/lws/render.yaml)

The button targets this Blueprint and prompts for the required base URL and trusted issuer. For the
controlled/secure path:

1. In the Render dashboard, choose **New → Blueprint** and connect this repository.
2. Select branch `main` and Blueprint Path `deploy/paas/lws/render.yaml`.
3. Supply the prompted base URL and trusted issuer, then deploy.

The Blueprint binds port 3000, probes `/readyz`, and pins `numInstances: 1`.

Config: [`lws/render.yaml`](lws/render.yaml)

### Railway

[![Deploy on Railway](https://railway.com/button.svg)](https://railway.com/new/template?template=https://github.com/sparq-org/sparq/tree/main/deploy/paas/lws&envs=SOLID_SERVER_BASE_URL%2CSOLID_SERVER_TRUSTED_ISSUER%2CSOLID_SERVER_BIND%2CPORT&SOLID_SERVER_BASE_URLDesc=Required+public+HTTPS+origin&SOLID_SERVER_TRUSTED_ISSUERDesc=Required+external+OIDC+issuer+URL&SOLID_SERVER_BINDDesc=Required+public+container+bind&SOLID_SERVER_BINDDefault=0.0.0.0%3A3000&PORTDesc=Required+Railway+health-check+port&PORTDefault=3000)

The button prompts for the required public HTTPS origin and trusted issuer and includes the bind and
port variables. For the controlled/secure CLI path, create the domain before the first deployment
so its HTTPS origin can be used as the LWS base URL. Replace `<generated>.up.railway.app` below with
the hostname printed by `railway domain`.

```bash
railway init
railway add --service sparq-lws
railway domain --port 3000
railway variables set PORT=3000 SOLID_SERVER_BIND=0.0.0.0:3000 --skip-deploys
railway variables set SOLID_SERVER_BASE_URL="https://<generated>.up.railway.app" --skip-deploys
railway variables set SOLID_SERVER_TRUSTED_ISSUER="https://your-oidc-provider.example.com" --skip-deploys
railway up deploy/paas/lws --path-as-root
```

The TOML pins `numReplicas = 1` and probes `/readyz`. Railway terminates TLS and redirects HTTP
for the generated domain.

Configs: [`lws/railway.toml`](lws/railway.toml) and [`lws/Dockerfile`](lws/Dockerfile)

## File layout

```text
deploy/paas/
  sparq-server/
    fly.toml
    render.yaml
    railway.toml
    Dockerfile
  lws/
    fly.toml
    render.yaml
    railway.toml
    Dockerfile
  README.md
```

## License

MIT — see [LICENSE](../../LICENSE).
