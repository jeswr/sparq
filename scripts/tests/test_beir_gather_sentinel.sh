#!/usr/bin/env bash
# [FABLE-5] Hermetic self-tests for the SENTINEL PROTOCOL of
# scripts/bench/canonical-beir-gather-instance.sh (PR #3488 review round 1).
#
# WHY: the gather deliberately WARN-and-continues past provisioning/build/per-cut
# failures (to keep the box alive and collect partial evidence), but it must NEVER
# launder those failures into success — GATHER_DONE + gather-meta.json canonical=true
# are allowed ONLY when every requested cut has a valid-JSON envelope for BOTH
# engines (lucene-anserini oracle AND sparq-text). Anything less must write the
# DISTINCT /root/GATHER_FAILED sentinel, canonical=false + status partial|failed,
# and exit nonzero, so the launcher can machine-distinguish an honest NOT-RUN from
# a completed canonical gather. This harness pins exactly that, covering TOTAL
# failure, PARTIAL output, invalid-JSON envelopes, and the success path (including
# console-recovery of the emitted envelopes via extract-console-envelopes.sh).
#
# HERMETIC: the script under test is copied into a scratch repo tree whose
# scripts/gather-competitors.sh is a scenario stub; apt-get/java/cargo/curl and the
# venv python3 are PATH-shadowed (python3 -m venv/-m pip never touch the network;
# everything else execs the real python3); SENTINEL_DIR/BEIR_VENV_DIR/BEIR_DATA_DIR
# point into the sandbox. No AWS, no network, no /root writes.
# Run: bash scripts/tests/test_beir_gather_sentinel.sh   (exit 0 = all pass)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
SCRIPT="$ROOT/scripts/bench/canonical-beir-gather-instance.sh"
EXTRACT="$ROOT/scripts/bench/extract-console-envelopes.sh"
[ -f "$SCRIPT" ] || { echo "FATAL: $SCRIPT not found"; exit 2; }
REAL_PY="$(command -v python3)" || { echo "FATAL: python3 required"; exit 2; }

pass=0; fail=0
ok()  { pass=$((pass + 1)); }
bad() { fail=$((fail + 1)); printf 'CASE FAILED: %s\n' "$1"; }

SANDBOX="$(mktemp -d)"
trap 'rm -rf "$SANDBOX"' EXIT
REPO="$SANDBOX/repo"; BIN="$SANDBOX/bin"; SBROOT="$SANDBOX/root"
VENVDIR="$SANDBOX/beir-venv"; CARGOHOME="$SANDBOX/cargo-home"
RESULTS="$REPO/bench/competitor-results"
mkdir -p "$REPO/scripts/bench" "$BIN" "$SANDBOX/home" "$CARGOHOME/bin"
cp "$SCRIPT" "$REPO/scripts/bench/"

# ---- scenario stub for scripts/gather-competitors.sh (behavior via STUB_GC_MODE) ----
cat > "$REPO/scripts/gather-competitors.sh" <<'STUB'
#!/usr/bin/env bash
out="bench/competitor-results"; mkdir -p "$out"; stamp="20260101T000000Z"
lucene() { printf '{"engine": "lucene-anserini", "suite": "beir-ir-%s", "values": {}}\n' "$BEIR_CUT" > "$out/lucene-anserini-beir-ir-${BEIR_CUT}-${stamp}.json"; }
sparq()  { printf '{"engine": "sparq-text", "suite": "beir-ir-%s", "values": {}}\n'     "$BEIR_CUT" > "$out/sparq-text-beir-ir-${BEIR_CUT}-${stamp}.json"; }
case "${STUB_GC_MODE:-fail}" in
  fail)           exit 1 ;;                    # total failure: nothing written
  lucene-only)    lucene; exit 1 ;;            # sparq-text half died mid-gather
  invalid-lucene) printf '{truncated' > "$out/lucene-anserini-beir-ir-${BEIR_CUT}-${stamp}.json"; sparq; exit 0 ;;
  both)           lucene; sparq; exit 0 ;;
