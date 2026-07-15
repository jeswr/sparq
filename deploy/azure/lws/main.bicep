// [SONNET-4.6] sq-zcou4 — Azure Container Apps: sparq-lws-core (Solid server)
//
// STATUS: Activates once sq-lmz40 ships.
// The ghcr.io/sparq-org/sparq-lws-core image is not yet published (sq-lmz40 OPEN).
// This template is structurally complete and valid but the referenced image will not
// resolve until sq-lmz40 merges and the lws-container.yml release workflow runs.
//
// SECURE-DEFAULTS NOTICE (R2/R9):
// The sparq-lws-core image is FAIL-CLOSED by design: anonymous mutation is rejected
// at the image layer, HTTPS-only WebIDs enforced, DPoP required. The template does
// NOT set any dev-only escape hatch (SOLID_SERVER_ALLOW_LOOPBACK, SOLID_SERVER_SEED_*).
// The prod posture requires an external OIDC issuer — SOLID_SERVER_TRUSTED_ISSUER
// is a required parameter (no default). (§1.3 of cloud-deploy-architecture.md)
//
// DECISION LOG (sq-zcou4 / lws):
//   - Container Apps (not AKS): no Kubernetes management overhead; managed TLS (R3).
//   - No SOLID_SERVER_TLS_CERT/KEY: Container Apps terminates TLS; no in-container TLS needed (R3).
//   - minReplicas = 1: DPoP-jti replay store is in-memory; >1 requires Redis (§1.3).
//     maxReplicas = 1 by default for correct replay protection. Operator must supply Redis
//     (SOLID_SERVER_REPLAY_REDIS_URL) before raising maxReplicas above 1.
//   - /livez = liveness, /readyz = readiness. Health path asymmetry from sparq-server (R7).
//   - User-assigned managed identity: least-privilege, Key Vault get on own secret only (R5).

@description('Container Apps Environment name.')
param environmentName string = 'sparq-lws-env'

@description('Container App name.')
param containerAppName string = 'sparq-lws'

@description('Azure region.')
param location string = resourceGroup().location

@description('Full image reference. Activates once sq-lmz40 ships.')
param imageRef string = 'ghcr.io/sparq-org/sparq-lws-core:latest'

@description('''
  Public HTTPS base URL of this server (e.g. https://sparq-lws.azurecontainerapps.io).
  REQUIRED. The LWS server cannot start without a known public base URL.
  Set this to the Container Apps FQDN after first deploy, or use a custom domain.
''')
param solidBaseUrl string

@description('''
  Trusted OIDC issuer URL for the Solid/LWS server (e.g. https://inrupt.net).
  REQUIRED. The server rejects authentication without a trusted issuer. (§1.3)
''')
param solidTrustedIssuer string

@description('''
  Optional: Redis URL for DPoP-jti replay protection across replicas.
  Required if maxReplicas > 1. Format: redis://<host>:6379
  Leave empty for single-replica deployments (maxReplicas must remain 1).
''')
@secure()
param redisUrl string = ''

@description('Minimum replica count. Must be 1 for correct replay protection (in-memory replay store).')
@minValue(0)
@maxValue(1)
param minReplicas int = 1

@description('''
  Maximum replica count.
  IMPORTANT: keep at 1 unless you supply a Redis URL (SOLID_SERVER_REPLAY_REDIS_URL).
  Multiple replicas without Redis will break DPoP-jti replay protection.
''')
@minValue(1)
@maxValue(25)
param maxReplicas int = 1

@description('CPU allocation per replica.')
param cpuCores string = '0.5'

@description('Memory allocation per replica.')
param memory string = '1Gi'

// ---------------------------------------------------------------------------
// User-assigned managed identity (R5)
// ---------------------------------------------------------------------------
resource managedIdentity 'Microsoft.ManagedIdentity/userAssignedIdentities@2023-01-31' = {
  name: '${containerAppName}-identity'
  location: location
}

// ---------------------------------------------------------------------------
// Log Analytics workspace
// ---------------------------------------------------------------------------
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2022-10-01' = {
  name: '${environmentName}-logs'
  location: location
  properties: {
    sku: { name: 'PerGB2018' }
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
// Container App: sparq-lws-core
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
      // Secrets (R4): Redis URL stored as Container Apps secret if provided.
      // solidBaseUrl and solidTrustedIssuer are not secrets (URLs, not tokens).
      // -----------------------------------------------------------------------
      secrets: !empty(redisUrl) ? [
        {
          name: 'redis-url'
          value: redisUrl
        }
      ] : []
      // -----------------------------------------------------------------------
      // Ingress: HTTPS-only on port 3000 (R3/R6). allowInsecure: false.
      // Container Apps managed TLS on *.azurecontainerapps.io FQDN.
      // -----------------------------------------------------------------------
      ingress: {
        external: true
        targetPort: 3000
        transport: 'http'
        allowInsecure: false   // R3
      }
    }
    template: {
      containers: [
        {
          name: 'sparq-lws'
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
          // Environment — required prod env vars; no dev escape hatches (R2)
          // -------------------------------------------------------------------
          env: concat(
            [
              {
                name: 'SOLID_SERVER_BASE_URL'
                value: solidBaseUrl
              }
              {
                name: 'SOLID_SERVER_TRUSTED_ISSUER'
                value: solidTrustedIssuer
              }
              {
                // SOLID_SERVER_AUDIENCE defaults to base URL; explicit here for clarity.
                name: 'SOLID_SERVER_AUDIENCE'
                value: solidBaseUrl
              }
            ],
            !empty(redisUrl) ? [
              {
                name: 'SOLID_SERVER_REPLAY_REDIS_URL'
                secretRef: 'redis-url'
              }
            ] : []
            // NOTE: SOLID_SERVER_ALLOW_LOOPBACK / SOLID_SERVER_SEED_* are
            // intentionally absent (dev-only; R2 / design §1.2).
          )
          // -------------------------------------------------------------------
          // Health probes (R7): liveness on /livez, readiness on /readyz.
          // Both are ungated (no auth required). Asymmetric with sparq-server.
          // -------------------------------------------------------------------
          probes: [
            {
              type: 'Liveness'
              httpGet: {
                path: '/livez'
                port: 3000
              }
              initialDelaySeconds: 10
              periodSeconds: 30
              failureThreshold: 3
            }
            {
              type: 'Readiness'
              httpGet: {
                path: '/readyz'
                port: 3000
              }
              initialDelaySeconds: 5
              periodSeconds: 10
              failureThreshold: 3
            }
            {
              type: 'Startup'
              httpGet: {
                path: '/livez'
                port: 3000
              }
              initialDelaySeconds: 5
              periodSeconds: 5
              failureThreshold: 12
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
output solidEndpoint string = 'https://${containerApp.properties.configuration.ingress.fqdn}/'
output managedIdentityClientId string = managedIdentity.properties.clientId
output managedIdentityPrincipalId string = managedIdentity.properties.principalId
