// Autonomous scheduler (epic sq-sgu1) — the self-driving bead-frontier loop.
//
// PURPOSE: keep the orchestrator OUT of the per-agent dispatch loop. Each wave it reads the
// launchable-bead frontier (scripts/push-frontier.sh), picks the dispatchable ready beads,
// and runs implement -> adversarially-verify -> arm-if-clean for each, then re-reads the
// frontier (newly-unblocked beads appear) and repeats until the frontier is dry or the
// per-run cap / token budget is reached.
//
// DURABLE: this file is committed to .claude/workflows/ and linked from AGENTS.md so it is
// NOT lost on a session restart. Re-run it any time with:
//   Workflow({ name: "autonomous-scheduler" })            // resolves this file by name
// or override the per-run cap / focus list via args:
//   Workflow({ name: "autonomous-scheduler", args: { maxBeads: 6, only: ["sq-...", "sq-..."] } })
//
// SAFETY: impl agents run in ISOLATED worktrees (never branch-switch the shared checkout).
// The verify step arms ONLY clean, low-risk PRs; honesty/soundness-sensitive surfaces
// (ZK soundness, security, anything needing a maintainer decision) are left OPEN for review.
// Bounded by maxBeads per run so it cannot flood; epics / needs-user / underspecified beads
// are skipped by the frontier filter and by the impl agent's own skip path.

export const meta = {
  name: 'autonomous-scheduler',
  description: 'Self-driving bead-frontier loop: read push-frontier.sh, implement -> verify -> arm each ready bead in waves until dry or capped. Keeps the orchestrator out of per-agent dispatch (epic sq-sgu1).',
  whenToUse: 'Run each orchestration tick to drain the launchable-bead frontier autonomously instead of hand-dispatching agents.',
  phases: [
    { title: 'Frontier', detail: 'read push-frontier.sh + pick dispatchable ready beads' },
    { title: 'Implement', detail: 'one isolated-worktree agent per bead -> opt-in PR' },
    { title: 'Verify', detail: 'adversarially verify each PR; arm clean low-risk ones' },
  ],
}

const FRONTIER_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    beads: {
      type: 'array',
      items: {
        type: 'object',
        additionalProperties: false,
        properties: {
          id: { type: 'string' },
          surface: { type: 'string' },
          title: { type: 'string' },
        },
        required: ['id', 'surface', 'title'],
      },
    },
  },
  required: ['beads'],
}

const IMPL_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    bead: { type: 'string' },
    pr_url: { type: 'string' },
    gates_green: { type: 'boolean' },
    skipped: { type: 'boolean' },
    reason: { type: 'string' },
  },
  required: ['bead', 'skipped'],
}

const VERDICT_SCHEMA = {
  type: 'object',
  additionalProperties: false,
  properties: {
    armed: { type: 'boolean' },
    honest: { type: 'boolean' },
    concerns: { type: 'array', items: { type: 'string' } },
  },
  required: ['armed', 'honest'],
}

function pickAgent(surface) {
  const s = (surface || '').toLowerCase()
  if (s.includes('site') || s.includes('gui')) return 'sparq-site'
  if (s.includes('ci') || s.includes('infra') || s.includes('release') || s.includes('workflow')) return 'sparq-ci-infra'
  if (s.includes('doc')) return 'sparq-docs'
  if (s.includes('sparq-') || s.includes('engine') || s.includes('serve') || s.includes('crate') || s.includes('zk') || s.includes('mpc') || s.includes('shacl') || s.includes('vectors') || s.includes('reason')) return 'sparq-rust-feature'
  return 'general-purpose'
}

function implPrompt(b) {
  return 'Implement bead ' + b.id + ' ("' + b.title + '", surface: ' + b.surface + '). ' +
    '`export PATH=$PATH:/home/ubuntu/.local/bin && bd show ' + b.id + '`. You are in an ISOLATED git worktree — work there; NEVER git-checkout/branch-switch the shared /home/ubuntu/sparq checkout. ' +
    'FIRST decide if this is genuinely implementable autonomously: if it is an EPIC, needs a MAINTAINER DECISION, is underspecified, or is already done on main, do NOT implement — return {bead, skipped:true, reason}. ' +
    'Otherwise implement per the bead + AGENTS.md conventions: new capability => opt-in/feature-gated (keep sparq-core/engine lean); gate BOTH feature states (clippy --all-targets --all-features -D warnings + tests in default AND feature-on), rustdoc --all-features -D warnings, cargo fmt, check-readme-template.py if you touch a crate README, check-privacy-claims.sh clean, no hard-coded perf numbers. Tag [OPUS-4.8] + Opus trailer. ' +
    'Close the bead on PR-open; do NOT stage .beads/ into the PR. Open ONE PR vs main, auto-merge OFF, with the 🤖 SPARQ-agent self-id blockquote. ' +
    'Return {bead, pr_url, gates_green, skipped:false}. If you hit a weekly limit, STOP and return {bead, skipped:true, reason:"weekly limit"}.'
}

