#!/usr/bin/env bash
# [FABLE-5] sq-tvzyi — INSTANCE-side canonical BEIR IR-quality gather (FTS quality axis).
#
# 🤖 SPARQ agent. Runs ON the dedicated EC2 gather box (launched by
# scripts/bench/canonical-beir-bench.sh) from a cloned sparq checkout, as root.
# Produces the BEIR quality-axis results `research/gap-fts-2026-07.md` §4 needs:
# sparq-text vs Lucene/Anserini (the kernel BM25 IR ORACLE, sq-1fz0), BOTH scored by
# the ONE shared scorer (scripts/bench-adapters/beir_ir_adapter.py) against the SAME
# BEIR cut + qrels — Recall@100 / nDCG@10 emitted as DEFICITS (smaller is better).
# GATHER-ONLY: the BEIR corpus is not redistributable in-repo, so this is a download
# step; a maintainer vendors reviewed deficits + provenance into §4 deliberately —
# results are never auto-committed.
#
# This script carries the heavy pip `pyserini`+`beir` provisioning that was NOT part
# of the wave-1 bin-packed box (scripts/bench/multi-axis-box.sh ran the latency axis
# only): JDK 21 for Anserini's Lucene, a dedicated venv, torch-CPU pulled by pip. A
# failed provision degrades to an honest NOT-RUN step log — a RE-RUN action, NEVER a
# sparq win by omission (a quality claim cannot be made either way without the run).
#
# Results: bench/competitor-results/*.json (one envelope per engine/cut, written by
# scripts/gather-competitors.sh) + $RESULTS_DIR/gather-meta.json (box provenance).
# Every envelope is ALSO cat'd into the gather log between ===ENVELOPE-BEGIN/END===
# markers so `aws ec2 get-console-output` can recover results even if the SSH pull
# path dies (envelopes are a few KB). Writes /root/GATHER_DONE when everything is on
# disk. NEVER shuts the box down — the launcher terminates it (its user-data watchdog
# is the orphan-proof backstop).
#
# Env knobs (all defaulted):
#   BEIR_CUTS="scifact"   space-separated BEIR cuts (scifact | trec-covid)
#   BEIR_K=100            top-k for Recall@k retrieval
#   BEIR_DATA_DIR=/root/beir-data   download cache for the BEIR cuts
#   PYSERINI_PIN= / BEIR_PIN=       optional pip version pins (e.g. "==1.2.0");
#                                   resolved versions are ALWAYS logged for provenance
set -uo pipefail   # NOT -e: one failed cut/provision step must never kill the gather

# [FABLE-5] DEFINE HOME/USER/LOGNAME BEFORE ANYTHING ELSE — cloud-init's root
# scripts_user context exports none of them, and under `set -u` the first bare $HOME
# aborts the gather (the sq-hmd7l.32 wave-2 stall). cargo/rustup/java/pip also read
# $HOME at runtime for their caches.
: "${HOME:=$(getent passwd "$(id -u)" 2>/dev/null | cut -d: -f6)}"
: "${HOME:=/root}"
export HOME
: "${USER:=$(id -un 2>/dev/null || echo root)}"; export USER
: "${LOGNAME:=$USER}"; export LOGNAME
export CARGO_HOME="${CARGO_HOME:-$HOME/.cargo}"
export RUSTUP_HOME="${RUSTUP_HOME:-$HOME/.rustup}"

# Mirror all output to the serial console — the only zero-dependency telemetry channel
# on a self-terminating box (`aws ec2 get-console-output`). Best-effort: fall through
# untee'd when /dev/console is not writable (a local test run).
if [ -w /dev/console ] 2>/dev/null; then
  exec > >(tee -a /dev/console) 2>&1
fi

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
cd "$ROOT"

BEIR_CUTS="${BEIR_CUTS:-scifact}"
BEIR_K="${BEIR_K:-100}"
BEIR_DATA_DIR="${BEIR_DATA_DIR:-/root/beir-data}"
PYSERINI_PIN="${PYSERINI_PIN:-}"
BEIR_PIN="${BEIR_PIN:-}"
RESULTS_DIR="$ROOT/bench/competitor-results"   # fixed by scripts/gather-competitors.sh
VENV="/root/beir-venv"

step() { echo "[STEP $(date -u +%Y-%m-%dT%H:%M:%SZ)] $*" | tee -a /root/GATHER_STEP >&2; }

mkdir -p "$RESULTS_DIR" "$BEIR_DATA_DIR"
step "canonical BEIR gather start: cuts='$BEIR_CUTS' k=$BEIR_K commit=$(git rev-parse --short HEAD 2>/dev/null || echo '?')"
df -h / >&2

# ---- 0. apt deps (self-contained; idempotent) ----------------------------------------
step "apt deps"
export DEBIAN_FRONTEND=noninteractive
apt-get update -qq || true
# openjdk-21: pyserini drives Anserini's Lucene through the JVM; python3-venv: Ubuntu
# 24.04 pip is PEP-668 externally-managed, so the heavy libs go in a dedicated venv.
apt-get install -y -qq build-essential pkg-config git curl jq unzip bc \
  python3 python3-pip python3-venv openjdk-21-jdk-headless || step "WARN: apt install returned non-zero"
