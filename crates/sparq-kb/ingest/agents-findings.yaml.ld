# agents-findings.yaml.ld — the WRITE-PATH authoring form for the PKG Findings tier
# (FO-bridge Phase 4, sq-mztg8.2; epic sq-mztg8; design record
# research/fo-llm-bridge.md §3.4 / §6 Phase 4, #1112). 🤖 SPARQ agent [OPUS-4.8].
# Written while Fable unavailable; flag for re-review when Fable returns.
#
# WHY THIS FILE EXISTS. This is the compact, IRI-free, xsd-free authoring source the
# author (a human, or a schema-constrained LLM) edits. A DETERMINISTIC compiler —
# crates/sparq-kb/ingest/yamlld_compile.py, NOT an LLM — expands it to the
# schema.org-typed PKG Turtle that used to be hand-authored as agents-findings.ttl.
# The generalised precedent is the shipped sec-prop.yaml.ld pattern (typed @context +
# @graph). The compile is gated by the same pkg.shapes.ttl SHACL shapes (the ONLY
# admission control); an ambiguous concept TOKEN is a HARD compile error (the
# write-path analogue of the read-path abstain) — never a silent wrong IRI.
#
# AUTHORING RULES (so neither author nor LLM ever writes raw RDF):
#   - `about` / `source` are concept TOKENS, resolved by the guarded V() resolver
#     against the `concepts:` catalog below (lexical: local-name OR normalised
#     prefLabel). Ambiguous = compile error.
#   - `confidence` is a plain number; the @context types it xsd:decimal.
#   - `assurance` is one of: proven | claimed | conjectured (-> secx: enum).
#   - `verdict` is one of: yes | no | partial (-> sigimpl: enum).
#   - every Finding MUST carry source / confidence / assurance / a >=12-char
#     justification — the SHACL FindingShape rejects anything that does not.

"@context":
  "@vocab": "https://sparq.dev/ns/pkg#"
  pkg:      "https://sparq.dev/ns/pkg#"
  kb:       "https://sparq.dev/ns/pkg/kb#"
  prov:     "http://www.w3.org/ns/prov#"
  skos:     "http://www.w3.org/2004/02/skos/core#"
  dcterms:  "http://purl.org/dc/terms/"
  fabio:    "http://purl.org/spar/fabio/"
  cito:     "http://purl.org/spar/cito/"
  sigimpl:  "https://w3id.org/zkp-sparql/sig-impl#"
  secx:     "https://w3id.org/zkp-sparql/sec-prop#"
  rdfs:     "http://www.w3.org/2000/01/rdf-schema#"
  xsd:      "http://www.w3.org/2001/XMLSchema#"
  # Author-facing fluent keys -> FO predicates (no IRIs/xsd in the @graph below).
  finding:        { "@id": "rdfs:label",            "@language": "en-GB" }
  prefLabel:      { "@id": "skos:prefLabel",        "@language": "en-GB" }
  definition:     { "@id": "skos:definition",       "@language": "en-GB" }
  inScheme:       { "@id": "skos:inScheme",         "@type": "@id" }
  about:          { "@id": "pkg:about",             "@type": "@token" }   # concept TOKEN
  subject:        { "@id": "dcterms:subject",       "@type": "@token" }   # concept TOKEN
  source:         { "@id": "prov:wasDerivedFrom",   "@type": "@id" }
  evidence:       { "@id": "cito:citesAsEvidence",  "@type": "@id" }
  fromSection:    { "@id": "dcterms:source" }
  verdict:        { "@id": "pkg:verdict",           "@type": "@verdict" }
  justification:  { "@id": "sigimpl:justification", "@language": "en-GB" }
  confidence:     { "@id": "pkg:confidence",        "@type": "xsd:decimal" }
  assurance:      { "@id": "pkg:assurance",         "@type": "@assurance" }
  exploredStatus: { "@id": "pkg:exploredStatus",    "@type": "@id" }
  followUpPriority: { "@id": "pkg:followUpPriority", "@type": "xsd:integer" }
  identifier:     { "@id": "dcterms:identifier" }
  format:         { "@id": "dcterms:format" }
  title:          { "@id": "dcterms:title",         "@language": "en-GB" }

# The concept catalog the guarded V() resolver grounds TOKENS against. Each entry is a
# skos:Concept; a TOKEN matches it by local-name (the part after `kb:`) OR by its
# normalised prefLabel. Authoring a TOKEN that matches >=2 of these is a HARD error.
concepts:
  - "@id": kb:topic-merge-discipline
    "@type": skos:Concept
    inScheme: pkg:Topics
    prefLabel: "Merge discipline"
    definition: "The PR + ci-summary gate + review-resolution + arming model that governs how a change lands on main."
  - "@id": kb:topic-subagent-rules
    "@type": skos:Concept
    inScheme: pkg:Topics
    prefLabel: "Sub-agent standing rules"
    definition: "The standing rules every dispatched sub-agent follows (the shared contract + worktree isolation + the proceed-without-greenlight rule)."
  - "@id": kb:topic-zk-discipline
    "@type": skos:Concept
    inScheme: pkg:Topics
    prefLabel: "ZK/MPC claim + circuit discipline"
    definition: "The honesty + gate-count discipline specific to the sparq-zk* / sparq-mpc estate."

