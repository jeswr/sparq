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

## In flight
- Timing sample of test_packages (8 packages) to estimate full 241-package run.

## Next command
cd /Users/jesght/Documents/GitHub/rdfjs/sparq-zk-xpath/zk/xpath/test_packages/<pkg> && nargo test
(then decide full run vs stratified sample >= 40 packages)

## Remaining tasks
- Run all (or stratified >=40) test packages; record pass/fail per package.
- Fix any beta.21 drift failures minimally; small logical commits.
- Write zk/xpath/VENDOR.md (provenance currently in README.vendor.md — fold in
  or rename) incl. function inventory table for SPARQL builtins.
