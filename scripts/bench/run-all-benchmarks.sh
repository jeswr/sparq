#!/usr/bin/env bash
# run-all-benchmarks.sh — resilient run-EVERYTHING benchmark orchestrator (bead sq-hz0g2).
#
# MAINTAINER PRIORITY: agents on this box regularly hit usage limits mid-session. This
# script runs the whole benchmark estate with per-suite isolation and streams results
# incrementally to a folder ON THIS MACHINE as each suite completes — so a mid-run kill
# (usage limit, shutdown, Ctrl-C) loses at most the in-flight suite; everything already
# finished is on disk for the next session.
#
# Catalog: every suite is enumerated in the CATALOG table below (--list prints it), each
# mapped to its bench/benchmarks.toml registry id(s). Suites whose external dependency is
# missing (QLever, EYE, nargo/bb, a GPU, an EC2 budget, an LLM agent) are SKIPPED with a
# recorded reason, never silently dropped. One red suite never kills the run.
#
# Results: ~/sparq-bench-results/<UTC-timestamp>-<git-sha>/
#   manifest.json      host + commit + toolchain + per-suite status table
#                      (re-written ATOMICALLY after EVERY suite)
#   suites/<id>.json   one machine-readable result per suite
#   suites/<id>.md     one human summary per suite (status + log tail)
#   suites/<id>.log    full stdout+stderr
#   suites/<id>.d/     extra artifacts a suite chooses to drop (via $SUITE_OUT)
#
# HONESTY: every result file is stamped NON-CANONICAL. This work box is shared and
# frequently busy — wall-clock numbers here are trend-only (see the QUIET-BOX convention
# in bench/CATALOG.md). Deterministic gates (counts, gate-counts, bytes, pass-rates) are
# load-robust and remain meaningful.
#
# EC2 mode (--remote): PREPARED, NOT LAUNCHED. The quota is currently not fixed; a bare
# `--remote` prints exactly what a launch would do (dry-run) and exits. A real launch
# additionally requires EXECUTE=1 in the environment. The launch follows the repo's EC2
# bench protocol (scripts/ec2-bench.sh + the EC2-benchmark feedback): purpose=sparq-bench
# tag, orphan-proof --instance-initiated-shutdown-behavior=terminate + a user-data
# shutdown watchdog + remote self-shutdown on completion, ephemeral keypair/SG, never
# touches prod/dev boxes, and streams each suite's results back to the SAME local folder
# as it completes (rsync poll — per-suite, not end-of-run).
#
# Usage:
#   scripts/bench/run-all-benchmarks.sh [--list] [--dry-run]
#       [--tier fast|standard|heavy] [--only id,id] [--skip id,id]
#       [--out DIR] [--timeout-mult N]
#       [--remote [--tier T]]        # dry-run unless EXECUTE=1
#
# Env knobs: SPARQ_ROOT (repo checkout to benchmark; default = this script's repo),
#   BENCH_RESULTS_DIR (default ~/sparq-bench-results), KEEP_SCRATCH=1,
#   AWS_REGION / BENCH_INSTANCE_TYPE / REMOTE_MAX_MINUTES (remote mode).
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="${SPARQ_ROOT:-$(cd "$SCRIPT_DIR/../.." && pwd)}"
CLI="$ROOT/target/release/sparq-cli"
GEN="$ROOT/target/release/sparq-bench"
OUT_BASE="${BENCH_RESULTS_DIR:-$HOME/sparq-bench-results}"
SCRATCH_BASE="/tmp/sparq-benchall"
NON_CANONICAL_NOTE="NON-CANONICAL: produced on a shared work box (not a quiet dedicated runner); wall-clock numbers are trend-only — see the QUIET-BOX convention in bench/CATALOG.md"

export ROOT CLI GEN

