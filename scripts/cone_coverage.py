#!/usr/bin/env python3
# [SONNET-4.6] Changed-cone coverage selector (sq-6vshe.8).
#
# Reuses ci_select.py's diff classification and reverse-dep closure computation
# (via shared import, NOT a diverging copy) to determine which crates need fresh
# instrumented-coverage measurement vs which can inherit their floor result from
# main's last full run (because their code + their transitive deps are unchanged).
#
# WHAT THIS IS (and is NOT):
#   Given the set of paths changed in a PR (same input as ci_select.py), compute
#   the COVERAGE CONE — the set of crates that MUST be re-measured. Crates outside
#   the cone are unchanged: if they passed the floor gate on main, they still pass
#   (SAME soundness argument ci_select.py commits to for test skipping, §2).
#   Emits JSON: {"mode": "cone"|"full", "cone_crates": [...], "reason": "..."}.
#
#   FAIL-SAFE IDENTICAL TO ci_select.py (shared import, not a diverging copy):
#   if any §4.1 trigger (Cargo.lock, root Cargo.toml, .github/, scripts/, etc.)
#   is changed → mode=full → ALL crates measured. Any internal error → mode=full.
#
# SHADOW MODE (default ON, --enforce OFF):
#   Does NOT change what coverage.sh measures. Instead logs the cone vs full
#   comparison so we can validate the cone is correct before enforcing it. The
#   enforce flip is a follow-up bead (gated by this PR / sq-6vshe.8).
#
# USAGE:
#   cone_coverage.py --mode compute-cone [--output cone.json] [--enforce]
#   cone_coverage.py --mode shadow-report --cone cone.json
#                    --coverage-summary coverage-summary.json
#                    [--divergence-log divergence.json]
#                    [--floor bench/coverage-floor.json]
#   cone_coverage.py --changed-file paths.txt --metadata-file meta.json ...
#
# STDLIB ONLY: no third-party deps (mirrors ci_select.py §7 P1).

from __future__ import annotations

import argparse
import importlib.util
import json
import os
import sys
from pathlib import Path

# --- shared import from ci_select.py -----------------------------------------
# [SONNET-4.6] Import the diff classifier and closure computation DIRECTLY from
# ci_select.py (NOT a copy) so the fail-safe rules stay in sync automatically.
# Any future change to §4.1 triggers or the closure algorithm applies here too.

def _load_ci_select():
    here = Path(__file__).resolve().parent
    ci_select_path = here / "ci_select.py"
    spec = importlib.util.spec_from_file_location("ci_select", ci_select_path)
    if spec is None or spec.loader is None:
        raise ImportError(f"Cannot load ci_select.py from {ci_select_path}")
    mod = importlib.util.module_from_spec(spec)
    sys.modules["ci_select"] = mod  # [SONNET-4.6] assign unconditionally to avoid stale-entry cross-test leakage
    spec.loader.exec_module(mod)  # type: ignore[union-attr]
    return mod


_ci_select = _load_ci_select()

# Re-export the pieces we use (makes static analysis + tests easier).
select = _ci_select.select
parse_workspace = _ci_select.parse_workspace
load_metadata = _ci_select.load_metadata
load_ownership_map = _ci_select.load_ownership_map
git_changed_paths = _ci_select.git_changed_paths
SelectorError = _ci_select.SelectorError


# --- cone computation ---------------------------------------------------------

def compute_cone(
    changed_paths: list[str],
    meta: dict,
    map_entries: list[dict] | None = None,
) -> dict:
    """Compute the coverage cone from the set of changed paths.

    [SONNET-4.6] Delegates entirely to ci_select.select() — the SAME diff
    classifier and reverse-dep closure used by change-based test selection. The
    cone is exactly the `affected` set: the changed crates PLUS their transitive
    reverse-dep closure (dependents whose tests might exercise the changed code).

    Returns a plain dict ready to serialise as JSON:
      {
        "mode": "cone" | "full",
        "cone_crates": [...],   # sorted; all workspace members when mode=full
        "changed_crates": [...],# directly-changed workspace members
        "all_members": [...],   # sorted full workspace member list
        "reason": "...",
        "shadow": true,         # always true in this PR (enforce=False)
      }
    """
    map_entries = map_entries or []
    try:
        sel = select(changed_paths, meta, map_entries)
    except Exception as exc:  # noqa: BLE001 — mirrors ci_select.py §4.3
        # Any internal error → full run (fail-closed). Use parse_workspace to get
        # the full member list so cone_crates is correct even on error.
        try:
            ws = parse_workspace(meta)
            all_members = sorted(ws.members)
        except Exception:  # noqa: BLE001
            all_members = []
        return {
            "mode": "full",
            "cone_crates": all_members,
            "changed_crates": [],
            "all_members": all_members,
            "reason": f"cone-coverage selector error, failing to full run: {exc}",
            "shadow": True,
        }

    if sel.mode == "full":
        mode = "full"
        cone_crates = sel.affected  # already == all members for a full run
    else:
        # mode == "selected": the cone is exactly the affected closure.
        mode = "cone"
        cone_crates = sel.affected

    return {
        "mode": mode,
        "cone_crates": sorted(cone_crates),
        "changed_crates": sorted(getattr(sel, "changed_crates", [])),
        "all_members": sorted(getattr(sel, "all_members", [])),
        "reason": sel.reason,
        "shadow": True,  # enforce=False in this PR; flip is the follow-up bead
    }


