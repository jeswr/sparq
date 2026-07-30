#!/usr/bin/env bash
# ci-cache-audit.sh — measure GitHub-Actions cache usage per key FAMILY against the
# repo's 10 GiB budget, and say whether the families are thrashing. [FABLE-5]
# Bead sq-6vshe.5 (cache topology); design research/ci-structural-speedup.md §6.4.
#
# A "family" is a cache key with its trailing hash segments stripped — for
# Swatinem/rust-cache keys that recovers `v0-rust-<shared-key>`; npm/buildx/
# actions/cache keys collapse similarly. The verdict thresholds:
#   < 70% of budget  → OK       (headroom; GHA backend is fine)
#   70–90%           → PRESSURE (LRU eviction is near; watch the top families)
#   > 90%            → THRASH   (entries are being evicted between save and restore —
#                                 execute the §6.4 contingency: sccache + S3 with a
#                                 ~30-day lifecycle expiry, OIDC-scoped read/write
#                                 role split per research/ci-ec2-design.md)
#
# Needs: `gh` authenticated with actions:read on the repo (GH_TOKEN in CI), python3.
# Set GH_REPO to override the repo (defaults to the current directory's origin).
# Output is GitHub-flavoured markdown (pipe to $GITHUB_STEP_SUMMARY in CI).
# Exit code: 0 whenever the audit RAN (any verdict); nonzero only on execution error.
set -euo pipefail

repo="${GH_REPO:-}"
if [ -z "$repo" ]; then
  repo="$(gh repo view --json nameWithOwner --jq .nameWithOwner)"
fi

# Paginate the whole cache list (the repo can hold hundreds of entries) into a temp
# file: the Python heredoc below occupies stdin (that's where `python3 -` reads its
# program), so the JSONL must arrive on an explicit path, not a pipe.
jsonl="$(mktemp)"
trap 'rm -f "$jsonl"' EXIT
gh api --paginate "repos/${repo}/actions/caches?per_page=100" \
  --jq '.actions_caches[] | {key, ref, size_in_bytes, last_accessed_at}' >"$jsonl"

python3 - "$repo" "$jsonl" <<'PY'
import json, re, sys
from collections import defaultdict

repo = sys.argv[1]
BUDGET = 10 * 1024**3  # GitHub's per-repo Actions cache budget: 10 GiB, LRU-evicted.

with open(sys.argv[2], encoding="utf-8") as fh:
    entries = [json.loads(line) for line in fh if line.strip()]

HEXISH = re.compile(r"^[0-9a-fA-F]{8,}$")

def family(key: str) -> str:
    # Strip trailing hash-like dash-segments (rust-cache appends env-hash + lockfile
    # hash; setup-node/actions-cache append a single long hash; buildx keys are opaque
    # and mostly collapse to their prefix).
    parts = key.split("-")
    while len(parts) > 1 and HEXISH.match(parts[-1]):
        parts.pop()
    return "-".join(parts)

total = sum(e["size_in_bytes"] for e in entries)
fam = defaultdict(lambda: {"bytes": 0, "count": 0, "main": 0, "pr": 0})
main_bytes = pr_bytes = 0
for e in entries:
    f = fam[family(e["key"])]
    f["bytes"] += e["size_in_bytes"]
    f["count"] += 1
    on_main = e.get("ref") in ("refs/heads/main", None)
    f["main" if on_main else "pr"] += e["size_in_bytes"]
    if on_main:
        main_bytes += e["size_in_bytes"]
    else:
        pr_bytes += e["size_in_bytes"]

pct = 100.0 * total / BUDGET
if pct > 90:
    verdict = "🔴 **THRASH LIKELY** — usage exceeds 90% of the budget; LRU eviction is racing the restore path. Execute the sccache+S3 contingency (research/ci-structural-speedup.md §6.4: ~30-day lifecycle expiry, OIDC read/write role split per research/ci-ec2-design.md)."
elif pct > 70:
    verdict = "🟡 **PRESSURE** — within 70–90% of the budget. Trim or consolidate the top families below before adding new ones; re-run after the next prime."
else:
    verdict = "🟢 **OK** — comfortable headroom on the GHA backend; no sccache/S3 move warranted on this measurement."

def gib(n): return f"{n / 1024**3:.2f} GiB"

print(f"## Actions-cache sizing audit — `{repo}`")
print()
print(f"- **Total**: {gib(total)} of {gib(BUDGET)} budget (**{pct:.0f}%**), {len(entries)} entries across {len(fam)} families")
print(f"- **Scope split**: {gib(main_bytes)} main-scoped / {gib(pr_bytes)} PR/branch-scoped "
      f"(PR-scoped bytes are mostly duplicates of main's dependency caches — the first "
      f"lever under pressure is `save-if: main` on the offending family)")
print(f"- {verdict}")
print()
print("| family | size | entries | main-scope | PR/branch-scope |")
print("|---|---|---|---|---|")
for name, f in sorted(fam.items(), key=lambda kv: -kv[1]["bytes"])[:20]:
    print(f"| `{name}` | {gib(f['bytes'])} | {f['count']} | {gib(f['main'])} | {gib(f['pr'])} |")
rest = len(fam) - 20
if rest > 0:
    dropped = sum(f["bytes"] for f in sorted(fam.values(), key=lambda v: -v["bytes"])[20:])
    print()
    print(f"…and {rest} smaller families totalling {gib(dropped)} (not shown).")
PY
