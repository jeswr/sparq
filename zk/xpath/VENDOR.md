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

## Planned follow-up (out of scope here)

- **Float API migration (stage 2):** a later deliverable will migrate this
  copy off the old free-function IEEE754 API (`vendor/ieee754`, v0.3.1 +
  `u1→u8`) onto the optimized vendored `sparq_ieee754` library at
  `../ieee754` (the `zk-ieee754` branch vendoring). A half-done reference
  migration exists at
  `jeswr/zkp-sparql-workspace:circuits/noir_XPath` branch
  `refactor/new-ieee754-api` (local checkout:
  `/Users/jesght/Documents/GitHub/jeswr/zkp-sparql-workspace/circuits/noir_XPath`)
  — use it as a reference, do not copy it wholesale.
- **Upstreaming:** the beta.21 dep-pinning story, the date-timezone semantic
  fixes, and restoring the 21 green float/double packages to the workspace
  should eventually be upstreamed to jeswr/noir_XPath.
