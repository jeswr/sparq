#!/usr/bin/env bash
# [SONNET-4.6] Mutation tests for the demo manifest acceptance check.
set -euo pipefail

demo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
scratch="$(mktemp -d)"
trap 'rm -rf "${scratch}"' EXIT

expect_rejected() {
  local label="$1"
  if bash "${demo_dir}/check.sh" "${scratch}/lws.yaml" "${scratch}/idp.yaml" >/dev/null 2>&1; then
    echo "demo check self-test failed to reject ${label}" >&2
    exit 1
  fi
}

cp "${demo_dir}/sparq-lws-demo.yaml" "${scratch}/lws.yaml"
cp "${demo_dir}/css-idp.yaml" "${scratch}/idp.yaml"
sed -i '/          startupProbe:/i\
            - name: SOLID_SERVER_DPOP_SK\
              value: "AAAAC3NzaC1lZDI1NTE5SUPERSECRETKEY"' "${scratch}/lws.yaml"
expect_rejected "literal sensitive environment variable"

cp "${demo_dir}/sparq-lws-demo.yaml" "${scratch}/lws.yaml"
sed -i '/SOLID_SERVER_TRUSTED_PROXY/{n;s/value: "1"/value: "0"/;}' "${scratch}/lws.yaml"
expect_rejected "changed trusted-proxy value"

cp "${demo_dir}/sparq-lws-demo.yaml" "${scratch}/lws.yaml"
sed -i 's|@sha256:SPARQ_LWS_DIGEST|:latest|' "${scratch}/lws.yaml"
expect_rejected "mutable image tag"

cp "${demo_dir}/sparq-lws-demo.yaml" "${scratch}/lws.yaml"
sed -i '/serviceAccountName:/d' "${scratch}/lws.yaml"
expect_rejected "missing runtime service account"

echo "deploy/demo check self-tests: OK"
