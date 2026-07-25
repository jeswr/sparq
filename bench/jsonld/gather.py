#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.15 — bench/jsonld gather orchestrator (called by
# `run.sh --gather` AFTER the smoke gates pass; see README.md).
#
# INVARIANT (the load-bearing rule of this suite): NO throughput row is emitted
# for any engine until the OUTPUT-EQUALITY gate for that (fixture, op) pair has
# passed on this run — expand via the conformance deep-equality comparator,
# flatten/tordf via canonical-RDF-dataset equality, compact via the
# losslessness oracle (every engine's compacted output must re-parse to the
# input's dataset). Pairs that fail the gate are EXCLUDED and the exclusion is
# recorded in the envelope (never silently dropped).
#
# Peers are GATHER-ONLY (never committed dependencies):
#   jsonld.js  — `npm install jsonld` into a scratch dir; pass --node-path or
#                set NODE_PATH to its node_modules.
#   titanium   — set TITANIUM_CP to the pinned jars (see jsonld_adapter.py);
#                skipped (and recorded as skipped) when unset.
#
# Results land in bench/competitor-results/<engine>-jsonld-bench-<UTC>.json
# (git-ignored, regenerable) per bench/competitors.json `results_layout`.
# Every timing in the envelope is advisory wall-clock; `quiet_box` records
# whether the host gives trustworthy absolute numbers. Work-box timings are
# NON-canonical and never land in committed markdown.
import argparse
import datetime
import json
import os
import platform
import subprocess
import sys

ROOT = os.path.dirname(os.path.dirname(os.path.dirname(os.path.abspath(__file__))))
FIX = os.path.join(ROOT, "bench", "jsonld", "fixtures")
OUTDIR = os.path.join(ROOT, "bench", "competitor-results")
BIN = os.environ.get(
    "BENCH_JSONLD", os.path.join(ROOT, "target", "release", "examples", "bench_jsonld")
)
FIXTURES = ["wdc-product", "people-graph", "context-heavy"]
OPS = ["expand", "flatten", "compact", "tordf"]


def run(cmd, **kw):
    return subprocess.run(cmd, capture_output=True, text=True, **kw)


def sparq_bin(args_list):
    return run([BIN] + args_list)


def env_block():
    cpu = ""
    try:
        with open("/proc/cpuinfo", encoding="utf-8") as f:
            for line in f:
                if line.startswith("model name"):
                    cpu = line.split(":", 1)[1].strip()
                    break
    except OSError:
        pass
    commit = run(["git", "-C", ROOT, "rev-parse", "--short", "HEAD"]).stdout.strip()
    return {
        "host_class": "github-runner" if os.environ.get("GITHUB_ACTIONS") == "true" else platform.node(),
        "cpu_model": cpu,
        "nproc": os.cpu_count() or 0,
        "os": platform.system(),
        "kernel": platform.release(),
        "quiet_box": bool(os.environ.get("SPARQ_QUIET_BOX")),
        "gathered_at_utc": datetime.datetime.now(datetime.timezone.utc).strftime("%Y-%m-%dT%H-%M-%SZ"),
        "git_commit": commit,
    }


def gate(fixture, op, out_path, tmp):
    """Output-equality gate for one (fixture, op, engine-output). True = row may be timed."""
    input_doc = os.path.join(FIX, fixture + ".jsonld")
    expected = os.path.join(FIX, "expected")
    if op == "expand":
        return sparq_bin(["equal", out_path, os.path.join(expected, fixture + ".expanded.jsonld")]).returncode == 0
    if op == "flatten":
        return sparq_bin(["dataset-equal", out_path, os.path.join(expected, fixture + ".flattened.jsonld")]).returncode == 0
    if op == "tordf":
        return sparq_bin(["dataset-equal", out_path, os.path.join(expected, fixture + ".nq")]).returncode == 0
    if op == "compact":
        # Losslessness: the compacted output must re-parse to the INPUT's dataset.
        return sparq_bin(["dataset-equal", out_path, input_doc]).returncode == 0
    return False


def out_ext(op):
    # dataset-equal sniffs by extension: toRdf artifacts are N-Quads, the rest JSON.
    return ".nq" if op == "tordf" else ".json"


def sparq_output(fixture, op, tmp):
    input_doc = os.path.join(FIX, fixture + ".jsonld")
    ctx = os.path.join(FIX, "contexts", fixture + "-context.jsonld")
    out = os.path.join(tmp, "sparq-{}-{}{}".format(fixture, op, out_ext(op)))
    args = [op, input_doc] + ([ctx] if op == "compact" else []) + ["--out", out]
    r = sparq_bin(args)
    return out if r.returncode == 0 else None


