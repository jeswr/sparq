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


def main():
    legs = load_legs()
    if "--names" in sys.argv[1:]:
        for name in sorted(f"opt-in {leg['name']}" for leg in legs):
            print(name)
        return
    # Emit a single-line JSON object so the workflow can capture it with
    # `echo "matrix=$(...)" >> "$GITHUB_OUTPUT"` and feed `fromJSON(... .matrix)`.
    print(json.dumps({"include": legs}, ensure_ascii=False, separators=(",", ":")))


if __name__ == "__main__":
    main()
