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
import argparse, json, os, re, sys

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
    name = os.path.basename(parent)
    # [SONNET-4.6] sq-has2g: a SUB-SHARDED crate's dir carries the shard suffix the CI lane
    # appends (target/mutants/sparq-engine-shard-2-6/...). Strip it so all of a crate's
    # shards fall under one key and measured_rows() can aggregate them. Only reached when
    # the outcomes doc has no Mutant scenario to read the package from.
    return re.sub(r"-shard-\d+-\d+$", "", name)


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


def completeness(outcomes_path, completed):
    """[SONNET-4.6] sq-has2g: TRUNCATION DETECTION -> (planned, completed), planned=None
    when it cannot be determined.

    cargo-mutants writes mutants.out/outcomes.json INCREMENTALLY, and its rollup counts
    (`total_mutants`/`missed`/`caught`/…) describe only what has finished so far. So a run
    killed by the job timeout or a runner shutdown — the normal outcome for the heaviest
    crates, which is exactly why sparq-engine is still unseeded — leaves behind a perfectly
    well-formed artifact that is INDISTINGUISHABLE from a complete run of a smaller crate.
    Seeding from one would commit a ceiling derived from the handful of mutants that
    happened to finish: far too tight, and it would read as an excellent result.

    mutants.out/mutants.json is the FULL planned list for the run (already written up-front,
    and shard-aware — it holds only this `--shard k/n` slice), so comparing its length to
    the number of finished outcomes detects the truncation. It is only advisory here: when
    the file is absent (older artifacts, or an upload that took outcomes.json alone) we
    return None and fall back to the previous behaviour rather than guessing.
    """
    planned_path = os.path.join(os.path.dirname(os.path.abspath(outcomes_path)),
                                "mutants.json")
    if not os.path.exists(planned_path):
        return None, completed
    try:
        planned = load(planned_path)
    except (ValueError, OSError):
        return None, completed
    if not isinstance(planned, list):
        return None, completed
    return len(planned), completed


def measured_rows(outcome_paths):
    """Build {crate: {surviving, caught, unviable, timeout, caught_pct, shards}} from
    outcomes files, skipping EXCLUDED crates.

    [SONNET-4.6] sq-has2g: SHARD AGGREGATION. `cargo mutants --shard k/n` slices a crate's
    mutant list into n disjoint parts, and the nightly runs each slice as its OWN matrix
    job writing its OWN outcomes.json (sparq-reason is already 3-sharded; sparq-engine is
    sharded here because it is the largest crate). Those per-shard files all report the
    SAME package, so the previous `rows[crate] = {...}` was LAST-WRITE-WINS: seeding a
    sharded crate from its n artifacts recorded only the final shard's counts and would
    have committed a ceiling roughly 1/n of the true surviving count — a bogus, far too
    tight floor that reads as a strong result. Since the shards partition the mutant set,
    the honest whole-crate measurement is the per-category SUM, so accumulate. `shards`
    records how many outcomes files contributed, which lets --check flag a PARTIAL
    measurement instead of silently under-reporting.
    """
    rows = {}
    seen_paths = set()
    for p in outcome_paths:
        # Guard against the same artifact being passed twice (e.g. a glob that expands
        # over both a copy and the original): summing would double-count it.
        real = os.path.realpath(p)
        if real in seen_paths:
            print(f"  skip {p} (duplicate path already counted)")
            continue
        seen_paths.add(real)
        doc = load(p)
        crate = crate_of(p, doc)
        if crate in EXCLUDED:
            print(f"  skip {crate} (excluded: see baseline _comment)")
            continue
        surviving, caught, unviable, timeout, total = summarise(doc)
        acc = rows.setdefault(crate, {
            "surviving": 0, "caught": 0, "timeout": 0, "unviable": 0, "shards": 0,
            "truncated": [],
        })
        planned, done = completeness(p, surviving + caught + timeout + unviable)
        if planned is not None and done < planned:
            acc["truncated"].append((os.path.basename(os.path.dirname(real)), done, planned))
        acc["surviving"] += surviving
        acc["caught"] += caught
        acc["timeout"] += timeout
        acc["unviable"] += unviable
        acc["shards"] += 1
    for crate, acc in rows.items():
        total = acc["surviving"] + acc["caught"] + acc["timeout"]
        killed = acc["caught"] + acc["timeout"]
        acc["caught_pct"] = round(100.0 * killed / total, 2) if total else 0.0
        if acc["shards"] > 1:
            print(f"  agg  {crate:<20} summed {acc['shards']} shard outcome file(s)")
        for name, done, planned in acc["truncated"]:
            print(f"  TRUNC {crate:<19} {name}: only {done}/{planned} mutants completed")
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
        "[SONNET-4.6] sq-has2g SHARDED CRATES (`shards: n` on the entry — sparq-reason, "
        "sparq-engine): the nightly splits these across n `cargo mutants --shard k/n` matrix "
        "jobs, one outcomes.json artifact each. Pass ALL n files to a SINGLE --seed "
        "invocation (scripts/mutants-gate.py --seed target/mutants/<crate>-shard-*/…/"
        "outcomes.json) — the shards partition the mutant set, so the whole-crate figure is "
        "their SUM and --seed aggregates them. Seeding from ONE shard would commit a ceiling "
        "~1/n of the truth. --check on a partial shard set is tolerated (it can only "
        "under-report, never spuriously fail) and prints a PARTIAL marker.",
    ]


