#!/usr/bin/env python3
# [OPUS-4.8] Per-crate MUTATION-TESTING RATCHET gate (sq-8kt3, epic sq-qcnn).
#
# The test-QUALITY companion to the line-coverage ratchet (scripts/coverage-gate.py +
# bench/coverage-floor.json). Line coverage proves a line RAN; mutation testing proves a
# test would actually NOTICE if that line's behaviour changed. cargo-mutants applies small
# semantic mutations (`<` -> `<=`, `a && b` -> `a || b`, replace a body with its default,
# …) and re-runs the crate's tests per mutant. A mutant the tests still PASS on ("missed"
# / surviving) is a test GAP — code that 100% line coverage happily hides because some test
# executes the line without asserting on its result.
#
# Mirrors the coverage ratchet idiom EXACTLY (a committed per-crate FLOOR reviewed in
# diffs). Two modes:
#
#   --seed   <outcomes...>   regenerate bench/mutants-baseline.json from one or more
#                            cargo-mutants `mutants.out/outcomes.json` files (one per
#                            crate, or a workspace run). Each crate's ratchet is the
#                            SURVIVING-mutant CEILING set to measured_surviving + MARGIN
#                            (a little slack so a mutation-set reshuffle on a toolchain bump
#                            never trips the gate). The ceiling only ever FALLS (--seed will
#                            not RAISE it without --allow-higher) — fewer survivors == better
#                            tests, the ratchet of record.
#   --check  <outcomes...>   FAIL (exit 1) if any crate in the baseline has MORE surviving
#                            mutants than its committed ceiling. A crate present in the
#                            baseline but MISSING from the measured set is reported (only
#                            fatal with --require-all — the nightly tier need not measure
#                            every crate every run; whichever crates ran are checked).
#
# WHY SURVIVING-COUNT (not caught%) IS THE STABLE METRIC
# ------------------------------------------------------
# caught% = caught / (caught + missed) has a moving DENOMINATOR: cargo-mutants regenerates
# its mutant set from the current source, so a refactor that adds/removes mutable sites
# shifts the % even when test quality is unchanged — a noisy gate. The SURVIVING (missed)
# COUNT is the thing we actually want to drive to zero: it is an absolute count of
# detected test gaps, monotone in test quality, and unaffected by caught mutants being
# added elsewhere. So the ceiling ratchets the surviving COUNT down. caught% is recorded
# alongside for humans but is NOT the gate. `unviable` mutants (didn't compile) and
# `timeout` mutants are NOT counted as survivors (a timeout means the mutant changed
# behaviour enough to hang — effectively caught; an unviable mutant tests nothing).
#
# WHY a margin: cargo-mutants' generated set is deterministic for a fixed source+toolchain,
# but a compiler/std bump can shift which mutants are viable. MARGIN=2 (mutants) of slack
# keeps the gate meaningful without flaking on that. The ceiling is the ratchet of record —
# LOWER it deliberately (re-seed) as tests improve.
#
# Regenerate after tests improve (fewer survivors):
#   cargo mutants -p <crate> -o target/mutants/<crate>
#   scripts/mutants-gate.py --seed target/mutants/<crate>/outcomes.json
import argparse, json, os, sys

MARGIN = 2  # surviving-mutant slack above the measured value

ROOT = os.path.abspath(os.path.join(os.path.dirname(__file__), ".."))
DEFAULT_BASELINE = os.path.join(ROOT, "bench", "mutants-baseline.json")

# Crates EXCLUDED from mutation testing — same rationale as the coverage ratchet's
# floor:0 crates (bench/coverage-floor.json). These are presence-gated there and would
# produce misleading surviving-mutant noise here (out-of-process / device / non-host
# target), so they are never seeded and are skipped if they somehow appear in a summary.
EXCLUDED = {
    "sparq-cli": "subprocess artifact: tests spawn the COMPILED binary (assert_cmd / "
                 "CARGO_BIN_EXE), so a library mutant is not exercised in-process and "
                 "would survive spuriously. Presence-gated in the coverage ratchet.",
    "sparq-conformance": "test-driver crate: the W3C suites run as the BINARIES "
                 "(`cargo run`), not `cargo test`, so mutants in the driver survive "
                 "spuriously. Floor 0 in the coverage ratchet.",
    "sparq-gpu": "GPU kernels need a real device; host-only mutation testing surfaces "
                 "spurious survivors.",
    "sparq-bench": "perf-harness crate (no #[test]s to catch mutants).",
    "sparq-py": "pyo3 bindings tested via pytest, not the host Rust test profile.",
}


def load(p):
    with open(p) as f:
        return json.load(f)


