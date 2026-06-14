#!/usr/bin/env python3
# [OPUS-4.8] Per-crate line-coverage RATCHET gate (sq-hbg7).
#
# Mirrors the conformance / perf ratchet idiom (a committed FLOOR that only ever
# RISES, reviewed in diffs). Two modes:
#
#   --seed   <summary.json>   regenerate bench/coverage-floor.json from a measured
#                             summary (scripts/coverage.sh output). Each crate's floor
#                             is set to floor(measured) - MARGIN (min 0) so ordinary
#                             run-to-run noise never trips the gate. Crates known to
#                             report a misleading number (sparq-cli subprocess artifact,
#                             sparq-conformance test-driver, sparq-gpu device-gated) get
#                             floor 0 + an annotated note (still presence-gated).
#   --check  <summary.json>   FAIL (exit 1) if any crate in the floor regressed below
#                             its floor in the measured summary. A crate present in the
#                             floor but MISSING from the summary is reported (only fatal
#                             with --require-all, e.g. the per-commit tier need not
#                             measure nightly-only crates).
#
#   --check-robust <summary>  [OPUS-4.8] sq-x4jy: the ROBUST gate driver — MEASURE-AND-
#                             TAKE-MAX across up to K independent measurements, then check.
#                             This replaces the old "re-measure the WHOLE suite once on
#                             failure" CI backstop, which was structurally insufficient
#                             (PR #62: pass-1 sparq-core flaked low -> re-measure the WHOLE
#                             suite -> sparq-engine, UNCHANGED by the PR, then flaked 0.28%
#                             low -> job failed). See the MAX-REMEASURE rationale below.
#
#   --merge-max <a> <b> [-o]  [OPUS-4.8] sq-x4jy: pure helper — merge two summaries by
#                             per-crate MAX(lines_pct), writing the result (used by an
#                             external shell loop, and unit-tested directly).
#
# THE MAX-REMEASURE PRINCIPLE (sq-x4jy) [OPUS-4.8]
# ------------------------------------------------
# llvm-cov instrumentation only ever UNDERCOUNTS: when a test process aborts/OOMs or a
# `.profraw` fails to merge, that contribution is LOST, pulling the number DOWN. It can
# NEVER spuriously overcount. So the principled "true" coverage of a crate is the MAXIMUM
# across repeated independent measurements. The robust gate therefore:
#   1. measures all crates once (the summary passed in),
#   2. finds the crates BELOW floor,
#   3. RE-MEASURES ONLY those crates (not the whole suite — re-measuring everything is
#      exactly why a *second*, PR-unrelated crate flaked in #62, and is needlessly slow),
#      keeping the per-crate MAX seen,
#   4. repeats (2)-(3) up to K total measurements per crate,
#   5. FAILS only if a crate is STILL below floor after K independent measurements — that
#      is a genuine regression. Otherwise PASS, and the final summary carries the per-crate
#      MAX (the most accurate number).
# This is SAFE — it never ACCEPTS a low number; it re-measures and keeps the best. A real
# regression is reproducible and fails ALL K measurements; a transient undercount is not.
#
# The floor only RISES: --seed will NOT lower an existing floor unless --allow-lower is
# passed (a deliberate, reviewed regression — e.g. a refactor that legitimately drops a
# crate's measurable surface). Seeding NEW crates / RAISING floors is automatic.
#
# WHY a margin: line% drifts a little with toolchain bumps / nondeterministic test
# ordering. MARGIN=2 (points) keeps the gate meaningful without flaking. The floor is
# the ratchet of record — raise it deliberately as coverage grows.
import argparse, json, math, os, subprocess, sys

MARGIN = 2  # percentage points of slack below the measured value

# [OPUS-4.8] sq-x4jy: total independent measurements per crate in the robust gate
# (1 initial + up to K-1 targeted re-measures). K=3 caps the worst-case wall-clock of
# the targeted re-measure loop while giving any transiently-undercounted crate two extra
# chances to record its true (higher) number. Keep small: each round shells coverage.sh.
DEFAULT_K = 3

