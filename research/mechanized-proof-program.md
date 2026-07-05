# Mechanized-proof program — property inventory, tool triage, ranked targets

<!-- [FABLE] sq-ksvpk (architect design spike) under epic sq-sqtk2 (maintainer 2026-07-04
     roadmap item 1). Authored by the Fable architect stage of the collaboration tier;
     grounded against the actual code on origin/main at the commit this branch forked from,
     with three read-only code surveys plus direct reads of the decision/eval cores. -->

Status: design record (decomposition; Phase-1 child beads created). Author: SPARQ architect stage (Fable).

**Claim discipline, stated first.** This program mechanizes **FUNCTIONAL properties only** —
fail-closed decision structure, result-equivalence, order laws, id-space integrity. It makes
**no security claim**: nothing here labels any ZK/MPC component "proved secure" or "sound" —
every outward ZK/MPC security property remains gated on the external accredited-cryptographer
audit (`sq-qhy4`), and `sparq-mpc` remains honest-majority semi-honest only. A machine-checked
functional property *complements* that audit; it never substitutes for it.

---

## 1. Scope and the assurance ladder

"Proof" is used loosely across the industry; this record fixes the vocabulary the whole
program (and the `sq-wir9k` assurance walkthrough) uses:

| Tier | Meaning | Strength |
|------|---------|----------|
| **PROVED (complete-domain)** | A `#[kani::proof]` harness whose symbolic input covers the property's *entire* domain (pure integer/enum domains — CBMC handles these exhaustively) | Machine-checked over the whole domain; TCB = Kani/CBMC + the harness-as-spec |
| **PROVED (bounded)** | A Kani harness with stated `unwind`/length/alphabet bounds | Machine-checked *within the stated bounds*, nothing beyond them |
| **EXHAUSTIVELY TESTED (bounded domain)** | A plain `#[test]` that enumerates a finite domain completely (small alphabets/lengths) | Complete within the domain; cheaper than Kani where strings make CBMC blow up |
| **DIFFERENTIALLY TESTED** | Agreement with an independent oracle on a corpus (the M1 pattern) | Verification, not proof — TCB includes the oracle |
| **CONFORMANCE-PINNED** | A ratcheted W3C/OGC/spec-derived suite floor | Spec-parity on the suite's cases only |
| **FUZZED** | Coverage-guided robustness (no crash/UB), not output correctness | Robustness only |

Every claim below and every child bead names its tier and its bounds. A small proved property
beats a grand unproved claim; where a property is not tractable now it goes in the §6 ledger
instead of being over-claimed.

## 2. The estate today (verified against the code, not taken on faith)

- **Kani lane exists**: `.github/workflows/kani.yml` — nightly + `workflow_dispatch`,
  informational/non-blocking, runs `cargo kani -p sparq-vectors` and
  `cargo kani -p sparq-core --features mmap`. Four harnesses today:
  `open_from_bytes_never_panics` / `open_validated_v2_tail_never_panics`
  (`crates/sparq-vectors/src/store.rs`, `mod kani_proofs`) and
  `validate_dict_bytes_never_panics` / `validate_dict_bytes_sized_tail_never_panics`
  (`crates/sparq-core/src/dict.rs`, `mod kani_proofs`, `#[cfg(all(kani, feature = "mmap"))]`).
  All four are *no-panic/no-UB* properties on hostile-input validators (threat-model boundary
  B5), not semantic-correctness properties — this program adds the semantic tier.
- **Adjacent lanes**: `miri.yml` (UB detection over the `sparq-core` unsafe surface),
  `asan.yml` (mmap corruption corpus), `fuzz.yml` (4+ parser/loader targets, PR/main/nightly
  budgets). These stay the robustness floor; they are not correctness proofs.
- **Differential floor (in-tree, merged)**: `crates/sparq-engine/tests/differentials/` —
  `dp_planner_differential.rs` (DP join order vs greedy, result-identical),
  `yannakakis_differential.rs` + `semijoin_differential.rs` (WCOJ/semi-join vs baseline),
  `vectorized_exec_differential.rs` (columnar aggregates vs scalar), `zk_trace_differential.rs`,
  `window_inline_over.rs`. `crates/sparq-difftest` is the engine-independent value-level
  normalisation layer (design `research/differential-testing-value-level.md`, sq-qcnn);
  external-oracle wiring (Oxigraph et al.) is a separate, not-yet-landed DAG node.
