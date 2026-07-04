// [OPUS-4.8] sq-gum8 — the paper-factory registry. ONE source of truth for the /papers
// index, the per-paper static routes (generateStaticParams), the sidebar nav, and the PDF
// build step (scripts/build-papers.mjs reads `slug` + `source` from this file).
//
// To register a new paper: add an entry here + a site/papers/<source>.typ. Everything else
// (index card, route, nav link, PDF + HTML build) is data-driven off this list.
//
// HONESTY: `status` is the paper-factory readiness verdict from
// research/paper-contributions-inventory.md. A C-family (ZK/MPC) paper would be `wip-arxiv`
// and must cite sq-qhy4; neither pilot is C-family. Numbers inside each paper are gated to
// `environment: canonical` by papers/_lib/bench.typ + scripts/build-papers.mjs.

export type PaperStatus = "publishable-now" | "wip-arxiv" | "draft";
export type PaperFamily = "A" | "B" | "C";

export interface Paper {
  /** URL slug AND the basename of the built PDF/HTML artifacts. */
  slug: string;
  /** The .typ source under site/papers/ (relative path). */
  source: string;
  title: string;
  /** One-line summary for the index card + page subtitle. */
  blurb: string;
  /** Authors; rendered as "anonymized" by the .typ when an anon build is requested. */
  authors: string;
  /** Target venue / track (honest target, not an acceptance). */
  venue: string;
  status: PaperStatus;
  family: PaperFamily;
  /** What the headline evidence is — surfaced on the card so the framing is honest up front. */
  evidence: string;
}

export const STATUS_LABEL: Record<PaperStatus, string> = {
  "publishable-now": "Publishable now",
  "wip-arxiv": "WIP · arXiv only",
  draft: "Draft",
};

export const STATUS_VARIANT: Record<
  PaperStatus,
  "success" | "warning" | "muted" | "default"
> = {
  "publishable-now": "success",
  "wip-arxiv": "warning",
  draft: "muted",
};

export const FAMILY_LABEL: Record<PaperFamily, string> = {
  A: "Systems / DB",
  B: "Semantic Web",
  C: "Crypto (WIP)",
};

