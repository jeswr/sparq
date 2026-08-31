# Corpus-row audit of the four sibling noir drafts (#13263–#13266) — issue #5686

> 🤖 **SPARQ agent** [OPUS-5] — follow-up to issue #5063 / `sq-b0vpc`, epic `sq-uuvac`.
> §6.3(a) of `noir-bounded-vec-capacity-assert-5027.md` found the **"sparq compose
> corpus (8 packages): 0 delta"** row of draft noir#13314 **non-evidential**: the patch
> changes `BoundedVec::push`, which no package in the corpus (nor its dependency
> closure) can call, so the row would have read identically for a broken patch. This
> record applies **the same test** to the four sibling drafts the program record lists
> in §10.1 — noir#13263, #13264, #13265, #13266 — against the question the issue asks:
> *does any package in the corpus actually reach the changed code path?*
> **Date:** 2026-08-02.
>
> **The question is not fully answerable today, and this record does not pretend it is.**
> None of the four PRs names the packages in its corpus (§3), so the audit runs against a
> **substitute** — §5.2's documented 8 `zk/compose` packages — and every per-PR finding in
> §4.2–4.5 and §5 is conditional on that substitution being right (condition **(C)**,
> §4.1). (C) is unestablished for all four. The verdicts on the PRs' *actual* rows are
> therefore **unresolved pending provenance**; the one-line repair ask in §5 is what
> resolves them. Two findings do **not** depend on (C) and stand outright: #13264's row is
> a *disclosed* null because its body states the mechanism, and #13266 is carried by 6
> named in-tree movers rather than by its corpus row.
>
> **Nothing here is a measurement.** `command -v nargo bb` is empty in this session, as
> it was in the two sessions this record follows. Every finding below is either (i) text
> quoted from an upstream page fetched read-only on 2026-08-02, (ii) a source-level grep
> over this repo at `HEAD` and over the corpus's two pinned dependencies fetched at their
> tags, or (iii) a `git` fact about this repo. No circuit was compiled and no delta was
> re-taken. Per `sq-qhy4` nothing below certifies any ZK claim as sound.
>
> **Reference convention** (as in `noir-bounded-vec-capacity-assert-5027.md` §6):
> `§5.1`, `§5.2`, `§6 item 5`, `§10.x` are `noir-optimization-program.md`; references to
> *this* record are written `§3`, `§4.2`, etc.

## 1. Two corrections to the premise, before the audit

**(a) The sibling drafts carry no corpus row in their commit messages, and none in this
repo's records either.** Issue #5686 states that *"the same corpus is quoted in the
sibling program records"*. It is not. All eight commit messages across the four PRs were
read in full from `13263.patch`, `13264.patch`, `13265.patch`, `13266.patch` (fetched
2026-08-02): **not one contains the word "corpus", a delta table, or a package count** —
that shape is unique to #13314's `efec45ef`. The in-repo companion records are likewise
clean: `noir-single-limb-decomposition-13265.md` says *"no ACIR-opcode delta and no gate
count"* and `noir-truncate-after-and-mask-8628.md` says *"No opcode delta, no gate count,
and no compile-time figure were produced here"*; both label corpus frequency **unmeasured**.

The corpus rows the issue is after live in the **PR bodies** (§2). That relocates the
repair target: there is nothing in this repository to repair, and nothing an agent can
repair at all — a PR body is editable only by its named human author, which is exactly the
§6 protocol boundary. The deliverable of this audit is therefore the per-row verdict plus
the repair text (§5), for @jeswr to apply before the draft → ready flip.

