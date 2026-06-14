---
name: mpc
description: Run a SPARQL query across multiple mutually-distrusting RDF data holders (federated MPC) — per-holder local sub-evaluation, crypto-free disclosed-key global-IRI joins, and an honest-majority Shamir secret-sharing backend for secure cumulative aggregates + hidden-value (private-key) joins. Use for confidential federated SPARQL / secret-shared aggregates / private set-intersection joins over RDF. EARLY/RESEARCH, native-only; honest-majority with RS-consistency-checked reconstruction (tamper DETECTION + abort, and robust correction where redundancy allows) — semi-honest is the floor when no redundancy exists (e.g. the degree-2t equality open at n=2t+1, detection-only); NOT dishonest-majority / not full malicious security. Collaborative ZK proof of correctness+attestation is a stub.
---

# sparq-mpc — MPC over federated SPARQL

`sparq-mpc` lets a set of mutually-distrusting **holders**, each owning private RDF named graphs, jointly answer ONE SPARQL query over the *union* of their data while minimising what each holder reveals. Today it delivers a real per-holder local evaluator, a crypto-free join on disclosed global IRIs, and an honest-majority Shamir secret-sharing backend that powers secure cumulative aggregates and a hidden-value (private-key) join. The reconstruction path is **honest-majority with tamper DETECTION + abort** — and **robust correction where the `(n, t)` redundancy allows** — over RS-consistency-checked opens; the degree-`2t` equality open is **detection-only above `n = 2t + 1`**, and semi-honest is the floor when a given configuration carries no redundancy. It is **not** dishonest-majority and **not** full malicious security; each backend reports its exact level via `BackendInfo.malicious_security`.

> **Maturity (read first).** EARLY / RESEARCH, **native-only** (deliberately not in the wasm build). The Shamir layer's reconstruction is **RS-consistency-checked**: it detects (and, with enough redundancy, robustly corrects) actively-tampered shares and reports the exact guarantee via `BackendInfo.malicious_security` — but the degree-`2t` equality open in the hidden-value join is **not** hardened at `n = 2t+1` (see *Gotchas*). It runs as an **in-process simulation** (one process plays all parties — there is no network transport). The **collaborative ZK proof** of correctness + issuer-attestation is **not implemented**: those methods return `MpcError::NotYetImplemented` naming their gate. No fake crypto anywhere. See *Gotchas* for the full envelope.

## Quickstart

`Cargo.toml` (the crate is in-workspace, `publish = false`, so depend by path):

```toml
[dependencies]
sparq-mpc = { path = "crates/sparq-mpc" }
oxrdf = { version = "0.3", features = ["rdf-12"] }   # for Term / Variable / Literal
```

Two holders each evaluate the same fragment **locally over their own data**, then join on a shared global IRI — no crypto needed because the join key is a disclosed IRI:

```rust
use sparq_mpc::{Holder, DisclosedKeyJoin, GlobalJoin, JoinPlan};
use oxrdf::Variable;

const PFX: &str = "@prefix ex: <http://ex/> .\n";

// Holder A: a "knows" graph. Holder B: a "name" graph. ?x is a global IRI shared across both.
let a = Holder::from_rdf("a", &format!("{PFX} ex:p1 ex:knows ex:x1 . ex:p2 ex:knows ex:x2 ."), "turtle")?;
let b = Holder::from_rdf("b", &format!("{PFX} ex:x1 ex:name \"Xena\" . ex:x2 ex:name \"Yuri\" ."), "turtle")?;

// Each holder runs ITS OWN fragment over ITS OWN graph; only the projected rows leave the holder.
let pa = a.evaluate_local("PREFIX ex: <http://ex/> SELECT ?p ?x WHERE { ?p ex:knows ?x }")?;
let pb = b.evaluate_local("PREFIX ex: <http://ex/> SELECT ?x ?n WHERE { ?x ex:name ?n }")?;

// Disclosed-key equi-join on the global IRI ?x (crypto-free; equals union-store evaluation).
let plan = JoinPlan { join_var: Variable::new_unchecked("x"), key_disclosed: true };
let joined = DisclosedKeyJoin::new().join(&[pa, pb], &plan)?;

assert_eq!(joined.rows.len(), 2);                 // p1-x1-Xena, p2-x2-Yuri
// joined.holder == HolderId("federation"); joined.vars == [?p, ?x, ?n]
# Ok::<(), sparq_mpc::MpcError>(())
```

