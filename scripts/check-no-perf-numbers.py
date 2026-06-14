#!/usr/bin/env python3
# [OPUS-4.8] No-hard-coded-performance-numbers check for markdown (bead sq-5fd1).
#
# AGENTS.md house rule: benchmark figures MUST NOT be baked into markdown — they
# drift, and the single source of truth is the generated perf dashboard + the bench
# registry (`bench/benchmarks.toml` / `bench/CATALOG.md`). This script greps tracked
# markdown for measured-throughput / latency / ratio / size-per-triple patterns and
# reports them so reviewers can relocate the figure to bench/research and link it.
#
# STATUS: ADVISORY (run with --advisory in CI — never fails the build) because the
# violation backlog is still widespread (tracked beads: sq-h1s7 / sq-q2em / sq-rt1t /
# sq-puyy / sq-kuqa / sq-p3fk + the research/bench corpus). As crates are de-perf'd
# their READMEs can be moved into --enforce scope. With --enforce it exits non-zero
# on any finding outside the allowlist, so a clean directory can be ratcheted to HARD.
#
# It is deliberately conservative: it allowlists legit, NON-perf numeric facts
# (algorithm constants, platform/spec limits, dataset/fixture sizes, layout
# constants, dates/versions/hashes) so it flags *measured performance*, not "any
# number". False positives are tuned out via ALLOW_LINE_SUBSTRINGS / ALLOW_PATHS.

from __future__ import annotations

import argparse
import os
import re
import subprocess
import sys

# --- perf-figure patterns (case-insensitive). A NUMBER immediately followed by a
#     throughput / latency / ratio / size-per-record unit is treated as a measured
#     performance figure. Spec/algorithm constants are excluded by ALLOW_* below. ---
NUM = r"[0-9][0-9_.,]*"
PERF_PATTERNS = [
    # throughput
    (re.compile(rf"{NUM}\s*(?:[GMK]?B/s|[GMK]B/sec)\b", re.I), "throughput (bytes/s)"),
    (re.compile(rf"{NUM}\s*(?:M|K|G)?(?:triples?|rows?|entries|ops|queries|req)/s(?:ec)?\b", re.I), "throughput (records/s)"),
    (re.compile(rf"{NUM}\s*M/s\b"), "throughput (M/s)"),
    (re.compile(rf"{NUM}\s*ns/op\b", re.I), "latency (ns/op)"),
    # latency (a bare number + time unit, often with /query or /op)
    (re.compile(rf"{NUM}\s*(?:ns|µs|us|ms)\s*/\s*(?:op|query|row|call|item)\b", re.I), "latency (/op)"),
    (re.compile(rf"{NUM}\s*(?:ns|µs|us)\b(?!\w)", re.I), "latency (ns/µs)"),
    # speed-up ratios: "12× faster", "1.27x faster", "3× speedup"
    (re.compile(rf"{NUM}\s*(?:×|x)\s*(?:faster|slower|speed-?up)", re.I), "speed-up ratio"),
    (re.compile(rf"{NUM}\s*(?:×|x)\b(?=.{{0,30}}(?:faster|slower|speedup|throughput|latency))", re.I), "ratio (perf context)"),
    # bytes-per-record layout *measurements* (vs. fixed layout constants, see allow)
    (re.compile(rf"{NUM}\s*B/(?:triple|term|posting|row|node)\b", re.I), "size per record (B/record)"),
    (re.compile(rf"{NUM}\s*bytes?/(?:triple|term|posting|row|node)\b", re.I), "size per record (bytes/record)"),
]

# --- ALLOWLIST: lines containing any of these substrings are skipped entirely.
#     These mark legit non-perf numeric facts or already-relocated references. ---
ALLOW_LINE_SUBSTRINGS = [
    # algorithm / spec constants (not measurements)
    "k1=1.2", "b=0.75", "BM25",
    # platform / spec limits
    "wasm32", "4 GB", "2 GB", "4 GiB",
    # the canonical perf homes — a line that POINTS to them is fine
    "jeswr.github.io/sparq/dev/bench", "benchmarks.toml", "bench/CATALOG.md",
    "perf-baseline.json", "perf dashboard",
    # explicit opt-out marker a reviewer can add on a justified line
    "<!-- perf-ok -->", "perf-ok:",
]

# --- ALLOWLIST: whole paths/prefixes that are EXEMPT from the check.
#     bench/ and research/ are the SANCTIONED homes for measured numbers (AGENTS.md:
#     "Durable knowledge → … a research/ design record"; bench/ holds methodology +
#     baselines). The rule is about not baking numbers into USER-FACING docs. ---
ALLOW_PATH_PREFIXES = [
    "bench/",            # the benchmark corpus — numbers belong here
    "research/",         # design records + measured verdicts — numbers belong here
    "CHANGELOG.md",      # historical record of measured deltas at release time
    "conformance-report.md",
    "inference-conformance-report.md",
    "crates/sparq-conformance/FINDINGS.md",
    "zk/",               # research scaffolds (separate workspaces)
    "vendor/",
    "node_modules/",
    "target/",
    ".claude/",
    ".github/",
]

# Globs of files to scan (markdown only).
SCAN_GLOBS = ["*.md"]


