#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.7 / sq-hmd7l.31 — Nemo materialization adapter for the same-box
# reasoning comparison harness (scripts/bench/materialize-same-box.sh).
#
# 🤖 SPARQ agent. Nemo (github.com/knowsys/nemo, Apache-2.0) is a Rust-native Datalog
# reasoner — the Rust-native peer column for the materialization comparison. This
# adapter runs Nemo's forward materialization over the SAME LUBM (ABox + TBox)
# N-Triples that sparq `reason` closes, then reports the materialized triple count
# (correctness oracle) + best-of-N wall time.
#
# TSV OUTPUT (stdout, one line):
#   nemo\t<closure_triples|NOT-RUN-LOCALLY|ERROR>\t<materialize_best_us|reason>
#
# ── VALIDATED ENCODING (sq-hmd7l.31) ──────────────────────────────────────────
# Nemo is a GENERAL Datalog engine, NOT a native OWL 2 RL reasoner. A like-for-like
# comparison with `sparq-cli reason … {owl,rdfs}` needs a `.rls` rule program whose
# materialization REPRODUCES sparq's closure count. Those validated programs now live
# at bench/reason-encodings/nemo/{owl-rl,rdfs}.rls and are wired as the DEFAULT rules
# file per profile (override with NEMO_RULES or argv[4]). Each program closes the SAME
# LUBM ABox+TBox into a single ternary predicate `closed(?s,?p,?o)`; the EXPORTED
# `closed` relation equals sparq's closure count and — VALIDATED — is set-identical to
# sparq's closure (0 diff both directions: rdfs=126732, owl=150589; see the encoding
# headers + the harness count_crosscheck).
#
# COUNT NOTE (load-bearing): Nemo's own "Derived N facts" log line counts EVERY IDB
# predicate, including the tiny helper relations (svfR / int{1,2}def / intTail) the
# owl-rl encoding folds restriction/intersection schema into. The CLOSURE size is the
# EXPORTED `closed` relation, so the encoding @exports ONLY `closed` and this adapter
# counts lines across the output dir (which therefore contains only closed.csv).
#
# The .rls files carry @@DATA@@/@@OUT@@ placeholders (so the git-tracked encoding is
# path-independent); this adapter substitutes the corpus path + an output CSV before
# running `nmo`.
#
# INSTALL (gather-only, per AGENTS.md engines-stay-out-of-git): build from source —
#   git clone https://github.com/knowsys/nemo && cd nemo && cargo build -r -p nemo-cli
# then set NEMO=/path/to/nmo (target/release/nmo). Absent, the column emits NOT-RUN-LOCALLY.
#
# USAGE:
#   nemo_adapter.py <data.nt> <profile> <iters> [rules_file]
#     <profile> in {rdfs, owl}; <rules_file> = a Nemo .rls program (defaults to the
#     validated bench/reason-encodings/nemo/<profile-or-owl-rl>.rls).
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def emit(count, us):
    print("nemo\t{}\t{}".format(count, us))


def default_rules(profile):
    name = "owl-rl.rls" if profile == "owl" else "{}.rls".format(profile)
    path = os.path.join(ROOT, "bench", "reason-encodings", "nemo", name)
    return path if os.path.exists(path) else ""


def main():
    if len(sys.argv) < 4:
        sys.stderr.write("usage: nemo_adapter.py <data.nt> <profile> <iters> [rules_file]\n")
        sys.exit(2)
    data = sys.argv[1]
    profile = sys.argv[2]
    iters = max(1, int(sys.argv[3]))
    rules_file = sys.argv[4] if len(sys.argv) > 4 else os.environ.get("NEMO_RULES", "")
    if not rules_file:
        rules_file = default_rules(profile)

    # Nemo's CLI is `nmo`; accept NEMO override too.
    nemo_bin = os.environ.get("NEMO", shutil.which("nmo") or shutil.which("nemo") or "")
    if not nemo_bin or not os.path.exists(nemo_bin):
        emit("NOT-RUN-LOCALLY", "nmo-not-on-PATH (build github.com/knowsys/nemo: cargo build -r -p nemo-cli; or set NEMO=/path)")
        return

    if not rules_file or not os.path.exists(rules_file):
        emit(
            "NOT-RUN-LOCALLY",
            "no-validated-{}-nemo-rls-encoding (bench/reason-encodings/nemo/ absent; a .rls reproducing sparq's closure is required)".format(
                profile
            ),
        )
        return

    workdir = tempfile.mkdtemp(prefix="nemo-mat-")
    try:
        # Substitute the @@DATA@@ / @@OUT@@ placeholders in the git-tracked encoding.
        with open(rules_file) as fh:
            program = fh.read().replace("@@DATA@@", os.path.abspath(data)).replace("@@OUT@@", "closed.csv")
        local_rls = os.path.join(workdir, "program.rls")
        with open(local_rls, "w") as fh:
            fh.write(program)

        best_us = None
        count = "ERROR"
        for _ in range(iters):
            outdir = os.path.join(workdir, "results")
            if os.path.isdir(outdir):
                shutil.rmtree(outdir)
            t0 = time.perf_counter()
            try:
                subprocess.run(
                    [nemo_bin, local_rls, "--export-dir", outdir, "--overwrite-results"],
                    capture_output=True, text=True, cwd=workdir,
                    timeout=int(os.environ.get("TIMEOUT_S", "900")),
                )
            except subprocess.TimeoutExpired:
                emit("ERROR", "timeout")
                return
            dt_us = (time.perf_counter() - t0) * 1e6
            best_us = dt_us if best_us is None else min(best_us, dt_us)
            # Closure size = exported `closed` facts. The encoding @exports ONLY closed,
            # so summing lines across the output dir counts exactly the closure.
            total = 0
            for root, _dirs, files in os.walk(outdir):
                for fn in files:
                    try:
                        with open(os.path.join(root, fn), encoding="utf-8", errors="replace") as fh:
                            total += sum(1 for _ in fh)
                    except OSError:
                        pass
            count = str(total) if total else "ERROR"
        emit(count, "{:.1f}".format(best_us) if best_us is not None else "na")
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    main()