- **The M1 pattern**: the ZK proof program's differential-oracle milestone — a Rust harness
  asserting bit-equality against a trusted independent reference — is the proven honest floor.
  At the time of writing its design record is **open for review as PR #1466**
  (`research/zk-correctness-and-proof-program.md`, branch `feat/zk-correctness-proof-program`)
  and the first M1 harness (ieee754 vs hardware f32/f64 oracle) is **open as PR #1471** —
  cite both as *pending*, not merged, until they land.
- **Conformance floors**: W3C SPARQL 1.1 query/protocol/service-description/GSP lanes, OWL 2
  QL certain-answer oracle, OGC GeoSPARQL topology ratchet, JSON-LD expand oracle, SHACL —
  all ratcheted in CI. For WAC/ACP specifically: `crates/sparq-solid/tests/conformance_wac.rs`
  and `tests/conformance_acp.rs` (beads sq-3jtd.8/.9, closed) pin decision parity against
  spec-derived corpora, plus `tests/differential_oracle.rs` and `tests/hardening.rs`.
- **The Lean precedent, honestly**: the maintainer's prior `jeswr/sparql_noir` work shipped
  prose `SAFETY PROOF` doc-comments + differential testing; the Lean-equivalence
  correspondence was **evaluated and deferred as not-yet-cost-justified** — no Lean proof was
  actually shipped. That precedent is a *cost datum* (Lean equivalence for SPARQL evaluation
  is expert-months, research-tier), not an existing artifact to build on.
- **Property-based testing**: the workspace has **no proptest/quickcheck usage** — tests are
  hand-curated units, differentials, and libFuzzer targets. (Relevant when choosing between
  proptest and bounded-exhaustive `#[test]`s below; this record prefers the latter plus Kani,
  keeping zero new dev-dependencies.)

## 3. Property inventory

### 3.1 Surface A — WAC/ACP authorization decisions (`crates/sparq-solid`)

