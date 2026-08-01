#!/usr/bin/env python3
# [OPUS-5] sq-jfrp0 (issue #2679) — assemble sparq's bench registries from per-suite
# FRAGMENT files so concurrent bench PRs stop serial-conflicting.
#
# WHY: every bench PR used to edit the SAME three files — bench/benchmarks.toml,
# bench/competitors.json, bench/CATALOG.md — appending its new suite at the same
# textual spot. Two sibling bench PRs therefore conflicted by construction (#1932
# was rebased twice in one session, going re-DIRTY each time main advanced with
# another bench PR), which forced bench PRs to be merged strictly one-at-a-time.
# This mirrors the fix already proven for the opt-in feature matrix
# (.github/feature-matrix.d/ + scripts/assemble-feature-matrix.py, bead sq-ibrze):
# a new entry gets its OWN file, so two PRs for DIFFERENT suites never touch the
# same path and auto-merge.
#
# THE THREE REGISTRIES AND THEIR FRAGMENT DIRS
#   bench/benchmarks.toml   <- bench/registry.d/<suite>.toml     ([[benchmark]] entries)
#   bench/competitors.json  <- bench/competitors.d/<id>.json     ({"competitors": [...]})
#   bench/CATALOG.md        <- bench/catalog.d/<suite>.md        (per-suite human notes)
#
# The committed base file stays the TRUNK (the historical bulk); fragments are the
# APPEND POINT. Nothing is regenerated into the trunk — consumers assemble at READ
# time, in deterministic sorted-filename order, so the assembled view is stable and
# no generated artifact has to be re-committed (which would just move the conflict).
#
# BEHAVIOUR-PRESERVATION INVARIANT: with ZERO fragments in a dir, the assembled text
# is BYTE-IDENTICAL to the trunk file. Every consumer therefore keeps its exact
# pre-split behaviour until someone adds the first fragment.
#
# Usage:
#   python3 scripts/bench_registry.py --check          # validate all three; exit!=0 on error
#   python3 scripts/bench_registry.py --registry       # assembled benchmarks.toml -> stdout
#   python3 scripts/bench_registry.py --competitors    # assembled competitors.json -> stdout
#   python3 scripts/bench_registry.py --catalog        # assembled CATALOG.md -> stdout
#   python3 scripts/bench_registry.py --ids            # every benchmark id, one per line
#
# Importable as a library (`import bench_registry`) by the sibling gate scripts in
# scripts/ — they call registry_text()/competitors_obj() instead of reading the
# trunk file directly.

from __future__ import annotations

import json
import re
import sys
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parent.parent

BENCH_DIR = REPO_ROOT / "bench"

REGISTRY_TRUNK = BENCH_DIR / "benchmarks.toml"
REGISTRY_FRAGMENT_DIR = BENCH_DIR / "registry.d"

COMPETITORS_TRUNK = BENCH_DIR / "competitors.json"
COMPETITORS_FRAGMENT_DIR = BENCH_DIR / "competitors.d"

CATALOG_TRUNK = BENCH_DIR / "CATALOG.md"
CATALOG_FRAGMENT_DIR = BENCH_DIR / "catalog.d"

# Directory names directly under bench/ that are FRAGMENT DIRS, not bench suites.
# Consumers that enumerate `bench/<suite>/` from a diff (scripts/check-new-bench-
# registered.py) must skip these, or adding a fragment would be mis-read as adding
# a brand-new unregistered suite.
NON_SUITE_BENCH_DIRS = frozenset(
    {
        REGISTRY_FRAGMENT_DIR.name,
        COMPETITORS_FRAGMENT_DIR.name,
        CATALOG_FRAGMENT_DIR.name,
    }
)

# `id = "..."` at the start of a line — the registry is hand-authored, prose-heavy
# TOML and every consuming gate already matches it textually rather than parsing the
# table array (see scripts/check-new-bench-registered.py's module header). We keep
# the SAME predicate so the assembler and the gates cannot disagree.
_ID_RE = re.compile(r'^\s*id\s*=\s*"([^"]+)"', re.MULTILINE)
_BENCHMARK_HEADER_RE = re.compile(r"(?m)^\s*\[\[benchmark\]\]\s*$")
# Any other top-level TOML table/array-of-table header. A fragment may only declare
# `[[benchmark]]` entries — a stray `[foo]` would silently swallow the following
# key/value lines into a table the registry schema knows nothing about.
_OTHER_TABLE_RE = re.compile(r"(?m)^\s*\[(?!\[benchmark\]\])[^\n]*\]\s*$")


