# sparq Terraform — Azure submodule outputs (sq-sos84) [SONNET]

output "endpoint_url" {
  description = "Container App HTTPS endpoint (automatic TLS from Container Apps, R3)"
  value       = "https://${azurerm_container_app.sparq.ingress[0].fqdn}"
}

output "service_name" {
  description = "Container App resource name"
  value       = azurerm_container_app.sparq.name
}

output "secret_id" {
  description = "Key Vault secret ID for the auth token (R4)"
  value       = azurerm_key_vault_secret.auth_token.id
  sensitive   = true
}

output "identity_principal_id" {
  description = "Managed identity principal ID (R5)"
  value       = azurerm_user_assigned_identity.sparq.principal_id
}

output "resource_group_name" {
  description = "Resource group name"
  value       = azurerm_resource_group.sparq.name
}