Grounding (direct read): the decision core is **pure and small**. `decide.rs` (553 LOC)
computes `decide_one(index: &AclIndex, auth: &AuthIndex, session: &Session, resource: &str,
mode: Mode) -> WacDecision` with **a single fail-closed constructor** —
`WacDecision::deny(status)` — for every uncertainty path ("so deny-by-default is impossible
to forget", per its doc comment). `AclStatus ∈ {Resolved, Unloaded, NoAcl, Transient}`;
the doc contract states `allow` is `false` for every non-`Resolved` status. Container-default
inheritance is `AclIndex::resolve_acl` — own `.acl`/`.acr` wins (`AccessTo` scope), else a
`parent_iri` walk to the nearest ancestor holding a control document (`Default` scope), else
`None` (caller fails closed). The per-mode oracle is `AuthIndex::accessible(s, mode) ->
Vec<NamedNode>` — a `∪ allow ∖ ∪ deny` set. No I/O, no async, no `sparq-core` index machinery
inside the decision path.

| Id | Property | Current tier | Target tier |
|----|----------|--------------|-------------|
| A-1 | **Fail-closed structure**: every `decide_one` path with `status != Resolved` yields `allow == false` and empty `granted_modes`; `allow == true` implies `status == Resolved` and the mode is in `granted_modes` | Unit + conformance tests | PROVED (bounded) — Kani over bounded index/auth/session states |
| A-2 | **Container-default walk**: own ACL beats ancestors; else *nearest* ancestor with a control doc governs by `acl:default`; walk terminates (`parent_iri` strictly shortens) | Unit tests | PROVED (bounded) for termination; EXHAUSTIVELY TESTED (bounded domain) for nearest-ancestor over a small segment alphabet (Kani string costs are prohibitive) |
| A-3 | **Deny-wins / monotonicity**: `accessible` equals `∪ allows ∖ ∪ denies`; monotone in the allow set, antitone in the deny set (adding an allow never revokes; a deny always wins over a matching allow) | Implicit in conformance corpus | PROVED (bounded) — Kani over bounded grant vectors |
| A-4 | **Spec decision-parity** (full WAC/ACP semantics vs the Solid specs) | CONFORMANCE-PINNED (sq-3jtd.8/.9 corpora) | Stays conformance-pinned — see §6 ledger (no machine-readable spec semantics to prove against) |

Value: this is the authorization boundary — the highest-consequence pure logic in the repo.
Tractability: excellent (pure, finite enums, single deny constructor). A-1..A-3 are **Phase-1
bead sq-sqtk2.1**.

### 3.2 Surface B — SPARQL evaluation (`crates/sparq-substrate` + `crates/sparq-engine`)

Grounding: the shared eval substrate is real and small — `sparq-substrate` ships `compare`
(418 LOC, `compare_terms<T: CompareTerm>(x, y) -> Option<Ordering>` over the minimal
`CompareTerm` observation trait, with cross-class precedence `ErrorOrUnbound < Blank < Iri <
Literal < Triple`, an f64-collapse `exact_cmp` recheck tier for exact numerics around the
2^53 boundary, and RDF 1.2 triple-term recursion), `join` (1339 LOC: merge / hash / bind /
leapfrog-trie kernels, generic over `Budget`), `numeric` (the XSD value tower), `rows` — all
default-off features, all pure. The engine implements `CompareTerm` for its private `Value`
in `exec.rs`; join ordering is greedy GOO (`exec.rs`) or opt-in DPccp (`dp.rs`), both pure
`Plan -> Plan`; aggregation has a scalar path (`eval_aggregate`/`group_aggregate`, `exec.rs`)
and a feature-gated columnar path (`reduce.rs`, 236 LOC: `reduce_sum` with exact i128
accumulation, `reduce_count`, `reduce_min_id`/`reduce_max_id`, `narrow_sum_to_i64` overflow
guard) already covered by `vectorized_exec_differential.rs`.

| Id | Property | Current tier | Target tier |
|----|----------|--------------|-------------|
| B-1 | **`compare_terms` order laws**: reflexivity; antisymmetry-consistency (`cmp(x,y) == Some(o)` iff `cmp(y,x) == Some(o.reverse())`); transitivity on the defined domain; within-class totality. These are what `ORDER BY`'s `sort_by` validity rests on (an inconsistent comparator makes Rust's sort panic or produce garbage), and the f64-collapse recheck at 2^53 exists precisely because the naive collapse breaks them | Hand-curated units (incl. a 2^53+1 pin) | PROVED (bounded) over an in-harness model type implementing `CompareTerm`; the engine's `Value` instance stays unit/conformance-covered (§6) |
| B-2 | **Columnar reducers ≡ scalar reference**: `reduce_sum/count/min/max` equal an independent scalar fold; `narrow_sum_to_i64` returns `Some` exactly on the representable range | DIFFERENTIALLY TESTED (`vectorized_exec_differential.rs`) | PROVED (bounded: slice length; complete over element values) — Kani |
| B-3 | **Join-kernel result-equivalence** (merge/hash/bind/LFTJ ≡ nested-loop reference, multiset semantics) | DIFFERENTIALLY TESTED (yannakakis/semijoin differentials) | §6 ledger now; bounded merge-join harness is the plausible next wave |
| B-4 | **Optimizer rewrite soundness** (GOO/DPccp orders are result-equivalent — join commutativity/associativity as *used*; sargable filter pushdown) | DIFFERENTIALLY TESTED (`dp_planner_differential.rs`) + conformance | §6 ledger (plan-space proof intractable now; the differential is the honest floor) |
| B-5 | **Solution-modifier semantics** (DISTINCT/ORDER/LIMIT/projection pipeline) | CONFORMANCE-PINNED (W3C suites) | Rides on B-1 (sort validity); otherwise stays conformance-pinned |
| B-6 | **Aggregation semantics vs spec** (AVG/GROUP_CONCAT/error propagation on the scalar path) | CONFORMANCE-PINNED + differential | §6 ledger (interleaved with expression eval; no clean pure seam today) |

B-1 and B-2 are **Phase-1 beads sq-sqtk2.4 and sq-sqtk2.3**.

### 3.3 Surface C — id/term substrate (`crates/sparq-core`) + parsers

Grounding: `dict.rs` is a content-addressed single-storage interner with a partitioned
id space — `NO_ID` (0), dict ids `[1, INLINE_BASE)`, canonical inline `xsd:integer` ids
`[INLINE_BASE, INLINE_BASE + 2^30)` (value recoverable by subtraction; `try_inline` accepts
only the canonical lexical form), engine local-vocab ids above. It already hosts the two
B5-validator Kani harnesses.

| Id | Property | Current tier | Target tier |
|----|----------|--------------|-------------|
| C-1 | **Inline-id round-trip + id-partition disjointness**: `inline_id_of_int`/`try_inline` encode/decode is a bijection on the canonical range; the four id regions are pairwise disjoint | Unit tests | PROVED (complete-domain) — pure integer arithmetic, CBMC covers the full domain symbolically |
| C-2 | **Dict `intern`/`lookup` bijectivity over full terms** (strings/heap) | Unit round-trip tests | §6 ledger (Kani string cost; structural hash-consing argument + tests remain the tier) |
| C-3 | **Term canonicalization idempotence** (numeric canonical form at intern; lang-tag lowercasing; `sparq-canon`) | Unit tests | Partially subsumed by C-1 (the canonical-form acceptance check); the broader `sparq-canon` surface needs its own grounding pass — §6 ledger |
| C-4 | **Parser round-trips** (parse → serialize → parse fixpoint for N-Triples/Turtle/SPARQL) | FUZZED (robustness only) — no round-trip tests exist today | §6 ledger: this is a *testing* gap (a round-trip differential), not a mechanized-proof target; worth a future test bead, mis-scoped as proof |