esac
STUB

# ---- PATH shadows: no apt/network/toolchain touches anything real -------------------
printf '#!/usr/bin/env bash\nexit 0\n' > "$BIN/apt-get"
printf '#!/usr/bin/env bash\necho "stub openjdk 21"\nexit 0\n' > "$BIN/java"
printf '#!/usr/bin/env bash\nexit 1\n' > "$BIN/curl"
printf '#!/usr/bin/env bash\nexit "${STUB_CARGO_RC:-0}"\n' > "$BIN/cargo"
cp "$BIN/cargo" "$CARGOHOME/bin/cargo"   # the script prepends $CARGO_HOME/bin later
# python3 shadow: block `-m venv` (the script then falls back to the pre-made fake
# venv, or honestly fails provisioning when there is none); exec the real python3
# for everything else (JSON validation + gather-meta.json need it).
cat > "$BIN/python3" <<PYSTUB
#!/usr/bin/env bash
if [ "\${1:-}" = "-m" ] && [ "\${2:-}" = "venv" ]; then exit 1; fi
exec "$REAL_PY" "\$@"
PYSTUB
# fake VENV python3 (installed per-scenario): pip install/show are no-op stubs and
# `import pyserini, beir` succeeds; everything else execs the real python3.
cat > "$SANDBOX/venv-python3" <<PYSTUB
#!/usr/bin/env bash
if [ "\${1:-}" = "-m" ] && [ "\${2:-}" = "pip" ]; then
  if [ "\${3:-}" = "show" ]; then printf 'Name: stub\nVersion: 0.0-stub\n'; fi
  exit 0
fi
if [ "\${1:-}" = "-c" ]; then
  case "\${2:-}" in *"import pyserini"*) exit 0 ;; esac
