#!/usr/bin/env python3
# [OPUS-5] Docs anti-drift: content-DUPLICATION detection across doc locations.
# Bead sq-w9sr, follow-up 1 of research/docs-site-single-sourcing-anti-drift.md §10.
# Specification: that record's §4 (surface, normalisation, two tiers, escape hatches,
# both-direction self-test) and §5 (BEGIN-INJECT regions are skipped). 🤖 SPARQ agent.
#
# WHY. The docs mandate is a site that cannot silently drift from the canonical prose:
# where the same content must appear twice it should be INJECTED at build time, not
# maintained twice. mdBook's `{{#include}}` already does that for the guide, but a
# `{{#include}}`d region exists exactly ONCE in the source tree — so a scanner over the
# committed markdown sees only the HAND-COPIES. That is the property that makes this
# gate cheap: it needs no knowledge of any injection mechanism, and its finding count is
# a direct measure of un-single-sourced content.
#
# SURFACE (§4): root `*.md`, `docs/**`, `book/src/**`, `skills/**/*.md`,
# `crates/*/README.md`. Deliberately EXCLUDES `research/**` (design records legitimately
# restate context, and markdownlint already treats research/ as its own tier),
# `.claude/**` and `AGENTS-worker-core.md` (the agent-contract replication is
# intentional and is governed by the contract's own single-source rule), and vendor
# subtrees.
#
# NORMALISATION (§4): strip YAML frontmatter + HTML comments; split on blank lines
# (fence-aware — a fenced code block is ONE block even if it contains blank lines);
# collapse whitespace; lowercase; and STRIP MARKDOWN LINK TARGETS while keeping the link
# TEXT. That last step is load-bearing: the same paragraph copied between two mounts
# usually differs ONLY in whether its links are repo-relative or absolute, which is
# exactly the difference scripts/mdbook-rewrite-links.py exists to erase. A comparator
# that keeps targets scores such a pair as merely "similar" and lets it through.
#
# TWO TIERS, matching the repo's advisory->gating probation convention:
#   * EXACT tier — GATING. An identical normalised block in two or more DIFFERENT files
#     on the surface fails the build (`--enforce`, the default).
#   * NEAR tier — ADVISORY. 8-gram Jaccard at or above NEAR_THRESHOLD is REPORTED
#     (`--near`), never failed, while the §3 backlog is cleared. It promotes to gating
#     by MOVING its step from docs-quality.yml's `quick-advisory` job into `quick-gates`
#     and adding `--enforce`; nothing in this script changes.
#
# ESCAPE HATCHES (both narrow, both reviewed in the diff — never a path exclusion):
#   1. an inline `<!-- doc-duplication-allow: <why> -->` marker inside the block, or on
#      the comment line(s) immediately preceding it (same shape as `terminology-allow:` /
#      `privacy-claims-allow:`). The reason after the colon is required — a bare marker
#      is itself reported, so the opt-out stays auditable.
#   2. a file-PAIR entry in scripts/doc-duplication-allowlist.json, each carrying a
#      `why` (mirroring banned-terminology.json's `exemptPaths` discipline).
# Broadening either to make a violation pass defeats the gate; fix the wording, or
# single-source the block.
#
# BEGIN-INJECT REGIONS (§5). Lines between `<!-- BEGIN-INJECT: … -->` and
# `<!-- END-INJECT -->` are skipped entirely, so generated/injected copy is never
# reported as a duplicate while a HAND-copy of the same text still is. The generator
# that writes those regions is a separate follow-up bead (§10 item 2) and has NOT
# landed; this is the consumer half, kept here so the two do not have to land together.
# Because skipping is an EXEMPTION, the marker parser is strict and FAIL-CLOSED: only the
# two documented shapes, each alone on its line, open and close a region, and a malformed,
# nested, unmatched or UNTERMINATED marker skips NOTHING and is itself a gating finding.
# Otherwise a lone `<!-- BEGIN-INJECT -->` would hide every following line — the whole
# rest of the document — from the exact tier.
#
# Deterministic: stdlib-only, no network, no build. Reproduce locally with
#   python3 scripts/check-doc-duplication.py            # exact tier (what CI gates on)
#   python3 scripts/check-doc-duplication.py --near     # near tier (advisory report)
#   python3 scripts/check-doc-duplication.py --self-test # hermetic both-direction test

