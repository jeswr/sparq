# sparq Terraform — Azure submodule (sq-sos84) [SONNET]
#
# Provisions: Container Apps Environment + Container App (image from §1),
# Key Vault + secret (auth token, R4), user-assigned managed identity (R5),
# Log Analytics workspace, resource group.
#
# Secure defaults enforced per research/cloud-deploy-architecture.md §2:
# R1: SPARQ_AUTH_TOKEN injected from Key Vault via managed identity + secretRef.
# R2: unauthenticated writes blocked; LWS dev escape hatches absent.
# R3: Container Apps automatic HTTPS on managed FQDN; ingress on app port only.
# R4: token stored in Key Vault, sensitive variable, never a literal in config.
# R5: user-assigned managed identity; Key Vault access policy on own secret only.
# R6: external ingress on targetPort only; internal traffic restricted.
# R7: liveness + readiness probes wired to var.health_path (and /livez for lws).
# R8: non-root not overridden; read-only rootfs where Container Apps allows.
# R9: sparq-server open-by-default at image layer — token wiring is mandatory here.

terraform {
  required_providers {
    azurerm = {
      source  = "hashicorp/azurerm"
      version = ">= 3.85.0, < 5.0.0"
    }
    random = {
      source  = "hashicorp/random"
      version = ">= 3.5.0"
    }
  }
}

data "azurerm_client_config" "current" {}

# ---------------------------------------------------------------------------
# Resource group
# ---------------------------------------------------------------------------

resource "azurerm_resource_group" "sparq" {
  name     = var.azure_rg_name
  location = var.azure_location

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }
}

# ---------------------------------------------------------------------------
# User-assigned managed identity (R5)
# ---------------------------------------------------------------------------

