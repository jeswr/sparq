// [GPT-5.6] sq-44ga1 — canonical data for the statically exported /deploy surface.

export type DeployTarget = "sparq-server" | "solid-lws";

export interface DeployButton {
  label: string;
  target: DeployTarget;
  href: string;
}

export interface DeployOption {
  id:
    | "aws"
    | "azure"
    | "gcp"
    | "fly"
    | "render"
    | "railway"
    | "terraform"
    | "helm";
  name: string;
  mode: "One-click" | "One-click + CLI" | "CLI / IaC";
  summary: string;
  security: string;
  docsHref: string;
  buttons: readonly DeployButton[];
  commandLabel?: string;
  command?: string;
  caveat?: string;
}

const REPO = "https://github.com/sparq-org/sparq";

export const OPEN_BY_DEFAULT_CAVEAT =
  "sparq-server is open-by-default at the image layer; these templates gate it with a token — do not remove the token wiring.";

export const SECURE_DEFAULTS = [
  {
    rule: "Auth on",
    detail:
      "Public sparq-server deployments inject a token from the provider secret store. Anonymous writes must return 401 or 403.",
  },
  {
    rule: "HTTPS only",
    detail:
      "Terminate TLS at the managed edge. Never send a bearer token to a public plaintext endpoint.",
  },
  {
    rule: "Secrets stay secret",
    detail:
      "Templates reference Secrets Manager, Key Vault, Secret Manager, Kubernetes Secrets, or the PaaS vault; they do not commit credential values.",
  },
  {
    rule: "Server-specific health",
    detail:
      "Probe /health for sparq-server, and /livez plus /readyz for Solid/LWS. Keep the checked-in single-instance defaults unless shared state is configured.",
  },
] as const;

// [OPUS-5] sq-cepjb — the ephemeral Cloud Run demo posture (deploy/demo/), kept here so
// site/test/deploy.test.mjs can pin the links and the caveats a visitor must read first.
export const DEMO_ENVIRONMENT = {
  manifestsHref: `${REPO}/blob/main/deploy/demo/README.md`,
  designHref: `${REPO}/blob/main/research/lws-demo-architecture.md`,
  caveats: [
    {
      rule: "Throwaway identities",
      detail:
        "The bundled identity provider accepts unverified registrations and holds accounts, signing keys, and tokens in memory. Nothing in the demo attests that an identity flow is production-ready.",
    },
    {
      rule: "No isolation between visitors",
      detail:
        "Every signed-in visitor shares one playground container and can read, change, or delete what another visitor wrote. Only anonymous writes are refused.",
    },
    {
      rule: "Wiped when idle",
      detail:
        "Cloud Run reclaims idle instances on its own schedule and the two services scale down independently, so there is no guaranteed wipe deadline. If a request starts returning 401 or data disappears, register again.",
    },
  ],
} as const;

