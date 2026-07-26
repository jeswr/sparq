#!/usr/bin/env bash
# [OPUS-5] sq-iib9h — CSS ↔ sparq-lws-core Solid-OIDC compatibility smoke.
#
# The go/no-go de-risk of "Community Solid Server as the throwaway IdP" from
# research/lws-demo-architecture.md §1. That section's compatibility claim is DESIGNED-ONLY
# until this script is green: §1.2 says in as many words "do not claim the pairing works
# before that smoke is green". This is that smoke.
#
# WHAT MAKES IT NON-VACUOUS
# -------------------------
# Every assertion below runs against a REAL access token that a real CSS instance minted
# through its real client-credentials + DPoP flow. Nothing is stubbed, mocked, or replayed
# from a fixture. The token therefore carries whatever `typ`, `aud`, `webid`, `cnf.jkt` and
# signing algorithm CSS actually emits today, and it is verified by whatever the pinned
# `solid-oidc-verifier` rev actually enforces today. Concretely, this script goes RED if:
#
#   * CSS stops emitting RFC 9068 `typ: at+jwt` access tokens             → assertion B
#   * CSS stops injecting the Solid-OIDC `webid` claim                    → assertion B
#   * CSS changes its `aud` away from "solid"                             → assertions B + F
#   * CSS stops binding tokens to the DPoP proof key (`cnf.jkt`)          → assertion B
#   * CSS signs with an algorithm the verifier will not accept            → assertion B
#   * CSS's discovery `issuer` string stops matching what we configure    → assertion 0
#   * the generated WebID profile loses its `solid:oidcIssuer` triple, so
#     the strict bidirectional check can no longer close the loop         → assertions B + E
#   * LWS stops rejecting anonymous writes to the demo playground         → assertion A
#   * LWS stops enforcing single-use DPoP `jti` replay protection         → assertion D
#   * `SOLID_SERVER_AUDIENCE=solid` stops being load-bearing (i.e. a CSS
#     token would be accepted by a default-audience LWS too)              → assertion F
#
# Assertion F is the one that is easy to get wrong by omission. A smoke that only checked
# "the happy path works" would stay green even if the audience check were removed from the
# verifier entirely, which would silently make the demo manifest's `SOLID_SERVER_AUDIENCE`
# line decorative. So the harness runs a SECOND LWS instance that differs in exactly that
# one variable and requires the very same token to be REJECTED by it.
#
# The DPoP proofs are generated here in pure bash + openssl rather than by pulling in a
# JOSE library, so the harness has no npm/pip dependency and no lockfile to rot. That code
# is load-bearing enough to deserve its own check: `--self-test` verifies the base64url,
# RFC 7638 thumbprint and ES256 signature helpers against published test vectors and against
# openssl itself, and runs automatically before Docker is touched, so a broken helper is
# reported in a second instead of masquerading as a compatibility failure ten minutes later.
#
# USAGE
#   bash deploy/demo/compat/smoke.sh              # full smoke (needs Docker; builds the LWS image)
#   bash deploy/demo/compat/smoke.sh --self-test  # JOSE helper vectors only; no Docker needed
#   bash deploy/demo/compat/smoke.sh --keep       # leave the stack up afterwards for poking at
#
# The first run builds sparq-lws-core from source in a container and takes a while; later
# runs reuse the Docker layer cache. See README.md in this directory.

set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
COMPOSE_FILE="${HERE}/docker-compose.yml"

# These must agree with docker-compose.yml. The issuer is additionally cross-checked against
# the compose file and against CSS's own discovery document in step 0, so a drift between the
# three is reported rather than silently producing an unexplained 401.
CSS_BASE="http://localhost:3000"
CSS_ISSUER="http://localhost:3000/"
LWS_STRICT="http://localhost:3001"
LWS_DEFAULTAUD="http://localhost:3002"

KEEP=0
SELF_TEST_ONLY=0
for arg in "$@"; do
  case "${arg}" in
    --keep) KEEP=1 ;;
    --self-test) SELF_TEST_ONLY=1 ;;
    -h|--help) sed -n '/^# USAGE/,/^$/p' "${BASH_SOURCE[0]}"; exit 0 ;;
    *) echo "!! unknown argument: ${arg}" >&2; exit 2 ;;
  esac
done

WORK="$(mktemp -d)"
FAILURES=0
STACK_UP=0

