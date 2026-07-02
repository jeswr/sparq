// [OPUS-4.8] sq-gum8 — Paper C3: the coZK soundness RE-AUDIT as a witness-validation
// NEGATIVE RESULT (security-engineering lessons). Source: research/mpc-cozk-reaudit.md
// (bead sq-9hrn), the adversarial re-audit of the collaborative-proof path against eprint
// 2025/1026, yielding R-WV — a witness-validation-before-proving test obligation encoded as
// an enforceable build-time gate.
//
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
//
// FRAMED HONESTLY as a NEGATIVE-RESULT / security-engineering-lessons contribution. This is
// emphatically NOT a security / soundness / privacy / attestation proof: the collaborative path
// is UNBUILT (crates/sparq-mpc/src/proof.rs is all NotYetImplemented), so the re-audit CANNOT
// certify soundness. EVERY soundness-adjacent sentence is written NEGATED ("not yet sound",
// "cannot certify", "must validate", "remains open") so it clears the privacy-claims gate from
// the start. The gates cited are bead sq-qhy4 (external single-prover audit, OPEN) and bead
// sq-9hrn (this coZK re-audit). The concept the literature calls the hyphenated adjective is
// referred to here as "malicious security" (the noun) on purpose.

// [OPUS-4.8] sq-iixdh — import paper_heading_numbering so the Abstract is un-numbered and
// sections render as "1.", "2." (not "0.1", "0.2").
#import "_lib/bench.typ": headline, ev, provenance, authors, anon, paper_heading_numbering

#set document(title: "A Collaborative-zk-SNARK Re-Audit as a Witness-Validation Negative Result for Federated SPARQL")
#set text(size: 11pt)
#set par(justify: true)
// Section numbering switched on here; the Abstract below is explicitly un-numbered so it
// renders as front matter (venue convention), and == sections number as "1.", "2.", ...
#set heading(numbering: paper_heading_numbering)

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Cannot Certify, So We Encode the Obligation:
    A Collaborative-zk-SNARK Re-Audit as a Witness-Validation Negative Result for Federated SPARQL
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A negative-result / security-engineering contribution. It is _not_ a security, soundness,
  privacy, or attestation proof: the collaborative (multi-prover) proof path it audits is
  *unbuilt* — every method on the boundary returns a fail-closed `NotYetImplemented` — so the
  re-audit *cannot certify soundness* and makes no such claim. The contribution is the adversarial
  re-audit methodology, the honest finding that the path is *not yet* sound to claim, and a
  *witness-validation-before-proving* test obligation (R-WV) that converts a cryptographic
  precondition from a documented caveat into an enforceable build-time gate. The estate is under
  two open audit gates — an external single-prover cryptographer audit (`sq-qhy4`) and this
  collaborative re-audit (`sq-9hrn`) — neither of which is discharged here.
]]

#heading(level: 2, numbering: none, outlined: false)[Abstract]

A federated query engine that wants to let several data holders jointly prove one statement over
their private graphs is reaching for a _collaborative_ zero-knowledge SNARK: an MPC among the
provers that produces a single proof a relying party can check with an ordinary single-prover
verifier. CRYPTO'25 (eprint 2025/1026) showed that this template carries failure modes that are
easy to miss — most sharply, that proving over an _inconsistent or maliciously extended_ witness
can leak an honest prover's private inputs even when the verifier still rejects the proof, and
that the folklore "honest-majority semi-honest gives malicious security for free" result holds
for collaborative proving _only if the extended witness is validated before proving_. We report an
adversarial re-audit of one engine's _intended_ collaborative path against these failure modes.
The honest disposition is a _negative result_: the path is *unbuilt* —
#headline("cozk.deferred_proof_methods") proof/attestation entry points return a gate-naming
`NotYetImplemented` rather than executing — so there is no prover to forge against and *no
soundness can be certified*. We assign each of the #headline("cozk.reaudit_lenses") 2025/1026
lenses a _RE-OPEN_ verdict (never CLOSED), state plainly that the validate-before-prove
precondition is _unmet_, and that the closest usable third-party stack is itself _unaudited_. The
durable output is _R-WV_, a #headline("cozk.test_obligation_clauses")-clause witness-validation
test obligation that we encode as a build-time gate: a passing meta-test pins the current
fail-closed posture and fires if a future prover ever returns success while the obligation is
unmet, and an `#[ignore]`d obligation suite is written against the contract the eventual prover
must satisfy. We make no security, privacy, or attestation claim; we make the gap _enforceable_
so it cannot silently re-open when the path is built.