C-1 is **Phase-1 bead sq-sqtk2.2**.

## 4. Tool triage — honest tractability

### 4.1 Per-tool verdicts

- **Kani (CBMC bounded model checking) — the workhorse. ADOPT.** Already installed in CI with
  four passing harnesses; zero runtime-code changes needed (`#[cfg(kani)]` modules); excellent
  on pure functions over integers/enums/small slices — and on *purely numeric* domains the
  symbolic coverage is the **whole domain** (a complete proof modulo the Kani/CBMC TCB).
  Weaknesses to respect: `String`/`format!`/hash-map-heavy code blows up (hence the
  bounded-exhaustive-`#[test]` fallback for the IRI walk), every claim must state its
  `unwind`/length bounds, and the lane is nightly + non-blocking (a failed proof does not
  gate merges — see §6 for the ratchet question).
- **Creusot / Prusti (deductive, unbounded) — NOT NOW.** Both demand pervasive spec
  annotation (Pearlite/contracts), have incomplete std/trait support, and would couple proof
  maintenance to every refactor of a fast-moving codebase. For the small pure functions we
  target, bounded/complete-domain Kani already delivers the property at a fraction of the
  cost; deductive tools earn their keep only on a *frozen* module where an unbounded loop
  invariant matters. Revisit if/when a surface freezes (candidate: the dict id partition,
  which is already complete-domain under Kani anyway).
- **Lean equivalence against a reference implementation — RESEARCH-TIER, NOT BEADED.** The
  `sparql_noir` precedent is the honest cost datum: the Lean correspondence was evaluated
  there and deferred as not-cost-justified; nothing shipped. A Lean formalization of even the
  Pérez–Arenas–Gutiérrez algebra fragment plus an extraction-faithful reference impl is
  expert-months before the first theorem. Where Lean *could* eventually pay is the
  ZK-adjacent pure-data composition obligations (`ProofManifest × Query -> Bool`, per
  `research/mpc-zkp-research-and-architecture.md`) — out of this program's functional scope
  and still subject to the `sq-qhy4` framing for anything security-flavored.
- **hax (Rust → F*/Coq extraction) — NOT APPLICABLE NOW.** Built for crypto-code extraction;
  our Phase-1 targets are not crypto, and the crypto surface that *would* motivate hax (the
  ZK verifier) must not be advertised as "proved secure" ahead of the external audit
  regardless of what an extraction proves. Keep on the watchlist for post-audit hardening.
- **Differential harnesses vs independent oracles — the honest floor everywhere proofs are
  intractable.** The proven M1 pattern (PR #1471, open at time of writing; the engine's
  in-tree differentials, merged). Rule: when §6 says "not tractable now", the differential
  floor is not optional — it is the tier of record, and the assurance table says so plainly.
- **Bounded exhaustive enumeration `#[test]`s — small, honest, underused.** Where a domain is
  naturally finite and small (path alphabets, enum products), a plain test that enumerates it
  completely gives "complete within the domain" at trivial cost and zero new dependencies.
  Used for A-2's nearest-ancestor semantics.

### 4.2 Per-property triage table

Effort tiers: S (≤1 agent-day), M (a few agent-days), L (weeks), R (research/expert-months).

| Property | Tool | Effort | Resulting TCB (what you must trust) |
|----------|------|--------|--------------------------------------|
| A-1 fail-closed | Kani | M | Kani/CBMC + harness bounds + harness-as-spec |
| A-2 container walk | Kani (termination) + exhaustive `#[test]` (semantics) | S–M | Kani/CBMC + the enumerated domain's representativeness |
| A-3 deny-wins/monotonicity | Kani | S–M | Kani/CBMC + harness bounds |
| A-4 spec parity | — (stays conformance) | — | Corpus fidelity to the Solid specs |
| B-1 compare laws | Kani over a model `CompareTerm` impl | M | Kani/CBMC + **model fidelity to the trait's real impls** + bounds |
| B-2 reducers ≡ reference | Kani | S | Kani/CBMC + slice-length bound + in-harness reference-as-spec |
| B-3 join kernels | differential (now); bounded Kani merge-join (next wave) | M (next wave) | Oracle correctness (now) |
| B-4 optimizer rewrites | differential | — | Baseline-plan correctness |
| C-1 inline ids | Kani | S | Kani/CBMC only (complete domain) |
| C-2 dict bijectivity | tests + structural argument | — | Test corpus |
| C-4 parser round-trip | (future differential/test bead) | S | Serializer as oracle |