say()  { printf '>> %s\n' "$*"; }
pass() { printf '   ok   %s\n' "$*"; }
fail() { printf '   FAIL %s\n' "$*" >&2; FAILURES=$((FAILURES + 1)); }
die()  { printf '!! %s\n' "$*" >&2; exit 1; }

# Setup steps that must not fail silently: every CSS account-API call goes through this. It
# accepts any 2xx rather than one exact code — these are the harness getting itself into
# position, not the thing under test, so pinning CSS's choice of 200-vs-201 here would only
# manufacture false failures. The real assertions below are exact.
expect_2xx() { # expect_2xx <actual> <body-file> <what>
  case "$1" in
    2??) return 0 ;;
  esac
  printf '!! %s: expected 2xx, got HTTP %s\n' "$3" "$1" >&2
  printf '   response body: %s\n' "$(head -c 800 "$2" 2>/dev/null || true)" >&2
  exit 1
}

assert_status() { # assert_status <expected> <actual> <what>
  if [ "$2" = "$1" ]; then pass "$3 → HTTP $2"; else fail "$3 → expected HTTP $1, got $2"; fi
}

cleanup() {
  local rc=$?
  if [ "${STACK_UP}" = "1" ] && { [ "${rc}" != "0" ] || [ "${FAILURES}" != "0" ]; }; then
    echo "---- docker compose logs (tail) ----" >&2
    docker compose -f "${COMPOSE_FILE}" logs --tail=120 >&2 2>&1 || true
    echo "------------------------------------" >&2
  fi
  if [ "${STACK_UP}" = "1" ] && [ "${KEEP}" != "1" ]; then
    docker compose -f "${COMPOSE_FILE}" down -v --remove-orphans >/dev/null 2>&1 || true
  elif [ "${STACK_UP}" = "1" ]; then
    say "--keep: stack left running. Tear down with:"
    printf '   docker compose -f %s down -v\n' "${COMPOSE_FILE}"
  fi
  rm -rf "${WORK}"
}
trap cleanup EXIT

# =====================================================================================
# JOSE / DPoP helpers — pure bash + openssl.
# =====================================================================================

# Binary on stdin → unpadded base64url on stdout.
b64url() { openssl base64 -A | tr '+/' '-_' | tr -d '='; }

# Hex string (either case) → raw bytes on stdout. The command substitution only ever carries
# the ASCII escape TEXT (`\x4f\x00...`), never the bytes themselves, so NUL bytes — which are
# perfectly normal inside an ECDSA r/s pair — survive. Capturing binary in `$(...)` would not.
hex2bin() { printf '%b' "$(printf '%s' "$1" | sed 's/../\\x&/g')"; }

