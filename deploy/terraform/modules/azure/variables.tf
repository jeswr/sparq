# sparq Terraform — Azure submodule variables (sq-sos84) [SONNET]

variable "name" {
  description = "Base name for all resources (max 17 chars; Key Vault adds a 6-char suffix)"
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
  description = "Readiness health-check path (/health or /readyz) — parameterised per R7"
  type        = string
}

variable "auth_token" {
  description = "SPARQ_AUTH_TOKEN — stored in Key Vault, never a literal (R4)"
  type        = string
  sensitive   = true
}

variable "azure_location" {
  description = "Azure region (e.g. 'eastus')"
  type        = string
  default     = "eastus"
}

variable "azure_rg_name" {
  description = "Resource group name (created if it does not exist)"
  type        = string
  default     = "sparq-rg"
}

variable "cpu" {
  description = "CPU cores for the container (Container Apps: 0.25 | 0.5 | 1.0 | 2.0)"
  type        = string
  default     = "0.5"
}

variable "memory" {
  description = "Memory for the container (Container Apps format, e.g. '1.0Gi')"
  type        = string
  default     = "1.0Gi"
}

variable "min_replicas" {
  description = "Minimum Container App replicas"
  type        = number
  default     = 1
}

variable "max_replicas" {
  description = "Maximum Container App replicas"
  type        = number
  default     = 3
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
