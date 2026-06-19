#!/usr/bin/env python3
# [OPUS-4.8] Terminology HARD gate (bead sq-2p8z / issue #812).
#
# WHAT: greps the user-facing + governance markdown surface for the deprecated,
# community-era RDF/SPARQL feature names and FAILS the build on a hit. The feature
# historically nicknamed "RDF-star" / "RDF*" is STANDARDISED in **RDF 1.2**; the
# construct is a **triple term** (with **reifiers** / **reified triples** and
# `rdf:reifies`). The community spellings are imprecise on two counts — they name a
# non-spec version AND imply the old subject+object `<<s p o>>` model rather than the
# RDF 1.2 object-position-only `<<( s p o )>>` triple-term model. The single source of
# truth for the preferred wording is `skills/terminology/SKILL.md`.
#
# BANNED (case-insensitive): RDF-star, RDF*, RDF star, SPARQL-star, SPARQL*, SPARQL
# star, quoted triple(s), embedded triple(s).
#
# ALLOWED (built-in exemptions, see skills/terminology/SKILL.md § Allowed exceptions):
#   - the W3C "RDF-star Working Group" / "RDF-star Community Group" proper nouns,
#   - the Hartig & Thompson paper title "Foundations of RDF★ and SPARQL★",
#   - third-party product/doc names (RDF4J `rdfstar`, GraphDB `rdf-sparql-star`,
#     Jena `rdf-star`, …) and any URL/identifier that literally contains the token,
#   - any line carrying an inline `terminology-allow: <why>` justification marker.
# The Unicode-star forms (RDF★/SPARQL★) are NOT in the banned set — they only ever
# appear in the fixed prior-art paper title, which is an allowed citation.
#
# SCOPE: the SAME markdown doc surface the other docs-quality HARD gates use — root
# *.md + skills/ + docs/ + crates/ (+ book/, the published guide). research/ is the
# sanctioned home for design records that DISCUSS the historical model and competitor
# support, so it is path-exempt (the same reasoning the privacy gate path-excludes the
# audit docs): those docs cite prior art and other systems' real doc names by design.
#
# Deterministic, grep-only, no network. HARD: the docs-quality `terminology` job name
# carries no "advisory"/"informational" token, so the ci-summary aggregator gates on it.
#
# Usage:
#   python3 scripts/check-terminology.py            # glob the tracked doc surface
#   python3 scripts/check-terminology.py FILE ...   # explicit files
#   python3 scripts/check-terminology.py --self-test # hermetic both-direction self-test

from __future__ import annotations

import re
import subprocess
import sys

ALLOW_MARKER = "terminology-allow"

# --- banned spellings (case-insensitive). Each is (regex, human label). ---
# Word-boundary anchored so "GeoSPARQL", "Text-to-SPARQL", a bolded "**…RDF**", an
# italic-closer "*…SPARQL*" etc. do NOT match. `RDF\*` / `SPARQL\*` require the star to
# be the feature suffix (followed by a separator or the historical "RDF*/SPARQL*" pair),
# never a markdown emphasis marker glued to following bold/italic text.
BANNED = [
    (re.compile(r"\bRDF[- ]star\b", re.I), "RDF-star (say: RDF 1.2)"),
    (re.compile(r"\bSPARQL[- ]star\b", re.I), "SPARQL-star (say: SPARQL 1.2)"),
    # `RDF*` / `SPARQL*` as a feature name. ONLY the paired community form (`RDF*/SPARQL*`
    # or `SPARQL*/RDF*`) and `RDF*`-immediately-followed-by-`/SPARQL` fire — a bare
    # `SPARQL*` / `RDF*` before whitespace is, in this corpus, an italic-CLOSER for an
    # emphasised "*…SPARQL*" / "*…RDF*" run (e.g. "real *SPARQL* against") and must NOT
    # match; markdown bold `RDF**`/`SPARQL**` is likewise excluded.
    (re.compile(r"\bRDF\*/SPARQL\*?"), "RDF*/SPARQL* (say: RDF 1.2 / SPARQL 1.2)"),
    (re.compile(r"\bSPARQL\*/RDF\*?"), "SPARQL*/RDF* (say: SPARQL 1.2 / RDF 1.2)"),
    (re.compile(r"\bquoted triples?\b", re.I), "quoted triple (say: triple term)"),
    (re.compile(r"\bembedded triples?\b", re.I), "embedded triple (say: triple term)"),
]

# --- ALLOWED: a line is exempt if it matches any of these (the documented exceptions).
#     Kept as case-insensitive regexes so the proper noun / title / product-doc name is
#     recognised wherever it appears on the line. ---
ALLOW_PATTERNS = [
    re.compile(r"rdf[- ]star\s+(?:working|community)\s+group", re.I),  # the W3C bodies
    re.compile(r"rdf[- ]star[- ](?:wg|cg)\b", re.I),                   # their abbreviations
    re.compile(r"foundations of rdf", re.I),                          # Hartig paper title
    re.compile(r"reification done right", re.I),                       # paper subtitle
    # third-party product / doc names + any URL or path containing the token.
    re.compile(r"rdf-?sparql-?star", re.I),  # GraphDB doc page name
    re.compile(r"/rdfstar\b", re.I),         # RDF4J doc path
    re.compile(r"documentation/rdf-star", re.I),  # Jena doc path
    re.compile(r"rdf-star-wg", re.I),        # the WG GitHub repo
    re.compile(r"https?://\S*rdf-?star", re.I),  # any URL containing the token
    re.compile(r"https?://\S*sparql-?star", re.I),
    # explicit per-line opt-out the author adds with a reason.
    re.compile(re.escape(ALLOW_MARKER) + r"\s*:", re.I),
]

