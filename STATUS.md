# STATUS — zk-xpath audit/completion (successor agent)

Branch: zk-xpath, worktree /Users/jesght/Documents/GitHub/rdfjs/sparq-zk-xpath
Toolchain: nargo 1.0.0-beta.21, bb 5.0.0-nightly.20260324

## Done
- Verified predecessor commits 95e8019 + 81d08c5 present, tree clean.
- Workspace layout confirmed: zk/xpath/Nargo.toml has 243 members
  (xpath lib + xpath_unit_tests + 241 test_packages/*). Workspace manifest is
  byte-identical to upstream (diff vs 95e8019 vendored copy: no changes).
  test_packages/ dir contains 360 dirs; 119 are NOT workspace members upstream
  (prod* XQuery-production tests, *_double/*_float variants, opdaytimeduration_equal).
- `nargo check` in zk/xpath/xpath: PASS (warnings only), 1m24s wall.
- `nargo test` in zk/xpath/xpath: 67/67 pass, 2m18s wall.
- `nargo test` in zk/xpath/xpath_unit_tests: 244/244 pass, 3m54s wall.

- Verified vendored tree byte-identical to upstream fe88a5d except
  xpath/Nargo.toml (documented dep swap), .gitignore, README.vendor.md, vendor/.
- Classified the 241 member test packages by chunk_0.nr content:
  71 REAL (converted qt3tests), 152 STUB (call stub_* which assert(false)
  "not available in ZK" — fail BY DESIGN), 18 PLACEHOLDER (single
  assert(false) "no converted tests" — fail BY DESIGN).
- FULL run of all 241 member packages in progress (4-way parallel,
  /tmp/run_xpath_pkg.sh; per-pkg logs /tmp/xpath_test_logs/*.log, results
  appended to /tmp/xpath_test_logs/summary.txt; pkg list
  /tmp/xpath_member_pkgs.txt; classes /tmp/xpath_pkg_class.txt).
  As of 120/241: all REAL pass except fnadjust_{date,datetime,time}_to_timezone
  (semantic constraint failures, sources identical to upstream → pre-existing
  implementation gaps, NOT beta.21 drift; upstream's own partial result.txt on
  beta.16 also shows non-stub failures e.g. fnround_double).

## In flight
- Remaining ~121 packages running (background). If the background job was
  killed by timeout, compute remaining via:
  comm -13 <(awk '{print $1}' /tmp/xpath_test_logs/summary.txt | sort) <(sort /tmp/xpath_member_pkgs.txt)
  and rerun those with: <list> | xargs -n1 -P4 /tmp/run_xpath_pkg.sh
- Aggregation script ready: /tmp/aggregate_results.sh

## Remaining tasks
- Aggregate final pass/fail; confirm zero compile errors (drift) anywhere.
- Write zk/xpath/VENDOR.md (rename/fold README.vendor.md) with toolchain,
  test results, function inventory table for SPARQL builtins, known gaps,
  stage-2 sparq_ieee754 note. Commit.