export const PAPERS: Paper[] = [
  {
    slug: "filtered-ann",
    source: "filtered-ann.typ",
    title:
      "Filter-as-Query: Filtered Approximate Nearest-Neighbour Search over SPARQL, where the Filter is an Exact BGP over the Engine's Own Dictionary Ids",
    blurb:
      "An RDF-native filtered-ANN integration: the filter on a vector-neighbour variable is the join-connected sub-BGP of the query itself, evaluated exactly by the engine and materialised as an id-mask over the shared dictionary-id space (no metadata mirroring, no boundary id translation), with the pre≡post answer contract enforced as machine-checked invariants — exact path by construction, approximate path preconditioned and verified on broad-mask fixtures. Related work covers the 2023–26 filtered-ANN wave incl. the engine-integrated systems (VBASE/NaviX) it is closest to; the performance evaluation is pre-registered (baselines, workloads, falsification criteria) pending the canonical runner.",
    authors: "Jesse Wright · the sparq project",
    venue:
      "EDBT short / demo or ESWC in-use/resources — results-free systems description; research-track submission deferred until the pre-registered evaluation is executed",
    status: "draft",
    family: "A",
    evidence:
      "Deterministic only: recall floors (0.95 unfiltered, 0.90 filtered — sanity checks, labelled as such) + asserted pre-filter ≡ post-filter equivalence (single / transitive / cyclic; exact path unconditional, approximate path broad-mask) + the cost-model crossover constant. No latency/throughput claim; the performance evaluation is pre-registered but unexecuted (blocked on the canonical runner).",
  },
  {
    slug: "solid-acl-conformance",
    source: "solid-acl-conformance.typ",
    title:
      "Library-Level Solid Access-Control Conformance: WAC and ACP Decision-Parity Ratchets Joining a Cross-Family Scoreboard",
    blurb:
      "A library-level conformance signal for both Solid access-control models: a per-construct, fail-closed-checked decision-parity corpus for WAC and for ACP, each ratcheted to a monotone floor and registered as a row of the single cross-family scoreboard with a drift guard. The honest evidence is conformance, not speed, and no security property is claimed; HTTP / CTH wire conformance is explicitly out of scope.",
    authors: "Jesse Wright · the sparq project",
    venue: "ISWC resources / in-use track (or demo)",
    status: "publishable-now",
    family: "B",
    evidence:
      "Deterministic conformance ratchet floors: Solid WAC decision parity (12) and Solid ACP decision parity (12), as the two newest rows of a 7-suite cross-family scoreboard totalling 3442, each floor CI-enforced, monotone, and guarded against drift. No latency claim and no security/soundness claim; library-level decision parity only, not HTTP/CTH wire conformance.",
  },
  {
    // [FABLE-5] sq-gum8.3-odrl REVISION 2 (PR #1330 review response): ONE track picked (ESWC
    // research track — the "in-use" framing is dropped: library-level, single-node, no
    // deployment/users); the WAC/ACP floors reframed as CONTEXT for the target layer, not
    // bridge evidence (the only direct bridge evidence is the four invariants); related work
    // extended to the decision-caching / authorization-recycling / materialised-view
    // neighborhood for C3; the conflict default corrected against ODRL IM 2.2 (`invalid` is
    // the spec default — the bridge hard-wires `prohibit` and cannot honour `perm`/`invalid`,
    // a first-class limitation); "pre-registered" corrected to "specified" (no registry
    // deposit exists); worked end-to-end example + artifact statement added. Status stays
    // `draft` until the §5.3 comparative study runs — the explicit bar for submission.
    slug: "odrl-policy-bridge",
    source: "odrl-policy-bridge.typ",
    title:
      "An ODRL Policy Bridge for SPARQL Access Control: Fail-Closed Compilation of Usage Policies into a Queryable Solid Access-Control View",
    blurb:
      "Compile, don't co-evaluate: a matched ODRL Permission/Prohibition compiles into the same triples the engine's existing, queryable WAC/ACP view already understands — no second enforcement engine, and every compiled decision is auditable, provenance-tagged RDF (the paper walks one policy end-to-end: Turtle policy → compiled auth/provenance triples → SPARQL audit query and its result). The contribution is the fail-closed lifecycle discipline compilation demands: ODRL's prohibit conflict strategy realised structurally by allow-minus-deny set subtraction (the perm and spec-default invalid strategies are not representable — a disclosed, first-class limitation), asymmetric three-valued deny retraction on policy refresh (positioned against decision caching, authorization recycling, Zanzibar-style consistent authorization, and materialised-view maintenance — not only ODRL enforcers), per-session re-checked conditional grants with safe one-shot fallback, and atomic count budgets. Evidence, honestly: four machine-checked invariants are the only direct bridge evidence; the WAC/ACP decision-parity floors are context for the pre-existing target layer, not bridge evaluation; the specified comparative decision-agreement study vs ODRE/OAC has not run (hence draft — not submittable until it has). The federated ODRL→MPC / ODRL-Duty→ZK composition is deferred, unbuilt, and claims nothing (sq-qhy4).",
    authors: "Jesse Wright · the sparq project",
    venue:
      "ESWC research track (policy) — draft; not submittable until the §5.3 comparative study runs",
    status: "draft",
    family: "B",
    evidence:
      "Honestly partitioned. Direct evidence (all of it, today): deterministic, test-proven answer-safety invariants of the bridge (prohibit-strategy set subtraction correct through unchanged enforcement; asymmetric fail-closed deny retraction; recipient constraints persist as re-checked conditional grants with one-shot fallback; atomic stateful count enforcement). Context — NOT bridge evidence: the CI-enforced, drift-guarded Solid WAC (12) and ACP (12) decision-parity ratchet floors of the pre-existing target layer. PENDING — the blocking gap to submission: a fully specified (not pre-registered: no registry deposit exists) decision-agreement study vs the ODRE enforcement engines and an OAC-style matcher over the cited systems' own policy corpora. No latency claim; no novel-semantics claim; no in-use claim; the federated/ZK disclosure half is explicitly deferred (research-grade crypto, not externally audited; sq-qhy4).",
  },
  {
    slug: "cozk-witness-validation",
    source: "cozk-witness-validation.typ",
    title:
      "Cannot Certify, So We Encode the Obligation: A Collaborative-zk-SNARK Re-Audit as a Witness-Validation Negative Result for Federated SPARQL",
    blurb:
      "An adversarial re-audit of an engine's intended collaborative (multi-prover) zk-SNARK path against the CRYPTO'25 (eprint 2025/1026) failure modes. The honest disposition is a negative result: the path is unbuilt, so soundness cannot be certified and every lens is RE-OPEN. The durable output is R-WV, a witness-validation-before-proving test obligation encoded as a build-time gate. NO security, privacy, or attestation property is claimed.",
    authors: "Jesse Wright · the sparq project",
    venue: "PoPETs / a security workshop (negative-result · security-engineering lessons)",
    status: "wip-arxiv",
    family: "C",
    evidence:
      "Negative result — asserts NO proven security/privacy/soundness/attestation property (the collaborative path is unbuilt: 6 proof/attestation entry points fail closed with NotYetImplemented). Committed structural counts only: 4 re-audit lenses, all RE-OPEN; a 5-clause R-WV witness-validation obligation encoded as a build-time gate; 12 prior single-prover findings under the open external-audit gate. Estate is research-grade and not externally audited; cites the gates sq-qhy4 (external single-prover audit, open) + sq-9hrn (coZK re-audit). No performance claim.",
  },
  {
    slug: "fo-km-agent",
    source: "fo-km-agent.typ",
    title:
      "Formal Ontologies for LLM-Agent Knowledge Management: schema.org vs gUFO vs DOLCE",
    blurb:
      "A controlled pilot asking whether typing a project knowledge graph under a formal upper ontology helps an LLM agent answer knowledge-management questions — and which ontology. Holding instance data, agent, tasks, and grading fixed across four committed typing overlays in a fully-crossed single run, two overlays beat the untyped baseline — schema.org-as-top by the largest margin, DOLCE modestly — while gUFO fell below it. The overlay ranking is consistent with LLM training-data fluency, stated as a correlational hypothesis with its confounds named (overlay verbosity/closure noise, hand-authored overlay quality), not a demonstrated mechanism. Every accuracy figure is an indicative single-run measurement from one small agent model (Claude Haiku); a multi-run scale-up, to be pre-registered before it runs, is the stated gate before any venue submission.",
    authors: "Jesse Wright · the sparq project",
    venue:
      "ISWC / ESWC research (empirical) after the scale-up (to be pre-registered); K-CAP or an LLM+KG workshop as a pilot",
    status: "draft",
    family: "B",
    evidence:
      "Indicative-only pilot: deterministically-graded answer accuracy of a non-deterministic LLM agent (Claude Haiku, one fresh instance per condition-task pair) over a committed 16-task corpus fully crossed with 4 committed ontology-overlay conditions (bench/fo-km). The only canonical records are structural corpus counts (16 tasks, 4 conditions); every accuracy/abstention figure is environment=indicative (single run, heuristic gold-key grading, dev work-box) and is structurally barred from headline citation. No latency claim, no significance claim.",
  },
  {
    // [FABLE-5] sq-gum8.4 — Paper C-SoK: the systematization paper over the verifiable/
    // confidential federated-SPARQL estate (zkSPARQL single-prover lane + MPC-SPARQL lane +
    // attested-input binding). C-family, so wip-arxiv and it cites the open external-audit
    // gate sq-qhy4 throughout; it asserts NO security/privacy/soundness/attestation property.
    slug: "verifiable-fed-sparql",
    source: "verifiable-fed-sparql.typ",
    title: "SoK: Systematizing Verifiable and Private Federated SPARQL",
    blurb:
      "A systematization of verifiable and confidential federated SPARQL under a stated method (inclusion criteria plus a dated search protocol) along three independent axes — cryptographic mechanism (single-prover ZK over committed graphs / MPC evaluation over secret-shared graphs / attested-input binding), prover topology (one holder vs N mutually distrusting holders), and adversary model (semi-honest / covert / malicious, crossed with output guarantees under Cleve's bound) — axes assembled from established MPC and collaborative-proof taxonomy, applied jointly to federated SPARQL. Contributes an operator-level capability matrix cross-referenced to the published systems, exposing the disclosed/hidden two-regime split and the disclosed-key-join cost advantage global IRIs give RDF — analyzed together with its privacy price (the cleartext join disclosing exactly the cross-source linkage that source-unlinkability protects); a catalogue of settled negatives (Cleve, the post-quantum boundary of the signature and SNARK layers — with the hash-based Poseidon2 commitment binding stated as PQ-resilient, blank-node cross-graph joins, in-circuit entailment) distinguished from merely vacant cells; and a method-bounded statement of the open frontier — verifying issuer signatures over a secret-shared witness inside one source-unlinkable collaborative proof, a composition no publication in the searched peer-reviewed venues or preprint archives instantiates as of the stated survey dates. Anchored in one research-grade estate whose capability tiers are self-reported documentary provenance (a stated independence limitation): single-prover layer not externally audited (sq-qhy4, open), MPC layer semi-honest by default, collaborative-proof path unbuilt and fail-closed. No security property is claimed and no wall-clock number appears.",
    authors: "Jesse Wright · the sparq project",
    venue:
      "PoPETs / IEEE S&P SoK track — WIP; C-family, so arXiv-only until the external audit gate (sq-qhy4) and the frontier's publication-absence claims are re-verified at submission time",
    status: "wip-arxiv",
    family: "C",
    evidence:
      "Systematization — asserts NO proven security/privacy/soundness/attestation property and cites no wall-clock measurement. The only build-injected counts are the deterministic structural facts already gated as canonical for the C-family estate (fail-closed collaborative entry points, re-audit lenses all RE-OPEN, witness-validation obligation clauses, prior single-prover audit findings under the open gate sq-qhy4). Capability tiers are documentary (spec corpus + reconciled capability review), stated as such in the paper's limitations.",
  },
  {
    // [SONNET-4.6] sq-gum8.9 — Register the SPARQL logic-bug testing paper. Status is
    // `draft`: the merged harness (sq-gum8.6, crates/sparq-metamorph) is the committed
    // instrument, but the cross-engine bug-hunting campaign (bead sq-gum8.11) has NOT run
    // yet. No third-party bug is claimed anywhere; every campaign table in the .typ is an
    // explicitly marked PLACEHOLDER. Evidence keys wired: metamorph.selftest_oracles,
    // metamorph.selftest_seeds, metamorph.grammar_exclusion_seeds (all deterministic, canonical).
    // Campaign keys (confirmed/reported/rejected) are pending sq-gum8.11 and NOT wired here.
    slug: "sparql-logic-bugs",
    source: "sparql-logic-bugs.typ",
    title:
      "Reifying the Error: Metamorphic and Differential Logic-Bug Testing for SPARQL Engines",
    blurb:
      "SPARQL has no dedicated logic-bug testing work. We re-derive TLP and NoREC for SPARQL by reifying its third evaluation outcome — a type error, not a value — with the language's only error-absorbing forms, yielding a partition that provably recomposes the unpartitioned query under the SPARQL 1.1 spec. The merged instrument (crates/sparq-metamorph) includes a TLP + NoREC + differential oracle suite whose self-tests assert non-vacuity against a seeded wrong-result mutant on the real engine. This is a first draft against the merged instrument only; the cross-engine bug-hunting campaign has not run, every campaign table is an explicit placeholder, and the paper's honest publishability condition — previously-unknown, developer-confirmed bugs in third-party engines — is not yet met.",
    authors: "Jesse Wright · the sparq project",
    venue:
      "ISSTA 2027 (testing track) — first choice; PVLDB Vol 20 rolling or FSE 2027 — second choice. Draft; not submittable until the campaign (bead sq-gum8.11) yields confirmed third-party bugs",
    status: "draft",
    family: "A",
    evidence:
      "Deterministic instrument self-tests only (no campaign results yet): 3 oracle types (TLP, NoREC, differential) each asserting non-vacuity against a seeded wrong-result mutant on the real sparq engine; oracle correctness verified across 50 generated seeds on the pristine engine; grammar exclusion list verified across 200 seeds (no banned non-deterministic construct). Campaign-dependent evidence (bugs confirmed/reported/rejected) is pending bead sq-gum8.11 and is shown as an explicit PLACEHOLDER in the paper. No third-party bug is claimed.",
  },
];

export function paperBySlug(slug: string): Paper | undefined {
  return PAPERS.find((p) => p.slug === slug);
}
