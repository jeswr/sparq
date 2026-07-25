// Autonomous scheduler (epic sq-sgu1) — the self-driving bead-frontier loop.
//
// PURPOSE: keep the orchestrator OUT of the per-agent dispatch loop. Each wave it reads the
// launchable-bead frontier (scripts/push-frontier.sh), picks the dispatchable ready beads,
// and runs implement -> adversarially-verify -> arm-if-clean for each, then re-reads the
// frontier (newly-unblocked beads appear) and repeats until the frontier is dry or the
// per-run cap / token budget is reached.
//
// DURABLE: this file is committed to .claude/workflows/ and linked from AGENTS.md (the
// Maintenance-loop step "Drive bd ready" + the script catalog entry for this file) so it is
// NOT lost on a session restart. Re-run it any time with:
//   Workflow({ name: "autonomous-scheduler" })            // resolves this file by name
// or override the per-run cap / focus list via args:
//   Workflow({ name: "autonomous-scheduler", args: { maxBeads: 6, only: ["sq-...", "sq-..."] } })
//
// The recurring per-agent briefs below DELEGATE their judgment rule to a committed SKILL
// rather than re-stating it inline (so a hand-dispatched agent inherits the same rule):
// the proceed-on-decision standing rule is the `proceed-and-document` skill
// (.claude/skills/proceed-and-document/SKILL.md). Durability contract for any such durable
// workflow/skill: committed under .claude/ + linked from AGENTS.md + a one-line description.
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
    // [OPUS-4.8] sq-4vo9m: per-tick disk guard. The frontier agent runs
    // `scripts/disk-guard.sh --apply` BEFORE reading the frontier (its safe worktree prune
    // is harmless when disk is fine and frees the most when it is not) and reports the
    // resulting disk state so the scheduler can back off dispatch under disk pressure.
    disk_state: { type: 'string', enum: ['OK', 'WARN', 'CRITICAL', 'UNKNOWN'] },
    disk_free_gb: { type: 'number' },
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
    'FIRST decide if this is genuinely implementable autonomously: if it is an EPIC, is already done on main, or needs an EXTERNAL credential/access you cannot obtain (OS signing certs, npm/PyPI tokens, an external auditor), do NOT implement — return {bead, skipped:true, reason}. If it merely needs a DESIGN/JUDGMENT decision or is somewhat underspecified, PROCEED anyway per the `proceed-and-document` skill (.claude/skills/proceed-and-document/SKILL.md): make the best-judgment choice, DOCUMENT it in the PR body + a one-line bead note, and open a short GitHub issue (🤖 SPARQ self-id) flagging the decision for the maintainer to steer later. (STANDING RULE — do not block on the maintainer greenlight; the skill is the single source of this procedure, AGENTS.md "STANDING RULE …" is its authority.) ' +
    'Otherwise implement per the bead + AGENTS.md conventions: new capability => opt-in/feature-gated (keep sparq-core/engine lean); gate BOTH feature states (clippy --all-targets --all-features -D warnings + tests in default AND feature-on), rustdoc --all-features -D warnings, cargo fmt, check-readme-template.py if you touch a crate README, check-privacy-claims.sh clean, no hard-coded perf numbers. Tag the RUNNING model\'s marker + Co-Authored-By trailer (canonical per-tier table: .claude/workflows/fable-architect-drain.js — Opus 5 primary; a downgraded session\'s marker flags the work for re-review under Opus 5). ' +
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
    // [OPUS-4.8] sq-4vo9m: DISK GUARD each tick — run BEFORE reading the frontier so we prune
    // merged worktrees and measure free space every wave (the autonomous loop's per-agent
    // target/ dirs re-fill the disk; an unguarded loop hit 99%/ENOSPC and crashed a verify
    // build). The guard's worktree prune is SAFE (it keeps anything unmerged/dirty), so --apply
    // it unconditionally; the main-checkout target/ reclaim is opt-in + CRITICAL-only + gated on
    // no live build, so passing --reclaim-main-target is safe (it only fires when truly starved).
    'FIRST run the per-tick disk guard (READ-ONLY repo; do NOT git-checkout): ' +
    '`cd /home/ubuntu/sparq && bash scripts/disk-guard.sh --apply --reclaim-main-target`. ' +
    'Capture its final `[disk-guard] done (state=..., advisory exit=N)` line: report disk_state ' +
    '(OK/WARN/CRITICAL/UNKNOWN) and disk_free_gb (the "Xg free" it printed, as a number). ' +
    'Then read the launchable-bead frontier: `export PATH=$PATH:/home/ubuntu/.local/bin && cd /home/ubuntu/sparq && bash scripts/push-frontier.sh` (READ-ONLY; do NOT git-checkout). ' +
    'Each line is `bead-id  surface  title`. Return up to 6 DISPATCHABLE ready beads as {beads:[{id,surface,title}]}. ' +
    'EXCLUDE: epics; in-progress; and beads needing an EXTERNAL credential/access an agent cannot obtain (OS signing certs, npm/PyPI publish tokens, the external cryptographer audit); and these already-handled ids: ' + JSON.stringify([...seen]) + '. ' +
    'INCLUDE beads that merely need a DESIGN/JUDGMENT decision — the impl agent decides with best judgment + documents it per the `proceed-and-document` skill (.claude/skills/proceed-and-document/SKILL.md): proceed without the maintainer greenlight, document the choice, steer post-hoc. ' +
    // [OPUS-4.8] sq-7mwun: belt-and-suspenders open-PR exclusion. push-frontier.sh already
    // subtracts in-flight beads, but it matches a bead id as a SUBSTRING of an open-PR
    // branch/title, which over/under-matches dotted CHILD ids (sq-ixc3.11 vs sq-ixc3.12) and
    // misses worktree-named branches (worktree-wf_...) whose PR title omits the exact id token.
    // Those leaks let already-in-flight beads (and already-merged-but-bd-still-open ones) get
    // re-picked + re-skipped every wave, wasting agent slots. So the frontier agent ALSO does an
    // explicit, EXACT per-candidate open-PR check before returning each bead.
    'For EACH candidate bead the frontier script emitted, run `gh pr list --search "<bead-id>" --state open --json number,title,headRefName` and DROP the bead if any returned PR matches it (its EXACT id token appears in a PR title or head-branch name) — that work is already in flight, so re-dispatching it would re-implement in-flight work. Only return beads with NO matching open PR. ' +
    (ONLY ? 'RESTRICT to only these bead ids if present on the frontier: ' + JSON.stringify(ONLY) + '. ' : '') +
    'Prefer well-scoped feature/task/bug beads an implementation agent can finish autonomously. If the frontier has no dispatchable beads, return {beads:[]}.',
    { label: 'frontier', phase: 'Frontier', schema: FRONTIER_SCHEMA, agentType: 'general-purpose' }
  )
  // [OPUS-4.8] sq-4vo9m: disk-pressure backoff. The guard already pruned + (if CRITICAL)
  // reclaimed before this point; we still THROTTLE new dispatch under pressure because each
  // dispatched impl agent allocates a fresh multi-GB target/. WARN → cap the wave to 1 new
  // agent; CRITICAL → dispatch NOTHING this wave (let the just-run prune/reclaim take effect
  // and re-measure next tick) rather than pour more target/ dirs onto a near-full disk.
  const diskState = (fr && fr.disk_state) || 'UNKNOWN'
  if (typeof fr === 'object' && fr) {
    log('disk: state=' + diskState + (fr.disk_free_gb != null ? ' (~' + fr.disk_free_gb + 'G free)' : ''))
  }
  if (diskState === 'CRITICAL') {
    log('disk CRITICAL — skipping dispatch this wave (guard pruned/reclaimed; re-measuring next tick)')
    dry++
    continue
  }
  const diskCap = diskState === 'WARN' ? 1 : 6
  const beads = ((fr && fr.beads) || [])
    .filter(b => b && b.id && !seen.has(b.id))
    .slice(0, Math.min(diskCap, MAX_BEADS - opened.length))
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