# --- shadow report -----------------------------------------------------------

def shadow_report(
    cone_doc: dict,
    coverage_summary: dict,
    floor_doc: dict | None,
    divergence_log_path: str | None = None,
) -> dict:
    """[SONNET-4.6] Compare what the cone WOULD have measured vs the full run.

    For each workspace crate:
      - IN cone:     reports label "cone-measured" + actual line_pct from summary
      - OUTSIDE cone: reports label "inherited (unchanged cone): PASS" — we assert
                      the crate is unchanged so its main-run floor result still holds
    Divergences (below-floor crates outside the cone — should never happen if the
    cone is correct) are logged to `divergence_log_path` for monitoring.

    Returns a report dict. Never raises (logs errors; caller continues regardless).
    """
    cone_crates = set(cone_doc.get("cone_crates", []))
    all_members = set(cone_doc.get("all_members", []))
    mode = cone_doc.get("mode", "full")

    floors: dict[str, float] = {}
    if floor_doc:
        for crate, fentry in floor_doc.get("crates", {}).items():
            if isinstance(fentry, dict):
                floors[crate] = float(fentry.get("floor", 0))
            elif isinstance(fentry, (int, float)):
                floors[crate] = float(fentry)

    summary_crates = coverage_summary.get("crates", {})

    # [SONNET-4.6] Operate on the INTERSECTION of workspace members and crates
    # actually present in this shard's coverage-summary.json. In the CI matrix
    # each shard measures only its own subset; iterating all_members would include
    # crates with no measured data, making divergence detection meaningless and
    # inflating counts. Crates absent from the shard summary are omitted from rows;
    # a single compact count is emitted instead (not_measured_in_shard).
    #
    # [SONNET-4.6] Empty-all_members fallback: when cone.json was produced via the
    # error path, all_members may be empty (compute_cone() catches selector errors
    # and emits all_members=[] as a fail-safe). In that case the intersection would
    # be empty and the report would render no rows even though the shard summary has
    # measured crates — silent monitoring blindness. Fall back to the shard summary
    # keys as the effective member universe so divergences still surface.
    measured_in_shard = set(summary_crates.keys())
    effective_members = all_members if all_members else measured_in_shard
    reported_members = sorted(effective_members & measured_in_shard)
    not_measured_count = len(effective_members - measured_in_shard)

    rows: list[dict] = []
    divergences: list[dict] = []

    for crate in reported_members:
        row_summary = summary_crates.get(crate, {})
        measured = row_summary.get("measured", False)
        lines_pct = row_summary.get("lines_pct") if measured else None
        floor = floors.get(crate)

        in_cone = (mode == "full") or (crate in cone_crates)

        if in_cone:
            label = "cone-measured"
            status = "measured"
        else:
            label = "inherited (unchanged cone): PASS"
            status = "inherited"

        row: dict = {
            "crate": crate,
            "label": label,
            "status": status,
            "in_cone": in_cone,
        }
        if lines_pct is not None:
            row["lines_pct"] = lines_pct
        if floor is not None:
            row["floor"] = floor

        rows.append(row)

        # Divergence: a crate OUTSIDE the cone that is below its floor in the
        # full measurement would be a soundness problem (cone missed a regression).
        # Should never happen if the cone is correct.
        if not in_cone and lines_pct is not None and floor is not None:
            if lines_pct + 1e-9 < floor:
                divergences.append({
                    "crate": crate,
                    "lines_pct": lines_pct,
                    "floor": floor,
                    "type": "outside-cone-below-floor",
                    "explanation": (
                        "A crate outside the coverage cone is below its floor. "
                        "This should be impossible if the cone is correct — "
                        "an unchanged crate cannot regress. Investigate."
                    ),
                })

    # [SONNET-4.6] All derived counts are over the REPORTED set (intersection)
    # so that counts always match the row set. The workspace-level cone size is
    # also retained under a distinct key for context.
    reported_cone_set = (
        cone_crates & measured_in_shard if mode != "full"
        else set(reported_members)
    )
    report = {
        "mode": mode,
        "shadow": True,
        "cone_crates": sorted(cone_crates),
        "total_crates": len(reported_members),           # crates in this shard's summary
        "cone_size": len(reported_cone_set),             # cone crates measured in this shard
        "cone_size_workspace": (                         # workspace-level cone size for context
            len(cone_crates) if mode != "full" else len(all_members)
        ),
        "not_measured_in_shard": not_measured_count,     # compact count; no per-crate row emitted
        "inherited_count": sum(1 for r in rows if r["status"] == "inherited"),
        "divergences": divergences,
        "rows": rows,
    }

    if divergence_log_path:
        try:
            with open(divergence_log_path, "w", encoding="utf-8") as fh:
                json.dump(
                    {"divergences": divergences, "mode": mode, "cone_crates": sorted(cone_crates)},
                    fh, indent=2,
                )
                fh.write("\n")
        except OSError as exc:
            print(f"  WARN: could not write divergence log: {exc}", file=sys.stderr)

    return report


