<!-- [OPUS-5] sq-iigf — GitHub Pages cutover record. Docs only.
     Rewritten for #5530: the original runbook described a pre-cutover state that no
     longer exists, and recommended a rollback that would now take the live site down. -->

# GitHub Pages: cutover record

> **This is no longer an actionable runbook.** The cutover it sequenced — moving
> `sparq` from a branch-served Pages site to a GitHub Actions producer — is **done on
> the repo side**: the producer exists and every prerequisite the original document
> listed is either implemented or tracked below. The matching **Pages service
> setting** is reported as flipped but is not verified by this document; see
> [Provenance](#provenance-of-the-claims-below).
>
> It is kept as a record of the resulting topology and, more importantly, as a
> **warning**: the rollback the original document recommended would now take the
> live site offline. See [Do not "roll back"](#do-not-roll-back).

## Provenance of the claims below

Verified on 2026-08-01 by reading the **in-repo workflows** at commit `83b592c2` —
**not** by probing the live site or the Pages API. The file tree can only tell you
what the producer *would* publish, not what the Pages service is currently serving.

So read the two kinds of claim below differently:

- **Artifact shape** (what `pages.yml` builds, what it overlays, what its smoke check
  asserts) is established by the workflow source, and is as reliable as the file tree.
- **Pages *service settings*** (the source mode, the custom domain the service has
  registered, which producer actually owns the live deploy) is **not** established
  here. Where this document reports them it is relaying an **undated** comment in
  [`pages.yml`](../.github/workflows/pages.yml) (its `PAGES SOURCE (DONE …)` header,
  which records a `gh api` result without a date), not a probe run for this document.

**Before acting on anything in the settings class, confirm it yourself:**

```bash
gh api repos/sparq-org/sparq/pages --jq '{build_type, cname, html_url}'
curl -sSI https://sparq.jeswr.org/ | head -1
```

Note the repo also moved orgs: it is **`sparq-org/sparq`**, not `jeswr/sparq`. Every <!-- terminology-allow: names the OLD owner in order to contrast it with the new one -->
API call and URL in the original document targeted the old owner and the old
`github.io` host, and would misfire today.

## What the repo is configured to publish

Established by the workflow source (artifact class):

- **The only Actions Pages producer is
  [`.github/workflows/pages.yml`](../.github/workflows/pages.yml).** It builds the
  Next.js feature-showcase, overlays the benchmark dashboards and the operational GUI,
  runs an artifact smoke check, and deploys via `actions/deploy-pages`. (The original
  document's central claim — that `.github/workflows/` contained *no* Actions Pages
  producer — has not been true for a long time.)
- **The artifact is built for the custom domain `https://sparq.jeswr.org/`**,
  root-relative (`basePath ''`): [`site/public/CNAME`](../site/public/CNAME) is copied
  to `out/CNAME` and the smoke step fails the run if it is missing or wrong. The old
  `https://jeswr.github.io/sparq/...` URLs are historical.
- **No other workflow ships a deploy job**, so within the repo nothing competes with
  `pages.yml` for Pages' single deploy slot.

Relayed, **not** verified here (settings class — confirm with the commands above
before acting):

- `pages.yml`'s header comment states Pages source is `build_type: workflow` and the
  one-time owner flip (`sq-vbq9`) is resolved, which would make its deploy step the
  live publisher. That comment is undated and this document did not re-check it. If
  the source mode were *not* `workflow`, `pages.yml`'s deploy step would be failing
  and the live site would still be served from a branch — a state that changes which
  advice below applies.

The single artifact assembles three things:

| Path | Contents | Source |
|---|---|---|
| `/` | Next.js feature-showcase + papers | `site/`, built in `pages.yml` |
| `/dev/bench*/` | benchmark dashboards | overlaid from the `benchmark-data` branch |
| `/app/` | operational GUI workbench | `gui/app`, built in `pages.yml` |

### The dashboards are folded in, not served from a branch

`bench.yml` and `bench-ec2.yml` still push results to the **`benchmark-data`** branch
— that has not changed. What changed is that Pages no longer *serves* that branch:
`pages.yml` **reads** it (`git archive origin/benchmark-data dev`) and overlays the
whole `dev/` tree into the artifact.

The fold is by **glob (`dev/bench*`)**, not an enumerated list, so every series is
carried automatically. There are now **three**: `dev/bench` (per-commit CI, from
`bench.yml`), `dev/bench-ec2` and `dev/bench-nightly` (both from `bench-ec2.yml`).
A build-time assertion fails the run if any `dev/bench*` subtree present on the
branch is missing from the artifact, so narrowing that scope cannot regress silently.

`bench-ec2.yml` remains **manual-dispatch only** — its OIDC role (`AWS_BENCH_ROLE_ARN`)
was descoped and its crons retired in
[#3784](https://github.com/sparq-org/sparq/issues/3784) — so its directories may not
exist on the branch. The smoke check asserts `dev/bench-ec2` is in the artifact **iff**
it exists on the branch, rather than hard-failing every deploy on its absence.

### Chart.js is vendored

The original document listed "vendor Chart.js off the CDN" as an outstanding
recommendation. It is **done**: `bench/dashboard/vendor/Chart.min.js` is tracked,
`bench/dashboard/index.html` loads it by relative path, and
[`scripts/check-dashboard-publish-wiring.py`](../scripts/check-dashboard-publish-wiring.py)
enumerates it in the seed list so it cannot fall out of the published set.
Provenance is recorded in [`bench/dashboard/vendor/README.md`](../bench/dashboard/vendor/README.md).

## Do not "roll back"

The original document recommended, as its headline action, reverting the Pages
**Source** to the `benchmark-data` branch. **That recommendation is now inverted.**
Applying it today would point Pages at a branch that contains only a root redirect
and the `dev/` dashboard tree, so:

- the feature-showcase root, the papers and every `/surface/*` route would `404`;
- the operational GUI at `/app/` would `404`;
- only the dashboards under `/dev/bench*/` would survive.

There is no longer a scenario in which that is the recovery action. This holds
whichever source mode the service is actually in: if it is already `workflow`, the
flip breaks the live site; if it is not, the flip is not a recovery either, because
the branch has not been a complete site since `pages.yml` took over the root.

If a deploy publishes a bad artifact, the fix is to **fix forward or re-run
`pages.yml` from a good commit** — a re-run republishes the whole tree. That
presumes the deploy step is in fact the live publisher; if a re-run visibly changes
nothing, check the source mode (above) before assuming the workflow is at fault.

## Where the invariants live now

The prerequisites the original document *specified* in prose are now **executable
assertions inside the producer**, which is the right place for them. If you are
tempted to re-specify them here, edit the workflow instead:

| Invariant | Enforced by |
|---|---|
| Custom domain is set on every deploy | `out/CNAME` check in `pages.yml`'s smoke step |
| Every `dev/bench*` series is folded in | glob-fold assertion in the overlay step |
| Core dashboard files present, `data.js` non-empty | smoke step |
| GUI lands at `/app/` with the right asset prefix | overlay assertion in `pages.yml` |
| Dashboard assets stay wired into the published set | `scripts/check-dashboard-publish-wiring.py` |
| Built site has no broken internal links | `scripts/check-site-links.sh` (lychee, pre-deploy) |

## The one thing still open

**Where the mdBook guide is published.** [`.github/workflows/docs.yml`](../.github/workflows/docs.yml)
builds and validates the guide but deliberately ships **no deploy job** — a second
`actions/deploy-pages` would race `pages.yml` for the single slot and last-writer-wins
over the showcase root. The `sq-w9sr` design placed the guide at the site root, which
predates the showcase and is no longer viable. Reconciling that is tracked as the open
product decision `sq-svtt` (guide at a `/guide/` sub-path vs. at the root vs. a merged
deploy); mounting it at `/guide/` on the same artifact is implemented by
[#5022](https://github.com/sparq-org/sparq/issues/5022), which had not landed as of the
commit above — `pages.yml` overlays no `guide/` tree.

Until that lands, the guide is a CI artifact only, not a published page. This is the
sole remaining item from the original prerequisite list.

## See also

- [`.github/workflows/pages.yml`](../.github/workflows/pages.yml) — the live producer.
- [`.github/workflows/docs.yml`](../.github/workflows/docs.yml) — mdBook build/validate (no deploy).
- [`.github/workflows/bench.yml`](../.github/workflows/bench.yml) — writes `dev/bench` on `benchmark-data`.
- [`.github/workflows/bench-ec2.yml`](../.github/workflows/bench-ec2.yml) — writes `dev/bench-ec2` and `dev/bench-nightly`.
- [`docs/branch-protection.md`](branch-protection.md) — the required-checks record.
