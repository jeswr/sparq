#!/usr/bin/env bash
# [SONNET-4.6] sq-80afk — structural acceptance check for the ephemeral demo manifests.
set -euo pipefail

demo_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
lws="${1:-${demo_dir}/sparq-lws-demo.yaml}"
idp="${2:-${demo_dir}/css-idp.yaml}"

python3 - "${lws}" "${idp}" <<'PY'
import re
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

documents = {}
for path in sys.argv[1:]:
    with open(path, encoding="utf-8") as source:
        text = source.read()
    documents[path] = text
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

def scalar(text, key):
    match = re.search(
        rf"^[ \t]*{re.escape(key)}:[ \t]*[\"']?([^\"'# \t\r\n]+)", text, re.M
    )
    return match.group(1) if match else None

def env_values(text):
    pairs = {}
    lines = text.splitlines()
    for index, line in enumerate(lines):
        match = re.match(r"^[ \t]*-[ \t]+name:[ \t]*[\"']?([^\"' #]+)", line)
        if match and index + 1 < len(lines):
            value = re.match(r"^[ \t]+value:[ \t]*[\"']?([^\"'#\r\n]*)", lines[index + 1])
            if value:
                pairs[match.group(1)] = value.group(1).strip()
    return pairs

lws_path, idp_path = sys.argv[1:]
expected_lws = {
    "SOLID_SERVER_AUDIENCE": "solid",
    "SOLID_SERVER_TRUSTED_PROXY": "1",
    "SOLID_SERVER_SEED_DEMO": "1",
    "PSS_SPARQ_BACKEND": "memory",
    "SOLID_SERVER_RATE_LIMIT_PER_IP": "20",
    "SOLID_SERVER_RATE_LIMIT_BURST": "40",
    "SOLID_SERVER_MAX_BODY_BYTES": "2097152",
    "SOLID_SERVER_REQUEST_TIMEOUT_SECS": "30",
}
actual_lws = env_values(documents[lws_path])
for name, expected in expected_lws.items():
    if actual_lws.get(name) != expected:
        raise SystemExit(
            f"{lws_path}: expected {name}={expected!r}, got {actual_lws.get(name)!r}"
        )

sensitive = re.compile(
    r"(PASSWORD|PASSWD|SECRET|API_?KEY|PRIVATE_?KEY|CLIENT_?SECRET|TOKEN|_SK)$",
    re.I,
)
for path, text in documents.items():
    exposed = sorted(name for name in env_values(text) if sensitive.search(name))
    if exposed:
        raise SystemExit(
            f"{path}: sensitive environment variable must use secretKeyRef: {', '.join(exposed)}"
        )
    service_account = scalar(text, "serviceAccountName")
    if not service_account or service_account.endswith("-compute@developer.gserviceaccount.com"):
        raise SystemExit(f"{path}: dedicated serviceAccountName is required")
    image = scalar(text, "image")
    digest = image.rsplit("@sha256:", 1)[-1] if image and "@sha256:" in image else ""
    if (
        not image
        or ":latest" in image
        or not (
            re.fullmatch(r"[0-9a-fA-F]{64}", digest)
            or digest in {"SPARQ_LWS_DIGEST", "CSS_DIGEST"}
        )
    ):
        raise SystemExit(f"{path}: image must use an immutable digest or digest placeholder")
PY

for manifest in "${lws}" "${idp}"; do
  grep -Fq 'autoscaling.knative.dev/minScale: "0"' "${manifest}"
  grep -Fq 'autoscaling.knative.dev/maxScale: "1"' "${manifest}"
  grep -Fq 'run.googleapis.com/cpu-throttling: "true"' "${manifest}"
  grep -Fq 'tcpSocket:' "${manifest}"
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