from __future__ import annotations

import argparse
import fnmatch
import json
import os
import re
import subprocess
import sys
import tempfile
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent
CONFIG_PATH = REPO_ROOT / "scripts" / "doc-duplication-allowlist.json"

# --- Surface (§4). git pathspecs with `:(glob)` magic so `*` does not cross `/`,
#     which is what pins the first entry to ROOT-level markdown only. ---
SURFACE_PATHSPECS = [
    ":(glob)*.md",
    ":(glob)docs/**/*.md",
    ":(glob)book/src/**/*.md",
    ":(glob)skills/**/*.md",
    ":(glob)crates/*/README.md",
    # The agent contract is replicated on purpose and single-sourced by its own rule.
    ":(exclude)AGENTS-worker-core.md",
    ":(exclude,glob)vendor/**",
    ":(exclude,glob)**/vendor/**",
]

# Blocks shorter than this (in NORMALISED characters) are not compared at all. A
# heading, a one-line command, a licence line or a short admonition repeats across docs
# for good reasons and carries no drift risk; the floor is what keeps the exact tier
# free of that noise without a single path exclusion.
MIN_BLOCK_CHARS = 200

# Near tier: Jaccard over word 8-grams. 8 words is long enough that two independently
# written paragraphs on the same topic rarely share one.
#
# CALIBRATION (measured on this tree when the gate landed, not guessed). §4 warns that
# the alternative to an advisory near tier is "picking a threshold high enough to be
# decorative", so the threshold was chosen by reading the findings at several settings:
# 0.30 reports 8 pairs and MISSES the SKILL.md <-> crate-README code-example class that
# §3 names as the dominant duplicate kind (sparq-prov 0.29, sparq-shacl 0.22); 0.20
# reports 20, and every one of the 20 was inspected and is a genuine hand-copy — the
# mutual *-wasm how-to stanzas, the SKILL/README code examples, and the two
# engine-facade seam READMEs. So 0.20 is where precision was still 20/20 by inspection.
# Lower it further only with the same evidence.
NEAR_NGRAM = 8
NEAR_THRESHOLD = 0.20

ALLOW_MARKER = "doc-duplication-allow"
ALLOW_RE = re.compile(r"<!--\s*" + re.escape(ALLOW_MARKER) + r"\s*:(.*?)-->", re.DOTALL)

FRONTMATTER_RE = re.compile(r"\A---\r?\n.*?\r?\n---\r?\n", re.DOTALL)
HTML_COMMENT_RE = re.compile(r"<!--.*?-->", re.DOTALL)
FENCE_RE = re.compile(r"^\s{0,3}(```|~~~)")

# §5 marker grammar — STRICT, because matching one is an exemption. The documented forms
# are exactly `<!-- BEGIN-INJECT: <source> -->` (the source is required: it is what the
# generator's `--check` mode regenerates from) and `<!-- END-INJECT -->`, each alone on
# its line. INJECT_SHAPE_RE is the wider net used only to REPORT a line that is trying to
# be a marker and is not one; it is anchored so a marker quoted mid-sentence in prose is
# not mistaken for a real one.
BEGIN_INJECT_RE = re.compile(r"^[ \t]*<!--[ \t]*BEGIN-INJECT:[ \t]*(\S.*?)[ \t]*-->[ \t]*$")
END_INJECT_RE = re.compile(r"^[ \t]*<!--[ \t]*END-INJECT[ \t]*-->[ \t]*$")
INJECT_SHAPE_RE = re.compile(r"^[ \t]*<!--[ \t]*(?:BEGIN|END)-INJECT\b")

MARKER_FORMS = ("the documented forms are `<!-- BEGIN-INJECT: <source> -->` and "
                "`<!-- END-INJECT -->`, each alone on its line")

