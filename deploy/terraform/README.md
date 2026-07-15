# sparq multi-cloud Terraform module (sq-sos84)

Deploy either `sparq-server` (SPARQL 1.1 endpoint) or the Solid/LWS server
(`sparq-lws-core`) into AWS, Azure, or GCP using a single Terraform root module.

**Security note (R9):** `sparq-server` is open-by-default at the image layer
(bakes `SPARQ_ALLOW_REMOTE=1`, no auth). This module enforces auth ON at the
template layer via `auth_token` (R1). **Do not remove the token wiring.**

## Quick start

```sh
# AWS — ECS Fargate + ALB + Secrets Manager
terraform init
terraform apply \
  -var target=aws \
  -var auth_token="$(openssl rand -hex 32)" \
  -var aws_region=us-east-1

# Azure — Container Apps + Key Vault
terraform apply \
  -var target=azure \
  -var auth_token="$(openssl rand -hex 32)" \
  -var azure_location=eastus

# GCP — Cloud Run + Secret Manager
terraform apply \
  -var target=gcp \
  -var auth_token="$(openssl rand -hex 32)" \
  -var gcp_project=my-project \
  -var gcp_region=us-central1
```

Never commit `auth_token` to `.tfvars` or version control. The token is stored
in the target cloud's secret store and injected at runtime (R4).

## Structure

```
deploy/terraform/
  main.tf          Root module — delegates to ./modules/<target>
  variables.tf     All root variables
  outputs.tf       endpoint_url, service_name, secret_id
  modules/
    aws/           ECS Fargate + ALB + Secrets Manager
    azure/         Container Apps + Key Vault
    gcp/           Cloud Run + Secret Manager
```

## Servers

| `server` | Image | Port | Health |
|---|---|---|---|
| `sparq-server` (default) | `ghcr.io/sparq-org/sparq-server` | 3030 | `GET /health` |
| `lws` | `ghcr.io/sparq-org/sparq-lws-core` | 3000 | `GET /readyz` (readiness) + `GET /livez` (liveness) |

## Key variables

| Variable | Default | Description |
|---|---|---|
| `target` | (required) | `aws` / `azure` / `gcp` |
| `server` | `sparq-server` | `sparq-server` or `lws` |
| `auth_token` | (required) | Bearer token — stored in cloud secret store, never a literal |
| `image_tag` | `latest` | Image tag to deploy |
| `name` | `sparq` | Base name for all resources |
| `solid_server_base_url` | `""` | Required when `server=lws` |
| `solid_server_trusted_issuer` | `""` | Required when `server=lws` |

See `variables.tf` for all variables and per-submodule options (region, sizing,
ACM cert ARN for AWS HTTPS, replica counts, etc.).

## Secure defaults

| Rule | What this module does |
|---|---|
| R1 — Auth ON | `SPARQ_AUTH_TOKEN` injected from cloud secret store; no unauthenticated public endpoint |
| R2 — No anonymous write | Token required for writes; LWS dev escape hatches (`ALLOW_LOOPBACK` etc.) absent |
| R3 — TLS at edge | AWS: ALB + ACM; Azure: Container Apps auto-HTTPS; GCP: Cloud Run auto-HTTPS |
| R4 — Secrets in store | Token in Secrets Manager / Key Vault / Secret Manager; `sensitive = true` variable |
| R5 — Least-privilege IAM | Dedicated task/execution role (AWS) / managed identity (Azure) / service account (GCP); no wildcard grants |
| R6 — Ingress scoped | App port only; ALB SG source-based (AWS); Container Apps external ingress on targetPort only |
| R7 — Health checks | `/health` (sparq-server) or `/readyz`+`/livez` (lws); parameterised, not shared constant |
| R8 — Non-root | Images run non-root by default; not overridden; read-only rootfs where supported |

## LWS notes

LWS requires an external OIDC provider — it cannot self-issue. Set
`solid_server_base_url` and `solid_server_trusted_issuer`. For multi-replica
LWS deployments a shared Redis replay store is needed (single-instance default
avoids DPoP replay collision; `max_replicas=1` is the safe default for lws).

## Static CI validation

The `.github/workflows/deploy-terraform-lint.yml` workflow runs
`terraform init -backend=false && terraform validate && terraform fmt -check`
per submodule, plus a secret-literal grep (R4 hygiene), on every PR touching
`deploy/terraform/**`. No cloud credentials required. This workflow is
non-gating (jobs are marked advisory, per design record §4).

`terraform plan` requires provider credentials and is not run in CI.
`terraform apply` requires credentials and is run locally or from a secure
pipeline with cloud credentials.