# ---------------------------------------------------------------------------
# Catalog: id|tier|timeout_s|min_free_gb|registry_ids|description
#   tier: fast (< ~2 min each once built) | standard | heavy | external
#   external = cannot run unattended on this box; ALWAYS skipped with a recorded
#   reason (the reason documents how to run it manually).
# Registry ids reference bench/benchmarks.toml (the machine-readable catalog).
# ---------------------------------------------------------------------------
CATALOG=(
  "build-core|fast|5400|8|-|release build of sparq-cli + sparq-bench (prerequisite for most suites)"
  "competitor-dry|fast|300|1|competitor-gather|competitor gather DRY-RUN: tool/version/env report (runs no benchmark, pulls no image)"
  "deep-taxonomy|fast|1800|2|deep-taxonomy|DeepTaxonomy N3 closure gate, per-commit depths (dt1k+dt10k), self-asserting"
  "owl-sameas|fast|1800|2|owl-sameas|OWL sameAs equality micro-suite, per-commit tiers (N=8+32), self-asserting"
  "fuzz-smoke|fast|1800|2|sparq-bench-fuzz|differential fuzz vs Oxigraph, seeds 0..2000, all categories (deterministic)"
  "fts|fast|2700|2|text-index-bench|full-text BM25 suite (synthetic in-process corpus), self-asserting"
  "rsp-oracle|fast|2700|2|rsp-ql|RSP-QL clock-free replay oracle (3 EvalModes + SRBench join), self-asserting"
  "hdt-suite|fast|2700|2|hdt-suite|HDT load-and-decode oracle on vendored snikmeta.hdt, self-asserting"
  "geo-bench|fast|2700|3|geo-bench|GeoSPARQL validation suite (~100k points, counts-not-coordinates), self-asserting"
  "reason-el|fast|2700|2|reason-el-classify|EL classifier bench (chain+cr4, closed-form self-asserting gates)"
  "reason-ql|fast|2700|2|reason-ql-rewrite|QL PerfectRef rewrite bench (closed-form UCQ-size gates)"
  "sparq-bench-compare|standard|3600|4|sparq-bench-compare|sparq vs Oxigraph differential + perf, scale 50k entities, min-of-4"
  "operator-coverage|standard|2700|2|operator-coverage|per-operator latency suite (count/materialize/json)"
  "sp2b|standard|3600|4|sp2b|SP2Bench 250k corpus (real Freiburg generator, g++), 14 per-commit queries"
  "dbpsb|standard|3600|6|dbpsb|DBPSB/FEASIBLE 750k DBpedia cut (network fetch, sha256-pinned), 13 queries"
  "watdiv|standard|3600|4|watdiv|WatDiv SF=1 (real Waterloo generator, g+++Boost), self-asserting run.sh"
  "bsbm|standard|3600|4|bsbm|BSBM Explore mix -pc 300 (bsbmtools, JRE+unzip+network), self-asserting run.sh"
  "lubm|standard|3600|4|lubm|LUBM(1) reasoning suite (javac+rapper), OWL-RL closure, both tiers, self-asserting"
  "shacl|standard|3600|4|shacl-validate-bench|SHACL validation over LUBM(1) ABox x 5 shapes (javac+rapper), self-asserting"
  "vector-ann|standard|3600|4|vector-ann-bench|vector/ANN recall@10-deficit gate (HNSW/Vamana/PQ, 50k x 32), self-asserting"
  "selective-bindjoin|standard|2700|4|selective-bindjoin|selective bind-join vs merge-join probe (500k synthetic)"
  "u64-valueids|standard|2700|4|u64-valueids|u64 value-id FILTER/BIND/GROUP/ORDER paths (1M literals)"
  "ingest-index|standard|3600|10|cli-ingest, cli-save-build, cli-probe-compress, cli-compare-compress, cli-bench-mmap|ingest + save + probe-compress + compare-compress + bench-mmap over one synthetic corpus"
  "bench-remap|standard|3600|6|cli-bench-remap|dictionary-id remap gather (box-scaled 5M triples / 12.5M dict), prefetch on+off"
  "parse-baseline|standard|3600|8|parse-baseline|parse throughput baseline (bench/parse standalone project) on a synthetic NT corpus"
  "hdt-stage-split|standard|3600|6|hdt-load-bench, hdt-stage-split|HDT direct-decode stage split + NT A/B (bench/parse gen-hdt/bench-hdt)"
  "dict-baseline|standard|3600|6|dict-baseline|dict bytes/term + build throughput (bench/dict standalone project)"
  "owl-bench|standard|2700|2|inference-owl-bench|OWL closure bench (rdfs-instances / owl-route-rdfs / transitive / restrictions)"
  "incremental-reason|standard|2700|4|inference-incremental|incremental maintenance vs re-materialization (olympics corpus)"
  "solid-wac|standard|2700|2|solid-wac-bench|Solid WAC auth-view materialization + per-query cost"
  "policy-odrl|standard|2700|2|policy-odrl-eval|ODRL policy parse + per-request evaluation latency"
  "serve-core|standard|2700|2|serve-core-bench|sparq-serve core example bench"
  "serve-throughput-smoke|standard|2700|2|serve-throughput|canonical loopback HTTP throughput harness, --smoke profile"
  "mcp-roundtrip|standard|2700|2|mcp-dispatch-overhead|MCP dispatch round-trip overhead"
  "nlq-offline|standard|2700|2|nlq-offline-bench|NLQ offline bench (no network, no live feature)"
  "mpc-matrix|standard|3600|2|mpc-bench-matrix|MPC cost matrix (default build: matrix only, correctness gates skipped)"
  "algos|standard|2700|2|sparq-algos|graph analytics (PageRank/centrality/community) micro-bench"
  "geo-report|standard|2700|2|geo-index-bench|geo index build/query latency report (bench_geo report mode)"
  "vectors-throughput|standard|3600|2|vectors-throughput|vector store put/finalize/open + exact vs HNSW query throughput"
  "criterion-fedplan|standard|3600|2|-|criterion bench: sparq-fedplan (crates/sparq-fedplan/benches)"
  "criterion-substrate|standard|3600|2|-|criterion bench: sparq-substrate (crates/sparq-substrate/benches)"
  "wasm-bundle|standard|3600|3|wasm-bundle|wasm bundle size (wasm32-unknown-unknown release build + byte count)"
  "site-bundle|standard|3600|3|-|site bundle guard: npm ci + next build (postbuild runs scripts/check-bundle.mjs)"
  "sparql-conformance|standard|3600|3|sparql-conformance|W3C SPARQL conformance (fetch + run; correctness, not perf)"
  "inference-conformance|standard|3600|3|inference-conformance|W3C reasoning conformance (fetch + run; correctness, not perf)"
  "zk-gate-counts|standard|5400|3|zk-compose-gates|zk circuit gate-count snapshot (nargo + bb; deterministic)"
  "zk-prove-verify|standard|5400|3|zk-compose-prove-verify|zk prove+verify one circuit member (filter_int_d1; nargo + bb)"
  "sim-olympics|standard|2700|4|sim-olympics-eval|sparq-sim olympics evaluation (needs fetched olympics.nt)"
  "introspect-olympics|standard|2700|4|introspect-olympics|sparq-introspect olympics run (needs fetched olympics.nt)"
  "ci-bench|heavy|10800|12|ci-bench|the full per-commit CI emitter (scripts/ci-bench.sh 200000) — many sub-suites, one JSON"
  "hw-bench|heavy|10800|10|hw-bench|hardware sweep (scripts/hw-bench.sh 500000)"
  "cli-scaling|heavy|10800|8|cli-scaling|thread-count scaling sweep (10M synthetic, threads 1,2,4,8)"
  "zk-commit-bench|heavy|10800|6|zk-commit-throughput|zk commitment pipeline criterion bench (bench/zk standalone project)"
  "zk-trace-bench|heavy|10800|6|zk-trace-overhead|zk trace seam overhead criterion bench (bench/zk-trace standalone project)"
  "eye-comparison|heavy|10800|4|inference-eye-comparison|N3 closure vs EYE (needs EYE binary; dt100k excluded by default)"
  "kge-ablation|heavy|14400|4|kge-ablation|KGE link-prediction ablation matrix (synthetic dataset; trains embeddings)"
  "serve-throughput-full|heavy|3600|2|serve-throughput|canonical loopback HTTP throughput harness, full profile"
  "gpu-bench|heavy|3600|2|gpu-bench|GPU bench (skipped when no GPU adapter is visible)"
  "qlever-olympics|external|0|0|qlever-olympics|needs a QLever install + index/server dance — run per bench/qlever-olympics/README.md"
  "qlever-synthetic-10m|external|0|0|qlever-synthetic-10m|needs QLever — run per bench/qlever-synthetic/README.md"
  "qlever-synthetic-100m|external|0|0|qlever-synthetic-100m|needs QLever + large disk — run per bench/qlever-100m docs"
  "wikidata-8b|external|0|0|wikidata-8b|external-cost EC2 build, budget- and dict-spill-gated — bench/wikidata-8b/RUNBOOK.md paragraph 0"
  "pss-update-parity|external|0|0|pss-update-parity|needs a writable QLever server + access token — bench/pss-update-set/compare.py"
  "competitor-gather-real|external|0|0|competitor-gather|real competitor gather is maintainer-reviewed: scripts/gather-competitors.sh --run --only <id>"
  "serve-spikes|external|0|0|serve-spikes|research spikes, not maintained regression benchmarks (some bins carry API drift)"
  "memtier-spikes|external|0|0|memtier-spikes|research spikes, not maintained regression benchmarks"
  "fo-km|external|0|0|fo-km|LLM-agent KM-task A/B — needs model access, not a machine benchmark"
  "pkg-dogfood|external|0|0|pkg-dogfood|LLM token A/B over real agent transcripts — needs agent runs"
  "terse|external|0|0|terse|LLM token A/B (terse dialect) — needs agent runs"
  "ci-bench-ec2|external|0|0|ci-bench-ec2|use --remote (this script) or .github/workflows/bench-ec2.yml"
  "sparq-arrow|external|0|0|sparq-arrow|registry status=planned: export-rate micro-bench not built yet (correctness suite runs in CI)"
)

