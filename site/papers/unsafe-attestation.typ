// [OPUS-4.8] sq-gum8 — Paper A5: a confined, registered, CI-ratcheted `unsafe`-Rust surface as
// a machine-checkable memory-safety ATTESTATION for an RDF engine.
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
// Framed HONESTLY: the evidence is a set of DETERMINISTIC, committed, CI-enforced COUNTS + a
// per-site register + a coverage matrix. It is a COVERAGE / DISCIPLINE contribution — emphatically
// NOT a proof of memory safety and NOT a performance result. The known-open soundness beads on the
// untrusted-input mmap boundary (B5) are stated as limitations, not hidden.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "A Confined, Registered, CI-Ratcheted unsafe Surface as a Memory-Safety Attestation for an RDF Engine")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Auditing the `unsafe`: A Confined, Registered, CI-Ratcheted `unsafe`-Rust Surface
    as a Machine-Checkable Memory-Safety Attestation for an RDF Engine
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A resources / security-engineering contribution. The evidence is a set of deterministic,
  committed, CI-enforced integers — an `unsafe`-site count ratchet, a crate-level confinement
  partition, and a per-site justification register — plus the coverage matrix that bounds each
  site. It is a _coverage and discipline_ contribution: it bounds and documents the `unsafe`
  surface and proves it cannot silently grow. It is _not_ a proof that the engine is free of
  undefined behaviour, _not_ a performance result, and the known-open soundness gaps on the
  untrusted-input boundary are stated as limitations.
]]

== Abstract

A Rust system earns its memory-safety reputation only over the code it writes `unsafe` — the
escape hatch where the compiler's guarantees are suspended and a human invariant takes their
place. RDF engines need that hatch: zero-copy dictionaries, `mmap`-backed on-disk indexes,
parallel scatter builds, and SIMD prefetch are where the bytes meet the metal. We describe how
one such engine makes its `unsafe` surface _machine-checkable_ rather than a matter of trust. The
surface is _confined_: of #headline("memsafety.total_crates") first-party crates,
#headline("memsafety.forbid_crates") carry a crate-root `#![forbid(unsafe_code)]` that makes any
`unsafe` a compile error, leaving exactly #headline("memsafety.unsafe_crates") crates that may
contain it. The surface is _counted and ratcheted_: a deterministic source scan fixes the total
at #headline("memsafety.unsafe_sites_total") sites against a committed snapshot, and a required CI
lane fails any change that raises the count without a matching register row, a `// SAFETY:`
comment, and a re-seed. The surface is _registered_: every site is enumerated with the invariant
it relies on, why that invariant holds, and the test or sanitizer that bounds it. We are explicit
that this is a _coverage_ guarantee, not a proof of soundness: the ratchet proves the surface cannot
grow unaccounted-for, the register proves each site is justified, and a coverage matrix
(`miri` / sanitizer / fuzz / deterministic corruption oracle) bounds each class — but none of
these is a proof of the absence of undefined behaviour, and we name the open soundness gaps on the
untrusted-input boundary rather than paper over them.

== Contributions

This paper substantiates the following claims; each is refutable and forward-references its
evidence. None is a performance claim, and _none asserts that the engine is free of undefined
behaviour_.

- *The `unsafe` surface is confined to a named, compile-checked minority of crates* (§3) — of
  #headline("memsafety.total_crates") first-party crates, #headline("memsafety.forbid_crates")
  forbid `unsafe` at the crate root, so a new `unsafe` outside the remaining
  #headline("memsafety.unsafe_crates") crates fails to compile; the partition is exact and
  accounted-for.
- *The surface is counted and held by a CI ratchet* (§4) — a deterministic scan fixes the
  first-party `unsafe` total at #headline("memsafety.unsafe_sites_total") sites against a
  committed snapshot, and a _required_ CI lane fails any change that raises the count without the
  paired register + `// SAFETY:` + re-seed; the count may only fall freely.
- *Every site is justified in a per-site register, lint-pinned in source* (§5) — each of the
  #headline("memsafety.unsafe_sites_total") sites carries a register row (invariant, why-sound,
  how-bounded) and a literal `// SAFETY:` comment that a clippy lint mechanically requires.
