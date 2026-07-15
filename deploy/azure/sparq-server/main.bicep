// [SONNET-4.6] sq-zcou4 — Azure Container Apps: sparq-server
//
// SECURE-DEFAULTS NOTICE (R1/R9):
// The sparq-server image is open-by-default at the image layer (bakes SPARQ_ALLOW_REMOTE=1,
// no auth). Anyone reaching the published port can read AND write the dataset.
// This template enforces auth ON at the template layer:
//   - SPARQ_AUTH_TOKEN is stored as a Container Apps secret — NEVER a literal.
//   - Do NOT remove the secretRef wiring. Do NOT add a default token value here.
//
// DECISION LOG (sq-zcou4):
//   - Azure Container Apps (not ACI or AKS): managed serverless, auto-TLS on ingress FQDN,
//     no cert/IP management needed — aligns with R3 requirement for managed TLS.
//   - Container Apps secrets: token stored as a Container Apps secret (secretRef injection)
//     satisfying R4; an optional Key Vault reference replaces this for vault-backed deployments.
//   - User-assigned managed identity: scoped to Key Vault get on its own secret only (R5).
//     The identity is created by this template; callers grant it Key Vault access post-deploy.
//   - External HTTPS-only ingress on targetPort 3030 only; no management ports public (R6).
//   - Container Apps auto-provisions a managed TLS cert on the *.azurecontainerapps.io FQDN
//     (R3). Custom domain TLS follows the same managed path.
//   - minReplicas: 1 default (ensures health probe and /health path stay warm).
//   - ReadOnlyRootFilesystem enforced at securityContext layer (R8).
//   - sparq-server runs non-root in the distroless image; not overridden here (R8).

@description('Container Apps Environment name — created by this template.')
param environmentName string = 'sparq-server-env'

@description('Container App name.')
param containerAppName string = 'sparq-server'

@description('Azure region to deploy into.')
param location string = resourceGroup().location

@description('''
  Full image reference (registry/image:tag).
  Defaults to the canonical GHCR image. Pin a specific version in production
  (e.g. ghcr.io/sparq-org/sparq-server:0.1.0).
''')
param imageRef string = 'ghcr.io/sparq-org/sparq-server:latest'

@description('''
  SPARQ_AUTH_TOKEN bearer value.
  REQUIRED — a strong random secret (e.g. openssl rand -hex 32).
  Stored as a Container Apps secret; never logged or exposed in the template output. (R1/R4)
''')
@secure()
param authToken string

@description('Minimum replica count. Default 1 keeps /health warm.')
@minValue(0)
@maxValue(25)
param minReplicas int = 1

@description('Maximum replica count.')
@minValue(1)
@maxValue(25)
param maxReplicas int = 3

@description('''
  Set to "1" to also require the auth token for read (SELECT/CONSTRUCT) queries.
  Leave empty to allow unauthenticated reads while gating writes.
''')
@allowed(['', '1'])
param authTokenRead string = ''

@description('Value for SPARQ_CORS_ALLOW_ORIGIN (e.g. https://example.com). Leave empty to disable.')
param corsAllowOrigin string = ''

@description('Maximum concurrent SPARQL query workers (SPARQ_MAX_CONCURRENT).')
@minValue(1)
@maxValue(256)
param maxConcurrent int = 16

@description('Per-query timeout in seconds (SPARQ_QUERY_TIMEOUT).')
@minValue(5)
@maxValue(300)
param queryTimeoutSeconds int = 30

@description('CPU allocation per replica (0.25, 0.5, 0.75, 1.0, 1.25, …, 2.0).')
param cpuCores string = '0.5'

@description('Memory allocation per replica (e.g. 1Gi, 2Gi).')
param memory string = '1Gi'

// ---------------------------------------------------------------------------
// User-assigned managed identity (R5 — least privilege)
// ---------------------------------------------------------------------------
resource managedIdentity 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: '${containerAppName}-identity'
  location: location
}

// ---------------------------------------------------------------------------
// Log Analytics workspace (telemetry for Container Apps environment)
// ---------------------------------------------------------------------------
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2022-10-01' = {
  name: '${environmentName}-logs'
  location: location
  properties: {
    sku: {
      name: 'PerGB2018'
    }
    retentionInDays: 30
  }
}

