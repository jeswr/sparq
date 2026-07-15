# sparq Terraform — GCP submodule variables (sq-sos84) [SONNET]

variable "name" {
  description = "Base name for all resources (max 28 chars for service account)"
  type        = string
  default     = "sparq"
}

variable "server" {
  description = "sparq-server | lws"
  type        = string
  default     = "sparq-server"

  validation {
    condition     = contains(["sparq-server", "lws"], var.server)
    error_message = "server must be sparq-server or lws."
  }
}

variable "image" {
  description = "Full container image ref including tag"
  type        = string
}

variable "container_port" {
  description = "Container port (3030 for sparq-server, 3000 for lws)"
  type        = number
}

variable "health_path" {
  description = "Health-check path (/health or /readyz) — parameterised per R7"
  type        = string

  validation {
    condition     = startswith(var.health_path, "/")
    error_message = "health_path must start with '/'."
  }
}

variable "auth_token" {
  description = "SPARQ_AUTH_TOKEN — stored in Secret Manager, never a literal (R4)"
  type        = string
  default     = null
  nullable    = true
  sensitive   = true
}

variable "gcp_project" {
  description = "GCP project ID"
  type        = string

  validation {
    condition     = trimspace(var.gcp_project) != ""
    error_message = "gcp_project must not be empty."
  }
}

variable "gcp_region" {
  description = "GCP region (e.g. 'us-central1')"
  type        = string
  default     = "us-central1"
}

variable "cpu" {
  description = "CPU limit for Cloud Run container (1, 2, 4, 6, or 8 vCPU)"
  type        = string
  # [GPT-5.6] Cloud Run rejects the root's former AWS-style value and current
  # Cloud Run v2 accepts integer vCPU quantities.
  default = "1"

  validation {
    condition     = contains(["1", "2", "4", "6", "8"], var.cpu)
    error_message = "cpu must be one of Cloud Run's supported vCPU values: 1, 2, 4, 6, 8."
  }
}

variable "memory" {
  description = "Memory limit for Cloud Run container (e.g. '512Mi', '1Gi')"
  type        = string
  default     = "512Mi"
}

variable "min_instances" {
  description = <<-EOT
    Minimum Cloud Run instances. Set >= 1 to avoid cold-start latency on the
    health path (R7). Single-instance avoids multi-replica DPoP replay
    collision for lws (§1.3 of design record).
  EOT
  type        = number
  default     = 1
}

variable "max_instances" {
  description = "Maximum Cloud Run instances"
  type        = number
  # [GPT-5.6] Safe for direct lws module use; the root raises this only for
  # sparq-server and pins lws to one without Redis replay wiring.
  default = 1
}

variable "allow_unauthenticated" {
  description = <<-EOT
    Allow unauthenticated Cloud Run invocations (allUsers invoker). Default true
    because sparq-server uses its own Bearer token (R1) as the access control.
    Set false to add Cloud Run IAM as a second layer for internal deployments.
  EOT
  type        = bool
  default     = true
}

variable "solid_server_base_url" {
  description = "SOLID_SERVER_BASE_URL (lws only)"
  type        = string
  default     = ""
}

variable "solid_server_trusted_issuer" {
  description = "SOLID_SERVER_TRUSTED_ISSUER (lws only)"
  type        = string
  default     = ""
}
