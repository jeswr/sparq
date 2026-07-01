// [FABLE-5] sq-gum8.3-odrl — Paper B3 REWRITE: "An ODRL Policy Bridge for SPARQL Access Control".
// Venue-bar rewrite executing the sq-gum8.2 audit verdict (REWRITE): related work now
// differentiates feature-by-feature from OAC / ODRE / Slabbinck-class systems; novelty is
// regrounded in the single-node lifecycle discipline (refresh, retraction, statefulness) and
// NOT in the deferred crypto half; the four machine-checked invariants are honestly demoted
// from "the evaluation" to verification evidence, alongside a pre-registered comparative
// decision-agreement protocol whose results are PENDING (this is why the registry status is
// `draft`, not `publishable-now`).
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
// EMPIRICAL-HONESTY (sq-gum8): no wall-clock claim (work-box timings are non-canonical and are
// refused by the paper factory's headline gate); no security/soundness/attestation claim; the
// federated ODRL->MPC / ODRL-Duty->ZK composition is deferred, unbuilt, research-grade, and not
// externally audited (open gate sq-qhy4) — see §7.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "An ODRL Policy Bridge for SPARQL Access Control")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    An ODRL Policy Bridge for SPARQL Access Control:
    Fail-Closed Compilation of Usage Policies into a Queryable Solid Access-Control View
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A systems / in-use contribution, submitted as a *draft*: the verification evidence
  (machine-checked answer-safety invariants; CI-ratcheted decision-parity floors of the target
  access-control layer) is deterministic and machine-independent, and the comparative
  decision-agreement study against independent ODRL enforcers is specified in §5.3 but *not yet
  run*. No wall-clock performance is claimed, and no security, soundness, or completeness
  property of any cryptographic component is asserted; the federated ODRL-to-MPC /
  ODRL-Duty-to-ZK composition is deferred, unbuilt future work (§7).
]]

== Abstract

Solid's access-control models answer _may this agent read graph G?_; ODRL usage control answers
a richer question — _may this party use this asset, for purpose P, until time T, with obligation
O discharged, disclosing only to recipient R?_ Existing ODRL enforcement systems answer it with
a *separate policy engine* evaluated per request. We take a different route: a matched ODRL rule
is _compiled_ into the very triples the host SPARQL engine's existing Web-Access-Control (WAC) /
Access-Control-Policy (ACP) enforcement already understands — a Permission becomes an
`auth:<mode>` grant in a queryable access-control view, a Prohibition an explicit
`auth:deny<mode>` triple, and a faithfully-mappable recipient or time constraint a _re-checked
conditional grant_ rather than a frozen decision. One decision surface — ordinary RDF, answered
by the same SPARQL evaluator that serves user queries — then serves WAC, ACP, and ODRL at once,
and every usage-control decision is itself queryable and provenance-tagged. Compilation raises
lifecycle problems that per-request evaluators do not face, and those are where we claim our
contribution: fail-closed materialisation, deny-overrides realised structurally by the target
layer's allow-minus-deny set subtraction, _asymmetric fail-closed deny retraction_ on policy
refresh (a deny survives any re-evaluation that cannot prove the prohibition withdrawn), and
atomic stateful `odrl:count` budgets. We position the bridge feature-by-feature against the
nearest prior art — the OAC profile, the ODRE enforcement framework, and Slabbinck et al.'s
ODRL/Solid integration — and evaluate honestly: four machine-checked invariants and the
CI-enforced WAC/ACP decision-parity floors of the target layer are verification evidence, not a
comparative result; the comparative decision-agreement study we pre-register in §5.3 is pending,
and we make no performance or cryptographic claim.

== Introduction

