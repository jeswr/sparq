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
# The floor only RISES: --seed will NOT lower an existing floor unless --allow-lower is
# passed (a deliberate, reviewed regression — e.g. a refactor that legitimately drops a
# crate's measurable surface). Seeding NEW crates / RAISING floors is automatic.
#
# WHY a margin: line% drifts a little with toolchain bumps / nondeterministic test
# ordering. MARGIN=2 (points) keeps the gate meaningful without flaking. The floor is
# the ratchet of record — raise it deliberately as coverage grows.
import argparse, json, math, os, sys

MARGIN = 2  # percentage points of slack below the measured value

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
            ".github/workflows/ci.yml. scripts/coverage-gate.py --check FAILS CI when a "
            "crate's measured line% drops below its floor.",
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

def main():
    ap = argparse.ArgumentParser(description="per-crate coverage ratchet gate")
    ap.add_argument("summary", help="coverage-summary.json from scripts/coverage.sh")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--seed", action="store_true", help="(re)generate the floor file")
    g.add_argument("--check", action="store_true", help="enforce the floor file")
    ap.add_argument("--floor", default=os.path.join(os.path.dirname(__file__), "..",
                    "bench", "coverage-floor.json"))
    ap.add_argument("--allow-lower", action="store_true",
                    help="permit --seed to LOWER a floor (deliberate regression)")
    ap.add_argument("--require-all", action="store_true",
                    help="--check fails if a floor crate is absent from the summary")
    a = ap.parse_args()
    floor = os.path.abspath(a.floor)
    if a.seed:
        sys.exit(seed(a.summary, floor, a.allow_lower))
    sys.exit(check(a.summary, floor, a.require_all))

if __name__ == "__main__":
    main()