# Crates whose llvm-cov line% is NOT a meaningful gate (floor pinned to 0). They stay
# in the floor file (and the presence gate still guards their tests existing).
ARTIFACT_ZERO = {
    "sparq-cli": "subprocess artifact: tests spawn the COMPILED binary (assert_cmd / "
                 "CARGO_BIN_EXE), not the instrumented profile — line% is ~0 by "
                 "construction, not a real gap. Presence-gated instead.",
    "sparq-conformance": "test-driver crate: the W3C suites run as the BINARIES "
                 "(cargo run), so `cargo llvm-cov --package` (which runs cargo test) "
                 "sees only a few unit tests. Low % is expected; floor 0.",
    "sparq-gpu": "GPU kernels need a device; CPU-only line coverage is low by design.",
}
# Crates measured only in the nightly tier (heavy tests). The per-commit --check must
# NOT fail just because they are absent from a per-commit summary.
NIGHTLY_NOTE = {
    "sparq-vectors": "per-commit floor is measured with the two "
                     "*_recall_at_10_vs_brute_force_on_50k tests SKIPPED; the nightly "
                     "tier measures the full set.",
}

def load(p):
    with open(p) as f:
        return json.load(f)

def seed(summary_path, floor_path, allow_lower):
    s = load(summary_path)
    existing = load(floor_path)["crates"] if os.path.exists(floor_path) else {}
    out = {}
    raised, kept, new, lowered = [], [], [], []
    for crate, row in sorted(s["crates"].items()):
        if not row.get("measured", False):
            # carry an existing floor forward; never invent one for an unmeasured crate
            if crate in existing:
                out[crate] = existing[crate]; kept.append(crate)
            continue
        if crate in ARTIFACT_ZERO:
            out[crate] = {"floor": 0, "note": ARTIFACT_ZERO[crate]}
            continue
        measured = row["lines_pct"]
        proposed = max(0, math.floor(measured) - MARGIN)
        prev = existing.get(crate, {}).get("floor")
        note = NIGHTLY_NOTE.get(crate)
        if prev is None:
            chosen = proposed; new.append(f"{crate}={chosen}")
        elif proposed > prev:
            chosen = proposed; raised.append(f"{crate} {prev}->{chosen}")
        elif proposed < prev and allow_lower:
            chosen = proposed; lowered.append(f"{crate} {prev}->{chosen}")
        else:
            chosen = prev; kept.append(crate)  # ratchet: never auto-lower
        entry = {"floor": chosen}
        if note: entry["note"] = note
        out[crate] = entry
    doc = {
        "_comment": [
            "[OPUS-4.8] COVERAGE RATCHET (sq-hbg7) — committed per-crate line-coverage "
            "FLOOR, reviewed in diffs exactly like the conformance ratchet counts in "
            ".github/workflows/ci.yml. CI runs scripts/coverage-gate.py --check-robust "
            "(sq-x4jy): it re-measures ONLY the sub-floor crates up to K=3 times and keeps "
            "the per-crate MAX (llvm-cov only undercounts), FAILING only if a crate is "
            "STILL below its floor after K independent measurements — a genuine regression.",
            f"Floor = floor(measured) - {MARGIN} (min 0): a small margin so run-to-run / "
            "toolchain noise never trips the gate. The floor only ever RISES "
            "(--seed will not lower it without --allow-lower).",
            "Tiering: the FAST/MEDIUM crates are measured + gated PER-COMMIT; the heavy "
            "sparq-vectors 50k recall/diskann tests are EXCLUDED per-commit (measured in "
            "the NIGHTLY tier — see the .github/workflows/ci.yml coverage-nightly job and "
            "the per-crate notes below). No crate is silently dropped.",
            "Floors with floor:0 are crates whose llvm-cov line% is a known measurement "
            "artifact (see each note) — they are guarded by the test-PRESENCE gate "
            "(bench/coverage-presence.json) instead of the % gate.",
            "Regenerate after a deliberate coverage rise: scripts/coverage.sh && "
            "scripts/coverage-gate.py --seed target/coverage/coverage-summary.json",
        ],
        "margin_points": MARGIN,
        "crates": out,
    }
    with open(floor_path, "w") as f:
        json.dump(doc, f, indent=2, sort_keys=True); f.write("\n")
    print(f"seeded {floor_path}: {len(out)} crates")
    if new:     print("  NEW:    " + ", ".join(new))
    if raised:  print("  RAISED: " + ", ".join(raised))
    if lowered: print("  LOWERED (--allow-lower): " + ", ".join(lowered))
    if kept:    print(f"  kept:   {len(kept)} unchanged")
    return 0

