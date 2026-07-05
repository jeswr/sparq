#!/usr/bin/env python3
"""
scripts/export-kb-dump.py — tier-aware KB dump assembler (sq-tzars.8)

Assembles a versioned dump from per-tier KB artifacts into dumps/YYYY-MM-DD/:

  pkg-ontology.ttl.gz               — the PKG vocabulary (pkg.ttl)
  pkg-hand-authored.ttl.gz          — hand-authored tier (ingest_pkg.py output)
  pkg-machine.ttl.gz                — machine tier (literature pipeline, if present)
  pkg-restricted-projection.ttl.gz  — metadata-only public view of restricted tier (if present)
  manifest.json                     — per-file tier + license class + triple counts + commit
  dump-provenance.ttl               — PROV-O activity for this dump (prov:generatedAtTime)

The LICENSE-RESTRICTED tier is NEVER exported; only its metadata-only projection.
A mandatory leak check scans all dump files for restricted-tier markers and secrets.

Usage:
  # Self-test (writes to a temp dir + runs the injected-leak negative):
  python3 scripts/export-kb-dump.py --dry-run

  # Full dump run:
  python3 scripts/export-kb-dump.py [--out-dir dumps/] [--commit <sha>]
  [--hand-authored PATH] [--machine-tier PATH] [--restricted-projection PATH]

The GitHub Actions workflow (kb-dump.yml) calls this script, then pushes the
output directory to sparq-org/research-kb.

(sq-tzars.8) [SONNET-4.6] 🤖 SPARQ agent — sparq-org/research-kb dump assembler.
"""

import argparse
import gzip
import hashlib
import json
import os
import re
import subprocess
import sys
import tempfile
from datetime import datetime, timezone
from pathlib import Path

# ── Tier graph IRIs (byte-pinned in crates/sparq-kb/src/vocab.rs) ─────────────
TIER_HAND_AUTHORED_GRAPH = "https://sparq.dev/ns/pkg/graph#hand-authored"
TIER_MACHINE_GRAPH = "https://sparq.dev/ns/pkg/graph#machine"
TIER_LICENSE_RESTRICTED_GRAPH = "https://sparq.dev/ns/pkg/graph#license-restricted"

# ── Leak-check patterns ───────────────────────────────────────────────────────
# Each entry is (compiled_regex_or_None, plain_string_or_None, human-readable reason).
# Exactly one of regex/string is non-None per entry.
# String markers use exact substring matching; regex markers use re.search per line.

# Global: applies to ALL dump files.
#
# Note on `Bearer`: task titles in the PKG legitimately contain the English word "Bearer"
# (e.g., "Gate WebSocket with the Bearer read-token") — an exact-string match would
# produce false positives.  We instead require `Bearer ` followed by ≥ 32 alphanumeric
# characters (the shortest realistic credential), which catches real token leaks without
# flagging natural language.
LEAK_PATTERNS_GLOBAL: list[tuple[re.Pattern | None, str | None, str]] = [
    (
        re.compile(r"Bearer\s+[A-Za-z0-9_\-]{32,}"),
        None,
        "Bearer auth-token (≥32 alphanumeric chars after 'Bearer ') — "
        "a real credential that must never appear in a dump",
    ),
    (
        None,
        "CORE_API_KEY",
        "CORE API key env-var name — must never appear in a dump",
    ),
    (
        None,
        "machine findings from unknown/absent/non-redistributable sources (PRIVATE)",
        "restricted FULL-tier graph label — indicates the private artifact was "
        "accidentally included instead of the public projection",
    ),
]

# Restricted-projection specific: the metadata-only PUBLIC projection must carry
# ZERO abstract-derived text.  These markers are only checked on the projection file.
LEAK_PATTERNS_RESTRICTED_PROJECTION: list[tuple[re.Pattern | None, str | None, str]] = [
    (
        None,
        "sigimpl:justification",
        "abstract-derived justification text — restricted full tier only; "
        "the public projection must not carry any sigimpl:justification triple",
    ),
    (
        None,
        "dcterms:abstract",
        "raw abstract text — restricted full tier only; "
        "the public projection must not carry any dcterms:abstract triple",
    ),
    (
        None,
        "a pkg:Finding",
        "full Finding triple — the restricted projection carries only source metadata, "
        "never Finding nodes",
    ),
]

