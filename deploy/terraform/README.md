# sparq multi-cloud Terraform module (sq-sos84)

Deploy either `sparq-server` (SPARQL 1.1 endpoint) or the Solid/LWS server
(`sparq-lws-core`) into AWS, Azure, or GCP using a single Terraform root module.

**Security note (R9):** `sparq-server` is open-by-default at the image layer
(bakes `SPARQ_ALLOW_REMOTE=1`, no auth). This module enforces auth ON at the
template layer via `auth_token` (R1). **Do not remove the token wiring.**

<!-- [GPT-5.6] Document the state contract behind the scaling invariant. -->
**Single-writer note:** `sparq-server` keeps its dataset only in process memory;
separate replicas would accept writes into divergent datasets. These Terraform
templates therefore enforce exactly one `sparq-server` instance on every cloud.
Raise that limit only after wiring a shared persistent backing store, which
these templates do not provision.

## Quick start

```sh
# AWS — ECS Fargate + ALB + Secrets Manager
terraform init
terraform apply \
  -var target=aws \
  -var aws_region=us-east-1 \
  -var 'aws_acm_certificate_arn=arn:aws:acm:us-east-1:123456789012:certificate/REPLACE-ME' \
  -var aws_public_hostname=sparq.example.com \
  -var aws_route53_zone_id=Z1234567890

# Azure — Container Apps + Key Vault
terraform apply \
  -var target=azure \
  -var azure_location=eastus

# GCP — Cloud Run + Secret Manager
terraform apply \
  -var target=gcp \
  -var gcp_project=my-project \
  -var gcp_region=us-central1
```

<!-- [GPT-5.6] The default avoids command-line secret exposure. -->
Omit `auth_token` to generate a strong token during apply. Terraform writes it
to the target cloud's secret store, and the container receives only that secret
reference (R4). Retrieve it later using the cloud secret-store CLI.

Never commit an `auth_token` override to `.tfvars` or version control. Like all
Terraform-managed secret values, a generated or supplied token is also present
in Terraform state; use an encrypted remote backend with tightly restricted
state access.

## Structure

```
deploy/terraform/
  main.tf          Root module — delegates to ./modules/<target>
  variables.tf     All root variables
  outputs.tf       endpoint_url, service_name, secret_id, AWS DNS target
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
| `auth_token` | generated | Optional 32+ character override; stored in the cloud secret store |
| `image_tag` | `latest` | Image tag to deploy |
| `name` | `sparq` | Base name for all resources |
| `aws_acm_certificate_arn` | `""` | Required for `target=aws`; public plaintext mode is unavailable |
| `aws_public_hostname` | `""` | Required for `target=aws`; must match the ACM certificate |
| `aws_route53_zone_id` | `""` | Optional Route 53 zone; otherwise create the DNS record externally |
| `min_replicas` / `max_replicas` | `1` | Reserved Azure sizing inputs; forced to 1 until shared state is provisioned |
| `min_instances` / `max_instances` | `1` | Reserved GCP sizing inputs; forced to 1 until shared state is provisioned |
| `solid_server_base_url` | `""` | Required when `server=lws` |
| `solid_server_trusted_issuer` | `""` | Required when `server=lws` |

See `variables.tf` for all variables and per-submodule options (region, sizing,
ACM cert ARN for AWS HTTPS, replica counts, etc.).

## Secure defaults

| Rule | What this module does |
|---|---|
| R1 — Auth ON | `SPARQ_AUTH_TOKEN` comes from the cloud secret store and `SPARQ_AUTH_TOKEN_READ=1` gates reads and writes; health remains public |
| R2 — No anonymous write | Token required for writes; LWS dev escape hatches (`ALLOW_LOOPBACK` etc.) absent |
| R3 — TLS at edge | AWS requires ALB + ACM and redirects HTTP; Azure rejects insecure ingress; GCP uses Cloud Run HTTPS |
| R4 — Secrets in store | Token in Secrets Manager / Key Vault / Secret Manager; `sensitive = true` variable |
| R5 — Least-privilege IAM | Dedicated empty task role + scoped execution role (AWS) / managed identity (Azure) / service account (GCP); no global wildcard grants |
| R6 — Ingress scoped | AWS tasks accept traffic only from the ALB; Container Apps and Cloud Run expose only managed HTTPS ingress |
| R7 — Health checks | `/health` (sparq-server) or `/readyz`+`/livez` (lws); parameterised, not shared constant |
| R8 — Non-root | Images run non-root; AWS also uses a read-only rootfs and drops all Linux capabilities |

## LWS notes

LWS requires an external OIDC provider — it cannot self-issue. Set
`solid_server_base_url` and `solid_server_trusted_issuer`. For multi-replica
LWS deployments a shared Redis replay store is needed. This root module does not
wire Redis, so it forces exactly one LWS replica/instance on every provider.

## AWS networking

The default-VPC quick start assigns ECS tasks a public IP so they can pull the
public GHCR image and reach AWS APIs, but the task security group permits no
public ingress: only the ALB security group can reach the app port. For private
tasks, pass `aws_task_subnet_ids` for subnets with NAT or the necessary VPC
endpoints and set `aws_assign_public_ip=false`. Pass public subnets separately
through `aws_alb_subnet_ids`.

AWS returns `https://<aws_public_hostname>` rather than the generated ALB name:
ACM cannot issue a certificate for an `amazonaws.com` hostname. When Route 53 is
not authoritative, point `aws_public_hostname` at the
`aws_load_balancer_dns_name` root output using your external DNS provider before
sending credentials.

## Static CI validation

The `.github/workflows/deploy-terraform-lint.yml` workflow runs
`terraform init -backend=false && terraform validate && terraform fmt -check`
for the root module and each submodule, plus a secret-literal grep (R4 hygiene),
on every PR touching `deploy/terraform/**`. No cloud credentials required. All of
it runs in a single job so the checkout, the Terraform install and the provider
install are paid once rather than once per module. This workflow is non-gating:
its job is declared in `.github/advisory-registry.json`, which since #3773 is the
only thing that excludes a check from the `ci-summary / gate` verdict (per design
record §4).

`terraform plan` requires provider credentials and is not run in CI.
`terraform apply` requires credentials and is run locally or from a secure
pipeline with cloud credentials.
