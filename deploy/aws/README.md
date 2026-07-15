<!-- [SONNET-4.6] sq-17rgw — AWS CloudFormation Launch-Stack deploy for sparq-server + sparq-lws-core. -->

# AWS one-click deploy (sq-17rgw)

CloudFormation templates for running sparq on AWS **ECS Fargate** with an
**Application Load Balancer** and **HTTPS via ACM**.

Two templates:

| Template | Image | Port | Health path | Status |
|---|---|---|---|---|
| `sparq-server.yaml` | `ghcr.io/sparq-org/sparq-server` | 3030 | `GET /health` | Available |
| `sparq-lws.yaml` | `ghcr.io/sparq-org/sparq-lws-core` | 3000 | `GET /readyz` | Activates once sq-lmz40 ships |

---

## Secure-defaults notice (R1/R2/R4/R9)

> **sparq-server is open-by-default at the image layer.** It bakes `SPARQ_ALLOW_REMOTE=1`
> and ships with no auth. Anyone who reaches the published port can read AND write the
> dataset. **These templates enforce auth ON** by injecting `SPARQ_AUTH_TOKEN` from AWS
> Secrets Manager (ValueFrom — the literal value never appears in CloudFormation). Do not
> remove the Secrets Manager wiring or set a default token value in the template.

Key rules applied:

- **R1 (auth ON):** `SPARQ_AUTH_TOKEN` is required and comes from Secrets Manager — never
  a template parameter value.
- **R2 (no anonymous write):** Unauthenticated writes return 401. sparq-lws-core is
  fail-closed by design; anonymous mutation is rejected at the image layer.
- **R3 (TLS at edge):** The ALB terminates HTTPS (TLS 1.2+ with TLS 1.3 preferred) via an
  ACM certificate. Port 80 redirects to 443. The task SG accepts traffic only from the ALB
  SG on the app port — not from the internet directly.
- **R4 (secrets in Secrets Manager):** No literal token, password, or key in any committed
  file. Secrets are referenced by ARN only.