- *Each site class is bounded by a coverage matrix* (§6) — pure-Rust `unsafe` runs under `miri`;
  the file-backed `mmap` sites Miri cannot interpret are bounded by a deterministic corruption
  oracle, a fuzz target, and AddressSanitizer.
- *Honest limitation: coverage, not soundness* (§7) — we make no proof of the absence of UB, and
  we name the open soundness beads on the untrusted-input `mmap` boundary.

== The surface is confined <confine>

Memory safety in Rust is a property of the `unsafe` you write, not of the language in the
abstract: inside an `unsafe` block the borrow checker's guarantees are suspended and the
programmer must uphold an invariant by hand. The first discipline, therefore, is to make `unsafe`
the exception and to _mechanically confine_ where it may appear.

The engine partitions its #headline("memsafety.total_crates") first-party crates exactly:
#headline("memsafety.forbid_crates") of them carry a crate-root `#![forbid(unsafe_code)]`, which
is not a convention but a _compile error_ — an `unsafe` block, `impl`, or `extern` anywhere in
such a crate fails to build. That leaves exactly #headline("memsafety.unsafe_crates") crates that
are permitted to contain `unsafe`, and the accounting closes:
#headline("memsafety.forbid_crates") forbid + #headline("memsafety.unsafe_crates") unsafe-bearing
= #headline("memsafety.total_crates") total, with no crate unaccounted-for.

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header[Partition][Crates],
    [Total first-party crates], [#headline("memsafety.total_crates")],
    [`#![forbid(unsafe_code)]` (a new `unsafe` is a compile error)],
    [#headline("memsafety.forbid_crates")],
    [Permitted to contain `unsafe` (the confined surface)],
    [#headline("memsafety.unsafe_crates")],
  ),
  caption: [
    The crate-level confinement partition. Of #headline("memsafety.total_crates") first-party
    crates, #headline("memsafety.forbid_crates") forbid `unsafe` at the crate root and only
    #headline("memsafety.unsafe_crates") may contain it. Deterministic and machine-independent (a
    de-duplicated source grep over committed code), not a timing.
  ],
)

#provenance("memsafety.forbid_crates")

The confinement is load-bearing for what _is not_ in the surface. The query parser, the SPARQL
planner and streaming executor, the RDFS / OWL-RL / N3 reasoner, and the SHACL validator are all
in the forbid set — so the path from _untrusted query or data text_ to evaluation never touches
`unsafe`. Only the on-disk index loader does, which is precisely the boundary we examine next.

== The surface is counted, and a ratchet holds it <ratchet>

A confinement says _where_ `unsafe` may live; it does not say _how much_ lives there, nor stop the
amount from drifting upward release by release. The second discipline is a count with a one-way
gate.

