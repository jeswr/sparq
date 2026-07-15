<!-- [SONNET-4.6] sq-zcou4 — Azure Container Apps Deploy-to-Azure for sparq-server + sparq-lws-core. -->

# Azure one-click deploy (sq-zcou4)

Bicep templates (with compiled ARM JSON) for running sparq on **Azure Container Apps**
with managed HTTPS, Container Apps secrets, and a user-assigned managed identity.

Two templates:

| Template | Image | Port | Health path | Status |
|---|---|---|---|---|
| `sparq-server/` | `ghcr.io/sparq-org/sparq-server` | 3030 | `GET /health` | Available |
| `lws/` | `ghcr.io/sparq-org/sparq-lws-core` | 3000 | `GET /livez` + `GET /readyz` | Activates once sq-lmz40 ships |

---

## Secure-defaults notice (R1/R2/R4/R9)

> **sparq-server is open-by-default at the image layer.** It bakes `SPARQ_ALLOW_REMOTE=1`
> and ships with no auth. Anyone who reaches the published port can read AND write the
> dataset. **These templates enforce auth ON** by storing `SPARQ_AUTH_TOKEN` as a
> Container Apps secret and injecting it via `secretRef` — the literal value never appears
> in the template output or logs. Do not remove the `secretRef` wiring or set a default
> token value.

Secure defaults enforced per `research/cloud-deploy-architecture.md`:

- **R1 (auth ON):** `SPARQ_AUTH_TOKEN` is required and stored as a Container Apps secret
  — never a template parameter literal.
- **R2 (no anonymous write):** Unauthenticated writes return 401. sparq-lws-core is
  fail-closed by design; anonymous mutation is rejected at the image layer. No dev escape
  hatches (`SOLID_SERVER_ALLOW_LOOPBACK`, `SOLID_SERVER_SEED_*`) appear in any template.
- **R3 (TLS at edge):** Container Apps terminates HTTPS automatically on the
  `*.azurecontainerapps.io` FQDN. `allowInsecure: false` is set on both templates —
  plaintext HTTP connections are rejected.
- **R4 (secrets in Container Apps secrets / Key Vault):** No literal token, password, or
  key in any committed file. See the Key Vault reference pattern in the `azuredeploy.parameters.json`
  files.
- **R5 (least-privilege identity):** A user-assigned managed identity is created by each
  template. After deploy, grant it `Key Vault Secrets User` on only its secrets (see below).
- **R6 (ingress on app port only):** External ingress is enabled on the app's `targetPort`
  only (3030 for sparq-server, 3000 for lws). Management or debug ports are not public.
- **R7 (health probes):** Container Apps liveness + readiness + startup probes wired per
  server: `/health` (sparq-server) and `/livez`+`/readyz` (lws). Health paths differ
  between servers — not shared or parameterised-away.
- **R8 (non-root, read-only rootfs):** Both images run non-root; `readOnlyRootFilesystem`
  and `allowPrivilegeEscalation: false` are set on the container security context.

---

## sparq-server — Deploy to Azure

### Prerequisites

1. An Azure subscription with the `Microsoft.App` and `Microsoft.OperationalInsights`
   providers registered.
2. A resource group in the target region.
3. The auth token value you will inject (generate one: `openssl rand -hex 32`).

### Deploy-to-Azure button

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Fsparq-server%2Fazuredeploy.json)

The portal will prompt you for parameters including `authToken`. Supply a strong random
value — the portal sends it directly to the ARM deployment as a `securestring` and it is
stored immediately as a Container Apps secret; it is not logged or exposed.

### CLI deploy

```bash
# 1. Create a resource group (if needed)
az group create --name sparq-rg --location eastus

# 2. Deploy — supply the auth token directly (it is a securestring; not logged)
az deployment group create \
  --resource-group sparq-rg \
  --template-file deploy/azure/sparq-server/main.bicep \
  --parameters authToken="$(openssl rand -hex 32)"

# Or, using the compiled ARM JSON:
az deployment group create \
  --resource-group sparq-rg \
  --template-file deploy/azure/sparq-server/azuredeploy.json \
  --parameters authToken="$(openssl rand -hex 32)"
```

