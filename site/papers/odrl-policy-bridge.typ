// [OPUS-4.8] sq-gum8 — Paper B3: compiling ODRL usage policies into a queryable Solid/SPARQL
// access-control view (the single-node ODRL -> WAC/ACP conditional-grant bridge).
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).
// Framed HONESTLY as a SYSTEMS / IN-USE contribution: the evidence is a set of DETERMINISTIC,
// test-proven answer-safety invariants of the bridge PLUS the CI-enforced WAC/ACP decision-parity
// ratchet floors the bridge materialises into. NOT a performance result, NOT a novelty claim about
// usage-control semantics, and explicitly NOT a security/soundness claim. The genuinely-novel
// federated ODRL->MPC disclosure / ODRL-Duty->ZK-proof-obligation composition is DEFERRED as honest
// future work: the cryptographic estate is research-grade and not externally audited (gate sq-qhy4),
// and sparq-mpc is honest-majority semi-honest only — see §6.

#import "_lib/bench.typ": headline, ev, provenance, authors, anon

#set document(title: "Compiling ODRL Usage Policies into a Queryable Access-Control View")
#set text(size: 11pt)
#set par(justify: true)
#set heading(numbering: "1.")

#align(center)[
  #text(size: 17pt, weight: "bold")[
    Compiling ODRL Usage Policies into a Queryable Access-Control View for Solid/SPARQL:
    A Single-Node, Fail-Closed Conditional-Grant Bridge
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A systems / in-use contribution. The evidence is deterministic and machine-independent:
  test-proven answer-safety invariants of the bridge (deny-overrides, fail-closed retraction,
  re-checked conditional grants, atomic count enforcement) and the CI-enforced Solid WAC and ACP
  decision-parity ratchet floors the bridge materialises into. No wall-clock performance is
  claimed, and no security, soundness, or completeness property of the cryptographic estate is
  asserted. The federated ODRL-to-MPC disclosure and ODRL-Duty-to-ZK-proof-obligation composition
  is _deferred_ as honest future work; the relevant crypto is research-grade and not externally
  audited (§6).
]]

== Abstract

Solid access control answers _may this agent read graph G?_; ODRL usage control answers a
different question — _may this party use this asset, for purpose P, until time T, with obligation O
discharged, disclosing only to recipient R?_ The two are usually kept in separate engines. We
describe a single-node bridge that _compiles_ a matched ODRL rule into the very same triples the
host's existing Web-Access-Control / Access-Control-Policy enforcement already understands: a
Permission becomes an `auth:<mode>` grant in a queryable access-control view, a Prohibition becomes
an explicit `auth:deny<mode>` triple, and a faithfully-mappable recipient or time constraint becomes
a re-checked conditional grant rather than a frozen decision. No new enforcement engine is added —
the existing view (`<urn:sparq:auth>`), which the engine answers SPARQL against, is the single
decision surface. The contribution is the _integration and its fail-closed discipline_, not a new
usage-control semantics. We substantiate it with deterministic, test-proven invariants —
deny-overrides, asymmetric fail-closed deny-retraction on policy refresh, re-checked conditional
grants, and atomic stateful count enforcement — and we anchor the access-control layer the bridge
targets in the project's CI-enforced WAC and ACP decision-parity ratchet floors. We make no
wall-clock claim and assert no cryptographic property; the genuinely-novel federated / ZK
disclosure composition is honestly deferred.

== Contributions

This paper substantiates the following claims; each is refutable and forward-references its
evidence. None is a performance claim, none claims a novel usage-control semantics, and none
asserts a security, soundness, or attestation property of the engine or its cryptographic estate.

- *ODRL rules compile into the existing queryable access-control view* (§3) — a matched
  Permission materialises an `auth:<mode>` grant and a matched Prohibition an `auth:deny<mode>`
  triple, both appended to the `<urn:sparq:auth>` view the engine already answers SPARQL against, so
  no separate usage-control enforcement engine is introduced.