## 5. Ranking and the Phase-1 decomposition (child beads — created)

Ranking = value × tractability. Quality over volume: five beads, one crate/surface each,
**disjoint by construction** (no two beads touch the same file; ≤1 bead per crate), all
**harness-only diffs** (runtime logic byte-unchanged — that is each bead's load-bearing
invariant, and what lets the fleet's mechanical verify stay objective).

| # | Bead | Crate/surface | Tier | Property | Why this rank |
|---|------|---------------|------|----------|----------------|
| 1 | `sq-sqtk2.1` | `sparq-solid` (files: `src/decide.rs`, `src/authindex.rs` `#[cfg(kani)]` modules + a bounded-enumeration test) | opus | A-1 + A-2 + A-3 | Highest consequence (the authorization boundary) × high tractability (pure, finite, single deny constructor). Security-adjacent + harness-design subtlety ⇒ opus, not sonnet |
| 2 | `sq-sqtk2.2` | `sparq-core` (file: `src/dict.rs`, extending the existing `kani_proofs` module) | sonnet | C-1 | Foundational (every result rides on id integrity) × best tractability in the program (complete-domain integer proof); spec is fully written ⇒ cheap tier |
| 3 | `sq-sqtk2.3` | `sparq-engine` (file: `src/reduce.rs` + its `#[cfg(kani)]` module) | sonnet | B-2 | Answer-correctness on the fast aggregate path; crisp bounded spec (independent in-harness reference fold — deliberately NOT the exec.rs scalar path, so the reference is the spec, not the twin) |
| 4 | `sq-sqtk2.4` | `sparq-substrate` (file: `src/compare.rs` + harness module) | opus | B-1 | ORDER BY/sort validity for engine + reasoners via the shared substrate; the 2^53 collapse tier is exactly where a cheap model would write a vacuous harness ⇒ opus |
| 5 | `sq-sqtk2.5` | CI (file: `.github/workflows/kani.yml` ONLY) | haiku | wires 1–4 into the nightly lane | Multiplier — makes the proofs continuously checked. Pattern-following YAML edit; blocked by beads 1–4 (`bd dep` edges wired) |

Every bead carries `{crate, model_tier, invariant, acceptance_test}` in its `bd` body; the
acceptance commands are mechanical (`cargo kani -p <crate> [--features <f>]` + the crate's
`cargo test`), so downstream verify needs no judgment. Beads 1–4 are independent and
parallelize; bead 5 lands last.

**Non-vacuity is mandatory.** Each harness bead's acceptance includes a mutation spot-check
(e.g. force `allow = true` on an `Unloaded` path / weaken the model's `exact_cmp` at the
2^53 boundary — the harness must go red). A proof harness that cannot fail is worse than no
harness: it launders confidence.

### 5.1 Mandatory domain-coverage self-checks (the anti-vacuity program — sq-og8u8)

The mutation spot-check above is **necessary but NOT sufficient**. It catches a harness whose
*assertion* is too weak; it does **not** catch a harness whose *input domain* has been silently
emptied of the interesting inputs — because the mutant is pruned on exactly the same paths as
the property, so it stays green too.

**The failure class (sq-sqtk2.1, 2026-07-04).** The `sparq-solid` decision harnesses bounded
their symbolic principals with an `assume` and an `#[kani::unwind(24)]`. Under that unwind the
`PUBLIC` / `AUTHENTICATED` / `ANY_CLIENT` / `ANY_ISSUER` principal identities — 32–39-byte
strings — needed more loop iterations than the bound admitted, so CBMC pruned those paths via
`assume(false)`. The deny-wins / fail-closed harnesses were therefore **VACUOUS for exactly the
security-relevant identities**, and reported nothing wrong. A mutation spot-check could not
catch it: the mutant that would have broken deny-wins for `PUBLIC` was pruned alongside the
input. The hole was found only by re-scoping the harness (sq-sqtk2.7: `FlattenCompat`
elimination, shorter symbolic strings, `unwind 24 → 40`) and noticing the newly-reachable
paths — i.e. by luck, not by construction. **This must never depend on luck again.**

**The requirement.** Every harness *suite* (every `#[cfg(kani)]` module in the program) MUST
ship at least one **domain-coverage self-check** — a dedicated `#[kani::proof]` harness that
proves the suite's *interesting* inputs genuinely survive the bounds and are genuinely
adversarial. Two complementary shapes, use whichever the suite needs (usually both):

- **Exact-image / domain pinning** — plain `assert!`s over the domain CONSTANTS (tables,
  partition bounds, generator ranges) proving the interesting input is present and has the
  adversarial property. These use no symbolic loop, so they **cannot themselves be pruned**;
  they go red if a future re-scope collapses the domain. *Exemplar:*
  `domain_exhibits_the_2p53_collapse` and `domain_cf_numeric_is_collapse_free_with_signed_zero_pair`
  in `sparq-substrate/src/compare.rs` (PR #1477), documented there under
  "DOMAIN-COVERAGE SELF-CHECKS".
- **Witness survival** — a `kani::cover!` (Kani's SAT-reachability primitive) asserting that a
  MAXIMAL / most-interesting input is reachable under the suite's own `unwind` bound. If a bound
  tightening `assume(false)`-prunes that input (the sq-sqtk2.1 mode), the cover becomes
  UNREACHABLE and goes **red** — the direct, mechanical guard against silent pruning. For a
  no-panic *totality* harness the analogous check is that the ACCEPT / deep-validation path is
  reachable within the bound (a concrete accepted input, or a `cover!(result.is_ok())`), so the
  proof is non-vacuous on the code that does the work, not only the early-reject branches.

**Applied to the merged suites (sq-og8u8 audit).**

| Suite | Structure | Self-check shipped |
|-------|-----------|--------------------|
| `sparq-substrate/src/compare.rs` (#1477/#1502) | model `M`, per-kind `unwind`; strings compared are the short `STRS` (≤2 chars) only — after the sq-wjl8i kind-first fix, cross-kind pairs rank by enum and same-kind numerics by f64/exact, so the long `INT_STRS`/`DBL_STRS` forms are never byte-compared and cannot be unwind-pruned | **exemplar** — `domain_exhibits_the_2p53_collapse`, `domain_cf_numeric_is_collapse_free_with_signed_zero_pair`, `domain_x2_doubles_are_exact` (no change needed) |
| `sparq-engine/src/reduce.rs` (#1476) | bounded slice generator (`len ≤ MAX_SLICE = 8`, `unwind(12)`) | `domain_reducer_slice_is_adversarial_and_survives_the_bound` — adversarial value/id domain + `cover!` that the full-length varied slice survives `unwind(12)` |
| `sparq-core/src/dict.rs` (#1480 id harnesses) | `assume`-restricted, complete-domain, no loop | `domain_id_partition_regions_are_all_nonempty` — four-region non-emptiness + `cover!`s that the round-trip assume-domain reaches its `INLINE_MAX` boundary and zero |
| `sparq-core/src/dict.rs` (mmap validators, sq-ueuk) | bounded symbolic byte buffer, `unwind(28)` | `domain_dict_validator_accepts_the_empty_dict` — accept path reachable in-domain; deep-record accept path pinned by the existing `validate_dict_bytes_seam_accepts_valid_and_rejects_corruption` unit test |
| `sparq-vectors/src/store.rs` | bounded symbolic byte buffer, `unwind(40)` | SPLIT form (see below): compile-time `MAX_LEN >= HEADER_LEN` binding in `kani_proofs` + the `domain_bounded_buffer_contains_an_accepted_store` unit test (a concrete minimal well-formed `.spqv` validates `Ok` on every `cargo test`) |

**The store suite needed the SPLIT form — and surfaced an honesty correction.** A
`#[kani::proof]` accept-path pin for `open_from_bytes` does not terminate practically under
Kani 0.67 even on a fully CONCRETE buffer (measured locally on the work box, non-canonical:
two runs each cut at the 20–25-minute timeout, dominated by CBMC memory-model churn in the
`AlignedBytes` raw-pointer copy + the error-arm `format!`/alloc machinery). The same cost
class affects the PRE-EXISTING totality harnesses: the nightly lane log (run 28701008984,
2026-07-04, main) shows `open_validated_v2_tail_never_panics` **VERIFICATION FAILED** on a
Kani *unsupported-construct* check (a reachable `__rust_alloc_error_handler` foreign call)
and `open_from_bytes_never_panics` reaching no verdict before the job was cut — i.e. the
`sparq-vectors` "PROVED (bounded)" claim is **currently NOT being re-established by CI**
(§7 caveated accordingly; per-harness lane timeouts are bead `sq-otpxg`, store-harness
viability under current Kani is follow-up work). Where a Kani self-check is cost-prohibitive,
the SPLIT form is the sanctioned fallback: a compile-time `const` binding ties the *domain*
claim to the Kani build (red at kani-build time if the domain is re-scoped below the
interesting input), and a plain `#[test]` pins the concrete accept path on every PR — the
input is concrete, so native execution checks the identical property, and the buffer remains
inside the symbolic harnesses' domain.

### 5.2 STANDING HARNESS-BRIEF TEMPLATE (every future proof-program bead inherits this)

A proof-program bead's `bd` body carries `{crate, model_tier, invariant, acceptance_test}`
plus these standing clauses — copy them verbatim; downstream `verify` is mechanical against
them:

1. **Harness-only diff.** Runtime logic is byte-unchanged; the whole change lives in a
   `#[cfg(kani)]` module. (Keeps the fleet's mechanical verify objective and merge risk ~0.)
2. **State the tier and the bounds.** PROVED (complete-domain) vs PROVED (bounded) with the
   explicit `unwind` / length / alphabet bounds, per §1. No "proved" without its bounds.
3. **Mutation spot-check.** Name one local perturbation of the runtime code that makes the
   harness go RED (documented in the PR body). A harness that cannot fail launders confidence.
4. **Domain-coverage self-check (MANDATORY — §5.1).** Ship at least one `#[kani::proof]`
   self-check proving the suite's interesting inputs (a) are genuinely adversarial (exact-image
   / domain pinning over the constants) and (b) survive the bounds (`kani::cover!` witness
   survival, or a concrete accepted input for a totality harness). Keep it CHEAP — the smallest
   `unwind` its loops need — so it does not inflate the nightly lane. If the harness form is
   cost-prohibitive under the current Kani (measure it; > a few minutes is prohibitive), use
   the sanctioned SPLIT form (§5.1, the store case): a compile-time `const` binding of the
   domain claim inside the `#[cfg(kani)]` module + a plain `#[test]` pinning the concrete
   witness — never silently drop the obligation.
5. **Acceptance.** `cargo kani -p <crate> [--features <f>] --harness <name>` green for every
   harness INCLUDING the self-check, plus `clippy -D warnings` and the crate's `cargo test`
   green in both feature states (the `#[cfg(kani)]` module is stripped from those, so they only
   confirm no runtime-code drift).

## 6. NOT-TRACTABLE-NOW ledger (honesty over ambition)

| Deferred item | Why not now | Tier of record meanwhile |
|---------------|-------------|--------------------------|
| Full SPARQL-evaluation correctness vs W3C semantics (Lean/PAG-algebra formalization) | Expert-months before theorem one; the `sparql_noir` precedent already deferred it as not-cost-justified | Conformance suites + differentials |
| Join-kernel equivalence proofs (B-3) | `SmallVec`/hashing state space; bounded merge-join-vs-nested-loop Kani at tiny sizes is *plausible next wave*, not Phase 1 | `yannakakis`/`semijoin` differentials |
| Optimizer rewrite soundness (B-4: GOO/DPccp, filter pushdown) | Plan-space equivalence proof intractable; rewrites are inlined, no pure rewrite seam to harness | `dp_planner_differential.rs` + conformance |
| Aggregation full-path semantics (B-6) | Scalar path interleaved with expression eval; no clean pure seam | Conformance + `vectorized_exec_differential.rs` |
| `compare_terms` laws for the engine's real `Value` impl | The Phase-1 proof covers the shared algorithm over a model; `Value` carries engine-private state — next wave once the model harness exists to copy | Unit tests + W3C ORDER BY conformance |
| Dict full-term `intern`/`lookup` bijectivity (C-2) | String-heavy — Kani cost prohibitive | Unit round-trips + hash-consing structure |
| WAC/ACP spec-parity as a *proof* (A-4) | No machine-readable Solid WAC/ACP semantics to prove against; the corpora *are* the spec proxy | `conformance_wac.rs` / `conformance_acp.rs` |
| Parser round-trip fixpoints (C-4) | A testing gap, not a proof target; grammar-level proofs out of scope | Fuzz (robustness only) — a future round-trip test bead is worth filing on its own merits |
| Creusot/Prusti deductive adoption | Annotation burden on a moving codebase; no property here needs unbounded loop invariants Kani can't reach | — |
| ZK/MPC security properties | **Out of scope by rule**: functional properties only; security claims are `sq-qhy4`-gated (external audit), full stop | Internal re-audit + differential harnesses, labeled as exactly that |
| Making the Kani lane merge-blocking | Deliberate: keep it informational until the Phase-1 harnesses have soaked (flaky-proof risk on toolchain bumps); revisit as a ratchet decision after they are stable | Nightly informational lane |

## 7. Assurance-walkthrough summary (the lines `sq-wir9k` can cite)

One line per surface: what is PROVED / DIFFERENTIALLY TESTED / CONFORMANCE-PINNED, and the
TCB. Entries marked *(Phase 1, pending `sq-sqtk2.N`)* are **not true yet** — they become
citable only when the bead's PR merges; everything unmarked is true today.

- **WAC/ACP authorization (`sparq-solid`)** — PROVED (bounded): fail-closed structure,
  deny-wins algebra, walk termination *(Phase 1, pending `sq-sqtk2.1`)*; EXHAUSTIVELY TESTED:
  nearest-ancestor walk over a bounded alphabet *(same bead)*; CONFORMANCE-PINNED: WAC + ACP
  decision parity corpora (today). TCB: Kani/CBMC + harness bounds + corpus fidelity.
- **SPARQL term ordering (`sparq-substrate`)** — PROVED (bounded, over a model
  `CompareTerm`): reflexivity/antisymmetry/transitivity/within-class totality *(Phase 1,
  pending `sq-sqtk2.4`)*; CONFORMANCE-PINNED: W3C ORDER BY behavior (today). TCB: Kani/CBMC +
  model fidelity; the engine `Value` instance is NOT covered by the proof (ledgered).
- **Aggregation fast path (`sparq-engine/reduce.rs`)** — PROVED (bounded length,
  complete over values): columnar reducers ≡ independent scalar reference *(Phase 1, pending
  `sq-sqtk2.3`)*; DIFFERENTIALLY TESTED: columnar vs scalar engine paths (today). TCB:
  Kani/CBMC + slice bound + reference-as-spec.
- **Id substrate (`sparq-core/dict.rs`)** — PROVED (complete-domain): inline-id round-trip +
  id-partition disjointness *(Phase 1, pending `sq-sqtk2.2`)*; PROVED (bounded): B5 dict-file
  validator never panics — verified locally; NOTE the nightly lane has not been reaching the
  dict harnesses recently (the single job runs `sparq-vectors` first and stalls there — §5.1;
  lane repair is `sq-otpxg`); DIFFERENTIALLY/UNIT TESTED: full-term round-trips (today).
  TCB: Kani/CBMC.
- **Vector store (`sparq-vectors`)** — harnesses EXIST for "`.spqv` open never panics on
  hostile bytes" but the claim is **not currently re-established by CI** (sq-og8u8 honesty
  correction, 2026-07-05: the nightly lane shows `open_validated_v2_tail_never_panics`
  failing a Kani unsupported-construct check and `open_from_bytes_never_panics` exceeding
  the budget — see §5.1; lane timeout handling is `sq-otpxg`). Tier of record meanwhile:
  UNIT/EXHAUSTIVE tests incl. the `domain_bounded_buffer_contains_an_accepted_store`
  accept-path pin + the corrupt-store rejection suite. TCB when the harnesses verify again:
  Kani/CBMC + buffer-size bounds.
- **Query evaluation at large (joins, optimizer, modifiers)** — DIFFERENTIALLY TESTED (DP vs
  greedy; WCOJ/semijoin vs baseline; vectorized vs scalar) + CONFORMANCE-PINNED (W3C suites)
  (today). No proof claim. TCB: baseline/oracle correctness + suite coverage.
- **Parsers/loaders** — FUZZED (robustness) + Miri/ASan on the unsafe surface (today).
  No output-correctness proof claim.
- **ZK/MPC estate** — NO security claim from this program. Functional tier: differential
  harnesses vs independent oracles (M1; PR #1471 open at time of writing); internal re-audit
  done; **external accredited-cryptographer sign-off PENDING (`sq-qhy4`)**; `sparq-mpc` is
  honest-majority semi-honest only. TCB: not applicable — the point is that no relying party
  should treat any of it as sound before the audit.

## 8. Decisions made (proceed-and-document)

1. **Kani-first, no new proof toolchains.** The lane, the toolchain pin, and the in-repo
   harness idiom already exist; every Phase-1 property fits bounded/complete-domain model
   checking. Creusot/Prusti/Lean/hax add cost without adding reachable properties (§4.1).
2. **Harness-only diffs as the load-bearing bead invariant.** No Phase-1 bead may touch
   runtime logic; that keeps the fleet's verify mechanical and the merge risk near zero.
3. **One crate per bead, `kani.yml` owned by exactly one bead.** The CI file is the shared
   resource every harness bead would otherwise collide on; bead 5 owns it and is
   dependency-sequenced behind beads 1–4.
4. **The lane stays non-blocking for now** (ledger item) — soak first, ratchet later.
5. **Tiering**: security-adjacent or subtlety-trapped harnesses (authorization, the 2^53
   compare tier) are opus; fully-specced integer/slice harnesses are sonnet; the YAML edit is
   haiku. Cheapest tier that is still *sound* — per the collaboration-tier doctrine.