def sparq_time(fixture, op, iters, warmup):
    input_doc = os.path.join(FIX, fixture + ".jsonld")
    ctx = os.path.join(FIX, "contexts", fixture + "-context.jsonld")
    args = ["time", op, input_doc] + ([ctx] if op == "compact" else [])
    args += ["--iters", str(iters), "--warmup", str(warmup)]
    r = sparq_bin(args)
    if r.returncode != 0:
        return None
    # jsonld_<op>  file  bytes  iters  us_per_op  docs_per_s  mb_per_s
    f = r.stdout.strip().split("\t")
    return {"bytes": int(f[2]), "us_per_op": float(f[4]), "docs_per_s": float(f[5]), "mb_per_s": float(f[6])}


def peer_run(engine, fixture, op, iters, warmup, tmp, node_path):
    input_doc = os.path.join(FIX, fixture + ".jsonld")
    ctx = os.path.join(FIX, "contexts", fixture + "-context.jsonld")
    out = os.path.join(tmp, "{}-{}-{}{}".format(engine, fixture, op, out_ext(op)))
    if engine == "jsonld-js":
        cmd = ["node", os.path.join(ROOT, "scripts", "bench-adapters", "jsonld_adapter.mjs")]
        env = dict(os.environ, NODE_PATH=node_path or os.environ.get("NODE_PATH", ""))
    else:  # titanium-json-ld
        cmd = [sys.executable, os.path.join(ROOT, "scripts", "bench-adapters", "jsonld_adapter.py")]
        env = os.environ.copy()
    cmd += ["--op", op, "--input", input_doc, "--iters", str(iters), "--warmup", str(warmup), "--out", out]
    if op == "compact":
        cmd += ["--context", ctx]
    r = run(cmd, env=env)
    if r.returncode != 0:
        return None, None
    envelope = json.loads(r.stdout.strip().splitlines()[-1])
    return out, envelope


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--iters", type=int, default=50)
    ap.add_argument("--warmup", type=int, default=5)
    ap.add_argument("--node-path", help="node_modules dir containing a pinned jsonld.js install")
    args = ap.parse_args()

    os.makedirs(OUTDIR, exist_ok=True)
    import tempfile

    engines = ["jsonld-js"] + (["titanium-json-ld"] if os.environ.get("TITANIUM_CP") else [])
    if "titanium-json-ld" not in engines:
        print("[gather] TITANIUM_CP unset — titanium-json-ld SKIPPED (recorded)", file=sys.stderr)

    env = env_block()
    stamp = env["gathered_at_utc"]
    results = {e: {"engine": e, "suite": "jsonld-bench", "env": env, "rows": [], "excluded": [],
                   "caveats": [
                       "advisory wall-clock; quiet_box=false means a shared work box (non-canonical)",
                       "compact rows compare DIFFERENT pipelines at the same task: sparq = toRdf + "
                       "RDF-writer compaction; peers = W3C document-level Compaction Algorithm. The "
                       "shared gate is round-trip dataset losslessness.",
                   ]}
               for e in ["sparq"] + engines}

    with tempfile.TemporaryDirectory(prefix="jsonld-gather-") as tmp:
        for fixture in FIXTURES:
            for op in OPS:
                # 1. sparq output + gate.
                s_out = sparq_output(fixture, op, tmp)
                if s_out is None or not gate(fixture, op, s_out, tmp):
                    results["sparq"]["excluded"].append({"fixture": fixture, "op": op, "why": "sparq output failed the equality gate"})
                    continue
                t = sparq_time(fixture, op, args.iters, args.warmup)
                if t:
                    results["sparq"]["rows"].append(dict(fixture=fixture, op=op, **t))
                # 2. peers: gate THEIR output, then take the adapter's timing.
                for e in engines:
                    p_out, envl = peer_run(e, fixture, op, args.iters, args.warmup, tmp, args.node_path)
                    if p_out is None or not gate(fixture, op, p_out, tmp):
                        results[e]["excluded"].append({"fixture": fixture, "op": op, "why": "peer output failed the equality gate (or adapter error)"})
                        continue
                    results[e]["rows"].append({
                        "fixture": fixture, "op": op, "bytes": envl["bytes"],
                        "us_per_op": envl["us_per_op"], "docs_per_s": envl["docs_per_s"],
                        "mb_per_s": envl["mb_per_s"], "engine_version": envl.get("engine_version"),
                    })

    for e, payload in results.items():
        path = os.path.join(OUTDIR, "{}-jsonld-bench-{}.json".format(e, stamp))
        with open(path, "w", encoding="utf-8") as f:
            json.dump(payload, f, indent=1)
        print("[gather] wrote {} ({} rows, {} excluded)".format(path, len(payload["rows"]), len(payload["excluded"])))
    return 0


if __name__ == "__main__":
    sys.exit(main())