def crate_of(outcomes_path, doc):
    """Crate (package) name for an outcomes.json. cargo-mutants stamps the package into
    every mutant scenario (`scenario.Mutant.package`), so read it from the first mutant
    entry — robust to where the file lives on disk. Falls back to the mutants.out parent
    dir name only if no Mutant scenario is present (e.g. an all-unviable edge case)."""
    for o in doc.get("outcomes", []):
        sc = o.get("scenario")
        if isinstance(sc, dict) and "Mutant" in sc:
            pkg = sc["Mutant"].get("package")
            if pkg:
                return pkg
    # Fallback: the CI lane writes target/mutants/<crate>/mutants.out/outcomes.json, so
    # the crate is the dir ABOVE mutants.out.
    parent = os.path.dirname(os.path.abspath(outcomes_path))          # .../mutants.out
    if os.path.basename(parent) == "mutants.out":
        parent = os.path.dirname(parent)
    return os.path.basename(parent)


def summarise(doc):
    """Reduce a cargo-mutants outcomes.json to (surviving, caught, unviable, timeout,
    total_viable). Prefer the top-level rollup counts cargo-mutants writes
    (`missed`/`caught`/`timeout`/`unviable`); fall back to tallying per-mutant `summary`
    strings if an older format omits them. MissedMutant == surviving (a test GAP);
    CaughtMutant + Timeout == killed; Unviable is excluded from the denominator (it never
    compiled, so it tests nothing)."""
    if all(k in doc for k in ("missed", "caught", "timeout", "unviable")):
        surviving = doc["missed"]; caught = doc["caught"]
        timeout = doc["timeout"]; unviable = doc["unviable"]
    else:
        surviving = caught = unviable = timeout = 0
        for o in doc.get("outcomes", []):
            s = o.get("summary", "")
            if s == "MissedMutant":   surviving += 1
            elif s == "CaughtMutant": caught += 1
            elif s == "Timeout":      timeout += 1
            elif s == "Unviable":     unviable += 1
            # "Success" (the unmutated baseline) and anything else are ignored.
    total_viable = surviving + caught + timeout
    return surviving, caught, unviable, timeout, total_viable


def measured_rows(outcome_paths):
    """Build {crate: {surviving, caught, unviable, timeout, caught_pct}} from outcomes
    files, skipping EXCLUDED crates."""
    rows = {}
    for p in outcome_paths:
        doc = load(p)
        crate = crate_of(p, doc)
        if crate in EXCLUDED:
            print(f"  skip {crate} (excluded: see baseline _comment)")
            continue
        surviving, caught, unviable, timeout, total = summarise(doc)
        killed = caught + timeout
        caught_pct = round(100.0 * killed / total, 2) if total else 0.0
        rows[crate] = {
            "surviving": surviving,
            "caught": caught,
            "timeout": timeout,
            "unviable": unviable,
            "caught_pct": caught_pct,
        }
    return rows


def _comment_block():
    return [
        "[OPUS-4.8] MUTATION-TESTING RATCHET (sq-8kt3, epic sq-qcnn) — committed per-crate "
        "SURVIVING-mutant CEILING, reviewed in diffs exactly like the line-coverage ratchet "
        "(bench/coverage-floor.json). scripts/mutants-gate.py --check FAILS CI when a crate "
        "has MORE surviving (test-undetected) mutants than its ceiling. This measures test "
        "QUALITY: a mutant that survives is code a test ran over but never asserted on — a "
        "vacuous-test gap that line coverage cannot see.",
        f"ceiling = measured_surviving + {MARGIN} (a small margin so a toolchain bump that "
        "reshuffles the viable mutant set never trips the gate). The ceiling only ever FALLS "
        "(--seed will not raise it without --allow-higher). caught_pct is recorded for humans "
        "but the GATE is the surviving COUNT (stable: caught% has a moving denominator).",
        "Tiering: cargo-mutants rebuilds + reruns the suite ONCE PER MUTANT, so it is far too "
        "slow for per-commit. It runs in the NIGHTLY tier only — .github/workflows/ci.yml "
        "`mutants-nightly` (schedule + workflow_dispatch), and is ADVISORY (non-gating: the "
        "lane name contains \"advisory\" so the ci-summary aggregator excludes it) until this "
        "baseline is fully seeded across the workspace, then it ratchets to gating. It is NOT "
        "on the per-PR required ci-summary path — adding it there would blow the PR budget.",
        "Excluded crates (sparq-cli/-conformance/-gpu/-bench/-py and *-wasm): same rationale "
        "as the coverage ratchet's floor:0 crates — see scripts/mutants-gate.py EXCLUDED and "
        ".cargo/mutants.toml exclude_globs.",
        "HONESTY: a crate appears here ONLY once it has actually been measured by a real "
        "cargo-mutants run. Crates not yet listed are seeded on their first nightly run — the "
        "absence of a crate is NOT a claim that it has zero surviving mutants.",
        "Regenerate after tests improve: cargo mutants -p <crate> -o target/mutants/<crate> "
        "&& scripts/mutants-gate.py --seed target/mutants/<crate>/outcomes.json",
    ]