class RegistryError(Exception):
    """A fragment is malformed, or two fragments collide on an id."""


# --------------------------------------------------------------------------- #
# Fragment discovery
# --------------------------------------------------------------------------- #
def fragments(directory: Path, suffix: str) -> list[tuple[str, str]]:
    """(relative-name, text) for every `*<suffix>` file in `directory`, sorted by
    filename so the assembled order is deterministic run-to-run.

    A missing directory yields no fragments (the pre-split state). Non-matching
    files and the directory's own README.md are ignored, so the convention can be
    documented in place (the catalog's fragments are themselves markdown, so the
    README has to be excluded by name, not by suffix).
    """
    if not directory.is_dir():
        return []
    out: list[tuple[str, str]] = []
    for path in sorted(directory.glob(f"*{suffix}")):
        if path.name == "README.md":
            continue
        out.append((path.name, path.read_text(encoding="utf-8")))
    return out


def registry_ids(text: str) -> list[str]:
    """Every benchmark `id` declared in registry text, in file order."""
    return _ID_RE.findall(text)


def benchmark_blocks(text: str) -> list[str]:
    """The body of each `[[benchmark]]` table in `text`, in file order.

    A block runs from just after its own header to the start of the next one (or
    end of text). Validation has to be PER BLOCK: a fragment-wide `id` scan would
    accept `[[benchmark]] id="ok"` followed by an id-less second entry, putting an
    unidentifiable benchmark into the assembled registry — invisible to every gate
    that resolves a suite by id.
    """
    heads = list(_BENCHMARK_HEADER_RE.finditer(text))
    return [
        text[m.end() : (heads[i + 1].start() if i + 1 < len(heads) else len(text))]
        for i, m in enumerate(heads)
    ]


# --------------------------------------------------------------------------- #
# benchmarks.toml
# --------------------------------------------------------------------------- #
def assemble_registry(trunk_text: str, frags: list[tuple[str, str]]) -> str:
    """Trunk text followed by each fragment, in the given (sorted) order.

    Each fragment is preceded by a provenance comment naming its file, so a reader
    of the assembled output can tell which fragment an entry came from. Raises
    RegistryError on a fragment that declares no `[[benchmark]]`, declares a
    non-`[[benchmark]]` table, has any `[[benchmark]]` entry without exactly one
    `id`, or reuses an `id` already claimed by the trunk or an earlier fragment (a
    duplicate id would make the registry ambiguous for every gate that resolves a
    suite by id).
    """
    seen: dict[str, str] = {i: REGISTRY_TRUNK.name for i in registry_ids(trunk_text)}
    parts = [trunk_text.rstrip("\n")]
    for name, text in frags:
        where = f"{REGISTRY_FRAGMENT_DIR.name}/{name}"
        head = _BENCHMARK_HEADER_RE.search(text)
        if not head:
            raise RegistryError(
                f"{where}: declares no [[benchmark]] entry — a registry fragment "
                f"must contain at least one"
            )
        stray = _OTHER_TABLE_RE.search(text)
        if stray:
            raise RegistryError(
                f"{where}: unexpected top-level table {stray.group(0).strip()!r} — a "
                f"registry fragment may only declare [[benchmark]] entries"
            )
        if registry_ids(text[: head.start()]):
            raise RegistryError(
                f"{where}: an `id` is declared before the first [[benchmark]] header "
                f"— every id must belong to a [[benchmark]] entry"
            )
        ids = []
        for position, block in enumerate(benchmark_blocks(text), start=1):
            block_ids = registry_ids(block)
            if not block_ids:
                raise RegistryError(
                    f"{where}: [[benchmark]] entry #{position} has no `id` field"
                )
            if len(block_ids) > 1:
                raise RegistryError(
                    f"{where}: [[benchmark]] entry #{position} declares {len(block_ids)} "
                    f"`id` fields {block_ids} — an entry carries exactly one"
                )
            ids.extend(block_ids)
        for bid in ids:
            if bid in seen:
                raise RegistryError(
                    f"{where}: duplicate benchmark id {bid!r} (first declared in "
                    f"{seen[bid]}); every id must be unique across the assembled "
                    f"registry"
                )
            seen[bid] = where
        parts.append(f"# --- assembled from bench/{where} ---\n{text.strip()}")
    return "\n\n".join(parts) + "\n"


