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
    (R1/R2/R4) Optional pre-existing Bearer token for sparq-server. When null,
    Terraform generates a 48-character token and writes it to the selected
    cloud secret store. The container receives only a secret-store reference.
    LWS is fail-closed and does not create or consume this secret.
  EOT
  type        = string
  default     = null
  nullable    = true
  sensitive   = true

  # [GPT-5.6] Reject empty or trivially weak operator-supplied credentials.
  validation {
    condition     = var.auth_token == null || try(length(var.auth_token) >= 32, false)
    error_message = "auth_token must be null (generate one) or at least 32 characters."
  }
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

  validation {
    condition     = trimspace(var.image_tag) != ""
    error_message = "image_tag must not be empty."
  }
}

# ---------------------------------------------------------------------------
# Health-check (R7)
# ---------------------------------------------------------------------------

variable "health_path_override" {
  description = <<-EOT
    Override the HTTP health-check path. Defaults: sparq-server=/health,
    lws=/readyz. The health-check path is per-server and parameterised (R7).
  EOT
  type        = string
  default     = ""

  validation {
    condition     = var.health_path_override == "" || startswith(var.health_path_override, "/")
    error_message = "health_path_override must be empty or start with '/'."
  }
}

# ---------------------------------------------------------------------------
# Naming
# ---------------------------------------------------------------------------

variable "name" {
  description = "Base name for all provisioned resources (e.g. 'sparq-prod')"
  type        = string
  default     = "sparq"

  # [GPT-5.6] Portable subset of AWS ALB/IAM, Azure Key Vault, and Cloud Run
  # naming rules; avoids provider-specific apply failures from one root name.
  validation {
    condition = (
      can(regex("^[a-z][a-z0-9-]{2,15}[a-z0-9]$", var.name)) &&
      length(regexall("--", var.name)) == 0
    )
    error_message = "name must be 4-17 lowercase letters, digits, or hyphens; start with a letter and end with a letter or digit."
  }
}

# ---------------------------------------------------------------------------
# Sizing (shared across targets where applicable)
# ---------------------------------------------------------------------------

variable "cpu" {
  description = "Optional provider-native CPU override (AWS units; Azure cores; GCP integer vCPU). Empty selects a valid per-provider default."
  type        = string
  default     = ""
}

variable "memory" {
  description = "Optional provider-native memory override (AWS MiB; Azure Gi; GCP Mi/Gi). Empty selects a valid per-provider default."
  type        = string
  default     = ""
}

# ---------------------------------------------------------------------------
# AWS-specific
# ---------------------------------------------------------------------------

variable "aws_region" {
  description = "AWS region (e.g. 'us-east-1')"
  type        = string
  default     = "us-east-1"
}

# [GPT-5.6] AWS is always public HTTPS at the ALB. The certificate must already
# exist in ACM in aws_region; plaintext public listeners are not supported.
variable "aws_acm_certificate_arn" {
  description = "ACM certificate ARN for the public AWS ALB HTTPS listener; required when target=aws"
  type        = string
  default     = ""
}

variable "aws_public_hostname" {
  description = "Public DNS hostname covered by the ACM certificate; required when target=aws"
  type        = string
  default     = ""
}

variable "aws_route53_zone_id" {
  description = "Optional Route 53 hosted zone ID in which to create aws_public_hostname; use external DNS when empty"
  type        = string
  default     = ""
}

variable "aws_vpc_id" {
  description = "AWS VPC ID; empty uses the account's default VPC"
  type        = string
  default     = ""
}

variable "aws_alb_subnet_ids" {
  description = "Public subnet IDs for the internet-facing ALB; empty uses all default-VPC subnets"
  type        = list(string)
  default     = []
}

variable "aws_task_subnet_ids" {
  description = "ECS task subnet IDs; empty reuses the ALB/default-VPC subnets"
  type        = list(string)
  default     = []
}

variable "aws_assign_public_ip" {
  description = "Assign task public IPs for outbound GHCR/cloud API access. Set false only for private subnets with NAT or required VPC endpoints."
  type        = bool
  default     = true
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
  description = "Azure resource group name to create (import it first if it already exists)"
  type        = string
  default     = "sparq-rg"
}

variable "min_replicas" {
  # [GPT-5.6] Retained for interface compatibility; the root enforces one replica.
  description = "Reserved Azure Container Apps minimum replica count; forced to 1 because these templates provision no shared state"
  type        = number
  default     = 1
}

variable "max_replicas" {
  # [GPT-5.6] Retained for interface compatibility; the root enforces one replica.
  description = "Reserved Azure Container Apps maximum replica count; forced to 1 because these templates provision no shared state"
  type        = number
  default     = 1
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
  # [GPT-5.6] Retained for interface compatibility; the root enforces one instance.
  description = "Reserved GCP Cloud Run minimum instance count; forced to 1 because these templates provision no shared state"
  type        = number
  default     = 1
}

variable "max_instances" {
  # [GPT-5.6] Retained for interface compatibility; the root enforces one instance.
  description = "Reserved GCP Cloud Run maximum instance count; forced to 1 because these templates provision no shared state"
  type        = number
  default     = 1
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
  type        = string
  default     = ""
}

variable "solid_server_trusted_issuer" {
  description = <<-EOT
    (LWS only, required when server=lws) HTTPS URL of the trusted OIDC issuer
    (e.g. 'https://idp.example.com'). Dev-only escape hatches
    (SOLID_SERVER_ALLOW_LOOPBACK, SOLID_SERVER_SEED_CONFORMANCE) are NEVER set
    in these templates (R2).
    Ignored for server=sparq-server.
  EOT
  type        = string
  default     = ""
}
