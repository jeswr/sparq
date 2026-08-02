<!-- [SONNET-4.6] sq-gg0qq.6 (issue #2746): the jeswr/lws-spec conformance vectors wired as a
     data-driven suite over the spec's own executable N3 access-decision semantics. -->

# `lws-spec` conformance — vendored fixtures

[`jeswr/lws-spec`](https://github.com/jeswr/lws-spec) is the **contract** for the JLWS (Linked Web
Storage) profile: per-document-class SHACL shapes, a normative *executable* N3 access-decision rule
set, and a language-neutral test-vector corpus. Where this crate and that spec disagree, **the spec
is right and the code is the bug** — the same discipline `research/lws-design-records.md` §8
records for the DPoP-SK spec.

Everything under this directory is **copied verbatim** from
`jeswr/lws-spec@ffaea0497de41cd709a742e0c4a90831a500fd97` (`main`, 2026-07-12). Do not edit it;
re-vendor instead (see below). The harness that reads it is
[`../../tests/lws_spec_access_decision.rs`](../../tests/lws_spec_access_decision.rs).

## Vendored, not a submodule — the decision

The bead left this open ("vendored fixtures or pinned submodule — decide + document"). **Vendored**,
because:

- The gate must run on the ordinary `cargo test -p sparq-lws-core` lane, on a runner with no network
  and no `git submodule update`. A submodule makes the crate's own test suite conditional on an
  external fetch; sparq's CI-economy posture is that a gate that can be skipped is not a gate.
- The corpus is small (this slice is ~67 KB of JSON/N3) and changes at spec cadence, not at build
  cadence. A pinned submodule would give the same immutability at a higher operational cost.
- The pinned revision is a single constant (`VENDORED_REV` in the harness) and the fixtures are
  plain data, so a re-vendor is a reviewable diff rather than a pointer bump nobody can read.

## What is wired, and what is not

The corpus at the pinned revision is **157 vectors across 10 suites**. This directory vendors
**one** of them; the harness asserts the corpus-level arithmetic from the vendored top-level
manifest, so the denominator cannot drift unnoticed.

| Suite | Cases | Wired here |
|---|---:|---|
| `access-grants` | 24 | **19** (`evaluate-access`) — executed against the N3 oracle below |
| `resources` | 17 | no |
| `containers` | 14 | no |
| `metadata` | 7 | no |
| `discovery` | 8 | no |
| `auth` | 36 | no |
| `dpop-sk` | 14 | no |
| `notifications` | 12 | no |
| `rdf-transform` | 21 | no |
| `errors` | 4 | no |

The remaining 138 vectors are **not** wired and are not claimed. 74 of the corpus's cases are
`http-exchange` (counted over the full upstream corpus at the pinned revision — the slice vendored
here does not contain them, so that figure is not re-derivable from this directory) — a full-stack
request/response against a JLWS server, with the vector's own resource state, keyring and
evaluation instant — and this crate implements the **Solid/LDP**
surface, not the JLWS profile. Standing up that binding is profile code that belongs in a
`sparq-lws` crate over this one, which does not exist yet (its creation is the phase-5 item of
`research/lws-3-crate-split.md`, blocked on #2747). The five `validate-access-document` cases in
the wired suite need the SHACL shapes under `shapes/`, which are likewise not vendored yet.

`evaluate-access` was chosen first because it is the one family whose **oracle is already in the
tree**: the decision function is defined as N3, and sparq has an N3 reasoner.

## The oracle — the spec's N3, run by sparq

`semantics/access-decision.n3` **is** the definition of the strict ODRL access profile's
`evaluate-access` decision function; `semantics/access-decision.query.n3` is its
decision-extraction query. The spec runs them under EYE (`eyereasoner … --query …`). The harness
runs the same file under `sparq-reason`'s N3 engine, in-process, so:

- there is **no node/EYE opt-in CI lane** to build or keep alive — the gate is an ordinary
  `cargo test`, green on the standard workspace lane and on `--no-default-features`;
- the reasoner under test is sparq's own, which means the gate doubles as a check of sparq's N3
  engine against an externally-authored, adversarially-written rule set.

`permit` ⟺ at least one `ax:permittedBy` justification is derived; `deny` ⟺ none is (the
decision-time closed-world absence — default deny).

### Stratification is load-bearing

The rule set is stratified and says so in its own header (`K -> M -> N, O -> D`), and its only
negation is `log:collectAllIn` over predicates an earlier stratum derives. **A single-pass closure
is not a sound driver for it.** Measured on the vendored file: run in one pass, rules N/O
(prohibition / obligation matching) and rule D (the permit) become eligible in the same fixpoint
round, and `access-grants/prohibition-denies-despite-permission` derives `ax:prohibitedIn` *and*
`ax:permittedBy` together — a matching prohibition fails **open**. Two of the nineteen vectors
(`prohibition-denies-despite-permission`, `unmet-obligation-fail-closed`) detect this.

So the harness cuts the vendored rule set at its own section banners and feeds the pieces to
`sparq_reason::n3::reason_n3_stratified` in the declared stratum order. The cut leaves the vendored
bytes untouched, and a dedicated test asserts the split still describes the file (the permit
predicate is derived only in the last stratum; the predicates it negates only in earlier ones), so
a re-vendor that reshapes the sections fails loudly instead of silently mis-evaluating.

### Input encoding

`input.grants[]` + `input.request` are JSON; the reasoner needs N3. The harness implements the
mapping table in the spec's `semantics/README.md` § *Input encoding*, and its security invariant:
triples are emitted **only** for the fields that table names, so the profile facts
(`ax:KnownAction` membership, the `odrl:includedIn` lattice) are never readable from an evaluated
document and a hostile grant cannot inject widening. Anything unrecognised — action, left operand,
operator, context key, target class, record `@type` — is an encode **error**, never a silent drop;
those edges carry their own unit tests.

## Honest coverage limits

Passing these vectors is **necessary, not sufficient**. Beyond the 138 unwired vectors:

- The corpus deliberately leaves normative statements unpinned — the upstream `GAPS.md` enumerates
  them (network/trust, stateful/temporal, behavioural-emission classes a request/response vector
  cannot observe).
- The 19 wired vectors do not cover every fail-closed property the rule set implements. Measured:
  deleting the trailing-slash segment-safety guard from rule C3 (the guard that stops
  `…/notes/` covering `…/notes-evil.txt`) leaves all 19 green. Upstream keeps those adversarial
  probes in its own `test-suite/`, outside the vector corpus, and they are not vendored here.
- No claim is made that `sparq-lws-core`'s **own** WAC/ACP authorization path implements this
  profile. It does not; the profile is a separate decision function, and what is gated here is that
  the spec's semantics evaluate correctly on sparq's reasoner.

## Running it

```sh
cargo test -p sparq-lws-core --test lws_spec_access_decision
```

A failure names the suite and the vector: `suite=access-grants vector=access-grants/<id> — expected
permit, oracle derived deny (<the vector's title>)`.

## Re-vendoring

1. `git clone https://github.com/jeswr/lws-spec` and check out the new revision.
2. Copy `semantics/access-decision*.n3`, `test-vectors/manifest.json`, and
   `test-vectors/vectors/access-grants/` over the trees here, verbatim.
3. Move `VENDORED_REV` — and, if the corpus grew, the `CORPUS_*` / `*_CASES` constants — in
   `tests/lws_spec_access_decision.rs`. The manifest tests fail until they agree.
4. If a verdict changes, **the spec wins**: fix this crate, or open a change proposal on `lws-spec`
   for review. Do not edit a vendored fixture to make a test pass.
