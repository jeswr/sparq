#!/usr/bin/env python3
# [FABLE-5] sq-98w7z.4 — A/B report for the PGO (+BOLT) evaluation.
#
#   report.py <results-dir> <baseline-label> <variant-label>...
#
# Reads, per variant label, the files run.sh measure-variant wrote:
#   <label>/{watdiv,sp2b,bsbm}.tsv   `sparq-cli bench` stdout: name<TAB>rows<TAB>min_us
#   <label>/ingest.txt               best ingest wall seconds (single float)
#   <label>/serve.json               serve_driver.py best batch (rps/p50/p99/bytes)
#
# Emits <results-dir>/REPORT.md (also printed) + <results-dir>/summary.json, and enforces
# the HARD correctness differential: every query's row count and the serve response byte
# total must be IDENTICAL across variants (a PGO/BOLT build must never change results) —
# exit 1 on any mismatch.
#
# Delta convention: speedup % = (baseline/variant - 1) * 100 (positive = variant faster).
# The BOLT gate (bead sq-98w7z.4): stack BOLT only if the PGO query geomean is >= 3%.
import json
import math
import sys
from pathlib import Path

SUITES = ["watdiv", "sp2b", "bsbm"]
BOLT_GATE_PCT = 3.0


def read_suite(path: Path):
    rows = {}
    if not path.is_file():
        return None
    for line in path.read_text().splitlines():
        parts = line.rstrip("\n").split("\t")
        if len(parts) != 3:
            continue
        name, nrows, min_us = parts
        if nrows == "ERROR":
            sys.exit(f"[report] FATAL: query {name} errored in {path}: {min_us}")
        try:
            rows[name] = (int(nrows), float(min_us))
        except ValueError:
            continue
    if not rows:
        sys.exit(f"[report] FATAL: no parseable rows in {path}")
    return rows


def read_variant(root: Path, label: str):
    d = root / label
    v = {"suites": {}, "ingest_s": None, "serve": None}
    for s in SUITES:
        v["suites"][s] = read_suite(d / f"{s}.tsv")
    ing = d / "ingest.txt"
    if ing.is_file():
        v["ingest_s"] = float(ing.read_text().strip())
    srv = d / "serve.json"
    if srv.is_file():
        v["serve"] = json.loads(srv.read_text())
    return v


def structure_mismatches(base_label, base, variant_labels, data):
    # The hard differential requires identical result SHAPE, not just agreement on
    # the intersection: a truncated variant (missing suites/queries) or a padded one
    # (extra queries) must fail loudly instead of silently biasing the geomean.
    # (serve.json/ingest.txt presence stays soft — PGO_SKIP_SERVE=1 is a documented
    # cli-only flow; the serve byte differential applies whenever both are present.)
    problems = []
    for lab in variant_labels:
        for suite in SUITES:
            b, vs = base["suites"][suite], data[lab]["suites"][suite]
            if b is None and vs is None:
                continue
            if vs is None:
                problems.append(f"{lab}/{suite}: suite missing from variant (baseline has it)")
                continue
            if b is None:
                problems.append(
                    f"{lab}/{suite}: suite present in variant but missing from baseline {base_label}"
                )
                continue
            missing = sorted(set(b) - set(vs))
            extra = sorted(set(vs) - set(b))
            if missing:
                problems.append(f"{lab}/{suite}: queries missing vs baseline: {', '.join(missing)}")
            if extra:
                problems.append(f"{lab}/{suite}: extra queries vs baseline: {', '.join(extra)}")
    return problems


def speedup_pct(base, new):
    return (base / new - 1.0) * 100.0


def geomean_speedup(pairs):
    # geomean over per-query (base/new) ratios, expressed as a %.
    logs = [math.log(b / n) for b, n in pairs if b > 0 and n > 0]
    return (math.exp(sum(logs) / len(logs)) - 1.0) * 100.0 if logs else 0.0


