# STATUS — zk/xpath IEEE754 migration (branch zk-xpath-ieee754-migration)

Goal: migrate zk/xpath off vendored old noir_IEEE754 (zk/xpath/vendor/ieee754)
onto sparq_ieee754 (zk/ieee754, path dep). NEVER push/merge.

Model: Opus 4.8 (Fable 5 unavailable — flag for re-review/upgrade when Fable
returns). Finishing edits (floor/ceil_float bit-twiddle, bench re-validation,
VENDOR.md migration section) done on Opus 4.8.

## COMPLETE
- Call-site map: ALL old-API usage confined to xpath/src/numeric_types.nr.
- BEFORE gate baseline: /tmp/xpath_float_before_n8_32.json (harness
  zk/xpath/scripts/bench_float_migration.py, n_small=8 n_big=32). Commit 32cd4d8.
- Migration committed (b4aaa18): sparq_ieee754::{f32,f64} wrappers, std::ops
  arithmetic, eq/ne/lt/le/gt/ge kernels, floor/ceil_double library kernels,
  round kept local (ties-toward-+inf), abs via sign-bit mask, from_small_int
  via f32::from(i8), bit-level NaN/special predicates.
- vendor/ieee754 removed (705799e); nargo check green.
- FINISHING EDIT (this session, Opus 4.8): floor_float / ceil_float switched
  from the regressed sparq_ieee754 f32 library kernel to local bit-twiddling
  (130.4 gates/call vs ~325 lib-kernel vs 185.0 old lib). Mirrors the
  already-committed local round_float. KEPT after empirical re-validation;
  inline [OPUS-4.8] tags added.
- Oracle gate RE-RUN (numeric_types.nr changed, prior pass not trusted):
  21/21 float/double packages green, 283/283 tests. xpath lib 67/67;
  xpath_unit_tests 244/244. Zero regression.
- AFTER gate benchmark RE-RUN with bit-twiddle in place:
  /tmp/xpath_float_after_n8_32.json. Net broad win (div_double -95.6%,
  mul_double -85.0%, floor/ceil_float -29.5% vs old lib). Only abs_double
  shows +77.7 gates/call, an artefact of the old foldable .abs(); honest,
  tiny, documented in VENDOR.md.
- VENDOR.md §"Float API migration — sparq_ieee754 (DONE)" written: what
  changed, API differences table, before/after gate-count table, re-gate
  results.
- Toolchain: nargo 1.0.0-beta.21 (noirc 89a0f0fa), bb 5.0.0-nightly.20260324.

## Nothing in flight — migration COMPLETE.
