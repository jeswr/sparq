// [FABLE-5] sq-3kd2g.1 (epic sq-3kd2g / GitHub issue #1591) — the PURE SINGLE-PROVER
// zkSPARQL ARCHITECTURE paper: system design + measured artifact facts + research-grade
// security analysis. Verified gap (research/zksparql-fragment-extension.md §6): no paper
// describes the single-prover architecture itself. This paper does NOT overlap:
//   - cozk-witness-validation.typ — the COLLABORATIVE (multi-prover) negative result;
//   - verifiable-fed-sparql.typ    — the SoK across the whole estate.
// This one is the single-prover SYSTEM: commitment scheme, fixed named circuit family,
// manifest composition, verifier re-derivation, the provable fragment, and an honest
// cost + security analysis.
//
// HONESTY FRAME (the empirical-honesty mandate + gate sq-qhy4, C-family / wip-arxiv):
// this paper asserts NO proven security, privacy, soundness, or attestation property. The
// single-prover ZK verifier is INTERNALLY RE-AUDITED but has NO external accredited
// cryptographer audit (bead sq-qhy4, OPEN). Every soundness-adjacent sentence is written
// negated or hedged (so it clears the privacy-claims gate BY MEANING, not by evasion), and
// every performance figure is a DETERMINISTIC ARTIFACT FACT (bb ultra_honk gate counts from
// the regression-gated snapshot) pulled through #headline(...) — NO wall-clock number is
// claimed as a headline (work-box timings are non-canonical; see §7). "Malicious security"
// / "soundness" appear as NOUNS or under an explicit negator on purpose.
//
// Single-source Typst. Numbers come ONLY from #headline(...) / #ev(...) (paper-evidence.json),
// never hard-coded. Compiles to BOTH a PDF (the download) and semantic HTML (the in-site page).

#import "_lib/bench.typ": headline, ev, provenance, authors, anon, paper_heading_numbering

#set document(title: "A Single-Prover Zero-Knowledge Architecture for Verifiable SPARQL over Committed RDF Graphs")
#set text(size: 11pt)
#set par(justify: true)
#show figure: set block(breakable: true)
// Section numbering is switched on AFTER the abstract; the abstract renders unnumbered as
// venue-conventional front matter and == sections number "1.", "2.", ...
#set heading(numbering: paper_heading_numbering)

#align(center)[
  #text(size: 17pt, weight: "bold")[
    A Single-Prover Zero-Knowledge Architecture for Verifiable SPARQL over Committed RDF Graphs
  ]
]
#authors()

#align(center)[#text(style: "italic", size: 0.9em)[
  A systems-and-design contribution under an open audit gate. It describes the architecture of
  a single-prover zero-knowledge query-answering stack and reports deterministic artifact facts
  about it (circuit-family gate counts). It is _not_ a security, soundness, privacy, or
  attestation proof: the verifier is _internally re-audited but not externally audited_ (the
  accredited-cryptographer review, gate `sq-qhy4`, is *open*), the hidden-holder tiers are
  explicitly _not yet sound_, and the dual-leaf value lane carries an accepted invariant
  downgrade. No property below may be read as a settled guarantee. "Soundness" and "binding"
  are used as the technical _nouns_ of the goals the design reaches for, never as achieved
  claims. Wall-clock proving cost is _non-canonical_ on our development host and is presented
  only as a model driven by the gate counts (§7).
]]

#heading(level: 2, numbering: none, outlined: false)[Abstract]