def tracked_markdown() -> list[str]:
    """All git-tracked markdown files (deterministic, no untracked scratch)."""
    out = subprocess.run(
        ["git", "ls-files", *SCAN_GLOBS],
        capture_output=True, text=True, check=True,
    ).stdout
    return [p for p in out.splitlines() if p]


def is_exempt_path(path: str) -> bool:
    return any(path == p or path.startswith(p) for p in ALLOW_PATH_PREFIXES)


class FileReadError(Exception):
    """[OPUS-4.8] Raised when a scanned file cannot be read/decoded.

    Surfaced rather than swallowed: silently skipping an unreadable file would hide
    any perf-number violations inside it and make --enforce non-deterministic.
    """


def scan_file(path: str) -> list[tuple[int, str, str]]:
    findings: list[tuple[int, str, str]] = []
    in_code_fence = False
    try:
        with open(path, encoding="utf-8") as fh:
            for lineno, raw in enumerate(fh, 1):
                line = raw.rstrip("\n")
                # [OPUS-4.8] Strip leading blockquote markers (`>` + spaces, possibly
                # nested e.g. `> > `) BEFORE the fence test, so blockquoted code fences
                # (`> ```lang`, as in docs/upstream-proposals.md) are recognised — else
                # perf-number lines inside them would be mis-scanned as prose.
                stripped = re.sub(r"^(?:\s*>)+\s*", "", line).lstrip()
                # Track fenced code blocks; perf numbers inside example output /
                # console transcripts are illustrative, not doc claims — skip them.
                if stripped.startswith("```") or stripped.startswith("~~~"):
                    in_code_fence = not in_code_fence
                    continue
                if in_code_fence:
                    continue
                if any(sub in line for sub in ALLOW_LINE_SUBSTRINGS):
                    continue
                for pat, label in PERF_PATTERNS:
                    m = pat.search(line)
                    if m:
                        findings.append((lineno, label, m.group(0).strip()))
                        break  # one finding per line is enough to flag it
    except (OSError, UnicodeDecodeError) as e:
        # [OPUS-4.8] Don't swallow the error: surface it so the caller can warn and,
        # in --enforce mode, fail (a skipped file could hide real violations).
        raise FileReadError(f"{path}: could not read: {e}") from e
    return findings


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument(
        "--advisory", action="store_true",
        help="report findings but always exit 0 (default CI mode while the backlog clears)",
    )
    ap.add_argument(
        "--enforce", action="store_true",
        help="exit non-zero if any finding remains (use once a scope is clean to ratchet it HARD)",
    )
    ap.add_argument(
        "paths", nargs="*",
        help="optional explicit paths to scan (default: all git-tracked markdown, "
             "minus the bench/research/changelog allowlisted homes)",
    )
    args = ap.parse_args()

    files = args.paths or [p for p in tracked_markdown() if not is_exempt_path(p)]

    total = 0
    read_errors = 0  # [OPUS-4.8] unreadable files — never silently skipped
    summary_lines: list[str] = []
    for path in sorted(files):
        if not args.paths and is_exempt_path(path):
            continue
        try:
            file_findings = scan_file(path)
        except FileReadError as e:
            # [OPUS-4.8] Warn on stderr always; in --enforce/non-advisory mode this
            # also counts as a failure below so a hidden file can't pass silently.
            read_errors += 1
            print(f"::warning::{e}", file=sys.stderr)
            summary_lines.append(f"- `{path}` — UNREADABLE (counted as a failure under --enforce)")
            continue
        for lineno, label, snippet in file_findings:
            total += 1
            print(f"{path}:{lineno}: perf-number [{label}]: {snippet!r}")
            summary_lines.append(f"- `{path}:{lineno}` — {label}: `{snippet}`")

    # Feed the GitHub step summary when available.
    step_summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if step_summary:
        with open(step_summary, "a", encoding="utf-8") as fh:
            if total == 0 and read_errors == 0:
                fh.write("### no-perf-numbers: clean ✅\n\n"
                         "No hard-coded performance figures found in the scanned docs.\n")
            else:
                fh.write(f"### no-perf-numbers (ADVISORY): {total} finding(s)"
                         f"{f', {read_errors} unreadable file(s)' if read_errors else ''} ⚠️\n\n")
                fh.write("Benchmark figures drift — relocate these to `bench/`/`research/` "
                         "and link the perf dashboard (AGENTS.md house rule). "
                         "Allowlist a justified line with `<!-- perf-ok -->`.\n\n")
                # Cap the inline list so the summary stays readable.
                for ln in summary_lines[:60]:
                    fh.write(ln + "\n")
                if len(summary_lines) > 60:
                    fh.write(f"\n…and {len(summary_lines) - 60} more (see the job log).\n")

    print(f"\nno-perf-numbers: {total} finding(s) across {len(files)} scanned file(s)"
          f"{f'; {read_errors} unreadable file(s)' if read_errors else ''}.")
    # [OPUS-4.8] --advisory always exits 0 (reports only). In --enforce/non-advisory
    # mode, BOTH perf-number findings AND unreadable files are failures — a file we
    # couldn't scan might be hiding a violation, so it must not pass silently.
    if args.enforce and not args.advisory and (total or read_errors):
        if total:
            print("::error::hard-coded performance numbers found (see above) — relocate to bench/research.")
        if read_errors:
            print(f"::error::{read_errors} file(s) could not be read (see warnings) — cannot certify clean.")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
