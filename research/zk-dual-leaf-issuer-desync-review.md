<!-- [OPUS-4.8] Adversarial review authored by Opus 4.8 (1M context) (Fable unavailable) — re-review when Fable returns. -->
# Adversarial review: malicious-issuer value↔lexical DESYNC in the dual-leaf ZK encoding

> 🤖 SPARQL/ZK research agent record. This is a **design-for-review** adversarial
> lens report on the proposed **dual-leaf** ZK term encoding (issue **#769**, the
> design draft extending PR #765's value-only `VALUE_HOOK`). It changes no
> `.nr` / `.rs` source and creates no bead — the orchestrator owns bead structure
> (recommended children in §8). Parent epic **sq-1s2**; external sign-off
> **sq-qhy4** is the gate for any production reliance.

## 0. Verdict (read first)

**Through the issuer-desync lens the design's CENTRAL THREAT-MODEL CLAIM does NOT
hold.** The draft's load-bearing assertion — that value↔lexical desync gives a
malicious issuer *no new value-lie capability* because "a malicious issuer can
already lie about the value directly" (draft §5.2, §12) — is **false against the
actual current circuit**. Today the value-FILTER verdict is **sound against a
malicious issuer** for value↔lexical consistency, and the dual-leaf **removes**
that existing soundness property. This is not a "new bounded residual within the
existing trust boundary"; it is a **regression of the trust model** for the one
operation the whole change exists to optimise.

The structural claim that in-circuit binding cannot be added without re-introducing
the lexical parse **does hold** — and that is precisely what makes the finding
serious: the gate win is *bought by giving up an in-circuit soundness check that
currently holds for free against malicious issuers*.

Severity: **HIGH**. The fix is not "register an audit obligation"; it is to
**correct the draft's threat-model framing** (it understates the loss), keep the
design only under the explicit **honest-issuer-for-value** model, and add the
**mandatory same-row value↔lexical co-binding** (§6) that bounds the regression to
exactly that model — plus the canonical-issuance exit (#769's own long-term idea)
as the real mitigation.

## 1. What the current circuit ACTUALLY enforces (verified against the checkout)

This is the fact the draft mis-states. Read `filter_int.nr` and
`filter_signed.nr` carefully:

- `filter_int.nr:67-69` — the compared `value` is computed **from the witnessed
  `digits`**:

  ```text
  let mut value: u64 = 0;
  for i in 0..D { value = value * 10 + ((digits[i] - 48) as u64); }
  ```

- `filter_int.nr:72-92` — the **same** `digits` are rebuilt into the canonical
  N-Triples token, blake3-hashed, truncated, and asserted to equal the committed
  leaf: `assert_eq(h2(TYPE_CODE_LITERAL, hs), operand_enc, ...)`.

So in the current single-leaf design **the compared value and the committed
lexical form are derived from ONE witness (`digits`)**. There is no second,
independent value handle. The circuit therefore enforces, *as a hard constraint
inside the proof*, that **the value being compared is `parse(lexical)` of the
committed literal** — and `operand_enc` is the scan-anchored, issuer-signed leaf
(`scan.nr:101-107`, `verifier.rs:2063-2087` binding edge,
`verifier.rs:2210+` issuer Schnorr over `C(G)`).

`filter_signed.nr` is identical in structure: `mag` is built from `mag_digits`
(`:150-153`), the **same** `mag_digits` are rebuilt into the token and bound
(`:160-180`, via `assert_literal_binding` `:93-115`), and the verdict is over that
`mag` (`:182-183`). `filter_decimal` likewise. `filter_f64`'s composable form
(`filter_float.nr` header + `:50+`) derives the IEEE bits **from the bound
canonical digits**, same discipline.

**Consequence — the property the draft misses:** *today, even a fully malicious
trusted issuer cannot produce a leaf that compares as 18 under a value-FILTER while
its lexical/term-identity is "5".* If the issuer commits `lexical="5"`, the FILTER
member can only satisfy the binding with `digits = "5"`, and then `value = 5`. The
only "lie" a malicious issuer has today is to commit `lexical="18"` for a 5-year-old
— and then **the term identity is also "18"**: value and lexical AGREE. There is no
desync state reachable today.

This is a real, currently-enforced soundness invariant:

> **INV-VL (today):** for any literal consumed by a value-FILTER, the compared
> numeric value equals `parse(committed_lexical)`, enforced in-circuit, against an
> arbitrary (incl. malicious) committer.

## 2. The break: the dual-leaf DELETES INV-VL

The dual-leaf FILTER member (draft §4) is, verbatim:

```text
inner = Poseidon2([VALUE_HOOK, DATATYPE_CONST, LANG_CONST], 3)
leaf  = Poseidon2([inner, lexical_component_witness, TYPE_CODE_LITERAL], 3)
assert_eq(leaf, operand_enc)
// re-derive VALUE_HOOK into u64/sign/...; compare with bound; assert verdict
```

Here `VALUE_HOOK` and `lexical_component_witness` are **two independent witnesses**.
The binding asserts the leaf contains *both*, but **nothing constrains
`VALUE_HOOK == parse(preimage(lexical_component))`** — the draft says exactly this
in §5.1 and is correct that it cannot be added in-circuit without re-deriving the
value from the lexical bytes (which is the blake3+parse the change removes). The
draft's §5.1 structural claim **holds**.

But the draft then draws the WRONG threat-model conclusion. The dual-leaf does not
merely "rest on issuer honesty for a NEW property the system never had." It
**destroys INV-VL**, which the system *did* have, against malicious issuers. The
verdict's soundness for the value lane changes from:

- **today:** "value = parse(lexical) of the issuer-signed leaf — true even if the
  issuer is malicious" → to →
- **dual-leaf:** "value = whatever VALUE_HOOK the issuer committed, which equals
  parse(lexical) **only if the issuer was honest at commit time**."

That is a strict trust-model regression *for the value-FILTER lane specifically*.

### 2.1 Why "the issuer could already lie about the value" is a false equivalence

The draft equates two different lies:

1. **Honest-but-wrong / lying issuer, value and lexical AGREE** (commit
   `lexical="18"` for a child). Possible today and after. The relying party trusts
   the issuer for the *truth of the asserted value*; a dishonest authority defeats
   that, and there is nothing any encoding can do about it. **Not the issue.**

2. **Desync: value and lexical DISAGREE** (commit `VALUE_HOOK=18`, `lexical="5"`).
   **Impossible today** (INV-VL forbids it), **possible under the dual-leaf.** The
   same signed credential now answers a value question as 18 and a
   term-identity/`sameTerm`/`DISTINCT`/join question as 5.

The draft collapses (1) and (2) and concludes "no new capability." But (2) is
genuinely new, and it is not equivalent to (1): it lets a *single signed credential*
exhibit a state that **no honest issuance can ever produce and that the current
circuit structurally forbids regardless of issuer honesty.** That is the new power.

## 3. Where the disagreement bites — and the draft's confinement is too narrow

The draft (§5.2, §12) confines the residual to "a verifier consuming BOTH a
value-FILTER AND an identity op over the SAME hidden column from a SINGLE
credential." Two problems with that confinement:

### 3.1 It is not confined to mixed queries — the value lane ALONE is now weaker

Even a **value-FILTER-only** query is affected. Consider an access-control verifier
that today relies on INV-VL: it accepts "this credential's `:age` is `>= 18`" and,
separately, displays / logs / re-presents the credential's lexical age elsewhere in
the pipeline (a disclosed row, an audit log, a downstream non-ZK consumer reading
the same committed graph). Today those two views are *guaranteed consistent* by
INV-VL. Under the dual-leaf a malicious issuer can make the ZK verdict say "adult"
while every lexical/disclosed view says "5". The draft frames this as needing a
*single proof* mixing two in-circuit ops; in reality the lexical side can be
consumed **anywhere in the system** (disclosed scan rows are PUBLIC —
`scan.nr:88` `rows`, surfaced via `verifier.rs:2063-2067`), not just inside a second
in-circuit identity gadget. The attack surface is "any consumer that trusts the
lexical/term side to agree with a value-FILTER verdict," which is broader than the
draft's "mixed single-credential query."

### 3.2 The join lane (`join_eq`) makes the inconsistency directly exploitable

`join.nr:170-177` joins on the **full leaf** (`select_slot` over a row of `enc`,
then `assert(a_val == b_val)`). The draft is right that join uses lexical identity.
But combine with §3.1: a malicious issuer issues credential A with
`(VALUE_HOOK=18, lexical="5")`. Credential B (a different, honestly-issued record
keyed on the lexical "5") joins to A on the term "5" (`join_eq` matches the leaves
because both carry `lexical=hash("5")` — note: the leaves only match if A's *full
leaf* equals B's, i.e. the join still requires lexical-leaf equality, which holds by
construction here). The prover then *also* runs a value-FILTER on A's column
proving `>= 18`. Result: a proof that "the entity that joins to the '5'-keyed record
is an adult." No honest issuance can produce this; INV-VL forbids it today. This is
the draft's "mixed query" locus — but it is reachable with **one malicious issuer
and one honest issuer**, not "a malicious issuer + a self-contained mixed query,"
and it produces a concretely misleading cross-credential claim.

### 3.3 The "no untrusted party can exploit it" claim is correct but not exculpatory

The draft leans on "no *untrusted* party can forge anything; the scan/issuer chain
still binds the leaf to a trusted signature." True — `verifier.rs:2210+` still
requires an external-`K` Schnorr signature over `C(G)`. So the desync requires a
*trusted* issuer to be malicious (or coerced/compromised). But this is the standard
reason the conclusion should be **"the value lane is now only as sound as the
issuer's honesty, whereas it used to be sound unconditionally"** — not "the residual
is negligible." Many real trust models trust an issuer for *identity attestation*
(it correctly says "this DID is this person") while NOT fully trusting it for
*value semantics*, or trust a large issuer's key but worry about compromise/insider
risk. INV-VL gave defence-in-depth against exactly that: a compromised issuer key
could still not desync value from lexical. The dual-leaf surrenders that
defence-in-depth.

## 4. Is the in-circuit binding genuinely impossible? (confirmed — and that's the point)

Yes. To bind `VALUE_HOOK == parse(lexical)` in-circuit you must witness the lexical
preimage bytes and parse them into the numeric domain — which is **exactly**
`filter_int.nr:57-92` / `filter_signed.nr:142-180`: the digit-range asserts, the
base-10 fold, the canonical-form asserts, the token rebuild, and the blake3. That
*is* the ~14,300-gate blake3-bound member the value handle exists to delete. So:

> **CONFIRMED:** in-circuit value↔lexical consistency is unenforceable *given the
> gate goal* — enforcing it re-introduces the lexical parse and erases the saving.

The draft's §5.1 is correct on this. The honest framing the draft *should* have
drawn from it: **the gate win is not free; its price is INV-VL.** The draft instead
presents the price as "a bounded residual within the existing honest-issuer
boundary," which understates it because the existing boundary did **not** include
trusting the issuer for value↔lexical agreement — the circuit enforced it.

## 5. Counter-arguments I tried, and why they do not save the framing

- *"The relying party already trusts the issuer for the value, so trusting
  value↔lexical agreement is free."* — Rejected. INV-VL means the value-FILTER
  verdict today is NOT a pure trust-the-issuer statement: it is "the issuer signed a
  leaf whose lexical form parses to a value satisfying the predicate." The lexical
  form is the issuer's attestation; the *value used in the comparison is forced to
  be that lexical form's value by the circuit*. The relying party trusts the issuer
  for the lexical attestation only; the value-consistency was machine-enforced. The
  dual-leaf moves value-consistency from machine-enforced to issuer-trusted.

- *"B4 / canonical-form binds cover this."* — Rejected, and the draft itself
  separates them correctly (§8 B4). B4 stops a *prover* binding a second VALUE_HOOK
  encoding of the same value (canonical scale / IEEE bits / no `-0`). It says nothing
  about whether VALUE_HOOK equals the lexical. B4 is prover-side; INV-VL was the
  value↔lexical tie. Losing INV-VL is orthogonal to keeping B4.

- *"Identity ops still use the full leaf, so term identity is preserved."* — True
  and good (the reject-list item (v) is correct and necessary). But term-identity
  preservation is exactly what makes the desync *observable* and exploitable (§3.2):
  the two components disagree precisely because identity stays on lexical while the
  FILTER reads value. Preserving term identity does not mitigate the desync; it is
  what surfaces it.

- *"It's research-grade / unaudited anyway, so any gap is just an audit
  obligation."* — Partially. But there is a difference between "an unproven new
  property the auditor must check" and "a deliberately-accepted REMOVAL of a property
  the current circuit enforces." The latter must be surfaced as a **design decision
  with a named trust-model cost**, not folded into a HIGH audit obligation whose
  prose says "within the existing honest-issuer trust boundary." That prose is
  inaccurate and should be corrected before sq-qhy4 sees it, or the auditor is primed
  with a wrong premise.

## 6. Required design fix (bounds the regression to exactly the honest-issuer-for-value model)

The dual-leaf is still the right realisation of #769's decision (it is the only way
to keep term identity for externally-committed graphs AND get the value handle). The
fix is not to abandon it but to **stop understating the cost and to add a
host-side + audit-side co-binding** so the regression is exactly "value lane trusts
the issuer for value↔lexical agreement," no broader:

1. **Correct the threat-model framing (MANDATORY, doc-only, blocks the draft).**
   Replace the draft's "no new value-lie capability / bounded residual within the
   existing boundary" with: *"the dual-leaf REMOVES the in-circuit invariant INV-VL
   that the current `filter_int`/`signed`/`decimal`/`float` members enforce against
   arbitrary committers (value = parse(committed lexical)); after the change the
   value-FILTER lane is sound only under an explicit honest-issuer-for-value
   assumption, a strict trust-model regression for that lane, accepted as the price
   of the gate win."* This is the load-bearing correction.

2. **Host-side same-leaf co-binding at commit (`encode.rs`/`commit.rs`).** At ingest
   the host MUST compute `VALUE_HOOK = parse(canonical(lexical))` from the SAME
   bytes it hashes into `lexical_component`, and MUST refuse to commit a literal
   whose `VALUE_HOOK` does not match its lexical form (fail-closed at ingest). This
   does NOT defend against a malicious issuer running a patched committer (it
   cannot — that's INV-VL's irreducible loss), but it (a) guarantees *honest* sparq
   ingest never produces a desynced leaf, and (b) makes desync a detectable
   protocol violation if the lexical preimage is ever disclosed. Make this an
   asserted, tested ingest invariant.

3. **Register the REMOVAL, not a vague residual (CR-G8 wording fix).** The
   gap-register entry and SKILL/README caveats must state that an existing in-circuit
   value↔lexical consistency check is being REMOVED and replaced by an issuer-honesty
   assumption — phrased so `scripts/check-privacy-claims.sh` still passes
   (obligation/negation framing), but *accurate* about the regression. The draft's
   §9 wording ("no in-circuit check that the value matches the lexical form — rests
   on issuer honesty") is acceptable as far as it goes but omits "this replaces an
   invariant the current circuit enforces against malicious issuers." Add that
   clause.

4. **Canonical-issuance exit is the REAL mitigation, not a footnote.** #769's own
   "force issuers to issue canonical-form values" is the only thing that *recovers*
   value↔lexical agreement as an issuance invariant. The draft files it as a future
   option (§7). Given that the dual-leaf surrenders INV-VL, canonical-issuance
   conformance should be elevated from "documented future option" to a **named
   precondition for relying on the value-FILTER lane in any adversarial-issuer
   setting** — i.e. the value lane is honest-issuer-only until canonical-issuance
   conformance exists.

5. **Identity-op regression guard (already in the draft's §11 bead 4) — keep, and
   ADD a desync-detection test.** Beyond proving `value_component` is never consulted
   by identity ops, add a host-level test that a desynced leaf (VALUE_HOOK=18,
   lexical="5") is REJECTED at ingest by fix (2), and a doc-test asserting the
   verifier cannot detect desync once committed (documenting the irreducible loss
   honestly).

## 7. What the draft got RIGHT (so the maintainer can weigh it)

- §5.1 structural impossibility of in-circuit binding — **correct and confirmed**.
- Reject-list (v) (identity ops must never consult `value_component`) — **correct
  and necessary**; without it the dual-leaf would also corrupt term identity.
- The full-leaf-equality observation for scan/join (`scan.nr:143-145`,
  `join.nr:112/170-177`) — **verified accurate**; identity ops need no mechanism
  change.
- The gate-win arithmetic shape (two Poseidon2 perms replace one in-circuit blake3)
  and the "ESTIMATE bracketed by measured anchors, NOT a claim, re-measure with
  `bb gates`" framing — **appropriately caveated**; not a fabricated number.
- The honest-issuer canonical-issuance exit as the long-term close — **correct
  direction**, just under-weighted (see fix 4).

The single substantive error is the **threat-model conclusion**: presenting the
loss of INV-VL as "no new capability / bounded residual within the existing
boundary" rather than "removal of an existing in-circuit invariant; value-lane
trust-model regression."

## 8. Recommended beads (orchestrator to create — ordered; children of sq-1s2)

These ADD to / amend the draft's §11 list; they do not replace it.

1. **Amend the dual-leaf draft's §5 / §12 + CR-G8 wording** to state the INV-VL
   removal explicitly (fix 1 + 3). Doc-only; land before sq-qhy4 reviews the draft so
   the auditor is not primed with the understated framing. No code.
2. **Host same-leaf co-binding at ingest** (fix 2): `encode.rs`/`commit.rs` compute
   and assert `VALUE_HOOK == parse(canonical(lexical))`, fail-closed; tested. Folds
   into the draft's §11 bead 2 (host encoding overhaul) as a hard requirement.
3. **Desync-detection + irreducibility tests** (fix 5): ingest rejects a desynced
   leaf; documented doc-test that the verifier cannot detect post-commit desync.
   Depends on 2.
4. **Elevate canonical-issuance conformance to a named value-lane precondition**
   (fix 4): the value-FILTER lane is honest-issuer-only until a canonical-issuance
   conformance mechanism exists; scope that mechanism. Depends on the draft's §11
   bead 5. Audit-gated.

Ordering: **1** (correct the framing first) → **2** → **3** → **4**. All
audit-gated behind **sq-qhy4** for any production reliance, consistent with the ZK
estate; implementable at research grade before sign-off.

## 9. Open questions for the maintainer

1. **Is the value-FILTER lane acceptable as honest-issuer-only?** The dual-leaf
   surrenders INV-VL; the value verdict becomes sound only if the issuer is honest
   about value↔lexical agreement. Today it is sound against malicious issuers. Is
   that regression acceptable for the target deployments, or should the value lane be
   gated on canonical-issuance conformance from the start (fix 4)?
2. **Should ingest hard-fail on desynced literals (fix 2)?** This makes honest sparq
   ingest desync-free and desync a detectable protocol violation, at the cost of an
   ingest-time parse+compare per hookable literal (off-circuit Rust; measurable).
3. **Threat-model scope:** are there deployments that trust an issuer for *identity*
   but not for *value*, or that worry about issuer-key compromise/insider risk? Those
   are exactly the models INV-VL protected and the dual-leaf weakens.

## 10. Honesty framing (load-bearing)

Nothing here is a security guarantee. sparq's v1 ZK verifier is remediated and
internally re-audited but **NOT externally audited**, documented **NOT-yet-sound**
for production reliance (sq-qhy4, sq-9hrn, sq-1s2; `SECURITY.md`;
`compliance/cryptoreview/gap-register.md` CR-G1). External accredited-cryptographer
sign-off (sq-qhy4, P0) is REQUIRED before any ZK soundness/privacy/integrity
property may be relied upon. The MPC estate is semi-honest-only and not invoked
here. All gate counts referenced are the measured `bb gates -s ultra_honk`
`circuit_size` snapshots; every projected figure is an estimate bracketed by
measured anchors and is NOT a claim until re-measured. EC2 / work-box timings are
NON-canonical and none appear here. The INV-VL invariant, the desync analysis, and
the mitigations above are an **adversarial design review** and an **open external-
audit obligation** (sq-qhy4) — not established properties.