fi
exec "$REAL_PY" "\$@"
PYSTUB
chmod +x "$REPO/scripts/gather-competitors.sh" "$BIN"/* "$CARGOHOME/bin/cargo" "$SANDBOX/venv-python3"

run_gather() {  # run_gather <provision:yes|no> <cargo_rc> <gc_mode> <logfile> -> echoes rc
  local provision="$1" cargo_rc="$2" mode="$3" logf="$4" rc
  rm -rf "$SBROOT" "$VENVDIR" "$RESULTS" "$SANDBOX/beir-data"
  mkdir -p "$SBROOT" "$SANDBOX/beir-data"
  if [ "$provision" = "yes" ]; then
    mkdir -p "$VENVDIR/bin"
    cp "$SANDBOX/venv-python3" "$VENVDIR/bin/python3"
  fi
  set +e
  env HOME="$SANDBOX/home" PATH="$BIN:$PATH" CARGO_HOME="$CARGOHOME" \
      SENTINEL_DIR="$SBROOT" BEIR_VENV_DIR="$VENVDIR" BEIR_DATA_DIR="$SANDBOX/beir-data" \
      BEIR_CUTS="scifact" BEIR_K=100 \
      STUB_GC_MODE="$mode" STUB_CARGO_RC="$cargo_rc" \
      bash "$REPO/scripts/bench/canonical-beir-gather-instance.sh" > "$logf" 2>&1
  rc=$?
  set -e
  echo "$rc"
}

check_meta() {  # check_meta <case> <expected-status> <expected-canonical:true|false>
  if "$REAL_PY" -c '
import json, sys
m = json.load(open(sys.argv[1]))
assert m["status"] == sys.argv[2], m
assert m["canonical"] is (sys.argv[3] == "true"), m
' "$RESULTS/gather-meta.json" "$2" "$3" 2>/dev/null; then ok; else bad "$1: gather-meta.json lacks status=$2 canonical=$3"; fi
}

# ---- 1. TOTAL FAILURE: provision + build + cut all fail → failed, no GATHER_DONE ----
rc="$(run_gather no 1 fail "$SANDBOX/s1.log")"
[ "$rc" != 0 ] && ok || bad "total failure must exit nonzero (got rc=$rc)"
[ ! -e "$SBROOT/GATHER_DONE" ] && ok || bad "total failure must NOT emit the GATHER_DONE success sentinel"
[ -f "$SBROOT/GATHER_FAILED" ] && ok || bad "total failure must emit the GATHER_FAILED sentinel"
grep -q "status=failed" "$SBROOT/GATHER_FAILED" && ok || bad "GATHER_FAILED must carry status=failed"
grep -qa "gather FAILED" "$SANDBOX/s1.log" && ok || bad "total-failure console log must say 'gather FAILED'"
grep -qa "gather complete" "$SANDBOX/s1.log" && bad "total-failure log must not say 'gather complete' (launcher greps for it)" || ok
check_meta "total failure" "failed" "false"

# ---- 2. PARTIAL: lucene envelope lands, sparq-text missing → partial, nonzero -------
rc="$(run_gather yes 0 lucene-only "$SANDBOX/s2.log")"
[ "$rc" != 0 ] && ok || bad "partial gather must exit nonzero (got rc=$rc)"
[ ! -e "$SBROOT/GATHER_DONE" ] && ok || bad "partial gather must NOT emit GATHER_DONE"
grep -q "cut:scifact:sparq-text" "$SBROOT/GATHER_FAILED" 2>/dev/null && ok || bad "GATHER_FAILED must name the missing sparq-text envelope"
check_meta "partial" "partial" "false"
grep -qa "===ENVELOPE-BEGIN lucene-anserini-beir-ir-scifact" "$SANDBOX/s2.log" && ok || bad "partial evidence must still be dumped to the console for recovery"

# ---- 3. INVALID JSON: an envelope exists but does not parse → partial, nonzero ------
rc="$(run_gather yes 0 invalid-lucene "$SANDBOX/s3.log")"
[ "$rc" != 0 ] && ok || bad "invalid-JSON envelope must exit nonzero (got rc=$rc)"
[ ! -e "$SBROOT/GATHER_DONE" ] && ok || bad "invalid-JSON envelope must NOT emit GATHER_DONE"
grep -q "cut:scifact:lucene-anserini" "$SBROOT/GATHER_FAILED" 2>/dev/null && ok || bad "GATHER_FAILED must name the invalid lucene-anserini envelope"
check_meta "invalid json" "partial" "false"

# ---- 4. SUCCESS: both engines valid on every cut → GATHER_DONE, canonical=true ------
rc="$(run_gather yes 0 both "$SANDBOX/s4.log")"
[ "$rc" = 0 ] && ok || bad "validated success must exit 0 (got rc=$rc)"
[ -f "$SBROOT/GATHER_DONE" ] && ok || bad "validated success must emit GATHER_DONE"
[ ! -e "$SBROOT/GATHER_FAILED" ] && ok || bad "validated success must not emit GATHER_FAILED"
grep -qa "gather complete" "$SANDBOX/s4.log" && ok || bad "success log must say 'gather complete' (launcher greps for it)"
check_meta "success" "complete" "true"
# end-to-end recovery: the success console dump round-trips through the extractor
if [ -f "$EXTRACT" ]; then
  rm -rf "$SANDBOX/recovered"
  bash "$EXTRACT" "$SANDBOX/s4.log" "$SANDBOX/recovered" 2>/dev/null
  for n in lucene-anserini-beir-ir-scifact-20260101T000000Z.json sparq-text-beir-ir-scifact-20260101T000000Z.json; do
    if "$REAL_PY" -c 'import json, sys; json.load(open(sys.argv[1]))' "$SANDBOX/recovered/$n" 2>/dev/null; then
      ok
    else
      bad "console round-trip: $n not recovered as valid JSON"
    fi
  done
else
  bad "extractor $EXTRACT missing"
fi

echo ""
echo "test_beir_gather_sentinel: ${pass} passed, ${fail} failed."
[ "$fail" -eq 0 ] || exit 1
echo "test_beir_gather_sentinel: OK — failed/partial/invalid gathers never emit GATHER_DONE; success validates + round-trips."
