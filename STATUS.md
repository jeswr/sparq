# STATUS — zk/xpath IEEE754 migration (branch zk-xpath-ieee754-migration)

Goal: migrate zk/xpath off vendored old noir_IEEE754 (v0.3.1+u1fix at
zk/xpath/vendor/ieee754) onto sparq_ieee754 (zk/ieee754, path dep).

## Done
- Skills loaded (noir-developer/idioms/testing + vendored noir-optimisation,
  noir-circuit-patterns). VENDOR.md + AGENTS.md read.
- Call-site map complete: ALL old-API usage is confined to
  `zk/xpath/xpath/src/numeric_types.nr` (imports at lines 19-30).
  No other file (lib, unit tests, test_packages) touches `ieee754::*`,
  `IEEE754Float*`, or `.value`. Public consumers use only re-exported
  wrapper functions + XsdFloat/XsdDouble::from_bits/to_bits.

## In flight
- Writing gate benchmark harness `zk/xpath/scripts/bench_float_migration.py`;
  will capture BEFORE baseline on the old dep prior to any source change.

## Next command
- `python3 zk/xpath/scripts/bench_float_migration.py --output /tmp/xpath_float_before.json`

## Test gates (to run after migration, in order)
1. Oracle: 21 float/double packages (283 tests) — temporarily appended to
   workspace members (VENDOR.md procedure), must be 21/21 green.
2. xpath lib `nargo test` (67) + xpath_unit_tests (244).
3. 71 REAL test packages: must stay 63 green / 8 pre-existing red (list per
   VENDOR.md; the 8: fnmonths_from_duration, fnyears_from_duration,
   fnadjust_date_to_timezone, fnadjust_datetime_to_timezone,
   fnadjust_time_to_timezone, opdate_equal, opdate_less_than, opsubtract_dates).
4. AFTER benchmark + VENDOR.md §migration table.