def registry_text(root: Path | None = None) -> str:
    """The assembled bench registry (trunk + bench/registry.d/*.toml).

    Byte-identical to bench/benchmarks.toml when no fragments exist. Returns '' if
    the trunk is unreadable, matching the pre-split behaviour of every caller.
    """
    trunk, frag_dir = _registry_paths(root)
    try:
        trunk_text = trunk.read_text(encoding="utf-8")
    except OSError:
        return ""
    frags = fragments(frag_dir, ".toml")
    if not frags:
        return trunk_text
    return assemble_registry(trunk_text, frags)


def _registry_paths(root: Path | None) -> tuple[Path, Path]:
    if root is None:
        return REGISTRY_TRUNK, REGISTRY_FRAGMENT_DIR
    return root / "bench" / "benchmarks.toml", root / "bench" / "registry.d"


# --------------------------------------------------------------------------- #
# competitors.json
# --------------------------------------------------------------------------- #
def assemble_competitors(trunk_obj: dict, frags: list[tuple[str, str]]) -> dict:
    """Trunk object with each fragment's `competitors` entries appended.

    A fragment is a JSON object with a single `competitors` array (the same entry
    shape the trunk uses) — mirroring the trunk's schema so an entry can be moved
    between the two verbatim. Raises RegistryError on a malformed fragment or a
    duplicate competitor `id` (scripts/gather-competitors.sh resolves engines by
    id, so a duplicate would silently shadow one entry).
    """
    merged = dict(trunk_obj)
    competitors = list(trunk_obj.get("competitors") or [])
    seen = {c.get("id"): COMPETITORS_TRUNK.name for c in competitors if isinstance(c, dict)}
    for name, text in frags:
        where = f"{COMPETITORS_FRAGMENT_DIR.name}/{name}"
        try:
            obj = json.loads(text)
        except json.JSONDecodeError as e:
            raise RegistryError(f"{where}: not valid JSON ({e})") from e
        if not isinstance(obj, dict):
            raise RegistryError(
                f"{where}: top level must be a JSON object with a `competitors` array"
            )
        extra = set(obj) - {"competitors"}
        if extra:
            raise RegistryError(
                f"{where}: unexpected key(s) {sorted(extra)} — a competitor fragment "
                f"carries only `competitors` (schema/results_layout stay in the trunk)"
            )
        entries = obj.get("competitors")
        if not isinstance(entries, list) or not entries:
            raise RegistryError(
                f"{where}: `competitors` must be a non-empty array of entries"
            )
        for entry in entries:
            # A blank id is as unaddressable as a missing one — gather-competitors.sh
            # resolves engines BY id, so an entry keyed on "" (or "  ") can never be
            # selected. Mirror the registry side, where `_ID_RE`'s `[^"]+` already
            # makes an empty `id = ""` read as "no id at all".
            cid = entry.get("id") if isinstance(entry, dict) else None
            if not isinstance(cid, str) or not cid.strip():
                raise RegistryError(
                    f"{where}: every competitor entry must be an object with a "
                    f"non-empty string `id`"
                )
            if cid in seen:
                raise RegistryError(
                    f"{where}: duplicate competitor id {cid!r} (first declared in "
                    f"{seen[cid]})"
                )
            seen[cid] = where
            competitors.append(entry)
    merged["competitors"] = competitors
    return merged