resource "azurerm_user_assigned_identity" "sparq" {
  name                = "${var.name}-identity"
  resource_group_name = azurerm_resource_group.sparq.name
  location            = azurerm_resource_group.sparq.location

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# Key Vault + secret (auth token, R4)
# ---------------------------------------------------------------------------

# Key Vault name must be globally unique and <=24 chars
resource "random_string" "kv_suffix" {
  count = var.server == "sparq-server" ? 1 : 0

  length  = 6
  special = false
  upper   = false
}

resource "azurerm_key_vault" "sparq" {
  count = var.server == "sparq-server" ? 1 : 0

  name                       = "${substr(var.name, 0, 17)}-${random_string.kv_suffix[0].result}"
  location                   = azurerm_resource_group.sparq.location
  resource_group_name        = azurerm_resource_group.sparq.name
  tenant_id                  = data.azurerm_client_config.current.tenant_id
  sku_name                   = "standard"
  soft_delete_retention_days = 7
  purge_protection_enabled   = true

  # Current Terraform executor can manage secrets
  access_policy {
    tenant_id = data.azurerm_client_config.current.tenant_id
    object_id = data.azurerm_client_config.current.object_id

    secret_permissions = [
      "Get",
      "List",
      "Set",
      "Delete",
    ]
  }

  # Managed identity: get own secret only (R5)
  access_policy {
    tenant_id = data.azurerm_client_config.current.tenant_id
    object_id = azurerm_user_assigned_identity.sparq.principal_id

    secret_permissions = ["Get"]
  }

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

resource "azurerm_key_vault_secret" "auth_token" {
  count = var.server == "sparq-server" ? 1 : 0

  name         = "sparq-auth-token"
  value        = var.auth_token
  key_vault_id = azurerm_key_vault.sparq[0].id
  content_type = "sparq-server SPARQ_AUTH_TOKEN — do not remove (R1/R4)"

  tags = {
    ManagedBy = "sparq-terraform"
  }

  lifecycle {
    # [GPT-5.6] Direct child-module callers cannot create an open server with
    # a missing or trivially weak token.
    precondition {
      condition     = try(length(var.auth_token) >= 32, false)
      error_message = "sparq-server requires auth_token with at least 32 characters."
    }
  }
}

# ---------------------------------------------------------------------------
# Log Analytics workspace
# ---------------------------------------------------------------------------

resource "azurerm_log_analytics_workspace" "sparq" {
  name                = "${var.name}-logs"
  location            = azurerm_resource_group.sparq.location
  resource_group_name = azurerm_resource_group.sparq.name
  sku                 = "PerGB2018"
  retention_in_days   = 30

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# Container Apps Environment
# ---------------------------------------------------------------------------

resource "azurerm_container_app_environment" "sparq" {
  name                       = "${var.name}-env"
  location                   = azurerm_resource_group.sparq.location
  resource_group_name        = azurerm_resource_group.sparq.name
  log_analytics_workspace_id = azurerm_log_analytics_workspace.sparq.id

  tags = {
    ManagedBy = "sparq-terraform"
  }
}

# ---------------------------------------------------------------------------
# Container App (R1–R8)
# ---------------------------------------------------------------------------

locals {
  # Environment variables: common + server-specific
  # R2: dev escape hatches NEVER appear here
  sparq_server_env = [
    {
      name  = "SPARQ_ALLOW_REMOTE"
      value = "1"
    },
    {
      # [GPT-5.6] Gate reads as well as writes; probes remain ungated.
      name  = "SPARQ_AUTH_TOKEN_READ"
      value = "1"
    }
  ]

  lws_env = [
    {
      name  = "SOLID_SERVER_BASE_URL"
      value = var.solid_server_base_url
    },
    {
      name  = "SOLID_SERVER_TRUSTED_ISSUER"
      value = var.solid_server_trusted_issuer
    }
  ]

  server_env = var.server == "sparq-server" ? local.sparq_server_env : local.lws_env

  # Secret ref name used in Container App env (points to Key Vault secret)
  secret_ref_name = "sparq-auth-token"
}

resource "azurerm_container_app" "sparq" {
  name                         = "${var.name}-app"
  container_app_environment_id = azurerm_container_app_environment.sparq.id
  resource_group_name          = azurerm_resource_group.sparq.name
  revision_mode                = "Single"

  identity {
    type         = "UserAssigned"
    identity_ids = [azurerm_user_assigned_identity.sparq.id]
  }

  # Auth token from Key Vault via managed identity (R4); LWS creates no unused
  # secret or data-plane permission.
  dynamic "secret" {
    for_each = var.server == "sparq-server" ? [1] : []
    content {
      name                = local.secret_ref_name
      key_vault_secret_id = azurerm_key_vault_secret.auth_token[0].versionless_id
      identity            = azurerm_user_assigned_identity.sparq.id
    }
  }

  template {
    min_replicas = var.min_replicas
    max_replicas = var.max_replicas

    container {
      name   = var.server
      image  = var.image
      cpu    = tonumber(var.cpu)
      memory = var.memory

      # Server-specific env (R2: no dev escape hatches)
      dynamic "env" {
        for_each = local.server_env
        content {
          name  = env.value.name
          value = env.value.value
        }
      }

      # SPARQ_AUTH_TOKEN injected from Key Vault secret (R1, sparq-server only)
      dynamic "env" {
        for_each = var.server == "sparq-server" ? [1] : []
        content {
          name        = "SPARQ_AUTH_TOKEN"
          secret_name = local.secret_ref_name
        }
      }

      # Readiness probe — /readyz for lws, /health for sparq-server (R7)
      readiness_probe {
        transport = "HTTP"
        path      = var.health_path
        port      = var.container_port
      }

      # Liveness probe — /livez for lws (process up), /health for sparq-server (R7)
      liveness_probe {
        transport = "HTTP"
        path      = var.server == "lws" ? "/livez" : var.health_path
        port      = var.container_port
      }

      startup_probe {
        transport               = "HTTP"
        path                    = var.health_path
        port                    = var.container_port
        failure_count_threshold = 10
        # [GPT-5.6] AzureRM names this field interval_seconds, not the
        # Kubernetes/Cloud Run period_seconds spelling.
        interval_seconds = 10
      }
    }
  }

  # Automatic HTTPS on Container Apps FQDN (R3); ingress on app port only (R6)
  ingress {
    external_enabled           = true
    target_port                = var.container_port
    transport                  = "http"
    allow_insecure_connections = false

    traffic_weight {
      percentage      = 100
      latest_revision = true
    }
  }

  tags = {
    ManagedBy = "sparq-terraform"
    Server    = var.server
  }

  lifecycle {
    precondition {
      condition = var.server != "lws" || (
        startswith(var.solid_server_base_url, "https://") &&
        startswith(var.solid_server_trusted_issuer, "https://")
      )
      error_message = "lws requires HTTPS solid_server_base_url and solid_server_trusted_issuer values."
    }
    precondition {
      condition     = var.min_replicas <= var.max_replicas
      error_message = "min_replicas must not exceed max_replicas."
    }
    precondition {
      condition     = var.server != "lws" || (var.min_replicas == 1 && var.max_replicas == 1)
      error_message = "lws must use exactly one replica unless shared Redis replay protection is wired."
    }
    # [GPT-5.6] Replicas would hold divergent process-local datasets.
    precondition {
      condition     = var.server != "sparq-server" || (var.min_replicas == 1 && var.max_replicas == 1)
      error_message = "sparq-server must use exactly one replica unless a shared persistent backing store is wired."
    }
    precondition {
      condition = (
        var.server == "sparq-server" && var.container_port == 3030
        ) || (
        var.server == "lws" && var.container_port == 3000
      )
      error_message = "container_port must be 3030 for sparq-server or 3000 for lws."
    }
  }
}
