#!/usr/bin/env python3
# [OPUS-4.8] ADVISORY README length-cap + required-sections check (bead sq-5fd1).
#
# Pairs with the crate-README template from the doc-overhaul design (bead sq-9jw5):
# a concise (<=120-line) per-crate README with three emoji section markers —
# "## 🚀 Quickstart", "## ✨ Features", "## 📚 Learn more" — plus a License line.
# `publish=false` crates get a short stub instead (no section requirement).
#
# STATUS: ADVISORY (run with --advisory). Today most crate READMEs predate the
# template and exceed the cap; the overhaul (sq-9jw5) brings them into line per-crate.
# Until then this check only REPORTS (into the job summary) so the gap stays visible
# without blocking. Once a README adopts the template it passes; once ALL do, flip the
# workflow step to --enforce to ratchet it HARD.

from __future__ import annotations

import argparse
import glob
import os
import re
import sys

MAX_LINES = 120
REQUIRED_SECTIONS = [
    ("🚀", re.compile(r"^##\s*🚀", re.M)),   # Quickstart
    ("✨", re.compile(r"^##\s*✨", re.M)),    # Features
    ("📚", re.compile(r"^##\s*📚", re.M)),    # Learn more
]
# [OPUS-4.8] Require a real License *section*, not any inline mention of the word
# "License" (e.g. "licensed under", a "license" badge). The template's License is a
# heading or a one-line section label at line-start: a markdown heading (`## License`),
# a bold label (`**License**`), or a leading `License` followed by `:`/`—`/`-`. Each
# alternative is anchored to the start of a line (re.M) so prose mentions don't count.
LICENSE_RE = re.compile(
    r"^\s{0,3}(?:#{1,6}\s*License\b"          # markdown heading: ## License
    r"|\*\*License\*\*"                         # bold label: **License**
    r"|License\s*(?:[:—-]|$))",                # one-line section: "License:" / "License —"
    re.M | re.I,
)
# A README that openly declares itself an internal/unpublished stub is exempt from the
# section requirement (the template allows a 15-line stub for publish=false crates).
STUB_MARKERS = ("not published", "internal, not published", "internal crate", "not on crates.io")


def check_readme(path: str) -> list[str]:
    issues: list[str] = []
    try:
        text = open(path, encoding="utf-8").read()
    except OSError as e:
        return [f"could not read: {e}"]
    n_lines = text.count("\n") + (0 if text.endswith("\n") else 1)

    is_stub = any(m in text.lower() for m in STUB_MARKERS)

    if is_stub:
        if n_lines > 30:
            issues.append(f"declared a stub but {n_lines} lines (stub target: <=30)")
        return issues

    if n_lines > MAX_LINES:
        issues.append(f"{n_lines} lines (template cap: <={MAX_LINES})")
    for emoji, rx in REQUIRED_SECTIONS:
        if not rx.search(text):
            issues.append(f"missing '## {emoji} …' section")
    if not LICENSE_RE.search(text):
        issues.append("no License section/line")
    return issues


def main() -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--advisory", action="store_true",
                    help="report only, always exit 0 (default while sq-9jw5 lands)")
    ap.add_argument("--enforce", action="store_true",
                    help="exit non-zero on any template deviation (ratchet to HARD)")
    ap.add_argument("paths", nargs="*",
                    help="explicit README paths (default: crates/*/README.md)")
    args = ap.parse_args()

    paths = args.paths or sorted(glob.glob("crates/*/README.md"))
    total = 0
    summary: list[str] = []
    for path in paths:
        issues = check_readme(path)
        if issues:
            total += len(issues)
            for it in issues:
                print(f"{path}: {it}")
                summary.append(f"- `{path}` — {it}")

    step_summary = os.environ.get("GITHUB_STEP_SUMMARY")
    if step_summary:
        with open(step_summary, "a", encoding="utf-8") as fh:
            if total == 0:
                fh.write("### readme-template: all conform ✅\n")
            else:
                fh.write(f"### readme-template (ADVISORY): {total} deviation(s) ⚠️\n\n")
                fh.write("Crate READMEs should follow the concise template from sq-9jw5 "
                         f"(<= {MAX_LINES} lines; 🚀 Quickstart / ✨ Features / 📚 Learn more; "
                         "License). The overhaul (sq-9jw5) brings them into line.\n\n")
                for ln in summary[:60]:
                    fh.write(ln + "\n")

    print(f"\nreadme-template: {total} deviation(s) across {len(paths)} README(s).")
    if total and args.enforce and not args.advisory:
        print("::error::crate READMEs deviate from the template (see above).")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