## Key APIs

Top-level re-exports (`use sparq_mpc::...`):

- `Holder` — one federation participant + its private `Graph`.
  - `Holder::from_rdf(id: impl Into<String>, text: &str, format: &str) -> Result<Holder, MpcError>` — `format` is `"turtle" | "ntriples" | "nquads" | "trig"`.
  - `Holder::new(id, graph: sparq_core::Graph) -> Holder`
  - `Holder::evaluate_local(&self, fragment_sparql: &str) -> HolderResult` (`= Result<PartialResult, MpcError>`) — runs a SELECT fragment over the holder's own graph; raw graph never leaves.
- `PartialResult { holder: HolderId, vars: Vec<oxrdf::Variable>, rows: Vec<Vec<Option<oxrdf::Term>>> }` — the unit of inter-holder sharing (same shape as `sparq_engine::QueryResult`). `.len()`, `.is_empty()`.
- `GlobalJoin` trait + `DisclosedKeyJoin` — `fn join(&self, partials: &[PartialResult], plan: &JoinPlan) -> Result<PartialResult, MpcError>`. SPARQL compatible-mapping inner join over shared columns; independently checks the planner-named key is present (does not trust the planner for soundness). `key_disclosed == false` returns `NotYetImplemented` (use `HiddenValueJoin` instead).
- `JoinPlan { join_var: oxrdf::Variable, key_disclosed: bool }`.
- `MpcBackend` trait — the secret-sharing primitive seam. `type Share`; `info() -> BackendInfo`; `share_private_input(&self, &Holder) -> Result<Vec<Self::Share>, MpcError>`; `run_secure(&self, &[Self::Share]) -> Result<Vec<Self::Share>, MpcError>`; `reconstruct_disclosed(&self, &[Self::Share]) -> Result<PartialResult, MpcError>`.
- `BackendInfo { name: &'static str, trust_model: TrustModel, malicious_security: MaliciousSecurity }` — the guarantees a federation inspects **before** trusting a backend. `BackendInfo::malicious_secure() -> bool` is a coarse "hardened at all?" accessor (`!= None`).
- `TrustModel { HonestMajority, DishonestMajority }` — the majority axis.
- `MaliciousSecurity { SemiHonestOnly, HonestMajorityAbort, HonestMajorityRobust { max_cheaters: usize } }` — the active-security axis (guarantee D), surfaced from the real RS reconstruction redundancy. `SemiHonestOnly` = semi-honest only, an active deviation is undetected (named so it cannot be confused with `Option::None`); `HonestMajorityAbort` = tampering is detected and the protocol aborts (no guaranteed output); `HonestMajorityRobust { max_cheaters }` = guaranteed-correct output even when up to `max_cheaters` parties actively cheat. Derives `Copy`/`Eq`.
- `ShamirBackend` — the only concrete backend (honest-majority Shamir `t`-of-`n`; its reconstruction reports tamper detect-and-abort or robust correction per the `(n, t)` redundancy via `BackendInfo.malicious_security`, falling back to `SemiHonestOnly` only where no redundancy exists).
  - `ShamirBackend::new(n: usize) -> Result<ShamirBackend, MpcError>` — `n >= 2`, threshold `t = (n-1)/2`; masks come from an OS-seeded ChaCha20 CSPRNG. **No seed parameter** by design.
  - `ShamirBackend::new_seeded(n, seed)` — **test/bench only**, behind `cfg(test)` or feature `insecure-test-rng`; predictable masks, no security.
  - `parties()`, `threshold()`, `malicious_security() -> MaliciousSecurity` (the active-security level for this `(n, t)`), `reconstruct(&[Share]) -> Result<Fp, MpcError>`, `dealer() -> ShamirDealer`.
