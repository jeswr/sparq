#!/usr/bin/env python3
# [FABLE-5] Output parser + fixture gate + envelope writer for bench/reason-deletion
# (bead sq-31fza). Invoked by run.sh with:
#   RD_TMP        dir holding <tier>.out driver transcripts
#   RD_EXP        expected.tsv path
#   RD_TIERS      space-separated tier names that ran green (driver exit 0)
#   RD_JSON_OUT   envelope output path
#   RD_SPARQ_REV  short git rev (provenance)
#
# Asserts, per tier (all DETERMINISTIC — fixed generator + fixed driver seeds):
#   parsed ABox triple-count == units*8            (generator drift gate)
#   RDFS / OWL-mono / OWL-fixpoint incremental closure sizes == expected.tsv
#     (silent under/over-derivation gate; the driver ALREADY asserted each equals the
#      from-scratch closure, so pinning the incremental size pins both sides)
# Exit 1 on any mismatch. Timings are parsed into the envelope but NEVER asserted
# (work-box wall-clock is non-canonical; see bench/CATALOG.md QUIET-BOX).
#
# stdlib-only.
import json
import os
import platform
import re
import sys
import time

TMP = os.environ["RD_TMP"]
EXP = os.environ["RD_EXP"]
TIERS = os.environ["RD_TIERS"].split()
JSON_OUT = os.environ["RD_JSON_OUT"]

# Driver output shapes (crates/sparq-reason/examples/incremental_olympics_bench.rs):
RE_PARSED = re.compile(r"^parsed (\d+) triples in ")
RE_SECTION = re.compile(r"^== (RDFS \(MaterializedGraph\)|OWL 2 RL \(MaterializedOwlGraph, (?:mono|fixpoint)\)|N3 / WAC) ")
RE_INCR = re.compile(r"^initial incremental build:\s+([0-9.]+)s\s+closure (\d+) \(base (\d+)")
RE_FULL = re.compile(r"^full materialize\S*(?: \(v1 path\))?:\s*([0-9.]+)s\s+closure (\d+)")
RE_DELTA = re.compile(
    r"^delta\s+(\d+): insert\s+([0-9.]+)s \(\s*(\S+)x vs full\)\s+delete\s+([0-9.]+)s \(\s*(\S+)x vs full\)"
)

WL_OF_SECTION = {
    "RDFS (MaterializedGraph)": "rdfs",
    "OWL 2 RL (MaterializedOwlGraph, mono)": "owlmono",
    "OWL 2 RL (MaterializedOwlGraph, fixpoint)": "owlfix",
}


def parse_tier(path):
    """-> {abox, workloads: {wl: {closure, base, incr_build_s, full_s, full_closure,
    deltas: [{n, insert_s, delete_s}]}}}"""
    out = {"abox": None, "workloads": {}}
    wl = None
    with open(path, encoding="utf-8") as f:
        for line in f:
            line = line.rstrip("\n")
            m = RE_PARSED.match(line)
            if m:
                out["abox"] = int(m.group(1))
                continue
            m = RE_SECTION.match(line)
            if m:
                wl = WL_OF_SECTION.get(m.group(1))  # None for the N3/WAC section
                if wl:
                    out["workloads"][wl] = {"deltas": []}
                continue
            if not wl:
                continue
            w = out["workloads"][wl]
            m = RE_INCR.match(line)
            if m:
                w["incr_build_s"] = float(m.group(1))
                w["closure"] = int(m.group(2))
                w["base"] = int(m.group(3))
                continue
            m = RE_FULL.match(line)
            if m:
                w["full_s"] = float(m.group(1))
                w["full_closure"] = int(m.group(2))
                continue
            m = RE_DELTA.match(line)
            if m:
                w["deltas"].append(
                    {"n": int(m.group(1)), "insert_s": float(m.group(2)), "delete_s": float(m.group(4))}
                )
    return out


