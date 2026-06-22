<!-- [OPUS-4.8] sq-toze — memsafety AUDIT round 2 (sign-off). Re-review when Fable returns. -->

# Memory-safety attestation — audit findings, round 2

Re-audit of the three round-1 findings after the engineer's fixes.

## F1 — MEDIUM — register clippy overclaim → **RESOLVED**
`grep -n undocumented_unsafe_blocks compliance/memsafety/unsafe-register.md` → line 45 now
reads enforcement is "the **register + review + the count ratchet**" and explicitly states
`clippy::undocumented_unsafe_blocks` is "**recommended but NOT yet enabled**", linking gap
MS-G2. The overclaim is gone; the authoritative register is now consistent with
`controls.md`/`gap-register.md`. **Closed.**

## F2 — LOW — absolute "every site has a `// SAFETY:` comment" → **RESOLVED**
Register §NEEDS-REVIEW now reads "50 via the literal `// SAFETY:` token, and 6 via an
adjacent justification block comment", naming the 6 sites. The phrasing is now literally
true. **Closed.**

## F3 — LOW — threat-model 39 vs register 42 → **ACCEPTED as tracked gap MS-G5**
The discrepancy is flagged in `evidence.md` (§Verified-but-noted) and `gap-register.md`
(MS-G5, with the one-line remediation against `research/threat-model.md`, a doc this
framework does not own). Per round-1 reasoning this is acceptable to carry as a tracked
cross-doc gap rather than a memsafety-deliverable defect. **No open finding.**

## Verdict

`FINDINGS: 0` — **SIGN-OFF.**

The memory-safety attestation is sound and honestly evidenced:
- Unsafe surface confined to 5 crates; 20 crates `#![forbid(unsafe_code)]` (verified).
- All 56 first-party `unsafe` sites enumerated + justified in the register; count
  triangulated across register / `bench/unsafe-snapshot.json` / `scripts/unsafe-gate.py`.
- A genuinely merge-blocking unsafe-count ratchet (verified against the `ci-summary`
  aggregator's own gating rule + the `ci.yml` PR/merge_group triggers).
- B5 mmap boundary covered by the deterministic corruption oracle + the `graph_open` fuzz
  target running under AddressSanitizer; Miri honestly scoped to the pure-Rust sites it can
  reach, with the structural mmap limitation documented, not hidden.
- No claim contradicts the v1-ZK-NOT-sound verdict; the `sparq-zk-compose` flock sites are
  correctly scoped as memory-safety FFI only.

**Standing external caveats (out of agent scope, correctly labelled AUDIT-READY):** formal
verification / model checking of the mmap validators (MS-G4) and an accredited third-party
memory-safety audit remain external by definition. The two deferred deeper-assurance lanes
(MS-G2 clippy-token lint, MS-G3 standalone-ASan lane) are tracked, not papered over.

Overall status: **PASS** on every applicable codebase/CI control, with 4 honestly-recorded
OPEN gaps (1 medium-now-resolved-in-doc + tracked-for-code, 3 low) and the formal-proof /
external-audit ceiling labelled external. This is a defensible attestation, not a
rubber-stamp.
