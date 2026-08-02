# Corpus-row audit of the four sibling noir drafts (#13263–#13266) — issue #5686

> 🤖 **SPARQ agent** [OPUS-5] — follow-up to issue #5063 / `sq-b0vpc`, epic `sq-uuvac`.
> §6.3(a) of `noir-bounded-vec-capacity-assert-5027.md` found the **"sparq compose
> corpus (8 packages): 0 delta"** row of draft noir#13314 **non-evidential**: the patch
> changes `BoundedVec::push`, which no package in the corpus (nor its dependency
> closure) can call, so the row would have read identically for a broken patch. This
> record applies **the same test** to the four sibling drafts the program record lists
> in §10.1 — noir#13263, #13264, #13265, #13266 — and answers, per PR, the question the
> issue asks: *does any package in the corpus actually reach the changed code path?*
> **Date:** 2026-08-02.
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
- **#13263's 13 is undocumented**: two programs beyond §5.2's 11, identity unknown.

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
cited in the companion records, not measured. The two undocumented packages in #13263's 13
and the out-of-tree probes are not covered at all. And because no PR names its packages
(§3), the audit is run against §5.2's documented 8: if a PR's set is a *different* 8, its
verdict has to be re-taken — which is itself the argument for the §5 provenance ask.

### 4.2 #13265 (single-limb `to_bits`/`to_radix`) — **the row is non-evidential, same class as #13314**

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

So on its own 8-program corpus the patched arm is **never reached — it neither fires nor
declines**, and "all unchanged" is entailed by the patch's shape exactly as #13314's row
was. The stated mechanism (*"real code rarely decomposes into a single limb explicitly"*)
is also the wrong reason: the corpus contains **no explicit decomposition of any width**,
so single-limb rarity is not what produced the null.

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

### 4.3 #13264 (truncate after an `and` mask) — **row is entailed, but the PR says so**

Firing needs an `and` by a **constant, non-`2^n − 1`** mask whose result feeds a
`Truncate`. In the closure:

- `compose_core`'s `&` operators are all **boolean** conjunctions of comparisons
  (`filter_int.nr:59`, `filter_float.nr:143`, `filter_signed.nr:63`, `scan.nr:144`). Those
  are still `And` instructions (on `u1`) so the new arm does *run* and decline on them —
  the guard is exercised — but their operand is a value, never a constant mask.
- `sparq_ieee754`'s constant masks are in `codegen.nr:134-136`, inside
  `decode_unconstrained` — a Brillig function. The constrained `f64::new`
  (`codegen.nr:112-131`) has **no mask at all**: it takes the unconstrained decode as a
  hint and pins it with `assert_eq(decoded.bits(), bits)` plus two
  `assert_max_bit_size` calls.
- The masks that do exist are `exponent_mask` / `mantissa_mask` = `2^n − 1`
  (`codegen.nr:81-82`) — the class the PR itself says is canonicalized to a `Truncate`
  before the pass runs — plus `sign_scale = 2^63` (`codegen.nr:79`), a genuinely
  non-`2^n − 1` mask, but one consumed by an `==` comparison, not a truncate.

So the arm's fire branch is unreachable on this corpus, and the row would read identically
for a broken bound computation. **But the PR body states that entailment** —
*"real-world code overwhelmingly masks with `2^n - 1`, which is already canonicalized before
this pass"* — and this audit finds that mechanism **correct as far as it goes**. The row is
therefore not being passed off as a regression check; it is a disclosed null. The honest
tightening is one word: *"overwhelmingly"* → the corpus contains **no** qualifying mask, so
the row cannot move at all.

### 4.4 #13263 (unsigned div/mod by an oversized constant) — **the row is evidential**

Firing needs an unsigned `Div`/`Mod` by a constant strictly greater than the dividend's
max; *reaching* the patched query needs only an unsigned `Div`/`Mod` by a constant. The
closure has these on the **constrained** path, reachable from `filter_f64*` via
`f64::from(u64)` (`filter_float.nr:178`): `uint_to_float_parts_u64`
(`ops/kernels.nr:1148`) computes `raw_significand / 8`, `raw_significand % 8` and
`retained % 2` at `ops/kernels.nr:1197-1199`, and `round_nearest_even`
(`ops/kernels.nr:449`) the same shape at `:450-452`. `poseidon2.nr:73,95` divides and mods
by `RATE`.