def main():
    expected = {}
    with open(EXP, encoding="utf-8") as f:
        for line in f:
            if line.startswith("#") or not line.strip():
                continue
            tier, units, abox, rdfs, mono, fix = line.rstrip("\n").split("\t")
            expected[tier] = {
                "units": int(units),
                "abox": int(abox),
                "rdfs": int(rdfs),
                "owlmono": int(mono),
                "owlfix": int(fix),
            }

    fail = 0
    tiers_json = []
    for tier in TIERS:
        exp = expected.get(tier)
        if exp is None:
            print(f"[reasondel] ERROR: tier {tier} missing from expected.tsv", file=sys.stderr)
            fail = 1
            continue
        parsed = parse_tier(os.path.join(TMP, f"{tier}.out"))
        if parsed["abox"] != exp["abox"]:
            print(
                f"[reasondel] ERROR: tier {tier} ABox={parsed['abox']} expected={exp['abox']}"
                " (generator or parser drift)",
                file=sys.stderr,
            )
            fail = 1
        for wl in ("rdfs", "owlmono", "owlfix"):
            w = parsed["workloads"].get(wl)
            got = w.get("closure") if w else None
            if got != exp[wl]:
                print(
                    f"[reasondel] ERROR: tier {tier} {wl} closure={got} expected={exp[wl]}"
                    " (derivation regression)",
                    file=sys.stderr,
                )
                fail = 1
                continue
            # Metric TSV (harvest-friendly; deterministic count + trend-only timings).
            print(f"reasondel_{tier}_{wl}_closure_triples\t{w['closure']}\ttriples")
            if "full_s" in w:
                print(f"reasondel_{tier}_{wl}_full_s\t{w['full_s']}\ts")
            for d in w["deltas"]:
                print(f"reasondel_{tier}_{wl}_ins{d['n']}_s\t{d['insert_s']}\ts")
                print(f"reasondel_{tier}_{wl}_del{d['n']}_s\t{d['delete_s']}\ts")
                d["delete_ratio"] = round(d["n"] / parsed["abox"], 6) if parsed["abox"] else None
        tiers_json.append({"tier": tier, "units": exp["units"], **parsed})

    envelope = {
        "suite": "reason-deletion",
        "bead": "sq-31fza",
        "generated_at_utc": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
        "sparq_rev": os.environ.get("RD_SPARQ_REV", "unknown"),
        "driver": "crates/sparq-reason/examples/incremental_olympics_bench.rs (UNMODIFIED; fixed-seed randomized ABox insert/delete batches)",
        "corpus": {
            "generator": "bench/reason-deletion/gen_reason_deletion.py (deterministic LCG; LUBM-class shape over the driver's TBox vocabulary)",
            "triples_per_unit": 8,
        },
        "correctness": {
            "differential": "pass" if not fail else "fail",
            "meaning": "driver exit 0 == built-in asserts held: incremental closure set-equal to from-scratch re-materialization after the randomized insert/delete sequence, zero full rebuilds on ABox deltas, in all three workloads; closure sizes additionally pinned to expected.tsv",
        },
        "host": {
            "platform": platform.platform(),
            "machine": platform.machine(),
            "nproc": os.cpu_count(),
            "quiet_box": False,  # flip only on a dedicated quiet gather box
            "note": "work-box timings are NON-canonical; see bench/CATALOG.md QUIET-BOX",
        },
        "tiers": tiers_json,
        "timing_note": "delta timings are single-shot wall-clock (the driver measures one batch per size); trend-only, never perf-gated; delete_ratio = delta / ABox triples",
    }
    os.makedirs(os.path.dirname(JSON_OUT) or ".", exist_ok=True)
    with open(JSON_OUT, "w", encoding="utf-8") as f:
        json.dump(envelope, f, indent=2)
        f.write("\n")
    return fail


if __name__ == "__main__":
    sys.exit(main())
