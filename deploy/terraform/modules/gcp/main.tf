# sparq Terraform — GCP submodule (sq-sos84) [SONNET]
#
# Provisions: Cloud Run service (fully managed), Secret Manager secret
# (auth token, R4), dedicated runtime service account (R5), IAM bindings.
#
# Secure defaults enforced per research/cloud-deploy-architecture.md §2:
# R1: SPARQ_AUTH_TOKEN from Secret Manager; Cloud Run secret env mount.
# R2: unauthenticated writes blocked; LWS dev escape hatches absent.
# R3: Cloud Run provides automatic HTTPS on run.app URL.
# R4: token in Secret Manager, sensitive variable, never a literal.
# R5: dedicated service account with secretmanager.secretAccessor on own secret only.
# R6: --no-allow-unauthenticated controls Cloud Run IAM front door (optional);
#     app-level auth is the Bearer token (R1).
# R7: startup + liveness probes on var.health_path.
# R8: non-root not overridden (image default); container sandbox is Cloud Run default.
# R9: sparq-server open-by-default at image layer — token wiring is mandatory.

terraform {
  required_providers {
    google = {
      source  = "hashicorp/google"
      version = ">= 5.0.0, < 7.0.0"
    }
  }
}

# ---------------------------------------------------------------------------
# Enable required APIs
# ---------------------------------------------------------------------------

resource "google_project_service" "run" {
  service            = "run.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "secretmanager" {
  count = var.server == "sparq-server" ? 1 : 0

  service            = "secretmanager.googleapis.com"
  disable_on_destroy = false
}

resource "google_project_service" "iam" {
  service            = "iam.googleapis.com"
  disable_on_destroy = false
}

# ---------------------------------------------------------------------------
# Dedicated runtime service account (R5)
# ---------------------------------------------------------------------------

resource "google_service_account" "sparq" {
  account_id   = "${substr(var.name, 0, 28)}-sa"
  display_name = "sparq ${var.server} runtime service account"
  # [GPT-5.6] LWS has no token secret, so do not claim or grant secret access.
  description = var.server == "sparq-server" ? (
    "Least-privilege SA: secretmanager.secretAccessor on own secret only (R5)"
  ) : "Dedicated sparq LWS runtime service account with no project roles (R5)"

  depends_on = [google_project_service.iam]
}

# ---------------------------------------------------------------------------
# Secret Manager — auth token (R4)
# ---------------------------------------------------------------------------

resource "google_secret_manager_secret" "auth_token" {
  count = var.server == "sparq-server" ? 1 : 0

  secret_id = "${var.name}-auth-token"

  replication {
    auto {}
  }

  labels = {
    managed-by = "sparq-terraform"
    server     = var.server
  }

  depends_on = [google_project_service.secretmanager]
}

resource "google_secret_manager_secret_version" "auth_token" {
  count = var.server == "sparq-server" ? 1 : 0

  secret      = google_secret_manager_secret.auth_token[0].id
  secret_data = var.auth_token

  lifecycle {
    # [GPT-5.6] Direct child-module callers cannot create an open server with
    # a missing or trivially weak token.
    precondition {
      condition     = try(length(var.auth_token) >= 32, false)
      error_message = "sparq-server requires auth_token with at least 32 characters."
    }
  }
}

# Grant service account access to its own secret only (R5)
resource "google_secret_manager_secret_iam_member" "sparq_sa_accessor" {
  count = var.server == "sparq-server" ? 1 : 0

  project   = var.gcp_project
  secret_id = google_secret_manager_secret.auth_token[0].secret_id
  role      = "roles/secretmanager.secretAccessor"
  member    = "serviceAccount:${google_service_account.sparq.email}"
  # Scoped to own secret ARN only — no project-level wildcard (R5)
}

# ---------------------------------------------------------------------------
# Cloud Run service (R1–R8)
# ---------------------------------------------------------------------------

locals {
  # Server-specific env vars (R2: dev escape hatches never appear here)
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
}

resource "google_cloud_run_v2_service" "sparq" {
  name     = var.name
  location = var.gcp_region
  ingress  = "INGRESS_TRAFFIC_ALL"

  template {
    service_account = google_service_account.sparq.email

    scaling {
      min_instance_count = var.min_instances
      max_instance_count = var.max_instances
    }

    containers {
      image = var.image

      ports {
        container_port = var.container_port
      }

      resources {
        limits = {
          cpu    = var.cpu
          memory = var.memory
        }
        # [GPT-5.6] Preserve Cloud Run request-based CPU allocation when a
        # resource-limits block is present.
        cpu_idle = true
      }

      # Server-specific env (R2: no dev escape hatches)
      dynamic "env" {
        for_each = local.server_env
        content {
          name  = env.value.name
          value = env.value.value
        }
      }

      # SPARQ_AUTH_TOKEN from Secret Manager (R1/R4; sparq-server only)
      dynamic "env" {
        for_each = var.server == "sparq-server" ? [1] : []
        content {
          name = "SPARQ_AUTH_TOKEN"
          value_source {
            secret_key_ref {
              # [GPT-5.6] Secret Manager is the only source of token bytes.
              secret  = google_secret_manager_secret.auth_token[0].secret_id
              version = "latest"
            }
          }
        }
      }

      # Startup probe — allow extra time on cold start (R7)
      startup_probe {
        http_get {
          path = var.health_path
          port = var.container_port
        }
        initial_delay_seconds = 10
        timeout_seconds       = 5
        period_seconds        = 10
        failure_threshold     = 10
      }

      # Liveness probe (R7)
      liveness_probe {
        http_get {
          path = var.server == "lws" ? "/livez" : var.health_path
          port = var.container_port
        }
        initial_delay_seconds = 30
        timeout_seconds       = 5
        period_seconds        = 30
        failure_threshold     = 3
      }
    }
  }

  # HTTPS is automatic on run.app domain (R3)

  depends_on = [
    google_project_service.run,
    google_secret_manager_secret_iam_member.sparq_sa_accessor,
  ]

  lifecycle {
    precondition {
      condition = var.server != "lws" || (
        startswith(var.solid_server_base_url, "https://") &&
        startswith(var.solid_server_trusted_issuer, "https://")
      )
      error_message = "lws requires HTTPS solid_server_base_url and solid_server_trusted_issuer values."
    }
    precondition {
      condition     = var.min_instances <= var.max_instances
      error_message = "min_instances must not exceed max_instances."
    }
    precondition {
      condition     = var.server != "lws" || (var.min_instances == 1 && var.max_instances == 1)
      error_message = "lws must use exactly one instance unless shared Redis replay protection is wired."
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

# ---------------------------------------------------------------------------
# Cloud Run IAM — unauthenticated access control (R6)
#
# DECISION: by default we allow unauthenticated invocations so the sparq-server
# endpoint is publicly reachable; the app-level Bearer token (R1) is the access
# control mechanism. Set var.allow_unauthenticated = false to add Cloud Run IAM
# as a second layer (suitable for internal/private endpoints).
# ---------------------------------------------------------------------------

resource "google_cloud_run_v2_service_iam_member" "public" {
  count    = var.allow_unauthenticated ? 1 : 0
  project  = var.gcp_project
  location = var.gcp_region
  name     = google_cloud_run_v2_service.sparq.name
  role     = "roles/run.invoker"
  member   = "allUsers"
}