have() { command -v "$1" >/dev/null 2>&1; }
free_gb() { df -BG --output=avail "${1:-/tmp}" 2>/dev/null | tail -1 | tr -dc '0-9'; }

# --- per-suite dependency checks: echo a skip reason (and return 0) if unmet ---
suite_skip_reason() {
  local id="$1"
  case "$id" in
    build-core) have cargo || echo "cargo not on PATH" ;;
    competitor-dry) [ -x "$ROOT/scripts/gather-competitors.sh" ] || echo "scripts/gather-competitors.sh missing" ;;
    deep-taxonomy|owl-sameas|owl-bench)
      have python3 || { echo "python3 missing"; return; }
      [ -x "$CLI" ] || echo "sparq-cli not built (run the build-core suite first)" ;;
    fuzz-smoke|sparq-bench-compare) have cargo || echo "cargo not on PATH" ;;
    operator-coverage|selective-bindjoin|u64-valueids|ingest-index|bench-remap|cli-scaling)
      [ -x "$CLI" ] || { echo "sparq-cli not built (run the build-core suite first)"; return; }
      case "$id" in operator-coverage|ingest-index|cli-scaling)
        [ -x "$GEN" ] || echo "sparq-bench not built (run the build-core suite first)" ;; esac ;;
    sp2b) have g++ || echo "g++ missing (SP2Bench generator build)"
          [ -x "$CLI" ] || echo "sparq-cli not built (run the build-core suite first)" ;;
    dbpsb) have curl || echo "curl missing (DBpedia slice fetch)"
           [ -x "$CLI" ] || echo "sparq-cli not built (run the build-core suite first)" ;;
    watdiv) have g++ || { echo "g++ missing (WatDiv generator build)"; return; }
            [ -f /usr/include/boost/version.hpp ] || echo "Boost headers missing (WatDiv generator needs Boost)" ;;
    bsbm) have java || echo "JRE missing (bsbmtools)"
          have unzip || echo "unzip missing (bsbmtools distribution)" ;;
    lubm|shacl) have javac || echo "javac missing (LUBM UBA generator)"
                have rapper || echo "rapper missing (RDF/XML -> NT for LUBM)" ;;
    incremental-reason|sim-olympics|introspect-olympics)
      [ -f "$ROOT/bench/qlever-olympics/olympics.nt" ] || echo "bench/qlever-olympics/olympics.nt not fetched (see bench/qlever-olympics/README.md)" ;;
    wasm-bundle)
      rustup target list --installed 2>/dev/null | grep -q wasm32-unknown-unknown || echo "wasm32-unknown-unknown target not installed (rustup target add wasm32-unknown-unknown)" ;;
    site-bundle) have npm || echo "npm missing (site build)" ;;
    zk-gate-counts|zk-prove-verify)
      have nargo || echo "nargo not on PATH (Noir toolchain; see skills/noir-circuit-patterns)"
      have bb || echo "bb not on PATH (Barretenberg; see skills/noir-circuit-patterns)" ;;
    eye-comparison)
      { have eye || [ -x "$HOME/.local/bin/eye" ]; } || echo "EYE reasoner not installed (needed for the head-to-head)" ;;
    gpu-bench)
      { have nvidia-smi || ls /dev/dri >/dev/null 2>&1; } || echo "no GPU adapter visible (nvidia-smi and /dev/dri both absent)" ;;
    serve-throughput-smoke|serve-throughput-full)
      [ -x "$ROOT/scripts/serve-throughput-bench.sh" ] || echo "scripts/serve-throughput-bench.sh missing" ;;
    qlever-*|wikidata-8b|pss-update-parity|competitor-gather-real|serve-spikes|memtier-spikes|fo-km|pkg-dogfood|terse|ci-bench-ec2|sparq-arrow)
      echo "external/manual suite — see catalog description" ;;
  esac
  # first non-empty line wins (some checks emit two reasons)
}