def seed(outcome_paths, baseline_path, allow_higher):
    measured = measured_rows(outcome_paths)
    existing = load(baseline_path)["crates"] if os.path.exists(baseline_path) else {}
    out = dict(existing)  # carry forward crates not measured this run
    lowered, raised, kept, new = [], [], [], []
    for crate, row in sorted(measured.items()):
        proposed = row["surviving"] + MARGIN
        prev = existing.get(crate, {}).get("ceiling")
        entry = {
            "ceiling": None,
            "measured_surviving": row["surviving"],
            "caught": row["caught"],
            "timeout": row["timeout"],
            "unviable": row["unviable"],
            "caught_pct": row["caught_pct"],
        }
        if prev is None:
            entry["ceiling"] = proposed
            new.append(f"{crate}={proposed} ({row['surviving']} survived)")
        elif proposed < prev:
            entry["ceiling"] = proposed
            lowered.append(f"{crate} {prev}->{proposed}")
        elif proposed > prev and allow_higher:
            entry["ceiling"] = proposed
            raised.append(f"{crate} {prev}->{proposed} (--allow-higher)")
        else:
            entry["ceiling"] = prev  # ratchet: never auto-raise
            kept.append(crate)
        out[crate] = entry
    doc = {"_comment": _comment_block(), "margin": MARGIN, "crates": out}
    with open(baseline_path, "w") as f:
        json.dump(doc, f, indent=2, sort_keys=True); f.write("\n")
    print(f"seeded {baseline_path}: {len(out)} crate(s) total")
    if new:     print("  NEW:     " + ", ".join(new))
    if lowered: print("  LOWERED: " + ", ".join(lowered))
    if raised:  print("  RAISED (--allow-higher): " + ", ".join(raised))
    if kept:    print(f"  kept:    {len(kept)} unchanged")
    return 0


def check(outcome_paths, baseline_path, require_all):
    ceilings = load(baseline_path)["crates"]
    measured = measured_rows(outcome_paths)
    fails, missing, oks = [], [], []
    for crate, centry in sorted(ceilings.items()):
        ceiling = centry["ceiling"] if isinstance(centry, dict) else centry
        row = measured.get(crate)
        if row is None:
            missing.append(crate); continue
        surviving = row["surviving"]
        if surviving > ceiling:
            fails.append((crate, surviving, ceiling, row["caught_pct"]))
        else:
            oks.append((crate, surviving, ceiling, row["caught_pct"]))
    # Crates measured but not yet in the baseline: report (informational — they get
    # seeded next --seed; we never fail on an unknown crate).
    unseeded = sorted(set(measured) - set(ceilings))
    for crate, surv, ceil, pct in oks:
        print(f"  ok   {crate:<20} surviving {surv:>3} <= ceiling {ceil:<3} (caught {pct}%)")
    for crate in missing:
        print(f"  --   {crate:<20} MISSING (not measured in this run)")
    for crate in unseeded:
        r = measured[crate]
        print(f"  new  {crate:<20} surviving {r['surviving']:>3} (UNSEEDED — run --seed)")
    for crate, surv, ceil, pct in fails:
        print(f"  FAIL {crate:<20} surviving {surv:>3} >  ceiling {ceil:<3} (caught {pct}%)")
    bad = bool(fails) or (require_all and bool(missing))
    if fails:
        print(f"::error::mutation ratchet regressed for {len(fails)} crate(s): more "
              f"surviving mutants than the committed ceiling — add/strengthen tests")
    if require_all and missing:
        print(f"::error::{len(missing)} baseline crate(s) absent from this run "
              f"(--require-all): {', '.join(missing)}")
    print(f"\nmutation gate: {len(oks)} ok / {len(fails)} fail / {len(missing)} missing "
          f"/ {len(unseeded)} unseeded")
    return 1 if bad else 0


def main():
    if "--self-test" in sys.argv[1:]:
        sys.exit(self_test())
    ap = argparse.ArgumentParser(description="per-crate mutation-testing ratchet gate")
    ap.add_argument("outcomes", nargs="+",
                    help="one or more cargo-mutants mutants.out/outcomes.json files "
                         "(target/mutants/<crate>/outcomes.json)")
    g = ap.add_mutually_exclusive_group(required=True)
    g.add_argument("--seed", action="store_true", help="(re)generate the baseline file")
    g.add_argument("--check", action="store_true", help="enforce the baseline file")
    ap.add_argument("--baseline", default=DEFAULT_BASELINE)
    ap.add_argument("--allow-higher", action="store_true",
                    help="permit --seed to RAISE a ceiling (deliberate regression)")
    ap.add_argument("--require-all", action="store_true",
                    help="--check fails if a baseline crate is absent from this run")
    a = ap.parse_args()
    baseline = os.path.abspath(a.baseline)
    if a.seed:
        sys.exit(seed(a.outcomes, baseline, a.allow_higher))
    sys.exit(check(a.outcomes, baseline, a.require_all))


