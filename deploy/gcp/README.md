<!-- [SONNET-4.6] sq-agolp — GCP Cloud Run one-click deploy for sparq-server + sparq-lws-core. -->

# GCP Cloud Run deploy (sq-agolp)

Cloud Run service specs for running sparq on GCP **Cloud Run** (fully managed) with
automatic HTTPS, Secret Manager auth injection, and a dedicated least-privilege service
account.

<!-- [GPT-5.6] Cloud Run's writable filesystem is ephemeral; these services are single-instance. -->
> **Durability limit:** both servers currently keep mutable state in the container instance.
> Cloud Run can restart that instance at any time, so this deployment is suitable for evaluation
> and reloadable datasets, not durable production storage. Both manifests cap scaling at one
> instance to prevent divergent copies of state.

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
  a spec parameter value or plaintext env var. `SPARQ_AUTH_TOKEN_READ=1` also gates reads.
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
- **R6 (intended ingress only):** Only the declared application port is exposed. Public
  invocation is explicit because application auth consumes the `Authorization` header.
- **R7 (health check):** Startup + liveness probes on `/health` (sparq-server) or
  `/readyz` + `/livez` (LWS); readiness on `/readyz` (LWS). Custom probes use
  instance-based CPU allocation, and `min-instances: 1` avoids cold starts. The LWS
  manifest explicitly opts into Cloud Run's BETA launch stage for its Preview readiness probe.
- **R8 (non-root):** Both images run as non-root. Cloud Run rejects Kubernetes
  `securityContext`/`readOnlyRootFilesystem`, but removes added capabilities and privilege
  escalation. Its writable root overlay is ephemeral.

<!-- [GPT-5.6] App Bearer auth and Cloud Run IAM cannot both occupy Authorization. -->
The manifests disable Cloud Run's Invoker IAM check and expose only the managed HTTPS endpoint.
This is intentional: requiring a Google identity token at the edge would consume the same
`Authorization` header used by sparq's Bearer token. If you make a service IAM-private instead,
put an authenticating proxy in front of it rather than removing application auth.

---

## sparq-server — Cloud Run deploy

### Prerequisites

1. Enable the required APIs:

   ```bash
   gcloud services enable run.googleapis.com secretmanager.googleapis.com
   ```

   Set deployment variables (the deployer principal may instead be a service account):

   ```bash
   PROJECT_ID="$(gcloud config get-value project)"
   PROJECT_NUMBER="$(gcloud projects describe "${PROJECT_ID}" --format='value(projectNumber)')"
   REGION="us-central1"
   DEPLOYER_MEMBER="user:$(gcloud config get-value account)"
   ```

2. Create the auth token secret in Secret Manager:

   ```bash
   # [GPT-5.6] Strip openssl's newline so the stored token matches the HTTP header exactly.
   openssl rand -hex 32 | tr -d '\n' | \
     gcloud secrets create sparq-auth-token \
       --replication-policy=automatic \
       --data-file=- \
       --project="${PROJECT_ID}"
   ```

3. Create a dedicated least-privilege service account (R5):

   ```bash
   gcloud iam service-accounts create sparq-server-sa \
     --display-name="sparq-server Cloud Run runtime SA" \
     --project="${PROJECT_ID}"

   gcloud secrets add-iam-policy-binding sparq-auth-token \
     --member="serviceAccount:sparq-server-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
     --role="roles/secretmanager.secretAccessor" \
     --project="${PROJECT_ID}"
   ```

4. Grant only the deployer permission to attach the runtime service account:

   ```bash
   gcloud iam service-accounts add-iam-policy-binding \
     "sparq-server-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
     --member="${DEPLOYER_MEMBER}" \
     --role="roles/iam.serviceAccountUser" \
     --project="${PROJECT_ID}"
   ```

   <!-- [GPT-5.6] This replaces the invalid runtime-SA roles/run.invoker grant. -->
   Do not grant `roles/run.invoker` to the runtime service account; that neither authorizes
   the deployer to attach it nor makes the service public.

### One-liner deploy

Render the project-scoped service-account and secret references into a temporary file, then
replace the Cloud Run service:

```bash
RENDERED="$(mktemp)"
trap 'rm -f "${RENDERED}"' EXIT
sed -e "s/PROJECT_ID/${PROJECT_ID}/g" \
    -e "s/PROJECT_NUMBER/${PROJECT_NUMBER}/g" \
    deploy/gcp/sparq-server.yaml >"${RENDERED}"
gcloud run services replace "${RENDERED}" \
  --region="${REGION}" \
  --project="${PROJECT_ID}"
```

The command outputs the service URL (`https://<hash>.run.app`). Your SPARQL endpoint
is at `https://<hash>.run.app/sparql`.

### Why there is no Cloud Run Button

<!-- [GPT-5.6] The removed button tried to source-build deploy/gcp and never applied this YAML. -->
The Cloud Run Button builds a container from the selected repository directory; it does not
apply these service manifests or safely provision their Secret Manager and IAM prerequisites.
Pointing it at `deploy/gcp/` would therefore fail or deploy the wrong artifact. Use the commands
above, which deploy the published image without rebuilding it and never put the token in plaintext
service configuration.

### Verify the deploy