# Link normalisation. Inline links/images keep their TEXT and lose their TARGET;
# reference links keep their text; link-reference DEFINITION lines vanish entirely;
# autolinks (`<https://…>`) vanish because they are a bare target with no text.
INLINE_LINK_RE = re.compile(r"!?\[([^\]]*)\]\([^)]*\)")
REF_LINK_RE = re.compile(r"!?\[([^\]]*)\]\[[^\]]*\]")
REF_DEF_RE = re.compile(r"^\s*\[[^\]]+\]:\s*\S+.*$", re.MULTILINE)
AUTOLINK_RE = re.compile(r"<https?://[^>\s]*>")


class Block:
    """One blank-line-separated markdown block, with its normalised comparison form."""

    __slots__ = ("path", "lineno", "raw", "norm", "allow_reason", "bare_marker")

    def __init__(self, path: str, lineno: int, raw: str):
        self.path = path
        self.lineno = lineno
        self.raw = raw
        self.norm = normalise(raw)
        marker = ALLOW_RE.search(raw)
        reason = marker.group(1).strip() if marker else ""
        self.allow_reason = reason
        # A marker with no justification must not silently exempt anything.
        self.bare_marker = marker is not None and not reason

    def excerpt(self, width: int = 90) -> str:
        text = " ".join(self.raw.split())
        return text if len(text) <= width else text[: width - 1] + "…"


def normalise(raw: str) -> str:
    """The §4 comparison form: link TARGETS stripped, comments gone, case- and
    whitespace-insensitive."""
    text = HTML_COMMENT_RE.sub(" ", raw)
    text = REF_DEF_RE.sub(" ", text)
    text = AUTOLINK_RE.sub(" ", text)
    text = INLINE_LINK_RE.sub(r"\1", text)
    text = REF_LINK_RE.sub(r"\1", text)
    return " ".join(text.split()).lower()


def inject_skip_lines(path: str, lines: list[str],
                      errors: list[tuple[str, int, str]]) -> set[int]:
    """1-based line numbers inside a WELL-FORMED `<!-- BEGIN-INJECT: … -->` region (§5).

    Fail-closed. A region is skipped only when its BEGIN and END markers both match the
    documented grammar exactly, nothing marker-shaped sits between them, and the BEGIN is
    actually closed. Every other case — malformed shape, nested BEGIN, unmatched END,
    unterminated BEGIN — skips NOTHING (so the lines stay in the comparison) and appends a
    gating error, because the alternative is a one-line exemption for the rest of a file.

    Markers inside a fenced code block are documentation of the convention, not markers,
    and are ignored — matching how the block splitter treats a fence.
    """
    skip: set[int] = set()
    in_fence = False
    open_line = 0
    region: list[int] = []
    region_bad = False

    for idx, line in enumerate(lines, 1):
        if open_line:
            if END_INJECT_RE.match(line):
                if not region_bad:
                    skip.update(region)
                    skip.add(open_line)
                    skip.add(idx)
                open_line = 0
                region = []
                region_bad = False
                continue
            if INJECT_SHAPE_RE.match(line):
                errors.append((path, idx,
                               "nested or malformed INJECT marker inside the region opened "
                               "at line %d — the region is NOT skipped" % open_line))
                region_bad = True
            region.append(idx)
            continue
        if FENCE_RE.match(line):
            in_fence = not in_fence
            continue
        if in_fence or not INJECT_SHAPE_RE.match(line):
            continue
        if BEGIN_INJECT_RE.match(line):
            open_line = idx
        elif END_INJECT_RE.match(line):
            errors.append((path, idx, "`<!-- END-INJECT -->` with no matching "
                                      "`<!-- BEGIN-INJECT: … -->`"))
        else:
            errors.append((path, idx, "malformed INJECT marker — %s" % MARKER_FORMS))

    if open_line:
        errors.append((path, open_line,
                       "`<!-- BEGIN-INJECT: … -->` is never closed by `<!-- END-INJECT -->` "
                       "— the region is NOT skipped"))
    return skip


