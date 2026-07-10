#!/usr/bin/env python3
# [FABLE-5] sq-hmd7l.7 / sq-hmd7l.30 — VLog materialization adapter for the same-box
# reasoning comparison harness (scripts/bench/materialize-same-box.sh).
#
# 🤖 SPARQ agent. VLog (github.com/karmaresearch/vlog, Apache-2.0) is a Datalog /
# existential-rule reasoner. This adapter runs VLog's forward materialization over
# the SAME LUBM (ABox + TBox) N-Triples that sparq `reason` closes, then reports the
# materialized triple count (correctness oracle) + best-of-N wall time.
#
# TSV OUTPUT (stdout, one line):
#   vlog\t<closure_triples|NOT-RUN-LOCALLY|ERROR>\t<materialize_best_us|reason>
#
# ── VALIDATED ENCODING (sq-hmd7l.30) ──────────────────────────────────────────
# VLog is a GENERAL Datalog engine, NOT a native OWL 2 RL reasoner. A like-for-like
# closure comparison with `sparq-cli reason … {owl,rdfs}` needs a Datalog program
# whose materialization REPRODUCES sparq's closure count. Those validated programs now
# live at bench/reason-encodings/vlog/{owl-rl,rdfs}.dlog and are wired as the DEFAULT
# rules file per profile (override with VLOG_RULES or argv[4]). Each encoding closes
# the SAME LUBM ABox+TBox into a single ternary predicate T(s,p,o); |T| equals sparq's
# closure count and — VALIDATED — is set-identical to sparq's closure (0 diff both
# directions: rdfs=126732, owl=150589; see the encoding headers + the harness
# count_crosscheck). An unvalidated encoding (e.g. VLog's shipped ~12-rule LUBM demo)
# would under-count LUBM Q6/Q9/Q11/Q12/Q13; this adapter refuses to run one.
#
# HOW VLog IS DRIVEN (matches the VLog CLI, not a guessed flag): the input N-Triples
# is bound to the EDB predicate TE via a generated edb.conf (EDB0_type=INMEMORY loads
# the .nt directly), then `mat --edb … --rules … --storemat_path … --decompressmat 1`
# writes the materialized T relation whose line count is the closure size. `#`-comment
# and blank lines are stripped from the .dlog before `mat` (VLog rejects them), and
# each rule must already be on ONE line in the encoding (VLog treats a newline as a
# rule terminator).
#
# INSTALL (gather-only, per AGENTS.md engines-stay-out-of-git): build from source —
#   git clone https://github.com/karmaresearch/vlog && cd vlog && mkdir build && cd build
#   cmake -DCMAKE_CXX_FLAGS="-include cstdint" .. && make -j    # the cstdint flag works
#   around a GCC-13 missing-include in the pinned trident/fft sources.
# then set VLOG=/path/to/vlog. Absent, the column emits NOT-RUN-LOCALLY.
#
# USAGE:
#   vlog_adapter.py <data.nt> <profile> <iters> [rules_file]
#     <profile> in {rdfs, owl}; <rules_file> = a VLog .dlog program (defaults to the
#     validated bench/reason-encodings/vlog/<profile-or-owl-rl>.dlog).
import os
import shutil
import subprocess
import sys
import tempfile
import time

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), "..", ".."))


def emit(count, us):
    print("vlog\t{}\t{}".format(count, us))


def default_rules(profile):
    name = "owl-rl.dlog" if profile == "owl" else "{}.dlog".format(profile)
    path = os.path.join(ROOT, "bench", "reason-encodings", "vlog", name)
    return path if os.path.exists(path) else ""


def main():
    if len(sys.argv) < 4:
        sys.stderr.write("usage: vlog_adapter.py <data.nt> <profile> <iters> [rules_file]\n")
        sys.exit(2)
    data = sys.argv[1]
    profile = sys.argv[2]
    iters = max(1, int(sys.argv[3]))
    rules_file = sys.argv[4] if len(sys.argv) > 4 else os.environ.get("VLOG_RULES", "")
    if not rules_file:
        rules_file = default_rules(profile)

    vlog_bin = os.environ.get("VLOG", shutil.which("vlog") or "")
    if not vlog_bin or not os.path.exists(vlog_bin):
        # HONEST not-run: record the exact blocker, never fabricate a timing.
        emit("NOT-RUN-LOCALLY", "vlog-not-on-PATH (set VLOG=/path; build: github.com/karmaresearch/vlog)")
        return

    if not rules_file or not os.path.exists(rules_file):
        # Installed but no VALIDATED OWL-RL/RDFS Datalog encoding available: refuse to
        # emit a number computed from a rule set that does not reproduce sparq's closure.
        emit(
            "NOT-RUN-LOCALLY",
            "no-validated-{}-datalog-encoding (bench/reason-encodings/vlog/ absent; a .dlog reproducing sparq's closure is required)".format(
                profile
            ),
        )
        return

    workdir = tempfile.mkdtemp(prefix="vlog-mat-")
    try:
        # 1. EDB config: bind the input .nt to predicate TE (INMEMORY loads .nt directly).
        #    INMEMORY wants a parent dir + a filename WITHOUT extension, so symlink/copy
        #    the corpus into workdir as data.nt.
        local_nt = os.path.join(workdir, "data.nt")
        shutil.copyfile(data, local_nt)
        edb = os.path.join(workdir, "edb.conf")
        with open(edb, "w") as fh:
            fh.write("EDB0_predname=TE\nEDB0_type=INMEMORY\nEDB0_param0={}\nEDB0_param1=data\n".format(workdir))

        # 2. Strip #-comments/blank lines from the .dlog (VLog rejects them in the stream).
        clean_rules = os.path.join(workdir, "rules.dlog")
        with open(rules_file) as src, open(clean_rules, "w") as dst:
            for line in src:
                s = line.strip()
                if s and not s.startswith("#"):
                    dst.write(line)

        best_us = None
        count = "ERROR"
        for _ in range(iters):
            storedir = os.path.join(workdir, "inf")
            if os.path.isdir(storedir):
                shutil.rmtree(storedir)
            t0 = time.perf_counter()
            try:
                subprocess.run(
                    [
                        vlog_bin, "mat", "--edb", edb, "--rules", clean_rules,
                        "--storemat_path", storedir, "--storemat_format", "csv",
                        "--decompressmat", "1", "-l", "error",
                    ],
                    capture_output=True, text=True,
                    timeout=int(os.environ.get("TIMEOUT_S", "900")),
                )
            except subprocess.TimeoutExpired:
                emit("ERROR", "timeout")
                return
            dt_us = (time.perf_counter() - t0) * 1e6
            best_us = dt_us if best_us is None else min(best_us, dt_us)
            # The closure size is the cardinality of the accumulated T relation, i.e.
            # the line count of the stored T file. (VLog stores one file per IDB pred;
            # the encodings put the whole materialized graph in T.)
            tfile = os.path.join(storedir, "T")
            if os.path.exists(tfile):
                with open(tfile, encoding="utf-8", errors="replace") as fh:
                    count = str(sum(1 for _ in fh))
        emit(count, "{:.1f}".format(best_us) if best_us is not None else "na")
    finally:
        shutil.rmtree(workdir, ignore_errors=True)


if __name__ == "__main__":
    main()
