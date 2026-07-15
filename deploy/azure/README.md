<!-- [SONNET-4.6] sq-zcou4 — Azure Container Apps Deploy-to-Azure for sparq-server + sparq-lws-core. -->
<!-- [GPT-5.6] Stronger-tier IaC security/correctness review fixes. -->

# Azure one-click deploy (sq-zcou4)

Bicep templates (with compiled ARM JSON) for running sparq on **Azure Container Apps**
with managed HTTPS. The sparq-server deployment creates a dedicated Key Vault and a
user-assigned identity with get-only access; LWS needs no Azure runtime identity or secret.

Two templates:

| Template | Image | Port | Health path | Status |
|---|---|---|---|---|
| `sparq-server/` | `ghcr.io/sparq-org/sparq-server` | 3030 | `GET /health` | Available |
| `lws/` | `ghcr.io/sparq-org/sparq-lws-core` | 3000 | `GET /livez` + `GET /readyz` | Activates once sq-lmz40 ships |

---

## Secure-defaults notice (R1/R2/R4/R9)

> **sparq-server is open-by-default at the image layer.** It bakes `SPARQ_ALLOW_REMOTE=1`
> and ships with no auth. Anyone who reaches the published port can read AND write the
> dataset. **The sparq-server template enforces auth ON** by storing `SPARQ_AUTH_TOKEN` in
> a dedicated Azure Key Vault, exposing it to Container Apps through a managed-identity
> Key Vault reference, and injecting it via `secretRef`. The literal value never appears in
> template outputs or logs. Do not remove the Key Vault or `secretRef` wiring.

Secure defaults enforced per `research/cloud-deploy-architecture.md`:

- **R1 (auth ON):** `SPARQ_AUTH_TOKEN` is a required `secureString` with a minimum length,
  written to the template's dedicated Key Vault, and always injected through `secretRef`.
- **R2 (no anonymous write):** Unauthenticated writes return 401. sparq-lws-core is
  fail-closed by design; anonymous mutation is rejected at the image layer. No dev escape
  hatches (`SOLID_SERVER_ALLOW_LOOPBACK`, `SOLID_SERVER_SEED_*`) appear in any template.
- **R3 (TLS at edge):** Container Apps terminates HTTPS automatically on the
  `*.azurecontainerapps.io` FQDN. `allowInsecure: false` is set on both templates, so Azure
  redirects plaintext HTTP to HTTPS before credentials reach the application.
- **R4 (Key Vault):** No literal token, password, or key exists in any committed file. The
  sparq-server runtime token lives in a dedicated Key Vault; the sample parameter file shows
  how an existing vault can securely supply the deployment input.
- **R5 (least privilege):** sparq-server gets a dedicated user-assigned identity whose access
  policy permits only `secrets/get` in its one-secret vault. LWS requires no cloud API access,
  so it creates no identity or role.
- **R6 (ingress on app port only):** External ingress is enabled on the app's `targetPort`
  only (3030 for sparq-server, 3000 for lws). Management or debug ports are not public.
- **R7 (health probes):** Container Apps liveness + readiness + startup probes wired per
  server: `healthPath=/health` (sparq-server) and `livenessPath=/livez` plus
  `readinessPath=/readyz` (LWS). These are separate, server-specific parameters.
- **R8 (non-root / rootfs):** Both images run non-root and the templates never override the
  image user. Container Apps API `2024-03-01` does not expose Kubernetes-style
  `securityContext` or `readOnlyRootFilesystem` on a container, so the templates do not emit
  unsupported properties that Azure would reject or ignore.

---

## sparq-server — Deploy to Azure

### Prerequisites

1. An Azure subscription with the `Microsoft.App`, `Microsoft.OperationalInsights`,
   `Microsoft.KeyVault`, and `Microsoft.ManagedIdentity` providers registered.
2. A resource group in the target region.
3. The auth token value you will inject (generate one: `openssl rand -hex 32`).

### Deploy-to-Azure button

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Fsparq-server%2Fazuredeploy.json)

The portal will prompt you for parameters including `authToken`. Supply a strong random
value — the portal sends it to ARM as a `securestring`, and the deployment writes it into
the dedicated Key Vault. It is not logged or exposed in deployment outputs.

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

### Supply the deployment parameter from an existing Key Vault

The template always creates a dedicated runtime Key Vault. To avoid passing the initial token
on the CLI, an existing source vault can provide the secure deployment parameter. ARM parameter
references require template-deployment access on that source vault:

```bash
# Create a key vault and store the token
az keyvault create --name sparq-kv --resource-group sparq-rg --location eastus \
  --enabled-for-template-deployment true
az keyvault secret set --vault-name sparq-kv --name sparq-auth-token \
  --value "$(openssl rand -hex 32)"

# Deploy using the parameters file (edit azuredeploy.parameters.json with your vault ID)
az deployment group create \
  --resource-group sparq-rg \
  --template-file deploy/azure/sparq-server/main.bicep \
  --parameters @deploy/azure/sparq-server/azuredeploy.parameters.json

# No post-deploy grant is needed: the template creates the runtime identity and its
# get-only access policy before Container Apps resolves the runtime secret reference.
```

### Post-deploy

The deployment outputs `fqdn`, `sparqlEndpoint`, and `keyVaultName`. Your SPARQL endpoint is:

```text
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

> **Note on replicas:** sparq-lws-core uses an in-memory DPoP-jti replay store. The canonical
> image contract does not promise the opt-in `redis-replay` build feature, so this template
> hard-pins `maxReplicas: 1`. A separately built Redis-enabled image and reviewed template
> are required before horizontal scaling.

### Deploy-to-Azure button

[![Deploy to Azure](https://aka.ms/deploytoazurebutton)](https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Flws%2Fazuredeploy.json)

### CLI deploy

```bash
# LWS requires a trusted HTTPS OIDC issuer. By default, the template derives its public HTTPS
# base URL from the Container Apps environment during the first deployment. After configuring
# a custom domain, pass its HTTPS origin with --parameters solidBaseUrl=https://solid.example

az group create --name sparq-lws-rg --location eastus

az deployment group create \
  --resource-group sparq-lws-rg \
  --template-file deploy/azure/lws/main.bicep \
  --parameters solidTrustedIssuer="https://YOUR-OIDC-ISSUER"
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