def split_blocks(path: str, text: str) -> tuple[list[Block], list[tuple[str, int, str]]]:
    """Split one markdown document into comparable blocks.

    Fence-aware (a fenced code block containing blank lines stays ONE block, which
    matters because the dominant duplicate kind on this surface is a code example) and
    INJECT-aware (§5). Returns the blocks plus any INJECT-marker errors, which the caller
    reports as gating findings.
    """
    text = FRONTMATTER_RE.sub("", text)
    lines = text.splitlines()
    errors: list[tuple[str, int, str]] = []
    skip = inject_skip_lines(path, lines, errors)
    blocks: list[Block] = []
    current: list[str] = []
    start = 1
    in_fence = False

    def flush() -> None:
        nonlocal current
        if current:
            blocks.append(Block(path, start, "\n".join(current)))
            current = []

    for idx, line in enumerate(lines, 1):
        if idx in skip:
            flush()
            continue
        if FENCE_RE.match(line):
            in_fence = not in_fence
            if not current:
                start = idx
            current.append(line)
            continue
        if not in_fence and not line.strip():
            flush()
            continue
        if not current:
            start = idx
        current.append(line)
    flush()

    # A marker written on its OWN line above the block — with the blank line markdown
    # convention puts there — must still attach. Without this, `<!-- doc-duplication-allow:
    # … -->\n\n<paragraph>` splits into two blocks and the marker silently exempts
    # nothing, which reads as the gate ignoring a justification that IS present.
    for prev, block in zip(blocks, blocks[1:]):
        if block.allow_reason or not prev.allow_reason:
            continue
        if _is_comment_only(prev.raw):
            block.allow_reason = prev.allow_reason

    kept = [b for b in blocks if len(b.norm) >= MIN_BLOCK_CHARS or b.bare_marker]
    return kept, errors


def _is_comment_only(raw: str) -> bool:
    """Is this block nothing but HTML comment(s)? (Then it is a marker line, not content.)"""
    return not HTML_COMMENT_RE.sub("", raw).strip()


def word_ngrams(norm: str, n: int = NEAR_NGRAM) -> set[tuple[str, ...]]:
    words = norm.split()
    if len(words) < n:
        return {tuple(words)} if words else set()
    return {tuple(words[i:i + n]) for i in range(len(words) - n + 1)}


# --------------------------------------------------------------------------- #
# Allowlist
# --------------------------------------------------------------------------- #

def load_pairs(config_path: Path = CONFIG_PATH) -> list[dict]:
    try:
        cfg = json.loads(config_path.read_text(encoding="utf-8"))
    except FileNotFoundError:
        return []
    pairs = cfg.get("pairs", [])
    for entry in pairs:
        for field in ("a", "b", "why"):
            if not entry.get(field):
                raise ValueError(
                    f"{config_path}: allowlist pair {entry!r} is missing {field!r} — "
                    "every exemption must name both files and say why."
                )
    return pairs


def pair_allowed(pairs: list[dict], first: str, second: str) -> bool:
    def hit(pattern: str, path: str) -> bool:
        return path == pattern or fnmatch.fnmatch(path, pattern)

    return any(
        (hit(e["a"], first) and hit(e["b"], second))
        or (hit(e["a"], second) and hit(e["b"], first))
        for e in pairs
    )


# --------------------------------------------------------------------------- #
# Collection + the two tiers
# --------------------------------------------------------------------------- #

def surface_files(root: Path = REPO_ROOT) -> list[str]:
    out = subprocess.run(
        ["git", "ls-files", "--"] + SURFACE_PATHSPECS,
        capture_output=True, text=True, check=True, cwd=root,
    )
    return sorted({p for p in out.stdout.splitlines() if p})


def collect_blocks(root: Path,
                   paths: list[str]) -> tuple[list[Block], list[tuple[str, int, str]]]:
    blocks: list[Block] = []
    errors: list[tuple[str, int, str]] = []
    for rel in paths:
        try:
            text = (root / rel).read_text(encoding="utf-8")
        except (OSError, UnicodeDecodeError):
            continue
        file_blocks, file_errors = split_blocks(rel, text)
        blocks.extend(file_blocks)
        errors.extend(file_errors)
    return blocks, errors