== What this paper is, and is not

We are explicit before any technical content, because the framing is the contribution.

This paper *is*:
- an _adversarial re-audit methodology_ — mapping a published set of collaborative-zk-SNARK
  failure modes (2025/1026) onto a concrete _intended_ stack, lens by lens, and assigning each a
  verdict that is CLOSED only when the design _provably avoids_ the pitfall and RE-OPEN otherwise;
- an _honest negative finding_ — the audited path is *not yet* sound to claim, and the reasons are
  named, not hedged;
- an _engineering artifact_ — a test obligation (R-WV) that turns the literature's precondition
  into a gate a build can check.

This paper *is not*, and must not be read as:
- a proof that the collaborative path is sound — it is *not yet* sound, and we *cannot certify* it
  (there is nothing built to certify);
- a privacy, integrity, or attestation guarantee for the engine — *none* is asserted; the estate
  provides _no_ production guarantee to a relying party pending external audit;
- a claim that the honest-majority "free" malicious-security result applies — whether it applies
  to this engine's specific relation is an *open* question for the eventual external audit;
- a performance result — the collaborative path has _zero_ timing data points and we assert no
  number about it.

Two open audit gates govern everything below and are never softened: the external single-prover
cryptographer audit (`sq-qhy4`) is *open*, and this collaborative re-audit (`sq-9hrn`) is itself
the _gating_ disposition for the multi-prover path. No security property may be claimed proven
until both close; this paper closes *neither*.

== Background: the collaborative-proving template and its hazards <bg>

The intended path is the standard collaborative-zk-SNARK shape. Several holders are the provers;
each holds a private witness — a committed graph, the values it discloses, the row encodings. They
run an honest-majority MPC to compute a federated correctness relation over secret-shared values
and to jointly emit one proof, checkable by an _unchanged_ single-prover verifier. This is exactly
the template 2025/1026 studies, and two of its results bite directly:

- *Privacy is not free.* Even where soundness would survive, a malicious prover who steers the MPC
  onto an inconsistent or extended witness and then induces an opening can exfiltrate honest
  provers' private witness bits. The defence is to _validate the extended witness for consistency
  before the proving phase opens or commits to anything derived from it_.
- *"Free" malicious security is conditional.* The honest-majority "semi-honest gives malicious
  security for free" folklore holds for collaborative proving _only if_ the extended witness is
  validated before proving, and generic semi-honest-to-malicious MPC compilers are _not_ safe to
  apply naively to the proving setting.

The engine's MPC layer is honest-majority by construction, which places it in the regime where the
"free" result _could_ apply — and that is precisely what makes the validate-before-prove
precondition the whole game, because the antecedent is satisfied and only the consequent is in
doubt.

== The re-audit: four lenses, every verdict RE-OPEN <lenses>

We audited the intended path against #headline("cozk.reaudit_lenses") lenses, each a documented
2025/1026 failure mode. A lens earns CLOSED only if the design _provably avoids_ the pitfall;
otherwise it is RE-OPEN with the specific unmet precondition named. _Every_ lens is RE-OPEN — not
because a present exploitable hole was found (there is no shippable prover to forge against), but
because the path is unbuilt and the preconditions are unfilled.

