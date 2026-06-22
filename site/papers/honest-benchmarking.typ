// [OPUS-4.8] sq-gum8 — Pilot paper A2: Honest same-box benchmarking methodology.
// A METHODS contribution. It needs no benchmark to describe; its evidence is the
// methodology + the environment discipline (canonical vs indicative) that this very
// paper factory enforces. No performance result is claimed.

#import "_lib/bench.typ": ev, headline, authors, anon

#set document(title: "Honest Same-Box Benchmarking for RDF Engines")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Honest Same-Box Benchmarking for RDF Engines: Differential-Correctness-Gated,
    Hardware-Labelled, Negative-Results-Inclusive
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A methodology / reproducibility contribution — not a performance result. It describes a
  benchmarking discipline and the build-time gate that enforces it; it reports no wall-clock
  number as evidence.
]]

== Abstract

RDF-engine performance claims are routinely undermined by three avoidable faults:
cross-machine number mixing, untrusted result correctness, and silently-dropped negative
results. We describe a same-box benchmarking methodology that gates every reported timing on
a differential-correctness check, labels every number with its full hardware/software
environment, separates _canonical_ (deterministic, machine-independent) evidence from
_indicative_ (development work-box) measurement, and keeps a negative-results ledger. We then
show how the discipline is made _mechanical_: a build-time honesty gate refuses to publish an
indicative number as a headline result. This is a methods contribution; it strengthens the
evaluation section of every performance paper it governs.

== Contributions

This paper substantiates the following claims; each forward-references the section that
delivers it.

- *Correctness gates timing* (§3) — no engine's timing for a query is trusted unless its
  result _count_ matches the cross-checked answer on the same input. A faster wrong answer is
  not a result.
- *The canonical/indicative boundary is enforced, not advised* (§4) — every number carries an
  `environment` label; a build-time gate _fails the build_ if an `environment = indicative`
  number is cited as a headline result, and the two tiers are never co-tabulated.
- *Same-box, back-to-back, fully-labelled* (§3) — all engines run on one idle machine in one
  session, with the complete platform spec (CPU/µarch, cores, clock policy, kernel, rustc,
  dataset name+version+hash+seed) recorded alongside every number.
- *Negative results are first-class* (§5) — measured rejections and "did-not-help" findings
  are recorded and citable, not discarded.

== The methodology <method>

The contribution is _methodological rigour_, not a new benchmark suite — established suites
(WatDiv, SP2Bench, WDBench, LUBM, BSBM) are used as-is. The discipline has four rules.

/ Same box, back-to-back: every engine under comparison runs on one idle machine within one
  gather session. Absolute microseconds across host classes are not directly comparable, so a
  comparison is only meaningful when the competitor number and the sparq number it is divided
  against were measured on the _same_ box.

/ Correctness before timing: each engine's result is cross-checked (result count, and where
  feasible the result set, against a reference engine) before its timing is admitted. A
  cross-engine speedup is computed _only_ from rows where the answers agree; a row whose
  counts disagree contributes no ratio.

/ Full environment labelling: every number is stamped with its CPU and micro-architecture,
  core count, clock/turbo policy, cache levels, RAM, kernel release, storage, the `rustc`
  version and `Cargo.lock`, and the dataset's name, version, SHA-256, and seed. Ratios are
  never reported without the absolutes behind them.

/ No hard-coded performance in source control: numbers live in a generated data artefact
  refreshed from a benchmark-data branch, never pasted into prose, so a stale figure cannot
  drift out of sync with its provenance.

== The honesty gate: canonical vs indicative <gate>

The load-bearing mechanism is the distinction between two tiers of evidence, made
_mechanical_ at the data layer:

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header[Tier][Definition and treatment],
    [`canonical`],
    [Deterministic, machine-independent metrics: conformance counts, byte-identity
     invariants, recall floors, gate/round/byte counts, differential-fuzz pass. These are
     reproducible anywhere and may back a headline claim.],
    [`indicative`],
    [Wall-clock / memory measured on a development work-box (here, a virtualised AWS host).
     Useful for development ordering; *never* a published headline, and never co-tabulated
     with a canonical number or folded into an aggregate.],
  ),
  caption: [
    The two evidence tiers. The boundary is enforced by a build-time gate.
  ],
)

The gate is not a guideline a reviewer must remember to apply; it is code. Each number a
paper may cite as a headline carries an `environment` field, and the paper accesses it
through a `headline()` helper that *refuses to render* — failing the build — any record whose
`environment` is not `canonical`. An indicative number can therefore only ever appear in an
explicitly-labelled "indicative development measurement" callout, never in a result table or
figure. The companion paper in this factory (the RDF-native filtered-ANN paper) is a live
demonstration: it relies entirely on canonical recall floors and answer-safety invariants,
and the canonical performance runner being unavailable, it makes _no_ wall-clock claim — the
gate would block one if it tried.

== Negative results as evidence <negatives>

A measured rejection is a contribution. The methodology keeps a negative-results ledger:
techniques that were implemented and measured _not_ to help (or to help only under conditions
worth stating) are recorded with their measurement context and kept citable, rather than
quietly dropped. This both protects the honesty of the positive claims and saves later effort
re-investigating settled questions.

== Status and honest limitation

This is a methodology description, and the canonical lane it specifies is _designed, not yet
executed_: the bare-metal, frequency-pinned canonical runner is blocked on an
infrastructure step. Until it lands, the discipline still binds — papers in this factory
publish only the deterministic (`canonical`) metrics that are reproducible today, and the
gate enforces exactly that. We state this limitation plainly rather than presenting the
canonical lane as operational.

== Conclusion

Honest same-box benchmarking for RDF engines is achievable by gating timing on correctness,
labelling every number with its full environment, separating canonical from indicative
evidence, and recording negative results — and the canonical/indicative boundary can be
enforced mechanically by a build-time gate rather than left to reviewer diligence. The
contribution is the discipline and its enforcement; it reports no performance number as a
result, by design.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · this paper is a methodology contribution and intentionally cites no
    wall-clock number; the canonical performance runner is not yet operational.
  ]
]
