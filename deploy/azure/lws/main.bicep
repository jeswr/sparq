// [SONNET-4.6] sq-zcou4 — Azure Container Apps: sparq-lws-core (Solid server)
// [GPT-5.6] Security/correctness review: deployable base URL, single-replica replay safety, valid ACA schema.
//
// STATUS: Activates once sq-lmz40 ships.
// The ghcr.io/sparq-org/sparq-lws-core image is not yet published (sq-lmz40 OPEN).
// This template is structurally complete and valid but the referenced image will not
// resolve until sq-lmz40 merges and the lws-container.yml release workflow runs.
//
// SECURE-DEFAULTS NOTICE (R2/R9):
// The sibling sparq-server image is open-by-default; its Azure template gates it with a Key Vault
// token. Do not remove that token wiring. This LWS image instead fails closed by design.
// The sparq-lws-core image is FAIL-CLOSED by design: anonymous mutation is rejected
// at the image layer, HTTPS-only WebIDs enforced, DPoP required. The template does
// NOT set any dev-only escape hatch (SOLID_SERVER_ALLOW_LOOPBACK, SOLID_SERVER_SEED_*).
// The prod posture requires an external OIDC issuer — SOLID_SERVER_TRUSTED_ISSUER
// is a required parameter (no default). (§1.3 of cloud-deploy-architecture.md)
//
// DECISION LOG (sq-zcou4 / lws):
//   - Container Apps avoids Kubernetes management overhead and supplies managed TLS (R3).
//   - No SOLID_SERVER_TLS_CERT/KEY: Container Apps terminates TLS; no in-container TLS needed (R3).
//   - maxReplicas is hard-pinned to 1 because the canonical image contract does not promise the
//     opt-in redis-replay feature. A Redis URL against a default build makes the binary fail closed.
//   - /livez = liveness, /readyz = readiness. Health path asymmetry from sparq-server (R7).
//   - No Azure identity or role is created: this template has no runtime cloud API/secret access,
//     so least privilege is no principal rather than a silently unused managed identity (R5).
//   - The HTTPS base URL is derived from the Container Apps environment's default domain, avoiding
//     the impossible "deploy once to discover FQDN, then redeploy" flow.

metadata gpt56Review = '[GPT-5.6] Corrected by the stronger-tier IaC security review.'
metadata securityNotice = 'The sibling sparq-server image is open-by-default; its Azure template gates it with a Key Vault token. Do not remove the token wiring.'

@description('Container Apps Environment name.')
param environmentName string = 'sparq-lws-env'

@description('Container App name.')
param containerAppName string = 'sparq-lws'

@description('Azure region.')
param location string = resourceGroup().location

@description('Full image reference. Activates once sq-lmz40 ships.')
param imageRef string = 'ghcr.io/sparq-org/sparq-lws-core:latest'

@description('Optional public HTTPS base URL override (without a trailing slash) for a configured custom domain. Empty derives the managed URL.')
param solidBaseUrl string = ''

@description('HTTP path used by the liveness and startup probes (R7).')
@minLength(1)
param livenessPath string = '/livez'

@description('HTTP path used by the readiness probe (R7).')
@minLength(1)
param readinessPath string = '/readyz'

@description('''
  Trusted OIDC issuer URL for the Solid/LWS server (e.g. https://inrupt.net).
  REQUIRED and HTTPS-only. The server rejects authentication without a trusted issuer. (§1.3)
''')
@minLength(9)
param solidTrustedIssuer string

@description('Minimum replica count. May be zero for scale-to-zero; maximum replicas is always one.')
@minValue(0)
@maxValue(1)
param minReplicas int = 1

@description('CPU allocation per replica. Memory is derived to keep a valid Container Apps consumption profile.')
@allowed([
  '0.25'
  '0.5'
  '0.75'
  '1.0'
  '1.25'
  '1.5'
  '1.75'
  '2.0'
])
param cpuCores string = '0.5'

var memoryByCpu = {
  '0.25': '0.5Gi'
  '0.5': '1Gi'
  '0.75': '1.5Gi'
  '1.0': '2Gi'
  '1.25': '2.5Gi'
  '1.5': '3Gi'
  '1.75': '3.5Gi'
  '2.0': '4Gi'
}