export JAVA_HOME="${JAVA_HOME:-/usr/lib/jvm/java-21-openjdk-amd64}"
export PATH="$JAVA_HOME/bin:$PATH"
java -version 2>&1 | head -2 >&2 || step "WARN: java not runnable — Anserini column will degrade honestly"

# ---- 1. the heavy pip provisioning (venv: pyserini + beir) ---------------------------
step "pip provision: venv + pyserini${PYSERINI_PIN} + beir${BEIR_PIN}"
PROVISIONED=0
if python3 -m venv "$VENV" 2>/dev/null || [ -x "$VENV/bin/python3" ]; then
  export PATH="$VENV/bin:$PATH"   # gather-competitors.sh calls bare `python3`
  python3 -m pip install -q --upgrade pip || true
  python3 -m pip install -q "pyserini${PYSERINI_PIN}" "beir${BEIR_PIN}" \
    || step "WARN: pip install pyserini/beir returned non-zero — trying import anyway"
  # Some pyserini releases import faiss at module load; pull faiss-cpu iff the plain
  # import fails, then re-check (kept out of the default install to stay minimal).
  if ! python3 -c 'import pyserini, beir' 2>/dev/null; then
    step "import failed — retrying with faiss-cpu"
    python3 -m pip install -q faiss-cpu || true
  fi
  if python3 -c 'import pyserini, beir' 2>/dev/null; then
    PROVISIONED=1
    step "provision ok: $(python3 -m pip show pyserini beir 2>/dev/null | grep -E '^(Name|Version):' | tr '\n' ' ')"
  fi
fi
[ "$PROVISIONED" = 1 ] || step "WARN: pyserini/beir provisioning FAILED — the gather below will die per-cut with the adapter's own error (honest NOT-RUN, a re-run action)"

# ---- 2. rust toolchain + the sparq-text beir_text retriever --------------------------
if ! command -v cargo >/dev/null 2>&1; then
  step "rustup install"
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal || true
fi
export PATH="/root/.cargo/bin:$HOME/.cargo/bin:$PATH"
. /root/.cargo/env 2>/dev/null || . "$HOME/.cargo/env" 2>/dev/null || true

step "build sparq-text beir_text example (release)"
SPARQ_TEXT_BEIR=""
if cargo build --release -p sparq-text --example beir_text; then
  SPARQ_TEXT_BEIR="$ROOT/target/release/examples/beir_text"
  step "beir_text built: $SPARQ_TEXT_BEIR"
else
  step "WARN: beir_text build FAILED — the sparq-text side of the comparison cannot run; the oracle-only gather still records Lucene/Anserini (a re-run action for the sparq column)"
fi

# ---- 3. the gather, per cut (recipe sq-1fz0: one shared scorer, same cut + qrels) ----
for cut in $BEIR_CUTS; do
  step "gather-competitors.sh --run --only lucene-anserini (cut=$cut k=$BEIR_K)"
  if env BEIR_CUT="$cut" BEIR_K="$BEIR_K" BEIR_DATA_DIR="$BEIR_DATA_DIR" \
      ${SPARQ_TEXT_BEIR:+SPARQ_TEXT_BEIR="$SPARQ_TEXT_BEIR"} \
      bash scripts/gather-competitors.sh --run --only lucene-anserini; then
    step "cut $cut done"
  else
    step "WARN: gather exited non-zero for cut $cut — that cut stays NOT-MEASURED (a re-run action, never a sparq win)"
  fi
done

# ---- 4. box provenance for the §4 transcription --------------------------------------
step "write gather-meta.json"
IID="$(TOK=$(curl -s -m 3 -X PUT http://169.254.169.254/latest/api/token -H 'X-aws-ec2-metadata-token-ttl-seconds: 60' 2>/dev/null); \
  curl -s -m 3 -H "X-aws-ec2-metadata-token: $TOK" http://169.254.169.254/latest/meta-data/instance-id 2>/dev/null || true)"
python3 - "$RESULTS_DIR/gather-meta.json" <<META || step "WARN: gather-meta.json write failed"
import json, subprocess, sys, datetime
def sh(c):
    try: return subprocess.run(c, shell=True, capture_output=True, text=True, timeout=30).stdout.strip()
    except Exception: return ""
json.dump({
    "canonical": True, "axis": "fts-beir-ir-quality", "bead": "sq-tvzyi",
    "utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "instance_id": "${IID}" or "unknown",
    "commit": sh("git rev-parse HEAD"),
    "cuts": "${BEIR_CUTS}".split(), "k": int("${BEIR_K}"),
    "java": sh("java -version 2>&1 | head -1"),
    "pyserini": sh("python3 -m pip show pyserini 2>/dev/null | grep ^Version:"),
    "beir": sh("python3 -m pip show beir 2>/dev/null | grep ^Version:"),
}, open(sys.argv[1], "w"), indent=1)
META

# ---- 5. console-output backstop + sentinel -------------------------------------------
step "envelopes in $RESULTS_DIR:"
ls -la "$RESULTS_DIR" >&2 || true
for f in "$RESULTS_DIR"/*.json; do
  [ -f "$f" ] || continue
  echo "===ENVELOPE-BEGIN $(basename "$f")==="
  cat "$f"
  echo "===ENVELOPE-END==="
done
df -h / >&2
step "gather complete"
touch /root/GATHER_DONE