def seed(outcome_paths, baseline_path, allow_higher, allow_truncated=False):
    measured = measured_rows(outcome_paths)
    # [SONNET-4.6] sq-has2g: REFUSE to seed from a truncated run. cargo-mutants writes
    # outcomes.json incrementally, so a job killed at its timeout yields a well-formed but
    # PARTIAL artifact; seeding from it commits a ceiling based only on the mutants that
    # happened to finish. That is the single most likely way this baseline acquires a
    # dishonest number, and it fails CLOSED here rather than being reviewable-in-diff (the
    # committed entry looks entirely plausible). --allow-truncated is the deliberate escape
    # hatch, and it is never appropriate for seeding a ceiling that will later gate.
    truncated = {c: r["truncated"] for c, r in measured.items() if r["truncated"]}
    if truncated and not allow_truncated:
        for crate, items in sorted(truncated.items()):
            for name, done, planned in items:
                print(f"::error::{crate}: {name} completed only {done}/{planned} mutants — "
                      f"the run was cut short (job timeout / runner shutdown), so its "
                      f"surviving count is NOT the whole-crate figure")
        print(f"\nrefusing to seed from {sum(len(v) for v in truncated.values())} truncated "
              f"run(s): re-run to completion, or pass --allow-truncated if you have "
              f"independently established the numbers are whole")
        return 1
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
        # [SONNET-4.6] sq-has2g: record the shard count for a SUB-SHARDED crate so the
        # committed entry states, in the reviewed diff, that it is the SUM of n disjoint
        # `--shard k/n` runs rather than one whole-crate run — and so --check can warn when
        # a later run supplies fewer shards than the entry was seeded from. Omitted for the
        # single-run crates so their existing entries stay byte-identical.
        if row["shards"] > 1:
            entry["shards"] = row["shards"]
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
    fails, missing, oks, partial = [], [], [], []
    for crate, centry in sorted(ceilings.items()):
        ceiling = centry["ceiling"] if isinstance(centry, dict) else centry
        row = measured.get(crate)
        if row is None:
            missing.append(crate); continue
        # [SONNET-4.6] sq-has2g: a SUB-SHARDED crate is checked per shard job, so this run
        # may hold only a SLICE of the crate's mutants. Its surviving count is then a lower
        # bound on the whole-crate figure and comparing it to the whole-crate ceiling is
        # unsound in the SAFE direction — it can say "ok" but never spuriously FAIL. Report
        # it as PARTIAL so a green line is not read as whole-crate evidence.
        seeded_shards = centry.get("shards", 1) if isinstance(centry, dict) else 1
        if row["shards"] < seeded_shards:
            partial.append((crate, row["shards"], seeded_shards))
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
        of_n = f" [{r['shards']} shard(s) only]" if r["shards"] > 1 else ""
        print(f"  new  {crate:<20} surviving {r['surviving']:>3} (UNSEEDED — run --seed){of_n}")
    for crate, got, want in partial:
        print(f"  ??   {crate:<20} PARTIAL: {got}/{want} shard(s) measured — the surviving "
              f"count above is a LOWER BOUND, not whole-crate evidence")
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
    ap.add_argument("--allow-truncated", action="store_true",
                    help="permit --seed from a run that did not finish all its planned "
                         "mutants (detected via mutants.json); normally refused")
    a = ap.parse_args()
    baseline = os.path.abspath(a.baseline)
    if a.seed:
        sys.exit(seed(a.outcomes, baseline, a.allow_higher, a.allow_truncated))
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

    # --- [SONNET-4.6] sq-has2g: SHARD AGGREGATION -------------------------------------
    # `cargo mutants --shard k/n` partitions a crate's mutant list across n jobs, each
    # writing its own outcomes.json for the SAME package. Those must SUM, not overwrite:
    # the old last-write-wins behaviour seeded a ceiling ~1/n of the true surviving count.
    with tempfile.TemporaryDirectory() as td:
        _m = [0]
        def shard(crate, missed=0, caught=0, timeout=0, unviable=0):
            """Write one shard's outcomes.json in cargo-mutants' real shape: a Baseline
            entry plus one Mutant-scenario entry per mutant, each stamping the package so
            crate_of() resolves every shard of a crate to the same key."""
            _m[0] += 1
            d = os.path.join(td, f"{crate}-shard-{_m[0]}"); os.makedirs(d, exist_ok=True)
            p = os.path.join(d, "outcomes.json")
            outcomes = [{"scenario": "Baseline", "summary": "Success"}]
            for summary, n in (("MissedMutant", missed), ("CaughtMutant", caught),
                               ("Timeout", timeout), ("Unviable", unviable)):
                outcomes += [{"scenario": {"Mutant": {"package": crate}},
                              "summary": summary}] * n
            with open(p, "w") as f: json.dump({"outcomes": outcomes}, f)
            return p

        # three disjoint shards of one crate: 4+3+2 survivors == 9 whole-crate survivors
        s1 = shard("sparq-engine", missed=4, caught=10)
        s2 = shard("sparq-engine", missed=3, caught=20)
        s3 = shard("sparq-engine", missed=2, caught=30)
        rows = measured_rows([s1, s2, s3])
        assert set(rows) == {"sparq-engine"}, rows
        assert rows["sparq-engine"]["surviving"] == 9, rows
        assert rows["sparq-engine"]["caught"] == 60, rows
        assert rows["sparq-engine"]["shards"] == 3, rows

        # the seeded ceiling reflects the SUMMED survivors (9 + MARGIN), not the last
        # shard's 2 — the exact bug this guards.
        base2 = os.path.join(td, "baseline2.json")
        assert seed([s1, s2, s3], base2, allow_higher=False) == 0
        e = load(base2)["crates"]["sparq-engine"]
        assert e["ceiling"] == 9 + MARGIN, e
        assert e["measured_surviving"] == 9 and e["shards"] == 3, e

        # a duplicate path must NOT double-count
        assert measured_rows([s1, s2, s3, s2])["sparq-engine"]["surviving"] == 9

        # --check with ALL shards is exact, passes at the seeded level, and is NOT partial
        import contextlib, io
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc_full = check([s1, s2, s3], base2, require_all=False)
        assert rc_full == 0 and "PARTIAL" not in buf.getvalue(), buf.getvalue()
        # ...and one shard alone under-reports (9 -> 4): tolerated (it can only under-report,
        # so it never spuriously FAILS) but it MUST be flagged PARTIAL rather than printed as
        # a clean "ok" that reads as whole-crate evidence.
        buf = io.StringIO()
        with contextlib.redirect_stdout(buf):
            rc_part = check([s1], base2, require_all=False)
        out = buf.getvalue()
        assert rc_part == 0, out
        assert "PARTIAL: 1/3 shard(s) measured" in out, out
        assert "LOWER BOUND" in out, out

        # a single-run crate records NO `shards` key, keeping existing entries unchanged
        solo = shard("sparq-hdt", missed=1, caught=5)
        assert seed([solo], base2, allow_higher=False) == 0
        assert "shards" not in load(base2)["crates"]["sparq-hdt"]

        # --- TRUNCATION: a run cut short by the job timeout must NOT seed a ceiling -----
        # cargo-mutants writes outcomes.json incrementally, so a killed job leaves a
        # well-formed PARTIAL artifact. mutants.json (the full planned list) reveals it.
        def plan(outcomes_path, n):
            with open(os.path.join(os.path.dirname(outcomes_path), "mutants.json"), "w") as f:
                json.dump([{"package": "sparq-engine"}] * n, f)

        t1 = shard("sparq-engine", missed=4, caught=10)   # 14 completed
        plan(t1, 14)                                       # ...of 14 planned -> COMPLETE
        assert not measured_rows([t1])["sparq-engine"]["truncated"]
        base3 = os.path.join(td, "baseline3.json")
        assert seed([t1], base3, allow_higher=False) == 0, "complete run must seed"

        t2 = shard("sparq-engine", missed=1, caught=3)     # only 4 completed
        plan(t2, 400)                                      # ...of 400 planned -> TRUNCATED
        assert measured_rows([t2])["sparq-engine"]["truncated"], "truncation must be seen"
        base4 = os.path.join(td, "baseline4.json")
        assert seed([t2], base4, allow_higher=False) == 1, "truncated run must NOT seed"
        assert not os.path.exists(base4), "a refused seed must not write a baseline"
        # the escape hatch still works, deliberately
        assert seed([t2], base4, allow_higher=False, allow_truncated=True) == 0
        assert load(base4)["crates"]["sparq-engine"]["ceiling"] == 1 + MARGIN

        # truncation in ANY contributing shard blocks the whole crate's seed — otherwise a
        # 7-of-8-shards-complete engine sweep would quietly commit a short count.
        ok1 = shard("sparq-engine", missed=2, caught=8); plan(ok1, 10)
        base5 = os.path.join(td, "baseline5.json")
        assert seed([ok1, t2], base5, allow_higher=False) == 1
        assert seed([ok1], base5, allow_higher=False) == 0  # the good shard alone is fine

        # absent mutants.json (older artifact) -> undetectable, fall back to seeding
        t3 = shard("sparq-engine", missed=1, caught=1)
        assert completeness(t3, 2) == (None, 2)
        base6 = os.path.join(td, "baseline6.json")
        assert seed([t3], base6, allow_higher=False) == 0

        # crate_of() strips the CI lane's -shard-k-n dir suffix when the doc carries no
        # package (all-unviable edge case), so shards still collapse to one key.
        d3 = os.path.join(td, "target", "mutants", "sparq-engine-shard-2-6", "mutants.out")
        os.makedirs(d3)
        p3 = os.path.join(d3, "outcomes.json")
        with open(p3, "w") as f: json.dump({"outcomes": []}, f)
        assert crate_of(p3, {"outcomes": []}) == "sparq-engine", crate_of(p3, {})

    print("mutants-gate self-test: ALL ASSERTIONS PASSED")
    return 0


if __name__ == "__main__":
    main()