def check(summary_path, floor_path, require_all):
    s = load(summary_path); floors = load(floor_path)["crates"]
    measured = s["crates"]
    fails, missing, oks = [], [], []
    for crate, fentry in sorted(floors.items()):
        floor = fentry["floor"] if isinstance(fentry, dict) else fentry
        row = measured.get(crate)
        if row is None or not row.get("measured", False):
            missing.append(crate); continue
        val = row["lines_pct"]
        if val + 1e-9 < floor:
            fails.append((crate, val, floor))
        else:
            oks.append((crate, val, floor))
    for crate, val, floor in oks:
        print(f"  ok   {crate:<20} {val:6.2f}% >= floor {floor}")
    for crate in missing:
        flag = "MISSING (not in this tier's summary)"
        print(f"  --   {crate:<20} {flag}")
    for crate, val, floor in fails:
        print(f"  FAIL {crate:<20} {val:6.2f}% < floor {floor}")
    bad = bool(fails) or (require_all and bool(missing))
    if fails:
        print(f"::error::coverage regressed below the floor for {len(fails)} crate(s)")
    if require_all and missing:
        print(f"::error::{len(missing)} floor crate(s) absent from the summary "
              f"(--require-all): {', '.join(missing)}")
    print(f"\ncoverage gate: {len(oks)} ok / {len(fails)} fail / {len(missing)} missing")
    return 1 if bad else 0

# --- pure aggregation primitives (unit-tested by --self-test) -----------------
# [OPUS-4.8] sq-x4jy: these are PURE functions over plain dicts so the robust
# max-remeasure logic can be exercised with synthetic measurement sequences WITHOUT
# reproducing the CI flake. The CI driver below is the only thing that does I/O.

def floor_of(fentry):
    """Floor value from a floor-file entry (a dict {"floor": N} or a bare number)."""
    return fentry["floor"] if isinstance(fentry, dict) else fentry

def sub_floor_crates(summary, floors):
    """Pure: the set of crates that are MEASURED in `summary` AND below their floor.

    A crate missing / not-measured is NOT returned (re-measuring a crate that did not
    even run cannot help; the absence is handled by --check's MISSING/--require-all
    semantics). This is exactly the set the robust gate re-measures."""
    out = []
    measured = summary.get("crates", {})
    for crate, fentry in floors.items():
        row = measured.get(crate)
        if row is None or not row.get("measured", False):
            continue
        if row["lines_pct"] + 1e-9 < floor_of(fentry):
            out.append(crate)
    return sorted(out)

