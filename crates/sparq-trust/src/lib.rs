//! # sparq-trust — trust-graph authorisation admission gate (research PoC)
//!
//! The proof-of-concept for the trust-graph authorisation system (design record
//! `research/solid-trust-graph-authz-design.md` §6.0; issue #940, epic `sq-pfae`).
//! It adds the **admission stratum** ahead of the shipped `sparq-solid` WAC/ACP
//! derivation stratum: *"is this externally-attested fact from a source I trust for
//! this statement-type?"* — and, on success, injects the issuer-tagged fact into the
//! materialiser so the existing N3 reasoner can merge it with the `.acr` rules and
//! derive access.
//!
//! ```text
//!   attested statements (VC claims, signed graphs)
//!                 │
//!                 ▼
//!   ADMISSION stratum  (this crate, NEW, OPT-IN)
//!   gate: trust statement + verified issuer signature + freshness + holder binding
//!                 │  admitted facts (issuer-tagged, trust:admitted)
//!                 ▼
//!   DERIVATION stratum  (shipped sparq-solid WAC/ACP N3 rules + ABAC rule)
//!                 │  ".acr rules over the admitted facts ⇒ canAccess"
//!                 ▼
//!        <urn:sparq:auth>  (allow-list, fail-closed)
//! ```
//!
//! ## Modules
//!
//! - [`vocab`] — the `trust:` IRIs of the 10-term minimal core, plus the one
//!   `forPredicate → forShape` desugaring (sugar, not a primitive).
//! - [`policy`] — parse a Control-gated trust-policy graph into [`policy::TrustRule`]s
//!   (fail-closed: a policy not Control-gated admits nothing).
//! - [`mod@admit`] — the admission gate: canonicalise (RDFC-1.0), verify the issuer
//!   signature, enforce statement-type scoping (SHACL shape), freshness, and holder
//!   binding; output [`admit::AdmittedFact`]s. [`admit::admit_static`] runs only the
//!   session-independent (STATIC) class and defers holder/freshness to a per-request
//!   conditional-grant check — the `sq-xc4y` static/dynamic split.
//! - [`wire`] — feed admitted facts into the N3 reasoner ahead of the materialiser;
//!   read off the derived `auth:*` grants ([`wire::derive_grants`]) OR the per-grant
//!   DYNAMIC conditions for a conditional grant ([`wire::derive_conditional_grants`]).
//!
//! ## Opt-in by construction (strict additivity — design §2.2 G6)
//!
//! Nothing in the workspace's default build depends on this crate. `sparq-solid`
//! pulls it in ONLY behind its **default-OFF `trust-graph` cargo feature**; with the
//! feature off, `sparq-solid` behaves **exactly** as WAC/ACP do today — the only edge
//! it gains is one feature-gated call into this admission gate before the
//! materialiser. A pod with no trust graph is byte-identical to today's Solid.
//!
//! ## Honest scope — RESEARCH PROTOTYPE, NOT a security guarantee (read first)
//!
//! This is a research prototype to take to the LWS / Solid working groups, **not a
//! shipped feature and not a security guarantee**. It demonstrates exactly one
//! claim: *a single externally-attested fact is admitted through a
//! trusted-source-scoped gate, then merged by N3 reasoning with an `.acr` rule to
//! derive `canAccess`.* It proves nothing stronger. In particular:
//!
//! - **No privacy / unlinkability / anonymity.** The gate verifies a CHECKED issuer
//!   signature, but the credential is admitted **in the clear** — the verifier learns
//!   the exact value (`age 25`, not just "≥ 18"). It does **NOT** deliver ZKAPs-grade
//!   unlinkable/anonymous presentation. The ZK estate it composes with
//!   (`sparq-zk` / `sparq-zk-compose`) is research-grade and **externally UNAUDITED**
//!   — external accredited-cryptographer sign-off is **pending** (`sq-qhy4`), and
//!   `sparq-mpc` is honest-majority semi-honest only.
//! - **Holder binding is the clear-WebID, non-anonymous DEGRADED path.** §3.4's
//!   holder binding (`credentialSubject == Session.agent`) authenticates the
//!   requester's WebID **in the clear**. This is the documented degraded path
//!   (`sq-wvne` / `sq-xc4y`), **not** silently "solved": presentations stay linkable
//!   by requester identity. A ZKAPs-equivalent presentation would have to REPLACE
//!   clear-WebID holder binding with an in-ZK holder-PoP plus a nullifier — an
//!   architecture change, not a composition step.
//! - **Issuer keys are operator-asserted.** sparq has no DID resolver yet
//!   (`sq-pfae.3`), so the `trust:issuerKey → verifying-key` binding is supplied
//!   directly rather than resolver-verified — the live forgery vector D′ (§3.3). This
//!   is honest, but **not** an end-to-end trust path.
//!
//! ## Open problems this PoC RESPECTS as documented limitations (never silently solved)
//!
//! - **`sq-xc4y`** — per-request holder-binding / freshness vs the
//!   session-independent materialise-once auth view: RESOLVED by the **static/dynamic
//!   admission split** (decision (a)). The static class (signature, statement-type
//!   scope, reserved-predicate guard, `trust:scope`) is session-independent and runs
//!   ONCE at materialise-time ([`admit::admit_static`]); the dynamic class (holder
//!   binding + freshness) is deferred to a **per-request conditional grant** re-checked
//!   at query time via the shipped sq-0q7n `auth:agent` / `auth:notAfter` path — so a
//!   stale or wrong-holder credential is denied per request WITHOUT a re-materialise
//!   and is never frozen into the view. (The combined [`admit::admit`] remains the
//!   single-request snapshot path.)
//! - **`sq-l5og`** — delegation invocation-binding: **out of PoC scope** (no
//!   delegation substrate exists; §4 of the design is design-only).
//! - **`sq-tu4e`** — no in-reasoner NAF over derived facts: `revoked` is an
//!   **input-only** per-request guard; there is **no deny-on-disagreement** rule.
//! - **`sq-wvne`** — ZKAPs-grade unlinkable presentation needs a 3-part ZK composite:
//!   **out of PoC scope** (no privacy/ZK).
//!
//! [OPUS-4.8] sq-pfae PoC (issue #940). Written while Fable unavailable — flag for
//! re-review when Fable returns. 🤖 SPARQ agent — trust-graph authorisation PoC.

#![forbid(unsafe_code)]
#![cfg_attr(docsrs, feature(doc_cfg))]

pub mod admit;
pub mod policy;
pub mod vocab;
pub mod wire;

pub use admit::{
    admit, admit_static, AdmittedFact, PresentedCredential, Session, StaticAdmittedFact,
};
pub use policy::{parse_policy, ControlGate, PolicyError, ShapeRef, TrustRule};
pub use wire::{derive_conditional_grants, derive_grants, ConditionalGrant};