# --- per-suite command: echoed as a bash -c string, run from $ROOT with
#     $CLI/$GEN/$SCRATCH/$SUITE_OUT exported. External suites have none. ---
suite_cmd() {
  case "$1" in
    build-core) echo 'cargo build --release -p sparq-cli -p sparq-bench' ;;
    competitor-dry) echo 'scripts/gather-competitors.sh --list && scripts/gather-competitors.sh' ;;
    deep-taxonomy) echo 'CLI="$CLI" bench/deep-taxonomy/run.sh' ;;
    owl-sameas) echo 'CLI="$CLI" bench/owl-sameas/run.sh' ;;
    fuzz-smoke) echo 'cargo run -p sparq-bench --release -- fuzz 0 2000 all' ;;
    fts) echo 'cargo build --release -p sparq-text --example bench_text && bench/fts/run.sh' ;;
    rsp-oracle) echo 'cargo build --release -p sparq-rsp --example rsp_oracle && bench/rsp/run.sh' ;;
    hdt-suite) echo 'cargo build --release -p sparq-hdt --example bench_oracle && bench/hdt/run.sh' ;;
    geo-bench) echo 'cargo build --release -p sparq-geo --example bench_geo && bench/geo/run.sh' ;;
    reason-el) echo 'cargo run -p sparq-reason-el --example classify_bench --release' ;;
    reason-ql) echo 'cargo run -p sparq-reason-ql --example ql_rewrite_bench --release --features experimental' ;;
    sparq-bench-compare) echo 'cargo run -p sparq-bench --release -- --scale 50000 --iters 4' ;;
    operator-coverage) echo '"$GEN" dump 2000 "$SCRATCH/op.nt" && for m in count materialize json; do "$CLI" bench "$SCRATCH/op.nt" ntriples bench/operators/queries 3 "$m"; done' ;;
    sp2b) echo 'CORPUS=$(bench/sp2b/gen.sh 250000) && "$CLI" bench "$CORPUS" turtle bench/sp2b/queries 3 count' ;;
    dbpsb) echo 'CUT=$(bench/dbpsb/fetch.sh 750000) && "$CLI" bench "$CUT" ntriples bench/dbpsb/queries 3 count' ;;
    watdiv) echo 'bench/watdiv/run.sh 1' ;;
    bsbm) echo 'bench/bsbm/run.sh' ;;
    lubm) echo 'bench/lubm/run.sh' ;;
    shacl) echo 'bench/shacl/run.sh' ;;
    vector-ann) echo 'cargo build --release -p sparq-vectors --example bench_vectors --features approx-ann && bench/vector/run.sh' ;;
    selective-bindjoin) echo 'python3 bench/selective/gen.py 500000 > "$SCRATCH/selective.nt" && "$CLI" bench "$SCRATCH/selective.nt" ntriples bench/selective/queries 3 count' ;;
    u64-valueids) echo 'python3 bench/u64-valueids/gen.py 1000000 "$SCRATCH/t3-literals.nt" && "$CLI" bench "$SCRATCH/t3-literals.nt" ntriples bench/u64-valueids/queries 3 materialize' ;;
    ingest-index) echo '"$GEN" dump 100000 "$SCRATCH/data.nt" && "$CLI" ingest "$SCRATCH/data.nt" full && "$CLI" save "$SCRATCH/data.nt" ntriples "$SCRATCH/idx" && "$CLI" probe-compress "$SCRATCH/idx/spo.perm" && "$CLI" compare-compress "$SCRATCH/data.nt" ntriples && "$CLI" bench-mmap "$SCRATCH/idx" bench/qlever-synthetic/queries 3 count' ;;
    bench-remap) echo '"$CLI" bench-remap 5000000 12500000 3 && SPARQ_NO_PREFETCH=1 "$CLI" bench-remap 5000000 12500000 3' ;;
    parse-baseline) echo '(cd bench/parse && cargo build --release) && "$GEN" dump 200000 "$SCRATCH/parse.nt" && bench/parse/target/release/parse-baseline bench-nt "$SCRATCH/parse.nt"' ;;
    hdt-stage-split) echo '(cd bench/parse && cargo build --release) && "$GEN" dump 125000 "$SCRATCH/h.nt" && bench/parse/target/release/parse-baseline gen-hdt "$SCRATCH/h.nt" "$SCRATCH/h.hdt" && bench/parse/target/release/parse-baseline bench-hdt "$SCRATCH/h.hdt"' ;;
    dict-baseline) echo '(cd bench/dict && cargo build --release) && bench/dict/target/release/dict-baseline gen 200000 "$SCRATCH/dictdata" && bench/dict/target/release/dict-baseline bench "$SCRATCH/dictdata/wikidata.nt"' ;;
    owl-bench) echo 'SPARQ_CLI="$CLI" bench/inference/owl-bench.sh' ;;
    incremental-reason) echo 'cargo run -p sparq-reason --example incremental_olympics_bench --release' ;;
    solid-wac) echo 'cargo run -p sparq-solid --example bench --release' ;;
    policy-odrl) echo 'cargo run -p sparq-policy --example bench --release' ;;
    serve-core) echo 'cargo run -p sparq-serve --example bench --release' ;;
    serve-throughput-smoke) echo 'scripts/serve-throughput-bench.sh --smoke --json "$SUITE_OUT/serve-throughput.json"' ;;
    serve-throughput-full) echo 'scripts/serve-throughput-bench.sh --json "$SUITE_OUT/serve-throughput.json"' ;;
    mcp-roundtrip) echo 'cargo run -p sparq-mcp --example mcp_roundtrip_bench --release' ;;
    nlq-offline) echo 'cargo run -p sparq-nlq --example bench --release' ;;
    mpc-matrix) echo 'cargo run -p sparq-mpc --release --example mpc_bench_matrix' ;;
    algos) echo 'cargo run -p sparq-algos --release --example bench_algos -- 50000 8' ;;
    geo-report) echo 'cargo run --release -p sparq-geo --example bench_geo' ;;
    vectors-throughput) echo 'cargo test -p sparq-vectors --release --test throughput -- --ignored --nocapture' ;;
    criterion-fedplan) echo 'cargo bench -p sparq-fedplan' ;;
    criterion-substrate) echo 'cargo bench -p sparq-substrate' ;;
    wasm-bundle) echo 'cargo build --release -p sparq-wasm --target wasm32-unknown-unknown && printf "wasm_bundle_bytes\t%s\n" "$(wc -c < target/wasm32-unknown-unknown/release/sparq_wasm.wasm)"' ;;
    site-bundle) echo 'cd site && npm ci --no-audit --no-fund && npm run build' ;;
    sparql-conformance) echo 'scripts/fetch-conformance.sh && cargo run --release -p sparq-conformance' ;;
    inference-conformance) echo 'scripts/fetch-inference-suites.sh && cargo run --release -p sparq-conformance --bin sparq-inference-conformance' ;;
    zk-gate-counts) echo 'bench/zk-compose/scripts/gate_counts.sh | tee "$SUITE_OUT/gate_counts_latest.json"' ;;
    zk-prove-verify) echo 'bench/zk-compose/scripts/prove_verify.sh filter_int_d1' ;;
    sim-olympics) echo 'cargo run --release -p sparq-sim --example olympics_eval' ;;
    introspect-olympics) echo 'cargo run -p sparq-introspect --example olympics_introspect --release' ;;
    ci-bench) echo 'cargo build --release -p sparq-cli -p sparq-bench && bash scripts/ci-bench.sh 200000 "$SUITE_OUT/ci-bench-results.json"' ;;
    hw-bench) echo 'cargo build --release -p sparq-cli -p sparq-bench && bash scripts/hw-bench.sh 500000 "$SUITE_OUT/hw-bench-results.csv"' ;;
    cli-scaling) echo '"$GEN" dump 1250000 "$SCRATCH/scaling.nt" && "$CLI" scaling "$SCRATCH/scaling.nt" ntriples bench/qlever-synthetic/queries 1,2,4,8 3' ;;
    zk-commit-bench) echo 'cd bench/zk && cargo bench' ;;
    zk-trace-bench) echo 'cd bench/zk-trace && cargo bench' ;;
    eye-comparison) echo 'EYE="${EYE:-$(command -v eye || echo "$HOME/.local/bin/eye")}" SPARQ_CLI="$CLI" bench/inference/eye-comparison.sh' ;;
    kge-ablation) echo 'cargo build -p sparq-vectors --release --features kge --example kge_ablation && target/release/examples/kge_ablation' ;;
    gpu-bench) echo 'cargo run --release -p sparq-gpu --example gpu_bench' ;;
    *) : ;;  # external suites: no runnable command by design
  esac
}