def merge_max(prev, new):
    """Pure: return a NEW summary that is `prev` with each crate's coverage replaced by
    the per-crate MAX(lines_pct) seen across `prev` and `new`.

    Undercount-only instrumentation (see header) means MAX is the most accurate estimate.
    The whole stat block (lines_covered/total/seconds/features) of whichever measurement
    had the higher lines_pct is kept, so the recorded row stays internally consistent.
    Crates only in `new` are added; crates only in `prev` are preserved. A non-measured
    `new` row never displaces a measured `prev` row (a failed re-measure must not lower
    the recorded max — that would defeat the point)."""
    out = {k: dict(v) for k, v in prev.get("crates", {}).items()}
    for crate, nrow in new.get("crates", {}).items():
        prow = out.get(crate)
        if prow is None:
            out[crate] = dict(nrow); continue
        # Only a MEASURED new row can win; and only if it is strictly higher.
        if not nrow.get("measured", False):
            continue
        if (not prow.get("measured", False)) or nrow["lines_pct"] > prow["lines_pct"]:
            out[crate] = dict(nrow)
    merged = dict(prev)
    merged["crates"] = out
    return merged

def robust_aggregate(measure_fn, initial, floors, k=DEFAULT_K, require_all=False, log=print):
    """Pure-ish ORCHESTRATION of the max-remeasure gate (the load-bearing logic), with
    all I/O injected via `measure_fn` so it is unit-testable with synthetic rounds.

    `measure_fn(crates)` -> a summary dict measuring exactly the named crates (a subset).
    `initial` is the round-1 whole-suite summary. Loops up to `k` TOTAL measurements,
    each round (a) finding crates still below floor, (b) re-measuring ONLY those, and
    (c) merging by per-crate MAX. Returns (exit_code, final_summary): 0 if every crate is
    at/above its floor on its best-of-k measurement, 1 if any crate is STILL below after k.
    Does NOT call sys.exit / subprocess — the CLI wrapper does the I/O + final --check."""
    summary = initial
    for rnd in range(2, k + 1):  # round 1 is `initial`; rounds 2..k are re-measures
        below = sub_floor_crates(summary, floors)
        if not below:
            log(f"  round {rnd-1}: all crates at/above floor — no re-measure needed")
            break
        log(f"  round {rnd-1}: {len(below)} crate(s) below floor -> re-measuring ONLY "
            f"{', '.join(below)} (round {rnd}/{k})")
        new = measure_fn(below)
        for crate in below:
            old = summary["crates"].get(crate, {}).get("lines_pct")
            nv = new.get("crates", {}).get(crate, {})
            nvp = nv.get("lines_pct") if nv.get("measured", False) else None
            log(f"    {crate:<20} prev={old}  remeasured="
                f"{nvp if nvp is not None else 'FAILED/absent'}  -> max="
                f"{max([x for x in (old, nvp) if x is not None], default=old)}")
        summary = merge_max(summary, new)
    # Final verdict on the per-crate MAX summary.
    still = sub_floor_crates(summary, floors)
    if still:
        log(f"  VERDICT: FAIL — {len(still)} crate(s) STILL below floor after {k} "
            f"measurement(s): {', '.join(still)} (genuine regression, not variance)")
        return 1, summary
    log(f"  VERDICT: PASS — every crate met its floor on its best of <= {k} measurements")
    return 0, summary