**(b) The issue's PR mapping is off by one row.** It names *"§10.13 `sq-9xhoa` /
noir#13266 truncate-after-and-mask"*. Per §10.1 and §10.13, `sq-9xhoa` (row 2,
truncate-after-`and`-mask, #8628) is **noir#13264**; **noir#13266** is row 4
(`sq-rxir8`, dominating-bound range/`Lt` elision, #9463) and has no companion record in
this repo. This audit covers all four regardless, so the mapping error changes no scope.

## 2. The four rows, quoted verbatim

From each PR's body, fetched read-only on 2026-08-02. Bold is this record's.

| PR (bead, row) | the corpus row, verbatim |
|---|---|
| **#13263** (`sq-3qwv1`, row 1 — unsigned div/mod by an oversized constant) | *"a **13-program** external corpus of real ZK circuits (fixed-point/IEEE-754 filter and scan kernels, 65–22313 opcodes): all unchanged"* — and in the summary caveat: *"noir benchmarks and a 13-program external corpus are unchanged"* |
| **#13264** (`sq-9xhoa`, row 2 — truncate after an `and` mask, #8628) | *"an **11-program** external corpus of real ZK circuits (fixed-point/IEEE-754 filter and scan kernels, 65–22313 opcodes): all unchanged — **real-world code overwhelmingly masks with `2^n - 1`, which is already canonicalized before this pass**"* |
| **#13265** (`sq-fwcuo`, row 3 — single-limb `to_bits`/`to_radix`) | *"an **8-program** external corpus of real ZK circuits (fixed-point filter/scan kernels, 65–22313 opcodes): all unchanged — **real code rarely decomposes into a single limb explicitly**"* |
| **#13266** (`sq-rxir8`, row 4 — dominating-bound range/`Lt` elision, #9463) | *"an external corpus of **31 real ZK circuit binaries and 8 library-kernel probes** (IEEE-754 double arithmetic, XPath string/numeric kernels, 34–22313 opcodes): all unchanged apart from the 6 programs listed above"* — summary caveat: *"an external **39-program** corpus … is unchanged — **the win concentrates in code with repeated dynamic-index bounds checks on the same value**"* |

Three of the four already attach a **mechanism** to the null (bold above). That is the
distinction #13314's row lacks, and it is the difference between *"we ran a regression
check and it passed"* and *"this corpus cannot move, and here is why"*. §4 tests whether
each stated mechanism is the true one.

**Provenance, in contrast to #13314.** §6.1 of the companion record found #13314's table
un-re-takeable *by anyone* because it names no fixture, no `nargo` version and no `bb`
version. **That finding does not replicate here.** All four sibling bodies name
`bb 5.0.0-nightly.20260522` and `bb gates -s ultra_honk`, give ACIR opcodes via
`nargo info --force`, and each names a **fixture committed in the PR itself** —
`div_mod_by_oversized_constant`, `truncate_after_and_mask`, `to_radix_single_limb`,
`redundant_range_check_elision`. The focused halves of these four tables are re-takeable
on any box with the two binaries; only the external-corpus half has an identity problem
(§3). §5.1's *"a fleet-box `bb gates` number is non-canonical"* still applies to any
re-take, and the upstream `noir-gates-diff` sticky comment remains the arbiter.

## 3. Which corpus? Four different answers, none of them named

The four counts are **13 / 11 / 8 / 39**, in PRs authored within 90 minutes of each other
on 2026-07-04. No PR names its packages or the sparq commit they came from, so no reader
can reconstruct any of the four sets. What *is* checkable from this repo:

- **§5.2's documented sparq corpus is 11 programs** — 8 `zk/compose` bins plus 3 probe
  bins written for that assessment and *"kept out of the sparq repo"*.
- **The "65–22313 opcodes" range in #13263/#13264/#13265 is exactly §5.2's own extremes**:
  `filter_int_d1` = 65 (min) and `scan_k2_n64_r8` = 22313 (max). That pins the two
  *extremal* §5.2 packages into all three sets — it does not by itself identify the rest —
  and #13265's count of 8 matches §5.2's compose half, #13264's 11 matches §5.2 entire.
- **#13266's "31 real ZK circuit binaries" checks out against this repo.** `zk/compose`
  held exactly **31** packages with `type = "bin"` throughout 2026-07-04 (checked at
  `397ce990`, that day's last commit; the next bin to land was the `path_reach_d*` family
  on 2026-07-06). Its "8 library-kernel probes" exceeds §5.2's three, and probes are out
  of tree, so that half is unverifiable here.
- **#13263's 13 is undocumented**: two more than §5.2's 11, and neither the two extras nor
  the other eleven are identified — the set is consistent with §5.2's 11 plus two unknowns,
  but nothing rules out a different 13 sharing the same extrema.

So the four counts do not describe one corpus, and only the largest is reconstructible.
This is a **provenance** defect, milder than #13314's and independent of the evidential
question in §4 — a row can be perfectly reproducible and still carry no information, which
is precisely #13314's failure mode.

## 4. The reachability test

### 4.1 Method, and what it can and cannot show

§6.3(a)'s test: take the patched code path, identify the source construct that reaches it,
and grep the corpus **plus its whole dependency closure** for that construct. If the
construct is absent, the "unchanged" row is *entailed by the shape of the patch* and would
read identically for a broken one.

The corpus audited below is **§5.2's 8 `zk/compose` packages** — the only 8 this repo
documents, matching #13265's count and reproducing the opcode range all three smaller
corpora report (§3). Their closure, established by reading each `main.nr`'s single `use`:

| package | entry | reached `compose_core` module |
|---|---|---|
| `scan_k2_n64_r8`, `scan_k1_n16_r4` | `scan_check` | `scan.nr` |
| `filter_f64`, `filter_f64_d4` | `filter_f64_check` / `filter_f64_composable_check` | `filter_float.nr` |
| `filter_decimal_i3_f2`, `filter_signed_int_d4` | `filter_decimal_check` / `filter_signed_int_check` | `filter_signed.nr` |
| `filter_int_d4`, `filter_int_d1` | `filter_int_check` | `filter_int.nr` |

plus `hashes.nr` (the only shared import) and, transitively, the two pinned external
dependencies — `poseidon` `v0.3.0` (9 `.nr` files) and `sparq_ieee754` `v0.11.0` (14 `.nr`
files), both fetched at their tags for this audit. The other eleven modules `lib.nr`
declares (`filter_value`, `revoke`, `issuer`, `holder`, `join`, `path`, `entail`,
`derivation`, `sameas`, `n3`, `tests`) are **not** imported by any of the 8 and so are never
monomorphised into their ACIR — the same mechanism that made #13314's row vacuous.

**Limits, stated up front.** These are source-level greps, not compilations. They can show
that a construct is *absent* from the source (a strong, checkable claim) and that one is
*present* (likewise); they cannot show what the compiler ultimately emits, so every
statement below about a *compiler-generated* site is argued from the pass sources already
cited in the companion records, not measured. The programs in #13263's 13 that are not in
§5.2, and the out-of-tree probes, are not covered at all.

**The substitution, and the condition it puts on every verdict below.** No PR names its
packages (§3), so this audit is run against a **substitute** corpus — §5.2's documented 8
`zk/compose` packages — and every verdict in §4.2–4.5 and §5 is therefore conditional on

> **(C)** the PR's own corpus being §5.2's 8 `zk/compose` packages, or a superset of them.

**Nothing in this repo establishes (C) for any of the four**, and the two facts that make
(C) *plausible* are not identifications:

- **Matching counts do not identify a set.** #13265's 8 equals §5.2's compose half and
  #13264's 11 equals §5.2 entire, but any other 8 or 11 packages would match equally.
- **Matching extrema pin two members, not eight.** The 65–22313 opcode range shared by
  #13263/#13264/#13265 is §5.2's own min and max, which places `filter_int_d1` and
  `scan_k2_n64_r8` in each of those three sets. It says nothing about the remaining six,
  and in particular does not place `filter_f64`, `filter_f64_d4`, `filter_decimal_i3_f2`,
  `filter_signed_int_d4`, `filter_int_d4` or `scan_k1_n16_r4` in any of them.

So what follows are verdicts about **§5.2's 8 packages**. **The verdict on each PR's actual
row is unresolved, and stays unresolved until the corpus is named** — which is the §5 ask
common to all four. If a PR's set turns out to differ, its verdict has to be re-taken
outright rather than adjusted.

### 4.2 #13265 (single-limb `to_bits`/`to_radix`) — **under (C), the row is non-evidential, same class as #13314**

The patch rewrites the `ToBits`/`ToRadix` arm of `simplify_call`. Reaching it at all
requires a radix decomposition in the SSA; *firing* additionally requires that
decomposition to be **one limb**.

- **Explicit sites in the closure: zero.** No `to_le_bits`, `to_be_bits`, `to_le_radix`,
  `to_be_radix`, `to_le_bytes` or `to_be_bytes` appears in `scan.nr`, `filter_int.nr`,
  `filter_float.nr`, `filter_signed.nr` or `hashes.nr`, nor anywhere in `poseidon` v0.3.0
  or `sparq_ieee754` v0.11.0. `compose_core` *does* call `to_le_bits` — at
  `holder.nr:164`, `issuer.nr:141`, `issuer.nr:346`, `revoke.nr:101`, `revoke.nr:286` —
  but all five are in modules no package in the 8-program corpus imports (§4.1).
- **Compiler-generated sites: none found.** §10.12's frequency finding is that
  `remove_bit_shifts` emits `to_le_bits(v) -> [u1; 1]` for a **variable** shift amount
  whose max bit width is 1. Every variable-amount shift in the closure is inside an
  `unconstrained` (Brillig) function — `ops/kernels.nr:267,305,309,376,403,410,672` — and
  the four constrained shifts (`ops/kernels.nr:9,49,58,164`) shift by comptime generic
  parameters. `sparq_ieee754` deliberately avoids constrained variable shifts: it hints
  the shift unconstrained and pins it with a multiplication (`left_shift_exact_u64_bounded`,
  `ops/kernels.nr:1035-1043`).

So on **§5.2's** 8-package corpus the patched arm is **never reached — it neither fires nor
declines**. Under (C) — i.e. *if* those are the 8 packages the row was taken over, which #13265's
body does not say (§4.1) — "all unchanged" is entailed by the patch's shape
exactly as #13314's row was, and the stated mechanism (*"real code rarely decomposes into a
single limb explicitly"*) is also the wrong reason, since §5.2's 8 contain **no explicit
decomposition of any width** and so single-limb rarity is not what produced the null. If #13265's
8 are some *other* 8 — its count matches §5.2's compose half but a count is not an
identification — none of that transfers and the row's status is simply unknown.

Two repairs are available and cheap, and they are not the same repair:

1. **To exercise the guard** (would catch a rule that fires when it must not): run the row
   over the packages that *do* decompose — `holder_pok`, `hidden_issuer_d4`,
   `holder_set_d4`, `revoke_unset_d10` were all already in #13266's 31-binary set on
   2026-07-04, so this costs nothing but a corpus selection; `revoke_hidden_ref_d10_a4` and
   the `path_reach_d*` family landed later and widen it further.
2. **To exercise the rewrite** (would catch a wrong single-limb output): impossible on this
   corpus at any selection. The smallest decomposition width anywhere in `zk/` is
   `D = 2` (`path_reach_d2_k1_n16`), with `D = 4`/`D = 10` elsewhere and
   `SCALAR_BITS = 251` at `issuer.nr:85` — **no `N = 1` site exists in the sparq corpus**,
   which is why the PR's own `to_radix_single_limb` fixture is the only thing that can move.

### 4.3 #13264 (truncate after an `and` mask) — **under (C), row is entailed, but the PR says so**

Two thresholds, and they are keyed on *different* instructions — the companion record's §2
is the source for both. **Reaching the new arm** needs only a `Binary { operator: And }`
instruction: the arm sits in the pass's own walk over instructions and is matched on the
`and` itself, where it records `mask.num_bits()` as a bound. **Firing** — actually removing
a truncation — needs that `and` to have a **constant, non-`2^n − 1`** mask *and* its result
to feed a `Truncate`, which is the pre-existing `Truncate` arm's job. In the closure:

- `compose_core`'s `&` operators are all **boolean** conjunctions of comparisons
  (`filter_int.nr:59`, `filter_float.nr:143`, `filter_signed.nr:63`, `scan.nr:144`). Their
  operand is a value, never a constant mask, so the new arm records no bound for them. If
  they survive to this pass as `And` instructions the arm therefore *runs and declines* on
  them — but that "if" is a claim about what the compiler emits, which a source grep cannot
  settle (§4.1's limits), so **treat the guard as exercised only conditionally**. Nothing
  here shows any of the four feeding a `Truncate`, so no removal can occur on them either
  way.
- `sparq_ieee754`'s constant masks are in `codegen.nr:134-136`, inside
  `decode_unconstrained` — a Brillig function. The constrained `f64::new`
  (`codegen.nr:112-131`) has **no mask at all**: it takes the unconstrained decode as a
  hint and pins it with `assert_eq(decoded.bits(), bits)` plus two
  `assert_max_bit_size` calls.
- The masks that do exist are `exponent_mask` / `mantissa_mask` = `2^n − 1`
  (`codegen.nr:81-82`) — the class the PR itself says is canonicalized to a `Truncate`
  before the pass runs — plus `sign_scale = 2^63` (`codegen.nr:79`), a genuinely
  non-`2^n − 1` mask, but one consumed by an `==` comparison, not a truncate.

So the arm's fire branch is unreachable on §5.2's 8 packages, and under (C) the row would
read identically for a broken bound computation. (#13264's count of 11 matches §5.2 entire,
but a count is not an identification, and the 3 probe bins in §5.2's 11 are out of tree and
were not audited at all.) **But the PR body states that entailment** —
*"real-world code overwhelmingly masks with `2^n - 1`, which is already canonicalized before
this pass"* — and this audit finds that mechanism **correct as far as it goes**. The row is
therefore not being passed off as a regression check; it is a disclosed null — and that
assessment does **not** depend on (C), since it rests on the PR body's own text rather than
on which packages the row covers. The honest tightening is one word: *"overwhelmingly"* →
§5.2's 8 contain **no** qualifying mask, so on those the row cannot move at all.

### 4.4 #13263 (unsigned div/mod by an oversized constant) — **under (C), the row is evidential — and (C) is weakest here**

Firing needs an unsigned `Div`/`Mod` by a constant strictly greater than the dividend's
max; *reaching* the patched query needs only an unsigned `Div`/`Mod` by a constant. The
closure has these on the **constrained** path, reachable from `filter_f64*` via
`f64::from(u64)` (`filter_float.nr:178`): `uint_to_float_parts_u64`
(`ops/kernels.nr:1148`) computes `raw_significand / 8`, `raw_significand % 8` and
`retained % 2` at `ops/kernels.nr:1197-1199`, and `round_nearest_even`
(`ops/kernels.nr:449`) the same shape at `:450-452`. `poseidon2.nr:73,95` divides and mods
by `RATE`.

None of these can fire the rule (a `u64`/`u128` significand's max is far above 8), so the
**decline** path is what §5.2's 8 exercise — and that is exactly the direction in which the
patch is dangerous. The rule fires when `constant > max`, so an **under**-estimating
`get_value_max_num_bits` (the query this PR extends to see through `Truncate`) is the
unsound direction, and an under-estimate would make `8 > max` true and delete a live
`% 8` in `sparq_ieee754`'s rounding path — a change a row over §5.2's 8 would catch as a
nonzero delta. So *under (C)* the bare "unchanged" row is **doing real work**.

**But (C) is least supported for this PR, and the gap lands on the load-bearing package.**
The evidential path above runs through `filter_f64` / `filter_f64_d4`, and nothing in #13263's
body or in this repo places either in its 13-program set: the opcode extrema pin
only `filter_int_d1` and `scan_k2_n64_r8` (§4.1), and neither is a `filter_f64*`. #13263 is
also the one PR whose count (13) **exceeds** §5.2's documented 11, so even the superset
reading of (C) is unestablished — the 13 are consistent with §5.2's 11 plus two unknowns,
but equally with a set that drops `filter_f64*`. Drop those two packages and the sharpest
half of the argument goes with them — the `ops/kernels.nr` sites are reached only via
`f64::from(u64)`, so the live-`% 8`-in-the-rounding-path danger disappears. What survives is
the weaker half: `poseidon2.nr:73,95`'s div/mod by `RATE`, reached through `hashes.nr`,
which all four `compose_core` modules import (`scan.nr:61`, `filter_int.nr:34`,
`filter_float.nr:52`, `filter_signed.nr:47`) and hence every one of §5.2's 8 pulls in — but
only §5.2's 8 are known to, and which packages #13263 actually compiled is the open
question. **The verdict on #13263's actual row is therefore unresolved**, and resolving it
needs nothing more than the package list (§5). Note the inversion that would hold if (C)
does: the one PR whose row carries no mechanism sentence is the one whose row is evidential.

### 4.5 #13266 (dominating-bound range/`Lt` elision) — **corpus half conditional on (C), but the PR does not rest on it**

The pass consumes `range_check`-derived facts and elides implied `range_check`s and
unsigned `Lt`s. Both constructs are dense in the closure — unsigned comparisons at
`filter_int.nr:59`, `filter_signed.nr:75`, `scan.nr:98`, and range checks from the
`assert_max_bit_size` calls in `f64::new` (`codegen.nr:108-109`) and the bounded-shift
kernels — but as everywhere above, that is a statement about §5.2's 8, and #13266's corpus
is a *third* shape again (31 binaries + 8 probes), so the corpus half of its evidence is
conditional on (C) exactly like the others. It matters least here. This PR is the only one
of the four whose measurement table reports **in-tree movers**: 3 programs improve in ACIR
and 3 in Brillig opcodes across noir's own 555-program suite, none regress. Those movers are
named, in-tree and independent of which sparq packages the row covers, so the PR's evidence
does not rest on the corpus row at all — the row is corroboration, and its caveat states a
mechanism for the null. No repair on this axis regardless of how (C) resolves.

## 5. Conditional verdicts and the repair asks

**Read the middle columns as conditional.** Every row below is a finding about **§5.2's 8
`zk/compose` packages**, which this audit substituted for the corpora the PRs decline to
name, and each holds only under **(C)** — that the PR's own corpus is that set or a superset
of it (§4.1). **(C) is unestablished for all four**, so **no row here is a verdict on the
PR's actual corpus row**; those verdicts are *unresolved pending provenance* and the last
column is what resolves them.

| PR | patched path reached by §5.2's 8? | what follows **if (C) holds** | ask before draft → ready |
|---|---|---|---|
| **#13265** | **no** — neither fires nor declines (§4.2) | the row would be **non-evidential**, same class as #13314, and its stated reason not the operative one | **Name the packages first** (below). Then repair or drop: either re-run over the decomposing packages (`holder_pok`, `hidden_issuer_d4`, `holder_set_d4`, `revoke_unset_d10`, `path_reach_d*`) and say the row exercises the *guard* only, or drop it and state plainly that no sparq package decomposes at any width in the measured set and none at `N = 1` at all |
| **#13264** | fire branch **no**; guard reached only on the compiler-emission assumption of §4.3 | the row would be **entailed** — but it is **disclosed either way**: the body already states the mechanism, which is a reading of the PR's text and does *not* depend on (C) | Optional one-word tightening: §5.2's 8 contain *no* qualifying mask, not merely few |
| **#13263** | **yes**, decline path on the constrained IEEE-754 rounding path (§4.4) | the row would be **evidential** — an under-estimating bound query would move it | **Name the packages.** (C) is weakest here: the path runs through `filter_f64*`, which nothing places in the 13-program set, and 13 exceeds §5.2's 11 |
| **#13266** | **yes**, densely | corpus half corroborative only | None on this axis — and this one does not need (C): the 6 named in-tree movers carry the PR independently of the corpus row (§4.5) |
| *(#13314, for contrast)* | *n/a — the patched method is never monomorphised, on any selection* | *non-evidential (§6.3(a))* | *already tracked as §4 item 1c of the companion record* |

**The one ask that unblocks the rest (provenance, §3):** name the corpus. A single line —
the sparq commit and the package list, or a count plus "the `zk/compose` bins at
`<sha>`" — makes every row re-takeable, resolves the 13/11/8/39 discrepancy, and settles
(C), at which point the middle column above becomes a verdict instead of a conditional. It
costs one sentence and is unblocked today; it does **not** need `sq-i50o4`. Until it is
answered, the honest status of #13263's and #13265's rows is **unresolved**, not "evidential"
and "non-evidential".

**What this does not change.** All four PRs remain blocked on @jeswr's §6 author review,
and their gate figures remain fleet-box numbers pending the upstream `noir-gates-diff`
arbiter (§5.1). Nothing here re-opens any of the four PRs' *correctness* verdicts: §10.12
and §10.13 stand unchanged, including §10.12's still-open red-on-wrong-answer gap and
§10.13's signed-type coverage gap. Nothing here is a soundness certification (`sq-qhy4`).

Companion records: `noir-bounded-vec-capacity-assert-5027.md` (§6.3(a), the method),
`noir-single-limb-decomposition-13265.md` (§10.12), `noir-truncate-after-and-mask-8628.md`
(§10.13), `noir-optimization-program.md` (§5.1 measurement doctrine, §5.2 baseline corpus,
§10.1 the four drafts, §10.14 status pointer).