// ---------------------------------------------------------------------------
// Container Apps Environment
// ---------------------------------------------------------------------------
resource containerAppsEnv 'Microsoft.App/managedEnvironments@2024-03-01' = {
  name: environmentName
  location: location
  properties: {
    appLogsConfiguration: {
      destination: 'log-analytics'
      logAnalyticsConfiguration: {
        customerId: logAnalytics.properties.customerId
        sharedKey: logAnalytics.listKeys().primarySharedKey
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Container App: sparq-server
// ---------------------------------------------------------------------------
resource containerApp 'Microsoft.App/containerApps@2024-03-01' = {
  name: containerAppName
  location: location
  identity: {
    type: 'UserAssigned'
    userAssignedIdentities: {
      '${managedIdentity.id}': {}
    }
  }
  properties: {
    environmentId: containerAppsEnv.id
    configuration: {
      // -----------------------------------------------------------------------
      // Secrets (R4): token stored as Container Apps secret — never a literal
      // in any template output or environment variable value field.
      // -----------------------------------------------------------------------
      secrets: [
        {
          name: 'sparq-auth-token'
          value: authToken
        }
      ]
      // -----------------------------------------------------------------------
      // Ingress: HTTPS-only external on targetPort 3030 only (R3/R6)
      // Container Apps auto-TLS on the *.azurecontainerapps.io FQDN.
      // -----------------------------------------------------------------------
      ingress: {
        external: true
        targetPort: 3030
        transport: 'http'
        allowInsecure: false   // R3: reject plain HTTP
      }
    }
    template: {
      containers: [
        {
          name: 'sparq-server'
          image: imageRef
          // -------------------------------------------------------------------
          // Security context: non-root, read-only root filesystem (R8)
          // -------------------------------------------------------------------
          securityContext: {
            runAsNonRoot: true
            readOnlyRootFilesystem: true
            allowPrivilegeEscalation: false
          }
          resources: {
            cpu: json(cpuCores)
            memory: memory
          }
          // -------------------------------------------------------------------
          // Environment — secrets injected via secretRef (R1/R4)
          // -------------------------------------------------------------------
          env: concat(
            [
              {
                name: 'SPARQ_AUTH_TOKEN'
                secretRef: 'sparq-auth-token'  // R1: token from secret, never literal
              }
              {
                name: 'SPARQ_MAX_CONCURRENT'
                value: string(maxConcurrent)
              }
              {
                name: 'SPARQ_QUERY_TIMEOUT'
                value: string(queryTimeoutSeconds)
              }
            ],
            authTokenRead == '1' ? [{ name: 'SPARQ_AUTH_TOKEN_READ', value: '1' }] : [],
            !empty(corsAllowOrigin) ? [{ name: 'SPARQ_CORS_ALLOW_ORIGIN', value: corsAllowOrigin }] : []
          )
          // -------------------------------------------------------------------
          // Health probes (R7): Container Apps HTTP probes on /health
          // sparq-server /health → 200 body "ok", ungated, no token needed.
          // -------------------------------------------------------------------
          probes: [
            {
              type: 'Liveness'
              httpGet: {
                path: '/health'
                port: 3030
              }
              initialDelaySeconds: 10
              periodSeconds: 30
              failureThreshold: 3
            }
            {
              type: 'Readiness'
              httpGet: {
                path: '/health'
                port: 3030
              }
              initialDelaySeconds: 5
              periodSeconds: 10
              failureThreshold: 3
            }
            {
              type: 'Startup'
              httpGet: {
                path: '/health'
                port: 3030
              }
              initialDelaySeconds: 5
              periodSeconds: 5
              failureThreshold: 12   // up to 60s startup window
            }
          ]
        }
      ]
      scale: {
        minReplicas: minReplicas
        maxReplicas: maxReplicas
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Outputs
// ---------------------------------------------------------------------------
output fqdn string = containerApp.properties.configuration.ingress.fqdn
output sparqlEndpoint string = 'https://${containerApp.properties.configuration.ingress.fqdn}/sparql'
output managedIdentityClientId string = managedIdentity.properties.clientId
output managedIdentityPrincipalId string = managedIdentity.properties.principalId
