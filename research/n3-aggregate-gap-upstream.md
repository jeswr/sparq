# Upstream w3c-cg/N3: no aggregate expression construct (verification + draft issue)

**Bead:** sq-6tykl.3.4 (from sq-6tykl.3 / issue #1993) · **Status:** gap **CONFIRMED**; issue drafted,
**NOT yet filed** upstream — awaiting @jeswr review per the upstream-contribution protocol
(`AGENTS.md` § *Upstream contributions*) · **Author:** SPARQ agent 🤖 [OPUS-5] · **Date:** 2026-07-27

Companion record: `gpt56-decomp-rules-substrate-2026-07.md` § 4 (the gap analysis this verifies).

## Why this record exists

Maintainer direction in issue #1993: N3 must be expressive enough to be a **superset** of RIF and the
RDFox rule syntax; any construct N3 cannot express → a clear, concise issue on the N3 CG repo stating
what is missing and that it was discovered implementing RIF/RDFox rules support. § 4 of the
decomposition record flagged exactly one candidate gap — RDFox's
`AGGREGATE { … } ON ?g BIND SUM(?v) AS ?x` — and cut this bead to verify it against the *current*
spec before anything is said upstream.

## Venue correction (found while verifying)

The CG has **renamed**: `w3c/N3` → **`w3c-cg/N3`**, and the spec host moved
`w3c.github.io/N3/spec/` → **`w3c-cg.github.io/N3/spec/`** (the old URL serves a `<meta refresh>`
redirect; the old GitHub org path resolves only through GitHub's rename redirect). The repo is live —
50 open issues, last push 2026-07-13. **Any upstream issue goes to `w3c-cg/N3`, not `w3c/N3`.**

## Verification (2026-07-27, against the live documents)

| document | version / status | `aggregat*` hits |
|---|---|---|
| [Notation3 Language](https://w3c-cg.github.io/N3/spec/) | W3C N3 Community Group draft | **0** |
| [Notation3 Builtin Functions](https://w3c-cg.github.io/n3Builtins/) | **Final Community Group Report, 3 June 2026** | **0** |

**Complete builtin inventory of the Final CG Report** (extracted from the published document):

- `log:` (22) — `collectAllIn`, `forAllIn`, `notIncludes`, `includes`, `semantics`, `conclusion`,
  `equalTo`, `uri`, `skolem`, …
- `math:` (25) — `sum`, `product`, `quotient`, `difference`, `remainder`, `rounded`,
  `absoluteValue`, `exponentiation`, `negation`, the comparisons, the trig family
- `list:` (9) — `append`, `first`, `last`, `in`, `iterate`, `length`, `member`, `memberAt`, `remove`
- `string:` (16), `time:` (7), `crypto:` (1)

### Confirmed: no aggregate expression construct

There is **no aggregate construct** in the grammar and **no aggregate builtin family**. The only
aggregation route is the list-valued findall idiom the gap analysis named:

```
( ?v { …clause… } ?list ) log:collectAllIn ?scope .
```

with the published schema `( $s.1- $s.2+ $s.3- )+ log:collectAllIn $o?` where `$s.2 : log:Formula`
and `$s.3 : rdf:List` — i.e. it is inherently **formula-valued and list-valued**.

### Two corrections to the prior gap analysis (both make the gap *sharper*)

1. **Grouping is NOT missing.** § 4 implied the idiom gives no equivalent of RDFox's `ON ?g`. It
   does: variables bound in the enclosing rule body act as the group key, so
   `AGGREGATE … ON ?hero` maps onto `?hero` being bound outside the `collectAllIn` triple (this is
   precisely the spec's own `:defeatedEnemies` example). This should not be claimed upstream.
2. **MIN/MAX are worse off than SUM/COUNT/AVG.** § 4 said the idiom "gives no direct
   SUM/MIN/MAX/AVG". More precisely, *through the list idiom*:

   | aggregate | reachable? | how |
   |---|---|---|
   | SUM | yes | `math:sum` — schema `( $s.i+ )+ math:sum $o-`, "the sum of the numbers given in the subject list" |
   | COUNT | yes | `list:length` |
   | AVG | yes, 3 steps | `math:sum` + `list:length` + `math:quotient` |
   | **MIN / MAX** | **no builtin at all** | there is **no `math:min`/`math:max`/`list:min`/`list:max`** in the registry — only hand-rolled recursion or a `list:member` + negation-as-failure idiom |

   So the honest claim is: *no aggregate expression construct at all*, **and** even the list
   workaround bottoms out for MIN/MAX.

### Prior art upstream — already surfaced once, closed unmerged, undiscussed

**[w3c-cg/N3 PR #119](https://github.com/w3c-cg/N3/pull/119) — "add aggregation operations"**
(@domel, opened 2023-01-16, **closed 2023-03-09 without merge, zero comments**). It added
`list:min`, `list:max`, `list:sum`, `list:avg`, `list:median`, `list:mode`, `list:variance`,
`list:stddev` to `ns/list.n3` (+64 lines, term status `unstable`). The diff also has Turtle syntax
errors — every term after `list:min` is missing its `a` (`list:max rdf:Property;`) — which may be
all that happened to it.

Full-text search of `w3c-cg/N3` issues **and** bodies for `aggregate` / `aggregation` / `list:sum` /
`list:avg` returns **only** PR #119. So: the request has been *made* upstream once, as a bare
vocabulary patch with no motivation and no discussion, and was dropped. **This is a dedupe-relevant
fact, and it shapes the draft below** — the issue is framed as reviving #119 *with* the motivation
and the use case it lacked, not as a fresh discovery.

### Why the list idiom does not close the gap for a compiled rule engine (the sparq-side point)

sparq's compiled/stratified N3 subset (`crates/sparq-reason/src/n3/compiled.rs`, module docs)
excludes, as a *loud compile error*: "list builtins/generators, `math:`/`time:` builtins, … formula-
or list-valued facts". The `collectAllIn` + `math:sum` idiom needs all three. sparq's full text
engine does implement `log:collectAllIn`/`log:forAllIn`
(`crates/sparq-reason/tests/n3_collect_stratified.rs`), so this is not a sparq capability gap — it is
that the *only* N3 spelling of an aggregate forces you out of the first-order, stratifiable fragment
that a compiled Datalog-style evaluator (and RDFox's own `AGGREGATE`) lives in.

## Draft issue — w3c-cg/N3 (NOT FILED; @jeswr review gate)

> **Title: No aggregate expression construct in N3 (reviving #119 with motivation): RDFox-style
> `AGGREGATE … ON … BIND SUM(?v) AS ?x` has no first-order N3 spelling**
>
> > 🤖 This issue was written by an autonomous agent (a SPARQ agent) operating on @jeswr's behalf,
> > and is posted with his review. It is a question/gap report, not a defect report.
>
> **Why this is being raised.** We are implementing a rules substrate that treats N3 as the common
> surface for a native Datalog dialect and for RIF, on the working assumption that N3 is a superset
> of both, plus of the RDFox rule syntax. Checking that assumption construct-by-construct, RIF-Core
> came out fully expressible (frames/membership/subclass lower to triples; the RIF builtins lower to
> `math:`/`string:`/`list:`), and RDFox's negation maps onto `log:notIncludes`. Exactly one construct
> did not map: **aggregation**.
>
> **The construct.** RDFox (and SPARQL, and RIF-BLD's usual extensions) have an aggregate
> *expression* that binds a scalar computed over a group:
>
> ```
> [?dept, :headcount, ?n] :- AGGREGATE( [?p, :worksIn, ?dept] ON ?dept BIND COUNT(?p) AS ?n ) .
> [?dept, :payroll,   ?t] :- AGGREGATE( [?p, :worksIn, ?dept], [?p, :salary, ?s] ON ?dept BIND SUM(?s) AS ?t ) .
> [?dept, :topSalary, ?m] :- AGGREGATE( [?p, :worksIn, ?dept], [?p, :salary, ?s] ON ?dept BIND MAX(?s) AS ?m ) .
> ```
>
> **What N3 has today.** As far as we can tell from the [Notation3 Language
> spec](https://w3c-cg.github.io/N3/spec/) and the [Notation3 Builtin Functions Final Community Group
> Report (3 June 2026)](https://w3c-cg.github.io/n3Builtins/), neither document contains the word
> "aggregate", and there is no aggregate builtin family. The only available idiom is the list-valued
> findall:
>
> ```n3
> { ?dept a :Dept .
>   ( ?s { ?p :worksIn ?dept . ?p :salary ?s } ?salaries ) log:collectAllIn _:t .
>   ?salaries math:sum ?total .
>   ?salaries list:length ?n .
> } => { ?dept :payroll ?total ; :headcount ?n } .
> ```
>
> To be clear about what *does* work, so this is not overstated: **grouping is fine** — `?dept`,
> bound in the enclosing body, is the group key, so `ON ?dept` is covered. **SUM** (`math:sum` over
> the collected list) and **COUNT** (`list:length`) are reachable, and **AVG** is reachable in three
> steps (`math:sum` + `list:length` + `math:quotient`).
>
> **Two things we could not do.**
>
> 1. **MIN and MAX have no builtin at all.** The registry has no `math:min`/`math:max` and no
>    `list:min`/`list:max`, so the `MAX(?s)` rule above has no direct spelling — it needs hand-rolled
>    recursion, or a `list:member` + `log:notIncludes` "no member is greater" encoding. That
>    asymmetry (SUM/COUNT yes, MIN/MAX no) looks more like an omission than a design decision, which
>    is really the narrow question here.
> 2. **Aggregation is only available as a formula-and-list-valued construct.** `log:collectAllIn`
>    takes a `log:Formula` and yields an `rdf:List`. For a full N3 engine that is no obstacle. But an
>    implementation that compiles a stratifiable, first-order subset of N3 down to a Datalog-style
>    evaluator (which is the point of the exercise for us, and is the fragment RDFox's `AGGREGATE`
>    itself inhabits) typically excludes formula-valued and list-valued terms from that subset
>    precisely to keep it first-order. Under that restriction aggregation becomes inexpressible —
>    not because the *capability* is missing from N3, but because its only spelling is a
>    higher-order one.
>
> **Prior art.** [#119](https://github.com/w3c-cg/N3/pull/119) proposed `list:min`/`max`/`sum`/`avg`/
> `median`/`mode`/`variance`/`stddev` in January 2023 and was closed unmerged with no discussion. We
> suspect it lapsed because it was a bare vocabulary patch with no stated use case (and the Turtle in
> it is malformed — every term after `list:min` is missing its `a`). This issue is an attempt to
> supply the missing motivation rather than to open a new front.
>
> **Questions for the CG.**
>
> 1. Is the absence of `math:min`/`math:max` (or `list:min`/`list:max`) intentional, or simply an
>    omission that #119 tried and failed to fix? If the latter, would a corrected version of #119 —
>    at minimum `min`/`max`, matching `math:sum`'s existing list-valued schema — be welcome?
> 2. Is `log:collectAllIn` + list builtins considered *the* intended and sufficient way to aggregate
>    in N3, with no aggregate expression construct planned? A sentence to that effect in the spec
>    would settle it for implementers either way.
> 3. Has the CG considered a first-order aggregate construct (a term-level `?x` bound to
>    `SUM/MIN/MAX/AVG/COUNT` over a group, without routing through a formula and a list)? We are not
>    proposing syntax — we are asking whether the higher-order-only status is deliberate, since it is
>    the one place where an N3 profile restricted to a compiled first-order fragment stops being a
>    superset of RDFox's rule language.

## Notes for @jeswr (review gate — nothing filed)

- **Venue is `w3c-cg/N3`** (renamed from `w3c/N3`), issue tracker open, 50 open issues.
- The draft deliberately **concedes** grouping, SUM, COUNT and AVG rather than claiming a blanket
  "N3 cannot aggregate" — the verified gap is narrower than § 4 assumed (MIN/MAX absent + the
  higher-order-only spelling), and overstating it would be wrong and would read badly upstream.
- It does **not** claim sparq is correct or that N3 should change; questions 1–3 are questions.
- Per `AGENTS.md` § *Upstream contributions*, if this is filed it carries the 🤖 agent self-id (in
  the draft above) and should not be marked ready for CG action without your say-so.
- Follow-up beads once upstream answers: (a) if `min`/`max` land, extend the RIF/N3 builtin lowering
  table; (b) if the CG confirms the list idiom is the intended route, record that in
  `gpt56-decomp-rules-substrate-2026-07.md` § 4 and drop the "N3 is not a superset" caveat to a
  documented, deliberate profile restriction instead.

## Verification method (reproducible)

Live documents fetched 2026-07-27: `https://w3c-cg.github.io/N3/spec/` and
`https://w3c-cg.github.io/n3Builtins/` (tags stripped, case-insensitive term search); builtin
inventory extracted by namespace-prefix scan of the published Final CG Report; upstream prior art via
the GitHub search API over `repo:w3c-cg/N3` titles **and** bodies for `aggregate`, `aggregation`,
`list:sum`, `list:avg`; PR #119 state, diff and comment count via the REST API.
