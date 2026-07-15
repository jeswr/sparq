# sparq multi-cloud Terraform root module (sq-sos84)
#
# [SONNET] This module provisions either sparq-server or sparq-lws-core from the
# published GHCR image into the target cloud (aws | azure | gcp) using the
# per-target submodule under ./modules/<target>/.
#
# SECURITY NOTE (R9): sparq-server is open-by-default at the image layer
# (bakes SPARQ_ALLOW_REMOTE=1, no auth). This module enforces auth ON at the
# template layer via auth_token (R1). Do NOT remove the token wiring.
#
# Usage:
#   terraform init
#   terraform apply -var target=aws -var auth_token=<secret>
#
# terraform plan is the CI dry-run; terraform apply requires provider credentials.

terraform {
  required_version = ">= 1.5.0"

  required_providers {
    aws = {
      source  = "hashicorp/aws"
      version = ">= 5.0.0, < 6.0.0"
    }
    azurerm = {
      source  = "hashicorp/azurerm"
      version = ">= 3.85.0, < 5.0.0"
    }
    google = {
      source  = "hashicorp/google"
      version = ">= 5.0.0, < 7.0.0"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.5.0"
    }
  }
}

# ---------------------------------------------------------------------------
# Root-level local resolution
# ---------------------------------------------------------------------------

locals {
  # Canonical image refs per server (§1 of the design record)
  image_defaults = {
    sparq-server = "ghcr.io/sparq-org/sparq-server"
    lws          = "ghcr.io/sparq-org/sparq-lws-core"
  }

  image_ref = var.image_override != "" ? var.image_override : local.image_defaults[var.server]
  full_image = "${local.image_ref}:${var.image_tag}"

  # Health-check path per server (R7 — parameterised, NOT shared constant)
  health_path_defaults = {
    sparq-server = "/health"
    lws          = "/readyz"
  }
  health_path = var.health_path_override != "" ? var.health_path_override : local.health_path_defaults[var.server]

  # Container port per server
  container_port_defaults = {
    sparq-server = 3030
    lws          = 3000
  }
  container_port = local.container_port_defaults[var.server]
}

# ---------------------------------------------------------------------------
# AWS submodule
# ---------------------------------------------------------------------------

module "aws" {
  count  = var.target == "aws" ? 1 : 0
  source = "./modules/aws"

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = var.auth_token

  aws_region = var.aws_region
  cpu        = var.cpu
  memory     = var.memory

  # LWS-only required params (ignored for sparq-server)
  solid_server_base_url       = var.solid_server_base_url
  solid_server_trusted_issuer = var.solid_server_trusted_issuer
}

# ---------------------------------------------------------------------------
# Azure submodule
# ---------------------------------------------------------------------------

module "azure" {
  count  = var.target == "azure" ? 1 : 0
  source = "./modules/azure"

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = var.auth_token

  azure_location      = var.azure_location
  azure_rg_name       = var.azure_rg_name
  cpu                 = var.cpu
  memory              = var.memory
  min_replicas        = var.min_replicas
  max_replicas        = var.max_replicas

  # LWS-only required params (ignored for sparq-server)
  solid_server_base_url       = var.solid_server_base_url
  solid_server_trusted_issuer = var.solid_server_trusted_issuer
}

# ---------------------------------------------------------------------------
# GCP submodule
# ---------------------------------------------------------------------------

module "gcp" {
  count  = var.target == "gcp" ? 1 : 0
  source = "./modules/gcp"

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = var.auth_token

  gcp_project  = var.gcp_project
  gcp_region   = var.gcp_region
  min_instances = var.min_instances
  max_instances = var.max_instances
  cpu           = var.cpu
  memory        = var.memory

  # LWS-only required params (ignored for sparq-server)
  solid_server_base_url       = var.solid_server_base_url
  solid_server_trusted_issuer = var.solid_server_trusted_issuer
}