def self_test():
    """[OPUS-4.8] sq-8kt3: unit-test the PURE summary + ratchet logic on synthetic
    outcomes docs — no files, no cargo. Mirrors coverage-gate.py --self-test."""
    def doc(missed=0, caught=0, timeout=0, unviable=0):
        out = []
        out += [{"summary": "MissedMutant"}] * missed
        out += [{"summary": "CaughtMutant"}] * caught
        out += [{"summary": "Timeout"}] * timeout
        out += [{"summary": "Unviable"}] * unviable
        out += [{"summary": "Success"}]  # the unmutated baseline build — must be ignored
        return {"outcomes": out}

    # --- summarise: counts + caught% (timeout counts as caught; unviable excluded) ----
    s, c, u, t, tot = summarise(doc(missed=3, caught=7, timeout=1, unviable=4))
    assert (s, c, t, u, tot) == (3, 7, 1, 4, 11), (s, c, t, u, tot)
    # caught% = (caught+timeout)/total_viable = 8/11
    killed = c + t
    assert round(100.0 * killed / tot, 2) == round(800.0 / 11, 2)
    # empty viable set -> 0% not a div0
    s0, c0, u0, t0, tot0 = summarise(doc(unviable=5))
    assert tot0 == 0 and (s0, c0) == (0, 0)

    # --- the ratchet: ceiling = surviving + MARGIN, only ever FALLS -------------------
    import tempfile
    with tempfile.TemporaryDirectory() as td:
        # write two crate outcomes under target/mutants/<crate>/outcomes.json shape
        # Each variant gets its OWN parent dir so distinct measurements of the same crate
        # do not overwrite one shared outcomes.json (crate_of reads the leaf dir name, so
        # the crate identity is preserved regardless of the unique parent).
        _n = [0]
        def write(crate, **kw):
            _n[0] += 1
            d = os.path.join(td, f"run{_n[0]}", crate); os.makedirs(d, exist_ok=True)
            p = os.path.join(d, "outcomes.json")
            with open(p, "w") as f: json.dump(doc(**kw), f)
            return p
        base = os.path.join(td, "baseline.json")
        pa = write("sparq-canon", missed=5, caught=30)
        # seed: new crate -> ceiling = 5 + MARGIN
        assert seed([pa], base, allow_higher=False) == 0
        b = load(base)["crates"]["sparq-canon"]
        assert b["ceiling"] == 5 + MARGIN, b
        assert b["measured_surviving"] == 5
        # check at the same level passes
        assert check([pa], base, require_all=False) == 0
        # MORE survivors than ceiling -> FAIL
        pa_worse = write("sparq-canon", missed=5 + MARGIN + 1, caught=30)
        assert check([pa_worse], base, require_all=False) == 1
        # FEWER survivors -> re-seed LOWERS the ceiling
        pa_better = write("sparq-canon", missed=1, caught=34)
        assert seed([pa_better], base, allow_higher=False) == 0
        assert load(base)["crates"]["sparq-canon"]["ceiling"] == 1 + MARGIN
        # a worse measurement does NOT auto-raise the (now lower) ceiling
        assert seed([pa_worse], base, allow_higher=False) == 0
        assert load(base)["crates"]["sparq-canon"]["ceiling"] == 1 + MARGIN
        # ...unless --allow-higher. pa_worse has (5 + MARGIN + 1) survivors, so the
        # raised ceiling is that + MARGIN.
        worse_surv = 5 + MARGIN + 1
        assert seed([pa_worse], base, allow_higher=True) == 0
        assert load(base)["crates"]["sparq-canon"]["ceiling"] == worse_surv + MARGIN
        # excluded crate is never seeded
        pcli = write("sparq-cli", missed=99, caught=0)
        n_before = len(load(base)["crates"])
        assert seed([pcli], base, allow_higher=False) == 0
        assert len(load(base)["crates"]) == n_before, "excluded crate must not seed"
        # --require-all: a baseline crate absent from the run fails
        pother = write("sparq-parse", missed=0, caught=10)
        assert seed([pother], base, allow_higher=False) == 0  # adds sparq-parse
        assert check([pa_better], base, require_all=True) == 1  # sparq-parse missing
        assert check([pa_better], base, require_all=False) == 0  # tolerated without flag

    print("mutants-gate self-test: ALL ASSERTIONS PASSED")
    return 0


if __name__ == "__main__":
    main()