# Unpadded base64url → raw bytes. openssl only speaks padded standard base64, so restore both.
b64url_decode() {
  local s="${1//-/+}"
  s="${s//_//}"
  while [ $((${#s} % 4)) -ne 0 ]; do s="${s}="; done
  printf '%s' "${s}" | openssl base64 -d -A
}

# Normalise an ASN.1 INTEGER's hex to exactly 32 bytes: DER drops leading zero bytes and adds
# one back when the high bit is set, but JOSE ES256 wants fixed-width big-endian R and S.
pad64() {
  local h="${1//[[:space:]]/}"
  while [ "${#h}" -gt 64 ] && [ "${h#0}" != "${h}" ]; do h="${h#0}"; done
  while [ "${#h}" -lt 64 ]; do h="0${h}"; done
  [ "${#h}" -eq 64 ] || die "malformed ECDSA integer (${#h} hex chars, expected 64)"
  printf '%s' "${h}"
}

# Write a fresh P-256 private key and echo its JWK public half (RFC 7638 canonical member
# order: crv, kty, x, y — the order matters because the thumbprint hashes these exact bytes).
new_es256_key() { # new_es256_key <key-path> → prints the public JWK
  local key="$1" pub_hex x y
  openssl ecparam -name prime256v1 -genkey -noout -out "${key}" 2>/dev/null
  # An uncompressed P-256 point is the last 65 bytes of the DER SubjectPublicKeyInfo:
  # 0x04 ‖ X(32) ‖ Y(32).
  pub_hex="$(openssl ec -in "${key}" -pubout -outform DER 2>/dev/null | tail -c 65 |
             od -An -tx1 -v | tr -d ' \n')"
  [ "${#pub_hex}" -eq 130 ] && [ "${pub_hex:0:2}" = "04" ] ||
    die "unexpected P-256 public point encoding from openssl"
  x="$(hex2bin "${pub_hex:2:64}" | b64url)"
  y="$(hex2bin "${pub_hex:66:64}" | b64url)"
  printf '{"crv":"P-256","kty":"EC","x":"%s","y":"%s"}' "${x}" "${y}"
}

# RFC 7638 JWK thumbprint (the `cnf.jkt` the access token is bound to).
jwk_thumbprint() { printf '%s' "$1" | openssl dgst -sha256 -binary | b64url; }

# Compact ES256 JWS over an already-serialised header and payload.
jws_es256() { # jws_es256 <key-path> <header-json> <payload-json>
  local key="$1" header="$2" payload="$3" signing_input ints r s
  signing_input="$(printf '%s' "${header}" | b64url).$(printf '%s' "${payload}" | b64url)"
  printf '%s' "${signing_input}" > "${WORK}/si.bin"
  openssl dgst -sha256 -sign "${key}" -out "${WORK}/sig.der" "${WORK}/si.bin"
  ints="$(openssl asn1parse -inform DER -in "${WORK}/sig.der" | sed -n 's/.*INTEGER *://p')"
  r="$(pad64 "$(printf '%s\n' "${ints}" | sed -n 1p)")"
  s="$(pad64 "$(printf '%s\n' "${ints}" | sed -n 2p)")"
  printf '%s.%s' "${signing_input}" "$(hex2bin "${r}${s}" | b64url)"
}

# An RFC 9449 DPoP proof. `ath` is included whenever an access token is supplied — the
# verifier requires it on resource requests and would 401 without it.
dpop_proof() { # dpop_proof <key-path> <jwk> <htm> <htu> [access-token]
  local key="$1" jwk="$2" htm="$3" htu="$4" token="${5:-}" payload ath
  payload="$(printf '{"htm":"%s","htu":"%s","iat":%s,"jti":"%s"' \
    "${htm}" "${htu}" "$(date +%s)" "$(openssl rand -hex 16)")"
  if [ -n "${token}" ]; then
    ath="$(printf '%s' "${token}" | openssl dgst -sha256 -binary | b64url)"
    payload="${payload},\"ath\":\"${ath}\""
  fi
  payload="${payload}}"
  jws_es256 "${key}" "{\"alg\":\"ES256\",\"typ\":\"dpop+jwt\",\"jwk\":${jwk}}" "${payload}"
}

# =====================================================================================
# Small JSON + HTTP helpers.
# =====================================================================================

# Read a dotted path out of a JSON file. python3 rather than jq: it is far more reliably
# present on a box that has Docker, and this harness already refuses to guess.
jget() { # jget <file> <dotted.path>
  python3 - "$1" "$2" <<'PY'
import json, sys
node = json.load(open(sys.argv[1]))
for key in sys.argv[2].split('.'):
    if not isinstance(node, dict) or key not in node:
        node = None
        break
    node = node[key]
print('' if node is None else node)
PY
}

# The single key of a one-entry object (CSS returns WebID links as {webid: resource-url}).
jfirstkey() { # jfirstkey <file> <dotted.path>
  python3 - "$1" "$2" <<'PY'
import json, sys
node = json.load(open(sys.argv[1]))
for key in sys.argv[2].split('.'):
    node = node.get(key, {}) if isinstance(node, dict) else {}
print(next(iter(node), '') if isinstance(node, dict) else '')
PY
}

json_obj() { # json_obj k v k v ... → a JSON object, correctly escaped
  python3 -c 'import json,sys; a=sys.argv[1:]; print(json.dumps(dict(zip(a[::2], a[1::2]))))' "$@"
}

# `id:secret`, each form-encoded then the pair base64'd — exactly what CSS documents for the
# client-credentials Basic header.
basic_auth() { # basic_auth <id> <secret>
  python3 - "$1" "$2" <<'PY'
import base64, sys
from urllib.parse import quote
pair = quote(sys.argv[1], safe='') + ':' + quote(sys.argv[2], safe='')
print(base64.b64encode(pair.encode()).decode())
PY
}

# Perform a request; write the body to <out> and echo the status code.
http() { # http <method> <url> <out> [curl args...]
  local method="$1" url="$2" out="$3"
  shift 3
  curl -sS -o "${out}" -w '%{http_code}' -X "${method}" "${url}" "$@"
}

# =====================================================================================
# Self-test of the JOSE helpers (runs before Docker; also available standalone).
# =====================================================================================

self_test() {
  say "self-test: JOSE helpers"

  # base64url must translate 62/63 and strip padding. 0x3f3f3e → "Pz8-", 0xfbff → "-_8".
  [ "$(hex2bin '3f3f3e' | b64url)" = "Pz8-" ] || fail "base64url '+' → '-' translation"
  [ "$(hex2bin 'fbff'   | b64url)" = "-_8"  ] || fail "base64url '/' → '_' translation + padding strip"

  # pad64 with explicit inputs rather than whatever openssl happens to emit. DER strips leading
  # zero bytes and prepends one when the high bit is set, so both directions are real cases —
  # but the strip case turns up in roughly half of all signatures while the pad case turns up in
  # about one in 256, far too rare to rely on a random key to exercise.
  [ "$(pad64 "00A1B2")" = "000000000000000000000000000000000000000000000000000000000000A1B2" ] ||
    fail "pad64 must left-pad a short integer to 32 bytes"
  [ "$(pad64 "001111111111111111111111111111111111111111111111111111111111111111")" = \
      "1111111111111111111111111111111111111111111111111111111111111111" ] ||
    fail "pad64 must strip DER's high-bit zero prefix"

  # RFC 9449 §6.1's published example JWK and its `jkt`. If the canonical member order or the
  # hashing were wrong this vector would not reproduce.
  local vector_jwk vector_jkt
  vector_jwk='{"crv":"P-256","kty":"EC","x":"l8tFrhx-34tV3hRICRDY9zCkDlpBhF42UQUfWVAWBFs","y":"9VE4jf_Ok_o64zbTTlcuNJajHmt6v9TDVrU0CdvGRDA"}'
  vector_jkt='0ZcOCORZNYy-DWpqq30jZyJGHTN0d2HglBV3uiguA4I'
  [ "$(jwk_thumbprint "${vector_jwk}")" = "${vector_jkt}" ] ||
    fail "RFC 9449 JWK thumbprint vector"

  # Round-trip a real signature: openssl must verify the DER form, and the raw R‖S we hand to
  # JOSE must decode back to the same two integers at the fixed 64-byte width.
  local key jwk proof sig_b64 raw_hex ints members
  key="${WORK}/selftest.pem"
  jwk="$(new_es256_key "${key}")"

  # The vector above only proves jwk_thumbprint hashes what it is handed. The thumbprint that
  # actually has to match CSS's `cnf.jkt` is taken over the JWK *we* build, so its members must
  # be in RFC 7638 lexicographic order too — assert that separately or a reordering here would
  # sail past the vector.
  members="$(python3 -c 'import json,sys; print(",".join(json.loads(sys.argv[1])))' "${jwk}")"
  [ "${members}" = "crv,kty,x,y" ] ||
    fail "generated JWK members must be in RFC 7638 canonical order, got '${members}'"
  proof="$(dpop_proof "${key}" "${jwk}" "POST" "http://localhost:3000/.oidc/token")"
  [ "$(printf '%s' "${proof}" | tr -cd '.' | wc -c)" -eq 2 ] || fail "compact JWS must have 3 parts"
  openssl ec -in "${key}" -pubout -out "${WORK}/selftest.pub" 2>/dev/null
  openssl dgst -sha256 -verify "${WORK}/selftest.pub" \
    -signature "${WORK}/sig.der" "${WORK}/si.bin" >/dev/null ||
    fail "openssl could not verify the ES256 signature it just produced"
  sig_b64="${proof##*.}"
  raw_hex="$(b64url_decode "${sig_b64}" | od -An -tx1 -v | tr -d ' \n')"
  [ "${#raw_hex}" -eq 128 ] || fail "ES256 signature must be 64 raw bytes, got $((${#raw_hex} / 2))"
  ints="$(openssl asn1parse -inform DER -in "${WORK}/sig.der" | sed -n 's/.*INTEGER *://p')"
  [ "${raw_hex:0:64}"  = "$(pad64 "$(printf '%s\n' "${ints}" | sed -n 1p)" | tr 'A-F' 'a-f')" ] ||
    fail "raw R does not round-trip from the DER signature"
  [ "${raw_hex:64:64}" = "$(pad64 "$(printf '%s\n' "${ints}" | sed -n 2p)" | tr 'A-F' 'a-f')" ] ||
    fail "raw S does not round-trip from the DER signature"

  # openssl verifying its own signature only proves internal consistency — it says nothing
  # about WHICH bytes were signed. JWS requires exactly `b64url(header).b64url(payload)`, so
  # pin that against the proof we emitted.
  [ "$(cat "${WORK}/si.bin")" = "${proof%.*}" ] ||
    fail "the signed bytes must be exactly the first two segments of the compact JWS"

  # And the proof has to be a DPoP proof, not merely a well-formed JWS: RFC 9449 fixes `typ`
  # and requires htm/htu/iat/jti. `ath` must appear only when an access token is bound.
  local decoded
  decoded="$(python3 - "${proof}" <<'PY'
import base64, json, sys
def seg(s):
    return json.loads(base64.urlsafe_b64decode(s + '=' * (-len(s) % 4)))
head, body = seg(sys.argv[1].split('.')[0]), seg(sys.argv[1].split('.')[1])
ok = (head.get('typ') == 'dpop+jwt' and head.get('alg') == 'ES256'
      and isinstance(head.get('jwk'), dict)
      and body.get('htm') == 'POST'
      and body.get('htu') == 'http://localhost:3000/.oidc/token'
      and isinstance(body.get('iat'), int) and body.get('jti')
      and 'ath' not in body)
print('ok' if ok else json.dumps({'header': head, 'payload': body}))
PY
)"
  [ "${decoded}" = "ok" ] || fail "DPoP proof claim set is wrong: ${decoded}"

  # Two proofs must never share a jti, or assertion D's replay check would be testing nothing.
  local jti_a jti_b
  jti_a="$(dpop_proof "${key}" "${jwk}" GET http://localhost:3001/x | cut -d. -f2)"
  jti_b="$(dpop_proof "${key}" "${jwk}" GET http://localhost:3001/x | cut -d. -f2)"
  [ "${jti_a}" != "${jti_b}" ] || fail "consecutive DPoP proofs must not be byte-identical"

  if [ "${FAILURES}" = "0" ]; then pass "JOSE helpers agree with the published vectors"; fi
}

# =====================================================================================
# Preflight.
# =====================================================================================

for tool in curl openssl python3 sed od; do
  command -v "${tool}" >/dev/null 2>&1 || die "missing required tool: ${tool}"
done

self_test
if [ "${FAILURES}" != "0" ]; then
  die "${FAILURES} self-test failure(s) — the JOSE helpers are broken, so any compatibility \
verdict from this harness would be meaningless"
fi
if [ "${SELF_TEST_ONLY}" = "1" ]; then
  say "self-test PASSED"
  exit 0
fi

command -v docker >/dev/null 2>&1 || die "missing required tool: docker"
docker compose version >/dev/null 2>&1 || die "Docker Compose v2 is required (\`docker compose\`)"

# Guard the harness's own invariants before spending a build on it: both LWS services must
# trust the SAME issuer string, and exactly ONE of them may pin the audience — otherwise
# assertion F is not testing what it claims to test.
compose_issuers="$(sed -n 's/.*SOLID_SERVER_TRUSTED_ISSUER: *//p' "${COMPOSE_FILE}" | sort -u)"
[ "$(printf '%s\n' "${compose_issuers}" | wc -l)" -eq 1 ] ||
  die "the two LWS services must trust one identical issuer string; found: ${compose_issuers}"
[ "${compose_issuers}" = "${CSS_ISSUER}" ] ||
  die "compose trusts '${compose_issuers}' but this script expects '${CSS_ISSUER}'"
audience_lines="$(grep -c 'SOLID_SERVER_AUDIENCE:' "${COMPOSE_FILE}" || true)"
[ "${audience_lines}" -eq 1 ] ||
  die "exactly one service may set SOLID_SERVER_AUDIENCE (found ${audience_lines}) — that \
one-variable delta IS assertion F"

# =====================================================================================
# Bring the stack up.
# =====================================================================================

say "building + starting the stack (the first LWS build is slow — it compiles from source)"
docker compose -f "${COMPOSE_FILE}" up -d --build
STACK_UP=1

wait_for() { # wait_for <url> <what>
  local url="$1" what="$2" i
  for i in $(seq 1 90); do
    if curl -fsS -o /dev/null --max-time 3 "${url}" 2>/dev/null; then
      say "${what} is up"
      return 0
    fi
    sleep 2
  done
  die "${what} never became reachable at ${url}"
}

wait_for "${CSS_BASE}/.well-known/openid-configuration" "CSS"
wait_for "${LWS_STRICT}/readyz"     "lws-strict"
wait_for "${LWS_DEFAULTAUD}/readyz" "lws-defaultaud"

# =====================================================================================
# Step 0 — discovery. The issuer is compared byte-for-byte because that is how the verifier
# compares it; a trailing-slash difference here is a real, if boring, incompatibility.
# =====================================================================================

say "step 0: OIDC discovery"
curl -fsS "${CSS_BASE}/.well-known/openid-configuration" -o "${WORK}/disco.json" ||
  die "CSS discovery document is not fetchable"
discovered_issuer="$(jget "${WORK}/disco.json" issuer)"
token_endpoint="$(jget "${WORK}/disco.json" token_endpoint)"
jwks_uri="$(jget "${WORK}/disco.json" jwks_uri)"

if [ "${discovered_issuer}" = "${CSS_ISSUER}" ]; then
  pass "CSS discovery issuer is byte-identical to SOLID_SERVER_TRUSTED_ISSUER (${CSS_ISSUER})"
else
  fail "issuer drift: CSS reports '${discovered_issuer}', compose trusts '${CSS_ISSUER}' \
(the verifier compares these as exact strings — update docker-compose.yml + this script together)"
fi
[ -n "${token_endpoint}" ] || die "discovery document has no token_endpoint"
[ -n "${jwks_uri}" ] || die "discovery document has no jwks_uri"
curl -fsS "${jwks_uri}" -o "${WORK}/jwks.json" || die "JWKS is not fetchable at ${jwks_uri}"
pass "JWKS fetchable at ${jwks_uri}"

# =====================================================================================
# Step 1 — register a throwaway CSS account, pod and WebID, then a client-credentials token.
# Follows the documented control-driven account API rather than hard-coded paths, so a CSS
# route rename shows up as a clear "control missing" failure instead of a 404 mystery.
# =====================================================================================

say "step 1: registering a throwaway CSS account + pod"
JSON_ACCEPT='accept: application/json'
SUFFIX="$(openssl rand -hex 4)"
EMAIL="compat-${SUFFIX}@example.invalid"
PASSWORD="compat-${SUFFIX}-pw"
POD_NAME="compat${SUFFIX}"

curl -fsS "${CSS_BASE}/.account/" -H "${JSON_ACCEPT}" -o "${WORK}/idx0.json" ||
  die "CSS account API index is not fetchable"
create_url="$(jget "${WORK}/idx0.json" controls.account.create)"
[ -n "${create_url}" ] || die "CSS index exposes no controls.account.create"

code="$(http POST "${create_url}" "${WORK}/acct.json" \
  -H 'content-type: application/json' -H "${JSON_ACCEPT}" --data '{}')"
expect_2xx "${code}" "${WORK}/acct.json" "create CSS account"
ACCOUNT_TOKEN="$(jget "${WORK}/acct.json" authorization)"
[ -n "${ACCOUNT_TOKEN}" ] || die "CSS account creation returned no authorization value"
AUTH_HDR="authorization: CSS-Account-Token ${ACCOUNT_TOKEN}"

# The controls object grows once authenticated — password/pod/client-credentials appear here.
curl -fsS "${CSS_BASE}/.account/" -H "${AUTH_HDR}" -H "${JSON_ACCEPT}" -o "${WORK}/idx1.json" ||
  die "authenticated CSS account API index is not fetchable"
password_url="$(jget "${WORK}/idx1.json" controls.password.create)"
pod_url="$(jget "${WORK}/idx1.json" controls.account.pod)"
webid_url="$(jget "${WORK}/idx1.json" controls.account.webId)"
cc_url="$(jget "${WORK}/idx1.json" controls.account.clientCredentials)"
for pair in "controls.password.create:${password_url}" "controls.account.pod:${pod_url}" \
            "controls.account.webId:${webid_url}" "controls.account.clientCredentials:${cc_url}"; do
  [ -n "${pair#*:}" ] || die "authenticated CSS index is missing ${pair%%:*}"
done

# A fresh account is unusable until it has a login method attached.
code="$(http POST "${password_url}" "${WORK}/pw.json" -H "${AUTH_HDR}" \
  -H 'content-type: application/json' -H "${JSON_ACCEPT}" --data "$(json_obj email "${EMAIL}" password "${PASSWORD}")")"
expect_2xx "${code}" "${WORK}/pw.json" "attach email/password login"

# No `webId` in the settings ⇒ CSS generates one inside the pod and links it to the account.
code="$(http POST "${pod_url}" "${WORK}/pod.json" -H "${AUTH_HDR}" \
  -H 'content-type: application/json' -H "${JSON_ACCEPT}" --data "$(json_obj name "${POD_NAME}")")"
expect_2xx "${code}" "${WORK}/pod.json" "create pod ${POD_NAME}"

curl -fsS "${webid_url}" -H "${AUTH_HDR}" -H "${JSON_ACCEPT}" -o "${WORK}/webid.json" ||
  die "could not list linked WebIDs"
WEBID="$(jfirstkey "${WORK}/webid.json" webIdLinks)"
[ -n "${WEBID}" ] || die "no WebID was linked to the account after pod creation"
say "WebID: ${WEBID}"

code="$(http POST "${cc_url}" "${WORK}/cc.json" -H "${AUTH_HDR}" \
  -H 'content-type: application/json' -H "${JSON_ACCEPT}" \
  --data "$(json_obj name "compat-${SUFFIX}" webId "${WEBID}")")"
expect_2xx "${code}" "${WORK}/cc.json" "create client credentials"
CC_ID="$(jget "${WORK}/cc.json" id)"
CC_SECRET="$(jget "${WORK}/cc.json" secret)"
[ -n "${CC_ID}" ] && [ -n "${CC_SECRET}" ] || die "client-credentials response lacked id/secret"

# =====================================================================================
# Step 2 — exchange the credentials for a real DPoP-bound access token.
# =====================================================================================

say "step 2: minting a DPoP-bound access token at ${token_endpoint}"
DPOP_KEY="${WORK}/dpop.pem"
DPOP_JWK="$(new_es256_key "${DPOP_KEY}")"
DPOP_JKT="$(jwk_thumbprint "${DPOP_JWK}")"

code="$(http POST "${token_endpoint}" "${WORK}/token.json" \
  -H "authorization: Basic $(basic_auth "${CC_ID}" "${CC_SECRET}")" \
  -H 'content-type: application/x-www-form-urlencoded' \
  -H "dpop: $(dpop_proof "${DPOP_KEY}" "${DPOP_JWK}" POST "${token_endpoint}")" \
  --data 'grant_type=client_credentials&scope=webid')"
expect_2xx "${code}" "${WORK}/token.json" "client-credentials token request"
ACCESS_TOKEN="$(jget "${WORK}/token.json" access_token)"
[ -n "${ACCESS_TOKEN}" ] || die "token response carried no access_token"

# Decode the claims for the report. These are NOT the assertions — LWS's verdict is. They
# exist so that when an assertion below fails, the log already says why.
python3 - "${ACCESS_TOKEN}" "${DPOP_JKT}" <<'PY'
import base64, json, sys
def seg(s):
    return json.loads(base64.urlsafe_b64decode(s + '=' * (-len(s) % 4)))
header, payload = seg(sys.argv[1].split('.')[0]), seg(sys.argv[1].split('.')[1])
print(f"   token typ={header.get('typ')!r} alg={header.get('alg')!r}")
print(f"   token iss={payload.get('iss')!r} aud={payload.get('aud')!r}")
print(f"   token webid={payload.get('webid')!r}")
print(f"   token cnf.jkt={(payload.get('cnf') or {}).get('jkt')!r} (our key: {sys.argv[2]!r})")
PY

# =====================================================================================
# Assertions.
# =====================================================================================

say "step 3: assertions against ${LWS_STRICT} (SOLID_SERVER_AUDIENCE=solid)"
TURTLE="<https://example.org/compat/${SUFFIX}#it> <http://xmlns.com/foaf/0.1/name> \"compat ${SUFFIX}\" ."
authed_put_target="${LWS_STRICT}/playground/note-${SUFFIX}"

# (A) Anonymous writes stay rejected. This is the ONLY write friction the public demo design
#     relies on (§3.1/§5 item 2), so it is checked before the happy path.
code="$(http PUT "${LWS_STRICT}/playground/anon-${SUFFIX}" "${WORK}/anon.txt" \
  -H 'content-type: text/turtle' --data-binary "${TURTLE}")"
assert_status 401 "${code}" "A: anonymous PUT under /playground/ is refused"

# (B) The whole point: a real CSS token is accepted. Passing this requires CSS's `typ`, `aud`,
#     `webid`, `cnf.jkt` and signing alg to all satisfy the pinned verifier AND the strict
#     bidirectional WebID→issuer check to close.
code="$(http PUT "${authed_put_target}" "${WORK}/put.txt" \
  -H "authorization: DPoP ${ACCESS_TOKEN}" \
  -H "dpop: $(dpop_proof "${DPOP_KEY}" "${DPOP_JWK}" PUT "${authed_put_target}" "${ACCESS_TOKEN}")" \
  -H 'content-type: text/turtle' --data-binary "${TURTLE}")"
assert_status 201 "${code}" "B: authed PUT under /playground/ with a real CSS token creates"

# (C) …and the bytes actually landed, so (B) is not passing on an empty write path.
code="$(http GET "${authed_put_target}" "${WORK}/get.ttl" \
  -H "authorization: DPoP ${ACCESS_TOKEN}" \
  -H "dpop: $(dpop_proof "${DPOP_KEY}" "${DPOP_JWK}" GET "${authed_put_target}" "${ACCESS_TOKEN}")" \
  -H 'accept: text/turtle')"
if [ "${code}" = "200" ] && grep -q "compat ${SUFFIX}" "${WORK}/get.ttl"; then
  pass "C: the written resource reads back with its content"
else
  fail "C: read-back returned HTTP ${code} without the written triple"
fi

# (D) DPoP `jti` is single-use. One proof, two identical requests: the second must 401. If
#     replay protection regressed the second would be an ordinary update (2xx), not a 401.
replay_target="${LWS_STRICT}/playground/replay-${SUFFIX}"
replay_proof="$(dpop_proof "${DPOP_KEY}" "${DPOP_JWK}" PUT "${replay_target}" "${ACCESS_TOKEN}")"
code="$(http PUT "${replay_target}" "${WORK}/r1.txt" \
  -H "authorization: DPoP ${ACCESS_TOKEN}" -H "dpop: ${replay_proof}" \
  -H 'content-type: text/turtle' --data-binary "${TURTLE}")"
if [ "${code}" = "201" ]; then
  pass "D(setup): first use of the proof creates"
else
  fail "D(setup): first use of the proof returned HTTP ${code}, expected 201"
fi
code="$(http PUT "${replay_target}" "${WORK}/r2.txt" \
  -H "authorization: DPoP ${ACCESS_TOKEN}" -H "dpop: ${replay_proof}" \
  -H 'content-type: text/turtle' --data-binary "${TURTLE}")"
assert_status 401 "${code}" "D: replaying the same DPoP proof (same jti) is refused"

# (E) The bidirectional half of (B), asserted directly on the source document so a failure
#     tells you WHICH side broke: CSS's profile template or the verifier's check.
webid_doc="${WEBID%%#*}"
code="$(http GET "${webid_doc}" "${WORK}/card.ttl" -H 'accept: text/turtle')"
# The `>?` accepts both the prefixed form (`solid:oidcIssuer <…>`) and the expanded one
# (`<http://www.w3.org/ns/solid/terms#oidcIssuer> <…>`), whichever CSS serialises.
if [ "${code}" = "200" ] &&
   grep -qE "oidcIssuer>?[[:space:]]*<${CSS_ISSUER}>" "${WORK}/card.ttl"; then
  pass "E: the CSS WebID document carries solid:oidcIssuer → ${CSS_ISSUER}"
else
  fail "E: WebID document at ${webid_doc} (HTTP ${code}) does not point back at ${CSS_ISSUER}"
fi

# (F) The control. Same token, same shape of request, an LWS that differs ONLY in leaving
#     SOLID_SERVER_AUDIENCE at its default. It must refuse — that is what makes the demo
#     manifest's `SOLID_SERVER_AUDIENCE=solid` line load-bearing rather than decorative.
say "step 4: control assertion against ${LWS_DEFAULTAUD} (SOLID_SERVER_AUDIENCE unset)"
defaultaud_target="${LWS_DEFAULTAUD}/playground/note-${SUFFIX}"
code="$(http PUT "${defaultaud_target}" "${WORK}/aud.txt" \
  -H "authorization: DPoP ${ACCESS_TOKEN}" \
  -H "dpop: $(dpop_proof "${DPOP_KEY}" "${DPOP_JWK}" PUT "${defaultaud_target}" "${ACCESS_TOKEN}")" \
  -H 'content-type: text/turtle' --data-binary "${TURTLE}")"
assert_status 401 "${code}" "F: the same token is refused when SOLID_SERVER_AUDIENCE is defaulted"

# =====================================================================================

echo
if [ "${FAILURES}" = "0" ]; then
  say "COMPAT SMOKE PASSED — CSS-minted DPoP tokens verify against sparq-lws-core"
  say "(local harness result: it de-risks the §1 pairing; it is not a deployment test)"
  exit 0
fi
die "COMPAT SMOKE FAILED — ${FAILURES} assertion(s) red"
