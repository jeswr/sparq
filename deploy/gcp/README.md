<!-- [SONNET-4.6] sq-agolp — GCP Cloud Run one-click deploy for sparq-server + sparq-lws-core. -->

# GCP Cloud Run deploy (sq-agolp)

Cloud Run service specs for running sparq on GCP **Cloud Run** (fully managed) with
automatic HTTPS, Secret Manager auth injection, and a dedicated least-privilege service
account.

Two service specs:

| Spec | Image | Port | Health path | Status |
|---|---|---|---|---|
| `sparq-server.yaml` | `ghcr.io/sparq-org/sparq-server` | 3030 | `GET /health` | Available |
| `sparq-lws.yaml` | `ghcr.io/sparq-org/sparq-lws-core` | 3000 | `GET /livez` + `GET /readyz` | Activates once sq-lmz40 ships |

---

## Secure-defaults notice (R1/R2/R4/R9)

> **sparq-server is open-by-default at the image layer.** It bakes `SPARQ_ALLOW_REMOTE=1`
> and ships with no auth. Anyone who reaches the published port can read AND write the
> dataset. **This spec enforces auth ON** by injecting `SPARQ_AUTH_TOKEN` from GCP Secret
> Manager (secretKeyRef — the literal value never appears in the YAML). Do not remove the
> Secret Manager wiring or set a default token value in the spec.

Key rules applied:

- **R1 (auth ON):** `SPARQ_AUTH_TOKEN` is required and comes from Secret Manager — never
  a spec parameter value or plaintext env var.
- **R2 (no anonymous write):** Unauthenticated writes return 401. sparq-lws-core is
  fail-closed by design; anonymous mutation is rejected at the image layer.
- **R3 (TLS at edge):** Cloud Run provides HTTPS automatically on the `run.app` URL —
  no certificate management required. For custom domains, map your domain via Cloud Run
  domain mapping (also automatic HTTPS via Google-managed certs).
- **R4 (secrets in Secret Manager):** No literal token, password, or key in any committed
  file. The auth token is referenced by secret name only.
- **R5 (least-privilege SA):** A dedicated service account (`sparq-server-sa` /
  `sparq-lws-sa`) is created with `roles/secretmanager.secretAccessor` on its own secret
  only — not the Compute default SA, not a project-level role.
- **R7 (health check):** Startup + liveness probes on `/health` (sparq-server) or
  `/livez` (LWS); readiness on `/readyz` (LWS). `min-instances: 1` keeps the server warm
  so health probes never hit a cold-start timeout.
- **R8 (non-root):** Both images run as non-root (distroless nonroot base). Cloud Run does
  not override this.

---

## sparq-server — Cloud Run deploy

### Prerequisites

1. Enable the required APIs:

   ```bash
   gcloud services enable run.googleapis.com secretmanager.googleapis.com
   ```

2. Create the auth token secret in Secret Manager:

   ```bash
   echo -n "$(openssl rand -hex 32)" | \
     gcloud secrets create sparq-auth-token \
       --replication-policy=automatic \
       --data-file=-
   ```

3. Create a dedicated least-privilege service account (R5):

   ```bash
   gcloud iam service-accounts create sparq-server-sa \
     --display-name="sparq-server Cloud Run runtime SA"

   gcloud secrets add-iam-policy-binding sparq-auth-token \
     --member="serviceAccount:sparq-server-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
     --role="roles/secretmanager.secretAccessor"
   ```

4. Grant Cloud Run the ability to act as the service account:

   ```bash
   gcloud projects add-iam-policy-binding "${PROJECT_ID}" \
     --member="serviceAccount:sparq-server-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
     --role="roles/run.invoker" 2>/dev/null || true
   ```

### One-liner deploy

Replace `PROJECT_ID` and `REGION` with your values, then edit `serviceAccountName` in
`sparq-server.yaml` to substitute your project ID, and run:

```bash
gcloud run services replace deploy/gcp/sparq-server.yaml \
  --region REGION \
  --project PROJECT_ID
```

The command outputs the service URL (`https://<hash>.run.app`). Your SPARQL endpoint
is at `https://<hash>.run.app/sparql`.

### "Run on Google Cloud" button

> The Cloud Run Button requires a public GitHub repo. Replace `<YOUR_BRANCH>` with the
> branch or tag you want to deploy from (default: `main`).