- *The bridge is fail-closed and respects deny-overrides* (§4) — when a Permission and a matched
  Prohibition collide on the same principal, action, and target, the materialised deny wins through
  the unchanged decision procedure; this is asserted as an invariant
  (#headline("policy_bridge.deny_overrides_correct")).
- *Policy refresh retracts grants asymmetrically and fail-closed* (§4) — a stale grant is removed
  on refresh, but a materialised _deny_ is retracted only when re-evaluation proves the prohibition
  definitely withdrawn; an ambiguous re-evaluation keeps the deny, so access is never silently
  widened (#headline("policy_bridge.fail_closed_deny_retraction")).
- *Faithfully-mappable constraints become re-checked conditions; the rest fall back safely* (§5) —
  a recipient constraint persists as an ACP conditional grant re-checked per session, while an
  unmappable constraint falls back to a one-shot evaluation rather than being mis-encoded
  (#headline("policy_bridge.conditional_grant_rechecked")); a stateful `odrl:count` budget is
  consumed atomically, granting up to the limit and then denying, with denials burning no budget
  (#headline("policy_bridge.count_enforcement_atomic")).
- *The target access-control layer carries deterministic, CI-enforced conformance floors* (§3) —
  the WAC and ACP decision procedure the bridge feeds is ratcheted at decision-parity floors of
  #headline("conformance.solid_wac_floor") and #headline("conformance.solid_acp_floor") passing
  scenarios, each a monotone lower bound CI enforces.

== ODRL into the existing access-control view <bridge>

The host engine governs graph access through an access-control _view_: a named graph
`<urn:sparq:auth>` of `principal auth:<mode> target` triples (and their `auth:deny<mode>` duals),
materialised from a resource's WAC `.acl` document or ACP access-control resource, against which one
decision procedure computes a principal's accessible set as the union of allow-grants minus the
union of deny-grants. Because that view is just RDF in the engine's own store, it is _queryable_ —
the same SPARQL evaluator that answers user queries answers "what may this principal do?".

The bridge re-uses that surface rather than adding to it. A `sparq-policy` ODRL evaluator answers a
usage-control request — party, action, target, satisfied constraints, discharged duties — with a
fail-closed allow/deny `Decision`. On a definite Permit, the bridge maps the request's _concrete_
ODRL action to the _narrowest_ access mode it denotes (`odrl:read` / `display` / `present` /
`print` / `play` to read; `odrl:append` to append; `odrl:modify` / `delete` / `write` to write; the
`odrl:use` umbrella is deliberately left unmapped, since materialising it as any single mode would
have to pick the widest and violate fail-closed) and appends the corresponding `auth:<mode>` grant
to the view. A matched Prohibition is the dual: it appends the explicit `auth:deny<mode>` triple the
decision procedure already subtracts. The net effect is that an ODRL rule becomes a concrete WAC/ACP
grant or denial honoured by the _existing_ enforcement, unchanged — the bridge only emits the
triples, and mirrors each into a provenance graph so a bridged triple stays structurally
distinguishable from a static one.

This is the implemented, single-node half of a larger thesis, and we are explicit that it is an
_integration_, not a new semantics: instantiating ODRL-to-access-control mappings has prior art
(for example OAC- and Pandit-class mappings of ODRL into access-control vocabularies). What is
re-used here is the access-control layer the bridge targets, whose decision content the project
gates with two deterministic ratchets: WAC decision parity at a floor of
#headline("conformance.solid_wac_floor") scenarios and ACP at
#headline("conformance.solid_acp_floor"), each a monotone lower bound CI enforces over a fixed
per-construct scenario table.

#provenance("conformance.solid_wac_floor")

== Fail-closed materialisation and deny-overrides <failclosed>

A grant is materialised _only_ on a definite Permit whose action maps to a concrete mode and whose
request names a concrete party (a WebID) and a target graph IRI. A Deny, an ambiguous evaluation, an
unmapped action (including the `odrl:use` umbrella), or a missing party/target materialises
_nothing_ — access is never widened on ambiguity. The dual holds for prohibitions.

When a Permission and a matched Prohibition apply to the same principal, action, and target, the
materialised deny is the operative decision: the decision procedure computes the accessible set as
union-of-allow minus union-of-deny, so the deny removes the mode regardless of any allow grant. This
is _deny-overrides_, the conflict default of the ODRL Formal Semantics, realised without a bespoke
resolver because the access-control layer already implements set subtraction. We assert it as a
deterministic invariant — with both a Permission and a matching Prohibition materialised for the
same principal, that principal's accessible set for the contested mode is empty:
#headline("policy_bridge.deny_overrides_correct").

The harder direction is _revocation_. Materialised grants are tracked in a ledger so that when the
underlying policy changes the bridge can refresh the view: each tracked rule is re-evaluated, and an
entry that no longer produces a grant (a withdrawn permission, a lapsed time window, a now-denying
re-evaluation) is dropped, so withdrawn access is gone. Deny retraction, however, is _asymmetric and
fail-closed_: a materialised deny is retracted only when re-evaluation proves the prohibition
_definitely_ withdrawn; an _ambiguous_ (unprovable) re-evaluation keeps the deny. Retracting a deny
on uncertainty would silently restore access, so the bridge refuses to — a three-valued
(applies / ambiguous / withdrawn) refinement asserted as an invariant:
#headline("policy_bridge.fail_closed_deny_retraction"). The two halves compose: deny-overrides still
holds after a refresh.

== Conditional grants and stateful constraints <conditional>

Many ODRL constraints are _stateful_ or _per-session_ and cannot be soundly frozen into a one-shot
decision. The bridge handles the faithfully-mappable ones by persisting them as ACP _conditional
grants_ re-checked at decision time, and falls back safely on the rest.

A recipient constraint (the rule names who may receive the asset) persists as a conditional grant
whose agent is re-checked per session, so only the named recipient is granted and the materialising
party is not auto-granted; a recipient _set_ becomes one condition per member (a union), and a
recipient _exclusion_ becomes a public grant with an explicit carve-out. A `dateTime` window
persists as live-clock bounds re-checked against the session clock, so a lapsed window denies
immediately without waiting for a refresh — and, for a _deny_, time windows are forbidden on the
conditional path (a lapsed deny would fail _open_), forcing a one-shot fallback instead. A
constraint the bridge cannot map faithfully (for example a `purpose` constraint) is _not_ encoded as
a re-checked condition that would silently mis-judge it; it falls back to a one-shot evaluation,
checked once and frozen. We assert the mapping discipline as an invariant — the named recipient is
granted, the materialising party is not, and an unmappable constraint takes the one-shot path:
#headline("policy_bridge.conditional_grant_rechecked").

A stateful `odrl:count` budget needs genuine state, not a re-check of static data. Behind an opt-in
count-enforcement feature, the bridge routes the grant through an evaluator that _atomically_
consumes exactly one unit of budget on a grant: it grants up to the limit and then denies, a denial
consumes nothing, and budgets are isolated per (rule, party, target). This is asserted as a
deterministic invariant over the integer counter — exactly-up-to-the-limit then deny, with denials
burning no budget: #headline("policy_bridge.count_enforcement_atomic"). The counter store is
pluggable (in-memory, an OS-lockfile-guarded file for cross-process use, or a backend trait), which
is an engineering choice, not a contribution.

== Honest limitations and deferred work <limits>

The headline of the broader thesis — the genuinely-novel part — is _not_ in this paper, and we say
so plainly. That headline is a _federated_ composition: per-node ODRL drawing the disclosed-vs-hidden
boundary for a multi-party computation, and an ODRL `Duty` becoming a zero-knowledge proof
obligation. That composition is _designed only_ and deferred. It would inherit the project's
multi-party-computation envelope, which is honest-majority and semi-honest over a low-latency
network, and it would build on a zero-knowledge estate that is _research-grade and has not been
externally audited_ — the external-audit gate (bead sq-qhy4) is open, and the collaborative
multi-prover path is itself the subject of an open re-audit. We therefore claim _no_ security,
privacy, integrity, or attestation property for any federated or cryptographic disclosure here; what
this paper ships is the single-node bridge only.

Three further limitations bound even the single-node claim. First, the action-to-mode mapping is
deliberately conservative and partial — the `odrl:use` umbrella is unmapped — so a request must name
a concrete action to be bridged; this is a fail-closed choice, not full ODRL action coverage.
Second, the constraints handled as re-checked conditions are exactly the faithfully-mappable
recipient and time constraints; everything else (notably `purpose`) is one-shot, so a purpose
decision is frozen at materialisation rather than re-checked. Third, this is a research-track,
single-node surface, not a production cutover: the evidence is the bridge's _answer-safety_
invariants and the conformance floors of the access-control layer it feeds, not a proof of
completeness or of the absence of an unsafe grant.

== Related work and honest positioning

ODRL-to-access-control mappings exist (OAC, Pandit-class), and WAC/ACP enforcement is implemented by
the Solid reference servers; we claim no advance over either, and the access-control constructs the
bridge targets are exactly those the specs define. The only systems angle is the _compilation into a
queryable view_ — an ODRL rule becomes ordinary RDF in the engine's own store, decided by the same
SPARQL machinery — together with the fail-closed discipline (deny-overrides, asymmetric deny
retraction, re-checked-vs-one-shot constraint handling, atomic count enforcement) that keeps the
materialisation answer-safe. The genuinely-novel federated / ZK disclosure composition is reported
elsewhere as honest design and is explicitly out of scope here. The methodology is engineering
rigour rather than a research result; it is presented as the machine-checkable _evidence_ for the
in-use claim.

== Conclusion

A usage-control layer need not bring its own enforcement engine: a matched ODRL rule can be compiled
into the very triples an existing, queryable Web-Access-Control / Access-Control-Policy view already
understands, so one SPARQL-backed decision procedure serves both layers. The honest evidence is
answer-safety and conformance, not speed — deny-overrides
(#headline("policy_bridge.deny_overrides_correct")), asymmetric fail-closed deny retraction
(#headline("policy_bridge.fail_closed_deny_retraction")), re-checked conditional grants
(#headline("policy_bridge.conditional_grant_rechecked")), and atomic count enforcement
(#headline("policy_bridge.count_enforcement_atomic")), over an access-control layer ratcheted at WAC
and ACP decision-parity floors of #headline("conformance.solid_wac_floor") and
#headline("conformance.solid_acp_floor"). We make no wall-clock claim and assert no cryptographic
property; the federated ODRL-to-MPC disclosure and ODRL-Duty-to-ZK composition is deferred as honest
future work, gated on an external audit that is not yet complete.

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