def competitors_obj(root: Path | None = None) -> dict:
    """The assembled competitor registry (trunk + bench/competitors.d/*.json)."""
    trunk = (
        COMPETITORS_TRUNK if root is None else root / "bench" / "competitors.json"
    )
    frag_dir = (
        COMPETITORS_FRAGMENT_DIR if root is None else root / "bench" / "competitors.d"
    )
    trunk_obj = json.loads(trunk.read_text(encoding="utf-8"))
    return assemble_competitors(trunk_obj, fragments(frag_dir, ".json"))


# --------------------------------------------------------------------------- #
# CATALOG.md
# --------------------------------------------------------------------------- #
def assemble_catalog(trunk_text: str, frags: list[tuple[str, str]]) -> str:
    """Trunk catalog followed by each per-suite note fragment, sorted by filename.

    The catalog is prose for humans, so a fragment is plain markdown — the only
    rule is that it must lead with a heading, so the assembled document keeps a
    navigable structure (and two fragments can never merge into one section).
    """
    parts = [trunk_text.rstrip("\n")]
    for name, text in frags:
        where = f"{CATALOG_FRAGMENT_DIR.name}/{name}"
        first = next((ln for ln in text.splitlines() if ln.strip()), "")
        if not first.startswith("#"):
            raise RegistryError(
                f"bench/{where}: must start with a markdown heading (e.g. "
                f"`## <suite>`) so the assembled catalog stays navigable"
            )
        parts.append(text.strip())
    return "\n\n".join(parts) + "\n"


def catalog_text(root: Path | None = None) -> str:
    trunk = CATALOG_TRUNK if root is None else root / "bench" / "CATALOG.md"
    frag_dir = CATALOG_FRAGMENT_DIR if root is None else root / "bench" / "catalog.d"
    trunk_text = trunk.read_text(encoding="utf-8")
    frags = fragments(frag_dir, ".md")
    if not frags:
        return trunk_text
    return assemble_catalog(trunk_text, frags)


# --------------------------------------------------------------------------- #
# CLI
# --------------------------------------------------------------------------- #
def _check() -> int:
    """Validate all three registries; print a one-line summary per registry."""
    failures = 0
    try:
        text = registry_text()
        print(
            f"benchmarks.toml + {REGISTRY_FRAGMENT_DIR.name}/: OK "
            f"({len(registry_ids(text))} benchmark ids, "
            f"{len(fragments(REGISTRY_FRAGMENT_DIR, '.toml'))} fragment(s))"
        )
    except RegistryError as e:
        sys.stderr.write(f"error: bench registry: {e}\n")
        failures += 1
    try:
        obj = competitors_obj()
        print(
            f"competitors.json + {COMPETITORS_FRAGMENT_DIR.name}/: OK "
            f"({len(obj['competitors'])} competitors, "
            f"{len(fragments(COMPETITORS_FRAGMENT_DIR, '.json'))} fragment(s))"
        )
    except (RegistryError, json.JSONDecodeError, OSError) as e:
        sys.stderr.write(f"error: competitor registry: {e}\n")
        failures += 1
    try:
        catalog_text()
        print(
            f"CATALOG.md + {CATALOG_FRAGMENT_DIR.name}/: OK "
            f"({len(fragments(CATALOG_FRAGMENT_DIR, '.md'))} fragment(s))"
        )
    except (RegistryError, OSError) as e:
        sys.stderr.write(f"error: bench catalog: {e}\n")
        failures += 1
    return 1 if failures else 0


def main(argv: list[str] | None = None) -> int:
    argv = list(sys.argv[1:] if argv is None else argv)
    try:
        if not argv or "--check" in argv:
            return _check()
        if "--registry" in argv:
            sys.stdout.write(registry_text())
        elif "--ids" in argv:
            for bid in registry_ids(registry_text()):
                print(bid)
        elif "--competitors" in argv:
            json.dump(competitors_obj(), sys.stdout, indent=2, ensure_ascii=False)
            sys.stdout.write("\n")
        elif "--catalog" in argv:
            sys.stdout.write(catalog_text())
        else:
            sys.stderr.write(f"error: unknown argument(s) {argv}\n")
            return 2
    except (RegistryError, json.JSONDecodeError, OSError) as e:
        sys.stderr.write(f"error: {e}\n")
        return 1
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
