# ZK proving performance landscape — synthesis for sparq derived credentials

Status: **synthesis of 9 sub-surveys, June 2026.** Companion to
`research/zkp-query-proofs-plan.md` (v2), which defers proof-system selection
to this document. All sub-surveys were run 2026-06-12 with live web research;
their full texts (with per-claim URLs) are preserved in the session
transcript; the key numbers and their provenance are reproduced here.
**Provenance discipline**: every load-bearing number is tagged
**[vendor]** (project's own blog/repo, unreplicated), **[independent]**
(third-party measurement), or **[academic]** (peer-reviewed, usually
author-measured). Where evidence is vendor-only or unpublished, this
document says so explicitly rather than rounding it up to fact.

## 0. The workload that matters

From the v2 plan (§1.1, §2.4, §4): a holder proves, **on their own M1-class
laptop, phone, or browser tab**, that a SPARQL query result holds over the
merge of **k ≈ 2–5 named graphs** of **tens of triples each** (~40–100
triples total in-circuit), with **1–3 issuer signatures** verified in-circuit
against a disclosed key set. The planned circuit budget is roughly
**60k–100k gates (~2^16–2^17 constraints)** for the cooperative-issuer path
(Poseidon2 flat-hash commitment recomputation + Schnorr-embedded signatures
+ small query machinery), ballooning to **+270–430k gates per credential**
if standard VC-DI suites (SHA-256 over RDFC10 N-Quads + non-native curve)
must be verified in-circuit. Latency target: ≤5 s native, browser-feasible;
proofs verified off-chain by a verifier running plain code over a manifest
(no EVM gas constraint). This scale assumption drives every verdict below:
**most of the ZK performance arms race (real-time Ethereum proving, GPU
clusters, ASICs) is solving a problem three to six orders of magnitude
larger than ours, and its winners are mostly the wrong tools here.**

## 1. Executive summary

**Recommendation: stay on Noir/UltraHonk (bb.js) for the primary path.**
The evidence supports it, with sharper reasons than inertia:

1. **It is fast enough, measured, on exactly our hardware.** The only
   neutral M1 client-side benchmark suite (PSE csp-benchmarks, updated
   2026-06-10, AWS mac2.metal = M1/16 GB) puts Barretenberg/Noir at
   **610 ms prove / 87 MB / 14.3 KB proof** for a SHA-256/128B statement
   [independent]. Our Poseidon2-heavy circuits are *more* bb-friendly than
   SHA-256 (74 gates/permutation, pinned in bb master). Independent browser
   data: bb.js proved a **~2M-constraint** p256 circuit **<3 s in-browser on
   M1 Air** — ~20x our gate budget — while circom/snarkjs could not run that
   circuit in a tab at all [independent, recovered via snippets — see §2.6
   caveat]. The 5 s stage-1 exit criterion has real headroom.
2. **Nothing that beats it on speed beats it on fit.** Binius64 (41 ms) and
   Spartan2 (73 ms) beat bb by ~10x on the M1 hash benchmark [independent],
   but Binius64 is an orphan (Irreducible shut down 2025-11-12; ZK privacy
   was still a roadmap item) and neither has a SPARQL-shaped frontend, a
   browser proving story as mature as bb.js, or our existing `sparql_noir`
   circuit investment. The 610 ms vs 41 ms gap is irrelevant when the UX
   budget is 5 s and the alternative costs a toolchain rewrite.
3. **Everything heavier is disqualified by measurement.** zkVMs: RISC0
   needs **18.5 s + 1.47 GB** for one small hash statement on M1
   [independent] — 2–3 orders of magnitude worse than purpose-built
   circuits; "real-time" zkVM claims all assume 16–160 datacenter GPUs.
   GPU: every independent Apple-GPU measurement puts the CPU/GPU MSM
   crossover at **~2^20–2^22 points**; our MSMs are ~2^16–2^17, squarely
   where Apple GPU is *slower* than CPU [independent]. bb has **no shipped
   GPU prover** anyway. Folding: ~10k-constraint per-step overhead and a
   final compression step that took **>2 min on desktop and was infeasible
   on phones** in the one production deployment measured [independent].
   FPGA/ASIC: nothing verifiably shipping; the one vendor that tried
   (Irreducible) publicly concluded "the market isn't ready" and exited.
4. **Two real hedges, both hash-driven.** (a) If the interop cliff bites
   (Q3 → in-circuit standard VC-DI verification), the production-proven
   answer is **Longfellow-zk-style Ligero+sumcheck** — Google Wallet ships
   it, it proves ECDSA-P256+SHA-256 mDoc statements in *hundreds of ms on
   phones* [vendor-authored paper, IETF-reviewed, independent Rust
   reimplementation exists]. (b) If k and the predicate count grow,
   **data-parallel GKR** is structurally ideal for "N copies of one small
   predicate circuit" — but the leading implementation (Expander) is
   AGPL-3.0, has no browser story, and every headline number is
   vendor-only. Watch, don't adopt.
5. **The watch-item is CHONK/ClientIVC**, Aztec's client-side folding stack
   (11.8k-gate inner verify vs 682k for Ultra-in-Ultra recursion
   [vendor docs]): purpose-built for phones/browsers, but **not usable for
   non-Aztec Noir programs as of mid-2026** [verified from docs/CLI state].
   When it opens up, it changes the recursion/compression economics of §4B
   of the plan.

## 2. Per-survey findings

### 2.1 Plonky3 vs Stwo (small-field STARKs)

- **Stwo/S-two (StarkWare)** is the production-mature small-field STARK:
  live on Starknet mainnet since 2025-11-03 (every block), v2.0.0 on
  crates.io (Jan 2026), **Apache-2.0** (verified; the restrictive-license
  memory belongs to the older Stone/Polaris era). Vendor throughput claims:
  >500k–620k Poseidon2/M31 hashes/s at 128-bit on laptop CPUs [vendor];
  a continuous public bench dashboard exists for reproduction.
- **It has the assets our deployment shape needs**: a **WebAssembly-SIMD
  backend** (plus NEON — M1 first-class; StarkWare reported wasm loses only
  ~30% vs native NEON on M1 Max [vendor]); **production recursion** since
  Mar 2026 (circuit-based recursive verification, ~1 min → ~3 s on a laptop
  [vendor]); and the best real-world mobile evidence in the field:
  **FibRace** (arXiv 2510.14693, Oct 2025; Cairo-M, Stwo-derived): 2.19M
  client-side proofs, 6,047 users, 1,420 device models — **most modern
  smartphones prove in <5 s**, ≥3 GB RAM needed, A19 Pro/M-series fastest
  [independent-ish: KKRT/Hyli, StarkWare-adjacent].
- **Plonky3 (Polygon)** is an actively maintained toolkit (commits the day
  of the survey; dual MIT/Apache-2.0; Least Authority audit 2024) with a
  dueling vendor claim (>2M Poseidon2/s on M3 Max, Dec 2024 [vendor]) —
  but its ecosystem trajectory is adverse: flagship user SP1 left for its
  own Hypercube proof system (live Feb 2026), and **Plonky3's first-party
  recursion library is explicitly unaudited and not recommended for
  production** (README, checked 2026-06-12).
- The two headline hash-rate claims are dueling vendor numbers at different
  dates with undisclosed parameters; **no neutral head-to-head exists**.
  Neither project publishes in-circuit signature-verification benchmarks —
  exactly the number our workload needs.
- **Verdict for our workload**: if we ever leave Noir, Stwo is the credible
  STARK destination (license, wasm, mobile, recursion all check out), but
  it means hand-writing AIRs and solving in-AIR signature verification
  ourselves — a rewrite with no measured payoff at 10^5 gates.

### 2.2 Apple Silicon / Metal GPU economics

- **The crossover is measured and it is against us.** mopro's Metal MSM v2
  (Jul 2025, BN254, M3 Air): Apple GPU is **6.5–22.3x slower** than
  arkworks CPU at 2^12–2^14 points, still 1.8x slower at 2^20, only
  1.6–2.8x faster at 2^22–2^24; their conclusion — GPUs excel near 2^26+
  while mobile circuits are 2^16–2^20 [independent]. A separate community
  deep-dive reached the same conclusion (hybrid CPU+GPU only ~30–40% over
  CPU-only) [independent]. EZKL ships an explicit "GPU off below k≤8 rows"
  switch and measured GPU *regressions* on small circuits [independent].
- Structural caps: NTT+MSM are 70–80% of Groth16 proving (≤3–5x E2E even
  with infinitely fast kernels), witness generation is serial CPU and
  GPU-untouchable [independent analyses].
- CPU is genuinely fast at our scale: rapidsnark Groth16 on iPhone 16 Pro —
  Keccak256 630 ms, SHA256 187 ms; M1 Max Keccak 528 ms [independent,
  mopro/PSE]. Real-world mobile Groth16 identity proofs ≈ 4 s (Anon
  Aadhaar production) [independent].
- ICICLE's Metal backend exists (v3.6+) but production use **requires a
  commercial license** (closed binaries phoning a license server) and the
  primitives gap at launch included Poseidon/Merkle [vendor docs].
  Tachyon (Kroma) is dead — no commits since Nov 2024 [independent, GitHub
  API].
- Cloud GPU economics (4090 ~$0.34–0.40/hr; whole-Ethereum-block proofs
  now sub-penny on 5090 clusters per ethproofs [independent]) are
  irrelevant to holder-side proving by construction: the holder's witness
  is private and the device is the deployment target.
- **Verdict: CPU wins at credential scale, on every independent
  measurement. Do not spend effort on GPU.**

### 2.3 Jolt + folding schemes

- **Jolt** (a16z): vendor claims >500 kHz RISC-V proving on a MacBook,
  ~50 KB proofs, ZK added Mar 2026 with +3 KB and ~no prover cost
  [vendor; no independent post-Twist-and-Shout benchmark exists]. But:
  README says **alpha, not for production**; **the prover does not run in
  WASM** (verifier only); phone proving is an unshipped roadmap item; no
  Poseidon2 precompile; the planned EVM verifier (~2M gas) is unshipped.
  Disqualifying for browser-primary deployment regardless of speed.
- **Folding (Nova family)**: per-step recursion overhead ≈ **10,000 R1CS
  constraints** [academic, eprint 2021/370], so folding a 100–3,000-gate
  predicate is overhead-dominated unless **5–30 predicates are batched per
  step**. The decisive client-side datapoint is Cursive's production Nova
  deployment [independent practitioner, Aug 2024]: 2^13-constraint steps =
  2.2–3.5 s/step in desktop browser, 10–20 s/step on phones, and **final
  SNARK compression >2 minutes on desktop, infeasible on mobile** (~1 GB
  browser memory cap). Their published conclusion matches the math:
  folding is uneconomical for small circuits.
- Sonobe (PSE) is the closest browser-WASM folding path but is explicitly
  "experimental, do not use in production, not audited" with no public E2E
  benchmark [repo state, Jun 2026]. microsoft/Nova: active, MIT, no audit;
  had a full soundness break in 2023 (fixed) [academic].
- **Verdict: matches the plan's §4C downgrade.** At a handful of rows over
  ~10^2 triples there is no row stream to amortize; the final-compression
  cliff alone kills phone deployment. Re-evaluate only via CHONK (which is
  folding, but with the compression problem absorbed by Aztec's stack).

### 2.4 Binius / GKR-class systems

- **Binius is dead as a platform.** Irreducible shut down 2025-11-12;
  original repo archived Sep 2025 at the Binius64 pivot. Binius64 survives
  as community-maintained dual MIT/Apache code (last push 2026-06-12,
  Bain/Paradigm-credited funding) but **ZK privacy, succinct verification,
  and recursion were still roadmap items** when the company died, there
  are no production users, and its headline benchmarks were **partially
  publicly retracted** by Irreducible itself ("several mistakes with our
  benchmarking methodology") [vendor, self-disclosed]. Its 41 ms M1
  SHA-256 number (csp-benchmarks) is real and independent — but you'd be
  building a credential product on an unfinished orphan.
- **Data-parallel GKR is structurally the best match for our circuit
  shape** — this survey's key positive finding. The prover commits only to
  inputs/outputs (sumcheck handles intermediate layers), and SIMD-GKR
  (Libra/Expander) makes verifier cost linear in (batch + circuit size)
  rather than their product: "k identical predicate circuits over
  Poseidon2-committed graphs" is literally the textbook case [academic:
  Thaler 2013 (200x prover speedup on data-parallel circuits); deVirgo/
  zkBridge proved >30k identical sig-verifications in production lineage].
- But the practical vehicle fails our constraints: **Expander (Polyhedra)
  is AGPL-3.0** (copyleft, a real problem for holder-side product code),
  has **no browser/WASM prover**, self-describes as not user-friendly, and
  **every headline number (2.16M Poseidon/s on Ryzen; 4,500 Keccak-f/s on
  M3 Max) is vendor-only with zero neutral replication as of June 2026**
  — their Proof Arena benchmark site is hosted by the vendor that wins it.
  Ceno (Scroll, Apache-2.0, GKR-based zkVM) is real but a zkVM — heavier
  than we need, and Fenbushi found GKR sumcheck is its CPU+memory
  bottleneck [independent].
- Ligero/Brakedown-class: linear-time low-memory provers, **O(√n) proofs**
  (high-hundreds-of-KB to MB at our scale — fine off-chain); Ligetron is
  the only major system genuinely running its prover in-browser
  [academic, IEEE S&P 2024], prover open-source Apache-2.0 and active;
  **Google announced adopting Ligero for a ZK stack** [vendor].
- **Verdict: GKR/Ligero is the named hedge, not the choice.** Structurally
  ideal, strategically unavailable (license/browser/replication). The
  cheap insurance is keeping our commitment scheme (Poseidon2 flat-hash
  per graph) prover-agnostic, which the plan already does.

### 2.5 FPGA / ASIC

- **Nothing is verifiably shipping.** Cysic's C1 ASIC: vendor-only claims,
  no tapeout date/die photo/third-party test, docs slipped to "expect to
  ship in 2026", absent from ethproofs leaderboards; its Dec 2025
  mainnet+token launch is a compute-marketplace play, not silicon evidence
  [independent absence-of-evidence]. Fabric Cryptography: still "Pre-Order"
  buttons ~2 years after "production later this year"; last blog post
  Oct 2024; no independent hands-on exists. Accseal has the strongest
  evidence of real low-volume silicon (Ingonyama tested its Leo chip,
  May 2024) but went quiet after Nov 2024.
- The decisive datapoint is a candid vendor exit: **Irreducible abandoned
  its FPGA business (Sept 2025) stating FPGAs "underperformed GPUs" and
  "the market isn't ready for ASICs"** — and pivoted to client-side CPU
  proving, i.e., to our deployment model [vendor, against interest].
  Meanwhile all of Ethereum real-time proving runs on commodity RTX 5090
  clusters at $0.016–0.06/block [independent, ethproofs].
- **Verdict: irrelevant at credential scale; zero action items. Useful
  only as a paper footnote that the hardware arms race targets workloads
  ~10^6x larger than holder-side credential proofs.**

### 2.6 UltraHonk vs Groth16

- **Proving**: the one solid independent 2025 head-to-head (Base,
  2025-09-02, ~2M-constraint P-256 passkey circuit): **UltraHonk 5–50x
  faster proving than Groth16 stacks** (2.1 s vs 15–50 s on 8 vCPU)
  [independent]. In-browser: **bb.js <3 s for p256 ECDSA on M1 Air**
  (multithreaded; needs COOP/COEP for SharedArrayBuffer), <10 s on a
  modern Android phone; the equivalent circom circuit couldn't run in a
  browser tab at all [independent — Hyli benchmark; **caveat: site was
  unreachable during the survey, numbers recovered via search snippets,
  date ~late 2024 unverified**].
- **Verification/size**: Groth16 wins where we don't compete — ~128–256 B
  proofs and ~350–410k gas vs UltraHonk's ~14–16 KB proofs and ~2.4M gas
  [independent, Base]. Our verifier is off-chain plain code; 14 KB
  manifests are a non-issue. Groth16's costs we'd actually pay: per-circuit
  trusted setup ceremonies, and **GPL-3.0 on snarkjs and rapidsnark**
  (copyleft in a holder-side app); gnark/arkworks are Apache but gnark has
  no NEON-optimized arm64 path.
- **Recursion**: Ultra-in-Ultra ≈ 682k gates per inner verify (plan's
  anchor) — uneconomical; **CHONK/ClientIVC** (HyperNova-style folding +
  Goblin, 11.8k-gate inner verify) is purpose-built for client-side but
  **confirmed not supported for arbitrary non-Aztec Noir programs as of
  mid-2026** (bb CLI has a `client_ivc` scheme; NoirJS/Mopro expose only
  UltraHonk) [repo/docs state]. No published gate count exists for the
  UltraHonk recursive verifier — treat current Honk recursion cost as
  unverified.
- Production: Apache-2.0 (verified at LICENSE level); Aztec Ignition live;
  ZKPassport generates UltraHonk proofs natively on mobile [vendor].
  **Caveat**: a critical proving-system vulnerability was disclosed by the
  Aztec team 2026-03-17 — verify details and patch status before pinning a
  bb version. Noir is at 1.0 *pre-release*, audits pending.
- **Verdict: confirms the plan's choice.** For holder-side proving with
  off-chain verification, UltraHonk wins on every axis we weight; Groth16's
  advantages (proof size, EVM gas) only matter if an on-chain consumer
  appears, at which point a wrap is the answer, not a migration.

### 2.7 zkVMs + neutral benchmarks

- **The neutral M1 benchmark exists and is decisive**: PSE/zkID
  **csp-benchmarks** (results on ethproofs.org/csp-benchmarks, updated
  2026-06-10; AWS mac2.metal = M1, 8 cores, 16 GB; 18 systems). SHA-256/
  128B: **Binius64 41 ms / 147 KB proof; Spartan2 73 ms / 47 KB;
  ProveKit-Groth16 382 ms / 200 B; Barretenberg(Noir) 610 ms / 14.3 KB;
  Miden 3.7 s; RISC0 18.5 s / 1.47 GB RAM** [independent]. Hand-rolled/
  hash-native SNARKs beat zkVMs by **2–3 orders of magnitude** at this
  statement size on laptop CPU. (Caveat: SHA-256/128B is not our exact
  workload; our Poseidon2-native circuits should favor bb *more*, since
  bb's weak spot is bit-oriented hashing.)
- zkVM real-time headlines (SP1 Hypercube 99.7% of Ethereum blocks <12 s;
  OpenVM 2.0 139 MHz) all assume **16+ RTX 5090s** [vendor]; no credible
  2025–2026 result shows a general-purpose RISC-V zkVM proving usefully on
  a phone or in a browser — only verification is routine there.
- **The production credential-ZK precedent is Google Longfellow-zk/libzk**:
  Ligero+sumcheck over ECDSA-P256+SHA-256, **ECDSA proofs in ~tens of ms,
  full ISO mDoc presentation "a few hundred ms on mobile"** [vendor-authored
  paper eprint 2024/2010, but IETF-reviewed (draft-google-cfrg-libzk) and
  **deployed in Google Wallet**; independent Rust reimplementation exists
  (abetterinternet/zk-cred-longfellow)]. This is the strongest evidence in
  the entire landscape that hash-heavy standard-credential verification is
  tractable on phones — and it's exactly the interop-cliff workload (§2.3
  of the plan). Microsoft **Crescent** (Groth16 over existing JWT/mDL
  credentials, prepare-once/show-fast) is the other production-track
  precedent [vendor; eprint timings unverified — page 403'd].
  An SoK on anonymous credentials for EUDI wallets (eprint 2026/330)
  systematizes BBS-style vs Longfellow/Crescent-class approaches — mine it
  for the paper's related work.
- The plan's "7.5 min for a tiny query in a zkVM" baseline [CEUR4085]
  could not be matched to a public source, but minutes-scale zkVM proving
  for small statements on CPU is independently corroborated (RISC0 18.5 s
  for *one hash*; 7m45 s Fibonacci-10M on 64-core EPYC).
- **Verdict: zkVMs stay the baseline to beat, and the beat margin is now
  independently documented at 2–3 orders of magnitude. Longfellow is the
  system to study (and cite) for the standard-suite interop path.**

### 2.8 ICICLE / sppark / cuZK (GPU libraries)

- Independent E2E measurements cap GPU gains at our scale: EZKL — MSM
  kernel 50x but **E2E only ~35% better**, regressions below k≤8
  [independent]; MAYA-ZK gnark+ICICLE — **1.2–3x full-run** speedups,
  far below kernel claims [independent]; ZKProphet (IISWC 2025) — MSM
  speedup collapses from 799x at 2^26 to ~34x at 2^15 against a *slow* CPU
  baseline, NTT transfer-bound, ZK arithmetic stuck on the GPU's 32-bit
  integer pipeline [independent/academic]. Vendor flagship numbers
  (ICICLE-Snark 63–320x) are quoted at 2^22+ with warm caches, and vendors
  publish **no small-size wins** at all.
- ICICLE practicalities: GPU backends are closed binaries; **production
  use requires a commercial license**; repo quiet since Nov 2025 (no
  release in ~11 months) — consistent with pivot or de-prioritization, no
  announcement either way [independent, GitHub API]. The rumored
  Fabric-acquires-Ingonyama event: **no evidence found**; likely confusion
  with the Cornami "computing fabric" partnership.
- cuZK/GZKP/Elastic-MSM remain academic artifacts; shipped GPU codepaths
  are sppark (RISC0, Filecoin — Apache-2.0, actively maintained),
  bellperson, ICICLE.
- **Verdict: expect ~1–3x E2E at 10^5–10^6 constraints on a discrete GPU
  and parity-or-worse below that — and we have neither a discrete GPU in
  the deployment picture nor a bb GPU backend to use one.**

### 2.9 bb GPU / zkVM GPU provers

- **Barretenberg has no shipped GPU prover** (README/docs contain zero
  GPU/CUDA/Metal content; the historical third-party CUDA port is
  PLONK-era and dead since 2023). Active 2026 WebGPU MSM work exists in
  aztec-packages (WGSL kernels targeting Apple/Adreno/Mali browser GPUs;
  measured A/B on M2: full-MSM GPU time −7% at logn=17) but sits on
  **unmerged feature branches** — in-flight experimentation, not shipped
  [repo state, 2026-06-12]. Aztec's own Chonk blog: "we haven't even
  merged-in GPU acceleration yet" [vendor]. Aztec production proving is
  CPU-only (prover-agent guidance is cores/RAM, no GPU anywhere).
- SP1/RISC0 GPU paths carry fixed overheads a small proof can't amortize
  (separate GPU-server process; minutes-long first-run JIT kernel
  compilation) and both vendors steer small users to their networks
  [vendor docs]. No vendor publishes a "GPU pays above N cycles"
  threshold — the spread in their own numbers implies small traces see
  the low end.
- The ecosystem consensus for genuinely small client-side proofs is
  explicit: **CPU/WASM** — Aztec targets sub-second phone proving with no
  GPU at all [vendor], FibRace showed phones prove CPU-only in <5 s
  [independent].
- **Verdict: closes the GPU question from the bb side too. The only GPU
  line worth watching is bb.js WebGPU MSM, because it targets exactly our
  browser deployment — revisit if it merges with measured wins at
  logn≈16–18.**

## 3. Comparison table — credential-scale workload (k≈2–5 graphs, ~10^2 triples, 1–3 sigs, M1/phone/browser)

| System | M1-class prove (closest measured anchor) | Browser/mobile prover | Proof size | License | Production status | Fit verdict |
|---|---|---|---|---|---|---|
| **Noir/UltraHonk (bb.js)** | 610 ms (SHA-256/128B, M1, csp-bench [indep]); <3 s in-browser for ~2M-constraint p256 [indep, snippet-recovered] | **Yes** — bb.js multithreaded WASM (COOP/COEP); native mobile via Mopro/ZKPassport | ~14–16 KB | Apache-2.0 | Aztec mainnet; ZKPassport mobile; Noir 1.0 pre-release; Mar 2026 vuln disclosure to check | **Choose.** Only stack with measured browser story + our existing circuits |
| Groth16 (rapidsnark/circom) | 187–630 ms/hash-circuit, iPhone 16 Pro [indep] | Native mobile yes; snarkjs WASM fails >1M constraints [indep] | 128–256 B | snarkjs/rapidsnark **GPL-3.0** | Mature, huge ecosystem | Backup if tiny proofs/EVM ever needed; trusted setup + GPL costs |
| Stwo/S-two | 620k Poseidon2/s on M3 [vendor]; FibRace phones <5 s [indep-ish] | **Yes** — wasm-SIMD backend; mobile proven at scale | no official KB figure | Apache-2.0 | Starknet mainnet (Nov 2025); recursion in production (Mar 2026) | Credible STARK alternative; costs a full AIR rewrite, no sig-verify benchmarks |
| Plonky3 | 2M Poseidon2/s M3 Max [vendor, contested] | No first-party story; recursion **unaudited** | tunable, no figures | MIT/Apache-2.0 | Toolkit; flagship user (SP1) left | Pass — ecosystem trajectory + recursion immaturity |
| Binius64 | **41 ms** (SHA-256/128B, M1 [indep]) | No browser story | 147 KB | MIT/Apache-2.0 | **Orphaned** (Irreducible dead 2025-11); ZK privacy unshipped | Fastest raw number; unownable platform risk |
| Spartan2 | 73 ms (same bench [indep]) | immature | 47 KB | MIT/Apache | Research-grade | Watch via csp-benchmarks |
| Longfellow-zk (Ligero) | ECDSA ~20–60 ms; full mDoc presentation ~100s of ms on mobile [vendor paper, IETF-reviewed] | **Yes** — designed for mobile wallets | larger (Ligero-class) | open (Google) + indep. Rust reimpl | **Google Wallet — THE production credential precedent** | **Hedge #1** for the standard-suite interop cliff |
| Expander (SIMD-GKR) | 4,500 Keccak-f/s on M3 Max [**vendor-only**] | **None** | n/a published | **AGPL-3.0** | zkBridge lineage; no neutral replication | Structurally ideal, strategically blocked; **hedge #2** (watch) |
| Jolt | >500 kHz RISC-V on MacBook [vendor, unreplicated] | **Prover not WASM-compatible**; phone unshipped | ~50 KB | MIT/Apache-2.0 | Alpha, no audit | Pass for browser-primary deployment |
| Nova/Sonobe folding | 2.2–3.5 s/step browser; compression **infeasible on phone** [indep, Cursive] | WASM folding yes; decider is the killer | 0.2–0.6 MB (Spartan-class) | MIT | Unaudited, experimental | Pass; CHONK is the folding path that could work |
| RISC0 / SP1 zkVMs | **18.5 s + 1.47 GB** for one hash on M1 [indep] | Verify-only in browser | 218 KB (R0) | Apache-2.0 | Real-time = 16+ datacenter GPUs | Baseline to beat (by 2–3 orders, documented) |
| GPU accel (any stack) | Apple GPU MSM crossover ~2^20–2^22 [indep]; bb has no GPU prover | n/a | n/a | ICICLE prod = commercial | — | Pass at our scale |
| FPGA/ASIC | nothing verifiably shipping [indep] | n/a | n/a | n/a | Vendor exits + vapor | Ignore |

## 4. Recommendation and hedges

**Primary: Noir/UltraHonk via bb.js (browser) and native bb (M1/mobile),
exactly as the v2 plan assumes.** The independent evidence base: 610 ms M1
proving for a hash-circuit of comparable size [csp-benchmarks], <3 s
in-browser for circuits ~20x our budget [Hyli, snippet-recovered], 5–50x
proving advantage over Groth16 stacks [Base], Apache-2.0 verified,
production credential users (ZKPassport, Google-scale precedent adjacent).
The stage-1 exit criteria (≤5 s native, <2^19 constraints) have margin
against all of it. No measured alternative offers a better
speed × browser × license × toolchain-fit product.

**Hedge 1 — Longfellow/Ligero-class for the interop cliff (plan Q3).**
If verifiers demand standard VC-DI suites verified in-circuit (SHA-256
over N-Quads + P-256/Ed25519), UltraHonk pays ~270–430k gates *per
credential* [plan's judgement arithmetic] and browser proving dies at k≥2.
Longfellow-zk proves precisely that statement class in hundreds of ms on
phones and is shipping in Google Wallet with an IETF draft. The right
move is *not* to migrate sparq's stack but to evaluate a **Ligero-style
side-proof for the signature/commitment layer** (or the BBS+ commitment
bridge from §2.3 of the plan) feeding the disclosed commitment into the
Noir query circuits. Action: when Q3 is decided, run a Longfellow/libzk
feasibility spike before writing any in-circuit SHA-256.

**Hedge 2 — data-parallel GKR for hash-heavy commitment recomputation at
larger k.** If the workload drifts toward many graphs or large
dataset-dump credentials (the §2.2 Merkle fallback territory), GKR's
"commit only inputs/outputs" structure beats Plonkish arithmetization by
roughly the circuit depth factor, and SIMD-GKR amortizes one predicate
circuit over all copies. Blockers today: AGPL (Expander), no browser
prover, vendor-only numbers. Action: none now; re-test the landscape if a
permissively-licensed GKR prover with a WASM target appears, or if
Binius64's community fork ships ZK privacy + a browser story.

**Hedge 3 — CHONK/ClientIVC for compression.** Manifest-of-proofs stays
the posture (plan §4B). The 682k-gate Ultra-in-Ultra recursion remains
uneconomical; CHONK's 11.8k-gate inner verify changes that *when* it is
usable for non-Aztec Noir — it is not, as of mid-2026 [verified].

**Explicitly rejected on evidence**: GPU acceleration in any form at this
scale (independent crossover measurements + no bb GPU prover), zkVMs as
the proving vehicle (2–3 orders of magnitude, independently benchmarked),
Nova-family folding (per-step overhead + phone-infeasible compression),
Jolt (no WASM prover, alpha), Binius-as-platform (dead sponsor),
FPGA/ASIC (nothing shipping), Expander-as-dependency (AGPL, no browser,
unreplicated numbers).

**Honesty ledger** — claims this document relies on that are *not*
independently verified: all Stwo/Plonky3/Expander throughput headlines
[vendor, mutually contested]; the Hyli <3 s browser number
(snippet-recovered, site unreachable); Longfellow's ms-class timings
(vendor-authored paper, though IETF-reviewed and production-deployed);
Jolt's MacBook kHz (no third-party benchmark post-Twist-and-Shout exists);
CHONK "sub-second" targets [vendor]; Crescent timings (eprint
unfetchable). None of these is load-bearing for the primary
recommendation, which rests on csp-benchmarks (M1, neutral, June 2026),
the Base head-to-head, mopro's measurements, and Cursive's production
folding data — all independent. The single most important unmeasured
thing remains **our own workload**: no published benchmark covers
"k Poseidon2 graph-commitment recomputations + Schnorr-embedded sigs +
query modules" — stage 1 of the plan measures it, and nothing here
removes that obligation.

## 5. Revisit triggers

Re-open proof-system selection when any of the following fires:

1. **CHONK/ClientIVC documented for arbitrary non-Aztec Noir programs**
   (watch bb CLI `--scheme client_ivc` docs and NoirJS/Mopro surface) —
   re-run the recursion/compression decision (plan Q8/Q9).
2. **Q3 resolves to in-circuit standard-suite verification** — run the
   Longfellow/libzk spike before implementing; budget comparison vs the
   270–430k-gate UltraHonk path.
3. **bb.js WebGPU MSM merges to `next`/release** with measured wins at
   logn 16–18 on Apple/mobile GPUs — free browser speedup, re-bench.
4. **Workload outgrows the browser ceiling** (~2^19–2^20 constraints):
   dataset-dump credentials, k_max > 8, or in-circuit closure
   materialization (plan §5 I3) — that's the GKR/Ligero re-entry point.
5. **A neutral benchmark covering signature+commitment workloads on
   M1/WASM** appears (csp-benchmarks is adding circuits; watch ECDSA and
   Poseidon2 tracks) showing a permissively-licensed system beating bb by
   >5x on our shape.
6. **Aztec/Noir risk events**: details of the 2026-03-17 proving-system
   vulnerability, Noir 1.0 audit outcomes, or any bb license change —
   re-check before each version pin.
7. **Ecosystem flips**: Expander relicenses or ships WASM; Binius64 fork
   ships ZK privacy + finds a steward; Stwo publishes an in-AIR signature
   verification benchmark competitive at 10^5 gates.
8. **An on-chain verifier requirement appears** — re-open the Groth16
   wrap question (proof size/gas is the one axis where UltraHonk loses).

## 6. Source index (primary anchors)

- PSE/zkID csp-benchmarks (M1 client-side, 18 systems):
  https://ethproofs.org/csp-benchmarks ·
  https://github.com/privacy-ethereum/csp-benchmarks [independent]
- Base ZKP benchmark (UltraHonk vs Groth16, 2025-09-02):
  https://blog.base.dev/benchmarking-zkp-systems [independent]
- mopro performance + Metal MSM v2: https://zkmopro.org/docs/performance/ ·
  https://zkmopro.org/blog/metal-msm-v2/ [independent]
- Cursive folding production retrospective:
  https://cursive.computer/posts/zk-summit-folded [independent]
- FibRace mobile proving study: https://arxiv.org/abs/2510.14693
  [independent-ish]
- Google Longfellow-zk: https://eprint.iacr.org/2024/2010 ·
  https://datatracker.ietf.org/doc/draft-google-cfrg-libzk/ ·
  https://github.com/abetterinternet/zk-cred-longfellow
- ZKProphet GPU characterization: https://arxiv.org/abs/2509.22684
  [independent]
- EZKL acceleration: https://blog.ezkl.xyz/post/acceleration/ [independent]
- ethproofs (datacenter context): https://ethproofs.org [independent]
- Irreducible shutdown/pivot: https://www.irreducible.com/posts/
  irreducible-shutting-down · /reinventing-irreducible [vendor, candid]
- Stwo: https://github.com/starkware-libs/stwo ·
  https://starkware-libs.github.io/stwo/dev/bench/ [vendor + bench harness]
- Barretenberg/bb.js: https://github.com/AztecProtocol/aztec-packages
  (barretenberg/) — README, LICENSE, ts/README, WebGPU MSM PRs #23664/#23724
- Full per-claim citations: the nine sub-survey texts (session transcript,
  2026-06-12), each line individually sourced and vendor/independent-tagged.