def exact_findings(blocks: list[Block], pairs: list[dict]) -> list[tuple[Block, Block]]:
    """Identical normalised blocks in two DIFFERENT files (§4 exact tier)."""
    by_norm: dict[str, list[Block]] = {}
    for b in blocks:
        if b.allow_reason:
            continue
        by_norm.setdefault(b.norm, []).append(b)
    findings: list[tuple[Block, Block]] = []
    for group in by_norm.values():
        seen_files: dict[str, Block] = {}
        for b in group:
            seen_files.setdefault(b.path, b)
        ordered = sorted(seen_files.values(), key=lambda b: (b.path, b.lineno))
        for i, first in enumerate(ordered):
            for second in ordered[i + 1:]:
                if not pair_allowed(pairs, first.path, second.path):
                    findings.append((first, second))
    return sorted(findings, key=lambda f: (f[0].path, f[0].lineno, f[1].path))


def near_findings(blocks: list[Block],
                  pairs: list[dict],
                  threshold: float = NEAR_THRESHOLD) -> list[tuple[Block, Block, float]]:
    """8-gram Jaccard pairs at or above `threshold`, excluding the exact tier (§4).

    Candidates are generated from a shared-8-gram inverted index rather than by
    comparing every pair, so the lane stays linear-ish in practice.
    """
    grams = [word_ngrams(b.norm) for b in blocks]
    index: dict[tuple[str, ...], list[int]] = {}
    for i, gs in enumerate(grams):
        for g in gs:
            index.setdefault(g, []).append(i)
    candidates: set[tuple[int, int]] = set()
    for bucket in index.values():
        if len(bucket) < 2:
            continue
        for i, a in enumerate(bucket):
            for b in bucket[i + 1:]:
                candidates.add((a, b))
    findings: list[tuple[Block, Block, float]] = []
    for i, j in candidates:
        first, second = blocks[i], blocks[j]
        if first.path == second.path or first.norm == second.norm:
            continue
        if first.allow_reason or second.allow_reason:
            continue
        if pair_allowed(pairs, first.path, second.path):
            continue
        union = grams[i] | grams[j]
        if not union:
            continue
        score = len(grams[i] & grams[j]) / len(union)
        if score >= threshold:
            findings.append((first, second, score))
    return sorted(findings, key=lambda f: (-f[2], f[0].path, f[0].lineno))


def bare_marker_findings(blocks: list[Block]) -> list[Block]:
    return [b for b in blocks if b.bare_marker]


# --------------------------------------------------------------------------- #
# Reporting
# --------------------------------------------------------------------------- #

def _summary(text: str) -> None:
    path = os.environ.get("GITHUB_STEP_SUMMARY")
    if path:
        with open(path, "a", encoding="utf-8") as fh:
            fh.write(text)