# ---------------------------------------------------------------------------
# result persistence (incremental, atomic)
# ---------------------------------------------------------------------------
write_header() {
  python3 - "$OUT_DIR" <<'PY'
import json, os, platform, subprocess, sys, datetime
out = sys.argv[1]
def run(cmd):
    try: return subprocess.run(cmd, capture_output=True, text=True, timeout=30).stdout.strip()
    except Exception: return ""
root = os.environ["ROOT"]
hdr = {
    "run_id": os.environ["RUN_ID"],
    "canonical": False,
    "non_canonical_note": os.environ["NON_CANONICAL_NOTE"],
    "started_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "args": os.environ.get("RUN_ARGS", ""),
    "host": {
        "hostname": platform.node(),
        "kernel": platform.release(),
        "machine": platform.machine(),
        "cpus": os.cpu_count(),
        "mem_gb": round(os.sysconf("SC_PAGE_SIZE") * os.sysconf("SC_PHYS_PAGES") / 2**30, 1),
    },
    "git": {
        "sha": run(["git", "-C", root, "rev-parse", "HEAD"]),
        "branch": run(["git", "-C", root, "rev-parse", "--abbrev-ref", "HEAD"]),
        "dirty": bool(run(["git", "-C", root, "status", "--porcelain"])),
        "root": root,
    },
    "toolchain": {
        "rustc": run(["rustc", "-V"]),
        "cargo": run(["cargo", "-V"]),
        "python3": run(["python3", "-V"]),
        "node": run(["node", "-v"]),
        "nargo": run(["nargo", "--version"]).splitlines()[0] if run(["nargo", "--version"]) else "",
        "bb": run(["bb", "--version"]),
    },
}
tmp = os.path.join(out, ".header.json.tmp")
with open(tmp, "w") as f: json.dump(hdr, f, indent=2)
os.replace(tmp, os.path.join(out, "header.json"))
PY
}

# record_suite: reads RS_* env vars, writes suites/<id>.json + .md, rebuilds manifest.json
record_suite() {
  python3 - "$OUT_DIR" <<'PY'
import json, os, sys, datetime, glob
out = sys.argv[1]
sdir = os.path.join(out, "suites"); os.makedirs(sdir, exist_ok=True)
e = os.environ
sid = e["RS_ID"]
rec = {
    "id": sid,
    "order": int(e["RS_ORDER"]),
    "tier": e["RS_TIER"],
    "status": e["RS_STATUS"],            # pass | fail | timeout | skipped | running
    "skip_reason": e.get("RS_REASON", ""),
    "exit_code": int(e["RS_RC"]) if e.get("RS_RC", "") != "" else None,
    "started_utc": e.get("RS_START", ""),
    "ended_utc": e.get("RS_END", ""),
    "duration_s": float(e["RS_DUR"]) if e.get("RS_DUR", "") != "" else None,
    "timeout_s": int(e["RS_TIMEOUT"]) if e.get("RS_TIMEOUT", "") != "" else None,
    "command": e.get("RS_CMD", ""),
    "log": f"suites/{sid}.log" if os.path.exists(os.path.join(sdir, f"{sid}.log")) else "",
    "registry_ids": [t.strip() for t in e.get("RS_REGISTRY", "").split(",") if t.strip() and t.strip() != "-"],
    "description": e.get("RS_DESC", ""),
    "canonical": False,
    "non_canonical_note": e["NON_CANONICAL_NOTE"],
}
tmp = os.path.join(sdir, f".{sid}.json.tmp")
with open(tmp, "w") as f: json.dump(rec, f, indent=2)
os.replace(tmp, os.path.join(sdir, f"{sid}.json"))

# human summary (.md) — header stamps non-canonical provenance
logpath = os.path.join(sdir, f"{sid}.log")
tail = ""
if os.path.exists(logpath):
    with open(logpath, errors="replace") as f:
        lines = f.readlines()
    tail = "".join(lines[-80:])
md = [
    f"# suite: {sid}",
    "",
    f"> **{e['NON_CANONICAL_NOTE']}**",
    "",
    f"- status: **{rec['status']}**" + (f" ({rec['skip_reason']})" if rec["skip_reason"] else ""),
    f"- tier: {rec['tier']}   registry: {', '.join(rec['registry_ids']) or '(n/a)'}",
    f"- started: {rec['started_utc']}   ended: {rec['ended_utc']}   duration_s: {rec['duration_s']}",
    f"- exit_code: {rec['exit_code']}   timeout_s: {rec['timeout_s']}",
    f"- command: `{rec['command']}`" if rec["command"] else "- command: (none — external/manual suite)",
    f"- description: {rec['description']}",
]
if tail:
    md += ["", "## log tail (last 80 lines)", "", "```", tail.rstrip("\n"), "```"]
tmp = os.path.join(sdir, f".{sid}.md.tmp")
with open(tmp, "w") as f: f.write("\n".join(md) + "\n")
os.replace(tmp, os.path.join(sdir, f"{sid}.md"))

# rebuild manifest.json from header + all suite records (atomic — crash loses <= 1 suite)
hdr = {}
hp = os.path.join(out, "header.json")
if os.path.exists(hp):
    hdr = json.load(open(hp))
suites = []
for p in glob.glob(os.path.join(sdir, "*.json")):
    try: suites.append(json.load(open(p)))
    except Exception: pass
suites.sort(key=lambda r: r.get("order", 0))
counts = {}
for r in suites:
    counts[r["status"]] = counts.get(r["status"], 0) + 1
man = dict(hdr)
man["updated_utc"] = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
man["counts"] = counts
man["suites"] = suites
tmp = os.path.join(out, ".manifest.json.tmp")
with open(tmp, "w") as f: json.dump(man, f, indent=2)
os.replace(tmp, os.path.join(out, "manifest.json"))
PY
}

