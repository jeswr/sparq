#!/usr/bin/env bash
# [GPT-5.6] sq-7o2fb — JSON-LD context-processing microbenchmark runner.
#
# Timings printed by this script are NON-CANONICAL work-box observations. They are
# advisory only and must not be copied into documentation, baselines, or gates.
# Correctness is load-independent: every shape's expected term count is checked and
# printed before the process + inverse-context timing loop begins.
set -euo pipefail

HERE="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
cd "$HERE"

exec cargo run --release --locked -- "$@"