- **R5 (least-privilege IAM):** Separate `ExecutionRole` (image pull + secret read, scoped
  to the stack's own secret ARN) and `TaskRole` (CloudWatch logs only).
- **R6 (ingress only on app port from ALB SG):** Tasks run in private subnets (no public
  IP). The ALB SG allows 443 from the internet; the task SG allows the app port only from
  the ALB SG.
- **R7 (health check):** ALB target group health check on `/health` (sparq-server) or
  `/readyz` (LWS), matcher 200. ECS liveness check via the server's built-in `--health-probe`.
- **R8 (non-root):** Both images run as non-root; `ReadonlyRootFilesystem: true` in the
  task definition.

---

## sparq-server — Launch Stack

### Prerequisites

1. An ACM certificate covering the domain you will serve (must be in the same AWS region).
2. A Secrets Manager secret holding the bearer token:

   ```bash
   aws secretsmanager create-secret \
     --name /sparq/server/auth-token \
     --secret-string "$(openssl rand -hex 32)"
   # Note the ARN from the output.
   ```

### Launch Stack button

Replace the `<REGION>` and URL-encode the parameter values before clicking.

[![Launch Stack](https://s3.amazonaws.com/cloudformation-examples/cloudformation-launch-stack.png)](https://console.aws.amazon.com/cloudformation/home#/stacks/create/review?templateURL=https://raw.githubusercontent.com/sparq-org/sparq/main/deploy/aws/sparq-server.yaml&stackName=sparq-server)

> The button above links to the template on the `main` branch. For production, pin to a
> specific release tag by substituting the commit SHA in the raw URL.

### CLI deploy

```bash
aws cloudformation create-stack \
  --stack-name sparq-server \
  --template-body file://sparq-server.yaml \
  --capabilities CAPABILITY_NAMED_IAM \
  --parameters \
    ParameterKey=AuthTokenSecretArn,ParameterValue=arn:aws:secretsmanager:<REGION>:<ACCOUNT>:secret:/sparq/server/auth-token-XXXXXX \
    ParameterKey=AcmCertificateArn,ParameterValue=arn:aws:acm:<REGION>:<ACCOUNT>:certificate/<UUID>
```

### Post-deploy

1. The stack outputs `AlbDnsName`. Create a CNAME from your domain to that value (or use
   Route 53 alias with the `AlbHostedZoneId` output).
2. Test the SPARQL endpoint:

   ```bash
   TOKEN=$(aws secretsmanager get-secret-value --secret-id /sparq/server/auth-token --query SecretString --output text)
   curl -H "Authorization: Bearer $TOKEN" https://<your-domain>/sparql \
     --data-urlencode 'query=SELECT * WHERE { ?s ?p ?o } LIMIT 1'
   ```

3. Confirm an unauthenticated write is rejected:

   ```bash
   curl -X POST https://<your-domain>/sparql/update \
     -d 'update=INSERT DATA { <urn:x> <urn:y> <urn:z> }' \
     -w "%{http_code}"
   # Expect: 401
   ```

### Parameters

| Parameter | Required | Default | Description |
|---|---|---|---|
| `AuthTokenSecretArn` | Yes | — | ARN of Secrets Manager secret holding the bearer token |
| `AcmCertificateArn` | Yes | — | ARN of ACM certificate for HTTPS |
| `ImageRef` | No | `ghcr.io/sparq-org/sparq-server:latest` | Full image reference |
| `TaskCpu` | No | `512` | Fargate CPU units |
| `TaskMemory` | No | `1024` | Fargate memory (MiB) |
| `DesiredCount` | No | `1` | Number of running tasks |
| `AuthTokenRead` | No | `""` | Set `"1"` to also gate reads |
| `CorsAllowOrigin` | No | `""` | Value for `SPARQ_CORS_ALLOW_ORIGIN` |
| `MaxConcurrent` | No | `16` | Max concurrent query workers |
| `QueryTimeoutSeconds` | No | `30` | Per-query timeout |

---

## sparq-lws-core (Solid server) — Launch Stack

> **Status: activates once [sq-lmz40](https://github.com/sparq-org/sparq/issues) ships.**
> The `ghcr.io/sparq-org/sparq-lws-core` image is not yet published. The template is
> structurally complete and cfn-lint valid; deploy once the image is available.

The LWS server requires an external OIDC issuer (`SOLID_SERVER_TRUSTED_ISSUER`) — it cannot
self-issue. This is a required parameter. Store the issuer URL in Secrets Manager:

```bash
aws secretsmanager create-secret \
  --name /sparq/lws/trusted-issuer \
  --secret-string "https://your-solid-oidc-provider.example"
```

### CLI deploy

```bash
aws cloudformation create-stack \
  --stack-name sparq-lws \
  --template-body file://sparq-lws.yaml \
  --capabilities CAPABILITY_NAMED_IAM \
  --parameters \
    ParameterKey=AcmCertificateArn,ParameterValue=arn:aws:acm:<REGION>:<ACCOUNT>:certificate/<UUID> \
    ParameterKey=SolidBaseUrl,ParameterValue=https://solid.example.com \
    ParameterKey=SolidTrustedIssuerSecretArn,ParameterValue=arn:aws:secretsmanager:<REGION>:<ACCOUNT>:secret:/sparq/lws/trusted-issuer-XXXXXX
```

### Multi-replica note

For `DesiredCount > 1`, you must supply a Redis URL via `RedisReplaySecretArn` for shared
DPoP-jti replay protection. With `DesiredCount=1` (the default), in-memory replay is
correct and no Redis is needed.

### Parameters

| Parameter | Required | Default | Description |
|---|---|---|---|
| `AcmCertificateArn` | Yes | — | ACM certificate for HTTPS |
| `SolidBaseUrl` | Yes | — | Public HTTPS base URL (`https://solid.example.com`) |
| `SolidTrustedIssuerSecretArn` | Yes | — | Secrets Manager ARN for the OIDC issuer URL |
| `ImageRef` | No | `ghcr.io/sparq-org/sparq-lws-core:latest` | Full image reference |
| `SolidAudience` | No | `""` | `SOLID_SERVER_AUDIENCE` (defaults to `SolidBaseUrl`) |
| `RedisReplaySecretArn` | No | `""` | Secrets Manager ARN for Redis URL (multi-replica) |
| `RedisReplayUrl` | No | `""` | Redis URL plaintext (single-replica only) |
| `TaskCpu` | No | `512` | Fargate CPU units |
| `TaskMemory` | No | `1024` | Fargate memory (MiB) |
| `DesiredCount` | No | `1` | Number of running tasks |

---

## Architecture

```
Internet
  │ HTTPS 443
  ▼
ALB (public subnets A+B)
  │ HTTP 3030 (sparq-server) or HTTP 3000 (LWS)
  │ ALB SG → Task SG (source-based, not CIDR)
  ▼
ECS Fargate task (private subnets A+B, no public IP)
  │ egress via NAT Gateway → GHCR (image pull) + Secrets Manager + CloudWatch
  ▼
CloudWatch Logs (/ecs/<stack>/sparq-server or sparq-lws)
```

Each template provisions a self-contained VPC. To deploy into an existing VPC, extract the
networking resources and supply your own subnet IDs — a future enhancement (note created as
discovered work).

---

## CI validation

These templates are validated by `cfn-lint` (exit 0 on both). The
`aws cloudformation validate-template` call requires `cloudformation:ValidateTemplate` IAM
permission and a live AWS endpoint — the dev box IAM role does not allow this, so it is
documented as manual validation.

Secret hygiene: `grep -rEin "SPARQ_AUTH_TOKEN\s*[:=]\s*['\"][a-zA-Z0-9+/]{16,}" deploy/aws/`
returns no matches.

A CI workflow for automatic cfn-lint on `paths: deploy/aws/**` is tracked as discovered work
(see below).

---

## Discovered work

The following was noted during implementation but is out of scope for this bead:

- A `deploy/aws/` CI workflow running `cfn-lint` on `push`/`PR` for `paths: deploy/aws/**`
  (non-gating, `workflow_dispatch` + path filter as per design record §4).
- Parameterising the VPC so operators can supply an existing VPC + subnets instead of having
  each stack create its own.
- An EC2 fallback template (user-data `docker run` behind Elastic IP or ALB) for operators
  who prefer EC2 over Fargate — noted in §3.1 of the design record as optional.
- Auto-generation of a random token at stack deploy time (via CloudFormation custom resource
  or Lambda-backed resource) so the `AuthTokenSecretArn` parameter is not required upfront.