# ---------------------------------------------------------------------------
# runner
# ---------------------------------------------------------------------------
run_suite() {
  local row="$1" order="$2"
  local id tier tmo min_gb registry desc
  IFS='|' read -r id tier tmo min_gb registry desc <<<"$row"
  local cmd; cmd="$(suite_cmd "$id")"
  local reason=""

  export RS_ID="$id" RS_ORDER="$order" RS_TIER="$tier" RS_TIMEOUT="" RS_CMD="$cmd" \
         RS_REGISTRY="$registry" RS_DESC="$desc" RS_RC="" RS_START="" RS_END="" RS_DUR="" RS_REASON=""

  # selection / dependency / disk gates -> skip with a recorded reason
  if [ -n "${SKIP_MAP[$id]:-}" ]; then reason="${SKIP_MAP[$id]}"
  elif reason="$(suite_skip_reason "$id" | head -1)" && [ -n "$reason" ]; then :
  elif [ -z "$cmd" ]; then reason="external/manual suite — see catalog description"
  else
    local free; free="$(free_gb /tmp)"
    if [ -n "$free" ] && [ "$free" -lt "$min_gb" ]; then
      reason="insufficient disk: ${free}G free < ${min_gb}G required (disk-safety gate)"
    fi
  fi
  if [ -n "$reason" ]; then
    RS_STATUS="skipped" RS_REASON="$reason" record_suite
    printf '%-24s SKIP  %s\n' "$id" "$reason"
    return 0
  fi

  local eff_tmo=$(( tmo * TIMEOUT_MULT ))
  export RS_TIMEOUT="$eff_tmo"
  export SCRATCH="$SCRATCH_BASE/$id" SUITE_OUT="$SUITES_DIR/$id.d"
  mkdir -p "$SCRATCH" "$SUITE_OUT"

  # mark running BEFORE starting: an interrupted run leaves an honest record
  RS_STATUS="running" record_suite
  printf '%-24s RUN   (timeout %ss)\n' "$id" "$eff_tmo"
  export RS_START; RS_START="$(date -u +%FT%TZ)"
  local t0; t0=$(date +%s)
  timeout --kill-after=30 "$eff_tmo" bash -o pipefail -c "cd \"\$ROOT\" && { $cmd; }" \
    >"$SUITES_DIR/$id.log" 2>&1
  local rc=$?
  local t1; t1=$(date +%s)
  export RS_END; RS_END="$(date -u +%FT%TZ)"
  export RS_RC="$rc" RS_DUR="$(( t1 - t0 ))"
  local status
  case "$rc" in
    0) status="pass" ;;
    124|137) status="timeout" ;;
    *) status="fail" ;;
  esac
  RS_STATUS="$status" record_suite
  printf '%-24s %-5s rc=%s %ss\n' "$id" "$(echo "$status" | tr a-z A-Z)" "$rc" "$(( t1 - t0 ))"
  [ "$status" = "pass" ] || FAILED_SUITES+=("$id:$status")
  [ "${KEEP_SCRATCH:-0}" = "1" ] || rm -rf "$SCRATCH"
  rmdir "$SUITE_OUT" 2>/dev/null || true   # drop the artifacts dir if the suite left nothing
  return 0
}

list_catalog() {
  printf '%-24s %-9s %8s %7s  %s\n' "SUITE" "TIER" "TIMEOUT" "MIN-GB" "DESCRIPTION"
  local row id tier tmo min_gb registry desc
  for row in "${CATALOG[@]}"; do
    IFS='|' read -r id tier tmo min_gb registry desc <<<"$row"
    printf '%-24s %-9s %8s %7s  %s\n' "$id" "$tier" "$tmo" "$min_gb" "$desc"
    [ "$registry" != "-" ] && printf '%-24s %-9s %8s %7s    registry: %s\n' "" "" "" "" "$registry"
  done
  echo
  echo "catalog: ${#CATALOG[@]} suites; registry: bench/benchmarks.toml (75 entries); human guide: bench/CATALOG.md"
}

