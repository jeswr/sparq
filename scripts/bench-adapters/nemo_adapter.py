#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.7 — Nemo materialization adapter for the same-box reasoning
# comparison harness (scripts/bench/materialize-same-box.sh).
#
# 🤖 SPARQ agent. Nemo (github.com/knowsys/nemo, Apache-2.0) is a Rust-native
# Datalog reasoner — the Rust-native peer column for the materialization comparison.
# This adapter runs Nemo's forward materialization over the SAME LUBM (ABox + TBox)
# N-Triples that sparq `reason` closes, then reports the materialized triple count
# (correctness oracle) + best-of-N wall time.
#
# TSV OUTPUT (stdout, one line):
#   nemo\t<closure_triples|NOT-RUN-LOCALLY|ERROR>\t<materialize_best_us|reason>
#
# ── FIDELITY GAP (recorded per-column, NEVER silently absorbed) ────────────────
# Nemo is a GENERAL Datalog engine, NOT a native OWL 2 RL reasoner. Like VLog, a
# like-for-like comparison with sparq `reason ... owl` (the FULL W3C OWL 2 RL/RDF
# rule table) requires a Nemo `.rls` rule program whose closure REPRODUCES sparq's
# closure count — an independently-VALIDATED artifact, not a drop-in. The repo has
# no such validated OWL-RL-in-`.rls` encoding; shipping an unvalidated one would be
# a MISLEADING comparison (it would under- or over-count the OWL rules LUBM depends
# on). Until that validated encoding exists (tracked as a follow-up bead), this
# column emits NOT-RUN-LOCALLY with the exact blocker rather than a fabricated number.
#
# INSTALL BLOCKER (this work box): `nmo` (the Nemo CLI) is NOT on PATH. Install
# (gather-only, per AGENTS.md engines-stay-out-of-git): `cargo install nemo-cli`
# OR download a release from github.com/knowsys/nemo/releases, then set NEMO=/path.
#
# USAGE:
#   nemo_adapter.py <data.nt> <profile> <iters> [rules_file]
#     <profile> in {rdfs, owl}; <rules_file> = a Nemo .rls program (optional).
import os
import shutil
import subprocess
import sys
import tempfile
import time


def emit(count, us):
    print("nemo\t{}\t{}".format(count, us))


def main():
    if len(sys.argv) < 4:
        sys.stderr.write("usage: nemo_adapter.py <data.nt> <profile> <iters> [rules_file]\n")
        sys.exit(2)
    data = sys.argv[1]
    profile = sys.argv[2]
    iters = max(1, int(sys.argv[3]))
    rules_file = sys.argv[4] if len(sys.argv) > 4 else os.environ.get("NEMO_RULES", "")

    # Nemo's CLI is `nmo`; accept NEMO override too.
    nemo_bin = os.environ.get("NEMO", shutil.which("nmo") or shutil.which("nemo") or "")
    if not nemo_bin or not os.path.exists(nemo_bin):
        emit("NOT-RUN-LOCALLY", "nmo-not-on-PATH (cargo install nemo-cli; or set NEMO=/path)")
        return

    if not rules_file or not os.path.exists(rules_file):
        emit(
            "NOT-RUN-LOCALLY",
            "no-validated-{}-nemo-rls-encoding (a .rls reproducing sparq's closure is a separate validated artifact)".format(
                profile
            ),
        )
        return

    best_us = None
    count = "ERROR"
    for _ in range(iters):
        outdir = tempfile.mkdtemp(prefix="nemo-mat-")
        t0 = time.perf_counter()
        try:
            subprocess.run(
                [nemo_bin, rules_file, "--output-dir", outdir, "--overwrite-results"],
                capture_output=True,
                text=True,
                timeout=int(os.environ.get("TIMEOUT_S", "900")),
            )
        except subprocess.TimeoutExpired:
            emit("ERROR", "timeout")
            return
        dt_us = (time.perf_counter() - t0) * 1e6
        best_us = dt_us if best_us is None else min(best_us, dt_us)
        # Count materialized facts across Nemo's output relation files.
        total = 0
        for root, _dirs, files in os.walk(outdir):
            for fn in files:
                try:
                    with open(os.path.join(root, fn), encoding="utf-8") as fh:
                        total += sum(1 for _ in fh)
                except OSError:
                    pass
        count = str(total) if total else "ERROR"
    emit(count, "{:.1f}".format(best_us) if best_us is not None else "na")


if __name__ == "__main__":
    main()
