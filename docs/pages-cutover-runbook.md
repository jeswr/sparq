<!-- [OPUS-4.8] sq-iigf — GitHub Pages cutover runbook + rollback. Docs only. -->

# GitHub Pages cutover runbook (benchmark-data branch → Actions producer)

This is an operational runbook for the **one repo-owner action** that flips
`jeswr/sparq` from serving GitHub Pages off a branch to serving it off a GitHub
Actions producer workflow — and, critically, the ordering and rollback that keep
the **live benchmark dashboard from going dark** during that switch.

It does **not** build the producer workflow. The producer is tracked separately
(see [Producer prerequisites](#producer-prerequisites-what-the-actions-workflow-must-satisfy)
below); this runbook only pins the *safe sequence* and the *one-step rollback* so
the owner cannot accidentally brick the dashboard on the first Actions deploy.

## TL;DR

- Pages is **half-switched**: `build_type` is already `workflow`, but there is
  **no Actions producer workflow** in the repo yet. The dashboard you see today
  is a **frozen snapshot** served from the `benchmark-data` branch — still up, no
  longer updated.
- **Do NOT let any Actions Pages deploy run** until the unified producer is landed
  and green. The first `deploy-pages` run replaces the live dashboard with
  whatever that workflow publishes — which, with no producer, is nothing.
- The **recommended interim state** is to revert Pages **Source** back to the
  `benchmark-data` branch (one click, or one `gh api` call). That restores the
  live, *updating* dashboard with zero downtime and keeps it live until the
  producer is ready.

## Current state (verified)

Confirmed against the live API and the in-repo workflows on 2026-06-15.

### Pages is configured for Actions, but no producer exists

`gh api repos/jeswr/sparq/pages` reports:

- `build_type: "workflow"` — the **Source** has already been switched to
  *GitHub Actions* in **Settings → Pages**.
- `source.branch: "benchmark-data"`, `source.path: "/"` — the previous
  branch-source is still recorded (this is what a one-step rollback restores).
- `status: "built"`, `html_url: https://jeswr.github.io/sparq/`.

But there is **no Actions Pages producer workflow** in `.github/workflows/`. There
is no `docs.yml`, and nothing in the tree calls `actions/deploy-pages`. So the
"workflow" source has **no producer** behind it. The build type is set; the thing
that would build is missing.

### The dashboard is served *frozen* from the `benchmark-data` branch

The benchmark dashboard is published by
[`.github/workflows/bench.yml`](../.github/workflows/bench.yml) and
[`.github/workflows/bench-ec2.yml`](../.github/workflows/bench-ec2.yml). Both use
`benchmark-action/github-action-benchmark` with
`gh-pages-branch: benchmark-data` — i.e. they push results to the
**`benchmark-data` branch**, which branch-Pages historically served. The
switch to `build_type: workflow` means Pages **no longer serves that branch**, so
the dashboard is now a **frozen snapshot**: the URL still returns `200`, but new
benchmark commits to `benchmark-data` are no longer published.

Live probe (2026-06-15):

| URL | Status | Meaning |
|---|---|---|
| `https://jeswr.github.io/sparq/dev/bench/` | `200` | frozen snapshot — still up, no longer updated |
| `https://jeswr.github.io/sparq/dev/bench-ec2/` | `404` | EC2 series dir not yet on the served snapshot |
| `https://jeswr.github.io/sparq/` | `404` | no site root (the dashboard lives under `dev/bench`) |

### Two benchmark series exist — the EC2 one is easy to miss

There are **two** dashboard directories on `benchmark-data`, written by two
different workflows:

- `dev/bench` — the per-commit CI series, written by `bench.yml`
  (`benchmark-data-dir-path: dev/bench`). This is the one currently served at
  `200`.
- `dev/bench-ec2` — the **heavy EC2 series**, written by `bench-ec2.yml`
  (`benchmark-data-dir-path: dev/bench-ec2`). This is the easy-to-miss one. Note
  `bench-ec2.yml` is **not live**: it authenticates through an AWS OIDC role
  (`vars.AWS_BENCH_ROLE_ARN`) that was deliberately descoped, and its crons were
  **retired** in [#3784](https://github.com/sparq-org/sparq/issues/3784) because
  each scheduled tick failed at the credentials step and — carrying no advisory
  declaration — gated `main`. It is now **manual dispatch only** (pick `lane:
  heavy` or `lane: full-suite`), so `dev/bench-ec2` may be empty or absent on the
  served snapshot today — hence its `404` above. It is still a declared producer of
  a **second** Pages directory, so any unified producer must account for it or the
  EC2 dashboard `404`s after cutover.

### Chart.js is loaded from a CDN

`bench/dashboard/index.html` loads Chart.js from the jsDelivr CDN
(`<script src="https://cdn.jsdelivr.net/npm/chart.js@.../dist/Chart.min.js">`).
This is fine for the branch-served snapshot, but for an Actions-built site the
**recommendation** is to vendor Chart.js locally rather than pull it from a CDN at
page load — for **supply-chain hygiene and availability** (the dashboard keeps
working offline / if the CDN is down, and the bytes are reviewable and
reproducible). This is a recommendation, not a check the repo's tooling enforces:
the repo's Scorecard posture covers pinning **GitHub Actions / Docker
dependencies** (`PinnedDependencies`) and **workflow token scopes**
(`TokenPermissions`) — it does not analyse runtime JS loaded by the Pages site, so
no existing gate would flag a CDN `<script>`. Vendoring is good hygiene regardless.

## Safe cutover sequence

The cutover is **single-source and mutually exclusive**: a GitHub repo has exactly
one Pages source. The instant the Actions producer's first `deploy-pages` runs, it
**replaces** whatever the branch source was serving. So the rule is:

> **Land and green the unified producer FIRST. Only THEN allow an Actions Pages
> deploy to run.**

Do not toggle anything, dispatch the producer, or let it run on a `main` push
until **all** of the prerequisites below are satisfied and verified on a real
producer run's artifact.

### Producer prerequisites: what the Actions workflow must satisfy

These are the acceptance criteria the producer (tracked as
beads `sq-h0tr` and `sq-zngy`; parent design `sq-w9sr`) **must** meet before its
first deploy is allowed to publish. This runbook *specifies* them; it does not
build them.

1. **Publish the mdBook docs site.** The unified producer builds the mdBook guide
   into the Pages artifact. **The mount is decided
   ([#5022](https://github.com/jeswr/sparq/issues/5022) — superseding this item's
   original "guide at the site root" wording):** the guide is published at
   the **`/guide/` sub-path**, not the root — `pages.yml` builds it via
   `scripts/build-guide.sh` and overlays the render at `out/guide/`, exactly as it
   already overlays `dev/bench*` and `/app`. The root is the Next.js showcase and
   does not move. See
   [`research/docs-site-single-sourcing-anti-drift.md`](../research/docs-site-single-sourcing-anti-drift.md)
   §7 option (a) for why guide-at-root and a second deploy workflow were rejected.
   Self-hosting `cargo doc` output was also dropped there in favour of linking
   docs.rs, so it is not a producer prerequisite.
2. **Publish BOTH benchmark series.** The artifact must contain **both**
   `dev/bench` **and** `dev/bench-ec2` (glob `dev/bench*`). Folding only
   `dev/bench` would make the EC2 dashboard `404` after cutover — exactly the
   reconciliation gap called out in `sq-zngy`.
3. **Read existing `benchmark-data` history.** The producer must `git checkout`
   (or otherwise read) the `benchmark-data` branch and copy the existing
   `dev/bench*` subtrees into the artifact, so the historical series is **not
   lost** at cutover. The benchmark series lives on that branch, not in `main`;
   the producer that builds the Actions artifact has to bring it across.
4. **Vendor Chart.js off the CDN (recommended).** The dashboard's Chart.js
   dependency should be vendored locally and referenced with a relative path,
   rather than loaded from a CDN at page load — for supply-chain hygiene and
   availability (offline / CDN-outage resilience, reviewable reproducible bytes).
   This is a recommendation, not a tooling-enforced gate: the repo's Scorecard
   posture pins Actions/Docker dependencies and workflow token scopes, not runtime
   JS served by Pages (see [Chart.js is loaded from a CDN](#chartjs-is-loaded-from-a-cdn)).
5. **Post-deploy smoke check.** A producer step (or a follow-up check) must assert
   the deployed site is intact: `dev/bench/{index.html, data.js (non-empty),
   dashboard.js}` **and** `dev/bench-ec2/index.html` all return `200` (per
   `sq-zngy`). This catches exactly the reconciliation regression the early
   source-flip created.

### Ordering checklist

1. **Keep the dashboard live in the interim** — if it is currently frozen and the
   producer is not ready, apply the [rollback](#rollback-restore-the-live-dashboard-in-one-step)
   now so the dashboard keeps updating while the producer is built. This is the
   recommended state until step 4.
2. **Land the producer** (`sq-h0tr` + `sq-zngy`) on `main` and confirm CI is
   green, with all five prerequisites above satisfied.
3. **Verify the producer artifact** on a run that builds but does **not** deploy
   (or inspect the uploaded artifact of a dry run): confirm it contains the guide,
   `dev/bench`, `dev/bench-ec2`, vendored Chart.js (no CDN reference), and the
   imported `benchmark-data` history.
4. **Flip the source to Actions** — only now. If a rollback (below) was applied,
   set Pages **Source** back to *GitHub Actions* so the producer's `deploy-pages`
   becomes the live source. (If `build_type` is already `workflow`, this step is a
   no-op and you proceed straight to letting the producer deploy.)
5. **Let the producer deploy and run the smoke check.** Trigger the producer (push
   to `main` or `workflow_dispatch`) and confirm the post-deploy smoke check
   passes — guide root `200`, both dashboard dirs `200`.

The owner-side toggle itself (flipping **Source** to *GitHub Actions*) is tracked
as the `needs:user` bead `sq-vbq9`; do not perform it before step 4.

## Rollback: restore the live dashboard in one step

If an Actions deploy has bricked the dashboard (blank / `404`), or you simply want
the **live, updating** dashboard back while the producer is still being built,
revert the Pages **Source** to the `benchmark-data` branch. This is a single
owner action.

**Via the GitHub UI:** Settings → Pages → "Build and deployment" → **Source =
"Deploy from a branch"**, Branch = **`benchmark-data` / `/ (root)`** → Save.

**Via the API** (equivalent, scriptable):

```sh
gh api -X PUT repos/jeswr/sparq/pages \
  -f 'build_type=legacy' \
  -f 'source[branch]=benchmark-data' \
  -f 'source[path]=/'
```

Within a minute or so, `https://jeswr.github.io/sparq/dev/bench/` is served from
the branch again — and, unlike the frozen snapshot, it **resumes updating** as
`bench.yml` pushes new points to `benchmark-data`. (`dev/bench-ec2` will appear
once `bench-ec2.yml` is live and has pushed at least one point — which now requires
both re-provisioning `AWS_BENCH_ROLE_ARN` and a manual dispatch, since #3784 retired
its crons.)

This rollback is the **recommended interim state** until the unified producer is
ready: it is the only configuration in which the dashboard both serves *and*
updates, and it costs nothing to maintain.

## Decision note for the owner

There are two viable states right now. The producer is not yet landed, so this is
a real choice, not a transient.

- **Option A — Revert now (recommended).** Apply the
  [rollback](#rollback-restore-the-live-dashboard-in-one-step) immediately. The
  dashboard goes from *frozen* back to *live and updating* with **zero downtime**,
  and stays that way with no further attention until the producer is ready. The
  cost is that the Actions-source switch is deferred — but nothing depends on that
  switch until the docs site exists, which it does not yet. This keeps the most
  visible artifact (the public perf trend) healthy while the producer is built.
- **Option B — Stay on Actions, accept a frozen dashboard.** Leave
  `build_type: workflow` as-is and accept that the dashboard is a **frozen
  snapshot** (no new points published) until the unified producer lands. This
  avoids a back-and-forth toggle, but the public dashboard silently stops
  reflecting reality in the meantime — and any *accidental* Actions Pages run
  (a stray `workflow_dispatch`, an early producer attempt) before the producer is
  complete would turn the frozen-but-present dashboard into a blank / `404`.

**Recommendation:** **Option A.** Reverting now is a single, reversible action
with zero downtime and no maintenance, and it removes the brick risk entirely
(there is no Actions producer that could overwrite the served branch while the
source points at `benchmark-data`). Switch to Actions only once the producer
satisfies every prerequisite in
[Producer prerequisites](#producer-prerequisites-what-the-actions-workflow-must-satisfy)
and its artifact has been verified.

## See also

- [`.github/workflows/bench.yml`](../.github/workflows/bench.yml) — the per-commit
  benchmark producer (writes `dev/bench` on `benchmark-data`).
- [`.github/workflows/bench-ec2.yml`](../.github/workflows/bench-ec2.yml) — the
  heavy EC2 benchmark producer (writes `dev/bench-ec2`).
- [`docs/branch-protection.md`](branch-protection.md) — the required-checks /
  branch-protection record this repo enforces.