# ---------------------------------------------------------------------------
# EC2 remote mode — PREPARED, NOT LAUNCHED (quota not fixed). Dry-run default;
# a real launch requires EXECUTE=1. Follows scripts/ec2-bench.sh + the EC2
# benchmark protocol: purpose=sparq-bench tag, orphan-proof shutdown-behavior
# terminate + user-data watchdog + remote self-shutdown, ephemeral keypair/SG,
# never touches existing (prod/dev) instances, per-suite result streaming.
# ---------------------------------------------------------------------------
remote_mode() {
  local region="${AWS_REGION:-eu-west-2}"
  local itype="${BENCH_INSTANCE_TYPE:-c7g.4xlarge}"
  local max_min="${REMOTE_MAX_MINUTES:-720}"
  local sha; sha="$(git -C "$ROOT" rev-parse HEAD)"
  local repo_url
  repo_url="${BENCH_REPO_URL:-$(git -C "$ROOT" remote get-url origin 2>/dev/null | sed -E 's#^git@github\.com:#https://github.com/#; s#\.git$##')}"
  [ -n "$repo_url" ] || repo_url="https://github.com/sparq-org/sparq"
  cat <<PLAN
== --remote plan (EC2 quota not fixed — DRY-RUN; a real launch needs EXECUTE=1) ==
 region / type      : $region / $itype (spot), 60G gp3, DeleteOnTermination
 AMI                : latest ubuntu noble arm64 (resolved via describe-images at launch)
 tags               : purpose=sparq-bench (never touches prod/dev/other instances;
                      all aws calls operate ONLY on the ids this run creates)
 orphan-proofing    : (1) --instance-initiated-shutdown-behavior terminate
                      (2) user-data watchdog: 'shutdown -h +$max_min' scheduled at boot
                      (3) remote command ends with 'sudo shutdown -h now'
                      => the instance dies even if THIS box vanishes mid-run
 credentials        : ephemeral ssh keypair + ssh-only security group (this box's IP),
                      both deleted by the local cleanup trap
 remote command     : clone $repo_url @ $sha, rustup minimal, then
                      scripts/bench/run-all-benchmarks.sh --tier $TIER_MAX --out ~/bench-results
                      (nohup — survives ssh drops), then self-shutdown
 result streaming   : local poll loop rsyncs remote:~/bench-results/ into
                      $OUT_DIR every 60s — each suite's json/md/log lands here AS IT
                      COMPLETES (not end-of-run); poll ends on the manifest ended_utc
                      or instance termination
 teardown           : local EXIT trap terminates the instance + deletes keypair/SG;
                      the watchdog + self-shutdown cover a dead local box
PLAN
  if [ "${EXECUTE:-0}" != "1" ]; then
    echo "== dry-run only: NOT launching (set EXECUTE=1 when the EC2 quota is back) =="
    return 0
  fi
  echo "== EXECUTE=1: launching (prepared per plan above; validate on first use) =="
  have aws || { echo "aws cli missing" >&2; return 1; }
  local work; work="$(mktemp -d)"
  local keyfile="$work/key" key_name="sparq-bench-$$-${RANDOM}" iid="" sg=""
  ssh-keygen -t ed25519 -N '' -f "$keyfile" -q
  cleanup_remote() {
    set +e
    [ -n "$iid" ] && aws ec2 terminate-instances --region "$region" --instance-ids "$iid" >/dev/null 2>&1
    [ -n "$sg" ] && { [ -n "$iid" ] && aws ec2 wait instance-terminated --region "$region" --instance-ids "$iid" >/dev/null 2>&1
                      aws ec2 delete-security-group --region "$region" --group-id "$sg" >/dev/null 2>&1; }
    aws ec2 delete-key-pair --region "$region" --key-name "$key_name" >/dev/null 2>&1
    rm -rf "$work"
  }
  trap cleanup_remote EXIT
  local ami vpc subnet myip
  ami=$(aws ec2 describe-images --region "$region" --owners 099720109477 \
    --filters "Name=name,Values=ubuntu/images/hvm-ssd-gp3/ubuntu-noble-24.04-arm64-server-*" "Name=state,Values=available" \
    --query 'sort_by(Images,&CreationDate)[-1].ImageId' --output text)
  vpc=$(aws ec2 describe-vpcs --region "$region" --filters Name=isDefault,Values=true --query 'Vpcs[0].VpcId' --output text)
  subnet=$(aws ec2 describe-subnets --region "$region" --filters Name=vpc-id,Values="$vpc" "Name=default-for-az,Values=true" --query 'Subnets[0].SubnetId' --output text)
  myip=$(curl -s https://checkip.amazonaws.com)
  aws ec2 import-key-pair --region "$region" --key-name "$key_name" --public-key-material "fileb://${keyfile}.pub" >/dev/null
  sg=$(aws ec2 create-security-group --region "$region" --group-name "$key_name" --description "sparq bench-all (ephemeral)" --vpc-id "$vpc" --query 'GroupId' --output text)
  aws ec2 authorize-security-group-ingress --region "$region" --group-id "$sg" --protocol tcp --port 22 --cidr "${myip}/32" >/dev/null
  # user-data watchdog: with shutdown-behavior=terminate, a scheduled halt == terminate.
  printf '#!/bin/bash\nshutdown -h +%s "sparq-bench watchdog: max lifetime reached"\n' "$max_min" > "$work/user-data"
  iid=$(aws ec2 run-instances --region "$region" --image-id "$ami" --instance-type "$itype" \
    --key-name "$key_name" --security-group-ids "$sg" --subnet-id "$subnet" --associate-public-ip-address \
    --instance-market-options 'MarketType=spot' \
    --instance-initiated-shutdown-behavior terminate \
    --user-data "file://$work/user-data" \
    --block-device-mappings '[{"DeviceName":"/dev/sda1","Ebs":{"VolumeSize":60,"VolumeType":"gp3","DeleteOnTermination":true}}]' \
    --tag-specifications 'ResourceType=instance,Tags=[{Key=purpose,Value=sparq-bench}]' \
    --query 'Instances[0].InstanceId' --output text)
  aws ec2 wait instance-running --region "$region" --instance-ids "$iid"
  local ip; ip=$(aws ec2 describe-instances --region "$region" --instance-ids "$iid" --query 'Reservations[0].Instances[0].PublicIpAddress' --output text)
  local ssho="-i $keyfile -o StrictHostKeyChecking=no -o UserKnownHostsFile=/dev/null -o ConnectTimeout=15"
  local _try
  for _try in $(seq 1 30); do ssh $ssho "ubuntu@$ip" true 2>/dev/null && break; sleep 10; done
  ssh $ssho "ubuntu@$ip" "nohup bash -c '
    set -e
    sudo apt-get update -qq && sudo DEBIAN_FRONTEND=noninteractive apt-get install -y -qq build-essential pkg-config git python3 curl default-jre default-jdk raptor2-utils unzip libboost-dev rsync >/dev/null 2>&1 || true
    command -v cargo >/dev/null || curl --proto \"=https\" --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal >/dev/null 2>&1
    . \$HOME/.cargo/env
    git clone -q $repo_url sparq && cd sparq && git checkout -q $sha
    scripts/bench/run-all-benchmarks.sh --tier $TIER_MAX --out \$HOME/bench-results || true
    sudo shutdown -h now
  ' >remote.log 2>&1 & disown" || true
  echo "== streaming results from $ip into $OUT_DIR (per-suite, every 60s) =="
  while :; do
    sleep 60
    rsync -az -e "ssh $ssho" "ubuntu@$ip:bench-results/" "$OUT_DIR/" 2>/dev/null || true
    local state
    state=$(aws ec2 describe-instances --region "$region" --instance-ids "$iid" \
      --query 'Reservations[0].Instances[0].State.Name' --output text 2>/dev/null || echo unknown)
    [ "$state" = "running" ] || { echo "instance state: $state — streaming done"; break; }
    grep -q '"ended_utc"' "$OUT_DIR/manifest.json" 2>/dev/null && { echo "remote run complete"; break; }
  done
  echo "== remote results in $OUT_DIR =="
}

# ---------------------------------------------------------------------------
# main
# ---------------------------------------------------------------------------
MODE="local"; DO_LIST=0; DRY=0; TIER_MAX="standard"; ONLY=""; SKIP=""; TIMEOUT_MULT=1; OUT_DIR=""
RUN_ARGS="$*"
while [ $# -gt 0 ]; do
  case "$1" in
    --list) DO_LIST=1 ;;
    --dry-run) DRY=1 ;;
    --remote) MODE="remote" ;;
    --tier) TIER_MAX="$2"; shift ;;
    --only) ONLY="$2"; shift ;;
    --skip) SKIP="$2"; shift ;;
    --out) OUT_DIR="$2"; shift ;;
    --timeout-mult) TIMEOUT_MULT="$2"; shift ;;
    -h|--help) sed -n '2,50p' "${BASH_SOURCE[0]}" | sed 's/^# \{0,1\}//'; exit 0 ;;
    *) echo "unknown arg: $1 (see --help)" >&2; exit 2 ;;
  esac
  shift