- `HiddenValueJoin::new(backend: ShamirBackend)` + `join(&self, left: &HiddenKeyedRows, right: &HiddenKeyedRows) -> Result<PartialResult, MpcError>` — joins on a **private** key via secret-shared equality; output schema is `left.payload_vars ++ right.payload_vars`, the key is never projected/reconstructed.
- `HiddenKeyedRows { holder: HolderId, payload_vars: Vec<Variable>, rows: Vec<(Fp, Vec<Option<Term>>)> }` — caller encodes each private key into `Fp` (must be injective; see Gotchas).
- `Fp` — field element over `F_p`, `p = 2^61 - 1`. `Fp::new(u64)`, `Fp::value() -> u64`, `Fp::zero()`, `Fp::one()`.
- `Share { x: u64, y: Fp }`; field/share helpers in `sparq_mpc::shamir` (`add_shares`, `sub_shares`, `mul_shares_raw`, `add_constant`, `scale`, `reconstruct_degree`).
- `MpcError::{ LocalEval { holder, message }, Protocol(String), NotYetImplemented { what, gated_on } }` — the single honest channel for deferred crypto.
- `CollaborativeProof<B: MpcBackend>`, `Attestation`, `ProofStatement`, `Proof`, `AttestationShare` — **interface only**; every method is a stub returning `NotYetImplemented`.
- `SecureRng` / `MpcRng` — the CSPRNG masking seam (you rarely touch these directly).

## Common recipes

### 1. Secure cumulative aggregate over private values (the "four flatmates" sum)
Each holder secret-shares one private integer; the sum is computed over shares (free local addition) and only the total is reconstructed — no individual value is revealed.

```rust
use sparq_mpc::{Holder, ShamirBackend, MpcBackend};

const PFX: &str = "@prefix ex: <http://ex/> . @prefix xsd: <http://www.w3.org/2001/XMLSchema#> .\n";
let alice = Holder::from_rdf("alice", &format!("{PFX} ex:a ex:salary \"30000\"^^xsd:integer ."), "turtle")?;
let bob   = Holder::from_rdf("bob",   &format!("{PFX} ex:b ex:salary \"45000\"^^xsd:integer ."), "turtle")?;

let backend = ShamirBackend::new(2)?;                // n parties; honest-majority
let mut shares = backend.share_private_input(&alice)?;   // shares Alice's ex:salary
shares.extend(backend.share_private_input(&bob)?);       // ... and Bob's
let summed = backend.run_secure(&shares)?;               // cumulative sum over shares (0 rounds)
let out = backend.reconstruct_disclosed(&summed)?;       // -> PartialResult: one ?cumulative integer (75000)
# Ok::<(), sparq_mpc::MpcError>(())
```
> `share_private_input` expects the holder's data to yield exactly **one row, one integer column** for `SELECT ?salary WHERE { ?p ex:salary ?salary }`; anything else is a `Protocol` error (it never guesses). The verifier recomputes any threshold like `sum > £100k` *outside* the crypto.

### 2. Hidden-value join on a private key (circuit-PSI core)
Join two holders on a key WITHOUT revealing the key — only the matched payload columns are disclosed.

