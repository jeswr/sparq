# sparq multi-cloud Terraform root — variable declarations (sq-sos84) [SONNET]

# ---------------------------------------------------------------------------
# Required: selector variables
# ---------------------------------------------------------------------------

variable "target" {
  description = "Target cloud provider: aws | azure | gcp"
  type        = string
  validation {
    condition     = contains(["aws", "azure", "gcp"], var.target)
    error_message = "target must be one of: aws, azure, gcp"
  }
}

variable "server" {
  description = "Which sparq server to deploy: sparq-server (SPARQL endpoint, port 3030) or lws (Solid/LWS server, port 3000)"
  type        = string
  default     = "sparq-server"
  validation {
    condition     = contains(["sparq-server", "lws"], var.server)
    error_message = "server must be one of: sparq-server, lws"
  }
}

variable "auth_token" {
  description = <<-EOT
    (R1/R2/R4) Bearer auth token for sparq-server. Injected as SPARQ_AUTH_TOKEN
    into the cloud secret store — never stored as a plaintext literal.
    sparq-server is OPEN BY DEFAULT at the image layer; this token is mandatory
    when exposing a public endpoint. For lws, this value is unused (lws is
    fail-closed by design), but the variable is required to avoid silent
    open deployments if target is later changed.
  EOT
  type        = string
  sensitive   = true
}

# ---------------------------------------------------------------------------
# Image ref (defaulted to canonical per §1 of design record)
# ---------------------------------------------------------------------------

variable "image_override" {
  description = "Override the image registry path (default: ghcr.io/sparq-org/<server>). Use for private registry or fork."
  type        = string
  default     = ""
}

variable "image_tag" {
  description = "Image tag to deploy (e.g. 'latest', 'v0.8.0')"
  type        = string
  default     = "latest"
}

# ---------------------------------------------------------------------------
# Health-check (R7)
# ---------------------------------------------------------------------------

variable "health_path_override" {
  description = <<-EOT
    Override the HTTP health-check path. Defaults: sparq-server=/health,
    lws=/readyz. The health-check path is per-server and parameterised (R7).
  EOT
  type    = string
  default = ""
}

# ---------------------------------------------------------------------------
# Naming
# ---------------------------------------------------------------------------

variable "name" {
  description = "Base name for all provisioned resources (e.g. 'sparq-prod')"
  type        = string
  default     = "sparq"
}

# ---------------------------------------------------------------------------
# Sizing (shared across targets where applicable)
# ---------------------------------------------------------------------------

variable "cpu" {
  description = "CPU units (AWS: 256/512/1024 vCPU fractions; Azure: cores; GCP: 1000m notation)"
  type        = string
  default     = "512"
}

variable "memory" {
  description = "Memory: AWS in MiB (e.g. '1024'), Azure in Gi (e.g. '2.0Gi'), GCP in Mi (e.g. '512Mi')"
  type        = string
  default     = "1024"
}

# ---------------------------------------------------------------------------
# AWS-specific
# ---------------------------------------------------------------------------

variable "aws_region" {
  description = "AWS region (e.g. 'us-east-1')"
  type        = string
  default     = "us-east-1"
}

# ---------------------------------------------------------------------------
# Azure-specific
# ---------------------------------------------------------------------------

variable "azure_location" {
  description = "Azure location (e.g. 'eastus')"
  type        = string
  default     = "eastus"
}

variable "azure_rg_name" {
  description = "Azure resource group name (created if it does not exist)"
  type        = string
  default     = "sparq-rg"
}

variable "min_replicas" {
  description = "Minimum replica count (Azure Container Apps / GCP Cloud Run)"
  type        = number
  default     = 1
}

variable "max_replicas" {
  description = "Maximum replica count (Azure Container Apps)"
  type        = number
  default     = 3
}

# ---------------------------------------------------------------------------
# GCP-specific
# ---------------------------------------------------------------------------

variable "gcp_project" {
  description = "GCP project ID"
  type        = string
  default     = ""
}

variable "gcp_region" {
  description = "GCP region (e.g. 'us-central1')"
  type        = string
  default     = "us-central1"
}

variable "min_instances" {
  description = "Minimum instances for GCP Cloud Run (>=1 avoids cold-start on health path, R7)"
  type        = number
  default     = 1
}

variable "max_instances" {
  description = "Maximum instances for GCP Cloud Run"
  type        = number
  default     = 3
}

# ---------------------------------------------------------------------------
# LWS-only parameters (required when server=lws)
# ---------------------------------------------------------------------------

variable "solid_server_base_url" {
  description = <<-EOT
    (LWS only, required when server=lws) Public HTTPS base URL of the Solid server
    (e.g. 'https://solid.example.com'). An LWS deploy cannot be zero-input:
    Solid RS cannot self-issue — it needs a known public base URL at boot.
    Ignored for server=sparq-server.
  EOT
  type    = string
  default = ""
}

variable "solid_server_trusted_issuer" {
  description = <<-EOT
    (LWS only, required when server=lws) HTTPS URL of the trusted OIDC issuer
    (e.g. 'https://idp.example.com'). Dev-only escape hatches
    (SOLID_SERVER_ALLOW_LOOPBACK, SOLID_SERVER_SEED_CONFORMANCE) are NEVER set
    in these templates (R2).
    Ignored for server=sparq-server.
  EOT
  type    = string
  default = ""
}
