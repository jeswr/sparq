{{/*
sparq Helm chart helper templates
[OPUS-4.8] sq-0d744 — cloud-deploy epic sq-3vjdr §3.5
*/}}

{{/*
Expand the chart name.
*/}}
{{- define "sparq.name" -}}
{{- default .Chart.Name .Values.nameOverride | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Create a default fully qualified app name.
We truncate at 63 chars because some Kubernetes name fields are limited to this.
If release name contains chart name it will be used as a full name.
*/}}
{{- define "sparq.fullname" -}}
{{- if .Values.fullnameOverride }}
{{- .Values.fullnameOverride | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- $name := default .Chart.Name .Values.nameOverride }}
{{- if contains $name .Release.Name }}
{{- .Release.Name | trunc 63 | trimSuffix "-" }}
{{- else }}
{{- printf "%s-%s" .Release.Name $name | trunc 63 | trimSuffix "-" }}
{{- end }}
{{- end }}
{{- end }}

{{/*
Create chart label value (chart name + version, sanitised).
*/}}
{{- define "sparq.chart" -}}
{{- printf "%s-%s" .Chart.Name .Chart.Version | replace "+" "_" | trunc 63 | trimSuffix "-" }}
{{- end }}

{{/*
Common labels applied to every resource.
*/}}
{{- define "sparq.labels" -}}
helm.sh/chart: {{ include "sparq.chart" . }}
{{ include "sparq.selectorLabels" . }}
{{- if .Chart.AppVersion }}
app.kubernetes.io/version: {{ .Chart.AppVersion | quote }}
{{- end }}
app.kubernetes.io/managed-by: {{ .Release.Service }}
{{- end }}

{{/*
Selector labels (must be stable across upgrades; do NOT include chart version).
*/}}
{{- define "sparq.selectorLabels" -}}
app.kubernetes.io/name: {{ include "sparq.name" . }}
app.kubernetes.io/instance: {{ .Release.Name }}
{{- end }}

{{/*
ServiceAccount name.
*/}}
{{- define "sparq.serviceAccountName" -}}
{{- if .Values.serviceAccount.create }}
{{- default (include "sparq.fullname" .) .Values.serviceAccount.name }}
{{- else }}
{{- default "default" .Values.serviceAccount.name }}
{{- end }}
{{- end }}

{{/*
Container port — 3030 for sparq-server, 3000 for lws.
(R7: health path and port must be per-server, never a shared constant.)
*/}}
{{- define "sparq.containerPort" -}}
{{- if eq .Values.server "lws" }}
{{- 3000 }}
{{- else }}
{{- 3030 }}
{{- end }}
{{- end }}

{{/*
Liveness probe path — /health for sparq-server, /livez for lws.
(R7: health path is per-server, never a shared constant.)
*/}}
{{- define "sparq.livenessPath" -}}
{{- if eq .Values.server "lws" -}}/livez{{- else -}}/health{{- end -}}
{{- end }}

{{/*
Readiness probe path — /health for sparq-server, /readyz for lws.
(R7: use /readyz for lws so a not-yet-ready instance deregisters from the LB.)
*/}}
{{- define "sparq.readinessPath" -}}
{{- if eq .Values.server "lws" -}}/readyz{{- else -}}/health{{- end -}}
{{- end }}

{{/*
Image reference: repo:tag (tag falls back to Chart.AppVersion when unset).
*/}}
{{- define "sparq.image" -}}
{{- $tag := .Values.image.tag | default .Chart.AppVersion }}
{{- printf "%s:%s" .Values.image.repository $tag }}
{{- end }}
