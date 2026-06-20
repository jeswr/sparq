<!-- [OPUS-4.8] sq-pfae PoC (issue #940). 🤖 SPARQ agent — trust-graph authorisation PoC. -->
# sparq-trust

<p>
  <a href="../../LICENSE"><img src="https://img.shields.io/badge/license-MIT-blue.svg" alt="License: MIT"></a>
</p>

The **proof-of-concept** for trust-graph authorisation over [sparq-solid](../sparq-solid/README.md)
(design record [`research/solid-trust-graph-authz-design.md`](../../research/solid-trust-graph-authz-design.md)
§6.0; issue #940, epic `sq-pfae`). It adds the **admission stratum** ahead of the shipped
WAC/ACP derivation stratum: *"is this externally-attested fact from a source I trust for this
statement-type?"* — and on success injects the issuer-tagged fact so the existing N3 reasoner
merges it with the `.acr` rules to derive access.

> **RESEARCH PROTOTYPE — NOT a shipped feature and NOT a security guarantee.** It demonstrates
> exactly one claim: a single externally-attested fact is admitted through a
> trusted-source-scoped gate, then merged by N3 reasoning with an `.acr` rule to derive
> `canAccess`. It does **not** provide privacy, unlinkability, or anonymity (see *Honest scope*).
> The ZK estate it composes with is externally **unaudited** (`sq-qhy4`) and is pending external
> accredited-cryptographer sign-off.

> Model: Opus 4.8 (Fable unavailable — flag for re-review when Fable returns).

## 🚀 Quickstart

`sparq-trust` is **opt-in**: nothing in the workspace's default build depends on it. The
admission gate is wired into `sparq-solid` behind its default-OFF `trust-graph` cargo feature.

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

From `sparq-solid` (with `--features trust-graph`), `PodStore::admit_trust_credential_with_rule`
runs the gate and installs the derived `auth:*` grants into `<urn:sparq:auth>` on top of the
unchanged WAC/ACP view.

## ✨ Features

- **The 10-term minimal ontology** (`vocab`) — `trust:TrustPolicy`, `TrustRule`,
  `trustsSourceFor`, `source`, `forShape`, `Source`, `issuerKey`, `scope`, `freshWithin`,
  `admitted` — plus the one `forPredicate → forShape` desugaring (sugar, not a primitive).
- **Fail-closed policy parsing** (`policy`) — a trust policy not presented through the
  Control-gated channel admits nothing (a type-level `ControlGate`).
- **The admission gate** (`admit`) — canonicalise (`sparq-canon` RDFC-1.0), verify the issuer
  signature over the commitment (`sparq-zk`, the **checked** signature, never a self-asserted
  triple), enforce statement-type scoping via a real SHACL shape (`sparq-shacl`), freshness, and
  the clear-WebID holder binding. Default-deny, short-circuit.
- **N3 merge** (`wire`) — feed the admitted facts into the shipped `sparq-reason` reasoner ahead
  of the materialiser; the `.acr` ABAC rule `{ ?x age ?y . ?y math:greaterThan 18 } => { ?x
  auth:read R }` derives the grant. The age>18 worked example runs end-to-end (see the tests).

## Honest scope — what this does and does NOT do

- **No privacy / unlinkability / anonymity.** The credential is admitted **in the clear**; the
  verifier learns the exact value (`age 25`, not "≥ 18"). This does **not** match ZKAPs-grade
  unlinkable presentation.
- **Holder binding is the clear-WebID, non-anonymous degraded path** (`sq-wvne` / `sq-xc4y`):
  `credentialSubject == Session.agent` authenticates the WebID in the clear — documented, not
  silently "solved". Presentations stay linkable by requester identity.
- **Issuer keys are operator-asserted** — sparq has no DID resolver yet (`sq-pfae.3`), so the
  `trust:issuerKey → verifying-key` binding is the live forgery vector D′ (§3.3), not an
  end-to-end trust path.
- **Open problems respected as documented limitations, never solved:** `sq-xc4y` (per-request
  admission vs materialise-once), `sq-tu4e` (no in-reasoner NAF over derived facts; `revoked` is
  input-only; no deny-on-disagreement), `sq-l5og` (delegation) and `sq-wvne` (ZK privacy) are
  **out of PoC scope**.
- **Strict additivity (G6):** with `sparq-solid`'s `trust-graph` feature OFF, the crate is not
  compiled and `sparq-solid` behaves exactly as WAC/ACP do today.

## 📚 Learn more

- The design record: [`research/solid-trust-graph-authz-design.md`](../../research/solid-trust-graph-authz-design.md)
  (§6.0 is the PoC spec; §7 the honest limitations).
- The host crate: [`sparq-solid`](../sparq-solid/README.md) (the WAC/ACP substrate this extends).
- `cargo doc -p sparq-trust` for the full module-level docs.

## License

MIT — see [LICENSE](../../LICENSE).