Two policy layers govern data in a Solid-style personal data store. The _access-control_ layer
(WAC's `.acl` documents, ACP's access-control resources) is enforced on every request by the
host server or engine and answers mode-level questions: read, append, write. The _usage-control_
layer (ODRL policies) is richer — parties, actions, targets, constraints, duties, prohibitions —
but is conventionally evaluated by a separate engine, disconnected from the machinery that
actually gates access. That disconnect has a cost: two evaluators to trust, two semantics to
keep aligned, and usage-control decisions that are ephemeral (an evaluator's yes/no) rather than
inspectable artefacts.

This paper describes and evaluates a bridge built on a different principle: *compile, don't
co-evaluate*. The host engine already maintains a queryable access-control _view_ — a named
graph of `principal auth:<mode> target` triples (and `auth:deny<mode>` duals) materialised from
WAC/ACP sources, against which a single decision procedure computes each principal's accessible
set as union-of-allow minus union-of-deny. The bridge evaluates an ODRL request fail-closed and,
on a definite Permit or matched Prohibition, appends the corresponding grant or deny triple to
that same view, mirrored into a provenance graph so bridged triples remain structurally
distinguishable from static ones. No new enforcement engine is introduced; the existing view is
the single decision surface, and because it is ordinary RDF in the engine's own store, _the
decisions themselves are queryable_ — the same SPARQL evaluator that answers user queries
answers "what may this principal do, and why?".

Compilation is not free. A per-request evaluator faces each question fresh; a compiled view must
answer three lifecycle questions that, to our knowledge, the nearest prior systems do not
address head-on (§3): What happens when the underlying policy _changes_ — in particular, when a
prohibition's continued applicability becomes _ambiguous_? What happens to constraints whose
truth varies per session (recipients, time windows) after the decision is frozen into a triple?
And what happens to constraints that are genuinely _stateful_, such as an `odrl:count` budget?
Our answers form a single fail-closed discipline, and they are the contribution.

*Contributions.* Each is refutable and forward-references its evidence; none is a performance
claim, none claims a novel usage-control semantics, and none asserts a security, soundness, or
attestation property.

- *C1 — Compilation into an existing queryable decision surface* (§4.1). A matched ODRL
  Permission/Prohibition materialises as grant/deny triples in the host's existing WAC/ACP
  view, under a deliberately partial, narrowest-mode action mapping (`odrl:use` is refused, not
  widened). One decision procedure serves three policy languages, and decisions are auditable
  RDF with provenance separation.
- *C2 — Deny-overrides by construction* (§4.2). The ODRL Formal Semantics conflict default is
  realised without a bespoke resolver, by the target layer's unchanged allow-minus-deny set
  subtraction; asserted as a machine-checked invariant
  (holds: #headline("policy_bridge.deny_overrides_correct")).
- *C3 — Asymmetric, fail-closed retraction on policy refresh* (§4.3). Stale grants are dropped
  eagerly; a materialised _deny_ is retracted only when re-evaluation _proves_ the prohibition
  withdrawn — an ambiguous re-evaluation keeps the deny, so refresh can never silently widen
  access (holds: #headline("policy_bridge.fail_closed_deny_retraction")). We argue this
  three-valued retraction rule is the load-bearing novelty of the compiled approach.
- *C4 — Faithful dynamic and stateful constraints* (§4.4–4.5). Recipient and time constraints
  persist as per-session re-checked conditional grants; unmappable constraints fall back to a
  one-shot evaluation rather than being mis-encoded
  (holds: #headline("policy_bridge.conditional_grant_rechecked")); a stateful `odrl:count`
  budget is consumed atomically, isolated per (rule, party, target), with denials burning no
  budget (holds: #headline("policy_bridge.count_enforcement_atomic")).
- *C5 — An honest evaluation frame* (§5). We separate what is verified (the invariants above;
  the CI-ratcheted WAC floor of #headline("conformance.solid_wac_floor") and ACP floor of
  #headline("conformance.solid_acp_floor") decision-parity scenarios of the target layer) from
  what is not yet evaluated (comparative decision agreement with independent ODRL enforcers,
  §5.3, pre-registered and pending), and from what we refuse to claim (performance;
  cryptography).

*Non-claims.* The bridge is a single-node integration, not a new semantics; the federated
ODRL-to-MPC disclosure and ODRL-Duty-to-ZK obligation composition sometimes associated with
this line of work is _designed only_, unbuilt, and out of scope (§7). No wall-clock number
appears in this paper: the project's paper factory injects every figure from an evidence file
whose headline channel refuses non-canonical (work-box) measurements by construction.

== Related Work <related>

*ODRL and its formal semantics.* The ODRL Information Model and Vocabulary are W3C
Recommendations; the ODRL Formal Semantics work of the W3C ODRL Community Group fixes rule
activation and conflict handling, with deny-overrides as the conflict default. We adopt that
default and claim no advance over the semantics itself; our C2 observation is that a compiled
target whose decision procedure already subtracts denies from allows realises deny-overrides
_structurally_, with no resolver code to verify.

*OAC — the ODRL profile for access control.* Esteves, Pandit et al. (ESWC 2022) define OAC, an
ODRL profile aligning ODRL with Solid access grants, together with editor and matching-algorithm
prototypes: policies live in the pod and a matching step evaluates an access request against
them, per request. OAC is the nearest _vocabulary-level_ prior art, and the bridge is
deliberately profile-compatible in spirit: like OAC it maps ODRL actions onto Solid access
modes. It differs in _where the decision lives_: OAC-class systems answer from a separate
evaluation over the policy graph; the bridge compiles the answer into the enforcement layer's
own view, so no second evaluator sits on the request path and the materialised decision is
itself queryable RDF. OAC does not, to our knowledge, treat retraction-under-ambiguity,
per-session re-checked conditions, or stateful counts (draft note: this feature reading is to be
re-verified against the cited artefacts before submission).

*ODRE — enforcement-first ODRL.* The ODRE framework (Cimmino et al., arXiv:2409.17602) argues
ODRL policies should be _enforceable_, not merely descriptive, and ships open-source engines
that interpret enriched ODRL policies at request time, including dynamic (e.g. temporal)
constraint evaluation. ODRE is the nearest _enforcement_ prior art and the natural comparison
system for §5.3. The architectural difference is the same as with OAC — ODRE brings its own
enforcement engine, whereas the bridge re-uses the host's — and the lifecycle difference is that
a per-request engine needs no retraction story at all: nothing is materialised, so nothing can
go stale. That is a genuine simplicity advantage of ODRE's design, and we say so; the compiled
approach buys a single decision surface and auditable decisions at the price of the refresh
discipline of §4.3, which is exactly where our verification effort is concentrated.

*ODRL/Solid integrations.* Slabbinck et al. study interpreting and enforcing ODRL alongside
Solid's WAC/ACP, including rule-based translations between the languages and usage-control
patterns for Solid ecosystems. That line established that ODRL-to-WAC/ACP mappings are viable
and precise about mismatches; the bridge inherits its caution (unmappable constructs are refused
or one-shot, never approximated) and adds the materialised-view lifecycle: refresh, asymmetric
deny retraction, and stateful budgets. UMA-based usage-control architectures for Solid
(arXiv:2601.18761) move the decision to an authorization server issuing tokens; the bridge is
the opposite trade — no new party, no token plane, decisions compiled down into the store's own
access-control RDF.

*Feature-by-feature positioning.* The table below summarises the differentiation; qualitative
cells describe the cited systems' published designs, not measurements (draft note: the
competitor cells are to be re-verified against the cited papers and released implementations
before submission; the pending §5.3 study operationalises the same comparison empirically).

#figure(
  table(
    columns: 4,
    align: (left, left, left, left),
    table.header[Dimension][OAC-class (profile + matcher)][ODRE (enforcement engine)][This bridge],
    [Decision surface],
    [separate matching step over policy graph, per request],
    [dedicated enforcement engine, per request],
    [compiled into the host's existing queryable WAC/ACP view; no second engine on the request path],
    [Conflict handling],
    [profile-level; evaluator-resolved],
    [engine-resolved conflict strategy],
    [deny-overrides by the target layer's allow-minus-deny set subtraction (C2, machine-checked)],
    [Policy change / revocation],
    [re-evaluate next request],
    [re-evaluate next request (nothing materialised)],
    [ledger + refresh; eager grant retraction; _asymmetric fail-closed_ deny retraction under a three-valued re-evaluation (C3, machine-checked)],
    [Session-varying constraints],
    [evaluated at request time],
    [evaluated at request time (incl. temporal)],
    [persisted as per-session _re-checked conditional grants_ (recipient, time); fail-open combinations refused; unmappable constraints fall back one-shot (C4)],
    [Stateful constraints (`odrl:count`)],
    [not treated (to verify)],
    [not treated (to verify)],
    [atomic budget, isolated per (rule, party, target); denials burn nothing (C4, machine-checked)],
    [Decisions auditable as RDF],
    [policies are RDF; decisions ephemeral],
    [policies are RDF; decisions ephemeral],
    [decisions are triples in the auth view, mirrored to a provenance graph],
  ),
  caption: [
    Qualitative positioning against the nearest prior art. "Machine-checked" cells are backed
    by the canonical invariants of §5.1; competitor cells reflect published designs and carry
    an explicit verification note — no empirical superiority is claimed here or anywhere in
    this paper.
  ],
)

What we do _not_ claim against any of these systems: broader ODRL construct coverage (ours is
deliberately partial, §6), better performance (unmeasured), or a superior semantics (we
implement the standard one). The claim is narrower and, we believe, defensible: compilation
into an existing queryable enforcement surface is a distinct design point whose lifecycle
obligations we identify and discharge fail-closed.

== The Bridge <method>

=== Compilation into the access-control view <compile>

The host engine governs graph access through an access-control _view_: a named graph
(`<urn:sparq:auth>`) of `principal auth:<mode> target` triples and their `auth:deny<mode>`
duals, materialised from WAC `.acl` documents or ACP access-control resources. One decision
procedure computes a principal's accessible set as the union of allow-grants minus the union of
deny-grants. Because the view is RDF in the engine's own store, it is queryable by the same
SPARQL evaluator that answers user queries.

The bridge re-uses that surface. An ODRL evaluator answers a usage-control request — party,
action, target, satisfied constraints, discharged duties — with a fail-closed allow/deny
decision. On a definite Permit, the bridge maps the request's _concrete_ ODRL action to the
_narrowest_ access mode it denotes: `odrl:read` / `display` / `present` / `print` / `play` to
read; `odrl:append` to append; `odrl:modify` / `delete` / `write` to write. The `odrl:use`
umbrella is deliberately left _unmapped_: materialising it as any single mode would have to
pick the widest and violate fail-closedness, so a `use`-level request is refused rather than
widened. A matched Prohibition is the dual: it appends the explicit `auth:deny<mode>` triple
the decision procedure already subtracts. Each bridged triple is mirrored into a provenance
graph, so a compiled decision stays structurally distinguishable from a static WAC/ACP one —
this is what makes the audit query "which of this principal's rights came from ODRL, and from
which rule?" a plain SPARQL query.

A grant is materialised _only_ on a definite Permit whose action maps to a concrete mode and
whose request names a concrete party (a WebID) and a target graph IRI. A Deny, an ambiguous
evaluation, an unmapped action, or a missing party/target materialises _nothing_.

=== Deny-overrides by set subtraction <denyoverrides>

When a Permission and a matched Prohibition apply to the same principal, action, and target,
the materialised deny is the operative decision: the unchanged decision procedure computes
allow-set minus deny-set, so the deny removes the mode regardless of any allow grant. This is
deny-overrides — the ODRL Formal Semantics conflict default — realised without a bespoke
resolver, because the target layer already implements set subtraction. The property is asserted
as a machine-checked invariant: with both a Permission and a matching Prohibition materialised
for the same principal, that principal's accessible set for the contested mode is empty
(holds: #headline("policy_bridge.deny_overrides_correct")).

=== Refresh and asymmetric fail-closed retraction <refresh>

The harder direction — and, we argue, the part of the design that a per-request evaluator never
has to get right — is _revocation_. Materialised triples are tracked in a ledger so that when
the underlying policy changes, the bridge can refresh the view. On refresh, each tracked rule is
re-evaluated. For _grants_, retraction is eager: an entry that no longer produces a grant (a
withdrawn permission, a lapsed time window, a now-denying re-evaluation) is dropped, so
withdrawn access is gone at the next decision.

For _denies_, eager symmetric retraction would be unsound: a re-evaluation can be _ambiguous_ —
unable to prove the prohibition either still applicable or definitely withdrawn — and retracting
a deny on ambiguity silently restores access. The bridge therefore refines re-evaluation to
three values (_applies_ / _ambiguous_ / _withdrawn_) and retracts a deny only on _withdrawn_.
An ambiguous outcome keeps the deny. The asymmetry is deliberate: for grants, the fail-closed
direction is to drop; for denies, it is to keep. Both halves compose — deny-overrides (§4.2)
still holds after any refresh — and the retraction rule is asserted as a machine-checked
invariant (holds: #headline("policy_bridge.fail_closed_deny_retraction")).

=== Conditional grants: re-check what varies, refuse what would fail open <conditional>

Many ODRL constraints vary per session and cannot be soundly frozen into a one-shot decision.
The bridge partitions them.

_Faithfully-mappable constraints_ persist as ACP _conditional grants_ re-checked at decision
time. A recipient constraint persists as a condition whose agent is re-checked per session, so
only the named recipient is granted — the materialising party is not auto-granted; a recipient
_set_ becomes one condition per member, and a recipient _exclusion_ becomes a public grant with
an explicit carve-out. A `dateTime` window persists as live-clock bounds re-checked against the
session clock, so a lapsed window denies immediately without waiting for a refresh.

_Fail-open combinations are refused._ For a _deny_, time windows are forbidden on the
conditional path: a lapsed conditional deny would fail _open_, so the bridge forces a one-shot
evaluation instead.

_Unmappable constraints fall back safely._ A constraint the bridge cannot map faithfully (for
example `odrl:purpose`) is _not_ encoded as a re-checked condition that would silently
mis-judge it; it falls back to a one-shot evaluation, checked once and frozen — and §6 is
explicit that this freezing is a real limitation, not a feature. The partition discipline is
asserted as a machine-checked invariant — named recipient granted, materialising party not,
unmappable constraint routed one-shot
(holds: #headline("policy_bridge.conditional_grant_rechecked")).

=== Stateful constraints: atomic count budgets <count>

An `odrl:count` budget needs genuine state, not a re-check of static data. Behind an opt-in
count-enforcement feature, the bridge routes the grant through an evaluator that _atomically_
consumes exactly one unit of budget per grant: it grants up to the limit and then denies, a
denial consumes nothing, and budgets are isolated per (rule, party, target) — one party's
exhaustion cannot starve another's, and probing denials cannot burn a budget. The property is
asserted as a machine-checked invariant over the integer counter
(holds: #headline("policy_bridge.count_enforcement_atomic")). The counter store is pluggable
(in-memory; an OS-lockfile-guarded file for cross-process use; a backend trait) — an
engineering choice we do not present as a contribution.

== Evaluation <eval>

We separate three tiers, in decreasing order of what is actually established: verified
invariants (§5.1), conformance of the target layer (§5.2), and the comparative study this
draft pre-registers but has _not run_ (§5.3). §5.4 states threats to validity. There is no
performance tier: the project's methodology distinguishes canonical (deterministic,
machine-independent) evidence from indicative work-box measurement, this paper's build refuses
non-canonical numbers in headline positions by construction, and no canonical performance
runner result exists for the bridge — so no timing appears anywhere in this paper.

=== Tier 1 — machine-checked answer-safety invariants <invariants>

Four invariants pin the fail-closed discipline of §4. Each is a deterministic, CI-enforced
assertion over the composed system (bridge + unchanged enforcement layer), injected here from
the project's canonical evidence channel rather than transcribed by hand:

#figure(
  table(
    columns: 3,
    align: (left, left, left),
    table.header[Invariant][Section][Holds],
    [Deny-overrides through unchanged enforcement],
    [§4.2],
    [#headline("policy_bridge.deny_overrides_correct")],
    [Asymmetric fail-closed deny retraction on refresh],
    [§4.3],
    [#headline("policy_bridge.fail_closed_deny_retraction")],
    [Recipient conditions re-checked per session; unmappable → one-shot],
    [§4.4],
    [#headline("policy_bridge.conditional_grant_rechecked")],
    [Atomic, isolated, deny-neutral count budgets],
    [§4.5],
    [#headline("policy_bridge.count_enforcement_atomic")],
  ),
  caption: [
    Answer-safety invariants of the bridge. Honest framing: these are existence proofs over
    constructed scenarios — they demonstrate the discipline holds where it is exercised, and
    they are regression-guarded in CI, but they are not a completeness result and not an
    evaluation on third-party policies (that is §5.3, pending).
  ],
)

#provenance("policy_bridge.deny_overrides_correct")

We are explicit about epistemic weight, because an earlier draft of this paper over-weighted
this tier: an invariant of this kind refutes the claim "the bridge can be driven to widen
access in scenario class X" for the constructed X, and nothing more. Their value is that each
encodes exactly one clause of the fail-closed discipline, so a future regression in any clause
fails CI loudly.

=== Tier 2 — conformance of the target layer <floors>

The bridge's output is only as trustworthy as the decision layer it compiles into. That layer
carries deterministic, CI-enforced decision-parity ratchet floors — per-construct scenario
corpora whose allow/deny decisions must match the WAC and ACP specifications, with a floor that
may only rise:

#figure(
  table(
    columns: 3,
    align: (left, right, left),
    table.header[Suite][Ratchet floor][Basis],
    [Solid WAC decision parity],
    [#headline("conformance.solid_wac_floor")],
    [per-construct `.acl` allow/deny scenarios (library-level)],
    [Solid ACP decision parity],
    [#headline("conformance.solid_acp_floor")],
    [per-construct ACR allow/deny scenarios (library-level)],
  ),
  caption: [
    Decision-parity floors of the access-control layer the bridge materialises into; two rows
    of the project's #headline("conformance.suite_count")-suite cross-family conformance
    scoreboard (total floor #headline("conformance.cross_family_total")). Library-level
    decision parity only — not HTTP/CTH wire conformance, and not a security property.
  ],
)

#provenance("conformance.solid_wac_floor")

Honest framing again: these corpora are hand-authored and small, and they test _our reading_ of
the WAC/ACP specifications, not conformance against the Solid Conformance Test Harness over the
wire. They bound the target layer's spec-faithfulness at the library level, no more.

=== Tier 3 — pre-registered comparative study (pending) <pending>

The audit gap this rewrite is honest about: no comparison against the direct competitors has
been run, and without it this paper does not meet the bar of the venue it targets. Rather than
gesture at future work, we pre-register the protocol so the study is falsifiable before it is
run:

- *Systems.* The bridge (materialised-view path) versus the ODRE reference enforcement engines,
  and versus an OAC-style matcher where a runnable artefact is available.
- *Corpus.* Policies drawn from the cited systems' own published examples and test suites plus
  the ODRL Implementation Best Practices examples, normalised into (party, action, target,
  context) request scenarios; the corpus and normalisation script to be published with the
  paper.
- *Measure.* Per-request decision agreement: agree / disagree / out-of-scope-for-system, with
  every disagreement classified as semantics divergence, coverage gap (e.g. our unmapped
  `odrl:use`), or lifecycle difference (only the bridge has a refresh path to exercise; the
  study includes policy-mutation sequences to probe exactly the §4.3 asymmetry).
- *Coverage matrix.* Per ODRL construct (actions, constraint left-operands, duty forms):
  supported / partial / refused, for each system, replacing the qualitative "(to verify)" cells
  of §3's table with measured ones.
- *Explicit non-goals.* No latency or throughput comparison unless and until a canonical
  (bare-metal, pinned) runner exists; work-box timings would be non-canonical and are excluded
  by the paper factory's honesty gate regardless.

Draft note (tracked in the project's task system as the sq-gum8.3 rewrite program): executing
this protocol is the single blocking item between this draft and submission; the registry
status of this paper remains `draft` until the results section exists.

=== Threats to validity <threats>

_Construct validity._ Tier 1 invariants are existence proofs over constructed scenarios;
passing them does not certify the absence of an unsafe grant outside the constructed classes.
_Internal validity._ Tier 2 parity is judged against our own encoding of the WAC/ACP specs — a
shared misreading would pass both the corpus and the bridge. _External validity._ Everything
verified is verified on one engine and one implementation of the bridge; nothing yet says the
compiled-view design point transfers, and until §5.3 runs there is no evidence about behaviour
on third-party policies. _Selection bias._ The §3 comparison table was authored by us from the
cited systems' papers; the pending study is designed to replace precisely those cells with
measured values.

== Limitations <limits>

Beyond the pending comparative study (§5.3), five limitations bound the claim.

First, the action-to-mode mapping is deliberately conservative and partial: the `odrl:use`
umbrella is unmapped, so a request must name a concrete action to be bridged. This is a
fail-closed choice, not full ODRL action coverage, and it will show up as "refused" rows in the
§5.3 coverage matrix.

Second, the constraints handled as re-checked conditions are exactly the faithfully-mappable
recipient and time constraints; everything else — notably `odrl:purpose` — is one-shot, so a
purpose decision is frozen at materialisation rather than re-checked. A purpose-heavy policy
regime loses the main benefit of the conditional path.

Third, the refresh discipline (§4.3) is only as good as the triggers that invoke it: a policy
change the host never surfaces to the bridge leaves stale grants until the next refresh. The
eager-grant/asymmetric-deny rule bounds the damage direction (staleness can persist access it
should have dropped only until refresh; it can never un-deny), but the window is real.

Fourth, this is a library-level, single-node surface: no HTTP wire conformance, no multi-server
deployment, no claim that the view scales to adversarial policy volumes — unmeasured, per the
no-non-canonical-numbers rule.

Fifth, nothing in this paper is a security result. The invariants are answer-safety properties
of the composition under the stated model, not a penetration analysis, a proof of completeness,
or a proof of the absence of an unsafe grant.

== Deferred: the federated and cryptographic composition <deferred>

An earlier framing of this work leaned on a federated composition — per-node ODRL policies
drawing the disclosed-versus-hidden boundary for a multi-party computation, and an ODRL `Duty`
compiling to a zero-knowledge proof obligation. We state plainly that this half is _designed
only_: it is unbuilt, it inherits a multi-party-computation envelope that is honest-majority
and semi-honest over a low-latency network, and it would build on a zero-knowledge estate that
is research-grade and has _not_ been externally audited — the project's external-audit gate
(bead sq-qhy4) is open, and the collaborative multi-prover path is itself under an open
re-audit. We therefore claim _no_ security, privacy, integrity, or attestation property for any
federated or cryptographic disclosure, and this paper's contribution stands or falls on the
single-node bridge alone. The deferred composition is mentioned only so that no reader infers
it is implied by what is built.

== Conclusion

A usage-control layer need not bring its own enforcement engine. A matched ODRL rule can be
compiled into the very triples an existing, queryable WAC/ACP view already understands, so one
SPARQL-backed decision procedure serves both layers and every usage-control decision is an
auditable, provenance-tagged artefact. The price of compilation is a lifecycle discipline, and
that discipline — fail-closed materialisation with a narrowest-mode partial action map,
deny-overrides by set subtraction
(holds: #headline("policy_bridge.deny_overrides_correct")), asymmetric three-valued deny
retraction on refresh (holds: #headline("policy_bridge.fail_closed_deny_retraction")),
re-checked conditional grants with safe one-shot fallback
(holds: #headline("policy_bridge.conditional_grant_rechecked")), and atomic count budgets
(holds: #headline("policy_bridge.count_enforcement_atomic")) — is the contribution, positioned
feature-by-feature against OAC-, ODRE-, and Slabbinck-class systems in §3. The evaluation is
honest about its tiers: verification evidence and conformance floors are in hand; the
pre-registered comparative decision-agreement study of §5.3 is the acknowledged, blocking gap
between this draft and submission. No wall-clock number is claimed, and no cryptographic
property is asserted; the federated composition remains deferred behind an open external audit.

== References <refs>

Author-year citations in the text resolve to the following (draft note: to be converted to the
venue's citation format, with page-level details verified, at submission time):

+ W3C. _ODRL Information Model 2.2_ and _ODRL Vocabulary & Expression 2.2_, W3C
  Recommendations, 2018; W3C ODRL Community Group, _ODRL Formal Semantics_ (deny-overrides
  conflict default).
+ Esteves, B., Rodríguez-Doncel, V., Pandit, H. J., et al. _Using the ODRL Profile for Access
  Control (OAC) for Solid Pod Resource Governance_, ESWC 2022.
+ Cimmino, A., et al. _ODRE: an enforcement framework for ODRL — from descriptive to
  enforceable policies_, arXiv:2409.17602.
+ Slabbinck, W., et al. — ODRL interpretation and enforcement alongside Solid WAC/ACP
  (rule-based ODRL/Solid integration line of work, IDLab).
+ UMA-based usage control for Solid-style personal data stores, arXiv:2601.18761.
+ Solid Community Group. _Web Access Control (WAC)_ and _Access Control Policy (ACP)_
  specifications; W3C. _SPARQL 1.1 Query Language_.

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project · evidence traces to `crates/sparq-solid/src/odrl_bridge.rs` and its tests in
    `crates/sparq-solid/tests/odrl_bridge.rs`, the count-enforcement tests in
    `crates/sparq-policy/tests/odrl_count.rs`, and the Solid WAC/ACP decision-parity ratchets in
    `crates/sparq-solid/tests/conformance_wac.rs` / `conformance_acp.rs` (mirrored in the
    cross-family scoreboard `crates/sparq-conformance/src/scoreboard.rs`, drift-guarded by
    `tests/scoreboard_floors.rs`). Numbers in this document are injected at build time from the
    paper-bound evidence file; see the provenance stamp on the published page.
  ]
]
