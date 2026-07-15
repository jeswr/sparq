# sparq Terraform — AWS submodule variables (sq-sos84) [SONNET]

variable "name" {
  description = "Base name for all resources"
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
  description = "SPARQ_AUTH_TOKEN — stored in Secrets Manager, never a literal (R4)"
  type        = string
  default     = null
  nullable    = true
  sensitive   = true
}

variable "vpc_id" {
  description = "VPC ID; defaults to the account's default VPC if empty"
  type        = string
  default     = ""
}

variable "alb_subnet_ids" {
  description = "Public subnet IDs for the internet-facing ALB; defaults to default VPC subnets"
  type        = list(string)
  default     = []
}

variable "task_subnet_ids" {
  description = "ECS task subnet IDs; defaults to alb_subnet_ids"
  type        = list(string)
  default     = []
}

variable "assign_public_ip" {
  description = "Assign task public IPs for outbound access; disable only with private-subnet NAT or endpoints"
  type        = bool
  default     = true
}

variable "cpu" {
  description = "ECS task CPU units (256 | 512 | 1024 | 2048 | 4096)"
  type        = number
  default     = 512
}

variable "memory" {
  description = "ECS task memory in MiB"
  type        = number
  default     = 1024
}

variable "desired_count" {
  description = "Number of ECS task instances to run"
  type        = number
  default     = 1
}

variable "acm_certificate_arn" {
  description = <<-EOT
    ACM certificate ARN for HTTPS termination on the ALB (R3). When set, the
    module creates an HTTPS:443 listener with TLS 1.3 preferred + an HTTP:80→443
    redirect. Public plaintext forwarding is intentionally unsupported.
  EOT
  type        = string

  # [GPT-5.6] Fail during planning instead of silently creating public HTTP.
  validation {
    condition     = can(regex("^arn:[^:]+:acm:[^:]+:[0-9]{12}:certificate/[0-9A-Za-z-]+$", var.acm_certificate_arn))
    error_message = "acm_certificate_arn must be a complete ACM certificate ARN."
  }
}

variable "public_hostname" {
  description = "Public DNS hostname covered by acm_certificate_arn"
  type        = string

  # [GPT-5.6] The ALB-generated amazonaws.com name cannot be covered by an
  # operator ACM certificate, so a real application hostname is mandatory.
  validation {
    condition     = can(regex("^[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?(\\.[a-z0-9]([a-z0-9-]{0,61}[a-z0-9])?)+$", var.public_hostname))
    error_message = "public_hostname must be a lowercase fully qualified DNS hostname without a scheme or trailing dot."
  }
}

variable "route53_zone_id" {
  description = "Optional Route 53 hosted zone ID for public_hostname; leave empty when DNS is managed elsewhere"
  type        = string
  default     = ""
}

variable "enable_deletion_protection" {
  description = "Enable ALB deletion protection (recommended for production)"
  type        = bool
  default     = false
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