done
case "$TIER_MAX" in fast|standard|heavy) : ;; *) echo "--tier must be fast|standard|heavy" >&2; exit 2 ;; esac

[ "$DO_LIST" = "1" ] && { list_catalog; exit 0; }

# selection map: tier filter + --only/--skip, with the reason recorded per suite
declare -A SKIP_MAP=()
tier_rank() { case "$1" in fast) echo 0 ;; standard) echo 1 ;; heavy) echo 2 ;; external) echo 3 ;; esac; }
MAX_RANK="$(tier_rank "$TIER_MAX")"
for row in "${CATALOG[@]}"; do
  id="${row%%|*}"
  tier="$(echo "$row" | cut -d'|' -f2)"
  if [ -n "$ONLY" ]; then
    case ",$ONLY," in *",$id,"*) : ;; *) SKIP_MAP[$id]="not selected by --only" ;; esac
  fi
  if [ -z "${SKIP_MAP[$id]:-}" ] && [ -n "$SKIP" ]; then
    case ",$SKIP," in *",$id,"*) SKIP_MAP[$id]="excluded by --skip" ;; esac
  fi
  if [ -z "${SKIP_MAP[$id]:-}" ] && [ -z "$ONLY" ] && [ "$tier" != "external" ] && [ "$(tier_rank "$tier")" -gt "$MAX_RANK" ]; then
    SKIP_MAP[$id]="tier '$tier' above --tier $TIER_MAX"
  fi
done

RUN_ID="$(date -u +%Y%m%dT%H%M%SZ)-$(git -C "$ROOT" rev-parse --short HEAD 2>/dev/null || echo nosha)"
[ -n "$OUT_DIR" ] || OUT_DIR="$OUT_BASE/$RUN_ID"
SUITES_DIR="$OUT_DIR/suites"
export RUN_ID OUT_DIR NON_CANONICAL_NOTE RUN_ARGS

if [ "$MODE" = "remote" ]; then
  mkdir -p "$OUT_DIR"
  remote_mode
  exit $?
fi

if [ "$DRY" = "1" ]; then
  echo "== dry-run: would write to $OUT_DIR =="
  for row in "${CATALOG[@]}"; do
    IFS='|' read -r id tier tmo min_gb registry desc <<<"$row"
    reason="${SKIP_MAP[$id]:-}"
    [ -z "$reason" ] && reason="$(suite_skip_reason "$id" | head -1)"
    [ -z "$reason" ] && [ -z "$(suite_cmd "$id")" ] && reason="external/manual suite"
    if [ -n "$reason" ]; then
      printf '%-24s SKIP  %s\n' "$id" "$reason"
    else
      printf '%-24s RUN   timeout=%ss  %s\n' "$id" "$(( tmo * TIMEOUT_MULT ))" "$(suite_cmd "$id")"
    fi
  done
  exit 0
fi

mkdir -p "$OUT_DIR" "$SUITES_DIR" "$SCRATCH_BASE"
echo "== run-all-benchmarks: $RUN_ID =="
echo "== results (incremental): $OUT_DIR =="
echo "== $NON_CANONICAL_NOTE =="
df -h /tmp | tail -1
write_header

FAILED_SUITES=()
order=0
for row in "${CATALOG[@]}"; do
  order=$(( order + 1 ))
  run_suite "$row" "$order"
done

# finalize manifest (ended_utc + counts) + print the summary table
python3 - "$OUT_DIR" <<'PY'
import json, sys, glob, os, datetime
out = sys.argv[1]
hdr = json.load(open(os.path.join(out, "header.json")))
suites = []
for p in glob.glob(os.path.join(out, "suites", "*.json")):
    try: suites.append(json.load(open(p)))
    except Exception: pass
suites.sort(key=lambda r: r.get("order", 0))
counts = {}
for r in suites: counts[r["status"]] = counts.get(r["status"], 0) + 1
man = dict(hdr)
now = datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")
man["updated_utc"] = man["ended_utc"] = now
man["counts"] = counts; man["suites"] = suites
tmp = os.path.join(out, ".manifest.json.tmp")
with open(tmp, "w") as f: json.dump(man, f, indent=2)
os.replace(tmp, os.path.join(out, "manifest.json"))
print("== summary ==")
for r in suites:
    print(f"  {r['id']:<24} {r['status']:<8} {r.get('skip_reason','')}")
print("counts:", json.dumps(counts))
PY

echo "== manifest: $OUT_DIR/manifest.json =="
if [ "${#FAILED_SUITES[@]}" -gt 0 ]; then
  echo "== FAILED/TIMEOUT suites: ${FAILED_SUITES[*]} ==" >&2
  exit 1
fi
exit 0