#figure(
  table(
    columns: (auto, 1fr, auto),
    align: (center, left, left),
    table.header[Lens][2025/1026 failure mode][Verdict],
    [1],
    [Witness-extension leakage — proving over an inconsistent / extended witness can leak honest
     inputs even when the verifier rejects.],
    [RE-OPEN],
    [2],
    [Malicious-compiler insecurity — a generic semi-honest-to-malicious compiler is _not_ safe to
     apply as-is to a collaborative prover.],
    [RE-OPEN],
    [3],
    [Honest-majority semi-honest attains malicious security _iff_ the extended witness is validated
     before proving.],
    [RE-OPEN],
    [4],
    [The closest usable third-party collaborative-proving stack is _unaudited_ and predates the
     paper.],
    [RE-OPEN],
  ),
  caption: [
    The #headline("cozk.reaudit_lenses") re-audit lenses and their verdicts. _Every_ verdict is
    RE-OPEN: the trust model places the engine in the regime where the honest-majority "free"
    result _could_ apply, but the validate-before-prove precondition is unmet and unencoded, no
    collaborative-proving construction is selected, and the nearest usable dependency is unaudited.
    A RE-OPEN is _not_ a finding of a live exploit — it is the honest disposition for an unbuilt,
    audit-gated path, and a CLOSED verdict on any lens would be premature.
  ],
)

#provenance("cozk.reaudit_lenses")

Lens 1 (witness-extension leakage) is the live shape. The MPC layer already documents the same
leak at its own level: a mid-pipeline open at the minimal honest-majority threshold carries no
redundancy to catch an inconsistent share, and opening a value computed on an inconsistent witness
is — per 2025/1026 — a confidentiality hazard, not only a correctness one. The planned mitigation
(authenticate every shared value and batch-check before any open) is the MPC-layer analogue of
the paper's validate-before-prove defence; it is correct in intent and _not yet landed_. For the
collaborative-_proving_ layer specifically, the validate-the-extended-witness-before-proving check
had _no_ design artifact and _no_ test obligation before this re-audit. Lens 3 is the load-bearing
one: the honest-majority antecedent is satisfied by construction, so the consequent — validate
before proving — is the single most important gating requirement, and it is _unmet_ because no
proving phase exists. Lenses 2 and 4 are dependency-and-construction risks: no
collaborative-proving construction has been chosen and patched, and the nearest usable stack is
explicitly _unaudited_ and predates the paper, so adopting it would import an _unverified_
cryptographic trusted-computing-base.

== The negative result is grounded in an unbuilt path <unbuilt>

The honesty of this contribution rests on a checkable fact: there is nothing to certify because
the collaborative path is _not built_. On the proof boundary,
#headline("cozk.deferred_proof_methods") entry points — the joint-proving `prove` and `verify`,
the distributed `attest_source`, and their stub-test counterparts — each return a gate-naming
`NotYetImplemented` rather than executing, and the in-circuit distributed signature over the
secret-shared witness is the deferred spike the project itself calls "the join nobody has built."

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header[Committed structural fact][Count],
    [Collaborative-proof / attestation entry points returning a gate-naming `NotYetImplemented`],
    [#headline("cozk.deferred_proof_methods")],
    [2025/1026 failure-mode lenses, _all_ assigned RE-OPEN (none CLOSED)],
    [#headline("cozk.reaudit_lenses")],
    [Confirmed findings in the prior _single-prover_ verifier audit this path would build on],
    [#headline("cozk.single_prover_audit_issues")],
  ),
  caption: [
    The committed structural facts behind the negative result — deterministic source/document
    scans over committed code and the re-audit record, _not_ measurements. The
    #headline("cozk.deferred_proof_methods") fail-closed entry points are the evidence that there
    is no prover to forge against; the prior single-prover audit's
    #headline("cozk.single_prover_audit_issues") confirmed findings are cited only to show the
    foundation is itself under an open external-audit gate (`sq-qhy4`), not to claim either is
    fixed.
  ],
)

#provenance("cozk.deferred_proof_methods")

This is why the re-audit _cannot certify soundness_, and says so. There is no collaborative prover
to attack; the verdicts are about whether the _intended design_ avoids documented failure modes,
and the answer is "not yet — the preconditions are unfilled." The current honest posture — every
collaborative-proof method failing closed with a gate-naming error — is the _correct_ state for an
unbuilt, audit-gated path; the re-audit's job is not to bless it but to make the gate enforceable
when the path is eventually built.

The foundation matters too. The single-prover verifier the collaborative path would plug into is
itself under an open external-audit gate: a prior adversarial audit recorded
#headline("cozk.single_prover_audit_issues") confirmed findings on a v1 verifier (a v1 documented
_not_ sound), and while the estate was internally re-audited as sound-as-landed for its stated
threat model, no accredited external cryptographer has reviewed it (`sq-qhy4`, _open_). Critically,
`sq-qhy4` audits the _single-prover_ verifier only — it does _not_ discharge the multi-prover
construction this re-audit governs. So the collaborative path inherits an _unverified_ foundation
on top of an _unbuilt_ superstructure: two independent reasons no soundness claim is available.