```bash
SERVICE_URL="$(gcloud run services describe sparq-server \
  --region="${REGION}" --project="${PROJECT_ID}" --format='value(status.url)')"

# Health check (ungated — should return "ok"):
curl "${SERVICE_URL}/health"

# Authenticated SPARQL query:
curl -H "Authorization: Bearer $(gcloud secrets versions access latest \
       --secret=sparq-auth-token --project="${PROJECT_ID}")" \
     -H "Accept: application/sparql-results+json" \
     --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 1" \
     "${SERVICE_URL}/sparql"

# Unauthenticated read must return 401 (SPARQ_AUTH_TOKEN_READ=1):
curl -sS -o /dev/null -w '%{http_code}\n' \
  --data-urlencode "query=SELECT * WHERE { ?s ?p ?o } LIMIT 1" \
  "${SERVICE_URL}/sparql"

# Unauthenticated write must return 401 (R2 acceptance check):
curl -sS -o /dev/null -w '%{http_code}\n' \
     -X POST -H "Content-Type: application/sparql-update" \
     --data "INSERT DATA { <urn:test> <urn:p> <urn:o> }" \
     "${SERVICE_URL}/sparql"
# Expected for both unauthenticated requests: 401
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

LWS requires one operator-supplied value and one deterministic Cloud Run value:

1. **`SOLID_SERVER_BASE_URL`** — rendered as Cloud Run's deterministic HTTPS URL,
   `https://sparq-lws-PROJECT_NUMBER.REGION.run.app`; use a custom HTTPS domain if needed.
2. **`SOLID_SERVER_TRUSTED_ISSUER`** — the URL of a trusted OIDC provider. LWS cannot
   self-issue tokens. Use your organisation's Solid-OIDC provider or a public provider.

Set variables, create the dedicated identity, and authorize only the deployer to attach it:

```bash
PROJECT_ID="$(gcloud config get-value project)"
PROJECT_NUMBER="$(gcloud projects describe "${PROJECT_ID}" --format='value(projectNumber)')"
REGION="us-central1"
TRUSTED_ISSUER="https://issuer.example.com"
DEPLOYER_MEMBER="user:$(gcloud config get-value account)"

case "${TRUSTED_ISSUER}" in https://*) ;; *) echo "TRUSTED_ISSUER must use HTTPS" >&2; exit 1;; esac

gcloud services enable run.googleapis.com --project="${PROJECT_ID}"
gcloud iam service-accounts create sparq-lws-sa \
  --display-name="sparq-lws Cloud Run runtime SA" \
  --project="${PROJECT_ID}"

gcloud iam service-accounts add-iam-policy-binding \
  "sparq-lws-sa@${PROJECT_ID}.iam.gserviceaccount.com" \
  --member="${DEPLOYER_MEMBER}" \
  --role="roles/iam.serviceAccountUser" \
  --project="${PROJECT_ID}"

RENDERED="$(mktemp)"
trap 'rm -f "${RENDERED}"' EXIT
sed -e "s/PROJECT_ID/${PROJECT_ID}/g" \
    -e "s/PROJECT_NUMBER/${PROJECT_NUMBER}/g" \
    -e "s/REGION/${REGION}/g" \
    -e "s|TRUSTED_ISSUER_URL|${TRUSTED_ISSUER}|g" \
    deploy/gcp/sparq-lws.yaml >"${RENDERED}"
gcloud run services replace "${RENDERED}" \
  --region="${REGION}" \
  --project="${PROJECT_ID}"
```

**Multi-instance note:** `max-instances` defaults to 1 because the in-memory DPoP replay
store does not survive across instances. To run multiple instances, provision a Redis
instance and set `SOLID_SERVER_REPLAY_REDIS_URL` via a Secret Manager secretKeyRef
(see the two commented blocks in `sparq-lws.yaml`). Create the secret, grant only
`sparq-lws-sa` `roles/secretmanager.secretAccessor` on that secret, pin its version, and
uncomment both the `run.googleapis.com/secrets` alias and the environment reference.

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
| `max-instances` | 1 for both | Both hold instance-local state; LWS additionally needs shared Redis replay protection before scale-out |
| CPU/memory | 2 vCPU / 2 GiB | Reasonable default for analytical SPARQL; increase for larger in-memory datasets |
| LWS `min-instances` | 1 | Same cold-start rationale; Solid server must be warm for WebID-TLS exchanges |
| Probes | startup (6 × 5 s = 30 s budget) + liveness (10 s period) | Matches sparq-server's typical startup time; aggressive enough to catch hangs |
| Cloud Run Invoker IAM check | Disabled explicitly | Public managed HTTPS reaches each server's application auth without competing for the Bearer header |
| Billing | Instance-based CPU | Required by Cloud Run custom health probes; `min-instances: 1` incurs standing cost |
| Root filesystem | Cloud Run writable ephemeral overlay | Cloud Run does not support `readOnlyRootFilesystem`; images remain non-root and the platform removes elevated capabilities |
| LWS image tag | `:latest` (parameterised comment to pin) | Image not yet published; operators must pin to a release tag in production |

---

*[SONNET-4.6] sq-agolp. Grounded against `research/cloud-deploy-architecture.md` §3.3 + §1
+ §2. House pattern follows `deploy/aws/` (sq-17rgw) and `deploy/paas/` (sq-dwame).*

*SPARQ agent 🤖 — do not remove the Secret Manager wiring.*