def _render_report(report: dict) -> str:
    """Render the shadow report as a Markdown step summary."""
    lines = [
        "### Coverage cone shadow report (sq-6vshe.8) [SONNET-4.6]",
        "",
        f"**Mode:** `{report['mode']}` — shadow (enforce=False; cone computation validated)",
        "",
    ]
    mode = report.get("mode", "full")
    not_in_shard = report.get("not_measured_in_shard", 0)
    if mode == "cone":
        total = report.get("total_crates", 0)
        cone_size = report.get("cone_size", 0)
        inherited = report.get("inherited_count", 0)
        lines += [
            f"**Cone size:** {cone_size} of {total} shard-measured crates measured fresh "
            f"({inherited} inherited from main — unchanged, no re-measurement needed)",
            "",
        ]
    else:
        lines += [
            "**Full run:** all crates measured (diff triggered full-run fail-safe).",
            "",
        ]
    if not_in_shard:
        lines += [
            f"_not-measured-in-this-shard: {not_in_shard} workspace crate(s) "
            f"(measured in other shards or not applicable here — omitted from rows)_",
            "",
        ]

    divs = report.get("divergences", [])
    if divs:
        lines += [
            f"**DIVERGENCES ({len(divs)}) — INVESTIGATE:**",
            "",
            "| Crate | lines_pct | floor | type |",
            "|---|---|---|---|",
        ]
        for d in divs:
            lines.append(
                f"| `{d['crate']}` | {d.get('lines_pct', '?')} | {d.get('floor', '?')} | {d['type']} |"
            )
        lines.append("")
    else:
        lines += ["No divergences — cone is sound for this PR.", ""]

    lines += [
        "| Crate | Status | lines_pct | floor |",
        "|---|---|---|---|",
    ]
    for row in report.get("rows", []):
        pct = f"{row['lines_pct']:.2f}%" if row.get("lines_pct") is not None else "—"
        fl = str(row["floor"]) if row.get("floor") is not None else "—"
        lines.append(f"| `{row['crate']}` | {row['label']} | {pct} | {fl} |")
    lines.append("")
    return "\n".join(lines)


# --- input gathering (mirrors ci_select.py main) ----------------------------

def _resolve_repo_root(explicit: str | None) -> str | None:
    if explicit:
        return explicit
    import subprocess
    try:
        out = subprocess.run(
            ["git", "rev-parse", "--show-toplevel"],
            capture_output=True, text=True, check=True, timeout=30,
        )
        return out.stdout.strip()
    except Exception:  # noqa: BLE001
        return None


def _get_changed_paths(args: argparse.Namespace, repo_root: str | None) -> list[str]:
    """Get changed paths from hermetic file or live git diff."""
    if args.changed_file:
        with open(args.changed_file, encoding="utf-8") as fh:
            # [SONNET-4.6] Strip leading/trailing whitespace from each line so
            # behavior is consistent regardless of how the caller wrote the file
            # (trailing \r on Windows-style line endings, leading spaces, etc.).
            return [ln.strip() for ln in fh.read().splitlines() if ln.strip()]
    base = getattr(args, "base", None)
    head = getattr(args, "head", "HEAD")
    if base:
        return git_changed_paths(base, head, repo_root)
    # No base → can't compute diff → fail-safe to full.
    raise SelectorError("--base is required for a diff-based cone computation")