def check_robust(summary_path, floor_path, k, require_all,
                 out_path=None, extra_env=None):
    """[OPUS-4.8] sq-x4jy: CLI driver for the robust gate. Loads the round-1 summary +
    floors, then drives `robust_aggregate`, shelling out to scripts/coverage.sh (subset
    mode via COVERAGE_CRATES) for each targeted re-measure round. Writes the final
    per-crate-MAX summary back to `out_path` (default: the input summary path, so the
    uploaded artifact reflects the most-accurate numbers). Then performs the canonical
    --check on that final summary for the human-readable per-crate table + exit code."""
    floors = load(floor_path)["crates"]
    initial = load(summary_path)
    script_dir = os.path.dirname(os.path.abspath(__file__))
    cov_sh = os.path.join(script_dir, "coverage.sh")
    out_path = out_path or summary_path

    def measure_fn(crates):
        # Re-measure ONLY `crates` into a TEMP summary (do not clobber the accumulator),
        # via coverage.sh's documented COVERAGE_CRATES subset mode + COVERAGE_OUT.
        tmp = out_path + ".remeasure.json"
        env = dict(os.environ)
        env["COVERAGE_CRATES"] = " ".join(crates)
        env["COVERAGE_OUT"] = tmp
        # Fixtures are already fetched by round 1; skip the re-fetch to save time.
        env.setdefault("COVERAGE_FETCH_FIXTURES", "0")
        if extra_env:
            env.update(extra_env)
        subprocess.run(["bash", cov_sh], env=env, check=True)
        with open(tmp) as f:
            return json.load(f)

    print(f"==> robust coverage gate: max across up to K={k} measurement(s) "
          f"(re-measure ONLY sub-floor crates)")
    code, final = robust_aggregate(measure_fn, initial, floors, k=k,
                                   require_all=require_all)
    # Persist the per-crate-MAX summary so the uploaded artifact is the accurate one.
    with open(out_path, "w") as f:
        json.dump(final, f, indent=2, sort_keys=True); f.write("\n")
    print(f"==> wrote final per-crate-MAX summary to {out_path}")
    # Canonical check for the familiar per-crate table + the authoritative exit code.
    print("==> final verdict (canonical --check over the per-crate-MAX summary):")
    return check(out_path, floor_path, require_all)

def main():
    # --self-test is a standalone mode (mirrors scripts/perf-gate.py --self-test):
    # unit-test the PURE aggregation logic on synthetic measurement sequences, no files.
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    # --merge-max is a tiny 2-positional helper with its own shape; parse it directly so
    # the main parser's single-`summary` contract is not muddied.
    if "--merge-max" in sys.argv[1:]:
        sys.exit(_cli_merge_max(sys.argv[1:]))

    ap = argparse.ArgumentParser(description="per-crate coverage ratchet gate")
    ap.add_argument("summary", help="coverage-summary.json from scripts/coverage.sh")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--seed", action="store_true", help="(re)generate the floor file")
    g.add_argument("--check", action="store_true", help="enforce the floor file")
    g.add_argument("--check-robust", action="store_true",
                   help="[OPUS-4.8] robust gate: re-measure ONLY sub-floor crates up to "
                        "K times, keep the per-crate MAX, fail only if still below floor")
    ap.add_argument("--floor", default=os.path.join(os.path.dirname(__file__), "..",
                    "bench", "coverage-floor.json"))
    ap.add_argument("--allow-lower", action="store_true",
                    help="permit --seed to LOWER a floor (deliberate regression)")
    ap.add_argument("--require-all", action="store_true",
                    help="--check fails if a floor crate is absent from the summary")
    ap.add_argument("-k", "--max-measurements", type=int, default=DEFAULT_K,
                    help=f"--check-robust: total measurements per crate (default {DEFAULT_K})")
    ap.add_argument("--out", default=None,
                    help="--check-robust: write the final per-crate-MAX summary here "
                         "(default: overwrite the input summary)")
    a = ap.parse_args()
    floor = os.path.abspath(a.floor)
    if a.seed:
        sys.exit(seed(a.summary, floor, a.allow_lower))
    if a.check_robust:
        sys.exit(check_robust(a.summary, floor, a.max_measurements, a.require_all,
                              out_path=a.out))
    sys.exit(check(a.summary, floor, a.require_all))

def _cli_merge_max(argv):
    """`coverage-gate.py --merge-max a.json b.json [-o out.json]` — per-crate MAX merge.
    Prints to stdout if no -o is given. A pure I/O wrapper over merge_max()."""
    args = [x for x in argv if x != "--merge-max"]
    out = None
    if "-o" in args:
        i = args.index("-o"); out = args[i + 1]; del args[i:i + 2]
    elif "--out" in args:
        i = args.index("--out"); out = args[i + 1]; del args[i:i + 2]
    if len(args) != 2:
        sys.stderr.write("usage: coverage-gate.py --merge-max <a.json> <b.json> "
                         "[-o out.json]\n")
        return 1
    merged = merge_max(load(args[0]), load(args[1]))
    text = json.dumps(merged, indent=2, sort_keys=True) + "\n"
    if out:
        with open(out, "w") as f:
            f.write(text)
        print(f"merged per-crate MAX -> {out}")
    else:
        sys.stdout.write(text)
    return 0

