#!/usr/bin/env bash
# [GPT-5.6] sq-lmz40 — build and exercise the native sparq-lws-core image.
#
# Usage: crates/sparq-lws-core/tests/container-smoke.sh [IMAGE_REF] [HOST_PORT]
# Set LWS_CONTAINER_SKIP_BUILD=1 only when the caller already built that exact
# IMAGE_REF (the release workflow does this to share its BuildKit cache).
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
IMAGE="${1:-sparq-lws-core:smoke}"
PORT="${2:-3000}"
NAME="sparq-lws-smoke-$$"
BASE="http://127.0.0.1:${PORT}"

cleanup() {
  echo "::group::container logs (${NAME})"
  docker logs "${NAME}" 2>&1 || true
  echo "::endgroup::"
  docker rm -f "${NAME}" >/dev/null 2>&1 || true
}
trap cleanup EXIT

cd "${ROOT}"

if [ "${LWS_CONTAINER_SKIP_BUILD:-0}" != "1" ]; then
  build_args=()
  if [ -n "${CARGO_FLAGS:-}" ]; then
    build_args+=(--build-arg "CARGO_FLAGS=${CARGO_FLAGS}")
  fi
  echo ">> building ${IMAGE}"
  docker build \
    --file crates/sparq-lws-core/Dockerfile \
    --tag "${IMAGE}" \
    "${build_args[@]}" \
    .
else
  echo ">> using caller-built ${IMAGE}"
fi

# Check the image declaration as well as the live process. An empty/root user is
# a packaging regression even on a host that happens to remap container uid 0.
configured_user="$(docker image inspect --format '{{.Config.User}}' "${IMAGE}")"
case "${configured_user}" in
  ""|0|0:0|root|root:root)
    echo "!! image is configured to run as root (${configured_user:-<empty>})" >&2
    exit 1
    ;;
esac

echo ">> running ${IMAGE} as configured user ${configured_user}"
docker run -d \
  --name "${NAME}" \
  -p "127.0.0.1:${PORT}:3000" \
  "${IMAGE}" >/dev/null

container_pid="$(docker inspect --format '{{.State.Pid}}' "${NAME}")"
runtime_uid="$(awk '/^Uid:/ { print $2; exit }' "/proc/${container_pid}/status")"
if [ -z "${runtime_uid}" ] || [ "${runtime_uid}" = "0" ]; then
  echo "!! container process is running as uid ${runtime_uid:-<unknown>}" >&2
  exit 1
fi
echo ">> process uid ${runtime_uid} is non-root"

for path in livez readyz; do
  echo ">> waiting for /${path}"
  reachable=""
  for attempt in $(seq 1 30); do
    if [ "$(docker inspect --format '{{.State.Running}}' "${NAME}" 2>/dev/null || echo false)" != "true" ]; then
      echo "!! container exited before /${path} became reachable" >&2
      exit 1
    fi
    if curl --fail --silent --show-error "${BASE}/${path}" >/dev/null 2>&1; then
      reachable=1
      echo ">> /${path} reachable (attempt ${attempt})"
      break
    fi
    sleep 1
  done
  if [ -z "${reachable}" ]; then
    echo "!! /${path} was not reachable within 30 seconds" >&2
    exit 1
  fi
done

echo ">> asserting an anonymous mutation fails closed"
status="$(curl --silent --show-error \
  --output /dev/null \
  --write-out '%{http_code}' \
  --request PUT \
  --header 'Content-Type: text/turtle' \
  --data '<> <https://example.test/predicate> "value" .' \
  "${BASE}/container-smoke-resource")"
case "${status}" in
  401|403)
    echo ">> anonymous PUT rejected with HTTP ${status}"
    ;;
  *)
    echo "!! anonymous PUT returned HTTP ${status}; expected 401 or 403" >&2
    exit 1
    ;;
esac

echo ">> SMOKE PASS: health reachable, anonymous mutation rejected, process non-root"
