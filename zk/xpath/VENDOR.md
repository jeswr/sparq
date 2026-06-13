# Vendored: noir_XPath

XPath 2.0 functions & operators in Noir (SPARQL FILTER semantics), vendored into
`sparq` as a subfolder for now (per Jesse: work with IEEE754 and XPath as
subfolders in this repository). This is the SPARQL-builtin function layer for
ZK query proofs; the ZK composition package will compose these functions.

## Provenance

- **Upstream:** https://github.com/jeswr/noir_XPath, branch `main`
- **Upstream commit:** `fe88a5d1dec1d6400e9e6c7dc37876753441d85a` (2026-01-29)
- **Vendored on:** 2026-06-12 (tracked files only; `result.txt` test-log artifact dropped)
- **Layout:** `xpath` (lib, ~10.9k lines), `xpath_unit_tests` (bin, 244 tests),
  `test_packages/*` (360 directories on disk, of which **241** are workspace
  members in `Nargo.toml` — the membership list is upstream's, unmodified).
  Test packages are auto-generated from the W3C **qt3tests** suite by
  `scripts/generate_tests.py` (elementpath as evaluation oracle).
- Vendored sources are **byte-identical to upstream** except `xpath/Nargo.toml`
  (dependency swap below), `.gitignore`, and this file (verified with
  `diff -r` against a checkout of fe88a5d).

## Toolchain

- **Verified with:** `nargo 1.0.0-beta.21` (noirc 89a0f0fa), `bb 5.0.0-nightly.20260324`
- Upstream pins Noir `1.0.0-beta.16` (`.github/noir-versions.json`); beta.21
  removed the `u1` type and broke both upstream git dependencies.

## Local changes vs upstream (toolchain drift fixes only)

1. `xpath/Nargo.toml` — both git deps replaced with path deps on `vendor/`:
   - `ieee754` was `jeswr/noir_IEEE754@v0.3.1` (old free-function API:
     `add_float32`, `float64_lt`, `IEEE754Float32`, `ROUNDING_MODE_*`, ...).
     No upstream tag or branch of noir_IEEE754 compiles on beta.21 (all refs
     still use `u1`), and nargo cannot pin git deps to commit SHAs, so the
     exact v0.3.1 tree is vendored at `vendor/ieee754/` with a minimal
     mechanical `u1` -> `u8` substitution (17 sites). See
     `vendor/ieee754/VENDOR-PROVENANCE.md`.
   - `json_parser` was `noir-lang/noir_json_parser@main` — a FLOATING tag
     (drift hazard: the nargo cache holds a stale, beta.21-broken snapshot of
     `main`, and the released tags v0.1.0–v0.4.0 are far too old for beta.21).
     Vendored at `vendor/json_parser/` from upstream `main` commit
     `695b25add4a3229a5808ec0a0d40089c6cecfa60` (2026-05-27), unmodified. See
     `vendor/json_parser/VENDOR-PROVENANCE.md`.
2. No `.nr` source changes in `xpath/`, `xpath_unit_tests/`, or
   `test_packages/` — upstream sources compile on beta.21 as-is (warnings
   only). **No further beta.21 drift fixes were needed** (full audit below).

## Verification on beta.21 (measured 2026-06-12/13)

| Target | Command | Result |
|---|---|---|
| `xpath` lib | `nargo check` | PASS (warnings only) |
| `xpath` lib | `nargo test` | **67/67 pass** (2m18s) |
| `xpath_unit_tests` | `nargo test` | **244/244 pass** (3m54s) |
| 241 member test packages | `nargo test` each (full run, 4-way parallel; per-package times sum to 7,701 s serial-equivalent) | **74 packages green / 167 red — all red packages fail BY DESIGN or are pre-existing upstream gaps; zero beta.21 drift failures** (breakdown below) |
| 21 non-member float/double packages | `nargo test` each (run by temporarily appending them to the workspace `members` list — nargo refuses non-member packages in-tree; manifest reverted afterwards) | **21/21 green, 283 tests total** |

Note: `xpath/src/json_new.nr` is dead code upstream (not declared in
`lib.nr`), so its 5 tests never run — that is why 72 `#[test]` attributes
yield 67 executed tests. Left as-is (minimal-diff policy).

### The 241-package breakdown (full run, every package, no sampling)

The qt3-derived suite was **never fully green upstream**: it doubles as a
coverage map. Generated packages fall into three classes (classified from
`chunk_0.nr` content):

| Class | Count | Behaviour | Result on beta.21 |
|---|---|---|---|
| Real converted qt3 tests | 71 | execute library code | **63 green / 8 red** (all 8 pre-existing, see below) |
| Stub-backed | 152 | call `stub_*()` which `assert(false, "... not available in ZK")` | all red **by design** (unimplemented features: regex, XML/document model, env/context functions, higher-order functions, collation, format-*) |
| Placeholder | 18 | single test "no converted tests" | 7 red by design (`assert(false)`), 11 vacuously green (`assert(true)` — upstream generator inconsistency) |

The 8 red real packages — all **pre-existing upstream** (sources
byte-identical to fe88a5d; failures are deterministic arithmetic/type issues,
not toolchain-dependent; upstream's own beta.16 test log shipped with the repo
also shows non-stub failures):

| Package | Failure | Nature |
|---|---|---|
| `fnmonths_from_duration`, `fnyears_from_duration` | compile error: generated test applies fn to `XsdDayTimeDuration`, lib models it only for `XsdYearMonthDuration` (XPath says return 0) | test-generator type gap |
| `fnadjust_date_to_timezone` (4/6), `fnadjust_datetime_to_timezone` (8/9), `fnadjust_time_to_timezone` (5/8) | local components not recomputed across day boundary when adjusting timezone | timezone semantics gap |
| `opdate_equal` (12/14), `opdate_less_than` (9/10), `opsubtract_dates` (1/3) | xs:date with explicit timezone compared by epoch-day only (tz offset ignored), e.g. `date(d, +00:00) == date(d, +09:00)` should be false | timezone semantics gap |

### Excluded float/double packages

21 generated packages (`fnceiling/fnfloor/fnround_{float,double}`,
`opnumeric_{add,subtract,multiply,divide,equal,less_than,greater_than}_{float,double}`,
`opdaytimeduration_equal`) exist on disk but are **not workspace members**
upstream. Run on beta.21 they are all green (283 tests, incl.
`fnround_double` 87/87 and `fnround_float` 88/88) — so IEEE-754 float/double
F&O on the vendored old float API is fully passing; their exclusion from the
workspace appears to be upstream membership drift, not known breakage.

## Core representations (relevant to composition design)

- integers: `i64` (`xs:integer`)
- strings: `str<N>` with comptime lengths (`u8` byte semantics; no Unicode normalization)
- float/double: `XsdFloat`/`XsdDouble` wrapping `IEEE754Float32`/`IEEE754Float64` (old vendored API)
- dateTime: `XsdDateTime { epoch_microseconds: Field, tz_offset_minutes: i16 }`; date/time analogous
- durations: `XsdDayTimeDuration` (microseconds), `XsdYearMonthDuration` (months)

## Function inventory — SPARQL builtins over XPath F&O

Naming is mechanical: XPath `fn:x-y` → `xpath::xpath_fn::x_y`, `op:x` →
`xpath::xpath_op::x`, casts in `xpath::xpath_xs`. Test evidence column:
"qt3 n" = green generated package with n tests; "unit" = covered by
`xpath_unit_tests`/lib inline tests; "untested" = implemented, no executable
coverage anywhere in the repo; "STUB" = `assert(false)` stub.

### Numeric (SPARQL `+ - * /`, `= != < > <= >=`, abs/round/ceil/floor)

| SPARQL | XPath F&O | Symbol | Status / evidence |
|---|---|---|---|
| `+` | op:numeric-add | `numeric_add` (i64), `numeric_add_float/_double` | PASS qt3 54 (int), 4+4 (f/d), unit |
| `-` | op:numeric-subtract | `numeric_subtract[_float/_double]` | PASS qt3 51 / 6+6 |
| `*` | op:numeric-multiply | `numeric_multiply[_float/_double]` | PASS qt3 28 / 5+5 |
| `/` | op:numeric-divide | `numeric_divide[_float/_double]` | PASS qt3 25 / 4+4 |
| (idiv) | op:numeric-integer-divide | `numeric_integer_divide` | qt3 47 green, **but aliased to `numeric_divide_int` — same i64 truncating impl** |
| (mod) | op:numeric-mod | `numeric_mod` | PASS qt3 20 |
| unary `-`/`+` | op:numeric-unary-minus/plus | `numeric_unary_minus/_plus` | PASS qt3 35/33 |
| `=` | op:numeric-equal | `numeric_equal[_float/_double]` | PASS qt3 63 / 9+9 |
| `<` `>` | op:numeric-less/greater-than | `numeric_less_than`, `numeric_greater_than` [..] | PASS qt3 58/58 / 8-9 each |
| `<=` `>=` | (derived) | `numeric_le/ge_int/_float/_double` | unit |
| ABS | fn:abs | `abs` (+`abs_float/_double`) | PASS qt3 13 (int); f/d **untested** |
| ROUND | fn:round | `round` (+`round_float/_double`) | PASS qt3 31 (int), 88+87 (f/d standalone) |
| CEIL | fn:ceiling | `ceiling` (+`ceiling_float/_double`) | PASS qt3 33; f/d qt3 3+3 standalone |
| FLOOR | fn:floor | `floor` (+`floor_float/_double`) | PASS qt3 33; f/d qt3 3+3 standalone |
| (xsd round) | fn:round-half-to-even | `round_half_to_even` (int only) | impl; qt3 STUB; **untested** |
| casts | xs:float/double/integer(...) | `xpath_xs::*` (5 casts) | PASS qt3 2–18 each, unit |

### Boolean (SPARQL `!`, `&&`, `||`, EBV comparisons)

| SPARQL | XPath F&O | Symbol | Status |
|---|---|---|---|
| `!` | fn:not | `not` | PASS qt3 5 |
| `=` `<` `>` on xsd:boolean | op:boolean-equal/less/greater | `boolean_equal/_less_than/_greater_than` (+le/ge) | PASS qt3 41/19/19 |
| true/false | fn:true, fn:false | `fn_true`, `fn_false` | impl, unit (qt3 pkgs stubbed — cast-heavy cases) |
| EBV | fn:boolean | `cast::fn_boolean_from_*` | partial: unit for string/uint; qt3 pkg STUB |

### Strings (STRLEN, SUBSTR, UCASE, LCASE, STRSTARTS, STRENDS, CONTAINS, STRBEFORE, STRAFTER, CONCAT, ENCODE_FOR_URI, comparisons)

| SPARQL | XPath F&O | Symbol | Status |
|---|---|---|---|
| STRLEN | fn:string-length | `string_length` | impl, unit (3); qt3 placeholder (29 unconvertible) |
| CONTAINS | fn:contains | `contains` | impl, unit (4); qt3 placeholder |
| STRSTARTS | fn:starts-with | `starts_with` | impl, unit (3); qt3 placeholder |
| STRENDS | fn:ends-with | `ends_with` | impl, unit (3); qt3 placeholder |
| `=` `<` `>` on strings | op:string-equal/less/greater | `string_equal/_less_than/_greater_than` | impl, **untested** (qt3 pkgs stubbed) |
| (collation compare) | fn:compare | `compare` | impl (codepoint only), **untested** |
| SUBSTR | fn:substring | `string::substring` | impl, **untested** |
| STRBEFORE | fn:substring-before | `string::substring_before` | impl, **untested** |
| STRAFTER | fn:substring-after | `string::substring_after` | impl, **untested** |
| UCASE / LCASE | fn:upper-case / fn:lower-case | `string::upper_case/lower_case` (ASCII) | impl, **untested** |
| CONCAT | fn:concat (2-arg) | `string::concat_bytes` | impl, **untested** |
| (string-join) | fn:string-join | `string::string_join_two` | impl, **untested** |
| ENCODE_FOR_URI | fn:encode-for-uri | `string::encode_for_uri` | impl, **untested** |
| IRI/URI escapes | fn:iri-to-uri, fn:escape-html-uri | `string::iri_to_uri/escape_html_uri` | impl, **untested** |
| REGEX | fn:matches | `stub_fnmatches` | **STUB** — "requires regex engine - not available in ZK" |
| REPLACE | fn:replace | `stub_fnreplace` | **STUB** |
| (tokenize) | fn:tokenize | `tokenize_whitespace` only | whitespace split impl + unit; regex form STUB |
| LANG / LANGMATCHES | fn:lang | `stub_fnlang` | **STUB** (RDF term layer must handle) |
| str(int/bool) | (casts) | `fn_string_from_integer/_boolean` | impl, unit |

### Date/time (YEAR, MONTH, DAY, HOURS, MINUTES, SECONDS, TIMEZONE/TZ, NOW, comparisons, arithmetic)

| SPARQL | XPath F&O | Symbol | Status |
|---|---|---|---|
| YEAR..SECONDS, TZ on xsd:dateTime | fn:*-from-dateTime (7 fns) | `year/month/day/hours/minutes/seconds/timezone_from_dateTime` | PASS qt3 4–9 each |
| components on xsd:date / xsd:time | fn:*-from-date / -time (7 fns) | analogous | PASS qt3 3–9 each |
| `= < >` xsd:dateTime | op:dateTime-equal/less/greater (+le/ge) | `dateTime_*` | PASS qt3 10/9/9 |
| `= < >` xsd:time | op:time-equal/less/greater (+le/ge) | `time_*` | PASS qt3 10/8/7 |
| `= < >` xsd:date | op:date-equal/less/greater (+le/ge) | `date_*` | **partial**: explicit-timezone dates compared by epoch-day only (qt3 12/14, 9/10) |
| dateTime − dateTime | op:subtract-dateTimes | `subtract_dateTimes` | PASS qt3 3 |
| time − time | op:subtract-times | `subtract_times` | PASS qt3 8 |
| date − date | op:subtract-dates | `subtract_dates` | **partial** (qt3 1/3, tz cases) |
| ± duration on dateTime/date/time | op:add/subtract-dayTimeDuration-to-* (6 ops) | `add/subtract_dayTimeDuration_*` | green where qt3 converted (date/time); dateTime variants placeholder-only |
| dayTimeDuration ops | op:add/subtract/multiply/divide/compare (10 ops) | `*_dayTimeDuration*` | PASS qt3 1–7 each (equal: 7 standalone) |
| yearMonthDuration ops | op:add/subtract/multiply/divide/compare (10 ops) + fn:years/months-from-duration | `*_yearMonthDuration*` | PASS qt3 1–23 each; component fns: lib impl + unit, qt3 pkgs have generator type bug (see above) |
| duration components | fn:days/hours/minutes/seconds-from-duration | `*_from_duration` | PASS qt3 5–7 each |
| adjust-*-to-timezone | fn:adjust-*-to-timezone (3 fns) | `adjust_*_to_timezone` | **partial** (day-boundary bugs; 17/23 qt3) |
| NOW | fn:current-dateTime | `stub_fncurrent_datetime` | **STUB** — context-dependent; in ZK must be a public input |
| (IETF date parse) | fn:parse-ietf-date | `ietf_date::parse_ietf_date` | impl, lib tests (9); qt3 pkg STUB |

### Other equality operators usable for RDF term comparison

| XPath F&O | Symbol | Status |
|---|---|---|
| op:anyURI-equal/less/greater | `anyURI_*` | impl, **untested** (qt3 stubbed) |
| op:hexBinary-equal/less/greater | `hexBinary_*` | impl, lib tests (3); qt3 stubbed |
| op:base64Binary-equal/less/greater | `base64Binary_*` | impl, **untested** (qt3 stubbed) |
| op:gYear/gMonth/gDay/gYearMonth/gMonthDay-equal | `g*_equal` | impl, lib tests (5); qt3 stubbed |
| op:QName-equal | `QName_equal` | impl, lib tests (4); qt3 stubbed |
| op:NOTATION-equal | — | not implemented (placeholder) |

### Sequences/aggregates (SPARQL aggregates COUNT/SUM/AVG/MIN/MAX live here)

`empty, exists, count, sum, avg, min, max, head, tail, reverse, index_of,
distinct_values, subsequence, remove, insert_before, sort, deep_equal,
zero_or_one, one_or_more, exactly_one, union, intersect, except, to` —
all implemented for **i64 sequences only** (`sum_int`, `avg_int`, ...), unit
+ lib tests green; the corresponding qt3 packages are stub-backed (generator
could not map sequence arguments). `fn:avg` is integer division (no decimal).

### Not available (stubs assert false; 152 packages document this)

Regex (matches/replace/tokenize/analyze-string), XML node/document model
(doc, id, path, root, node comparisons, ...), serialization/parsing
(parse-xml, serialize, xml-to-json escapes), environment/context
(current-*, environment-variable, implicit-timezone, default-collation,
static-base-uri, position/last), higher-order functions (apply, filter,
fold-*, for-each*, function-*), format-* (date/time/integer/number),
normalize-unicode, collation-key, resolve-QName/uri, json-doc/parse-json
facade (a JSON parser exists in `json.nr` + vendored `json_parser`, but the
fn:parse-json facade is stubbed).

## Known gaps (summary)

1. **Regex** (`REGEX`, `REPLACE`, regex `tokenize`) — stubbed; needs a ZK
   regex strategy (likely NFA-product circuits or out-of-circuit match +
   in-circuit verification) in the composition layer.
2. **Timezone-aware xs:date semantics** — `op:date-equal/less-than`,
   `op:subtract-dates`, `fn:adjust-*-to-timezone` fail 6 qt3 edge cases
   (explicit-tz dates, day-boundary shifts). Pre-existing upstream; fix
   belongs upstream.
3. **Untested string functions** — `substring`, `substring_before/after`,
   `upper/lower_case`, `concat_bytes`, `string_join_two`, `encode_for_uri`,
   `string_equal/less/greater`, `compare` have zero executable coverage.
   Add unit tests before relying on them for SPARQL builtins.
4. **Integer-only aggregates** and `numeric_integer_divide` aliased to plain
   division (same i64 impl); no xs:decimal anywhere.
5. **fn:months/years-from-duration on dayTimeDuration** — generated tests
   don't compile (type mismatch); XPath expects 0.
6. `json_new.nr` dead code; `result.txt`-era upstream failures (e.g.
   fnround_double) are now green — upstream's workspace membership of the 21
   float/double packages should be restored upstream.
7. Strings are byte/ASCII-level (`str<N>`); no Unicode case mapping or
   normalization.

## Float API migration — sparq_ieee754 (DONE 2026-06-13)

**Status: COMPLETE.** The vendored old free-function IEEE754 API
(`vendor/ieee754`, v0.3.1 + `u1→u8`) has been removed and the float layer
in `xpath/src/numeric_types.nr` now sits on `sparq_ieee754` (path dep on
`zk/ieee754`, the `zk-ieee754` branch vendoring). All old-API usage was
confined to `numeric_types.nr`; no public consumer (lib, unit tests,
`test_packages`) was touched — they call only the re-exported wrapper
functions and `XsdFloat`/`XsdDouble::{from_bits,to_bits}`.

`Model: Opus 4.8` — the bench-table re-validation, the floor/ceil_float
bit-twiddle finishing edit, and this section were completed on Opus 4.8
while Fable 5 was unavailable. Flag for re-review when Fable returns.

### What changed

- **Wrappers.** `XsdFloat { value: f32 }` / `XsdDouble { value: f64 }`,
  where `f32`/`f64` are the `sparq_ieee754`-generated types.
  `from_bits(b)` → `f32::new(b)`; `to_bits()` → `value.bits()`.
- **Arithmetic** (`numeric_{add,subtract,multiply,divide}_{float,double}`):
  `std::ops` operators on the wrapped value (`a.value + b.value`, etc.).
- **Comparison kernels** (`eq/ne/lt/le/gt/ge`): IEEE comparison via
  `value.eq(...)` and the `sparq_ieee754` ordering kernels (NaN-aware:
  any comparison with NaN is false; `±0` compare equal).
- **floor / ceil — split strategy, empirically chosen:**
  - `floor_double` / `ceil_double`: `sparq_ieee754` library kernels
    (`a.value.floor()` / `.ceil()`). Measured CHEAPER than the old lib
    (438.9 → 341.8 gates/call), so the kernel is kept.
  - `floor_float` / `ceil_float`: **local bit-twiddling** (exponent
    decode → fraction mask → directional round-away in the sign-appropriate
    direction; specials/integers pass through; `|x|<1` short-circuits to
    `±1`/`±0`). The `sparq_ieee754` f32 floor/ceil kernels REGRESSED
    (185.0 → ~325 gates/call); the local kernel recovers to 130.4/call —
    cheaper than BOTH the regressed library kernel and the original
    vendored lib. This mirrors the already-committed local `round_float`
    kernel, which was kept local for the same measured reason.
    `noir-optimisation §1/§2.4` is the governing rule: the saving is
    backed by the amplified `bb gates` harness, not `nargo info` or
    intuition (intuition has misfired on this codebase before — §3.3).
- **round** (`round_{float,double}`): kept local (ties-toward-+∞),
  bit-twiddling; faster than the old lib (float 256.4 → 201.7).
- **abs** (`abs_{float,double}`): sign-bit mask (`bits & 0x7FFF…`),
  NaN payloads preserved.
- **from_small_int**: `f32::from(n: i8)` / `f64::from` (exact over the
  full i8 range), replacing the old free-function int conversion.
- **NaN / special predicates**: bit-level on the canonical encoding
  (`f32_bits_is_nan`, `f32_bits_is_special`, f64 analogues). `sparq_ieee754`
  keeps struct fields and classifiers private, so these operate on the
  bit pattern directly — cheap mask/compare, no decode round-trip.

### API differences from the old vendored lib

| Concern | Old (`vendor/ieee754`) | New (`sparq_ieee754`) |
|---|---|---|
| Type | `IEEE754Float32` (free-function API) | `f32` (generated, methods) |
| Construct from bits | free fn | `f32::new(bits)` |
| Extract bits | `.value` field (`u1`/`u8` era) | `.bits()` method |
| Arithmetic | free functions | `std::ops` (`+ - * /`) |
| int → float | free fn | `f32::from(i8)` |
| Classifiers | exposed | private — use bit-level predicates |
| floor/ceil f32 | lib fn | local bit-twiddle (measured cheaper) |

### Before / after gate-count table

`bb gates` UltraHonk `circuit_size`, per-call cost via the amplification
harness `scripts/bench_float_migration.py` (N_small=8, N_big=32;
per-call = (gates@32 − gates@8) / 24). BEFORE = old vendored lib
(`/tmp/xpath_float_before_n8_32.json`); AFTER = `sparq_ieee754` +
bit-twiddle floor/ceil_float (`/tmp/xpath_float_after_n8_32.json`).

| op | BEFORE | AFTER | Δ | % |
|---|---:|---:|---:|---:|
| add_double | 1149.5 | 514.2 | −635.3 | −55.3% |
| mul_double | 2708.8 | 405.6 | −2303.1 | −85.0% |
| div_double | 9376.0 | 408.9 | −8967.1 | −95.6% |
| eq_double | 110.2 | 85.0 | −25.2 | −22.9% |
| lt_double | 146.8 | 136.3 | −10.4 | −7.1% |
| round_double | 528.2 | 428.4 | −99.8 | −18.9% |
| floor_double | 438.9 | 341.8 | −97.1 | −22.1% |
| ceil_double | 438.9 | 342.8 | −96.1 | −21.9% |
| abs_double | 0.0 | 77.7 | +77.7 | (see note) |
| add_float | 952.8 | 525.0 | −427.8 | −44.9% |
| mul_float | 1571.0 | 436.8 | −1134.2 | −72.2% |
| lt_float | 140.2 | 119.8 | −20.5 | −14.6% |
| round_float | 256.4 | 201.7 | −54.7 | −21.3% |
| floor_float | 185.0 | 130.4 | −54.7 | −29.5% |
| ceil_float | 185.0 | 130.4 | −54.7 | −29.5% |
| int_to_double | 236.9 | 111.1 | −125.8 | −53.1% |

Net: a broad win. Heavy arithmetic collapses (`div_double` −95.6%,
`mul_double` −85.0%) because `sparq_ieee754`'s kernels are the optimised
ones this workspace's IEEE754 effort produced. **`abs_double` note:** the
BEFORE measurement was 0.0/call because the old `.abs()` constant-folded
away entirely inside the amplified harness; the new sign-bit-mask abs does
real (witness-dependent) work that does not fold, hence 77.7 gates/call.
This is an honest, tiny regression on one op and an artefact of the old
form being foldable — not a correctness or net-cost concern.

### Test gates re-run with the floor/ceil_float bit-twiddle in place

(numeric_types.nr changed after the prior pass, so the oracle gate was
re-run — not trusted from the earlier run.)

- Oracle (21 non-member float/double packages, appended to `members`,
  manifest reverted after): **21/21 green, 283/283 tests** — incl.
  `fnceiling_float` 3/3, `fnceiling_double` 3/3, `fnfloor_float` 3/3,
  `fnfloor_double` 3/3, `fnround_float` 88/88, `fnround_double` 87/87.
- `xpath` lib: **67/67**. `xpath_unit_tests`: **244/244**.
- Toolchain: `nargo 1.0.0-beta.21` (noirc 89a0f0fa), `bb 5.0.0-nightly.20260324`.

## Planned follow-up (out of scope here)

- **Float API migration (stage 2):** ✅ DONE — see "Float API migration —
  sparq_ieee754" above. (The half-done reference migration at
  `jeswr/zkp-sparql-workspace:circuits/noir_XPath` branch
  `refactor/new-ieee754-api` was used as a reference only, not copied
  wholesale.)
- **Upstreaming:** the beta.21 dep-pinning story, the date-timezone semantic
  fixes, and restoring the 21 green float/double packages to the workspace
  should eventually be upstreamed to jeswr/noir_XPath.
