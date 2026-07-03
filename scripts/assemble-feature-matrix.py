#!/usr/bin/env python3
# Assemble the feature-matrix `opt-in-features` strategy matrix from the per-crate
# fragment files in .github/feature-matrix.d/*.yml and emit it as ONE JSON object,
# `{"include": [ {name, crate, features, test}, ... ]}`, ready for `fromJSON` in the
# workflow. [OPUS-4.8] bead sq-ibrze.
#
# WHY: the opt-in feature matrix used to be a single static `include:` list inside
# feature-matrix.yml. Every opt-in-feature PR appended a leg to that ONE shared list,
# so concurrent PRs collided textually on adjacent additions (this cost ~8 merge-fixer
# passes in one session). Splitting the list into per-crate fragment files means adding
# a crate's leg edits only that crate's fragment — concurrent PRs for DIFFERENT crates
# never touch the same file, so they auto-merge.
#
# GATE-CRITICAL CONTRACT (do NOT relax): the assembled leg's check-run NAME in CI is
# `opt-in <name>` (the job is `name: opt-in ${{ matrix.name }}`). The single required
# `ci-summary / gate` aggregator discovers every leg as a REQUIRED check BY NAME, so the
# set of emitted `name` values MUST stay byte-identical to the pre-split static list, or
# branch protection stops recognising required checks. This script therefore:
#   - reads fragments in DETERMINISTIC (sorted-filename) order so the emitted order is
#     stable run-to-run (order does not change check-run NAMES, but determinism keeps
#     logs/diffs clean);
#   - validates every leg has exactly {name, crate, features, test} with the right types
#     and a NON-EMPTY `features` (a matrix leg cannot pass an empty `--features`); and
#   - FAILS LOUD on a duplicate `name` across fragments (two legs with the same check-run
#     name would collapse into one gating check — a silent coverage hole).
#
# Usage:
#   python3 scripts/assemble-feature-matrix.py            # prints {"include": [...]} JSON
#   python3 scripts/assemble-feature-matrix.py --names    # prints the sorted check-run
#                                                         #   names ("opt-in <name>"), one
#                                                         #   per line (for the gate-name
#                                                         #   preservation proof / tests)
#   ... --select-mode <mode> --affected '<json array>'    # [FABLE-5] sq-fmx4u.3: when
#                                                         #   mode == "selected", keep only
#                                                         #   legs whose crate is affected;
#                                                         #   ANY other/malformed input
#                                                         #   fail-closes to the FULL set
# Exit non-zero (with a diagnostic on stderr) on any malformed fragment.

import glob
import json
import os
import sys

try:
    import yaml
except ImportError:  # pragma: no cover - CI always has pyyaml (actions/setup or apt)
    sys.stderr.write("error: PyYAML is required (pip install pyyaml)\n")
    sys.exit(2)

FRAGMENT_DIR = os.path.join(
    os.path.dirname(os.path.dirname(os.path.abspath(__file__))),
    ".github",
    "feature-matrix.d",
)

REQUIRED_KEYS = {"name", "crate", "features", "test"}


