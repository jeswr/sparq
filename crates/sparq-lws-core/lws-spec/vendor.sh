#!/usr/bin/env bash
# [SONNET-4.6] sq-gg0qq.6 — refresh the vendored jeswr/lws-spec conformance corpus.
#
# Usage: crates/sparq-lws-core/lws-spec/vendor.sh <upstream-commit-sha>
#
# Re-clones https://github.com/jeswr/lws-spec at the given commit and replaces the
# vendored subtrees wholesale (no merge, no local patching — see README.md: the SPEC
# WINS, so a vector is never edited here). Prints the new pin so README.md's
# provenance table can be updated in the same commit.
#
# After a refresh, `cargo test -p sparq-lws-core --features access-profile-odrl1
# --test lws_spec_vectors` is EXPECTED to fail if the corpus changed: the suite pins
# the case count and the per-operation coverage ledger in coverage-baseline.json, so
# an added, removed, or re-classified vector must be acknowledged explicitly.
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <upstream-commit-sha>" >&2
  exit 2
fi
sha="$1"

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
tmp="$(mktemp -d)"
trap 'rm -rf "$tmp"' EXIT

git clone --quiet https://github.com/jeswr/lws-spec "$tmp/lws-spec"
git -C "$tmp/lws-spec" checkout --quiet "$sha"

# Only these subtrees are vendored; everything else is read upstream (README.md).
rm -rf "$here/test-vectors" "$here/semantics"
mkdir -p "$here/test-vectors" "$here/semantics"
cp "$tmp/lws-spec/test-vectors/manifest.json" "$here/test-vectors/manifest.json"
cp -R "$tmp/lws-spec/test-vectors/vectors" "$here/test-vectors/vectors"
cp "$tmp/lws-spec/semantics/access-decision.n3" \
   "$tmp/lws-spec/semantics/access-decision.query.n3" "$here/semantics/"

resolved="$(git -C "$tmp/lws-spec" rev-parse HEAD)"
date="$(git -C "$tmp/lws-spec" log -1 --format=%cs)"
cases="$(find "$here/test-vectors/vectors" -name case.json | wc -l | tr -d ' ')"
echo "vendored lws-spec @ $resolved ($date) — $cases cases"
echo "update the provenance table in $here/README.md with that pin."