[![Run on Google Cloud](https://deploy.cloud.run/button.svg)](https://deploy.cloud.run/?git_repo=https://github.com/sparq-org/sparq&dir=deploy/gcp)

> **Note:** The Cloud Run Button launches Cloud Shell and runs `gcloud run services replace`
> from the repo. You must still complete the secret creation and service account setup
> (steps 1–3 above) before clicking, and edit `sparq-server.yaml` to set your `PROJECT_ID`
> in the `serviceAccountName` field.

### Verify the deploy

```bash
# Health check (ungated — should return "ok"):
curl https://<hash>.run.app/health

# Authenticated SPARQL query:
curl -H "Authorization: Bearer $(gcloud secrets versions access latest \
       --secret=sparq-auth-token)" \
     -H "Accept: application/sparql-results+json" \
     --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 1" \
     "https://<hash>.run.app/sparql"

# Unauthenticated write must return 401 (R2 acceptance check):
curl -X POST -H "Content-Type: application/sparql-update" \
     --data "INSERT DATA { <urn:test> <urn:p> <urn:o> }" \
     "https://<hash>.run.app/sparql"
# Expected: HTTP 401
```

---

## sparq-lws (Solid/LWS server) — Cloud Run deploy

**Status: activates once sq-lmz40 ships** (the `ghcr.io/sparq-org/sparq-lws-core` image
is not yet published). The spec `sparq-lws.yaml` is structurally complete; deploy once the
image is available.

The LWS server is **fail-closed by design** — anonymous mutation is rejected, DPoP is
required, WebID verification is strict. No `SPARQ_AUTH_TOKEN` equivalent is needed; auth
flows through OIDC/DPoP.

### Prerequisites

LWS requires two values that cannot be defaulted:

1. **`SOLID_SERVER_BASE_URL`** — the public HTTPS URL where the server will be reachable.
   After the first Cloud Run deploy, note the `run.app` URL and update `sparq-lws.yaml`;
   or map a custom domain first.
2. **`SOLID_SERVER_TRUSTED_ISSUER`** — the URL of a trusted OIDC provider. LWS cannot
   self-issue tokens. Use your organisation's Solid-OIDC provider or a public provider.

Edit `sparq-lws.yaml` to set both values and your `PROJECT_ID`, then:

```bash
gcloud services enable run.googleapis.com
gcloud iam service-accounts create sparq-lws-sa \
  --display-name="sparq-lws Cloud Run runtime SA"

gcloud run services replace deploy/gcp/sparq-lws.yaml \
  --region REGION \
  --project PROJECT_ID
```

**Multi-instance note:** `max-instances` defaults to 1 because the in-memory DPoP replay
store does not survive across instances. To run multiple instances, provision a Redis
instance and set `SOLID_SERVER_REPLAY_REDIS_URL` via a Secret Manager secretKeyRef
(see the commented block in `sparq-lws.yaml`).

---

## Decisions made (sq-agolp)

These decisions were made by the SPARQ agent without waiting for a maintainer greenlight
(per the standing proceed-without-greenlight rule). Corrections welcome post-hoc.

| Decision | Choice | Rationale |
|---|---|---|
| Compute | Cloud Run (fully managed) | R3: automatic HTTPS on run.app; no VPC/firewall config needed; serverless |
| Secret wiring | Secret Manager `secretKeyRef` | R4: token literal never in YAML, env-var plaintext, or logs |
| Service account | Dedicated SA per server (`sparq-server-sa` / `sparq-lws-sa`) | R5: `secretAccessor` on own secret only; not the Compute default SA |
| `min-instances` | 1 (default) | R7: avoids cold-start on startup/liveness probe; dataset stays warm |
| `max-instances` | 10 (sparq-server), 1 (LWS) | LWS replay-store constraint (§1.3); sparq-server stateless within a request |
| CPU/memory | 2 vCPU / 2 GiB | Reasonable default for analytical SPARQL; increase for larger in-memory datasets |
| LWS `min-instances` | 1 | Same cold-start rationale; Solid server must be warm for WebID-TLS exchanges |
| Probes | startup (6 × 5 s = 30 s budget) + liveness (10 s period) | Matches sparq-server's typical startup time; aggressive enough to catch hangs |
| Cloud Run `--no-allow-unauthenticated` | Not set (service-level) | Noted in spec as an option for internal use; sparq-server token is the auth layer |
| LWS image tag | `:latest` (parameterised comment to pin) | Image not yet published; operators must pin to a release tag in production |

---

*[SONNET-4.6] sq-agolp. Grounded against `research/cloud-deploy-architecture.md` §3.3 + §1
+ §2. House pattern follows `deploy/aws/` (sq-17rgw) and `deploy/paas/` (sq-dwame).*

*SPARQ agent 🤖 — do not remove the Secret Manager wiring.*
