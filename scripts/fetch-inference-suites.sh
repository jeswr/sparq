#!/usr/bin/env bash
# Fetches the inference/reasoning conformance suites into the gitignored
# tests/w3c/ directory, each at a PINNED revision (pass-rates are only
# comparable when the suite revision is fixed). Test data is never committed.
# Run once, then everything is runnable offline:
#   cargo run --release -p sparq-conformance --bin sparq-inference-conformance
#
# Suites:
#   1. w3c/rdf-tests           — RDF Semantics (rdf/rdf11/rdf-mt) + sparql11/entailment.
#      Same pinned clone as scripts/fetch-conformance.sh (delegated to it).
#   2. w3c/N3                  — the N3 Community Group test suite (tests/N3Tests),
#      the manifests EYE and cwm run.
#   3. OWL 2 test cases        — the W3C OWL WG test repository export (all.rdf,
#      every test case incl. premise/conclusion ontologies as literals). The
#      original wiki (owl.semanticweb.org) is offline; the canonical bulk export
#      is preserved by the Internet Archive. The snapshot TIMESTAMP in the URL is
#      the pin; the sha256 check makes any drift loud.
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/.." && pwd)"

# [OPUS-4.8] sq-nj0pd: `retry` + the pinned-clone idiom now live in the shared
# lib so every conformance suite-fetch retries identically (sq-y5dz seeded this
# resilience here; centralising it also covers the delegated fetch-conformance.sh
# in step 1 below — the w3c/rdf-tests clone that was the remaining un-retried
# network op in the inference lane). GitHub and the Internet Archive both reset CI
# transfers occasionally; one blip should not red-gate every engine PR.
# shellcheck source=scripts/lib/fetch-retry.sh
. "$ROOT/scripts/lib/fetch-retry.sh"

# --- 1. w3c/rdf-tests (shared pin with the SPARQL conformance harness) -------
"$ROOT/scripts/fetch-conformance.sh"

# --- 2. w3c/N3 ----------------------------------------------------------------
# Pinned w3c/N3 commit (master, 2026-06).
N3_PIN="23ccf3d56b25cb60a68878a04aae0d52493080f0"
N3_DEST="$ROOT/tests/w3c/n3"

retry_git_clone_pinned "https://github.com/w3c/N3" "$N3_DEST" "$N3_PIN"
echo "w3c/N3 pinned at $N3_PIN."

# --- 3. OWL 2 test cases (W3C OWL WG export, archived) -------------------------
# Pin = the Internet Archive snapshot timestamp (an immutable capture of
# http://owl.semanticweb.org/exports/all.rdf, the full test-case export of the
# OWL WG wiki referenced by the OWL 2 Conformance REC) + sha256 of the payload.
OWL_SNAPSHOT="20160703034201"
OWL_URL="http://web.archive.org/web/${OWL_SNAPSHOT}if_/http://owl.semanticweb.org/exports/all.rdf"
OWL_SHA256="446e9eae0488e7eb58a8bd7db92b5fb358316c63e3cad1749103cd912664bee4"
OWL_DEST="$ROOT/tests/w3c/owl2/all.rdf"

sha256_of() {
    if command -v sha256sum >/dev/null 2>&1; then
        sha256sum "$1" | awk '{print $1}'
    else
        shasum -a 256 "$1" | awk '{print $1}'
    fi
}

if [ -f "$OWL_DEST" ] && [ "$(sha256_of "$OWL_DEST")" = "$OWL_SHA256" ]; then
    echo "OWL 2 test cases already present (sha256 ok) — nothing to do."
else
    mkdir -p "$(dirname "$OWL_DEST")"
    echo "Downloading the OWL 2 test-case export (archived snapshot $OWL_SNAPSHOT)…"
    # [OPUS-4.8] The Internet Archive (web.archive.org) regularly RESETS large
    # transfers from CI runner IP ranges mid-stream: the symptom is
    #   curl: (56) Recv failure: Connection reset by peer
    # which exits 56. Plain `--retry` does NOT cover (56) — it only retries
    # transient HTTP statuses (408/429/5xx) and connection-establishment
    # failures, so the previous `--retry 3` never actually re-tried this
    # reset and red-gated every engine PR (sq-y5dz). `--retry-all-errors`
    # (curl ≥7.71, runner has 8.x) makes curl retry on ANY error incl. (56);
    # `--retry-connrefused` covers a refused connect; the connect/max-time
    # bounds keep a genuinely-dead source from hanging the lane. The sha256
    # gate below still makes any payload drift loud, so retrying is safe.
    if ! curl -sSfL \
            --retry 5 --retry-delay 5 --retry-all-errors --retry-connrefused \
            --connect-timeout 30 --max-time 300 \
            -o "$OWL_DEST.tmp" "$OWL_URL"; then
        rm -f "$OWL_DEST.tmp"
        echo "ERROR: could not download the OWL 2 test-case export after retries." >&2
        echo "  URL: $OWL_URL" >&2
        echo "  This is the Internet Archive snapshot of the (offline) OWL WG wiki" >&2
        echo "  export; transient connection resets are common. Re-run the job, or" >&2
        echo "  if the archive is persistently unreachable, see scripts/fetch-inference-suites.sh." >&2
        exit 1
    fi
    HAVE_SHA="$(sha256_of "$OWL_DEST.tmp")"
    if [ "$HAVE_SHA" != "$OWL_SHA256" ]; then
        rm -f "$OWL_DEST.tmp"
        echo "ERROR: OWL 2 export checksum mismatch (got $HAVE_SHA, want $OWL_SHA256)." >&2
        exit 1
    fi
    mv "$OWL_DEST.tmp" "$OWL_DEST"
    echo "OWL 2 test cases pinned (snapshot $OWL_SNAPSHOT, sha256 ok)."
fi
