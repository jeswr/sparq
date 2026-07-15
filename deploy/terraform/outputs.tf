# sparq multi-cloud Terraform root — outputs (sq-sos84) [SONNET]

output "endpoint_url" {
  description = "Public HTTPS endpoint of the deployed sparq service"
  value = (
    var.target == "aws" ? (
      length(module.aws) > 0 ? module.aws[0].endpoint_url : ""
    ) : var.target == "azure" ? (
      length(module.azure) > 0 ? module.azure[0].endpoint_url : ""
    ) : (
      length(module.gcp) > 0 ? module.gcp[0].endpoint_url : ""
    )
  )
}

output "service_name" {
  description = "Name of the deployed service resource"
  value = (
    var.target == "aws" ? (
      length(module.aws) > 0 ? module.aws[0].service_name : ""
    ) : var.target == "azure" ? (
      length(module.azure) > 0 ? module.azure[0].service_name : ""
    ) : (
      length(module.gcp) > 0 ? module.gcp[0].service_name : ""
    )
  )
}

output "secret_id" {
  description = "ID/ARN/name of the secret in the target cloud's secret store (R4)"
  value = (
    var.target == "aws" ? (
      length(module.aws) > 0 ? module.aws[0].secret_id : ""
    ) : var.target == "azure" ? (
      length(module.azure) > 0 ? module.azure[0].secret_id : ""
    ) : (
      length(module.gcp) > 0 ? module.gcp[0].secret_id : ""
    )
  )
  sensitive = true
}