def load_legs():
    fragments = sorted(glob.glob(os.path.join(FRAGMENT_DIR, "*.yml")))
    if not fragments:
        sys.stderr.write(f"error: no fragment files found under {FRAGMENT_DIR}\n")
        sys.exit(1)
    legs = []
    seen_names = {}
    for path in fragments:
        rel = os.path.relpath(path)
        with open(path, "r", encoding="utf-8") as fh:
            data = yaml.safe_load(fh)
        if data is None:
            # An empty (comment-only) fragment is allowed — a crate placeholder.
            continue
        if not isinstance(data, list):
            sys.stderr.write(
                f"error: {rel}: top level must be a YAML list of legs, got "
                f"{type(data).__name__}\n"
            )
            sys.exit(1)
        for idx, leg in enumerate(data):
            where = f"{rel}[{idx}]"
            if not isinstance(leg, dict):
                sys.stderr.write(f"error: {where}: leg must be a mapping\n")
                sys.exit(1)
            keys = set(leg.keys())
            if keys != REQUIRED_KEYS:
                missing = REQUIRED_KEYS - keys
                extra = keys - REQUIRED_KEYS
                msg = []
                if missing:
                    msg.append(f"missing {sorted(missing)}")
                if extra:
                    msg.append(f"unexpected {sorted(extra)}")
                sys.stderr.write(f"error: {where}: bad keys ({'; '.join(msg)})\n")
                sys.exit(1)
            if not isinstance(leg["name"], str) or not leg["name"].strip():
                sys.stderr.write(f"error: {where}: `name` must be a non-empty string\n")
                sys.exit(1)
            if not isinstance(leg["crate"], str) or not leg["crate"].strip():
                sys.stderr.write(f"error: {where}: `crate` must be a non-empty string\n")
                sys.exit(1)
            if not isinstance(leg["features"], str) or not leg["features"].strip():
                # cargo rejects a bare `--features` with no value, so an empty feature
                # set is never valid for a leg (the default build belongs in ci.yml's
                # workspace lane, not here).
                sys.stderr.write(
                    f"error: {where}: `features` must be a non-empty comma list\n"
                )
                sys.exit(1)
            if not isinstance(leg["test"], bool):
                sys.stderr.write(f"error: {where}: `test` must be a boolean\n")
                sys.exit(1)
            name = leg["name"]
            if name in seen_names:
                sys.stderr.write(
                    f"error: duplicate leg name {name!r} in {where} "
                    f"(first seen in {seen_names[name]}); two legs with the same "
                    f"check-run name would collapse into one gating check\n"
                )
                sys.exit(1)
            seen_names[name] = where
            legs.append(
                {
                    "name": leg["name"],
                    "crate": leg["crate"],
                    "features": leg["features"],
                    "test": leg["test"],
                }
            )
    return legs


def filter_legs_by_selection(legs, select_mode, affected_json):
    """[FABLE-5] sq-fmx4u.3 (design §5.2): change-based selection over the leg list.

    Keep only the legs whose `crate` is in the affected closure — but ONLY when
    the selection pre-job says `--select-mode selected`. Every other input is
    FAIL-CLOSED to the FULL leg set (running more is always sound, design §2/§4.3):
      * select_mode empty / "shadow" / "full" / anything else  => full set
      * affected missing, unparsable, or not a list of strings => full set
        (with a loud stderr warning — that combination means a wiring bug, and
        the sound degradation is the status quo, never a skip).
    An affected closure of [] legitimately yields ZERO legs; the workflow's
    `setup` job emits a `legs` count so the matrix job skips instead of
    exploding on an empty `include`. Note: matrix-key selection cannot be a
    job-level `if:` on GitHub Actions — the `matrix` context is not available
    there (docs: contexts availability), which is why this filtering happens at
    assembly time. The gate aggregator discovers checks by polling, so an
    unassembled leg is simply absent (never an "expected but missing" hang);
    requiredness continues to flow through `ci-summary / gate`.
    """
    if select_mode != "selected":
        return legs
    affected = None
    if affected_json:
        try:
            affected = json.loads(affected_json)
        except json.JSONDecodeError:
            affected = None
    if not isinstance(affected, list) or not all(isinstance(a, str) for a in affected):
        sys.stderr.write(
            "warning: --select-mode selected but --affected is missing/malformed; "
            "FAILING CLOSED to the full leg set (sq-fmx4u.3, design §4.3)\n"
        )
        return legs
    keep = set(affected)
    return [leg for leg in legs if leg["crate"] in keep]


def _flag_value(argv, flag):
    """Value of `--flag value` in argv, or None."""
    for i, a in enumerate(argv):
        if a == flag and i + 1 < len(argv):
            return argv[i + 1]
    return None


def main():
    legs = load_legs()
    if "--names" in sys.argv[1:]:
        # The golden gate-name proof ALWAYS dumps the full set — selection must
        # never make the byte-identical name contract unverifiable.
        for name in sorted(f"opt-in {leg['name']}" for leg in legs):
            print(name)
        return
    legs = filter_legs_by_selection(
        legs,
        _flag_value(sys.argv[1:], "--select-mode"),
        _flag_value(sys.argv[1:], "--affected"),
    )
    # Emit a single-line JSON object so the workflow can capture it with
    # `echo "matrix=$(...)" >> "$GITHUB_OUTPUT"` and feed `fromJSON(... .matrix)`.
    print(json.dumps({"include": legs}, ensure_ascii=False, separators=(",", ":")))


if __name__ == "__main__":
    main()