def self_test():
    """[OPUS-4.8] sq-x4jy: unit-test the PURE max-remeasure aggregation on SYNTHETIC
    measurement sequences — this is how we KNOW the robust gate works without reproducing
    the CI flake. No files, no subprocess; `measure_fn` is a synthetic round generator.
    Mirrors scripts/perf-gate.py --self-test."""

    def crate(pct, measured=True):
        # Minimal row in the coverage.sh summary shape.
        r = {"measured": measured}
        if measured:
            r.update(lines_pct=pct, lines_covered=int(pct), lines_total=100, seconds=1)
        return r

    def summ(d):
        return {"crates": {k: crate(*v) if isinstance(v, tuple) else crate(v)
                           for k, v in d.items()}}

    FLOORS = {"a": {"floor": 80}, "b": {"floor": 83}, "c": {"floor": 90}}
    quiet = lambda *a, **k: None

    # --- merge_max: per-crate MAX, measured-only, internal consistency ---------
    m = merge_max(summ({"a": 70.0, "b": 90.0}), summ({"a": 95.0, "b": 88.0}))
    assert m["crates"]["a"]["lines_pct"] == 95.0, m
    assert m["crates"]["b"]["lines_pct"] == 90.0, m            # prev higher wins
    # a FAILED (non-measured) re-measure must NOT displace a measured prev row.
    m2 = merge_max(summ({"a": 85.0}), {"crates": {"a": crate(0, measured=False)}})
    assert m2["crates"]["a"]["lines_pct"] == 85.0, m2
    # a crate only in `new` is added.
    m3 = merge_max(summ({"a": 80.0}), summ({"z": 99.0}))
    assert m3["crates"]["z"]["lines_pct"] == 99.0, m3

    # --- sub_floor_crates: only measured-and-below are returned ----------------
    s = summ({"a": 79.9, "b": 84.0, "c": 91.0})
    assert sub_floor_crates(s, FLOORS) == ["a"], sub_floor_crates(s, FLOORS)
    s_missing = summ({"a": 90.0})  # b, c absent -> not "below", just missing
    assert sub_floor_crates(s_missing, FLOORS) == [], sub_floor_crates(s_missing, FLOORS)
    s_unmeasured = {"crates": {"a": crate(0, measured=False)}}  # not measured -> skip
    assert sub_floor_crates(s_unmeasured, FLOORS) == [], "unmeasured must not be 'below'"

    # === SCENARIO 1: a crate flakes low once then recovers above floor -> PASS ===
    # (max wins) — the exact #62 sparq-core shape: pass-1 below floor, re-measure above.
    init1 = summ({"a": 69.6, "b": 90.0, "c": 91.0})   # 'a' flaked low (floor 80)
    rounds1 = iter([summ({"a": 91.2})])               # round-2 re-measure recovers
    code1, fin1 = robust_aggregate(lambda cs: next(rounds1), init1, FLOORS, k=3, log=quiet)
    assert code1 == 0, "scenario1 should PASS (max recovers above floor)"
    assert fin1["crates"]["a"]["lines_pct"] == 91.2, fin1

    # === SCENARIO 2: two DIFFERENT crates flake on different rounds -> PASS ======
    # round1: 'a' low. round2 re-measures {a}: 'a' recovers but the WHOLE-suite is NOT
    # touched, so 'b' can only flake if IT was re-measured. To exercise per-crate MAX
    # across rounds we model: round1 -> a low; round2 (remeasure a) -> a recovers AND we
    # also feed a fresh whole-ish summary where b dipped; per-crate MAX must keep both ups.
    init2 = summ({"a": 70.0, "b": 90.0, "c": 91.0})   # only 'a' below floor in round1
    # round-2 re-measures {a}; return a low 'a' AND a low 'b' (b shouldn't matter — only
    # 'a' was asked for, but merge_max must not let b's low value overwrite the good prev).
    rounds2 = iter([summ({"a": 81.0, "b": 10.0})])
    code2, fin2 = robust_aggregate(lambda cs: next(rounds2), init2, FLOORS, k=3, log=quiet)
    assert code2 == 0, "scenario2 should PASS"
    assert fin2["crates"]["a"]["lines_pct"] == 81.0, fin2   # a recovered (max 70->81)
    assert fin2["crates"]["b"]["lines_pct"] == 90.0, fin2   # b's good prev preserved

    # 2b: genuinely-distinct flakes across rounds both recover via per-crate MAX.
    #   round1: a=70 (low), b=82 (low). round2 remeasures {a,b}: a=85, b=70 (b still low,
    #   a ok). round3 remeasures {b}: b=84 (recovers). All three pass on best-of-K.
    init2b = summ({"a": 70.0, "b": 82.0, "c": 95.0})
    seq2b = iter([summ({"a": 85.0, "b": 70.0}), summ({"b": 84.0})])
    code2b, fin2b = robust_aggregate(lambda cs: next(seq2b), init2b, FLOORS, k=3, log=quiet)
    assert code2b == 0, "scenario2b should PASS (per-crate max across rounds)"
    assert fin2b["crates"]["a"]["lines_pct"] == 85.0 and fin2b["crates"]["b"]["lines_pct"] == 84.0, fin2b

    # === SCENARIO 3: a crate below floor on ALL K measurements -> FAIL (exit 1) ==
    init3 = summ({"a": 50.0, "b": 90.0, "c": 91.0})
    seq3 = iter([summ({"a": 51.0}), summ({"a": 52.0})])  # never reaches floor 80
    code3, fin3 = robust_aggregate(lambda cs: next(seq3), init3, FLOORS, k=3, log=quiet)
    assert code3 == 1, "scenario3 should FAIL (real regression below floor on all K)"
    assert fin3["crates"]["a"]["lines_pct"] == 52.0, "final keeps the best (max) seen"

    # 3b: K=1 means NO re-measure at all — a low initial fails immediately.
    code3b, _ = robust_aggregate(lambda cs: (_ for _ in ()).throw(AssertionError(
        "K=1 must not re-measure")), summ({"a": 50.0}), FLOORS, k=1, log=quiet)
    assert code3b == 1, "K=1 with a sub-floor crate must FAIL without re-measuring"

    # === SCENARIO 4: a floor crate MISSING from the summary ====================
    # sub_floor_crates ignores it (can't re-measure what didn't run); robust PASSES the
    # aggregation, and require_all is enforced by the FINAL canonical --check (CLI layer),
    # matching existing --check semantics. Verify robust_aggregate itself treats missing
    # as not-below (so it doesn't spuriously fail / loop).
    init4 = summ({"a": 90.0})  # b, c missing
    code4, fin4 = robust_aggregate(lambda cs: (_ for _ in ()).throw(AssertionError(
        "must not re-measure a missing crate")), init4, FLOORS, k=3, log=quiet)
    assert code4 == 0, "missing crates are not 'below floor' for the robust loop"

    # === SCENARIO 5: no re-measure needed (all pass round 1) -> measure_fn never called =
    init5 = summ({"a": 85.0, "b": 90.0, "c": 95.0})
    code5, _ = robust_aggregate(lambda cs: (_ for _ in ()).throw(AssertionError(
        "should not re-measure when all pass")), init5, FLOORS, k=3, log=quiet)
    assert code5 == 0

    print("coverage-gate self-test: ALL ASSERTIONS PASSED")
    return 0

if __name__ == "__main__":
    main()
