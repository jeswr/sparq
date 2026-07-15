# sparq Helm chart (`deploy/helm/sparq/`)

Helm chart for **sparq-server** (W3C SPARQL 1.1 Protocol HTTP server) and
**sparq-lws-core** (native Solid/LWS server, activates once sq-lmz40 ships).
Part of the cloud-deploy epic sq-3vjdr; see
`research/cloud-deploy-architecture.md` for the full design record.

## Secure-defaults posture (R9 — R1/R2/R4/R8)

> sparq-server is **open-by-default** at the image layer: it ships
> `SPARQ_ALLOW_REMOTE=1` and starts with NO auth unless `SPARQ_AUTH_TOKEN` is set.
> Anyone who can reach the published port can read AND write the dataset.
> **This chart enforces auth at the template layer** — do NOT remove the
> `auth.existingSecret` wiring.

The chart implements every applicable secure-default rule from the design record:

| Rule | What the chart does |
|------|---------------------|
| R1 — auth ON by default | `auth.existingSecret` is required for `server=sparq-server`; chart errors at render time if unset |
| R2 — no anonymous write | Token gates writes (set `auth.requireTokenForReads: true` to also gate reads); LWS is fail-closed by default |
| R3 — TLS at the edge | Ingress + TLS block wired via cert-manager annotations; chart notes warn when TLS is off |
| R4 — secrets in the store, never inlined | Token comes from an **existing** k8s Secret via `secretKeyRef`; no literal in `values.yaml` |
| R5 — least-priv identity | `ServiceAccount` created with `automountServiceAccountToken: false`; no cluster-admin |
| R7 — per-server health path | `/health` (sparq-server) or `/livez`+`/readyz` (lws); parameterised in `_helpers.tpl`, never a shared constant |
| R8 — non-root + read-only rootfs | `runAsNonRoot: true`, `readOnlyRootFilesystem: true`, `capabilities.drop: [ALL]` |

## Quick install (sparq-server)

```sh
# 1. Create the auth-token Secret (never commit this token to git)
kubectl create namespace sparq
kubectl create secret generic sparq-auth-token \
  --namespace sparq \
  --from-literal=SPARQ_AUTH_TOKEN=$(openssl rand -hex 32)

# 2. Install the chart
helm install sparq ./deploy/helm/sparq \
  --namespace sparq \
  --set auth.existingSecret=sparq-auth-token

# 3. Port-forward and query
kubectl port-forward -n sparq svc/sparq-sparq 3030:3030
curl -H "Authorization: Bearer <your-token>" \
     -H "Content-Type: application/sparql-query" \
     --data "SELECT * WHERE { ?s ?p ?o } LIMIT 10" \
     http://localhost:3030/sparql
```

## Quick install (lws — activates once sq-lmz40 ships)

```sh
# lws is fail-closed: anonymous mutation rejected, DPoP required, HTTPS-only WebIDs.
# An external OIDC IdP and a public base URL are required at boot.
helm install sparq-lws ./deploy/helm/sparq \
  --namespace sparq \
  --set server=lws \
  --set image.repository=ghcr.io/sparq-org/sparq-lws-core \
  --set lws.trustedIssuer=https://solidcommunity.net \
  --set lws.baseUrl=https://lws.example.com \
  --set ingress.enabled=true \
  --set ingress.host=lws.example.com \
  --set 'ingress.annotations.cert-manager\.io/cluster-issuer=letsencrypt-prod' \
  --set ingress.tls.enabled=true \
  --set ingress.tls.secretName=lws-tls
```

## Plain-manifest quickstart (no Helm)

See `deploy/helm/quickstart/sparq-server-quickstart.yaml` for a self-contained
`kubectl apply -f` manifest. Edit the `Secret` data field with your base64-encoded
token before applying.

## Key values

| Value | Default | Description |
|-------|---------|-------------|
| `server` | `sparq-server` | `sparq-server` or `lws` |
| `image.repository` | `ghcr.io/sparq-org/sparq-server` | Container image |
| `image.tag` | `""` (chart appVersion) | Image tag |
| `replicaCount` | `1` | Pod replicas; lws multi-replica requires `lws.replayRedisExistingSecret` |
| `auth.existingSecret` | `""` | **Required** for `server=sparq-server` — name of an existing k8s Secret |
| `auth.tokenKey` | `SPARQ_AUTH_TOKEN` | Key within the Secret |
| `auth.requireTokenForReads` | `false` | Also gate reads with the token |
| `ingress.enabled` | `false` | Enable Ingress (set `ingress.host`) |
| `ingress.tls.enabled` | `false` | TLS on the Ingress |
| `persistence.enabled` | `false` | PVC for `/data` (sparq-server dataset) |
| `lws.trustedIssuer` | `""` | **Required** for `server=lws` — OIDC IdP URL |
| `lws.baseUrl` | `""` | **Required** for `server=lws` — public https base URL |

Full value reference: `deploy/helm/sparq/values.yaml`.

## LWS multi-replica note

The lws server uses an in-memory DPoP-jti replay store per replica. Running
more than one replica requires a shared Redis instance:

```sh
kubectl create secret generic lws-redis \
  --namespace sparq \
  --from-literal=SOLID_SERVER_REPLAY_REDIS_URL=redis://redis:6379

helm install sparq-lws ./deploy/helm/sparq \
  --set server=lws \
  --set replicaCount=3 \
  --set lws.replayRedisExistingSecret=lws-redis \
  ...
```

Without `lws.replayRedisExistingSecret`, keep `replicaCount: 1`.

## CI validation

`helm lint` + `helm template` are run in `.github/workflows/deploy-lint.yml`
on every change to `deploy/helm/**`. This validates YAML structure and catches
template errors without requiring a cluster. The workflow is non-gating to the
Rust engine gate (`ci-summary`) — a Helm lint failure does not block Rust merges.

To run locally:

```sh
helm lint ./deploy/helm/sparq --set auth.existingSecret=my-secret
helm template sparq ./deploy/helm/sparq --set auth.existingSecret=my-secret
```
