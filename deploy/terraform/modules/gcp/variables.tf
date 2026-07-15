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
}

variable "auth_token" {
  description = "SPARQ_AUTH_TOKEN — stored in Secret Manager, never a literal (R4)"
  type        = string
  sensitive   = true
}

variable "gcp_project" {
  description = "GCP project ID"
  type        = string
}

variable "gcp_region" {
  description = "GCP region (e.g. 'us-central1')"
  type        = string
  default     = "us-central1"
}

variable "cpu" {
  description = "CPU limit for Cloud Run container (e.g. '1000m' or '1')"
  type        = string
  default     = "1000m"
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
  type    = number
  default = 1
}

variable "max_instances" {
  description = "Maximum Cloud Run instances"
  type        = number
  default     = 3
}

variable "allow_unauthenticated" {
  description = <<-EOT
    Allow unauthenticated Cloud Run invocations (allUsers invoker). Default true
    because sparq-server uses its own Bearer token (R1) as the access control.
    Set false to add Cloud Run IAM as a second layer for internal deployments.
  EOT
  type    = bool
  default = true
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
