#!/usr/bin/env bash
# [SONNET-4.6] sq-80afk — structural acceptance check for the ephemeral demo manifests.
set -euo pipefail

demo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
lws="${demo_dir}/sparq-lws-demo.yaml"
idp="${demo_dir}/css-idp.yaml"

python3 - "${lws}" "${idp}" <<'PY'
import sys

try:
    import yaml
except ImportError:
    yaml = None

def validate_subset(path, text):
    """Validate the mapping/list YAML subset used by these manifests."""
    levels = [-1]
    for number, raw in enumerate(text.splitlines(), 1):
        if not raw.strip() or raw.lstrip().startswith("#"):
            continue
        if "\t" in raw[:len(raw) - len(raw.lstrip())]:
            raise SystemExit(f"{path}:{number}: tabs are not valid indentation")
        indent = len(raw) - len(raw.lstrip())
        if indent % 2:
            raise SystemExit(f"{path}:{number}: indentation must use pairs of spaces")
        line = raw.strip()
        if line.startswith("- "):
            line = line[2:]
            if ":" not in line:
                while levels[-1] >= indent:
                    levels.pop()
                if indent > levels[-1] + 2:
                    raise SystemExit(f"{path}:{number}: indentation skips a level")
                levels.append(indent)
                continue
        if ":" not in line:
            raise SystemExit(f"{path}:{number}: expected a mapping entry")
        while levels[-1] >= indent:
            levels.pop()
        if indent > levels[-1] + 2:
            raise SystemExit(f"{path}:{number}: indentation skips a level")
        levels.append(indent)

for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as source:
        text = source.read()
    if yaml is not None:
        document = yaml.safe_load(text)
        if document.get("apiVersion") != "serving.knative.dev/v1":
            raise SystemExit(f"{path}: unexpected or missing apiVersion")
        if document.get("kind") != "Service":
            raise SystemExit(f"{path}: expected a Cloud Run Service")
    else:
        validate_subset(path, text)
        if "apiVersion: serving.knative.dev/v1" not in text or "\nkind: Service\n" not in text:
            raise SystemExit(f"{path}: expected a Cloud Run Service")
PY

for manifest in "${lws}" "${idp}"; do
  grep -Fq 'autoscaling.knative.dev/minScale: "0"' "${manifest}"
  grep -Fq 'autoscaling.knative.dev/maxScale: "1"' "${manifest}"
  grep -Fq 'run.googleapis.com/cpu-throttling: "true"' "${manifest}"
  grep -Fq 'tcpSocket:' "${manifest}"
done

required_lws=(
  'SOLID_SERVER_AUDIENCE'
  'value: "solid"'
  'SOLID_SERVER_TRUSTED_PROXY'
  'value: "1"'
  'SOLID_SERVER_SEED_DEMO'
  'PSS_SPARQ_BACKEND'
  'value: "memory"'
  'SOLID_SERVER_RATE_LIMIT_PER_IP'
  'SOLID_SERVER_RATE_LIMIT_BURST'
  'SOLID_SERVER_MAX_BODY_BYTES'
  'SOLID_SERVER_REQUEST_TIMEOUT_SECS'
)
for required in "${required_lws[@]}"; do
  grep -Fq "${required}" "${lws}"
done

forbidden=(
  'SOLID_SERVER_ALLOW_LOOPBACK'
  'SOLID_SERVER_SEED_CONFORMANCE'
  'SOLID_SERVER_SEED_BENCH'
  'SOLID_SERVER_ALLOW_SEED_NONMEMORY'
)
for name in "${forbidden[@]}"; do
  if grep -Fq "${name}" "${lws}" "${idp}"; then
    echo "forbidden demo environment variable found: ${name}" >&2
    exit 1
  fi
done

if grep -Eiq '(password|passwd|api[_-]?key|private[_-]?key|client[_-]?secret|token):[[:space:]]*[^[:space:]#]+' \
  "${lws}" "${idp}"; then
  echo "possible literal secret found in demo manifest" >&2
  exit 1
fi

echo "deploy/demo manifests: OK"
