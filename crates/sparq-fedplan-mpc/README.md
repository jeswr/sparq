<!-- [OPUS-4.8] sq-2q1x. Phase 1 skeleton of the untrusted-planner -> MPC-routing seam. -->

# sparq-fedplan-mpc

**Opt-in glue** between cost-based federated source selection (`sparq-fedplan`) and
MPC-over-federated-SPARQL routing (`sparq-mpc`). This crate turns a federated query plus a
per-source *privacy descriptor* into the typed plan that the existing `sparq-mpc` pipeline
consumes — **without coupling** the two upstream crates to each other.

It implements the design in
[`research/mpc-untrusted-planner-routing-design.md`](../../research/mpc-untrusted-planner-routing-design.md)
(epics `sq-pwr` / `sq-0jsc`).

## What this bead delivers (Phase 1 — skeleton only)

`sq-2q1x` lands the **seam scaffold**, not the protocol:

- **`SourcePrivacyDescriptor`** — the per-source privacy declaration the (later) routing pass
  reads: which predicates a source is willing to disclose in the clear, the opaque
  attestation-key id its graph is signed under (an opaque label only — no signature is checked
  here), and a reserved participation/authorisation field (the deferred B7 hook). Its posture
  is **default-deny** (see below).
- **Typed deferred-phase stubs** — `select_private_sources`, `route_operators`, and
  `assemble_leakage_envelope` have their real input/output signatures wired to the actual
  `sparq-fedplan` / `sparq-mpc` types, but each returns `Err(SeamError::Deferred { phase, gated_on })`
  naming the phase and the gate it waits on. They compile and are callable; there is no
  `todo!()` / `unimplemented!()` and they never panic — a caller gets an honest typed
  "deferred" error, not a crash and not a fabricated result.

## Default-deny (the load-bearing posture)

The whole point of the seam is to **minimise** what leaves a source in the clear, so the safe
default treats **every predicate as private** — disclosable only when the source has explicitly
opted it in:

- `SourcePrivacyDescriptor::deny_all(id)` discloses nothing.
- `SourcePrivacyDescriptor::may_disclose(predicate)` returns `true` **only** when the source
  explicitly marked `predicate` `Disclosability::Public`; `false` for every other predicate,
  including any never named.
- A planner cannot widen this. A later phase *reads* the descriptor, but the source re-enforces
  it fail-closed (the design record's constraint C-B), so an over-disclosing plan is intended
  to be rejected, not honoured. That fail-closed enforcement is itself a deferred phase (below).

## Opt-in (hard constraint)

The whole surface is behind the **`fedplan-mpc` cargo feature, OFF by default**, and the crate
is a standalone `publish = false` workspace member:

- `sparq-core` / `sparq-engine` never depend on it, so the default engine build and the WASM
  artifact are byte-identical with or without it.
- A build that does not enable `fedplan-mpc` compiles an **empty crate** and pulls in neither
  `sparq-fedplan` nor `sparq-mpc`.
- Enabling the feature pulls in both upstreams **here only** — neither upstream gains a
  cross-dependency on the other.

```toml
# Off by default; enable the seam surface:
sparq-fedplan-mpc = { path = "...", features = ["fedplan-mpc"] }
```

## Honest boundary — what this crate does NOT do

This is a **skeleton**. It performs **no** MPC, **no** secret-sharing, and runs **no**
privacy-bearing logic. It makes **no** soundness, privacy, or security claim.

- The MPC estate (`sparq-mpc`) is **research-grade and NOT externally audited**: it is
  honest-majority, semi-honest only. The external accredited-cryptographer sign-off (`sq-qhy4`)
  and the collaborative-coZK re-audit (`sq-9hrn`) are **pending**.
- The **privacy-bearing phases are deferred and audit-gated**: privacy-aware
  source-selection pruning (Phase 2), the disclosed/hidden routing decision (Phase 3), the
  leakage-envelope assembly + holder/verifier dual ratification (Phase 4), and the
  authenticated-input attestation (later). Until those land and the audits clear, nothing here
  should be presented as offering a privacy or soundness property — the typed stubs return a
  "deferred" error precisely so no caller mistakes the scaffold for a working protocol.

See `SECURITY.md` and the design record for the full caveat. The phased plan (Phases 2–7) is in
the design record's §8.

## Tested invariants (Phase 1)

- **Default-deny**: a bare / `deny_all` descriptor discloses nothing; only an explicit
  `Disclosability::Public` mark enables disclosure for that one predicate; everything else stays
  private.
- **Deterministic surface**: `public_predicates()` iterates in sorted order.
- **Honest deferral**: each deferred-phase stub returns `SeamError::Deferred` naming its phase
  and gate — never a panic, never a fabricated result.

## Gates

```sh
# Default (feature OFF): empty crate, pulls in no upstream.
cargo clippy -p sparq-fedplan-mpc --all-targets -- -D warnings
cargo test   -p sparq-fedplan-mpc

# Feature ON: the seam surface.
cargo clippy -p sparq-fedplan-mpc --all-targets --features fedplan-mpc -- -D warnings
cargo test   -p sparq-fedplan-mpc --features fedplan-mpc
```