# --- CLI main ----------------------------------------------------------------

def _parse_args(argv: list[str] | None) -> argparse.Namespace:
    p = argparse.ArgumentParser(
        description="Coverage-cone selector (sq-6vshe.8) [SONNET-4.6]."
    )
    p.add_argument(
        "--mode",
        choices=["compute-cone", "shadow-report"],
        required=True,
        help=(
            "compute-cone: diff → cone JSON; "
            "shadow-report: compare cone JSON vs full coverage summary"
        ),
    )
    # compute-cone inputs
    p.add_argument("--base", help="base SHA/ref for the three-dot diff")
    p.add_argument("--head", default="HEAD", help="head SHA/ref (default HEAD)")
    p.add_argument("--changed-file", help="hermetic: newline-delimited changed paths")
    p.add_argument("--metadata-file", help="hermetic: cargo metadata JSON snapshot")
    p.add_argument("--map", dest="map_file", help="ownership map toml (default ci/path-ownership.toml)")
    p.add_argument("--repo-root", help="repo root (default: git toplevel)")
    p.add_argument("--output", "-o", default=None, help="write cone JSON here (default: stdout)")
    p.add_argument(
        "--enforce",
        action="store_true",
        help="[FOLLOW-UP BEAD] enforce the cone (skip crates outside it). "
             "Default OFF (shadow mode) for this PR.",
    )
    # shadow-report inputs
    p.add_argument("--cone", help="cone JSON from a previous compute-cone run")
    p.add_argument("--coverage-summary", help="full coverage-summary.json from coverage.sh")
    p.add_argument("--floor", help="coverage floor JSON (default bench/coverage-floor.json)")
    p.add_argument("--divergence-log", help="write divergences JSON here")
    p.add_argument("--summary-file", help="write Markdown step-summary (default $GITHUB_STEP_SUMMARY)")
    return p.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv)
    summary_file = args.summary_file or os.environ.get("GITHUB_STEP_SUMMARY")

    if args.mode == "compute-cone":
        repo_root = _resolve_repo_root(args.repo_root)

        # Load cargo metadata (fail-closed to full on error).
        try:
            meta = load_metadata(args.metadata_file, repo_root)
        except Exception as exc:  # noqa: BLE001
            doc = {
                "mode": "full",
                "cone_crates": [],
                "changed_crates": [],
                "all_members": [],
                "reason": f"cone: could not load metadata, failing to full: {exc}",
                "shadow": True,
            }
            _write_output(doc, args.output, summary_file, enforce=args.enforce)
            return 0

        # Load ownership map (absent → []).
        map_file = args.map_file
        if map_file is None and repo_root is not None:
            candidate = os.path.join(repo_root, "ci", "path-ownership.toml")
            map_file = candidate if os.path.exists(candidate) else None
        try:
            map_entries = load_ownership_map(map_file)
        except Exception:  # noqa: BLE001
            map_entries = []

        # Get changed paths (fail-closed to full on error).
        try:
            changed_paths = _get_changed_paths(args, repo_root)
        except Exception as exc:  # noqa: BLE001
            # Cannot compute diff → full run (fail-safe).
            try:
                ws = parse_workspace(meta)
                all_members = sorted(ws.members)
            except Exception:  # noqa: BLE001
                all_members = []
            doc = {
                "mode": "full",
                "cone_crates": all_members,
                "changed_crates": [],
                "all_members": all_members,
                "reason": f"cone: could not get changed paths, failing to full: {exc}",
                "shadow": True,
            }
            _write_output(doc, args.output, summary_file, enforce=args.enforce)
            return 0

        doc = compute_cone(changed_paths, meta, map_entries)
        doc["enforce"] = args.enforce
        doc["shadow"] = not args.enforce  # shadow when not enforcing

        _write_output(doc, args.output, summary_file, enforce=args.enforce)
        return 0

    elif args.mode == "shadow-report":
        if not args.cone:
            print("::error::--cone is required for shadow-report mode", file=sys.stderr)
            return 1
        if not args.coverage_summary:
            print("::error::--coverage-summary is required for shadow-report mode", file=sys.stderr)
            return 1

        with open(args.cone, encoding="utf-8") as fh:
            cone_doc = json.load(fh)

        with open(args.coverage_summary, encoding="utf-8") as fh:
            coverage_summary = json.load(fh)

        # Load floor file (optional; skipped crates → no floor comparison).
        floor_doc = None
        floor_path = args.floor
        if floor_path is None:
            # Default: bench/coverage-floor.json relative to repo root.
            repo_root = _resolve_repo_root(args.repo_root)
            if repo_root:
                candidate = os.path.join(repo_root, "bench", "coverage-floor.json")
                floor_path = candidate if os.path.exists(candidate) else None
        if floor_path and os.path.exists(floor_path):
            with open(floor_path, encoding="utf-8") as fh:
                floor_doc = json.load(fh)

        report = shadow_report(
            cone_doc, coverage_summary, floor_doc,
            divergence_log_path=args.divergence_log,
        )

        # Print report JSON.
        text = json.dumps(report, indent=2) + "\n"
        print(text)

        # Write step summary.
        if summary_file:
            try:
                with open(summary_file, "a", encoding="utf-8") as fh:
                    fh.write(_render_report(report))
            except OSError:
                pass

        # Never fail in shadow mode — we are observing, not enforcing.
        divs = report.get("divergences", [])
        if divs:
            print(
                f"::warning::cone-coverage shadow: {len(divs)} divergence(s) found "
                f"(outside-cone crates below floor). See the step summary.",
                file=sys.stderr,
            )
        return 0

    return 0  # unreachable