def run(near: bool, enforce: bool) -> int:
    pairs = load_pairs()
    files = surface_files()
    blocks, marker_errors = collect_blocks(REPO_ROOT, files)

    bare = bare_marker_findings(blocks)
    for b in bare:
        print(f"{b.path}:{b.lineno}: empty `{ALLOW_MARKER}` marker — a reason is required.")
    for path, lineno, message in marker_errors:
        print(f"{path}:{lineno}: {message}")

    if near:
        findings = near_findings(blocks, pairs)
        label = "near-duplicate"
        for first, second, score in findings:
            print(f"{first.path}:{first.lineno} ~ {second.path}:{second.lineno} "
                  f"(8-gram Jaccard {score:.2f})")
            print(f"    {first.excerpt()}")
        lines = [
            f"### doc-duplication (near tier): {len(findings)} pair(s)"
            f"{'' if enforce else ' — ADVISORY'}\n\n",
            f"8-gram Jaccard >= {NEAR_THRESHOLD:.2f} over {len(blocks)} block(s) in "
            f"{len(files)} file(s) on the published doc surface. A near pair is prose "
            "that was hand-copied and has since drifted; fix it by single-sourcing the "
            "block, not by rewording it just past the threshold.\n\n",
        ]
        for first, second, score in findings[:40]:
            lines.append(f"- `{first.path}:{first.lineno}` ~ `{second.path}:{second.lineno}` "
                         f"— Jaccard {score:.2f}\n")
        if len(findings) > 40:
            lines.append(f"\n…and {len(findings) - 40} more (see the step log).\n")
        _summary("".join(lines))
    else:
        findings = exact_findings(blocks, pairs)
        label = "exact duplicate"
        for first, second in findings:
            print(f"{first.path}:{first.lineno} == {second.path}:{second.lineno}")
            print(f"    {first.excerpt()}")
        _summary(
            f"### doc-duplication (exact tier): {len(findings)} pair(s)\n\n"
            f"Identical normalised blocks across {len(files)} file(s) on the published "
            f"doc surface ({len(blocks)} comparable block(s)).\n"
        )

    if bare:
        print(f"\n{len(bare)} empty `{ALLOW_MARKER}` marker(s): the text after the colon "
              "is the required justification.", file=sys.stderr)
    if marker_errors:
        print(f"\n{len(marker_errors)} malformed INJECT marker(s) (§5): {MARKER_FORMS}. "
              "A region that is not well formed skips nothing — an unterminated "
              "`<!-- BEGIN-INJECT: … -->` would otherwise exempt the whole rest of the "
              "file from this gate.", file=sys.stderr)
    if findings:
        print(f"\n{len(findings)} {label} pair(s) on the published doc surface. "
              "Single-source the block — `{{#include}}` it in the guide, or move a code "
              "example into `crates/<crate>/examples/` and inject it "
              "(research/docs-site-single-sourcing-anti-drift.md §5/§6). If the repeat is "
              f"genuinely intentional, annotate the block `<!-- {ALLOW_MARKER}: <why> -->` "
              "or add the file pair to scripts/doc-duplication-allowlist.json with a "
              "`why`. Do NOT widen the surface exclusions.", file=sys.stderr)
    if not findings and not bare and not marker_errors:
        print(f"doc-duplication: clean — 0 {label} pair(s) across {len(blocks)} "
              f"comparable block(s) in {len(files)} file(s).")

    return 1 if (enforce and (findings or bare or marker_errors)) else 0


# --------------------------------------------------------------------------- #
# Hermetic both-direction self-test (§4). No git, no network: every case writes
# fixture files into a temp dir and drives the real collection + tier functions.
# --------------------------------------------------------------------------- #

# ~340 normalised characters, comfortably over MIN_BLOCK_CHARS.
_LONG = (
    "The store keeps every quad in a columnar dictionary so that a scan over one "
    "predicate touches only the columns it needs, and the planner can decide between a "
    "merge join and a hash join without materialising an intermediate result. This "
    "paragraph is deliberately long enough to clear the block floor the duplication "
    "gate applies before it compares anything at all."
)
_OTHER = (
    "Validation reports are streamed rather than buffered, so a shape that fails early "
    "in a large graph surfaces its first violation without waiting for the rest of the "
    "run to finish. Reports carry the focus node and the constraint component, which is "
    "what the command line renders when it prints a failure summary to the terminal."
)



# Two halves of one fenced example, each individually UNDER MIN_BLOCK_CHARS and together
# over it — so the fence-awareness case below can only pass if the blank line inside the
# fence does not split the block.
_FENCE_HALF_A = (
    'let graph = Graph::load_str(turtle, Format::Turtle)?;\n'
    'let store = Store::from_graph(graph);'
)
_FENCE_HALF_B = (
    'let results = store.query("SELECT ?s WHERE { ?s a :Thing }")?;\n'
    'for solution in results { println!("{solution:?}"); }'
)


def _analyse(files: dict[str, str], pairs: list[dict] | None = None):
    """Write `files` to a temp dir and return (exact, near, bare, marker) findings."""
    with tempfile.TemporaryDirectory() as tmp:
        root = Path(tmp)
        for name, body in files.items():
            target = root / name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_text(body, encoding="utf-8")
        blocks, marker_errors = collect_blocks(root, sorted(files))
        return (
            exact_findings(blocks, pairs or []),
            near_findings(blocks, pairs or []),
            bare_marker_findings(blocks),
            marker_errors,
        )