function verifyPrompt(impl, b) {
  return 'Adversarially verify PR ' + impl.pr_url + ' (bead ' + b.id + ', surface ' + b.surface + '). READ-ONLY review: `gh pr diff` + the changed files (do NOT git-checkout the shared tree). ' +
    'Check: (1) HONEST — no overclaim; any ZK/MPC claim must stay "research-grade / not externally audited (sq-qhy4)"; (2) gates green in BOTH feature states; (3) opt-in / core-lean respected; (4) tests are real, not vacuous; (5) the change actually does what the bead asked. ' +
    'If it is CLEAN and LOW-RISK, ARM it: `gh pr merge ' + impl.pr_url + ' --squash --auto`. ' +
    'Do NOT arm (leave OPEN for the maintainer) if: any honesty/soundness concern, a security- or ZK-soundness-sensitive surface, a public-facing irreversible change, or you are not confident. ' +
    'Return {armed, honest, concerns:[...]}.'
}

const MAX_BEADS = (args && args.maxBeads) ? args.maxBeads
  : (typeof budget !== 'undefined' && budget && budget.total) ? Math.max(3, Math.floor(budget.total / 150000))
  : 6
const ONLY = (args && Array.isArray(args.only)) ? args.only : null

const seen = new Set()
const opened = []
let dry = 0

while (opened.length < MAX_BEADS && dry < 2) {
  if (typeof budget !== 'undefined' && budget && budget.total && budget.remaining() < 120000) {
    log('token budget nearly exhausted (' + Math.round(budget.remaining() / 1000) + 'k left) — stopping scheduler')
    break
  }
  phase('Frontier')
  const fr = await agent(
    'Read the launchable-bead frontier: `export PATH=$PATH:/home/ubuntu/.local/bin && cd /home/ubuntu/sparq && bash scripts/push-frontier.sh` (READ-ONLY; do NOT git-checkout). ' +
    'Each line is `bead-id  surface  title`. Return up to 6 DISPATCHABLE ready beads as {beads:[{id,surface,title}]}. ' +
    'EXCLUDE: epics; anything needs-user / requiring a maintainer decision; in-progress; and these already-handled ids: ' + JSON.stringify([...seen]) + '. ' +
    (ONLY ? 'RESTRICT to only these bead ids if present on the frontier: ' + JSON.stringify(ONLY) + '. ' : '') +
    'Prefer well-scoped feature/task/bug beads an implementation agent can finish autonomously. If the frontier has no dispatchable beads, return {beads:[]}.',
    { label: 'frontier', phase: 'Frontier', schema: FRONTIER_SCHEMA, agentType: 'general-purpose' }
  )
  const beads = ((fr && fr.beads) || [])
    .filter(b => b && b.id && !seen.has(b.id))
    .slice(0, Math.min(6, MAX_BEADS - opened.length))
  if (!beads.length) { dry++; log('frontier dry (round ' + dry + ')'); continue }
  dry = 0
  beads.forEach(b => seen.add(b.id))
  log('wave: dispatching ' + beads.map(b => b.id).join(', '))

  const waveResults = await pipeline(
    beads,
    (b) => agent(implPrompt(b), { label: 'impl:' + b.id, phase: 'Implement', schema: IMPL_SCHEMA, agentType: pickAgent(b.surface), isolation: 'worktree' }),
    (impl, b) => {
      if (!impl || impl.skipped || !impl.pr_url) {
        return { bead: b.id, pr: null, armed: false, honest: false, skipped: true, reason: (impl && impl.reason) || 'no PR produced' }
      }
      return agent(verifyPrompt(impl, b), { label: 'verify:' + b.id, phase: 'Verify', schema: VERDICT_SCHEMA, agentType: 'general-purpose' })
        .then(v => ({ bead: b.id, pr: impl.pr_url, armed: (v && v.armed) || false, honest: (v && v.honest) || false, concerns: (v && v.concerns) || [] }))
    }
  )
  opened.push(...waveResults.filter(Boolean))
  log('wave complete — PRs so far: ' + opened.length)
}

return {
  runs: opened.length,
  armed: opened.filter(r => r && r.armed).map(r => r.bead),
  open_for_review: opened.filter(r => r && r.pr && !r.armed).map(r => ({ bead: r.bead, pr: r.pr, concerns: r.concerns })),
  skipped: opened.filter(r => r && r.skipped).map(r => ({ bead: r.bead, reason: r.reason })),
}