None of these can fire the rule (a `u64`/`u128` significand's max is far above 8), so the
**decline** path is what the corpus exercises — and that is exactly the direction in which
the patch is dangerous. The rule fires when `constant > max`, so an **under**-estimating
`get_value_max_num_bits` (the query this PR extends to see through `Truncate`) is the
unsound direction, and an under-estimate would make `8 > max` true and delete a live
`% 8` in `sparq_ieee754`'s rounding path — a change the row would catch as a nonzero
delta. The bare "unchanged" row is therefore **doing real work here**, and needs no repair.
Note the inversion: the one PR whose row carries no mechanism sentence is the one whose row
is evidential.

### 4.5 #13266 (dominating-bound range/`Lt` elision) — **the row is evidential, and is not the only evidence**

The pass consumes `range_check`-derived facts and elides implied `range_check`s and
unsigned `Lt`s. Both constructs are dense in the closure — unsigned comparisons at
`filter_int.nr:59`, `filter_signed.nr:75`, `scan.nr:98`, and range checks from the
`assert_max_bit_size` calls in `f64::new` (`codegen.nr:108-109`) and the bounded-shift
kernels. More decisively, this PR is the only one of the four whose measurement table
reports **in-tree movers**: 3 programs improve in ACIR and 3 in Brillig opcodes across
noir's own 555-program suite, none regress. A corpus row on top of that is corroboration,
not the load-bearing evidence, and its caveat states a mechanism for the null. No repair.

## 5. Verdicts and the repair asks

| PR | patched path reached by the corpus? | verdict on the row | ask before draft → ready |
|---|---|---|---|
| **#13265** | **no** — neither fires nor declines (§4.2) | **non-evidential**, same class as #13314, and its stated reason is not the operative one | **Repair or drop.** Either re-run over the decomposing packages (`holder_pok`, `hidden_issuer_d4`, `holder_set_d4`, `revoke_unset_d10`, `path_reach_d*`) and say the row exercises the *guard* only, or drop it and state plainly that no sparq package decomposes at any width in the measured set and none at `N = 1` at all |
| **#13264** | guard yes, **fire branch no** (§4.3) | **entailed, but disclosed** — the body already states the mechanism, so it does not read as a regression check | Optional one-word tightening: the corpus contains *no* qualifying mask, not merely few |
| **#13263** | **yes**, decline path on the constrained IEEE-754 rounding path (§4.4) | **evidential** — an under-estimating bound query would move it | None on this axis |
| **#13266** | **yes**, densely; plus 6 in-tree movers | **evidential** and corroborative only | None on this axis |
| *(#13314, for contrast)* | *no — the patched method is never monomorphised* | *non-evidential (§6.3(a))* | *already tracked as §4 item 1c of the companion record* |

**One ask common to all four (provenance, §3):** name the corpus. A single line —
the sparq commit and the package list, or a count plus "the `zk/compose` bins at
`<sha>`" — makes every row re-takeable and resolves the 13/11/8/39 discrepancy. It costs
one sentence and is unblocked today; it does **not** need `sq-i50o4`.

**What this does not change.** All four PRs remain blocked on @jeswr's §6 author review,
and their gate figures remain fleet-box numbers pending the upstream `noir-gates-diff`
arbiter (§5.1). Nothing here re-opens any of the four PRs' *correctness* verdicts: §10.12
and §10.13 stand unchanged, including §10.12's still-open red-on-wrong-answer gap and
§10.13's signed-type coverage gap. Nothing here is a soundness certification (`sq-qhy4`).

Companion records: `noir-bounded-vec-capacity-assert-5027.md` (§6.3(a), the method),
`noir-single-limb-decomposition-13265.md` (§10.12), `noir-truncate-after-and-mask-8628.md`
(§10.13), `noir-optimization-program.md` (§5.1 measurement doctrine, §5.2 baseline corpus,
§10.1 the four drafts, §10.14 status pointer).