// [GPT-5.6] Reject an HTTP/loopback issuer before the server reaches its own fail-closed startup.
var trustedIssuer = startsWith(solidTrustedIssuer, 'https://')
  ? solidTrustedIssuer
  : fail('solidTrustedIssuer must use https://.')

// ---------------------------------------------------------------------------
// Log Analytics workspace
// ---------------------------------------------------------------------------
resource logAnalytics 'Microsoft.OperationalInsights/workspaces@2022-10-01' = {
  name: take('${environmentName}-logs', 63)
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

// Container Apps forms the app FQDN as <app-name>.<environment-default-domain>.
var managedBaseUrl = 'https://${containerAppName}.${containerAppsEnv.properties.defaultDomain}'
var effectiveBaseUrl = empty(solidBaseUrl)
  ? managedBaseUrl
  : startsWith(solidBaseUrl, 'https://') ? solidBaseUrl : fail('solidBaseUrl must use https:// when supplied.')

// ---------------------------------------------------------------------------
// Container App: sparq-lws-core
// ---------------------------------------------------------------------------
resource containerApp 'Microsoft.App/containerApps@2024-03-01' = {
  name: containerAppName
  location: location
  properties: {
    environmentId: containerAppsEnv.id
    configuration: {
      activeRevisionsMode: 'Single'
      // -----------------------------------------------------------------------
      // Ingress: HTTPS-only on port 3000 (R3/R6). allowInsecure: false.
      // Container Apps managed TLS on *.azurecontainerapps.io FQDN.
      // -----------------------------------------------------------------------
      ingress: {
        external: true
        targetPort: 3000
        transport: 'http'
        allowInsecure: false // R3: redirect plain HTTP to HTTPS
      }
    }
    template: {
      containers: [
        {
          name: 'sparq-lws'
          image: imageRef
          // [GPT-5.6] The image supplies non-root; ACA 2024-03-01 has no securityContext here.
          resources: {
            cpu: json(cpuCores)
            memory: memoryByCpu[cpuCores]
          }
          // -------------------------------------------------------------------
          // Environment — required prod env vars; no dev escape hatches (R2)
          // -------------------------------------------------------------------
          // NOTE: SOLID_SERVER_ALLOW_LOOPBACK / SOLID_SERVER_SEED_* are intentionally absent.
          env: [
            {
              name: 'SOLID_SERVER_BASE_URL'
              value: effectiveBaseUrl
            }
            {
              name: 'SOLID_SERVER_TRUSTED_ISSUER'
              value: trustedIssuer
            }
            {
              // SOLID_SERVER_AUDIENCE defaults to base URL; explicit here for clarity.
              name: 'SOLID_SERVER_AUDIENCE'
              value: effectiveBaseUrl
            }
            {
              name: 'SOLID_SERVER_BIND'
              value: '0.0.0.0:3000'
            }
          ]
          // -------------------------------------------------------------------
          // Health probes (R7): liveness on /livez, readiness on /readyz.
          // Both are ungated (no auth required). Asymmetric with sparq-server.
          // -------------------------------------------------------------------
          probes: [
            {
              type: 'Liveness'
              httpGet: {
                path: livenessPath
                port: 3000
              }
              initialDelaySeconds: 10
              periodSeconds: 30
              failureThreshold: 3
            }
            {
              type: 'Readiness'
              httpGet: {
                path: readinessPath
                port: 3000
              }
              initialDelaySeconds: 5
              periodSeconds: 10
              failureThreshold: 3
            }
            {
              type: 'Startup'
              httpGet: {
                path: livenessPath
                port: 3000
              }
              initialDelaySeconds: 5
              periodSeconds: 6
              failureThreshold: 10 // [GPT-5.6] API maximum; preserves a 60s startup window
            }
          ]
        }
      ]
      scale: {
        minReplicas: minReplicas
        // [GPT-5.6] Hard safety boundary: canonical image uses per-instance replay protection.
        maxReplicas: 1
      }
    }
  }
}

// ---------------------------------------------------------------------------
// Outputs
// ---------------------------------------------------------------------------
output fqdn string = containerApp.properties.configuration.ingress.fqdn
output solidEndpoint string = '${effectiveBaseUrl}/'
