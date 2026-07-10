#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.7 — VLog materialization adapter for the same-box reasoning
# comparison harness (scripts/bench/materialize-same-box.sh).
#
# 🤖 SPARQ agent. VLog (github.com/karmaresearch/vlog, Apache-2.0) is a Datalog /
# existential-rule reasoner. This adapter runs VLog's forward materialization over
# the SAME LUBM (ABox + TBox) N-Triples that sparq `reason` closes, then reports the
# materialized triple count (correctness oracle) + best-of-N wall time.
#
# TSV OUTPUT (stdout, one line):
#   vlog\t<closure_triples|NOT-RUN-LOCALLY|ERROR>\t<materialize_best_us|reason>
#
# ── FIDELITY GAP (recorded per-column, NEVER silently absorbed) ────────────────
# VLog is a GENERAL Datalog engine, NOT a native OWL 2 RL reasoner. To compare
# like-for-like with sparq `reason ... owl` (the FULL W3C OWL 2 RL/RDF rule table:
# cls-*/cax-*/scm-*/prp-* incl. prp-trp/prp-inv/cls-svf/cls-int — the exact rules
# LUBM Q6/Q9/Q11/Q12/Q13 depend on), VLog needs a Datalog encoding of that rule set
# as a `.dlog`/`.rls` program whose closure REPRODUCES sparq's closure count. That
# encoding is a SEPARATE, INDEPENDENTLY-VALIDATED artifact (see the registry note on
# vlog + the EYE de-scope rationale in bench/competitors.json #eye): the repo's only
# OWL-in-N3 rule file is a ~12-rule demo that OMITS transitivity/someValuesFrom/
# intersectionOf/inverseOf, so it would UNDER-count. Wiring an unvalidated encoding
# would be a MISLEADING comparison, not a fair one. Until that validated encoding
# exists (tracked as a follow-up bead), this column emits NOT-RUN-LOCALLY with the
# exact blocker rather than a fabricated number.
#
# The RDFS profile is a smaller, well-understood rule set and is the natural first
# encoding to validate; even so we do NOT ship an unvalidated encoding here.
#
# INSTALL BLOCKER (this work box): `vlog` is NOT on PATH. Install (gather-only, per
# AGENTS.md engines-stay-out-of-git): download a release binary from
# github.com/karmaresearch/vlog/releases OR build with cmake, then set VLOG=/path.
#
# USAGE:
#   vlog_adapter.py <data.nt> <profile> <iters> [rules_file]
#     <profile> in {rdfs, owl}; <rules_file> = a VLog .dlog program (optional).
import os
import shutil
import subprocess
import sys
import time


def emit(count, us):
    print("vlog\t{}\t{}".format(count, us))


def main():
    if len(sys.argv) < 4:
        sys.stderr.write("usage: vlog_adapter.py <data.nt> <profile> <iters> [rules_file]\n")
        sys.exit(2)
    data = sys.argv[1]
    profile = sys.argv[2]
    iters = max(1, int(sys.argv[3]))
    rules_file = sys.argv[4] if len(sys.argv) > 4 else os.environ.get("VLOG_RULES", "")

    vlog_bin = os.environ.get("VLOG", shutil.which("vlog") or "")
    if not vlog_bin or not os.path.exists(vlog_bin):
        # HONEST not-run: record the exact blocker, never fabricate a timing.
        emit("NOT-RUN-LOCALLY", "vlog-not-on-PATH (set VLOG=/path; releases: github.com/karmaresearch/vlog)")
        return

    if not rules_file or not os.path.exists(rules_file):
        # Installed but no VALIDATED OWL-RL/RDFS Datalog encoding wired: refuse to
        # emit a number computed from a rule set that does not reproduce sparq's closure.
        emit(
            "NOT-RUN-LOCALLY",
            "no-validated-{}-datalog-encoding (a .dlog reproducing sparq's closure is a separate validated artifact)".format(
                profile
            ),
        )
        return

    # Installed AND a rules file was supplied: run the materialization best-of-N.
    # The count is the materialized-triple total VLog reports; the harness
    # cross-checks it against sparq's closure count before trusting the timing.
    best_us = None
    count = "ERROR"
    for _ in range(iters):
        t0 = time.perf_counter()
        try:
            out = subprocess.run(
                [vlog_bin, "mat", "--rules", rules_file, "--data", data],
                capture_output=True,
                text=True,
                timeout=int(os.environ.get("TIMEOUT_S", "900")),
            )
        except subprocess.TimeoutExpired:
            emit("ERROR", "timeout")
            return
        dt_us = (time.perf_counter() - t0) * 1e6
        best_us = dt_us if best_us is None else min(best_us, dt_us)
        # VLog prints the materialized count; parse defensively.
        for line in (out.stdout + out.stderr).splitlines():
            low = line.lower()
            if "derivation" in low or "materializ" in low or "#facts" in low:
                digits = "".join(ch for ch in line if ch.isdigit())
                if digits:
                    count = digits
    emit(count, "{:.1f}".format(best_us) if best_us is not None else "na")


if __name__ == "__main__":
    main()