def main():
    if len(sys.argv) < 3:
        sys.exit("usage: report.py <results-dir> <baseline-label> <variant-label>...")
    root = Path(sys.argv[1])
    base_label, variant_labels = sys.argv[2], sys.argv[3:]
    labels = [base_label] + variant_labels
    data = {lab: read_variant(root, lab) for lab in labels}
    base = data[base_label]

    structural = structure_mismatches(base_label, base, variant_labels, data)
    if structural:
        # Fail BEFORE computing any delta: a shape mismatch means every geomean
        # would be gate-worthy-looking but biased. Also drop any stale summary.json
        # so bolt.sh cannot gate on a previous run's numbers.
        (root / "summary.json").unlink(missing_ok=True)
        report = "\n".join(
            [
                "# PGO evaluation — A/B report (sq-98w7z.4)",
                "",
                "## CORRECTNESS FAILURE (result-set shape)",
                "",
                "No deltas computed: every variant must carry every baseline suite with",
                "an identical query-name set before any comparison is meaningful.",
                "",
            ]
            + [f"- {m}" for m in structural]
        ) + "\n"
        (root / "REPORT.md").write_text(report)
        print(report)
        print(
            "[report] FATAL: variant/baseline result sets differ in shape — no deltas computed:",
            file=sys.stderr,
        )
        for m in structural:
            print(f"  {m}", file=sys.stderr)
        sys.exit(1)

    mismatches = []
    lines = []
    summary = {"baseline": base_label, "variants": {}}

    env = (root / "env.txt").read_text() if (root / "env.txt").is_file() else "(no env.txt)"
    lines.append("# PGO evaluation — A/B report (sq-98w7z.4)")
    lines.append("")
    lines.append("> **NON-CANONICAL.** Produced on a work box (environment below); wall-clock")
    lines.append("> numbers are contention-sensitive. Query timings are engine-internal min-of-N")
    lines.append("> (ratio-robust), ingest/serve are wall-clock. Adoption decisions gate on the")
    lines.append("> canonical quiet-box re-measure bead — do NOT cite these numbers as canonical.")
    lines.append("")
    lines.append("```")
    lines.append(env.rstrip())
    lines.append("```")

    for suite in SUITES:
        b = base["suites"][suite]
        if b is None:
            continue
        lines.append("")
        lines.append(f"## {suite} (engine-internal min_us per query)")
        lines.append("")
        hdr = f"| query | rows | {base_label} µs |"
        sep = "|---|---:|---:|"
        for lab in variant_labels:
            hdr += f" {lab} µs | Δ% |"
            sep += "---:|---:|"
        lines.append(hdr)
        lines.append(sep)
        for q in sorted(b):
            row = f"| {q} | {b[q][0]} | {b[q][1]:.1f} |"
            for lab in variant_labels:
                # structure_mismatches() already guaranteed vs exists with the
                # exact baseline query set — no silent intersection here.
                vs = data[lab]["suites"][suite]
                if vs[q][0] != b[q][0]:
                    mismatches.append(
                        f"{lab}/{suite}/{q}: rows {vs[q][0]} != baseline {b[q][0]}"
                    )
                row += f" {vs[q][1]:.1f} | {speedup_pct(b[q][1], vs[q][1]):+.1f} |"
            lines.append(row)
        for lab in variant_labels:
            vs = data[lab]["suites"][suite]
            gm = geomean_speedup([(b[q][1], vs[q][1]) for q in b])
            summary["variants"].setdefault(lab, {})[f"{suite}_geomean_pct"] = round(gm, 2)
            lines.append("")
            lines.append(f"**{suite} geomean speedup ({lab} vs {base_label}): {gm:+.2f}%**")

    lines.append("")
    lines.append("## ingest (best wall-clock, full parse+intern+index build)")
    lines.append("")
    if base["ingest_s"] is not None:
        lines.append(f"| variant | seconds | Δ% vs {base_label} |")
        lines.append("|---|---:|---:|")
        lines.append(f"| {base_label} | {base['ingest_s']:.3f} | – |")
        for lab in variant_labels:
            s = data[lab]["ingest_s"]
            if s is None:
                continue
            pct = speedup_pct(base["ingest_s"], s)
            summary["variants"].setdefault(lab, {})["ingest_pct"] = round(pct, 2)
            lines.append(f"| {lab} | {s:.3f} | {pct:+.1f} |")

    lines.append("")
    lines.append("## serve (sparq-server binary, loopback; best batch)")
    lines.append("")
    if base["serve"] is not None:
        lines.append(f"| variant | req/s | p50 ms | p99 ms | Δ req/s % |")
        lines.append("|---|---:|---:|---:|---:|")
        bs = base["serve"]
        lines.append(f"| {base_label} | {bs['rps']} | {bs['p50_ms']} | {bs['p99_ms']} | – |")
        for lab in variant_labels:
            vs = data[lab]["serve"]
            if vs is None:
                continue
            if vs["total_bytes"] != bs["total_bytes"]:
                mismatches.append(
                    f"{lab}/serve: total response bytes {vs['total_bytes']} != baseline {bs['total_bytes']}"
                )
            pct = (vs["rps"] / bs["rps"] - 1.0) * 100.0
            summary["variants"].setdefault(lab, {})["serve_rps_pct"] = round(pct, 2)
            lines.append(f"| {lab} | {vs['rps']} | {vs['p50_ms']} | {vs['p99_ms']} | {pct:+.1f} |")

    # Overall query geomean across all suites -> the BOLT gate.
    lines.append("")
    lines.append("## verdict")
    lines.append("")
    for lab in variant_labels:
        pairs = []
        for suite in SUITES:
            b, vs = base["suites"][suite], data[lab]["suites"][suite]
            if b:
                pairs += [(b[q][1], vs[q][1]) for q in b]
        gm = geomean_speedup(pairs)
        summary["variants"].setdefault(lab, {})["query_geomean_pct"] = round(gm, 2)
        lines.append(f"- **{lab}: overall query geomean {gm:+.2f}%** vs {base_label}")
    pgo_gm = summary["variants"].get("pgo", {}).get("query_geomean_pct")
    if pgo_gm is not None:
        if pgo_gm >= BOLT_GATE_PCT:
            lines.append(f"- PGO geomean >= {BOLT_GATE_PCT}% — the BOLT stage is unlocked: `bench/pgo/bolt.sh`")
        else:
            lines.append(f"- PGO geomean < {BOLT_GATE_PCT}% — per the bead gate, do NOT stack BOLT")
    lines.append("- Adoption (shipped profiles / dist.yml) is OUT OF SCOPE here — a follow-up bead")
    lines.append("  decides from the canonical re-measure plus dist-pipeline complexity.")

    if mismatches:
        lines.append("")
        lines.append("## CORRECTNESS FAILURE")
        lines.append("")
        for m in mismatches:
            lines.append(f"- {m}")

    report = "\n".join(lines) + "\n"
    (root / "REPORT.md").write_text(report)
    print(report)

    if mismatches:
        # No summary.json on failure (and drop any stale one): bolt.sh gates on its
        # geomean, and a failed differential must never leave a gate-usable number.
        (root / "summary.json").unlink(missing_ok=True)
        print("[report] FATAL: correctness differential failed — an optimized build changed results:", file=sys.stderr)
        for m in mismatches:
            print(f"  {m}", file=sys.stderr)
        sys.exit(1)

    (root / "summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    print(f"[report] wrote {root / 'REPORT.md'} + summary.json", file=sys.stderr)


if __name__ == "__main__":
    main()