# The Source node every Finding is derived from (AGENTS.md). A pkg:Source needs an
# explored-status + a >=4-char title (SourceShape); the compiler emits both.
sources:
  - "@id": kb:doc-agents-md
    "@type": [pkg:Source, pkg:Document, fabio:Expression]
    title: "AGENTS.md — the sparq agent charter (how we work)"
    identifier: "AGENTS.md"
    format: "text/markdown"
    exploredStatus: pkg:Explored
    followUpPriority: 0
    confidence: 1.0
    subject: [merge-discipline, subagent-rules]

# The Findings. Each is sourced + confident + assured + non-filler, so each conforms
# to pkg:FindingShape. `about` / `subject` are concept TOKENS (resolved by V()).
"@graph":

  # Q1 — "what's the merge discipline for a ZK PR?"
  - "@id": kb:find-merge-ci-summary-gate
    "@type": pkg:Finding
    finding: "A PR merges only when ci-summary is green and every review thread is resolved"
    about: merge-discipline
    verdict: yes
    justification: "Changes land on main via PRs, not direct push; the single required check is ci-summary (it aggregates every workflow and passes only when all pass); merge only when ci-summary is green AND all review threads — especially Copilot's — are resolved, then squash-merge and delete the branch."
    confidence: 0.98
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#contribution-workflow--prs-reviews-resolved-the-ci-summary-gate"
    subject: merge-discipline

  - "@id": kb:find-zk-circuit-gate
    "@type": pkg:Finding
    finding: "A ZK-circuit change re-runs forge_gates + differential_fuzz + the gate-count snapshot and re-opens the soundness audit"
    about: zk-discipline
    verdict: yes
    justification: "Touching sparq-zk / sparq-zk-compose re-runs forge_gates + differential_fuzz and the gate-count snapshot (gate_count.rs + gate_count_snapshot.json); snapshot_covers_top_level_circuits (Gate G5) and snapshot_covers_every_member fail without a toolchain if a new circuit lacks a baseline; re-baseline via bench/zk-compose/scripts/gate_counts.sh and re-open the soundness audit."
    confidence: 0.95
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#post-batch-re-evaluation-checklist--what-to-re-run-after-a-change"
    subject: [zk-discipline, merge-discipline]

  - "@id": kb:find-zk-member-checklist
    "@type": pkg:Finding
    finding: "Adding/removing a zk/compose member needs the gate-count snapshot updated and the Rust gate run, not just nargo test"
    about: zk-discipline
    verdict: yes
    justification: "A zk/compose member is a bin Noir package using compose_core; on add/remove you must (1) update gate_count_snapshot.json's members map with the real circuit_size from bb gates and re-baseline, and (2) run cargo nextest run -p sparq-zk-compose (NOT just nargo test) — the membership-and-count gate lives in the Rust suite, so nargo test passing is not evidence the gate passes."
    confidence: 0.95
    assurance: proven
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#zk-circuit-member-checklist--adding-or-removing-a-zkcompose-member"
    subject: zk-discipline

  - "@id": kb:find-zk-privacy-claims-gate
    "@type": pkg:Finding
    finding: "Any ZK/MPC privacy or soundness claim must stay hedged/negative — the privacy-claims gate is a HARD ci-summary job"
    about: zk-discipline
    verdict: yes
    justification: "The v1 ZK verifier is pending external accredited-cryptographer sign-off (sq-qhy4) and sparq-mpc is honest-majority semi-honest only, so no doc/README/SKILL/site line may state a ZK/MPC privacy-or-soundness property as settled fact without the not-yet-sound caveat; scripts/check-privacy-claims.sh (the HARD privacy-claims job in docs-quality.yml, gated by ci-summary) fails the build on an unhedged hit unless the line carries privacy-claims-allow: <why>."
    confidence: 0.97
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#post-batch-re-evaluation-checklist--what-to-re-run-after-a-change"
    subject: [zk-discipline, merge-discipline]

  - "@id": kb:find-zk-arming-model
    "@type": pkg:Finding
    finding: "A verified-clean non-perf PR auto-arms; never arm auto-merge on a stacked PR whose base is not main"
    about: merge-discipline
    verdict: yes
    justification: "Honesty/correctness/scope are automated by adversarial-verify + ci-summary, so a verified-clean, non-perf-affecting PR auto-arms; a perf-affecting PR gets the sparq-perf-reviewer PreToolUse gate before arming; and NEVER arm auto-merge on a stacked PR (base != main) — GitHub squash-merges it into the stacked base and its content never reaches main (enforced by scripts/check-pr-arm-base.py)."
    confidence: 0.93
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#contribution-workflow--prs-reviews-resolved-the-ci-summary-gate"
    subject: merge-discipline

  # Q2 — "which standing rules apply to sub-agents?"
  - "@id": kb:find-subagent-shared-contract
    "@type": pkg:Finding
    finding: "Every dispatched sub-agent follows the shared contract; task briefs state the task only"
    about: subagent-rules
    verdict: yes
    justification: "The sub-agent shared contract is the single source of truth for the standing rules every dispatched sub-agent follows; the six role-agent system prompts carry it, so a dispatcher does not repeat it — a task brief states the task only (bead, target crate/surface, task-specific constraint) and otherwise says follow the shared contract."
    confidence: 0.97
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#the-sub-agent-shared-contract--write-terse-task-only-briefs"
    subject: subagent-rules

  - "@id": kb:find-subagent-worktree-isolation
    "@type": pkg:Finding
    finding: "Every mutating sub-agent runs in its OWN isolated worktree+branch, never the shared main checkout"
    about: subagent-rules
    verdict: yes
    justification: "Any sub-agent that writes files, runs git checkout -b, or commits MUST work in an isolated git worktree (isolation: worktree) and MUST NOT branch-switch the shared /home/ubuntu/sparq checkout — two mutating agents on one checkout race (observed 2026-06-20: two agents lost uncommitted work); read-only search/analysis/review agents may share the main checkout for a warm cache."
    confidence: 0.96
    assurance: proven
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#worktree-isolation--every-mutating-agent-gets-its-own-worktreebranch"
    subject: subagent-rules

  - "@id": kb:find-subagent-staging
    "@type": pkg:Finding
    finding: "Sub-agents stage only changed files by explicit path; never git add -A and never stage .beads/"
    about: subagent-rules
    verdict: yes
    justification: "Stage ONLY the files you change, by explicit path; NEVER git add -A and NEVER stage .beads/ — beads re-export is the orchestrator's job on its own chore-beads-resync branch (it conflicts at merge if folded into a feature branch); revert any beads churn that appears in your tree."
    confidence: 0.96
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#the-sub-agent-shared-contract--write-terse-task-only-briefs"
    subject: subagent-rules

  - "@id": kb:find-subagent-no-push-selfid
    "@type": pkg:Finding
    finding: "Sub-agents commit in-worktree (no push/merge), gate in-worktree without weakening a gate, and self-identify in every comment"
    about: subagent-rules
    verdict: yes
    justification: "A sub-agent commits in-worktree and reports — the orchestrator pushes, opens the PR, and merges (exception: sparq-merge-fixer); it gates in-worktree scoped to affected crates and NEVER disables/regex-weakens/blanket-excludes a gate to go green; and it begins every issue/PR/comment with the SPARQ-agent self-identification blockquote."
    confidence: 0.95
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#the-sub-agent-shared-contract--write-terse-task-only-briefs"
    subject: subagent-rules

  - "@id": kb:find-subagent-proceed-no-greenlight
    "@type": pkg:Finding
    finding: "Sub-agents proceed without the maintainer's greenlight, documenting the default and filing a feedback issue"
    about: subagent-rules
    verdict: yes
    justification: "The standing rule binds the orchestrator, every sub-agent, and the autonomous-scheduler: never stall waiting for a greenlight — proceed on a design/decision-blocked bead with the best-judgment default, document it (PR body / bead / issue), and open a feedback issue; the only valid blocker is an external credential an agent literally cannot obtain, and this never overrides honesty/soundness discipline."
    confidence: 0.94
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#standing-rule--proceed-without-waiting-for-the-maintainers-greenlight-2026-06-21"
    subject: subagent-rules

  - "@id": kb:find-subagent-provenance-honesty
    "@type": pkg:Finding
    finding: "Sub-agents tag [OPUS-4.8] + the Opus trailer and keep ZK/MPC + perf claims gate-safe"
    about: subagent-rules
    verdict: yes
    justification: "Add the [OPUS-4.8] inline marker on new code/notes plus the Co-Authored-By: Claude Opus 4.8 (1M context) commit trailer; keep mentions gate-safe — the privacy-claims gate is live (never an unqualified ZK/MPC soundness claim), no hard-coded performance numbers in markdown, and the typos gate flags ordinary doc words to reword."
    confidence: 0.93
    assurance: claimed
    source: kb:doc-agents-md
    evidence: kb:doc-agents-md
    fromSection: "AGENTS.md#the-sub-agent-shared-contract--write-terse-task-only-briefs"
    subject: [subagent-rules, zk-discipline]
