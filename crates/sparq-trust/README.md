<!-- [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC. -->
# sparq-trust

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **proof-of-concept** for trust-graph authorisation over [sparq-solid](../sparq-solid/README.md)
(design record §6.0; [issue #940](https://github.com/jeswr/sparq/issues/940); epic `sq-pfae`). It
adds the **admission stratum** ahead of the shipped WAC/ACP derivation stratum: *"is this
externally-attested fact from a source I trust for this statement-type?"* — and on success injects
the issuer-tagged fact so the existing N3 reasoner merges it with the `.acr` rules to derive access.

> **RESEARCH PROTOTYPE — NOT a shipped feature and NOT a security guarantee.** It demonstrates
> exactly one claim: a single externally-attested fact is admitted through a
> trusted-source-scoped gate, then merged by N3 reasoning with an `.acr` rule to derive
> `canAccess`. It does **not** provide privacy, unlinkability, or anonymity (see *Honest scope*).
> The ZK estate it composes with is externally **unaudited** (`sq-qhy4`), pending
> accredited-cryptographer sign-off.

<!-- separate blockquote (MD028): the model-provenance note is a distinct callout. [OPUS-4.8] -->

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).

## 🚀 Quickstart

`sparq-trust` is **opt-in**: nothing in the default build depends on it; the gate is wired into `sparq-solid` behind its default-OFF `trust-graph` cargo feature.

```rust
use sparq_trust::admit::{admit, PresentedCredential, Session};
use sparq_trust::policy::{parse_policy, ControlGate};
use oxrdf::NamedNode;

# fn demo(cred: &PresentedCredential, policy: &[oxrdf::Triple]) {
// Parse a Control-gated trust policy → rules (fail-closed: an un-gated policy admits nothing).
let rules = parse_policy(policy, ControlGate::assert_control_gated()).unwrap();

// The admission gate: RDFC-1.0 canonicalise → CHECKED issuer signature → SHACL statement-type
// scoping → freshness → clear-WebID holder binding. Output: the issuer-tagged admitted facts.
let session = Session { agent: NamedNode::new("https://jesse.ex/card#me").unwrap(), now_unix_secs: 1_700_000_000 };
let target = NamedNode::new("https://pod.ex/resourceX").unwrap();
let admitted = admit(cred, &rules, &session, &target);
# let _ = admitted;
# }
```

From `sparq-solid` (`--features trust-graph`), `PodStore::admit_trust_credential_with_rule` runs the
gate and installs the derived `auth:*` grants into `<urn:sparq:auth>` on top of the unchanged WAC/ACP
view; `--features trust-graph-did` forwards the DID issuer-key binding (`sq-pfae.3`).

## ✨ Features

- **The minimal `trust:` ontology** (`vocab`) — the 10-term core (`TrustPolicy`, `TrustRule`,
  `trustsSourceFor`, `source`, `forShape`, `Source`, `issuerKey`, `scope`, `freshWithin`,
  `admitted`) + `issuerDid` + the `forPredicate → forShape` desugaring. Published as Turtle
  ([`trust.ttl`](ontologies/trust/trust.ttl), [`SEMANTICS.md`](ontologies/trust/SEMANTICS.md)), pinned to the
  Rust constants by a sync test. **All `trust:` IRIs are NON-STANDARD** — a WG would rehome them.
- **Fail-closed policy parsing** (`policy`) — a trust policy not presented through the
  Control-gated channel admits nothing (a type-level `ControlGate`). Accepts the reified
  `trust:TrustRule` form AND the claim-level `trust:trustsSourceFor` relational form (per-(source,
  statement-type) trust — the compact replacement for ACP's type-only `acp:vc`, `sq-pfae.4`).
- **The admission gate** (`admit`) — canonicalise (`sparq-canon` RDFC-1.0), verify the **checked** issuer
  signature over the commitment (`sparq-zk`, never a self-asserted triple), enforce statement-type scoping
  via a real SHACL shape (`sparq-shacl`), freshness, and the clear-WebID holder binding. Default-deny.
- **N3 merge** (`wire`) — feed admitted facts into `sparq-reason`; the `.acr` ABAC rule derives the grant (age>18 e2e).
- **Storage / authoring model** (`store`, **opt-in `store` feature**, `sq-pfae.5`) — a server-wide
  default + per-`.acr` documents that **NARROW, never broaden** the server ceiling (`effective_rules`);
  monotone versioning (stale rejected) + revocation; the `AdmissionCacheKey` (evidence hash + policy
  version) composes with the sparq-solid epoch cache (edit/revoke invalidates the verdict). No new dep.
- **Static / dynamic admission split** (`admit_static` + `derive_conditional_grants`, `sq-xc4y`) —
  decides the **session-independent** class (signature, type-scope, scope) once at materialise-time
  and defers the **per-request** class (holder + freshness) to an `auth:ConditionalGrant` re-checked
  per request (shipped sq-0q7n path) — never frozen into the view.
- **The invocation-binding gate** (`delegation`, `sq-l5og`) — verify a carried ZCAP/UCAN-style
  delegation chain (each hop a CHECKED delegator signature over delegator/delegate **keys** +
  capability + expiry), enforce monotone attenuation (`child ⊆ parent`), and bind **authenticated
  invoker == the chain's terminal delegate, key-proven per request** via a fresh-challenge PoP.
  Folding each hop's `delegate_key` into the signed preimage defeats the key-substitution
  stolen-chain replay; the forgery matrix runs end-to-end.
- **Delegation PROV-O audit** (`delegation_prov`, **opt-in `delegation-prov`**, `sq-pfae.6`) — render an
  invocation-bound chain as a minimal W3C PROV-O graph: `prov:actedOnBehalfOf` per hop, the human/AI
  principal as an attested attribute (`trust:HumanPrincipal`/`AiAgent`, §4.2), the effective grant as
  `auth:*` RDF. Pure `oxrdf`, NO `sparq-prov` edge. A record, never authority: emitted *after* the gate,
  it only describes authority already conferred (never broader).
- **DID issuer-key binding** (`did`, **opt-in `did` feature**, `sq-pfae.3`) — a rule may name its issuer
  by `trust:issuerDid` instead of `trust:issuerKey` hex. `DidKeyResolver` decodes a `did:key` offline;
  `DidWebResolver` reads a `did:web` document via a **pluggable** fetcher (no HTTP client on the default
  build). **Narrows** the operator-asserted-key forgery vector D′ (not an absolute anchor).
- **Security-properties vocabulary** (`secprop`, **opt-in `secprop-vocab`**, `sq-5oru9`) — the sparq
  **`sec-prop:` extension** (constants + [`secprop-ext.ttl`](ontologies/zkp-sparql/secprop-ext.ttl)): the
  orthogonal proof-system dimensions + the **assurance / audit-status axis** the vendored `sec-prop:` lacks
  (extend, not fork). Records a claim + basis, NOT a proof; default `Claimed`, no `Proven` while `sq-qhy4` open.

## Honest scope — what this does and does NOT do

- **No privacy / unlinkability / anonymity.** The credential is admitted **in the clear**; the
  verifier learns the exact value (`age 25`, not "≥ 18"). This does **not** match ZKAPs-grade unlinkable presentation.
- **Holder binding is the clear-WebID, non-anonymous degraded path** (`sq-wvne`):
  `credentialSubject == Session.agent` authenticates the WebID in the clear — not silently "solved"; presentations stay linkable by requester identity.
- **Issuer keys: operator-asserted by default; DID-bindable (opt-in, `sq-pfae.3`).** The default
  `trust:issuerKey` hex binding is the live forgery vector D′ (§3.3); the `did` feature binds from a
  `trust:issuerDid` instead — **narrows** D′ but no absolute anchor (`did:key` self-cert; `did:web` host/TLS).
- **Delegation invocation is the clear-WebID, non-anonymous path too** (`sq-l5og`): the gate
  authenticates the invoker AS the terminal delegate's WebID in the clear. The `delegate_key` binding
  defeats key-substitution stolen-chain replay but **not** full non-replayability; deep-chain
  *incremental* revocation stays open. The `delegation-prov` PROV-O audit adds NO security property —
  it records *who acted on whose behalf* (a record, not an authority; rustdoc has the full reasoning).
- **Open problems respected as documented limitations:** `sq-tu4e` (no in-reasoner NAF; `revoked` input-only) + `sq-wvne` (ZK privacy) are **out of PoC scope**; `sq-xc4y` RESOLVED; `sq-l5og` **specified + enforced + tested**.
- **Strict additivity (G6):** with `sparq-solid`'s `trust-graph` feature OFF the crate is not compiled — `sparq-solid` behaves exactly as WAC/ACP do today (byte-identical).

## 📚 Learn more

- Machine-readable [`trust.ttl`](ontologies/trust/trust.ttl) + [`SEMANTICS.md`](ontologies/trust/SEMANTICS.md)
  (`sq-pfae.2`) + [`secprop-ext.ttl`](ontologies/zkp-sparql/secprop-ext.ttl); design record
  `research/solid-trust-graph-authz-design.md` (§3.2 storage; §4 delegation; §6.0 PoC) —
  [#940](https://github.com/jeswr/sparq/issues/940). `cargo doc -p sparq-trust --all-features`.

## License

MIT — see [LICENSE](../../LICENSE).