```rust
use sparq_mpc::{HiddenValueJoin, HiddenKeyedRows, ShamirBackend, HolderId, Fp};
use oxrdf::{Term, Variable, Literal};

let lit = |s: &str| Some(Term::Literal(Literal::new_simple_literal(s)));
// Keys are PRIVATE identifiers encoded as Fp (here a controlled injective integer map).
let left  = HiddenKeyedRows { holder: HolderId::new("L"), payload_vars: vec![Variable::new_unchecked("name")],
    rows: vec![(Fp::new(100), vec![lit("Alice")]), (Fp::new(200), vec![lit("Bob")])] };
let right = HiddenKeyedRows { holder: HolderId::new("R"), payload_vars: vec![Variable::new_unchecked("city")],
    rows: vec![(Fp::new(200), vec![lit("Leeds")]), (Fp::new(999), vec![lit("Hull")])] };

let backend = ShamirBackend::new(3)?;                // n=3, t=1 supports the 1 equality multiplication (2t+1<=n)
let joined = HiddenValueJoin::new(backend).join(&left, &right)?;
// joined.vars == [?name, ?city]; one row Bob-Leeds; key 100/999 dropped; keys never reconstructed.
# Ok::<(), sparq_mpc::MpcError>(())
```

### 3. Multi-holder chain join on disclosed global IRIs
`DisclosedKeyJoin` folds pairwise; chain joins by feeding the result back in with the next shared variable.

```rust
use sparq_mpc::{DisclosedKeyJoin, GlobalJoin, JoinPlan};
use oxrdf::Variable;
let v = |n: &str| Variable::new_unchecked(n);
// pa: ?p ?x , pb: ?x ?y , pc: ?y ?n  (each a PartialResult from evaluate_local)
let ab  = DisclosedKeyJoin.join(&[pa, pb], &JoinPlan { join_var: v("x"), key_disclosed: true })?;
let abc = DisclosedKeyJoin.join(&[ab, pc], &JoinPlan { join_var: v("y"), key_disclosed: true })?;
# Ok::<(), sparq_mpc::MpcError>(())
```

### 4. Inspect a backend's guarantees before trusting it
A federation should refuse a backend whose guarantees don't match its threat model.

```rust
use sparq_mpc::{ShamirBackend, MpcBackend, TrustModel, MaliciousSecurity};
let info = ShamirBackend::new(3)?.info();
assert_eq!(info.trust_model, TrustModel::HonestMajority);
// At n=3,t=1 there is one redundant share → tampering is detected & aborts
// (but a cheater can still force an abort: no guaranteed output). With more
// redundancy (n>=4) it becomes HonestMajorityRobust { max_cheaters }.
assert_eq!(info.malicious_security, MaliciousSecurity::HonestMajorityAbort);
assert!(info.malicious_secure());           // coarse "hardened at all?" bit
# Ok::<(), sparq_mpc::MpcError>(())
```

### 5. Reproducible tests/benches (predictable masks — never production)
```toml
[dev-dependencies]            # or [dependencies] only if you understand the risk
sparq-mpc = { path = "crates/sparq-mpc", features = ["insecure-test-rng"] }
```
```rust
let backend = sparq_mpc::ShamirBackend::new_seeded(3, 0xBEEF)?;   // deterministic masks; NO security
# Ok::<(), sparq_mpc::MpcError>(())
```

### 6. Handle deferred crypto honestly
The collaborative proof and the hidden-key path of `DisclosedKeyJoin` are gated; match the error rather than assuming success.

```rust
use sparq_mpc::MpcError;
match some_result {
    Err(MpcError::NotYetImplemented { what, gated_on }) =>
        eprintln!("deferred: {what} (gated on {gated_on})"),   // e.g. mentions M3/M4, Q1, ZK #3..#12
    Err(MpcError::Protocol(m)) => eprintln!("precondition: {m}"),
    Err(MpcError::LocalEval { holder, message }) => eprintln!("holder {holder} eval failed: {message}"),
    Ok(_partial) => { /* real result */ }
}
```

## Gotchas / feature flags / prerequisites

