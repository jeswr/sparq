#!/usr/bin/env python3
# [OPUS-4.8] Criterion -> github-action-benchmark TSV harvester for the ZK benches
# (sq dashboard-data-feed). Walks a criterion output tree and prints one
# `<metric_name>\t<unit>\t<value>` line per benchmark, so scripts/ci-bench.sh can
# `add` the ZK timing series into the customSmallerIsBetter feed (smaller-is-better:
# the value is the criterion MEAN point estimate, converted to microseconds).
#
# Both ZK criterion suites (bench/zk commitment pipeline, bench/zk-trace per-operator
# capture) are STANDALONE cargo projects whose `cargo bench` writes
#   target/criterion/<group>/<bench_fn>/<param>/new/estimates.json
# with mean.point_estimate in NANOSECONDS. Criterion sanitises any `/` inside the
# group/bench-id into `_`, so the relative path components ARE the metric tokens.
#
# Usage:
#   criterion_harvest.py <criterion_dir> <metric_prefix>
#     <criterion_dir>  target/criterion of a built+run ZK bench project
#     <metric_prefix>  e.g. "zk_commit" or "zk_trace" — the leading token MUST be
#                      `zk` so the dashboard's Zero-knowledge family prefix routing
#                      (familyOf: name.split('_')[0]) buckets the metric correctly.
#
# Emits `<prefix>_<path tokens>_us`  unit=us  value=<mean ns / 1000>. Deterministic
# key order (sorted) so re-runs diff cleanly. NON-CANONICAL timing (wall-clock; the
# value tracks a cross-commit trend on the dashboard, it is NOT a gate).
import json
import os
import re
import sys


def slug(s):
    # path component -> metric token: lowercase, non-alnum collapsed to `_`.
    return re.sub(r"[^a-z0-9]+", "_", s.lower()).strip("_")


def main():
    if len(sys.argv) != 3:
        sys.stderr.write("usage: criterion_harvest.py <criterion_dir> <metric_prefix>\n")
        return 0  # absent/empty tree is not an error — the suite is just skipped.
    root, prefix = sys.argv[1], sys.argv[2]
    if not os.path.isdir(root):
        return 0
    rows = {}
    for dirpath, _dirs, files in os.walk(root):
        if "estimates.json" not in files:
            continue
        # we only want the freshly-measured estimates (…/new/estimates.json), not …/base.
        if os.path.basename(dirpath) != "new":
            continue
        rel = os.path.relpath(os.path.dirname(dirpath), root)
        # rel = "<group>/<bench_fn>/<param>" (criterion already flattened internal `/`).
        tokens = [slug(p) for p in rel.split(os.sep) if slug(p)]
        if not tokens:
            continue
        try:
            with open(os.path.join(dirpath, "estimates.json")) as fh:
                est = json.load(fh)
            mean_ns = float(est["mean"]["point_estimate"])
        except (OSError, ValueError, KeyError):
            continue
        name = "%s_%s_us" % (prefix, "_".join(tokens))
        rows[name] = round(mean_ns / 1000.0, 3)  # ns -> us
    for name in sorted(rows):
        sys.stdout.write("%s\tus\t%s\n" % (name, rows[name]))
    return 0


if __name__ == "__main__":
    sys.exit(main())
