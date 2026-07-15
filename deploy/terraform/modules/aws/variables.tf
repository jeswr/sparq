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
  description = "SPARQ_AUTH_TOKEN — stored in Secrets Manager, never a literal (R4)"
  type        = string
  sensitive   = true
}

variable "aws_region" {
  description = "AWS region"
  type        = string
  default     = "us-east-1"
}

variable "vpc_id" {
  description = "VPC ID; defaults to the account's default VPC if empty"
  type        = string
  default     = ""
}

variable "subnet_ids" {
  description = "List of subnet IDs for the ALB and ECS tasks; defaults to default VPC subnets"
  type        = list(string)
  default     = []
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
    redirect. When empty, an HTTP-only listener is created (dev/internal only —
    not recommended for public endpoints bearing Bearer tokens).
  EOT
  type    = string
  default = ""
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
