# sparq Terraform — GCP submodule outputs (sq-sos84) [SONNET]

output "endpoint_url" {
  description = "Cloud Run service URL (automatic HTTPS on run.app domain, R3)"
  value       = google_cloud_run_v2_service.sparq.uri
}

output "service_name" {
  description = "Cloud Run service name"
  value       = google_cloud_run_v2_service.sparq.name
}

output "secret_id" {
  description = "Secret Manager secret ID for the auth token (R4)"
  value       = google_secret_manager_secret.auth_token.id
  sensitive   = true
}

output "service_account_email" {
  description = "Runtime service account email (R5)"
  value       = google_service_account.sparq.email
}
