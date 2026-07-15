# sparq Terraform — AWS submodule outputs (sq-sos84) [SONNET]

output "endpoint_url" {
  description = "ALB DNS name (HTTPS if ACM cert provided, HTTP otherwise)"
  value = var.acm_certificate_arn != "" ? (
    "https://${aws_lb.sparq.dns_name}"
  ) : "http://${aws_lb.sparq.dns_name}"
}

output "service_name" {
  description = "ECS service name"
  value       = aws_ecs_service.sparq.name
}

output "secret_id" {
  description = "Secrets Manager ARN for the auth token (R4)"
  value       = aws_secretsmanager_secret.auth_token.arn
  sensitive   = true
}

output "cluster_arn" {
  description = "ECS cluster ARN"
  value       = aws_ecs_cluster.sparq.arn
}

output "task_role_arn" {
  description = "IAM task role ARN (R5)"
  value       = aws_iam_role.task.arn
}

output "execution_role_arn" {
  description = "IAM execution role ARN (R5)"
  value       = aws_iam_role.execution.arn
}