Federated SPARQL assumes every endpoint discloses its solution mappings in cleartext. A single
data holder who wants to answer a query over credentials it holds — proving the answer follows
from _attested_ data without revealing the data — needs a different machine: a zero-knowledge
proof that a SPARQL result is a genuine evaluation over committed graphs. We describe the
architecture of one such single-prover stack. Its shape is deliberate: a per-graph algebraic
commitment over the RDF-canonical form; a _fixed, named family_ of #headline("zkarch.circuit_kinds")
circuit kinds (#headline("zkarch.circuit_members") compiled members across their size lattices),
each proving one operator instance of a small monotone SPARQL fragment; a JSON _manifest_ that
composes sub-proofs through binding edges; and a verifier that _re-derives_ the circuit identity
and the claimed statement from the query text and the relying party's trust anchors, trusting
nothing the prover declares. We give the fragment and its algebraic semantics, the commitment
and attestation layer, the manifest composition model, and the verifier's
#headline("zkarch.binding_obligations")-obligation fail-closed pipeline organised around
#headline("zkarch.audit_gates") cross-cutting audit gates. We report the family's cost as
deterministic `bb` gate counts — the scan lattice spans
#headline("zkarch.gates_scan_min")–#headline("zkarch.gates_scan_max") gates and the composable
filter lanes are gate-identical at #headline("zkarch.gates_filter_lane") — and derive a proving
cost model from them, because we hold our own wall-clock numbers to be non-canonical. On
security we are equally precise: the design has survived an internal adversarial audit that
found and remediated #headline("cozk.single_prover_audit_issues") issues, each now pinned closed
by a standing forge-negative regression test, but it has _no_ external audit, so we claim no
proven property and treat the internal audit as necessary and explicitly not sufficient.

= Introduction <intro>

A verifiable credential lets a holder prove a fact an issuer signed. But real questions are
rarely a single signed fact — they are _queries_: "does the union of my credentials contain a
person over 18 whose employer is on this list?" Answering such a question while disclosing only
the answer, and proving the answer is a faithful SPARQL evaluation over _attested_ data, is the
problem this architecture addresses. It is the single-holder, single-prover case: one party
holds the graphs, commits to them, and produces one proof a relying party checks.

The problem is hard for three reasons that a naive design misses. First, _the prover is the
adversary_. Everything in a proof request is prover-controlled — the query text, the declared
circuit, the public inputs, the verification key, even the freshness nonce if the design lets
it. A verifier that trusts any prover-declared quantity has no soundness to speak of. Second,
_composition is where forgeries live_. A result over "age ≥ 18 AND employer ∈ S" is not one
proof but several — a scan, a filter, a join — and the dangerous attacks are not against a
single circuit but against the _seams_: proving `17 ≥ 17` for a `≥ 18` query, pointing a filter
edge at the wrong column, or replaying an honest proof of a true-but-different statement. Third,
_the RDF data model adds its own hazards_ — blank-node identity is graph-scoped, so a join that
correlates blank nodes across two committed graphs is semantically meaningless and must be
excluded, not silently admitted.

Why has this not been solved off-the-shelf? Verifiable-database systems (IntegriDB
@integridb, vSQL @vsql, ZKSQL @zksql) prove SQL over a _known_ database
to a client that already trusts the schema; they do not hide the graph from the verifier, and
they do not bind results to _issuer attestations_. Anonymous-credential systems
@cl01 @zkcreds prove signed attributes but not _query evaluation_ over a set of
credentials. The closest semantic-web work proves selective-disclosure soundness of SPARQL
results @braun26; this paper is a companion _systems_ description of an
independently-built engine-integrated stack and its cost and audit posture, framed under an
open external-audit gate.

This paper contributes:

- *The provable fragment and its semantics* (§#ref(<fragment>, supplement: none)) — a monotone,
  open-world-conforming SPARQL subset (BGP scans, datatype-bucketed value `FILTER`, a
  hidden-credential equality `JOIN`, membership-indifferent modifiers), pinned to the
  Pérez–Arenas–Gutiérrez algebra @pag09 with a stated _result-membership_ correctness
  target and an explicit cross-graph blank-node exclusion.
- *The commitment and attestation layer* (§#ref(<commit>, supplement: none)) — per-graph
  Poseidon2 @poseidon2 commitments over the RDFC-1.0 @rdfc10 canonical form,
  bound to issuer keys by a Schnorr-over-Baby-Jubjub @schnorr91 @eip2494
  attestation, so a prover cannot be the issuer of its own facts.
- *The fixed named circuit family and the manifest composition model*
  (§#ref(<family>, supplement: none)) — #headline("zkarch.circuit_kinds") circuit kinds,
  #headline("zkarch.circuit_members") compiled members, composed by a manifest whose binding
  edges chain sub-proofs; and *the verifier re-derivation discipline*
  (§#ref(<verify>, supplement: none)) — the #headline("zkarch.binding_obligations") fail-closed
  obligations and #headline("zkarch.audit_gates") audit gates that reconstruct the statement
  from trust anchors rather than trust the manifest.
- *A deterministic cost characterisation and a proving-cost model*
  (§#ref(<cost>, supplement: none)) — `bb` UltraHonk gate counts from the regression-gated
  snapshot, and a proving-cost model over them, with an explicit statement that our own
  wall-clock numbers are non-canonical.
- *An honest security analysis under an open gate* (§#ref(<security>, supplement: none)) — the
  threat model, the #headline("cozk.single_prover_audit_issues") findings of an internal
  adversarial audit and their remediations pinned by #headline("zkarch.forge_findings_mapped")
  standing forge-negative regression tests, and a precise statement of what the _absence_ of an
  external audit (`sq-qhy4`) means for every claim.

= The provable SPARQL fragment <fragment>

The fragment is deliberately small, and the smallness is principled rather than incidental. The
target correctness property is _result membership_: a manifest disclosing a solution mapping μ
(or asserting one exists) for a pattern P over committed graphs G#sub[1] … G#sub[n] is correct
iff μ ∈ eval(P). The admission rule is: include exactly the constructs for which membership is
_monotone_ — a witness that survives the world gaining more data. That is the operational
content of "conforms to the open-world assumption", and it is exactly the deployment model, in
which a holder presents a _subset_ of its credentials.

*Grammar and algebra.* Following the SPARQL 1.1 algebra of Pérez, Arenas and Gutiérrez
@pag09 over the RDF 1.1 graph model @rdf11, a fragment pattern is

$ P ::= "BGP" | "Filter"(C, P) | "Join"(P_1, P_2) $

where a BGP is evaluated against _exactly one_ committed graph, `C` is a datatype-bucketed value
constraint, and `Join` is an equality join whose two sub-patterns range over _distinct_ committed
graphs and share a variable. Evaluation is the standard set semantics, with SPARQL expression
errors in a `Filter` treated as _not satisfied_ @sparql11.

#figure(
  table(
    columns: (auto, auto, 1fr),
    align: (left, left, left),
    table.header[Construct][Disposition][Why],
    [`SELECT` / `ASK`], [In], [Membership / non-emptiness of eval(P); monotone.],
    [BGP scan], [In], [Row soundness + per-scan completeness proved in-circuit.],
    [Value `FILTER`], [In (4+ datatype lanes)], [Monotone under error-as-unsatisfied semantics.],
    [Equality `JOIN`], [In], [Hidden-credential join key; cross-graph blank-node join excluded.],
    [`DISTINCT`/`REDUCED`/`LIMIT`/`OFFSET`/projection], [In], [Membership-indifferent modifiers.],
    [`OPTIONAL` / `MINUS` / `NOT EXISTS`], [Out], [Non-monotone: a closed-world "no extension exists" claim.],
    [Aggregation / `GROUP BY`], [Out], [An aggregate is a whole-pattern completeness claim — closed-world.],
    [`GRAPH`], [Out], [Naming graphs discloses the attribution the model hides.],
    [`SERVICE`], [Out], [Federation is out of scope by construction.],
    [`ORDER BY`], [Out], [Membership-indifferent, but a reader infers an unproved top-k claim.],
  ),
  caption: [
    The provable fragment. Constructs are admitted on _semantics_ (monotone result membership),
    not on circuit cost; the closed-world / completeness-dependent operators are excluded because
    additional undisclosed data could falsify a "proved" answer. This is the maximal-monotone
    position; property paths and most filter _expressions_ are a designed but not-yet-implemented
    extension (bead `sq-3kd2g`), out of scope for this paper's architecture.
  ],
)

*Cross-graph blank nodes.* Blank-node identity is scoped to a single graph @rdf11, and
per-graph canonicalisation cannot align blank-node labels _across_ graphs. A `Join` solution
binding a shared variable to a blank node in more than one committed graph is therefore _excluded_
from eval; the architecture enforces this exclusion (the "Q6" guard) rather than admitting a
correlation the data model does not support.

= Commitment and attestation <commit>

*Per-graph commitment.* Each source graph is canonicalised with RDF Dataset Canonicalization
(RDFC-1.0) @rdfc10 — so the commitment is independent of blank-node labelling and triple
order — and committed with the Poseidon2 permutation @poseidon2 over the BN254 scalar
field, one commitment per graph. Poseidon2 is chosen for in-circuit efficiency (it is
arithmetization-friendly, unlike a byte-oriented hash); the commitment is a standard binding
commitment in the sense of Pedersen @pedersen91, a design goal the architecture reaches
for and does not claim as an audited property.

*Issuer attestation.* A commitment alone lets a prover commit to any graph it invents — it would
be the issuer of its own facts. The architecture therefore binds each commitment to an issuer
key by a Schnorr signature @schnorr91 over the Baby-Jubjub curve @eip2494 with a
Poseidon2-derived challenge. The verifier accepts a key only if the relying party placed it in an
_external_ trusted key set K (§#ref(<verify>, supplement: none)) — never merely because the
manifest lists it. This attestation layer is the design's answer to the single most severe class
of the internal audit (§#ref(<security>, supplement: none)); it _transfers_ trust from issuer to
result, it does not create trust in the issuer's real-world honesty, which is out of cryptographic
scope.

*Post-quantum posture (a settled negative).* The signature and the commitment binding rest on
discrete-log and hash assumptions; the Schnorr/Baby-Jubjub attestation falls to a Shor-capable
adversary, so this stack offers no post-quantum guarantee and we state so plainly rather than
imply resilience.

= The circuit family and the manifest <family>

Every sub-proof is generated against exactly one circuit of a _fixed, named_ family — this is the
load-bearing architectural choice. A fixed family means the verifier can re-derive _which_
circuit a statement demands and recompute _that_ circuit's verification key, instead of trusting a
prover-supplied key over a prover-chosen circuit. The alternative — synthesising a bespoke circuit
per query — would require a circuit-identity-to-query binding the verifier could check, a much
larger trust-model surface, and is deliberately not taken here.

#figure(
  table(
    columns: (auto, 1fr),
    align: (left, left),
    table.header[Circuit kind][Statement proved (descriptive gloss)],
    [`Scan`], [A BGP scan matches against a committed graph (row soundness + per-scan completeness).],
    [`FilterInt` / `FilterF64` / `FilterSignedInt` / `FilterDecimal`], [A datatype-bucketed value FILTER holds, operand bound to the committed literal.],
    [`FilterValueDl*`], [An opt-in dual-leaf value-lane FILTER (accepted invariant downgrade; off by default).],
    [`JoinEq`], [Two hidden credentials agree on an equality join key.],
    [`RevokeUnset`], [A revocation bit is unset in a committed status snapshot.],
    [`HiddenIssuer`], [The issuer of a hidden credential lies in an attested key set.],
    [`HolderPok` / `HolderSet`], [Hidden-holder binding tiers — explicitly _not yet sound_; opt-in only.],
  ),
  caption: [
    The #headline("zkarch.circuit_kinds") circuit kinds of the fixed family
    (#headline("zkarch.circuit_members") compiled members across their size lattices). Each kind
    realises one operator instance of the fragment (§#ref(<fragment>, supplement: none)) or one
    auxiliary statement. The circuit identifier a manifest declares is re-derived by the verifier
    and never trusted on its own (§#ref(<verify>, supplement: none)). The hidden-holder kinds are
    labelled not-yet-sound in the implementation and are gated off by default.
  ],
)

*The manifest.* A proof is a JSON _manifest_ carrying the key set, the sub-proofs (each a
length-prefixed proof, public-input segment, and verification key), the attribution set relating
result rows to source graphs, and the binding material the verifier's obligations consume. The
verifier nonce is committed as public-input field 0 of _every_ sub-proof, so a manifest is bound
to a single request. Composition across operators is expressed by _binding edges_: an edge asserts
that, e.g., the scanned column a filter constrains equals the filter's operand, chaining sub-proofs
into one statement. The binding-edge mechanism is exactly where composition attacks live, and it
was itself the subject of an internal adversarial review — the analysis in
§#ref(<security>, supplement: none) is organised around it.

= Verification: re-derive, never trust <verify>

The verifier's discipline is a single principle: _reconstruct the claimed statement from the query
text and the relying party's trust anchors, and check the cryptography against the reconstruction_
— trusting no prover-declared quantity. It runs fail-closed: the first failed check rejects the
whole manifest, with no partial results and no downgrade to a warning.

The pipeline enforces #headline("zkarch.binding_obligations") binding obligations, structured
around #headline("zkarch.audit_gates") cross-cutting _audit gates_ whose individual failure would
each void the intended soundness on its own:

#figure(
  table(
    columns: (auto, 1fr),
    align: (left, left),
    table.header[Audit gate][What the verifier does instead of trusting the manifest],
    [1 — public-input reconstruction],
    [Independently reconstructs every sub-proof's expected public-input bytes — with the verifier
     nonce at field 0 — from the declared statement and compares byte-for-byte; any difference
     rejects. Without this, an honest proof of a _different_ true statement is replayable as a
     forgery.],
    [2 — canonical verification key],
    [Recomputes each sub-proof's verification key from the canonical circuit named by the
     _re-derived_ identifier; a manifest-supplied key is never trusted. Without this, a
     prover-chosen key over an unconstrained circuit defeats the whole gate.],
    [3 — issuer signature and key set],
    [Requires every issuer key used to be a member of the _external_ key set K, and the issuer
     signature over the graph commitment to verify. Without this, the prover is the issuer of its
     own facts.],
    [4 — nonce single-use and binding],
    [Mints a fresh single-use nonce, records it as burnt _before_ the cryptographic checks, and
     rejects any manifest that binds a different nonce. Without this, an accepting manifest is
     replayable forever.],
  ),
  caption: [
    The #headline("zkarch.audit_gates") audit gates. Backend proof verification is layered
    _around_ them: verifying a proof is meaningless unless gates 1 and 2 pin _what_ is being
    verified and _against which key_. The gates are internally re-audited, _not_ externally
    audited (`sq-qhy4`, open); this table describes the mechanism, not a proven guarantee.
  ],
)

Beyond the gates, the obligation set re-derives the circuit identifier from the statement,
re-checks the cross-graph blank-node exclusion of §#ref(<fragment>, supplement: none), enforces the
attribution superset rule binding result rows to source graphs, binds filter operators/bounds and
join keys to the query text, and re-checks any declared RDFS/OWL derivation steps against
_disclosed_ bases (only simple entailment is proved in zero knowledge; an in-circuit closure proof
is deferred). Each failure maps to an explicit variant of a closed error taxonomy.

= Cost characterisation and a proving-cost model <cost>

We characterise the family's cost by its _deterministic artifact facts_: `bb gates -s ultra_honk`
circuit sizes, the ground-truth constraint count under the pinned toolchain, taken from the
regression-gated snapshot (so the numbers cannot silently drift). These are machine-_independent_
integer facts of the compiled circuits — not timings — and are the only cost numbers this paper
treats as canonical.

#figure(
  table(
    columns: (1fr, auto),
    align: (left, right),
    table.header[Family member (representative)][UltraHonk gates],
    [`RevokeUnset` (revocation, depth 10)], [#headline("zkarch.gates_revoke")],
    [`FilterValueDl` (opt-in dual-leaf integer lane)], [#headline("zkarch.gates_filter_value_dl_int")],
    [`Scan` — smallest (k=1, n=16, r=4)], [#headline("zkarch.gates_scan_min")],
    [`JoinEq` — smallest (n#sub[a]=16, n#sub[b]=16)], [#headline("zkarch.gates_join_min")],
    [`HolderPok` (hidden-holder, not-yet-sound)], [#headline("zkarch.gates_holder_pok")],
    [`HiddenIssuer` (in-circuit Schnorr + key-set membership)], [#headline("zkarch.gates_hidden_issuer")],
    [`JoinEq` — largest (n#sub[a]=64, n#sub[b]=64)], [#headline("zkarch.gates_join_max")],
    [Composable filter lane (`filter_int`/`filter_f64`/… any digit count)], [#headline("zkarch.gates_filter_lane")],
    [`Scan` — largest (k=2, n=64, r=8)], [#headline("zkarch.gates_scan_max")],
  ),
  caption: [
    Deterministic circuit sizes across the family, from the regression-gated gate-count snapshot
    (`bb gates -s ultra_honk`, toolchain pinned to `bb 5.0.0-nightly.20260324` /
    `nargo 1.0.0-beta.21`). Two facts drive the design's cost story: the string-canonical filter
    lanes are _gate-identical_ at #headline("zkarch.gates_filter_lane") regardless of digit count
    (the blake3 binding of the canonical literal token dominates and fits one hash block), and the
    scan members scale with the (k·n) commitment-recompute sweep. The opt-in dual-leaf lane
    (#headline("zkarch.gates_filter_value_dl_int")) shows the cost the blake3 binding buys — it is
    cheaper but carries an accepted invariant downgrade and is off by default.
  ],
)

#provenance("zkarch.gates_scan_max")

*A proving-cost model, and why we publish a model rather than a headline time.* UltraHonk proving
cost is, to first order, linear in the circuit's gate count for a fixed backend and thread count;
proof size and verification cost are _constant_ across the family (the succinctness property of the
scheme). So the gate-count table above _is_ the cost profile up to a single machine-dependent
constant: prove-time ≈ κ · gates for a per-host κ, with verification and proof size flat. We do
_not_ publish a headline wall-clock number because our development measurements are taken on an
AWS EC2 host whose timings are non-canonical under the project's empirical-honesty mandate — a
speed claim would require the canonical runner, which is not yet available for this family.
Relative cost within the family, however, is a property of the gate counts and _is_ canonical: a
largest-scan proof carries about #calc.round(
  headline("zkarch.gates_scan_max") / headline("zkarch.gates_scan_min"), digits: 1)× the
constraints of the smallest scan, and about #calc.round(
  headline("zkarch.gates_scan_max") / headline("zkarch.gates_filter_lane"), digits: 1)× a
composable filter lane — ratios a reader can multiply by any host's measured κ. This is the honest form of a cost
result when the absolute timer is not yet canonical: publish the constraint counts and the linear
model, and name the missing constant.

= Security analysis under an open audit gate <security>

We are precise about status because the framing is the contribution. The architecture is
_research-grade_ and has _no_ external accredited-cryptographer audit; the external audit is an
open gate (`sq-qhy4`). Nothing below is a proven guarantee.

*Threat model.* The prover is fully adversarial: it controls the manifest, the query text, the
declared circuit and public inputs, the verification key, and any replayed material, and its goal
is to make the verifier accept a false SPARQL statement, reuse a proof, or smuggle in an untrusted
issuer. The verifier is honest-but-curious for privacy and is trusted by its relying party to run
the whole obligation set. Issuer content veracity, side channels, and transport are out of scope.

*The internal adversarial audit.* An internal adversarial audit of the verifier found and
confirmed #headline("cozk.single_prover_audit_issues") issues on a v1 verifier — that v1 was
documented _not_ sound. The confirmed classes were exactly the composition-seam attacks the
architecture must defend: public inputs never reconstructed from the declared statement, a
prover-supplied verification key trusted as-is, commitments accepted without an issuer signature,
manifests infinitely replayable for want of a nonce binding, and filter operator/bound/slot never
bound to the query's FILTER. Each has a stated remediation — the four audit gates of
§#ref(<verify>, supplement: none) are, in large part, that remediation — and each finding now
carries a standing _forge-negative_ regression test:

#figure(
  table(
    columns: 2,
    align: (left, right),
    table.header[Committed structural fact][Count],
    [Confirmed findings in the internal single-prover verifier audit], [#headline("cozk.single_prover_audit_issues")],
    [Findings pinned closed by a 1:1 forge-and-verify regression test], [#headline("zkarch.forge_findings_mapped")],
    [Fail-closed binding obligations enforced per manifest], [#headline("zkarch.binding_obligations")],
    [Cross-cutting audit gates], [#headline("zkarch.audit_gates")],
  ),
  caption: [
    The audit posture as _deterministic structural facts_ (source/document scans over committed
    code, not measurements). The #headline("zkarch.forge_findings_mapped") forge-negative tests
    each construct a specific historical forgery and assert the verifier rejects it with the mapped
    error, so a future refactor cannot silently re-open a remediated finding. This is _necessary
    and explicitly not sufficient_ evidence: it pins known attacks closed; it does not discover
    unknown ones, which is the job of the open external audit.
  ],
)

#provenance("zkarch.forge_findings_mapped")

*What the absence of an external audit means.* A passing verification must _not_ be read as a
settled guarantee against an adversarial prover. The internal audit and the forge-negative suite
raise assurance and are honestly reported as such; they are not a proof of soundness, and no
accredited cryptographer has reviewed the estate. Positive security properties are therefore at
most _claimed_, with audit status "external sign-off pending". The hidden-holder tiers
(`HolderPok`, `HolderSet`) are explicitly _not yet sound_ and are off by default; the dual-leaf
value lane carries an accepted, documented invariant downgrade and is opt-in; only simple
entailment is in zero knowledge. We report each of these with equal precision to the design's
strengths, because an honest architecture paper under an open gate is honest only if it does.

*Relation to the collaborative path.* This paper is strictly single-prover. The _multi-prover_
(collaborative) extension — several holders jointly proving over their private graphs — is a
separate, _unbuilt_ path with its own open gate and its own negative-result analysis against the
collaborative-zk-SNARK failure modes of Garg et al. @garg25; it is out of scope here and
claims nothing.

= Related work

_Verifiable databases._ IntegriDB @integridb, vSQL @vsql, and ZKSQL
@zksql prove SQL query results over an outsourced database. They target a different trust
shape: the querier trusts the schema and wants integrity of an untrusted _server's_ computation;
they do not hide the data from the verifier, do not operate over RDF, and do not bind results to
issuer attestations. Our architecture hides the committed graphs and roots results in signed
issuer keys.

_Anonymous credentials._ CL signatures @cl01, BBS @bbs, and zk-creds
@zkcreds prove possession of signed attributes with selective disclosure. They prove facts
about _one credential's fields_, not _query evaluation_ over a set of credentials with joins and
filters — which is the structure this fragment adds.

_Signed RDF and semantic-web ZK._ Signing RDF graphs is classical @carroll03; the
canonicalisation this architecture relies on is the modern RDFC-1.0 @rdfc10. The most
direct neighbour proves selective-disclosure soundness of SPARQL results over RDF datasets with
zero-knowledge proofs @braun26; this paper is a companion systems description of an
engine-integrated single-prover stack — its fixed-circuit-family architecture, its verifier
re-derivation discipline, its deterministic cost profile, and its audit posture under an open gate.
We claim no cryptographic novelty over the primitives we compose; the contribution is the system
and its honest characterisation.

= Conclusion

A single holder answering a SPARQL query in zero knowledge over attested credentials needs a
machine whose every part is built for an adversarial prover: a per-graph commitment bound to an
issuer signature, a _fixed named family_ of #headline("zkarch.circuit_kinds") circuit kinds
(#headline("zkarch.circuit_members") compiled members) so the verifier can recompute keys rather
than trust them, a manifest that composes sub-proofs through checkable binding edges, and a
#headline("zkarch.binding_obligations")-obligation, #headline("zkarch.audit_gates")-audit-gate
verifier that re-derives the claimed statement from the query and the relying party's trust
anchors. We reported the family's cost as deterministic gate counts
(#headline("zkarch.gates_scan_min")–#headline("zkarch.gates_scan_max") for the scan lattice,
#headline("zkarch.gates_filter_lane") for the composable filter lanes) and a linear proving-cost
model over them, and we named the one missing piece — a canonical wall-clock constant — rather than
quote a non-canonical time. On security we reported an internal audit that found and remediated
#headline("cozk.single_prover_audit_issues") issues, each pinned closed by a standing
forge-negative test, and we stated plainly that with _no_ external audit (`sq-qhy4`) the design
claims no proven property. The architecture is what it is: a carefully-composed, internally
re-audited, not-yet-externally-audited system, described so that both its design and its open gate
are legible.

#heading(level: 2, numbering: none)[References]
#bibliography("zksparql-architecture.refs.yml", style: "ieee", title: none)

#if not anon [
  #line(length: 100%)
  #text(size: 0.8em, fill: gray)[
    sparq project. This paper is a systems-and-design contribution under the OPEN external-audit
    gate `sq-qhy4`; it asserts _no_ proven security, soundness, privacy, or attestation property.
    Evidence traces to the fixed circuit family `zk/compose/`, the regression-gated gate-count
    snapshot `crates/sparq-zk-compose/tests/gate_count_snapshot.json`, the verifier
    `crates/sparq-zk-compose/src/verifier.rs`, the internal audit `research/zk-soundness-audit.md`,
    and the forge-negative regression map `crates/sparq-zk-compose/tests/audit_forge_map.rs`.
    Numbers are injected at build time from the paper-bound evidence file; see the provenance stamp
    on the published page.
  ]
]