# Self-test injected-leak marker: a sentinel value used ONLY in --dry-run.
# It is inserted as a sigimpl:justification value to simulate a leak of restricted-tier
# abstract-derived text, then the scanner must catch it (the negative self-test).
INJECTED_LEAK_MARKER = "SPARQ_KB_RESTRICTED_TIER_LEAK_MARKER_DO_NOT_EXPORT"

# ── Helpers ───────────────────────────────────────────────────────────────────


def _now_iso() -> str:
    """UTC ISO 8601 xsd:dateTime string (e.g. '2026-07-05T12:34:56Z')."""
    return datetime.now(tz=timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def _today() -> str:
    """UTC date string (e.g. '2026-07-05')."""
    return datetime.now(tz=timezone.utc).strftime("%Y-%m-%d")


def _sha256(data: bytes) -> str:
    return "sha256:" + hashlib.sha256(data).hexdigest()


def _count_triples(turtle_text: str) -> int:
    """
    Approximate triple count for a Turtle document.
    Uses rdflib if available; otherwise falls back to a heuristic that counts
    statement terminators (`.` for new subject, `;` for additional predicate,
    `,` for additional object) on non-comment, non-prefix lines.
    """
    try:
        import rdflib  # noqa: PLC0415

        g = rdflib.Graph()
        g.parse(data=turtle_text, format="turtle")
        return len(g)
    except Exception:
        count = 0
        for line in turtle_text.splitlines():
            s = line.strip()
            if not s or s.startswith("#") or s.startswith("@prefix") or s.startswith("@base"):
                continue
            # Count each statement terminator as one (or more) triples
            count += s.count(" .") + s.count("\t.")
            count += s.count(" ;") + s.count("\t;")
            count += s.count(" ,") + s.count("\t,")
            # A bare `.` at the end of a short line
            if s == ".":
                count += 1
        return max(count, 0)


def _git_head(repo_root: Path) -> str:
    """Return the current git HEAD SHA, or 'unknown' if git is unavailable."""
    try:
        result = subprocess.run(
            ["git", "-C", str(repo_root), "rev-parse", "HEAD"],
            capture_output=True,
            text=True,
            timeout=10,
        )
        if result.returncode == 0:
            return result.stdout.strip()
    except Exception:
        pass
    return "unknown"


def _compress(content: bytes) -> bytes:
    """gzip-compress *content* at default compression level."""
    import io

    buf = io.BytesIO()
    with gzip.GzipFile(fileobj=buf, mode="wb", mtime=0) as gz:
        gz.write(content)
    return buf.getvalue()


# ── Leak check ────────────────────────────────────────────────────────────────


class LeakViolation:
    def __init__(self, filename: str, marker: str, reason: str, line_no: int, snippet: str):
        self.filename = filename
        self.marker = marker
        self.reason = reason
        self.line_no = line_no
        self.snippet = snippet

    def __str__(self) -> str:
        return (
            f"  LEAK in {self.filename!r} at line {self.line_no}: "
            f"marker={self.marker!r}\n"
            f"    reason: {self.reason}\n"
            f"    snippet: {self.snippet!r}"
        )


def leak_check_file(
    filename: str,
    content: str,
    extra_patterns: list[tuple[re.Pattern | None, str | None, str]] | None = None,
) -> list[LeakViolation]:
    """
    Scan *content* for LEAK_PATTERNS_GLOBAL (and optional *extra_patterns*).
    Returns a list of LeakViolation for every hit.
    """
    violations: list[LeakViolation] = []
    all_patterns = list(LEAK_PATTERNS_GLOBAL)
    if extra_patterns:
        all_patterns.extend(extra_patterns)

    lines = content.splitlines()
    for lineno, line in enumerate(lines, start=1):
        for pat, literal, reason in all_patterns:
            hit = False
            marker_display: str
            if literal is not None:
                if literal in line:
                    hit = True
                    marker_display = literal
            elif pat is not None:
                m = pat.search(line)
                if m:
                    hit = True
                    marker_display = m.group(0)[:60]
            if hit:
                snippet = line.strip()[:120]
                violations.append(
                    LeakViolation(
                        filename=filename,
                        marker=marker_display,
                        reason=reason,
                        line_no=lineno,
                        snippet=snippet,
                    )
                )
    return violations


def run_leak_check(
    dump_files: dict[str, str],
    restricted_projection_key: str | None = None,
) -> tuple[bool, list[LeakViolation]]:
    """
    Run the full leak check over *dump_files* (mapping filename → text content).

    *restricted_projection_key* names the file that must pass the additional
    restricted-projection patterns (sigimpl:justification, dcterms:abstract, a pkg:Finding).

    Returns (passed, violations). *passed* is True iff the violation list is empty.
    """
    all_violations: list[LeakViolation] = []

    for filename, content in dump_files.items():
        extra: list[tuple[re.Pattern | None, str | None, str]] | None = None
        if filename == restricted_projection_key:
            extra = list(LEAK_PATTERNS_RESTRICTED_PROJECTION)
        violations = leak_check_file(filename, content, extra_patterns=extra)
        all_violations.extend(violations)

    return (len(all_violations) == 0), all_violations


# ── PROV-O dump-provenance.ttl ────────────────────────────────────────────────


def make_dump_provenance(
    dump_date: str,
    generated_at: str,
    source_commit: str,
    tier_file_iris: list[str],
) -> str:
    """
    Produce a PROV-O Turtle document describing this dump as a prov:Activity.
    The dump carries its OWN provenance activity per the sq-tzars.8 design.
    """
    dump_activity_iri = f"https://sparq.dev/ns/kb/dump/{dump_date}"
    used_triples = "\n".join(
        f'  prov:used <{iri}> ;' for iri in tier_file_iris
    )
    return f"""\
# dump-provenance.ttl — PROV-O activity record for the KB dump of {dump_date}
# Generated by scripts/export-kb-dump.py (sq-tzars.8) [SONNET-4.6]
# 🤖 SPARQ agent — do not hand-edit.

@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix rdfs:    <http://www.w3.org/2000/01/rdf-schema#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix sparq:   <https://sparq.dev/ns/> .

<{dump_activity_iri}> a prov:Activity ;
  rdfs:label "KB dump activity for {dump_date}"@en ;
  prov:startedAtTime "{generated_at}"^^xsd:dateTime ;
  prov:endedAtTime "{generated_at}"^^xsd:dateTime ;
  prov:generatedAtTime "{generated_at}"^^xsd:dateTime ;
{used_triples}
  dcterms:source <https://github.com/sparq-org/sparq/commit/{source_commit}> ;
  rdfs:comment "Assembled by scripts/export-kb-dump.py. The LICENSE-RESTRICTED tier "
               "is never exported; only its metadata-only projection is included."@en .
"""


# ── Manifest ─────────────────────────────────────────────────────────────────


def make_manifest(
    dump_date: str,
    generated_at: str,
    source_commit: str,
    file_entries: list[dict],
) -> dict:
    """Build the manifest.json dict."""
    return {
        "dump_version": dump_date,
        "source_commit": source_commit,
        "generated_at": generated_at,
        "generator": "scripts/export-kb-dump.py (sq-tzars.8) [SONNET-4.6]",
        "note": (
            "The LICENSE-RESTRICTED tier is never exported. "
            "Only its metadata-only public projection (source IRI + title + year "
            "+ licence status; no abstract-derived text) is included. "
            "This repo is PRIVATE until the maintainer reviews the tier-enforcement "
            "guarantees and explicitly flips visibility."
        ),
        "tier_semantics": {
            "hand-authored": (
                "Human-curated PKG data: task projections from bd + skill front-matter "
                "+ hand-authored AGENTS.md findings (ingest_pkg.py output)."
            ),
            "machine": (
                "Literature pipeline output for sources with a known redistributable "
                "licence (CC-BY, CC0, public-domain). Machine-tier findings are capped "
                "at confidence ≤ 0.7 and assurance ≤ Conjectured."
            ),
            "restricted-projection": (
                "Metadata-only public view of the license-restricted tier: source IRI, "
                "title, year, and licence status only. No abstract-derived text."
            ),
        },
        "files": file_entries,
    }


# ── Core assembly ─────────────────────────────────────────────────────────────


def assemble_dump(
    out_dir: Path,
    ontology_content: str | None,
    hand_authored_content: str | None,
    machine_content: str | None,
    restricted_projection_content: str | None,
    source_commit: str,
    dump_date: str | None = None,
    generated_at: str | None = None,
) -> dict:
    """
    Compress and write tier artifacts to *out_dir*, produce manifest.json and
    dump-provenance.ttl, run the leak check, and return the manifest dict.

    Raises SystemExit(1) on any leak check failure.
    """
    dump_date = dump_date or _today()
    generated_at = generated_at or _now_iso()
    out_dir.mkdir(parents=True, exist_ok=True)

    file_entries: list[dict] = []
    dump_texts: dict[str, str] = {}  # filename → text (for leak check)
    restricted_proj_key: str | None = None
    used_tier_iris: list[str] = []

    def _write_tier(
        filename: str,
        content: str,
        tier: str,
        license_class: str,
        graph_iri: str,
    ) -> None:
        nonlocal restricted_proj_key
        compressed = _compress(content.encode("utf-8"))
        (out_dir / filename).write_bytes(compressed)
        triple_count = _count_triples(content)
        raw_sha = _sha256(content.encode("utf-8"))
        file_entries.append(
            {
                "filename": filename,
                "tier": tier,
                "license_class": license_class,
                "graph_iri": graph_iri,
                "triple_count": triple_count,
                "source_sha256": raw_sha,
                "compressed_bytes": len(compressed),
            }
        )
        dump_texts[filename] = content
        if tier == "restricted-projection":
            restricted_proj_key = filename
        used_tier_iris.append(graph_iri)

    # 1. Ontology (optional — present if the ontology file exists)
    if ontology_content is not None:
        _write_tier(
            "pkg-ontology.ttl.gz",
            ontology_content,
            tier="ontology",
            license_class="public",
            graph_iri="https://sparq.dev/ns/pkg#",
        )

    # 2. Hand-authored tier
    if hand_authored_content is not None:
        _write_tier(
            "pkg-hand-authored.ttl.gz",
            hand_authored_content,
            tier="hand-authored",
            license_class="public",
            graph_iri=TIER_HAND_AUTHORED_GRAPH,
        )

    # 3. Machine tier (optional — may not exist yet)
    if machine_content is not None:
        _write_tier(
            "pkg-machine.ttl.gz",
            machine_content,
            tier="machine",
            license_class="public",
            graph_iri=TIER_MACHINE_GRAPH,
        )

    # 4. Restricted projection (optional — may not exist yet)
    if restricted_projection_content is not None:
        _write_tier(
            "pkg-restricted-projection.ttl.gz",
            restricted_projection_content,
            tier="restricted-projection",
            license_class="public",
            graph_iri=TIER_LICENSE_RESTRICTED_GRAPH,
        )

    # ── Leak check (mandatory, fail-closed) ───────────────────────────────────
    passed, violations = run_leak_check(dump_texts, restricted_proj_key)
    if not passed:
        print("\n[FAIL] Leak check detected restricted-tier content or secrets:", file=sys.stderr)
        for v in violations:
            print(str(v), file=sys.stderr)
        print(
            "\nABORTING: the assembled dump contains forbidden content. "
            "Do NOT push this dump.",
            file=sys.stderr,
        )
        sys.exit(1)

    print(
        f"[OK]   Leak check passed ({len(dump_texts)} file(s) scanned, 0 violations).",
        file=sys.stdout,
    )

    # ── manifest.json ─────────────────────────────────────────────────────────
    manifest = make_manifest(dump_date, generated_at, source_commit, file_entries)
    manifest_path = out_dir / "manifest.json"
    manifest_path.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")
    print(f"[OK]   manifest.json written ({len(file_entries)} file(s)).", file=sys.stdout)

    # ── dump-provenance.ttl ───────────────────────────────────────────────────
    prov_ttl = make_dump_provenance(dump_date, generated_at, source_commit, used_tier_iris)
    (out_dir / "dump-provenance.ttl").write_text(prov_ttl, encoding="utf-8")
    print("[OK]   dump-provenance.ttl written.", file=sys.stdout)

    return manifest


# ── Dry-run self-test ─────────────────────────────────────────────────────────


def run_dry_run_self_test(repo_root: Path) -> bool:
    """
    Run the --dry-run self-test:
    1. Assemble a synthetic dump with no restricted content → leak check PASSES.
    2. Inject a restricted-tier marker into the restricted projection file and
       re-assemble → leak check FAILS (the injected-leak negative).

    Returns True if the self-test passes (both assertions hold), False otherwise.
    """
    print("\n── Dry-run self-test ──────────────────────────────────────────────────", flush=True)

    # ── Synthetic tier content ─────────────────────────────────────────────────
    SYNTHETIC_HAND_AUTHORED = """\
# SYNTHETIC hand-authored tier (self-test only)
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix kb:      <https://sparq.dev/ns/pkg/kb#> .

# Tier stamp
<https://sparq.dev/ns/pkg/graph#hand-authored> <http://www.w3.org/2000/01/rdf-schema#label>
  "hand-authored tier -- self-test synthetic data" .

kb:finding-selftest-1 a pkg:Finding ;
  pkg:about kb:topic-test ;
  pkg:confidence "0.90"^^xsd:decimal ;
  pkg:assurance secx:Claimed ;
  sigimpl:justification "This is a hand-authored justification — legitimately in the public dump."@en ;
  prov:wasDerivedFrom kb:source-selftest-1 .
"""

    SYNTHETIC_MACHINE = """\
# SYNTHETIC machine tier (self-test only)
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix sigimpl: <https://w3id.org/zkp-sparql/sig-impl#> .
@prefix secx:    <https://w3id.org/zkp-sparql/sec-prop#> .
@prefix prov:    <http://www.w3.org/ns/prov#> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix kb:      <https://sparq.dev/ns/pkg/kb#> .

# Tier stamp
<https://sparq.dev/ns/pkg/graph#machine> <http://www.w3.org/2000/01/rdf-schema#label>
  "machine tier -- self-test synthetic data" .

kb:finding-machine-1 a pkg:Finding ;
  pkg:about kb:topic-query-opt ;
  pkg:confidence "0.65"^^xsd:decimal ;
  pkg:assurance secx:Conjectured ;
  sigimpl:justification "Machine-extracted justification for a CC-BY source — legitimately public."@en ;
  prov:wasDerivedFrom kb:source-openalex-1 .
"""

    # The restricted PROJECTION carries ONLY source metadata — NO justification,
    # NO abstract, NO pkg:Finding triples.
    SYNTHETIC_RESTRICTED_PROJECTION = """\
# SYNTHETIC restricted-tier public projection (self-test only)
# Metadata-only: DOI + title + year + licence status. NO abstract-derived text.
@prefix pkg:     <https://sparq.dev/ns/pkg#> .
@prefix dcterms: <http://purl.org/dc/terms/> .
@prefix schema:  <http://schema.org/> .
@prefix xsd:     <http://www.w3.org/2001/XMLSchema#> .
@prefix kb:      <https://sparq.dev/ns/pkg/kb#> .

# Tier stamp (metadata-only PUBLIC projection)
<https://sparq.dev/ns/pkg/graph#license-restricted>
  <http://www.w3.org/2000/01/rdf-schema#label>
  "license-restricted tier -- metadata-only PUBLIC projection (DOI + title + year + licence status; NO abstract-derived text)" .

# Restricted source metadata (no abstract, no findings, no justification)
<https://doi.org/10.5555/selftest.restricted.1> a pkg:Source ;
  dcterms:title "A Restricted Source (self-test)" ;
  schema:datePublished "2025"^^xsd:gYear ;
  pkg:exploredStatus pkg:Explored ;
  <https://sparq.dev/ns/pkg#licenseStatus> "unknown" .
"""

    SYNTHETIC_ONTOLOGY = """\
# SYNTHETIC ontology stub (self-test only)
@prefix owl: <http://www.w3.org/2002/07/owl#> .
@prefix rdfs: <http://www.w3.org/2000/01/rdf-schema#> .

<https://sparq.dev/ns/pkg#> a owl:Ontology ;
  rdfs:label "PKG vocabulary (self-test stub)" .
"""

    import tempfile

    with tempfile.TemporaryDirectory(prefix="sparq-kb-dump-selftest-") as tmpdir:
        out_path = Path(tmpdir) / "dump"

        # ── Step 1: clean assembly — expect PASS ──────────────────────────────
        print("\n[self-test] Step 1: assemble clean dump — leak check must PASS")
        try:
            assemble_dump(
                out_dir=out_path,
                ontology_content=SYNTHETIC_ONTOLOGY,
                hand_authored_content=SYNTHETIC_HAND_AUTHORED,
                machine_content=SYNTHETIC_MACHINE,
                restricted_projection_content=SYNTHETIC_RESTRICTED_PROJECTION,
                source_commit="selftest-0000000",
                dump_date="1970-01-01",
                generated_at="1970-01-01T00:00:00Z",
            )
        except SystemExit:
            print(
                "[FAIL] self-test step 1: clean dump UNEXPECTEDLY failed the leak check.",
                file=sys.stderr,
            )
            return False

        print("[self-test] Step 1 PASSED — clean dump clears the leak check.")

        # ── Step 2: injected-leak negative — expect FAIL ──────────────────────
        print(
            "\n[self-test] Step 2: inject restricted-tier content into the projection "
            "— leak check must FAIL (this is the NEGATIVE test proving the scanner works)"
        )

        # Inject a sigimpl:justification with the known restricted marker
        INJECTED_PROJECTION = SYNTHETIC_RESTRICTED_PROJECTION + (
            f'\n# INJECTED LEAK (self-test negative) — sigimpl:justification must be caught\n'
            f'<https://doi.org/10.5555/selftest.restricted.1>\n'
            f'  <https://w3id.org/zkp-sparql/sig-impl#justification>'
            f' "{INJECTED_LEAK_MARKER}" .\n'
        )

        # Redirect stderr to capture the failure message, then verify it
        import io

        captured_err = io.StringIO()
        import contextlib

        # We WANT assemble_dump to call sys.exit(1) here; catch it.
        exited_with_failure = False
        with contextlib.redirect_stderr(captured_err):
            try:
                # Use a fresh subdirectory for the injected-leak run
                injected_out = Path(tmpdir) / "dump-injected"
                assemble_dump(
                    out_dir=injected_out,
                    ontology_content=SYNTHETIC_ONTOLOGY,
                    hand_authored_content=SYNTHETIC_HAND_AUTHORED,
                    machine_content=SYNTHETIC_MACHINE,
                    restricted_projection_content=INJECTED_PROJECTION,
                    source_commit="selftest-injected",
                    dump_date="1970-01-01",
                    generated_at="1970-01-01T00:00:00Z",
                )
            except SystemExit as e:
                if e.code == 1:
                    exited_with_failure = True

        err_output = captured_err.getvalue()

        if not exited_with_failure:
            print(
                "[FAIL] self-test step 2: injected-leak dump did NOT fail the leak check "
                "— the scanner is not catching sigimpl:justification in the projection.",
                file=sys.stderr,
            )
            print(f"  stderr was: {err_output!r}", file=sys.stderr)
            return False

        # Verify the error output mentions the injected marker
        if INJECTED_LEAK_MARKER not in err_output and "sigimpl:justification" not in err_output:
            print(
                "[FAIL] self-test step 2: leak check failed as expected but did not "
                f"mention {INJECTED_LEAK_MARKER!r} or 'sigimpl:justification' in stderr.",
                file=sys.stderr,
            )
            print(f"  stderr was: {err_output!r}", file=sys.stderr)
            return False

        print("[self-test] Step 2 PASSED — injected-leak correctly caught by scanner.")
        print(
            "  (Scanner reported the violation; stderr suppressed during the negative test.)"
        )

    print("\n── Self-test result: ALL PASSED ────────────────────────────────────────")
    return True


# ── Argument parsing and main ─────────────────────────────────────────────────


def _find_repo_root(start: Path) -> Path:
    """Walk up from *start* looking for the .git directory."""
    p = start.resolve()
    while p != p.parent:
        if (p / ".git").exists():
            return p
        p = p.parent
    return start.resolve()


def main() -> int:
    ap = argparse.ArgumentParser(
        description="Assemble a tier-aware sparq KB dump (sq-tzars.8).",
        formatter_class=argparse.RawDescriptionHelpFormatter,
    )
    ap.add_argument(
        "--dry-run",
        action="store_true",
        help=(
            "Run the self-test: assemble a synthetic dump (temp dir) + run the "
            "injected-leak negative test. Does not modify the repository or push anything."
        ),
    )
    ap.add_argument(
        "--repo-root",
        type=Path,
        default=None,
        help="Path to the sparq repo root (auto-detected if omitted).",
    )
    ap.add_argument(
        "--out-dir",
        type=Path,
        default=None,
        help=(
            "Output directory. A dated subdirectory (YYYY-MM-DD) is created inside it. "
            "Defaults to <repo-root>/dumps/."
        ),
    )
    ap.add_argument(
        "--commit",
        type=str,
        default=None,
        help="Source commit SHA to embed in the manifest (auto-detected from git HEAD).",
    )
    ap.add_argument(
        "--hand-authored",
        type=Path,
        default=None,
        help=(
            "Path to the hand-authored tier TTL artifact "
            "(default: crates/sparq-kb/ingest/pkg-instances.ttl relative to repo root)."
        ),
    )
    ap.add_argument(
        "--machine-tier",
        type=Path,
        default=None,
        help=(
            "Path to the machine-tier TTL artifact (optional; omitted if the file does "
            "not exist)."
        ),
    )
    ap.add_argument(
        "--restricted-projection",
        type=Path,
        default=None,
        help=(
            "Path to the metadata-only public projection of the restricted tier (optional; "
            "omitted if the file does not exist)."
        ),
    )
    args = ap.parse_args()

    # ── Dry-run self-test ─────────────────────────────────────────────────────
    if args.dry_run:
        repo_root = args.repo_root or _find_repo_root(Path(__file__))
        ok = run_dry_run_self_test(repo_root)
        if ok:
            print("\n[PASS] --dry-run self-test GREEN.", flush=True)
            return 0
        else:
            print("\n[FAIL] --dry-run self-test FAILED.", file=sys.stderr, flush=True)
            return 1

    # ── Normal assembly run ───────────────────────────────────────────────────
    repo_root = args.repo_root or _find_repo_root(Path(__file__))

    # Resolve tier artifact paths
    hand_authored_path = args.hand_authored or (
        repo_root / "crates" / "sparq-kb" / "ingest" / "pkg-instances.ttl"
    )
    ontology_path = (
        repo_root / "crates" / "sparq-kb" / "ontology" / "pkg" / "pkg.ttl"
    )

    # Machine tier and restricted projection: optional
    machine_tier_path = args.machine_tier
    restricted_proj_path = args.restricted_projection

    # Output directory
    out_base = args.out_dir or (repo_root / "dumps")
    dump_date = _today()
    out_dir = out_base / dump_date

    # Source commit
    source_commit = args.commit or _git_head(repo_root)

    # Read tier artifacts
    def _read_opt(path: Path | None, label: str) -> str | None:
        if path is None:
            return None
        if not path.exists():
            print(f"[SKIP] {label}: {path} not found — skipping tier.", file=sys.stdout)
            return None
        content = path.read_text(encoding="utf-8")
        print(f"[READ] {label}: {path} ({len(content):,} chars).", file=sys.stdout)
        return content

    ontology_content = _read_opt(ontology_path, "ontology")
    hand_authored_content = _read_opt(hand_authored_path, "hand-authored tier")
    machine_content = _read_opt(machine_tier_path, "machine tier")
    restricted_projection_content = _read_opt(restricted_proj_path, "restricted projection")

    if hand_authored_content is None:
        print(
            f"[ERROR] Hand-authored tier artifact not found: {hand_authored_path}",
            file=sys.stderr,
        )
        print("  Run crates/sparq-kb/ingest/ingest_pkg.py first.", file=sys.stderr)
        return 1

    print(f"\nAssembling KB dump → {out_dir}", flush=True)

    assemble_dump(
        out_dir=out_dir,
        ontology_content=ontology_content,
        hand_authored_content=hand_authored_content,
        machine_content=machine_content,
        restricted_projection_content=restricted_projection_content,
        source_commit=source_commit,
        dump_date=dump_date,
    )

    print(f"\n[DONE] Dump written to: {out_dir}")
    print("  Review the manifest, then push to sparq-org/research-kb (kb-dump.yml).")
    return 0


if __name__ == "__main__":
    sys.exit(main())