# --- path prefixes that are EXEMPT (the sanctioned homes for historical discussion). ---
ALLOW_PATH_PREFIXES = (
    "research/",          # design records: discuss the historical model + competitors
    "scripts/",           # this gate + its self-tests reference the banned tokens
    "skills/terminology", # the guide itself lists the banned spellings to ban them
    ".github/",           # workflow comments may name the gate
)


def _tracked_markdown() -> list[str]:
    """The HARD doc surface: root *.md + skills/ + docs/ + crates/ + book/ (markdown)."""
    out = subprocess.run(
        [
            "git", "ls-files",
            "*.md",
            ":!:research/**", ":!:bench/**", ":!:js/**",
            ":!:.beads/**", ":!:.claude/**", ":!:.github/**",
            ":!:vendor/**", ":!:**/vendor/**",
        ],
        capture_output=True, text=True, check=True,
    )
    return sorted(set(p for p in out.stdout.splitlines() if p))


def _exempt_path(path: str) -> bool:
    return any(path.startswith(p) for p in ALLOW_PATH_PREFIXES)


def _line_allowed(line: str) -> bool:
    return any(p.search(line) for p in ALLOW_PATTERNS)


def scan_file(path: str) -> list[tuple[int, str, str]]:
    findings: list[tuple[int, str, str]] = []
    try:
        with open(path, encoding="utf-8") as fh:
            lines = fh.readlines()
    except (OSError, UnicodeDecodeError):
        return findings
    for n, line in enumerate(lines, 1):
        if _line_allowed(line):
            continue
        for rx, label in BANNED:
            if rx.search(line):
                findings.append((n, label, line.rstrip("\n")))
                break
    return findings


def run(paths: list[str]) -> int:
    if not paths:
        paths = _tracked_markdown()
    total = 0
    for path in paths:
        if _exempt_path(path):
            continue
        for n, label, line in scan_file(path):
            total += 1
            print(f"{path}:{n}: deprecated terminology — {label}")
            print(f"    {line.strip()}")
    if total:
        print(
            f"\n{total} deprecated-terminology hit(s). "
            "Use the spec-correct term (skills/terminology/SKILL.md): RDF 1.2 / "
            "SPARQL 1.2 / triple term / reifier / reified triple. For a legitimate "
            f"historical or proper-noun mention, add an inline `{ALLOW_MARKER}: <why>` "
            "marker.",
            file=sys.stderr,
        )
        return 1
    print("terminology: clean — no deprecated RDF-star/SPARQL-star/quoted-triple wording.")
    return 0


def self_test() -> int:
    """Hermetic both-direction check of the gate's classification (no git/network)."""
    must_fail = [
        "sparq supports RDF-star quoted triples.",
        "Run SPARQL-star queries over the store.",
        "Hartig co-originated RDF*/SPARQL* (statement-level metadata).",
        "We store embedded triples in a side table.",
        "The RDF star feature is first-class.",
    ]
    must_pass = [
        "sparq supports RDF 1.2 triple terms.",
        "Run SPARQL 1.2 queries over the store.",
        "The W3C RDF-star Working Group standardised triple terms.",  # proper noun
        "Hartig & Thompson, *Foundations of RDF★ and SPARQL★* (2014).",  # paper title
        "See GraphDB's rdf-sparql-star docs for their encoding.",  # 3rd-party doc name
        "Historically nicknamed RDF-star. terminology-allow: glossary note",  # marker
        "GeoSPARQL spatial filters and Text-to-SPARQL are unrelated.",  # no false pos
        "The *engine runs real SPARQL* against a graph.",  # italic-closer, no false pos
        "It loads the operator's own RDF**, then indexes it.",  # bold, no false pos
    ]
    failures = []
    for s in must_fail:
        if _line_allowed(s) or not any(rx.search(s) for rx, _ in BANNED):
            failures.append(f"  should-FAIL but PASSED: {s!r}")
    for s in must_pass:
        if not _line_allowed(s) and any(rx.search(s) for rx, _ in BANNED):
            failures.append(f"  should-PASS but FAILED: {s!r}")
    if failures:
        print("check-terminology self-test FAILED:")
        print("\n".join(failures))
        return 1
    print("check-terminology self-test: OK (both directions).")
    return 0


def main(argv: list[str]) -> int:
    if "--self-test" in argv:
        return self_test()
    return run([a for a in argv if not a.startswith("--")])


if __name__ == "__main__":
    raise SystemExit(main(sys.argv[1:]))