### Key Vault reference (recommended for production)

To avoid supplying the token on the CLI at all, create a Key Vault secret and reference it:

```bash
# Create a key vault and store the token
az keyvault create --name sparq-kv --resource-group sparq-rg --location eastus
az keyvault secret set --vault-name sparq-kv --name sparq-auth-token \
  --value "$(openssl rand -hex 32)"

# Deploy using the parameters file (edit azuredeploy.parameters.json with your vault ID)
az deployment group create \
  --resource-group sparq-rg \
  --template-file deploy/azure/sparq-server/main.bicep \
  --parameters @deploy/azure/sparq-server/azuredeploy.parameters.json

# Grant the managed identity Key Vault Secrets User on the secret
PRINCIPAL=$(az deployment group show \
  --resource-group sparq-rg --name main \
  --query properties.outputs.managedIdentityPrincipalId.value -o tsv)
az role assignment create \
  --role "Key Vault Secrets User" \
  --assignee-object-id "$PRINCIPAL" \
  --assignee-principal-type ServicePrincipal \
  --scope "$(az keyvault secret show --vault-name sparq-kv --name sparq-auth-token --query id -o tsv)"
```

### Post-deploy

The deployment outputs `fqdn` and `sparqlEndpoint`. Your SPARQL endpoint is:

```
https://<fqdn>/sparql
```

Test with the auth token:

```bash
curl -H "Authorization: Bearer <your-token>" \
  "https://<fqdn>/sparql?query=SELECT+*+WHERE+{+?s+?p+?o+}+LIMIT+1"
```

---

## sparq-lws-core — Deploy to Azure

**Status: activates once sq-lmz40 ships.** The image `ghcr.io/sparq-org/sparq-lws-core`
is not yet published. The template is structurally complete and valid but the image will not
resolve until sq-lmz40 merges.

> **Note on replicas:** sparq-lws-core uses an in-memory DPoP-jti replay store. Multiple
> replicas without a shared Redis backend will break replay protection. The template
> defaults `maxReplicas: 1`. To scale beyond one replica, supply
> `SOLID_SERVER_REPLAY_REDIS_URL` (via the `redisUrl` parameter, stored as a Container Apps
> secret) and raise `maxReplicas`.

### Deploy-to-Azure button

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Flws%2Fazuredeploy.json)

### CLI deploy

```bash
# LWS requires a public base URL and a trusted OIDC issuer.
# The base URL is often known only after first deploy — deploy once, note the FQDN,
# then redeploy with solidBaseUrl set to the Container Apps FQDN.

az group create --name sparq-lws-rg --location eastus

az deployment group create \
  --resource-group sparq-lws-rg \
  --template-file deploy/azure/lws/main.bicep \
  --parameters \
    solidBaseUrl="https://YOUR-APP.azurecontainerapps.io" \
    solidTrustedIssuer="https://YOUR-OIDC-ISSUER"
```

---

## Rebuilding the ARM JSON from Bicep

The `azuredeploy.json` files are compiled outputs of the Bicep sources. To rebuild:

```bash
az bicep build --file deploy/azure/sparq-server/main.bicep \
  --outfile deploy/azure/sparq-server/azuredeploy.json
az bicep build --file deploy/azure/lws/main.bicep \
  --outfile deploy/azure/lws/azuredeploy.json
```

---

## Discovered work

The following items were noted during implementation and are candidates for beads:

- Custom domain + managed certificate wiring (Container Apps `customDomains` resource).
- Key Vault reference as a first-class parameter option (replacing the `secretRef` to a
  Container Apps-stored secret with a `keyVaultUrl`-type secret ref, requires the managed
  identity to hold Key Vault Secrets User before deployment).
- Persistent storage mount for sparq-server dataset (Azure Files SMB share via
  `storageType: AzureFile` in the Container Apps volume mount configuration).
- Redis Cache for lws multi-replica DPoP-jti replay (an `Microsoft.Cache/Redis` resource
  added to the lws template or a shared infrastructure template).
- Smoke-test CI lane for the ARM template (non-gating, `azure/webapps-deploy@` or
  `azure/arm-deploy@` action in a separate advisory workflow, gated on `sq-lmz40` for lws).