A deterministic scan — comment- and string-aware, so a `// SAFETY:` argument or the word
`unsafe` inside a string is not miscounted — fixes the first-party `unsafe` total at
#headline("memsafety.unsafe_sites_total") sites, recorded per crate in a committed snapshot.

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header[Surface metric][Count],
    [First-party `unsafe` sites (workspace total)],
    [#headline("memsafety.unsafe_sites_total")],
    […of which in the storage / `mmap` / dictionary core crate],
    [#headline("memsafety.unsafe_sites_core")],
  ),
  caption: [
    The `unsafe`-site count, a CI-enforced ratchet _ceiling_. The total is
    #headline("memsafety.unsafe_sites_total") first-party sites, the majority
    (#headline("memsafety.unsafe_sites_core")) concentrated in the single storage core crate. A
    deterministic source scan, machine-independent, not a timing; the count may only _fall_ freely
    and may not _rise_ without a paired register update.
  ],
)

#provenance("memsafety.unsafe_sites_total")

The scan's output is checked against the committed snapshot by a CI lane on every pull request and
merge ref. The lane is _required_: its name carries no "informational" or "advisory" token, so the
gate aggregator treats it as blocking, distinct from a sibling visibility-only `cargo-geiger`
report that cannot even run the workspace's virtual manifest. The asymmetry is deliberate —
_smaller is always allowed_ (removing `unsafe` never needs ceremony), but a change that _raises_ a
crate's count fails CI until three artefacts land in the _same_ reviewable diff: a row in the
per-site register, a `// SAFETY:` comment in source, and a re-seed of the snapshot. The count is
therefore a one-way ratchet on the engineering effort, not a number a contributor can quietly
nudge.

== Every site is justified, lint-pinned <register>

A count bounds the surface; it does not argue that any individual site is sound. The third
discipline is a _per-site register_: each of the #headline("memsafety.unsafe_sites_total") sites
is enumerated with its file and line, its kind (a slice/pointer reinterpret, an `mmap` map, an FFI
call, an `unsafe impl Send`/`Sync`, a `set_len`), the invariant it relies on, why that invariant
holds, and how it is bounded by a test or sanitizer. The register distinguishes two trust classes,
because they carry very different risk:

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header[Trust class][What it is, and why it is or is not dangerous],
    [Trusted-input `unsafe`],
    [Pointer / slice / byte reinterprets whose backing this process just produced (a `Vec` we
     built, a file we wrote and own for the call). Soundness rests on plain-old-data + alignment +
     length invariants that hold _by construction_.],
    [Untrusted-input `unsafe` (boundary B5)],
    [The `mmap` loaders that map a `.spq` / `dict-*.bin` / vector / DiskANN file that may be
     _hostile or corrupt_, reinterpreting its bytes through offsets read out of that same untrusted
     file. The most dangerous class: an _untrusted-input → unsafe-code_ boundary. These are
     validated at open before any unchecked access, and the hot read path uses bounds- and
     UTF-8-_checked_ accessors.],
  ),
  caption: [
    The two trust classes the per-site register distinguishes. The untrusted-input class (the
    on-disk index loader, threat-model boundary B5) is the one that carries a memory-safety —
    rather than a merely wrong-answer or denial-of-service — risk, so the register and the coverage
    matrix lead with it.
  ],
)

Each register row's `// SAFETY:` argument also lives _in the source_, immediately above the
`unsafe`, and that is not left to reviewer diligence either: a clippy lint
(`undocumented_unsafe_blocks`), promoted to a hard error by the workspace's
`clippy -D warnings` gate, fails the build on any `unsafe` block / `impl` / `extern` that lacks a
preceding `// SAFETY:` comment. So a new site cannot land without _both_ the in-source argument and
the register row — the lint enforces the former at compile time, the ratchet enforces the latter at
review time.

== A coverage matrix bounds each class <coverage>

A justification is an argument; a coverage mechanism is a test that tries to break it. The fourth
discipline maps each `unsafe` class to a bounding mechanism, and is honest about what each
mechanism can and cannot reach.

#figure(
  table(
    columns: 2,
    align: (left, left),
    table.header[Mechanism][What it bounds — and its honest reach],
    [`miri` (nightly, Tree-Borrows)],
    [The pure-Rust `unsafe` reachable without a file-backed map: the parallel scatter writes, the
     plain-old-data ↔ bytes reinterprets, `from_utf8_unchecked` over in-memory buffers,
     `MaybeUninit` + `set_len`. Interprets every operation for aliasing / provenance / data-race
     UB. _Cannot_ run the `mmap` sites — Miri rejects file-backed mappings — so those are bounded
     below instead.],
    [Deterministic corruption oracle + `mmap`-loader fuzz target],
    [The B5 `mmap` sites Miri structurally cannot interpret: a sweep that truncates and bit-flips
     every on-disk section and asserts the loader rejects-or-stays-in-bounds, plus a fuzz target
     over a corrupt on-disk store. The mechanism Miri's gap is closed by.],
    [AddressSanitizer over the corruption corpus],
    [The same corruption corpus run under ASan (with an instrumented `std`), so a heap
     out-of-bounds / use-after-free / UB on a malformed on-disk file is caught even when it does
     not panic.],
  ),
  caption: [
    The coverage matrix: each `unsafe` class mapped to a bounding mechanism, with the honest reach
    of each stated. Miri covers the in-memory `unsafe`; the corruption oracle, fuzz target, and
    ASan cover the file-backed `mmap` boundary Miri cannot interpret.
  ],
)