export const DEPLOY_OPTIONS: readonly DeployOption[] = [
  {
    id: "aws",
    name: "AWS",
    mode: "CLI / IaC",
    summary: "ECS Fargate behind an HTTPS Application Load Balancer.",
    security:
      "The task reads its token from Secrets Manager; ACM terminates TLS and the task port accepts traffic only from the load balancer.",
    docsHref: `${REPO}/tree/main/deploy/aws`,
    buttons: [],
    commandLabel: "CloudFormation starter",
    command: [
      "aws cloudformation create-stack \\",
      "  --stack-name sparq-server \\",
      "  --template-body file://deploy/aws/sparq-server.yaml \\",
      "  --capabilities CAPABILITY_IAM \\",
      "  --parameters ParameterKey=AuthTokenSecretArn,ParameterValue=<SECRET_ARN> \\",
      "               ParameterKey=AcmCertificateArn,ParameterValue=<ACM_CERT_ARN>",
    ].join("\n"),
    caveat:
      "CloudFormation Launch Stack requires an S3-hosted template. sparq does not publish one to a project-owned bucket, so this page does not present a non-working launch button.",
  },
  {
    id: "azure",
    name: "Azure",
    mode: "One-click + CLI",
    summary: "Azure Container Apps with managed HTTPS ingress.",
    security:
      "The template stores the token in Key Vault, uses a dedicated managed identity, and pins the writable service to one replica.",
    docsHref: `${REPO}/tree/main/deploy/azure`,
    buttons: [
      {
        label: "Deploy SPARQL to Azure",
        target: "sparq-server",
        href: "https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Fsparq-server%2Fazuredeploy.json",
      },
      {
        label: "Deploy Solid/LWS to Azure",
        target: "solid-lws",
        href: "https://portal.azure.com/#create/Microsoft.Template/uri/https%3A%2F%2Fraw.githubusercontent.com%2Fsparq-org%2Fsparq%2Fmain%2Fdeploy%2Fazure%2Flws%2Fazuredeploy.json",
      },
    ],
    commandLabel: "Azure CLI starter",
    command: [
      "az group create --name sparq-rg --location eastus",
      "az deployment group create \\",
      "  --resource-group sparq-rg \\",
      "  --template-file deploy/azure/sparq-server/main.bicep \\",
      '  --parameters authToken="$(openssl rand -hex 32)"',
    ].join("\n"),
  },
  {
    id: "gcp",
    name: "Google Cloud",
    mode: "CLI / IaC",
    summary: "Cloud Run with managed HTTPS and provider-native health probes.",
    security:
      "The manifest references Secret Manager through a dedicated runtime service account and gates both reads and writes with the application token.",
    docsHref: `${REPO}/tree/main/deploy/gcp`,
    buttons: [],
    commandLabel: "Cloud Run deploy after prerequisites",
    command: [
      'RENDERED="$(mktemp)"',
      'trap \'rm -f "${RENDERED}"\' EXIT',
      "sed -e \"s/PROJECT_ID/${PROJECT_ID}/g\" \\",
      "    -e \"s/PROJECT_NUMBER/${PROJECT_NUMBER}/g\" \\",
      '    deploy/gcp/sparq-server.yaml >"${RENDERED}"',
      "gcloud run services replace \"${RENDERED}\" \\",
      '  --region="${REGION}" --project="${PROJECT_ID}"',
    ].join("\n"),
    caveat:
      "Cloud Run Button source-builds a repository directory; it cannot safely apply these manifests and their IAM/Secret Manager prerequisites. Use the reviewed guide instead.",
  },
  {
    id: "fly",
    name: "Fly.io",
    mode: "One-click + CLI",
    summary: "A small managed deployment with automatic HTTPS.",
    security:
      "The config fails closed until required secrets exist, forces HTTPS, and keeps one Machine for the current process-local stores.",
    docsHref: `${REPO}/blob/main/deploy/paas/README.md#flyio`,
    buttons: [
      {
        label: "Deploy SPARQL on Fly.io",
        target: "sparq-server",
        href: `https://fly.io/launch?source=github&template=${REPO}/tree/main/deploy/paas/sparq-server`,
      },
      {
        label: "Deploy Solid/LWS on Fly.io",
        target: "solid-lws",
        href: `https://fly.io/launch?source=github&template=${REPO}/tree/main/deploy/paas/lws`,
      },
    ],
    commandLabel: "Secure Fly CLI path",
    command: [
      "cd deploy/paas/sparq-server",
      "fly launch --copy-config --generate-name --no-deploy --ha=false",
      'fly secrets set SPARQ_AUTH_TOKEN="$(openssl rand -hex 32)"',
      "fly deploy --ha=false && fly scale count 1",
    ].join("\n"),
  },
  {
    id: "render",
    name: "Render",
    mode: "One-click",
    summary: "Blueprint deployments with automatic HTTPS.",
    security:
      "The SPARQL Blueprint generates its token in a secret environment group, requires it for reads and writes, and pins one instance.",
    docsHref: `${REPO}/blob/main/deploy/paas/README.md#render`,
    buttons: [
      {
        label: "Deploy SPARQL to Render",
        target: "sparq-server",
        href: `https://render.com/deploy?repo=${REPO}&branch=main&blueprint=deploy/paas/sparq-server/render.yaml`,
      },
      {
        label: "Deploy Solid/LWS to Render",
        target: "solid-lws",
        href: `https://render.com/deploy?repo=${REPO}&branch=main&blueprint=deploy/paas/lws/render.yaml`,
      },
    ],
  },
  {
    id: "railway",
    name: "Railway",
    mode: "One-click + CLI",
    summary: "Config-as-code deployments with managed TLS.",
    security:
      "The template prompts for required variables before launch, enables read auth for sparq-server, and pins one replica.",
    docsHref: `${REPO}/blob/main/deploy/paas/README.md#railway`,
    buttons: [
      {
        label: "Deploy SPARQL on Railway",
        target: "sparq-server",
        href: `https://railway.com/new/template?template=${REPO}/tree/main/deploy/paas/sparq-server&envs=SPARQ_AUTH_TOKEN%2CSPARQ_AUTH_TOKEN_READ%2CPORT&SPARQ_AUTH_TOKENDesc=Required+bearer+token+for+reads+and+writes&SPARQ_AUTH_TOKEN_READDesc=Require+the+token+for+reads+and+writes&SPARQ_AUTH_TOKEN_READDefault=1&PORTDesc=Fixed+sparq-server+listen+and+health-check+port&PORTDefault=3030`,
      },
      {
        label: "Deploy Solid/LWS on Railway",
        target: "solid-lws",
        href: `https://railway.com/new/template?template=${REPO}/tree/main/deploy/paas/lws&envs=SOLID_SERVER_BASE_URL%2CSOLID_SERVER_TRUSTED_ISSUER%2CSOLID_SERVER_BIND%2CPORT&SOLID_SERVER_BASE_URLDesc=Required+public+HTTPS+origin&SOLID_SERVER_TRUSTED_ISSUERDesc=Required+external+OIDC+issuer+URL&SOLID_SERVER_BINDDesc=Required+public+container+bind&SOLID_SERVER_BINDDefault=0.0.0.0%3A3000&PORTDesc=Required+Railway+health-check+port&PORTDefault=3000`,
      },
    ],
    commandLabel: "Secure Railway CLI path",
    command: [
      "railway init && railway add --service sparq-server",
      "openssl rand -hex 32 | railway variables set SPARQ_AUTH_TOKEN --stdin --skip-deploys",
      "railway variables set SPARQ_AUTH_TOKEN_READ=1 PORT=3030 --skip-deploys",
      "railway up deploy/paas/sparq-server --path-as-root",
    ].join("\n"),
  },
  {
    id: "terraform",
    name: "Terraform",
    mode: "CLI / IaC",
    summary: "One module selecting AWS, Azure, or GCP.",
    security:
      "The module generates a token when omitted, stores it in the selected cloud secret store, and marks supplied token input sensitive.",
    docsHref: `${REPO}/tree/main/deploy/terraform`,
    buttons: [],
    commandLabel: "Terraform starter",
    command: [
      "cd deploy/terraform",
      "terraform init",
      "terraform apply -var target=azure -var azure_location=eastus",
    ].join("\n"),
  },
  {
    id: "helm",
    name: "Kubernetes / Helm",
    mode: "CLI / IaC",
    summary: "A secure-default chart plus a plain-manifest quickstart.",
    security:
      "The chart requires an existing Secret, uses a minimal ServiceAccount, keeps the Service cluster-internal, and requires TLS for its ingress.",
    docsHref: `${REPO}/tree/main/deploy/helm`,
    buttons: [],
    commandLabel: "Helm starter",
    command: [
      "kubectl create namespace sparq",
      "openssl rand -hex 32 | tr -d '\\n' | kubectl create secret generic sparq-auth-token \\",
      "  --namespace sparq --from-file=SPARQ_AUTH_TOKEN=/dev/stdin",
      "helm install sparq ./deploy/helm/sparq --namespace sparq \\",
      "  --set auth.existingSecret=sparq-auth-token",
    ].join("\n"),
  },
];

export const ONE_CLICK_BUTTONS = DEPLOY_OPTIONS.flatMap((option) =>
  option.buttons.map((button) => ({ provider: option.id, ...button })),
);
