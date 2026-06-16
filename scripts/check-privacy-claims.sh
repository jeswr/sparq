#!/usr/bin/env bash
# [OPUS-4.8] sq-toze.35 (gate) + sq-qhy4 (the ZK-soundness external-audit gate).
#
# PRIVACY-CLAIM HONESTY GATE — the empirical-honesty mandate (MEMORY: feedback-
# empirical-honesty) in CI form.
#
# WHY: the v1 ZK query-proof verifier is NOT yet externally audited and `sparq-mpc`
# is honest-majority semi-honest only — see SECURITY.md and research/zk-*-audit.md.
# The documented, load-bearing verdict is that the ZK/MPC estate provides NO
# production privacy/soundness guarantee to a relying party pending an external
# cryptographer's sign-off (bead sq-qhy4). So NO doc / README / SKILL / site copy /
# marketing string may state a ZK/MPC privacy-or-soundness property as a *settled,
# achieved* fact without the not-yet-sound caveat. A hedged / aspirational / "model"
# / negative ("NOT sound") mention is fine; an unqualified claim of an achieved
# property is a high-severity honesty defect — this script fails the build on one.
#
# WHAT: greps a fixed user-facing surface (root docs, skills/, site copy, the
# compliance index) for a small set of FORBIDDEN unqualified-claim phrases. A hit is
# a failure UNLESS the offending line carries the inline allow-list marker
#   privacy-claims-allow: <justification>
# which records *why* that exact line is a legitimate hedged/negative/illustrative
# usage (the empirical-honesty audit trail). Whole files that are by-nature
# discussion of the (un)soundness — the research/ design records and the audit docs
# themselves — are path-excluded (they MUST be free to name the properties to argue
# about them); the honesty surface this gate defends is the *outward* claim surface.
#
# This is intentionally a coarse phrase gate, not an NLP classifier: it is tuned so
# the repo passes CLEAN today (after the sq-toze.35 fixes). A future unqualified
# claim trips it; the author then either (a) hedges the wording, or (b) — if the line
# really is a legitimate hedged/negative usage the regex over-matched — appends the
# `privacy-claims-allow:` marker with a one-line justification. Weakening the regex
# or blanket-excluding a live doc surface to "make it pass" defeats the gate and is
# itself an honesty defect.
#
# Run locally:  scripts/check-privacy-claims.sh   (exit 0 = clean, 1 = unqualified claim)
set -euo pipefail

# Repo root (this script lives in scripts/).
ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$ROOT"

# Inline allow-list marker: a scanned line containing this token is exempt. The text
# AFTER the marker is the required human justification (audit trail). [OPUS-4.8]
ALLOW_MARKER='privacy-claims-allow'

# FORBIDDEN phrases — settled-property claim shapes for the ZK/MPC estate. Case-
# insensitive, extended-regex. Each is a phrasing that, UNHEDGED, asserts a privacy
# or soundness guarantee the estate does not yet provide.
PATTERNS=(
  'zero[- ]?knowledge[- ]secure'
  'provably[- ](private|secure)'
  'sound(ness)?[- ](verifier|proof)s?'
  'privacy[- ]?preserving'
  'malicious(ly)?[- ]secure'
  'cryptographically[- ](private|sound|secure)'
)

# Path surface to scan = the OUTWARD claim surface (docs/READMEs/skills/site copy /
# the compliance index). Excludes:
#   - research/ + the audit docs : design records that must name the properties to
#     argue (un)soundness — the honesty source of truth, not a claim surface.
#   - this script + its CI wiring : they QUOTE the forbidden phrases by definition.
#   - the privacy/cryptoreview compliance bodies : they exist to discuss the gap and
#     are line-marked where they legitimately name a phrase.
#   - .git/target/node_modules/vendor : not authored claim surface.
mapfile -t FILES < <(
  git ls-files \
    '*.md' '*.mdx' '*.tsx' '*.ts' \
    ':!:research/**' \
    ':!:**/*audit*.md' \
    ':!:.claude/agents/**' \
    ':!:scripts/check-privacy-claims.sh' \
    ':!:.github/workflows/docs-quality.yml' \
    ':!:.git/**' ':!:target/**' ':!:**/target/**' \
    ':!:**/node_modules/**' ':!:vendor/**' ':!:**/vendor/**' \
    | sort -u
)

# Build one alternation regex.
RE="$(printf '%s|' "${PATTERNS[@]}")"
RE="${RE%|}"

fail=0
hits=0
for f in "${FILES[@]}"; do
  [ -f "$f" ] || continue
  # -n line numbers, -i case-insensitive, -E extended.
  while IFS= read -r line; do
    # `line` is "LINENO:CONTENT".
    if printf '%s\n' "$line" | grep -qiF "$ALLOW_MARKER"; then
      continue   # explicitly allow-listed with an inline justification.
    fi
    hits=$((hits + 1))
    if [ "$fail" -eq 0 ]; then
      echo "::error::unqualified ZK/MPC privacy/soundness claim(s) found — hedge the wording or add an inline '${ALLOW_MARKER}: <why>' marker (see SECURITY.md; sq-qhy4 / sq-toze.35)."
      echo "Forbidden claim phrases (must carry the not-yet-sound caveat):"
    fi
    echo "  ${f}:${line}"
    fail=1
  done < <(grep -niE "$RE" "$f" 2>/dev/null || true)
done

if [ "$fail" -ne 0 ]; then
  echo ""
  echo "check-privacy-claims: FAILED — ${hits} unqualified claim line(s) above."
  echo "The v1 ZK verifier is pending external audit (sq-qhy4); sparq-mpc is semi-honest only."
  echo "Either add the not-yet-sound caveat to the wording, or — if the line is a legitimate"
  echo "hedged/negative/illustrative usage the regex over-matched — append on that line:"
  echo "    ${ALLOW_MARKER}: <one-line justification>"
  exit 1
fi

echo "check-privacy-claims: OK — scanned ${#FILES[@]} file(s); no unqualified ZK/MPC privacy/soundness claim."