The matrix is _layered on purpose_: no single mechanism reaches every site, so the loader's
file-backed sites — exactly the untrusted-input boundary — are covered by three independent
methods (a deterministic oracle, a fuzzer, and a sanitizer) precisely because they are the highest
risk and the one place Miri is blind.

== Honest limitation: coverage, not soundness <limits>

We state plainly what this attestation is and is not. It is a _coverage and discipline_
contribution: the partition confines `unsafe` to #headline("memsafety.unsafe_crates") of
#headline("memsafety.total_crates") crates and proves a new site cannot appear elsewhere; the
ratchet fixes the total at #headline("memsafety.unsafe_sites_total") sites and proves the surface
cannot grow unaccounted-for; the register justifies every site and the lint pins each argument in
source; the coverage matrix bounds each class. Each of these is a _checkable, machine-enforced
property_.

None of them is a _proof that the engine is free of undefined behaviour_. A ratchet that holds a
count constant does not prove the held sites are sound; a `// SAFETY:` argument is a human
argument, not a machine-checked theorem; Miri, fuzzing, and ASan are _coverage_, not exhaustive
verification — they find UB they reach, they do not prove its absence. And we name the live gaps
rather than imply none exist: the untrusted-input `mmap` boundary (B5) carries documented,
tracked soundness work — open issues on a non-UTF-8 dictionary fast path, on compressed-permutation
header arithmetic, and on the breadth of negative-input fuzzing of the loader — that the threat
model records as open, not closed. The attestation's value is that these are _enumerated and
gated_, not that they are absent: an auditor is handed the exact surface, a justification per site,
a coverage mechanism per class, and a one-way gate against growth — which is a far stronger
starting point than a prose assurance, and an honestly weaker thing than a proof of soundness.

This is also why no number here is a performance figure. Every metric is a deterministic integer
over committed source — a crate partition, a site count — reproducible on any host. The same
honesty rule that keeps wall-clock numbers out of this paper factory's headline tables is what
lets these counts _be_ headline evidence.

== Related work and honest positioning

Confining `unsafe` with `#![forbid(unsafe_code)]`, documenting sites with `// SAFETY:` comments,
running Miri, and fuzzing parsers are each established Rust practice, and we claim no novelty in any
one of them. The contribution is the _composition_, made machine-checkable for an RDF engine: a
crate-level confinement partition, a per-site justification register at 100% coverage, a _required_
count ratchet that gates growth, a clippy lint that pins each argument in source, and a layered
coverage matrix that is explicit about Miri's blind spot on file-backed maps — assembled into a
single attestation an auditor can re-run, and presented with an honest "coverage, not soundness"
boundary and a named list of open gaps. The methodology is security-engineering discipline rather
than a research result; it is offered as the machine-checkable _evidence_ behind a memory-safety
posture, and as a template another systems crate could adopt.

== Conclusion

An RDF engine that maps untrusted on-disk indexes into typed memory cannot avoid `unsafe`, but it
can refuse to leave that `unsafe` to trust. This one confines it to
#headline("memsafety.unsafe_crates") of #headline("memsafety.total_crates") crates by compile-time
forbiddance, counts it at #headline("memsafety.unsafe_sites_total") sites behind a required CI
ratchet that gates any growth, justifies every site in a lint-pinned register, and bounds each
class with a layered Miri / corruption-oracle / fuzz / sanitizer matrix. We make no performance
claim and — this is the load-bearing honesty — no claim that the engine is free of undefined
behaviour: the attestation is a coverage and discipline guarantee with its open soundness gaps
named, not a proof of soundness. Its worth is that the `unsafe` surface is made small, enumerated,
justified, covered, and gated against growth — auditable by construction rather than by assurance.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · evidence traces to `bench/unsafe-snapshot.json` and its ratchet
    `scripts/unsafe-gate.py`, the per-site register `compliance/memsafety/unsafe-register.md`, the
    confinement accounting `compliance/memsafety/evidence.md`, the threat-model boundary B5 in
    `research/threat-model.md`, and the `miri` / fuzz / ASan lanes under `.github/workflows/`.
    Numbers in this document are injected at build time from the paper-bound evidence file; see the
    provenance stamp on the published page.
  ]
]
