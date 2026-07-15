# sparq Terraform — AWS submodule outputs (sq-sos84) [SONNET]

output "endpoint_url" {
  description = "Public HTTPS endpoint covered by the ACM certificate (R3)"
  value       = "https://${var.public_hostname}"
}

output "load_balancer_dns_name" {
  description = "Generated ALB DNS target for external DNS providers"
  value       = aws_lb.sparq.dns_name
}

output "service_name" {
  description = "ECS service name"
  value       = aws_ecs_service.sparq.name
}

output "secret_id" {
  description = "Secrets Manager ARN for the auth token (R4)"
  # [GPT-5.6] LWS does not create an unused bearer-token secret.
  value     = var.server == "sparq-server" ? aws_secretsmanager_secret.auth_token[0].arn : null
  sensitive = true
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