def self_test() -> int:
    failures: list[str] = []

    def case(label: str, files: dict[str, str], *, exact: int, near: int | None = None,
             bare: int = 0, markers: int = 0, pairs: list[dict] | None = None) -> None:
        got_exact, got_near, got_bare, got_markers = _analyse(files, pairs)
        problems = []
        if len(got_exact) != exact:
            problems.append(f"exact={len(got_exact)} want {exact}")
        if near is not None and len(got_near) != near:
            problems.append(f"near={len(got_near)} want {near}")
        if len(got_bare) != bare:
            problems.append(f"bare-marker={len(got_bare)} want {bare}")
        if len(got_markers) != markers:
            problems.append(f"marker-error={len(got_markers)} want {markers}")
        print(f"  [{'FAIL' if problems else 'PASS'}] {label}"
              + (f": {', '.join(problems)}" if problems else ""))
        if problems:
            failures.append(label)

    # --- Direction 1: it FIRES on a planted duplicate. ---
    case("exact: the same paragraph in two files is caught",
         {"README.md": f"# A\n\n{_LONG}\n", "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=1)
    # Each HALF of this fence is under MIN_BLOCK_CHARS and the whole fence is over it,
    # so the case is red unless the fence is genuinely kept together. (An earlier
    # fixture put a long comment in the second half; splitting on the blank line still
    # left one over-floor duplicate block, so it passed with fence-awareness deleted.)
    case("exact: a duplicated CODE FENCE (blank line inside) stays ONE block",
         {"skills/x/SKILL.md": f"# A\n\n```rust\n{_FENCE_HALF_A}\n\n{_FENCE_HALF_B}\n```\n",
          "crates/c/README.md": f"# B\n\n```rust\n{_FENCE_HALF_A}\n\n{_FENCE_HALF_B}\n```\n"},
         exact=1)
    case("exact: link TARGETS differ, text identical — still a duplicate (the §4 "
         "load-bearing normalisation step)",
         {"README.md": f"# A\n\n{_LONG} See [the guide](book/src/introduction.md).\n",
          "docs/a.md": f"# B\n\n{_LONG} See [the guide](https://example.invalid/guide).\n"},
         exact=1)
    case("near: a drifted hand-copy is reported by the near tier, not the exact tier",
         {"README.md": f"# A\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG} One extra sentence was added here later on.\n"},
         exact=0, near=1)
    case("bare marker: an unjustified opt-out is itself a finding",
         {"README.md": f"<!-- {ALLOW_MARKER}: -->\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=1, bare=1)

    # --- Direction 2: it stays QUIET on distinct prose and on every sanctioned escape. ---
    case("quiet: distinct prose in two files",
         {"README.md": f"# A\n\n{_LONG}\n", "docs/a.md": f"# B\n\n{_OTHER}\n"},
         exact=0, near=0)
    case("quiet: a justified inline allow marker exempts the block",
         {"README.md": f"<!-- {ALLOW_MARKER}: canonical copy, mirrored on purpose -->\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=0, near=0, bare=0)
    case("quiet: the marker also attaches from the comment line ABOVE a blank line "
         "(the shape markdown convention produces)",
         {"README.md": f"<!-- {ALLOW_MARKER}: canonical copy, mirrored on purpose -->\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=0, near=0, bare=0)
    case("firing check for the case above: the SAME shape with no marker is caught",
         {"README.md": f"<!-- an ordinary comment -->\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=1)
    case("quiet: an allowlisted file PAIR exempts the duplicate",
         {"README.md": f"# A\n\n{_LONG}\n", "docs/a.md": f"# B\n\n{_LONG}\n"},
         exact=0, near=0,
         pairs=[{"a": "README.md", "b": "docs/a.md", "why": "test fixture"}])
    case("quiet: a block under the length floor repeats freely",
         {"README.md": "# A\n\nRun `sparq query --help` to list the flags.\n",
          "docs/a.md": "# B\n\nRun `sparq query --help` to list the flags.\n"},
         exact=0, near=0)
    case("quiet: the SAME file repeating a block is not a cross-file duplicate",
         {"README.md": f"# A\n\n{_LONG}\n\n## Again\n\n{_LONG}\n"},
         exact=0, near=0)
    case("quiet: a BEGIN-INJECT region is skipped (§5) while a hand-copy still fires",
         {"README.md": f"# A\n\n{_LONG}\n",
          "crates/c/README.md":
              f"# B\n\n<!-- BEGIN-INJECT: README.md#quickstart -->\n{_LONG}\n"
              "<!-- END-INJECT -->\n"},
         exact=0, near=0)
    case("quiet: an INJECT marker shown inside a code FENCE documents the convention "
         "and is not a marker",
         {"README.md": "# A\n\n```text\n<!-- BEGIN-INJECT: README.md#quickstart -->\n"
                       "…generated copy…\n<!-- END-INJECT -->\n```\n"},
         exact=0, near=0, markers=0)

    # --- Direction 1 again: every MALFORMED delimiter fails CLOSED (§5). Each case plants
    # the duplicate INSIDE/AFTER the broken region, so it can only pass if the region is
    # not honoured as an exemption — and the marker defect is itself reported. ---
    case("fail-closed: an UNTERMINATED BEGIN-INJECT does not hide the rest of the file",
         {"README.md": f"# A\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n<!-- BEGIN-INJECT: README.md#quickstart -->\n\n{_LONG}\n"},
         exact=1, markers=1)
    case("fail-closed: a loose marker shape (no `: <source>`) is not honoured",
         {"README.md": f"# A\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n<!-- BEGIN-INJECT -->\n{_LONG}\n<!-- END-INJECT -->\n"},
         exact=1, markers=2)
    case("fail-closed: an END-INJECT with no BEGIN is reported",
         {"README.md": f"# A\n\n{_LONG}\n",
          "docs/a.md": f"# B\n\n{_LONG}\n\n<!-- END-INJECT -->\n"},
         exact=1, markers=1)
    case("fail-closed: a NESTED BEGIN-INJECT does not open an exempt region",
         {"README.md": f"# A\n\n{_LONG}\n",
          "docs/a.md": "# B\n\n<!-- BEGIN-INJECT: a.md#one -->\n"
                       f"<!-- BEGIN-INJECT: a.md#two -->\n{_LONG}\n<!-- END-INJECT -->\n"},
         exact=1, markers=1)
    case("quiet: YAML frontmatter is not compared",
         {"skills/a/SKILL.md": f"---\nname: a\ndescription: {_LONG}\n---\n\n# A\n",
          "skills/b/SKILL.md": f"---\nname: b\ndescription: {_LONG}\n---\n\n# B\n"},
         exact=0, near=0)

    # The live allowlist must stay well-formed (a malformed pair raises in load_pairs).
    try:
        live = load_pairs()
        print(f"  [PASS] live allowlist parses — {len(live)} pair exemption(s)")
    except (ValueError, json.JSONDecodeError) as exc:
        print(f"  [FAIL] live allowlist: {exc}")
        failures.append("live allowlist")

    if failures:
        print(f"\ncheck-doc-duplication self-test: {len(failures)} case(s) FAILED")
        return 1
    print("\ncheck-doc-duplication self-test: all cases PASS")
    return 0


def main(argv: list[str] | None = None) -> int:
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("--near", action="store_true",
                    help="report the NEAR tier (8-gram Jaccard) instead of the exact tier")
    ap.add_argument("--advisory", action="store_true",
                    help="report findings but always exit 0 (the advisory lane)")
    ap.add_argument("--self-test", action="store_true",
                    help="run the hermetic both-direction self-test and exit")
    args = ap.parse_args(argv)
    if args.self_test:
        return self_test()
    return run(near=args.near, enforce=not args.advisory)


if __name__ == "__main__":
    sys.exit(main())