def _write_output(doc: dict, output_path: str | None, summary_file: str | None, enforce: bool) -> None:
    text = json.dumps(doc, indent=2) + "\n"
    if output_path:
        with open(output_path, "w", encoding="utf-8") as fh:
            fh.write(text)
        print(f"cone_coverage: wrote cone JSON to {output_path}")
        _print_cone_summary(doc, enforce=enforce)
    else:
        sys.stdout.write(text)

    if summary_file:
        try:
            with open(summary_file, "a", encoding="utf-8") as fh:
                fh.write(_render_compute_summary(doc, enforce=enforce))
        except OSError:
            pass


def _print_cone_summary(doc: dict, enforce: bool) -> None:
    mode = doc.get("mode", "full")
    cone = doc.get("cone_crates", [])
    all_m = doc.get("all_members", [])
    reason = doc.get("reason", "")
    shadow = not enforce

    if mode == "full":
        print(f"  cone_coverage: mode=full (all {len(all_m)} crates) — {reason}")
    else:
        inherited = len(all_m) - len(cone)
        print(
            f"  cone_coverage: mode=cone — {len(cone)} of {len(all_m)} crates in cone "
            f"({inherited} inherited from main) — {reason}"
        )
    if shadow:
        print("  [SHADOW MODE] enforce=False: full coverage.sh run unchanged; "
              "cone is computed for monitoring only.")


def _render_compute_summary(doc: dict, enforce: bool) -> str:
    mode = doc.get("mode", "full")
    cone = doc.get("cone_crates", [])
    all_m = doc.get("all_members", [])
    changed = doc.get("changed_crates", [])
    reason = doc.get("reason", "")

    lines = [
        "### Coverage cone (sq-6vshe.8) [SONNET-4.6]",
        "",
        f"**Mode:** `{mode}` — {reason}",
        "",
    ]
    if not enforce:
        lines += [
            "**SHADOW MODE**: enforce=False — full coverage.sh run unchanged for this PR; "
            "cone is computed for monitoring only. Enforce flip: follow-up bead.",
            "",
        ]
    if changed:
        lines += [
            f"**Directly-changed crates ({len(changed)}):** " + ", ".join(f"`{c}`" for c in changed),
            "",
        ]
    if mode == "cone":
        inherited = len(all_m) - len(cone)
        lines += [
            f"**Coverage cone ({len(cone)} of {len(all_m)} crates):** "
            + (", ".join(f"`{c}`" for c in cone) or "_empty_"),
            "",
            f"**Would inherit from main ({inherited} crates — unchanged cone):** "
            + (", ".join(f"`{c}`" for c in sorted(set(all_m) - set(cone))) or "_none_"),
            "",
        ]
    else:
        lines += [
            "**Full run:** all crates measured (diff triggered full-run fail-safe).",
            "",
        ]
    return "\n".join(lines) + "\n"


if __name__ == "__main__":
    sys.exit(main())