== R-WV: encoding the precondition as an enforceable gate <rwv>

The durable output is to stop the validate-before-prove precondition from being a documented
caveat that a future implementer could miss, and make it a gate a build _checks_. We state it as a
requirement and decompose it into a #headline("cozk.test_obligation_clauses")-clause test
obligation.

*Requirement (R-WV).* _A collaborative-proving implementation must validate the shared extended
witness for cross-holder consistency, and must abort fail-closed before any value derived from the
extended witness is opened or committed into the joint proof, whenever the extended witness is
inconsistent or maliciously extended._ No "prove-anyway-and-let-the-verifier-reject" path may
exist — that path _is_ the 2025/1026 leak.

#figure(
  table(
    columns: (auto, 1fr),
    align: (left, left),
    table.header[Clause][The obligation it encodes],
    [T1],
    [Inconsistent-share _abort before open_: an off-codeword share of a witness value must drive a
     fail-closed abort _before_ any open or proof-commit step runs — asserted via an instrumented
     round-counter showing zero witness-derived opens after the inconsistency is introduced, and
     _no_ proof emitted.],
    [T2],
    [Witness-extension _leakage probe_: a witness inconsistent with its signed commitment must abort
     before proving, and an honest holder's private bits must be information-theoretically
     _unrecoverable_ from the transcript up to the abort (zero opens on honest-derived lineage).],
    [T3],
    [Validate-before-prove is _load-bearing, not advisory_: a differential test that, with
     validation disabled, the same adversarial input would otherwise reach an open/proof step —
     pinning the gate onto the critical path so a refactor cannot silently move proving ahead of
     validation.],
    [T4],
    [_Commitment-binding_ of the validated witness: a prover that validates witness A but proves
     over witness B must be rejected at the binding seam — closing the bait-and-switch between the
     validated and the proven witness.],
    [C],
    [Construction-_provenance_ assertion (documentary / CI, not a runtime test): the adopted
     construction must record the specific 2025/1026-patched variant it instantiates (_not_ a naive
     semi-honest-to-malicious compiler), and if a third-party stack is used, its pinned version plus
     a re-run of this lens-set against that exact version.],
  ),
  caption: [
    The #headline("cozk.test_obligation_clauses")-clause R-WV test obligation (re-audit §3). T1–T4
    are runtime obligations the eventual prover must satisfy; clause C is a documentary / CI
    provenance assertion. The obligation is _necessary_ and explicitly _not_ claimed _sufficient_ —
    it encodes the 2025/1026 precondition and the bind-the-validated-witness discipline; full
    soundness additionally requires the external multi-prover audit, which is _not_ in scope here.
  ],
)

#provenance("cozk.test_obligation_clauses")

The encoding is two-tier, and its disposition is _OPEN, not met_. First, a _passing_ meta-test
pins the current fail-closed posture: `prove` refuses with a gate-naming error and never opens a
witness-derived value or emits a proof, so no prove-over-an-invalid-witness path exists today —
vacuously, because no prover exists. That meta-test is the regression anchor: it fires if a future
`prove` ever begins returning success while R-WV is still unmet. Second, the T1–T4 suite is
`#[ignore]`d — a documented _open_ obligation, each clause citing the re-audit and the audit gate
— written against the prover contract the eventual implementation must satisfy, to be un-ignored
when that prover lands. This makes _no_ soundness claim and closes _no_ lens; it converts the
precondition from a documentary caveat into a build-time-measurable gate, so the gap cannot
silently re-open. Clause C remains a documentary obligation, because no construction has been
adopted to record.

== What this re-audit cannot conclude <limits>

We state the limits as plainly as the result, because an adversarial audit of an _unbuilt_ path is
honest only if it does.

