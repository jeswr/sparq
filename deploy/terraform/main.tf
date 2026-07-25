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
#   terraform apply -var target=azure
#
# terraform validate is the credential-free CI check; plan/apply require provider credentials.

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

# [GPT-5.6] Provider configurations belong in the root module. Keeping them out
# of counted child modules avoids Terraform's legacy-module apply failure.
provider "aws" {
  region = var.aws_region
}

provider "azurerm" {
  features {
    key_vault {
      purge_soft_delete_on_destroy    = false
      recover_soft_deleted_key_vaults = true
    }
  }
}

provider "google" {
  project = var.gcp_project != "" ? var.gcp_project : null
  region  = var.gcp_region
}

provider "random" {}

# ---------------------------------------------------------------------------
# Root-level local resolution
# ---------------------------------------------------------------------------

locals {
  # Canonical image refs per server (§1 of the design record)
  image_defaults = {
    sparq-server = "ghcr.io/sparq-org/sparq-server"
    lws          = "ghcr.io/sparq-org/sparq-lws-core"
  }

  image_ref  = var.image_override != "" ? var.image_override : local.image_defaults[var.server]
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

  # [GPT-5.6] A single literal default cannot be valid across ECS, Container
  # Apps, and Cloud Run. Resolve provider-native defaults before delegation.
  cpu_defaults = {
    aws   = "512"
    azure = "0.5"
    gcp   = "1"
  }
  memory_defaults = {
    aws   = "1024"
    azure = "1.0Gi"
    gcp   = "512Mi"
  }
  cpu    = var.cpu != "" ? var.cpu : local.cpu_defaults[var.target]
  memory = var.memory != "" ? var.memory : local.memory_defaults[var.target]

  # [GPT-5.6] Both images require one managed replica: sparq-server stores an
  # uncoordinated in-memory dataset, while LWS needs Redis-backed DPoP replay
  # protection before it can scale safely. These templates wire neither store.
  azure_min_replicas = 1
  azure_max_replicas = 1
  gcp_min_instances  = 1
  gcp_max_instances  = 1
}

# [GPT-5.6] Prefer generating the bearer token at deploy time. The container
# receives it only through the selected cloud's secret-store reference.
resource "random_password" "auth_token" {
  # [GPT-5.6] Count depends only on the non-sensitive server selector; Terraform
  # must not derive resource addresses from whether a sensitive override is set.
  count = var.server == "sparq-server" ? 1 : 0

  length  = 48
  special = false
}

locals {
  auth_token = var.server == "sparq-server" ? (
    var.auth_token != null ? var.auth_token : random_password.auth_token[0].result
  ) : null
}

# ---------------------------------------------------------------------------
# AWS submodule
# ---------------------------------------------------------------------------

module "aws" {
  count  = var.target == "aws" ? 1 : 0
  source = "./modules/aws"

  providers = {
    aws = aws
  }

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = local.auth_token

  cpu    = local.cpu
  memory = local.memory

  acm_certificate_arn = var.aws_acm_certificate_arn
  public_hostname     = var.aws_public_hostname
  route53_zone_id     = var.aws_route53_zone_id
  vpc_id              = var.aws_vpc_id
  alb_subnet_ids      = var.aws_alb_subnet_ids
  task_subnet_ids     = var.aws_task_subnet_ids
  assign_public_ip    = var.aws_assign_public_ip

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

  providers = {
    azurerm = azurerm
    random  = random
  }

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = local.auth_token

  azure_location = var.azure_location
  azure_rg_name  = var.azure_rg_name
  cpu            = local.cpu
  memory         = local.memory
  min_replicas   = local.azure_min_replicas
  max_replicas   = local.azure_max_replicas

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

  providers = {
    google = google
  }

  name           = var.name
  server         = var.server
  image          = local.full_image
  container_port = local.container_port
  health_path    = local.health_path
  auth_token     = local.auth_token

  gcp_project   = var.gcp_project
  gcp_region    = var.gcp_region
  min_instances = local.gcp_min_instances
  max_instances = local.gcp_max_instances
  cpu           = local.cpu
  memory        = local.memory

  # LWS-only required params (ignored for sparq-server)
  solid_server_base_url       = var.solid_server_base_url
  solid_server_trusted_issuer = var.solid_server_trusted_issuer
}