- **Native-only, not in wasm.** `sparq-mpc` is intentionally absent from `sparq-wasm`'s dependency graph (`cargo tree -p sparq-wasm` must not show it). The browser bundle carries zero MPC/crypto surface.
- **`insecure-test-rng` feature (OFF by default).** Gates `ShamirBackend::new_seeded` and `rng::InsecureTestRng` (a deterministic SplitMix64). The masks it produces are **predictable** — enabling it in a deployment reintroduces the very confidentiality weakness the CSPRNG default fixes. Use it only for reproducible tests/benchmarks. Default builds physically cannot construct a predictable masking RNG.
- **Malicious-security is now SURFACED, not blanket-absent.** Confidentiality holds against `<= t` colluding *honest-but-curious* parties. Against an *actively-deviating* party, `ShamirBackend` reports the precise guarantee via `BackendInfo.malicious_security` (`ShamirBackend::malicious_security()`): the WI-1 RS-checked / Berlekamp–Welch reconstruction **detects** tampered shares and aborts when there is redundancy (`n > t+1`, always true for the honest-majority `t`), and **robustly corrects** up to `max_cheaters = ⌊(n−t−1)/2⌋` cheaters when redundancy allows (`n >= 4`). **Boundaries that are NOT hardened (do not over-trust):** the degree-`2t` equality/mult open in the hidden-value join has *no* RS redundancy at `n = 2t+1` (the common odd-`n` case, e.g. n=3,5,7) — a tampered product share there is undetectable; a fix needs an information-theoretic MAC (deferred, bead sq-6d6g). Dishonest-majority remains future work behind the same `MpcBackend` trait.
- **In-process simulation, not a network protocol.** The "multi-party" computation runs in one process (the dealer/`HiddenValueJoin` plays all parties to deal shares and open results). There is no transport, no party-to-party messaging, no real party isolation yet. Cleartext inputs are passed to the simulator only to be shared internally.
- **Field & range.** `Fp` is over `p = 2^61 - 1`. Keep values (salaries, counts, key encodings) well under `2^61` so sums never wrap. `Fp::inv(0)` panics (only nonzero differences are inverted internally).
- **Shamir headroom.** A single multiplication (the equality test) needs `n >= 2t+1`. `ShamirBackend::new` picks `t = (n-1)/2`, so the happy path holds; `HiddenValueJoin` errors with `Protocol` if you somehow under-provision. `reconstruct` needs `>= t+1` distinct-`x` shares.
- **Hidden-join key encoding is YOUR responsibility.** `HiddenKeyedRows.rows` carry `Fp` keys; equality in `Fp` stands in for term equality and is only exact if your encoding is **injective** over the key domain. Production needs a collision-resistant hash proven in-circuit — not provided here.
- **Hidden join cost.** `HiddenValueJoin` is naive `O(|L|·|R|)` all-pairs (one secure equality test per pair). No cuckoo/oblivious-hashing PSI optimisation. Honest envelope: minutes-to-tens-of-minutes for ~10³–10⁴ rows/holder on a LAN — **not** sub-second; do not extrapolate beyond it.
- **Disclosed-key join is a faithful SPARQL inner join, not a naive key-merge.** It enforces agreement on *every* shared variable (compatible-mapping semantics) and does not trust the planner-named key for soundness; a key absent from any partial is a `Protocol` error. Output rows are canonicalised (order-independent multiset).
- **Collaborative proof is a stub.** `CollaborativeProof`, `Attestation`, `ProofStatement` are interface + docs only; every method returns `NotYetImplemented`. They are hard-gated on the single-prover ZK soundness remediation (issuer-signature / replay / FILTER-binding / attribution / revocation) and the open research question of verifying a signature over a *secret-shared* witness (Q1, milestone M4). Do not assume any correctness/attestation guarantee from this crate yet — only confidentiality of the secret-shared inputs under the semi-honest model.

## See also

- `mpc-protocols` — the SOTA / primitive background (secret sharing, garbled circuits, collaborative zk-SNARKs, authenticated-input MPC) behind the trust-model and primitive choices here.
- `noir-circuit-patterns`, `noir-optimisation`, `verifiable-credentials-zk`, `sparql-formal-semantics` — the single-prover ZK estate this crate will eventually compose with for the (deferred) collaborative proof.