- *It cannot certify the path is sound.* There is no collaborative prover to forge against; the
  verdicts concern whether the _intended design_ avoids documented failure modes, and the answer is
  "not yet — preconditions unfilled." A CLOSED verdict on any lens would be premature and is
  deliberately withheld.
- *The "free malicious security" applicability is unproven here.* The 2025/1026 positive result is
  stated for a _class_ of constructions; whether this engine's eventual federated-correctness +
  commitment-fold relation, under its chosen sharing scheme and proof system, lands inside that
  class is an _open_ question for the eventual external audit.
- *The third-party-stack risk is qualitative.* Absent an audit of a specific pinned version, we can
  only flag the nearest usable dependency as _unaudited_; we _cannot_ say it is broken, only that it
  is _unverified_ against the very failure modes this re-audit is about.
- *Performance is out of scope with zero data points.* The collaborative path has _no_ timing
  number, and we assert none.

The single durable output is the R-WV requirement and its
#headline("cozk.test_obligation_clauses")-clause obligation: it turns the 2025/1026 precondition
from a caveat into an enforceable gate. That is the whole contribution, and it is a _negative_ one.

== Related work and honest positioning

Collaborative zk-SNARKs (the line eprint 2025/1026 extends), honest-majority MPC, and IT-MAC
authentication are established cryptographic work, and we claim no novelty in any of them. We also
claim no novelty in the _content_ of the 2025/1026 results; we _apply_ them. The contribution is
the _methodology and the artifact_: an adversarial re-audit that maps a specific published set of
collaborative-proving hazards onto a concrete intended stack and refuses a CLOSED verdict the
evidence does not support, plus the R-WV test obligation that encodes the load-bearing precondition
as a build-time gate against an _unbuilt_ path. This is a security-engineering _lessons_ result, not
a cryptographic theorem — and its honesty is the point: it reports that the path is _not yet_ sound
to claim, names every reason, and ships a gate rather than a guarantee. The genre — a published
_negative result_ that overturns the comfortable assumption that an honest-majority semi-honest
collaborative prover is "free" — is itself the kind of contribution the literature asks for and the
project's empirical-honesty mandate requires.

== Conclusion

A federated engine that wants several holders to jointly prove one statement over their private
graphs is reaching for a collaborative zk-SNARK, and CRYPTO'25 showed that template hides leakage
and conditional-security pitfalls that are easy to miss. We re-audited one engine's _intended_ path
against #headline("cozk.reaudit_lenses") of those pitfalls and reached an honest _negative result_:
the path is *unbuilt* — #headline("cozk.deferred_proof_methods") proof/attestation entry points
fail closed with a gate-naming error — so we *cannot certify soundness* and assign _every_ lens
RE-OPEN. The foundation it would build on is itself under an open external audit
(`sq-qhy4`, #headline("cozk.single_prover_audit_issues") prior single-prover findings), and this
collaborative re-audit (`sq-9hrn`) is the gating disposition for the multi-prover path. We claim
_no_ security, privacy, or attestation property — there is nothing built to claim it of. What we
contribute is the methodology, the honest finding, and R-WV: a
#headline("cozk.test_obligation_clauses")-clause witness-validation-before-proving obligation,
encoded as a build-time gate that pins the current fail-closed posture and fires if a future prover
ever proves while the precondition is unmet. The value is not a guarantee — it is that the gap is
made _enforceable_ instead of merely documented, so it cannot silently re-open when the path is
finally built.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · this paper is a NEGATIVE-RESULT / security-engineering-lessons contribution and
    asserts _no_ proven security, privacy, soundness, or attestation property. Evidence traces to
    the adversarial re-audit `research/mpc-cozk-reaudit.md` (bead `sq-9hrn`), the encoded test
    obligation `crates/sparq-mpc/src/witness_validation_tests.rs` (bead `sq-7leq`), the deferred
    proof boundary `crates/sparq-mpc/src/proof.rs`, and the prior single-prover audit
    `research/zk-soundness-audit.md` under the open external-audit gate `sq-qhy4`. The estate is
    research-grade and _not_ externally audited; the collaborative path is unbuilt. Counts in this
    document are injected at build time from the paper-bound evidence file; see the provenance stamp
    on the published page.
  ]
]
